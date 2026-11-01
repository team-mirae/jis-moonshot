//! Kanji conversion (docs/moonshot.md §5.1): the converter on Mozc's evaluation set, its data
//! never panicking, and how conversions join the predictions (the top-n gate, F8, Shift+F8).
#![cfg(feature = "predict")]

use std::sync::{Arc, OnceLock};

use jis_korean::kanji::Kanji;
use jis_korean::predict::{KANJI_TOP_N, Kind, MAX_WITH_KANJI, Predictor};
use jis_korean::{Dictionary, Engine, Key, Mode, Mods, Script};

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/predict/");

fn data() -> &'static [u8] {
    static DATA: OnceLock<Vec<u8>> = OnceLock::new();
    DATA.get_or_init(|| std::fs::read(format!("{DIR}kanji.bin")).expect("predict/kanji.bin (engine/tools/kanji/build_kanji.py)"))
}

fn kanji() -> &'static Kanji {
    static KANJI: OnceLock<Kanji> = OnceLock::new();
    KANJI.get_or_init(|| Kanji::parse(data()).unwrap())
}

fn predictor() -> Arc<Predictor> {
    static PREDICTOR: OnceLock<Arc<Predictor>> = OnceLock::new();
    PREDICTOR
        .get_or_init(|| {
            let read = |f: &str| std::fs::read_to_string(format!("{DIR}{f}")).unwrap();
            let model = std::fs::read(format!("{DIR}ranker.bin")).unwrap();
            let p = Predictor::parse(&model, &read("common-en.txt"), &read("common-ko.txt"), &read("english-nouns.txt")).unwrap();
            let p = p.with_japanese_words(&read("common-ja.txt"));
            #[cfg(not(feature = "legacy-predict"))]
            let p = p.with_learned(&std::fs::read(format!("{DIR}learned.bin")).unwrap()).unwrap();
            Arc::new(p.with_kanji(data()).unwrap())
        })
        .clone()
}

fn engine() -> Engine {
    let mut e = Engine::new();
    e.set_predictor(Some(predictor()));
    e
}

fn press(e: &mut Engine, keys: &str) -> String {
    let mut committed = String::new();
    for token in keys.split_whitespace() {
        let (mods, code) = match token.strip_prefix("S-") {
            Some(code) => (Mods::SHIFTED, code),
            None => (Mods::NONE, token),
        };
        committed += &e.key(Key::from_code(code), mods).commit;
    }
    committed
}

fn predicted(e: &Engine) -> Vec<(String, Kind)> {
    e.predictions().into_iter().map(|p| (p.text, p.kind)).collect()
}

fn texts(list: &[(String, Kind)]) -> Vec<&str> {
    list.iter().map(|(t, _)| t.as_str()).collect()
}

#[test]
fn the_converter_meets_its_accuracy_floor_on_mozcs_evaluation_set() {
    // Exact whole-sentence matches on Mozc's own evaluation sentences (tests/data, BSD-3). The
    // whole Mozc dictionary gets 79% with a reference converter; build_kanji.py's cut about 70%.
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/kanji-evaluation.tsv")).unwrap();
    let (mut total, mut exact, mut listed) = (0, 0, 0);
    let mut misses = Vec::new();
    for line in text.lines().filter(|l| !l.starts_with('#')) {
        let fields: Vec<&str> = line.split('\t').collect();
        let [_, input, expected, command, ..] = fields[..] else { continue };
        if !command.starts_with("Conversion") {
            continue;
        }
        total += 1;
        let got = kanji().convert(input, 3);
        exact += (got.first().map(|g| g.0.as_str()) == Some(expected)) as usize;
        listed += got.iter().any(|g| g.0 == expected) as usize;
        if got.first().map(|g| g.0.as_str()) != Some(expected) && misses.len() < 8 {
            misses.push(format!("{input} → {:?}, wanted {expected}", got.first().map(|g| &g.0)));
        }
    }
    eprintln!("kanji: {exact}/{total} exact, {listed}/{total} in the top 3; misses: {misses:#?}");
    assert!(total > 600, "{total}");
    assert!(exact * 100 >= total * 66, "{exact}/{total} exact");
    assert!(listed * 100 >= total * 75, "{listed}/{total} in the top 3");
}

