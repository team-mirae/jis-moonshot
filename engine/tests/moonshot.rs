//! JIS Moonshot (docs/moonshot.md): kana typing, scripts, the switch keys and the kana reading.

use jis_korean::{Engine, Key, Mode, Mods, Script};

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

fn typed(keys: &str) -> Engine {
    let mut e = Engine::new();
    press(&mut e, keys);
    e
}

#[test]
fn modes_are_jis_only_and_kana_is_a_mode() {
    let ids: Vec<u32> = Mode::ALL.iter().map(|m| m.id()).collect();
    // The English modes only with the `english` feature (on by default).
    let english: &[u32] = if cfg!(feature = "english") { &[3, 7, 8] } else { &[] };
    assert_eq!(ids, [&[0, 11][..], english].concat());
    for retired in [1, 2, 9, 10, 12] {
        assert_eq!(Mode::from_id(retired), None, "{retired}");
    }
    let kana = Mode::from_id(11).unwrap();
    assert_eq!((kana.code(), kana.name(), kana.is_kana(), kana.is_english(), kana.is_korean()), ("A_KANA", "JIS Kana", true, false, false));
    assert_eq!(Mode::A.name(), "JIS Moonshot");
}

#[test]
fn the_kana_automaton_composes_dakuten_and_handakuten() {
    // §3.1: か゛ → が, は゜ → ぱ, う゛ → ゔ; a mark that can't attach stays.
    assert_eq!(typed("KeyT BracketLeft").kana(), "が");
    assert_eq!(typed("KeyF BracketRight").kana(), "ぱ");
    assert_eq!(typed("Digit4 BracketLeft").kana(), "ゔ");
    assert_eq!(typed("Digit3 BracketLeft").kana(), "あ゛");
    // Backspace takes the mark back off.
    let mut e = typed("KeyT BracketLeft");
    e.key(Key::Backspace, Mods::NONE);
    assert_eq!(e.kana(), "か");
}

#[test]
fn the_kana_reading_follows_the_jis_kana_layout() {
    assert_eq!(typed("KeyB KeyY KeyI KeyA KeyF").kana(), "こんにちは");
    assert_eq!(typed("Digit3 KeyL KeyT BracketLeft KeyS Digit4").kana(), "ありがとう");
    // Shift: small kana, を, and Japanese punctuation.
    assert_eq!(typed("KeyG S-Digit7").kana(), "きゃ");
    assert_eq!(typed("KeyG S-KeyZ KeyW").kana(), "きって");
    assert_eq!(typed("S-Digit0 S-Comma S-Period S-Slash S-BracketRight S-Backslash").kana(), "を、。・「」");
    assert_eq!(typed("KeyB IntlYen KeyV IntlYen").kana(), "こーひー");
}

#[test]
fn shift_z_is_no_longer_f10() {
    // §1: Shift+Z is ㅋ in Korean (standard 2-set), っ in kana, Z in English.
    let e = typed("KeyG KeyK KeyS S-KeyZ");
    assert_eq!(e.preedit(), "한ㅋ");
    assert_eq!(e.english(), "gksZ");
    assert_eq!(e.kana(), "きのとっ");
}

