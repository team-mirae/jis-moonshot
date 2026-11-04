//! The robot typing test: a robot types real text through the engine, key by key, and the screen
//! must match the text after every character. Run with `--nocapture` to see the report.
//!
//! Corpus: Samsung Electronics' latest "II. 사업의 내용" (fetch it with
//! `python3 -I tests/data/fetch_corpus.py` from `tests/data`). Without it, only the built-in
//! sample runs.


use jis_korean::plan::Robot;

/// Covers every part of the layout: dedicated keys, the implied ㅇ and the double press, symbols
/// off the legends, Latin words in all cases, digits, a hanja (pasted) and a bare jamo.
const SAMPLE: &str = "\
말이 많은 사람은 아이와 오이를 샀다. 닭이 울면 갈아입고 앉아서 읽어 봐요.
2026년 상반기 DRAM과 NAND Flash 가격은 3~5% 올랐다(예상). 全 세계 HBM3E 수요—「AI」·클라우드!
문의: help@example.com, 값은 1,234원; 됐어? 빠르게 꽤 왜 웬 쐐기 뾰족한 iPhone과 Galaxy를 ㄱ자로 놓았다.
";

fn run(name: &str, text: &str) -> Robot {
    let mut robot = Robot::new();
    let expected = robot.type_text(text);
    eprintln!("── robot: {name}\n{}", robot.report);
    assert!(robot.report.failures.is_empty(), "{name}: {} failures", robot.report.failures.len());
    assert_eq!(robot.document(), expected, "{name}: document differs from the text");
    robot
}

#[test]
fn robot_types_the_sample() {
    let robot = run("sample", SAMPLE);
    let r = &robot.report;
    assert!(r.dropped.is_empty(), "sample should be fully typable: {:?}", r.dropped);
    // Each feature actually got used.
    assert!(r.dedicated_keys > 0 && r.implied_ieung > 0 && r.double_presses > 0);
    assert!(r.pasted >= 1 && r.latin_words >= 7);
}

#[test]
fn robot_types_samsung_business() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/samsung-business.txt");
    let Ok(text) = std::fs::read_to_string(path) else {
        eprintln!("skipped: {path} missing; run tests/data/fetch_corpus.py");
        return;
    };
    let robot = run("Samsung Electronics, 사업의 내용", &text);
    assert!(robot.report.chars > 10_000);
}

#[test]
fn count_compares_jis_korean_with_conventional() {
    let counts = jis_korean::plan::count("말이 오이 같이 하자");
    assert_eq!(counts.chars, 11);
    assert_eq!(counts.dropped, 0);
    assert!(counts.jis < counts.conventional, "{counts:?}");
    // 말이: a k f ⇧l (5 with Shift) vs a k f d l (5); 오이: h ⇧l (3) vs d h d l (4) …
    assert_eq!(jis_korean::plan::conventional("까"), 3);
    assert_eq!(jis_korean::plan::count("까").jis, 2);
}

#[test]
fn symbols_off_the_legends_cost_three_on_both_layouts() {
    // ! is on stock JIS (Shift+1) but not on the JIS Moonshot legends (Shift+1 is 1); @ is on
    // both since 2026-11-05 (docs/moonshot.md §9.2).
    assert_eq!(jis_korean::plan::count("!").jis, 3);
    assert_eq!(jis_korean::plan::conventional("!"), 2);
    assert_eq!(jis_korean::plan::count("@").jis, 1);
    assert_eq!(jis_korean::plan::conventional("@"), 1);
    // ( ) have no key in JIS Moonshot's Mode A (docs/moonshot.md §9): three each; stock JIS has
    // them on Shift+8 / Shift+9.
    assert_eq!(jis_korean::plan::count("()").jis, 6);
    assert_eq!(jis_korean::plan::conventional("()"), 4);
}

#[test]
fn characters_no_key_types_cost_three_on_both_layouts() {
    let counts = jis_korean::plan::count("①");
    assert_eq!((counts.jis, counts.conventional, counts.off_keyboard, counts.dropped), (3, 3, 1, 0));
    // A hanja is entered from outside too (JIS Moonshot has no hanja list).
    assert_eq!(jis_korean::plan::count("學").jis, 3);
    // Spaces and line breaks are one press each, both ways.
    assert_eq!(jis_korean::plan::count(" \n").jis, 2);
    assert_eq!(jis_korean::plan::conventional(" \n"), 2);
}
