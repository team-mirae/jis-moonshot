//! Prints what JIS Moonshot shows for key sequences (docs/moonshot.md): the preedit and the ranked
//! predictions. `cargo run --example moonshot_probe -- "KeyB KeyY KeyI KeyA KeyF" ...`
use std::sync::Arc;

use jis_korean::predict::Predictor;
use jis_korean::{Engine, Key, Mods};

fn main() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/predict/");
    let read = |f: &str| std::fs::read_to_string(format!("{dir}{f}")).unwrap();
    let model = std::fs::read(format!("{dir}ranker.bin")).unwrap();
    let learned = std::fs::read(format!("{dir}learned.bin")).unwrap();
    let p = Predictor::parse(&model, &read("common-en.txt"), &read("common-ko.txt"), &read("english-nouns.txt")).unwrap();
    let p = p.with_learned(&learned).unwrap().with_japanese_words(&read("common-ja.txt"));
    // The kanji dictionary, as the shells load it (docs/moonshot.md §5.1).
    let p = Arc::new(p.with_kanji(&std::fs::read(format!("{dir}kanji.bin")).unwrap()).unwrap());
    for keys in std::env::args().skip(1) {
        let mut e = Engine::new();
        e.set_predictor(Some(p.clone()));
        for token in keys.split_whitespace() {
            let (mods, code) = match token.strip_prefix("S-") {
                Some(code) => (Mods::SHIFTED, code),
                None => (Mods::NONE, token),
            };
            e.key(Key::from_code(code), mods);
        }
        let shown: Vec<String> = e.predictions().iter().map(|p| format!("{}({:?} {:.1})", p.text, p.kind, p.score)).collect();
        println!("{keys}\n  preedit {} | kana {} | {}", e.preedit(), e.kana(), shown.join("  "));
    }
}