#[test]
fn f6_and_f7_show_the_staged_keys_as_kana_until_the_commit() {
    // Mode A stays: F6 / F7 are for this segment only (docs/moonshot.md §4).
    let mut e = typed("KeyB IntlYen KeyV IntlYen");
    e.key(Key::F7, Mods::NONE);
    assert_eq!((e.mode(), e.script(), e.preedit().as_str()), (Mode::A, Script::Katakana, "コーヒー"));
    e.key(Key::F6, Mods::NONE);
    assert_eq!((e.mode(), e.script(), e.preedit().as_str()), (Mode::A, Script::Hiragana, "こーひー"));
    // Keys typed after it are kana too, Shift+Z included.
    press(&mut e, "KeyG S-KeyZ KeyW");
    assert_eq!(e.preedit(), "こーひーきって");
    // Enter confirms without a new line; then predictive (Auto) again.
    let out = e.key(Key::Enter, Mods::NONE);
    assert_eq!((out.commit.as_str(), out.consumed, e.script()), ("こーひーきって", true, Script::Auto));
    // Shift+Enter commits the same way.
    press(&mut e, "KeyG KeyK KeyS F6");
    let out = e.key(Key::Enter, Mods::SHIFTED);
    assert_eq!((out.commit.as_str(), e.script()), ("きのと", Script::Auto));
    assert_eq!(typed("KeyG KeyK KeyS").preedit(), "한");
    // Space types a full-width space after kana.
    press(&mut e, "KeyT F6");
    assert_eq!(e.key(Key::Space, Mods::NONE).commit, "か\u{3000}");
}

#[test]
fn f6_and_f7_with_nothing_staged_make_the_next_segment_kana() {
    let mut e = Engine::new();
    let out = e.key(Key::F7, Mods::NONE);
    assert!(out.consumed);
    assert_eq!((e.mode(), e.script()), (Mode::A, Script::Katakana));
    press(&mut e, "KeyT BracketLeft");
    assert_eq!(e.preedit(), "ガ");
}

#[test]
fn jis_kana_keeps_its_script_across_commits() {
    let mut e = Engine::new();
    e.set_mode(Mode::AKana);
    assert_eq!(e.script(), Script::Hiragana);
    e.key(Key::F7, Mods::NONE);
    press(&mut e, "KeyB IntlYen KeyV IntlYen");
    assert_eq!(e.key(Key::Enter, Mods::NONE).commit, "コーヒー");
    assert_eq!((e.mode(), e.script()), (Mode::AKana, Script::Katakana));
    press(&mut e, "KeyT");
    assert_eq!(e.preedit(), "カ");
}

#[test]
fn the_kana_key_forces_korean_and_goes_back_to_mode_a() {
    // In Mode A, staged: forced Korean, the same keys.
    let mut e = typed("KeyG KeyK KeyS");
    e.kana_key();
    assert_eq!((e.mode(), e.script(), e.preedit().as_str()), (Mode::A, Script::Korean, "한"));
    // Forced Korean lasts for the segment; the next one is Auto again.
    let out = e.key(Key::Space, Mods::NONE);
    assert_eq!(out.commit, "한 ");
    assert_eq!(e.script(), Script::Auto);
    // Nothing staged: nothing.
    e.kana_key();
    assert_eq!((e.mode(), e.script()), (Mode::A, Script::Auto));
    // From kana (F6, or JIS Kana): Mode A, the staged keys re-read as Korean.
    let mut e = typed("KeyG KeyK KeyS");
    e.key(Key::F6, Mods::NONE);
    assert_eq!(e.preedit(), "きのと");
    e.kana_key();
    assert_eq!((e.mode(), e.script(), e.preedit().as_str()), (Mode::A, Script::Korean, "한"));
    let mut e = Engine::new();
    e.set_mode(Mode::AKana);
    press(&mut e, "KeyG KeyK KeyS");
    assert_eq!(e.preedit(), "きのと");
    e.kana_key();
    assert_eq!((e.mode(), e.script(), e.preedit().as_str()), (Mode::A, Script::Korean, "한"));
}

#[test]
fn setting_the_same_mode_keeps_a_forced_script() {
    // macOS echoes a switch back as a mode selection.
    let mut e = typed("KeyG KeyK KeyS");
    e.key(Key::F6, Mods::NONE);
    e.kana_key();
    e.set_mode(Mode::A);
    assert_eq!((e.script(), e.preedit().as_str()), (Script::Korean, "한"));
    let mut e = Engine::new();
    e.set_mode(Mode::AKana);
    e.key(Key::F7, Mods::NONE);
    e.set_mode(Mode::AKana);
    press(&mut e, "KeyT");
    assert_eq!(e.preedit(), "カ");
}

