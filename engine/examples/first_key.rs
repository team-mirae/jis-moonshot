//! Which language a word's first key press starts, from each language's likeliest words: the data
//! for a first-key prior (docs/research/first-key.md).
//!
//! Words: Korean from `predict/common-ko.txt`, English from `predict/common-en.txt` (by count),
//! Japanese from `predict/common-ja.txt` and the common words of Mozc's dictionary
//! (`predict/kanji.bin`, by cost), the `TOP` likeliest of each. Each language's words share its
//! weight by frequency. First presses and weights:
//!
//! - Korean, efficient (the robot's cheapest route: implied ㅇ, dedicated keys, Shift+vowel): 2;
//! - Korean, diligent (the 초성 first, ㅇ typed): 1, but only when it isn't the efficient press too
//!   (then the word counts once, at 2);
//! - English (the first letter, lowercase): 2;
//! - Japanese (the first kana's key on the JIS kana layout; が on か's key): 2.
//!
//! Usage: cargo run --release --example first_key > ../docs/research/first-key.tsv
//! (the table on stdout; a summary on stderr).

use std::collections::{BTreeMap, HashMap};

use jis_korean::kana::KanaScheme;
use jis_korean::kanji::Kanji;
use jis_korean::plan::keys_for_word;
use jis_korean::{Key, Layout, Mode, Out, output_in};

/// The likeliest words taken per language.
const TOP: usize = 5000;
/// Mozc costs above this aren't common words (the predictor's cutoff, `COMMON_KANJI_WORD`).
const COMMON_KANJI_WORD: u16 = 4500;
/// A word without a count in a hand-written list counts as this (the predictor's `HAND_COUNT`).
const HAND_COUNT: f64 = 1000.0;

/// `word<TAB>count` or `word` lines (count: `HAND_COUNT`), skipping comments and `#!` lines.
fn counts(text: &str) -> Vec<(String, f64)> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| match l.split_once('\t') {
            Some((w, n)) => (w.trim().to_string(), n.trim().parse().unwrap_or(HAND_COUNT)),
            None => (l.to_string(), HAND_COUNT),
        })
        .collect()
}

/// The `TOP` heaviest, as shares of their total weight.
fn top_shares(mut words: Vec<(String, f64)>) -> Vec<(String, f64)> {
    words.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    words.dedup_by(|a, b| a.0 == b.0);
    words.truncate(TOP);
    let total: f64 = words.iter().map(|w| w.1).sum();
    words.into_iter().map(|(w, n)| (w, n / total)).collect()
}

fn press_name(key: Key, shift: bool) -> String {
    format!("{}{}", if shift { "S-" } else { "" }, key.code())
}

fn is_syllable(c: char) -> bool {
    ('가'..='힣').contains(&c)
}

/// The key Mode A types a jamo with, unshifted first (ㅇ → D, ㄲ → 4).
fn jamo_key(jamo: char) -> Option<(Key, bool)> {
    [false, true].into_iter().find_map(|shift| {
        Key::ALL.iter().copied().find(|&k| output_in(Mode::A, Layout::JisKorean, k, shift) == Some(Out::Jamo(jamo))).map(|k| (k, shift))
    })
}