#[test]
fn everyday_phrases_convert() {
    for (kana, want) in [("きょうはいいてんきですね", "今日はいい天気ですね"), ("わたしはがくせいです", "私は学生です"), ("にほんご", "日本語")] {
        let got = kanji().convert(kana, 3);
        assert!(got.iter().any(|g| g.0 == want), "{kana}: {got:?}");
    }
}

#[test]
fn malformed_data_is_an_error_never_a_panic() {
    let data = data();
    assert!(Kanji::parse(&[]).is_err());
    assert!(Kanji::parse(b"JKKJ").is_err());
    // Every truncation of the header and the first runs, and a sample further in.
    for len in (0..4096).chain((4096..data.len()).step_by(data.len() / 64 + 1)) {
        assert!(Kanji::parse(&data[..len]).is_err(), "{len}");
    }
    // Flipped bytes in the header and the tables: parsed or refused, never a panic, and what
    // parses still converts.
    let mut seed = 0x2545_f491_u64;
    for _ in 0..24 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let mut copy = data.to_vec();
        let at = if seed % 3 == 0 { (seed as usize >> 8) % 32 } else { (seed as usize >> 8) % copy.len() };
        copy[at] ^= (seed >> 40) as u8 | 1;
        if let Ok(k) = Kanji::parse(&copy) {
            let _ = k.convert("きょうはいいてんきですね", 3);
        }
    }
    // Input the converter refuses or can't cover is fine too.
    assert!(kanji().convert("", 3).is_empty());
    assert!(kanji().convert(&"あ".repeat(jis_korean::kanji::MAX_INPUT + 1), 3).is_empty());
    assert!(!kanji().convert("゛ゔっ", 3).is_empty());
    assert!(!kanji().convert("abc한글", 3).is_empty());
}

#[test]
fn kanji_join_only_when_a_japanese_reading_is_in_the_top_n() {
    assert_eq!(KANJI_TOP_N, 2);
    // わたしは (0 Q D F) reads as kana first: its conversions follow it.
    let mut e = engine();
    press(&mut e, "Digit0 KeyQ KeyD KeyF");
    let list = predicted(&e);
    assert_eq!(list[0], ("わたしは".into(), Kind::Kana), "{list:?}");
    assert_eq!(list[1], ("私は".into(), Kind::Kanji), "{list:?}");
    assert!(list.len() <= MAX_WITH_KANJI);
    // The rule itself, on Korean and English words: kanji exactly when a Japanese reading is in
    // the top two, then right after it, and never on top.
    let (mut with, mut without) = (0, 0);
    for keys in ["KeyG KeyK KeyS KeyR KeyM KeyF", "KeyG KeyK KeyS KeyR KeyN KeyR", "KeyT KeyH KeyE", "KeyH KeyE KeyL KeyL KeyO", "KeyD KeyK KeyS KeyS KeyU KeyD", "KeyR KeyK KeyA KeyT KeyK KeyG KeyK KeyQ KeyS KeyL KeyE KeyK"] {
        let mut e = engine();
        press(&mut e, keys);
        let list = predicted(&e);
        let japanese = list.iter().take(KANJI_TOP_N).position(|(_, k)| matches!(k, Kind::Kana | Kind::Katakana));
        let kanji: Vec<usize> = (0..list.len()).filter(|&i| list[i].1 == Kind::Kanji).collect();
        match japanese {
            Some(at) => {
                assert!(!kanji.is_empty() && kanji[0] == at + 1, "{keys}: {list:?}");
                with += 1;
            }
            None => {
                assert!(kanji.is_empty(), "{keys}: {list:?}");
                without += 1;
            }
        }
        assert_ne!(list[0].1, Kind::Kanji, "{keys}: {list:?}");
    }
    assert!(with > 0 && without > 0, "both cases: {with} with, {without} without");
}