#[cfg(feature = "english")]
#[test]
fn the_kana_key_leaves_english_for_mode_a() {
    let mut e = Engine::new();
    e.set_mode(Mode::AQwerty);
    let out = e.kana_key();
    assert!(out.consumed);
    assert_eq!((e.mode(), e.script()), (Mode::A, Script::Auto));
}

#[test]
fn f6_after_f9_undoes_the_conversion_first() {
    let mut e = typed("KeyG KeyK KeyS");
    e.key(Key::F9, Mods::NONE);
    assert_eq!(e.preedit(), "ｇｋｓ");
    e.key(Key::F6, Mods::NONE);
    assert_eq!(e.preedit(), "きのと");
}

#[test]
fn f9_and_f10_in_kana_convert_the_keys() {
    let mut e = typed("KeyG KeyK Minus");
    e.key(Key::F6, Mods::NONE);
    assert_eq!(e.preedit(), "きのほ");
    e.key(Key::F10, Mods::NONE);
    assert_eq!(e.preedit(), "gk-");
}

#[test]
fn option_space_types_a_full_width_space() {
    let alt = Mods { alt: true, ..Mods::NONE };
    let mut e = Engine::new();
    // Off by default: a shortcut, committed and passed on.
    press(&mut e, "KeyG KeyK KeyS");
    let out = e.key(Key::Space, alt);
    assert_eq!((out.commit.as_str(), out.consumed), ("한", false));
    e.set_option_space(true);
    press(&mut e, "KeyG KeyK KeyS");
    let out = e.key(Key::Space, alt);
    assert_eq!((out.commit.as_str(), out.consumed), ("한\u{3000}", true));
    // With kana showing, and with nothing staged.
    press(&mut e, "KeyT F6");
    assert_eq!(e.key(Key::Space, alt).commit, "か\u{3000}");
    assert_eq!(e.key(Key::Space, alt).commit, "\u{3000}");
    // ⌘ or Ctrl with it is still a shortcut.
    assert!(!e.key(Key::Space, Mods { alt: true, meta: true, ..Mods::NONE }).consumed);
}

#[test]
fn option_symbols_type_full_width() {
    let alt = Mods { alt: true, ..Mods::NONE };
    let shift_alt = Mods { alt: true, shift: true, ..Mods::NONE };
    let mut e = Engine::new();
    // Off by default: a shortcut, committed and passed on.
    press(&mut e, "KeyG KeyK KeyS");
    assert_eq!((e.key(Key::Digit1, shift_alt).commit.as_str(), e.preedit().as_str()), ("한", ""));
    e.set_option_space(true);
    // ⇧⌥1 → ！ after what's staged; ⌥ , . / [ ] → 、。・「」 (JIS [ and ] are the BracketRight and
    // Backslash codes; BracketLeft is @); ⌥1 → １.
    press(&mut e, "KeyG KeyK KeyS");
    let out = e.key(Key::Digit1, shift_alt);
    assert_eq!((out.commit.as_str(), out.consumed), ("한！", true));
    let typed: String = [Key::Comma, Key::Period, Key::Slash, Key::BracketRight, Key::Backslash, Key::BracketLeft, Key::Digit1]
        .into_iter()
        .map(|k| e.key(k, alt).commit)
        .collect();
    assert_eq!(typed, "、。・『』＠１", "Mode A: ⌥[ ⌥] are the double brackets");
    assert_eq!(e.key(Key::Slash, shift_alt).commit, "？");
    // In JIS Kana too, after the kana; ⌥[ ⌥] are the single brackets there.
    e.set_mode(Mode::AKana);
    press(&mut e, "KeyT");
    assert_eq!(e.key(Key::Period, alt).commit, "か。");
    assert_eq!(e.key(Key::BracketRight, alt).commit + &e.key(Key::Backslash, alt).commit, "「」");
    // Letters stay shortcuts (passed on), and ⌘ with it too.
    e.set_mode(Mode::A);
    assert!(!e.key(Key::KeyA, alt).consumed);
    assert!(!e.key(Key::Digit1, Mods { alt: true, meta: true, ..Mods::NONE }).consumed);
}