fn main() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/predict/");
    let read = |f: &str| std::fs::read_to_string(format!("{dir}{f}")).unwrap();

    // Korean and English by count; Japanese half from the hand-written list, half from Mozc.
    let korean = top_shares(counts(&read("common-ko.txt")).into_iter().filter(|(w, _)| w.chars().all(is_syllable)).collect());
    let english = top_shares(
        counts(&read("common-en.txt")).into_iter().filter(|(w, _)| w.chars().all(|c| c.is_ascii_alphabetic())).map(|(w, n)| (w.to_lowercase(), n)).collect(),
    );
    let kanji = Kanji::parse(&std::fs::read(format!("{dir}kanji.bin")).unwrap()).unwrap();
    let hiragana_word = |w: &str| w.chars().count() >= 2 && w.chars().all(|c| ('ぁ'..='ゖ').contains(&c) || c == 'ー');
    // Mozc's costs are about 500 × −ln p: back to relative likelihoods.
    let mozc = top_shares(
        kanji.words().filter(|&(w, cost)| cost <= COMMON_KANJI_WORD && hiragana_word(w)).map(|(w, cost)| (w.to_string(), (-(cost as f64) / 500.0).exp())).collect(),
    );
    let listed = top_shares(
        counts(&read("common-ja.txt")).into_iter().map(|(w, n)| (jis_korean::kana::hiragana(&w), n)).filter(|(w, _)| hiragana_word(w)).collect(),
    );
    let mut japanese: HashMap<String, f64> = HashMap::new();
    for (w, s) in mozc.iter().chain(&listed) {
        *japanese.entry(w.clone()).or_default() += s / 2.0;
    }

    // kana → its key on the JIS kana layout (unshifted first), and a voiced kana's base kana.
    let mut kana_key: HashMap<char, (Key, bool)> = HashMap::new();
    for shift in [false, true] {
        for &k in Key::ALL {
            if let Some(c) = KanaScheme::Jis.kana(k, shift) {
                kana_key.entry(c).or_insert((k, shift));
            }
        }
    }
    let first_kana_key = |w: &str| {
        let c = w.chars().next()?;
        let c = if c == 'ゔ' { 'う' } else { c };
        // が = か + 1, ぱ = は + 2 in Unicode: the key of the kana ゛ / ゜ attaches to.
        [0u32, 1, 2].iter().find_map(|&d| char::from_u32(c as u32 - d).and_then(|b| kana_key.get(&b).copied()))
    };

    // language → press → weight
    let mut mass: BTreeMap<&str, HashMap<String, f64>> = BTreeMap::new();
    let mut add = |lang: &'static str, press: String, w: f64| *mass.entry(lang).or_default().entry(press).or_default() += w;
    let (mut same, mut differ, mut unplanned) = (0.0, 0.0, 0usize);
    for (word, share) in &korean {
        let first = word.chars().next().unwrap();
        let initial = ['ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'][(first as u32 - 0xAC00) as usize / 588];
        let Some((dk, ds)) = jamo_key(initial) else { continue };
        let diligent = press_name(dk, ds);
        let efficient = match keys_for_word(word).and_then(|k| k.first().copied()) {
            Some(p) => press_name(p.key, p.shift),
            None => {
                unplanned += 1;
                diligent.clone()
            }
        };
        if efficient == diligent {
            same += share;
            add("ko", efficient, 2.0 * share);
        } else {
            differ += share;
            add("ko", efficient, 2.0 * share);
            add("ko", diligent, *share);
        }
    }
    for (word, share) in &english {
        let c = word.chars().next().unwrap().to_ascii_uppercase();
        add("en", format!("Key{c}"), 2.0 * share);
    }
    let mut unmapped = 0.0;
    for (word, share) in &japanese {
        match first_kana_key(word) {
            Some((k, s)) => add("ja", press_name(k, s), 2.0 * share),
            None => unmapped += share,
        }
    }

    // The table: each press's weight per language, and the language given the press.
    let langs = ["ko", "en", "ja"];
    let mut presses: Vec<String> = mass.values().flat_map(|m| m.keys().cloned()).collect();
    presses.sort();
    presses.dedup();
    let total = |p: &str| langs.iter().map(|l| mass.get(l).and_then(|m| m.get(p)).copied().unwrap_or(0.0)).sum::<f64>();
    presses.sort_by(|a, b| total(b).total_cmp(&total(a)));
    println!("# press\tko\ten\tja\tP(ko|press)\tP(en|press)\tP(ja|press)");
    let (mut h_cond, mut decisive, mut all) = (0.0, 0.0, 0.0);
    for p in &presses {
        let w: Vec<f64> = langs.iter().map(|l| mass.get(l).and_then(|m| m.get(p)).copied().unwrap_or(0.0)).collect();
        let t: f64 = w.iter().sum();
        let post: Vec<f64> = w.iter().map(|x| x / t).collect();
        println!("{p}\t{:.5}\t{:.5}\t{:.5}\t{:.3}\t{:.3}\t{:.3}", w[0], w[1], w[2], post[0], post[1], post[2]);
        h_cond += t * post.iter().filter(|&&q| q > 0.0).map(|q| -q * q.log2()).sum::<f64>();
        if post.iter().any(|&q| q >= 0.8) {
            decisive += t;
        }
        all += t;
    }
    let prior: Vec<f64> = langs.iter().map(|l| mass.get(l).map_or(0.0, |m| m.values().sum::<f64>()) / all).collect();
    let h: f64 = prior.iter().filter(|&&q| q > 0.0).map(|q| -q * q.log2()).sum();
    let h_cond = h_cond / all;
    eprintln!("words: ko {} en {} ja {}; Korean efficient = diligent for {:.1}% of the weight ({:.1}% differ); {unplanned} unplanned", korean.len(), english.len(), japanese.len(), 100.0 * same, 100.0 * differ);
    eprintln!("Japanese weight without a key: {:.2}%", 100.0 * unmapped);
    eprintln!("prior ko {:.3} en {:.3} ja {:.3}: H(lang) = {h:.3} bits", prior[0], prior[1], prior[2]);
    eprintln!("H(lang | first press) = {h_cond:.3} bits: the first press tells {:.3} bits ({:.0}% of the uncertainty)", h - h_cond, 100.0 * (h - h_cond) / h);
    eprintln!("{:.0}% of the weight starts on a press that's ≥ 80% one language", 100.0 * decisive / all);
}