#[test]
fn no_hanja_readings_among_the_predictions() {
    // 학교 has a dictionary hanja form, 2026년 an everyday one: neither is predicted any more.
    let mut e = engine();
    e.set_dictionary(Arc::new(Dictionary::parse(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/dict/hanja.tsv")).unwrap())));
    for keys in ["KeyG KeyK KeyR KeyR KeyY", "Digit2 Digit0 Digit2 Digit6 KeyS KeyU KeyS"] {
        press(&mut e, keys);
        assert!(predicted(&e).iter().all(|(_, k)| *k != Kind::Hanja), "{keys}: {:?}", predicted(&e));
        e.clear();
    }
}

#[test]
fn f8_puts_kanji_on_top_until_the_commit() {
    // 한 (G K S) reads Korean; F8 converts its kana reading (きのと) and shows the conversions first,
    // then the kana reading, then the rest.
    let mut e = engine();
    press(&mut e, "KeyG KeyK KeyS");
    assert_eq!(predicted(&e)[0], ("한".into(), Kind::Korean));
    assert!(e.key(Key::F8, Mods::NONE).consumed);
    assert!(e.kanji_forced());
    let list = predicted(&e);
    assert_eq!(list[0].1, Kind::Kanji, "{list:?}");
    let kana = list.iter().position(|(_, k)| *k == Kind::Kana).unwrap();
    assert!(list[..kana].iter().all(|(_, k)| *k == Kind::Kanji), "{list:?}");
    assert!(texts(&list).contains(&"한"), "{list:?}");
    // The top conversion commits; the next segment is Auto again.
    let top = list[0].0.clone();
    let out = e.key_after_prediction(0, Key::Space, Mods::NONE);
    assert_eq!(out.commit, format!("{top} "));
    assert!(!e.kanji_forced());
    press(&mut e, "KeyG KeyK KeyS");
    assert_eq!(predicted(&e)[0], ("한".into(), Kind::Korean));
}

#[test]
fn f8_with_nothing_staged_converts_the_next_segment() {
    let mut e = engine();
    assert!(e.key(Key::F8, Mods::NONE).consumed, "taken, as F6 is");
    press(&mut e, "KeyG KeyK KeyS");
    assert_eq!(predicted(&e)[0].1, Kind::Kanji);
}

#[test]
fn f8_after_f6_goes_back_to_auto_with_kanji_and_f6_after_f8_shows_kana() {
    let mut e = engine();
    press(&mut e, "Digit0 KeyQ KeyD F6");
    assert_eq!(e.script(), Script::Hiragana);
    assert!(e.predictions().is_empty());
    e.key(Key::F8, Mods::NONE);
    assert_eq!(e.script(), Script::Auto);
    assert_eq!(predicted(&e)[0], ("私".into(), Kind::Kanji), "{:?}", predicted(&e));
    e.key(Key::F6, Mods::NONE);
    assert!(!e.kanji_forced());
    assert_eq!(e.preedit(), "わたし");
}

#[test]
fn f8_in_jis_kana_offers_conversions_above_the_kana() {
    let mut e = engine();
    e.set_mode(Mode::AKana);
    press(&mut e, "Digit0 KeyQ KeyD KeyF");
    assert!(e.predictions().is_empty(), "JIS Kana predicts nothing by itself");
    e.key(Key::F8, Mods::NONE);
    let list = predicted(&e);
    assert_eq!(list[0], ("私は".into(), Kind::Kanji), "{list:?}");
    assert_eq!(list.last().unwrap(), &("わたしは".into(), Kind::Kana));
    // Katakana script: the kana row is katakana.
    let mut e = engine();
    e.set_mode(Mode::AKana);
    press(&mut e, "F7 Digit0 KeyQ KeyD KeyF F8");
    assert_eq!(predicted(&e).last().unwrap(), &("ワタシハ".into(), Kind::Katakana));
    // Committed, JIS Kana stays in its script with F8 off.
    let out = e.key_after_prediction(0, Key::Enter, Mods::NONE);
    assert_eq!(out.commit, "私は");
    assert!(!e.kanji_forced());
    assert_eq!(e.script(), Script::Katakana);
}

#[test]
fn the_hanja_list_moved_to_shift_f8() {
    let mut e = engine();
    e.set_dictionary(Arc::new(Dictionary::parse(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/dict/hanja.tsv")).unwrap())));
    press(&mut e, "KeyG KeyK KeyS S-F7");
    assert!(!e.hanja_open(), "Shift+F7 is reserved");
    press(&mut e, "S-F8");
    assert!(e.hanja_open());
    let first = e.key(Key::F8, Mods::SHIFTED).candidates.unwrap();
    assert_eq!(first.selected, 1, "Shift+F8 again moves down the list");
    // Plain F8 closes the list and converts.
    e.key(Key::F8, Mods::NONE);
    assert!(!e.hanja_open());
    assert!(e.kanji_forced());
    // With nothing staged, Shift+F8 goes to the app.
    let mut e = engine();
    assert!(!e.key(Key::F8, Mods::SHIFTED).consumed);
}