#[cfg(feature = "english")]
#[test]
fn option_space_types_an_underscore_in_english() {
    let alt = Mods { alt: true, ..Mods::NONE };
    let mut e = Engine::new();
    e.set_mode(Mode::AQwerty);
    assert!(!e.key(Key::Space, alt).consumed, "off by default");
    e.set_option_space(true);
    let out = e.key(Key::Space, alt);
    assert_eq!((out.commit.as_str(), out.consumed), ("_", true));
    // Plain Space is still the app's.
    assert!(!e.key(Key::Space, Mods::NONE).consumed);
}

#[cfg(feature = "english")]
#[test]
fn option_brackets_type_guillemets_in_english() {
    let alt = Mods { alt: true, ..Mods::NONE };
    let mut e = Engine::new();
    e.set_mode(Mode::AQwerty);
    assert!(!e.key(Key::BracketRight, alt).consumed, "off by default");
    e.set_option_space(true);
    // JIS [ and ] are the BracketRight and Backslash codes.
    let out = e.key(Key::BracketRight, alt);
    assert_eq!((out.commit.as_str(), out.consumed), ("«", true));
    assert_eq!(e.key(Key::Backslash, alt).commit, "»");
    // Other symbols, and ⇧⌥[, are still the app's.
    assert!(!e.key(Key::Comma, alt).consumed);
    assert!(!e.key(Key::BracketRight, Mods { alt: true, shift: true, ..Mods::NONE }).consumed);
}

#[test]
fn the_hanja_list_moved_to_shift_f8() {
    let dict = jis_korean::Dictionary::parse(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/dict/hanja.tsv")).unwrap());
    let mut e = Engine::new();
    e.set_dictionary(std::sync::Arc::new(dict));
    press(&mut e, "KeyR KeyK KeyR KeyY");
    assert!(e.key(Key::F7, Mods::SHIFTED).candidates.is_none(), "Shift+F7 is reserved");
    let out = e.key(Key::F8, Mods::SHIFTED);
    assert!(out.candidates.is_some(), "{out:?}");
    assert_eq!(e.mode(), Mode::A);
}

#[cfg(feature = "predict")]
mod predictions {
    use std::sync::Arc;

    use jis_korean::predict::{Kind, Predictor};

    use super::*;

    fn predictor() -> Arc<Predictor> {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/predict/");
        let read = |f: &str| std::fs::read_to_string(format!("{dir}{f}")).unwrap();
        let model = std::fs::read(format!("{dir}ranker.bin")).unwrap();
        let learned = std::fs::read(format!("{dir}learned.bin")).unwrap();
        let p = Predictor::parse(&model, &read("common-en.txt"), &read("common-ko.txt"), &read("english-nouns.txt")).unwrap();
        Arc::new(p.with_learned(&learned).unwrap().with_japanese_words(&read("common-ja.txt")))
    }

    fn predicted(keys: &str) -> Vec<(String, Kind)> {
        let mut e = Engine::new();
        e.set_predictor(Some(predictor()));
        press(&mut e, keys);
        e.predictions().into_iter().map(|p| (p.text, p.kind)).collect()
    }

    fn top(keys: &str) -> (String, Kind) {
        predicted(keys).remove(0)
    }

    #[test]
    fn japanese_words_read_as_kana() {
        assert_eq!(top("KeyB KeyY KeyI KeyA KeyF"), ("こんにちは".into(), Kind::Kana));
        assert_eq!(top("Digit3 KeyL KeyT BracketLeft KeyS Digit4"), ("ありがとう".into(), Kind::Kana));
        assert_eq!(top("Digit0 KeyQ KeyD KeyF"), ("わたしは".into(), Kind::Kana));
        assert_eq!(top("KeyG S-KeyZ KeyW"), ("きって".into(), Kind::Kana));
        assert_eq!(top("KeyB IntlYen KeyV IntlYen"), ("コーヒー".into(), Kind::Katakana));
        // A phrase of known words and particles; a katakana word inside it stays katakana.
        assert_eq!(top("Digit3 KeyU KeyQ KeyK"), ("あなたの".into(), Kind::Kana));
        assert_eq!(
            top("Digit3 KeyU KeyQ KeyK KeyG BracketLeft KeyQ IntlYen KeyI KeyU KeyL KeyQ KeyE"),
            ("あなたのギターになりたい".into(), Kind::Kana)
        );
        assert!(predicted("Digit3 KeyU KeyQ KeyK KeyG BracketLeft KeyQ IntlYen KeyI KeyU KeyL KeyQ KeyE")
            .iter()
            .all(|(text, kind)| *kind == Kind::Kana || !text.contains('に')));
    }

    #[test]
    fn korean_and_english_words_stay_on_top_with_kana_offered() {
        for (keys, word) in [("KeyG KeyK KeyS KeyR KeyM KeyF", "한글"), ("KeyD KeyK KeyS KeyS KeyU KeyD", "안녕"), ("KeyT KeyK KeyF KeyK KeyD", "사랑")] {
            let p = predicted(keys);
            assert_eq!(p[0], (word.to_string(), Kind::Korean), "{keys}: {p:?}");
            assert!(p.iter().any(|(_, k)| *k == Kind::Kana), "{keys}: {p:?}");
        }
        assert_eq!(top("KeyH KeyE KeyL KeyL KeyO").0, "hello");
        assert_eq!(top("S-Digit2 S-Digit0 S-Digit2 S-Digit6").0, "2026");
    }

    #[test]
    fn misplaced_small_kana_rank_low() {
        let p = predicted("KeyT S-Digit7");
        assert_ne!(p[0].1, Kind::Kana, "{p:?}");
        assert_eq!(top("KeyG S-Digit7"), ("きゃ".into(), Kind::Kana));
    }

    #[test]
    fn the_kana_reading_takes_a_slot_of_its_own() {
        for keys in ["KeyB KeyY KeyI KeyA KeyF", "KeyG KeyK KeyS KeyR KeyM KeyF", "KeyH KeyE KeyL KeyL KeyO"] {
            assert!(predicted(keys).len() <= jis_korean::predict::MAX_WITH_KANA, "{keys}");
        }
    }

    #[test]
    fn forced_scripts_show_no_predictions() {
        let mut e = Engine::new();
        e.set_predictor(Some(predictor()));
        press(&mut e, "KeyB KeyY KeyI KeyA KeyF");
        assert!(!e.predictions().is_empty());
        e.kana_key();
        assert!(e.predictions().is_empty());
        e.key(Key::F6, Mods::NONE);
        assert!(e.predictions().is_empty());
        assert_eq!(e.preedit(), "こんにちは");
    }

    #[test]
    fn a_kana_training_example_is_learned_with_its_keys() {
        let mut e = Engine::new();
        e.set_predictor(Some(predictor()));
        let keys = jis_korean::plan::parse_keys("KeyG S-KeyZ KeyW").unwrap();
        assert_eq!(e.learn_example(Some(&keys), "きって"), Some(0));
        assert!(e.learner().weight(jis_korean::predict::Weight::Kana) > 1.0);
    }

    #[test]
    fn the_kana_weight_turns_it_off() {
        let mut e = Engine::new();
        e.set_predictor(Some(predictor()));
        e.set_prediction_weight(jis_korean::predict::Weight::Kana, 0.0);
        press(&mut e, "KeyB KeyY KeyI KeyA KeyF");
        assert!(e.predictions().iter().all(|p| p.kind != Kind::Kana));
    }
}
