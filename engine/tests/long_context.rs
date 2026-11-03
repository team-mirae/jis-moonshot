//! Long context (docs/moonshot.md §5.5): whole phrases typed with Space staging between the words,
//! read as one segment, against the same words typed and committed one at a time. The texts are
//! the evaluation texts (tests/corpora-en-ko, owner's and AI-extended, mixed-script and Korean);
//! the learned ranker never trains on them.
//!
//!     cargo test --release --test long_context -- --nocapture
#![cfg(feature = "predict")]

use std::sync::Arc;

use jis_korean::plan::keys_for_word;
use jis_korean::predict::Predictor;
use jis_korean::{Engine, Key, Mods};

fn predictor() -> Arc<Predictor> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/predict/");
    let read = |f: &str| std::fs::read_to_string(format!("{dir}{f}")).unwrap();
    let model = std::fs::read(format!("{dir}ranker.bin")).unwrap();
    let learned = std::fs::read(format!("{dir}learned.bin")).unwrap();
    let p = Predictor::parse(&model, &read("common-en.txt"), &read("common-ko.txt"), &read("english-nouns.txt")).unwrap();
    Arc::new(p.with_learned(&learned).unwrap())
}

/// A line's words, split at spaces and at punctuation (Research(이하 is Research, 이하); kanji and
/// words no key route types go.
fn words_of(line: &str) -> Vec<String> {
    line.split(|c: char| !c.is_alphanumeric() || ('\u{4E00}'..='\u{9FFF}').contains(&c))
        .filter(|w| !w.is_empty() && keys_for_word(w).is_some())
        .map(str::to_string)
        .collect()
}

/// The evaluation texts' words, split at spaces and punctuation (punctuation has its own tests),
/// one line per entry.
fn lines(prefix: &str) -> Vec<Vec<String>> {
    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpora-en-ko");
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(base)
        .unwrap()
        .chain(std::fs::read_dir(format!("{base}/extended")).unwrap())
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().starts_with(prefix)) && p.extension().is_some_and(|x| x == "txt"))
        .collect();
    files.sort();
    let mut out = Vec::new();
    for f in files {
        for line in std::fs::read_to_string(f).unwrap().lines().filter(|l| !l.starts_with("# ")) {
            let words = words_of(line);
            if words.len() >= 2 {
                out.push(words);
            }
        }
    }
    out
}

fn press_word(e: &mut Engine, word: &str) {
    for p in keys_for_word(word).unwrap() {
        e.key(p.key, if p.shift { Mods::SHIFTED } else { Mods::NONE });
    }
}

/// The top reading of `words` typed with Space between them (staged), and of each word alone.
fn staged(p: &Arc<Predictor>, words: &[String]) -> String {
    let mut e = Engine::new();
    e.set_predictor(Some(p.clone()));
    for (i, w) in words.iter().enumerate() {
        if i > 0 {
            e.key(Key::Space, Mods::NONE);
        }
        press_word(&mut e, w);
    }
    e.predictions().into_iter().next().map_or_else(|| e.preedit(), |t| t.text)
}

fn one_at_a_time(p: &Arc<Predictor>, words: &[String]) -> String {
    let mut e = Engine::new();
    e.set_predictor(Some(p.clone()));
    let mut out = Vec::new();
    for w in words {
        press_word(&mut e, w);
        let top = e.predictions().into_iter().next().map_or_else(|| e.preedit(), |t| t.text);
        // Commit what shows (Shift+Space takes the shown reading, as a shell does).
        e.key_after_prediction(0, Key::Space, Mods::SHIFTED);
        out.push(top);
    }
    out.join(" ")
}

#[derive(Default, Debug)]
struct Score {
    words: usize,
    right: usize,
    phrases: usize,
    exact: usize,
}

fn score(want: &[String], got: &str, s: &mut Score) {
    let got: Vec<&str> = got.split(' ').collect();
    s.phrases += 1;
    s.words += want.len();
    let right = want.iter().enumerate().filter(|(i, w)| got.get(*i) == Some(&w.as_str())).count();
    s.right += right;
    s.exact += (right == want.len() && got.len() == want.len()) as usize;
}

const WINDOW: usize = 8;

/// Word accuracy of staged phrases and of one word at a time, over windows of [`WINDOW`] words.
fn evaluate(prefix: &str, show: usize) -> (Score, Score) {
    evaluate_lines(lines(prefix), show)
}

fn evaluate_lines(all: Vec<Vec<String>>, show: usize) -> (Score, Score) {
    let p = predictor();
    let (mut long, mut short) = (Score::default(), Score::default());
    let mut shown = 0;
    for words in all {
        for chunk in words.chunks(WINDOW).filter(|c| c.len() >= 2) {
            let a = staged(&p, chunk);
            let b = one_at_a_time(&p, chunk);
            score(chunk, &a, &mut long);
            score(chunk, &b, &mut short);
            if a != chunk.join(" ") && shown < show {
                shown += 1;
                eprintln!("  want   {}\n  staged {a}\n  alone  {b}", chunk.join(" "));
            }
        }
    }
    (long, short)
}

fn pct(n: usize, d: usize) -> f64 {
    100.0 * n as f64 / d.max(1) as f64
}

#[test]
fn staged_phrases_read_at_least_as_well_as_words_alone() {
    for prefix in ["mixed-script", "korean"] {
        let (long, short) = evaluate(prefix, 6);
        eprintln!(
            "{prefix}: staged {:.1}% of {} words ({:.1}% of {} phrases exact); one at a time {:.1}% ({:.1}% exact)",
            pct(long.right, long.words),
            long.words,
            pct(long.exact, long.phrases),
            long.phrases,
            pct(short.right, short.words),
            pct(short.exact, short.phrases),
        );
        assert!(long.right + long.words / 100 >= short.right, "{prefix}: staged phrases read worse than words alone: {long:?} vs {short:?}");
    }
}

/// The long-context benchmark (engine/tools/corpora/build_long_eval.py): 480 mixed Korean +
/// English sentences from the open corpora, none of them trained on. Gitignored: skipped without it.
#[test]
fn the_long_context_benchmark() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../corpora/eval-long/mixed.txt");
    let Ok(text) = std::fs::read_to_string(path) else {
        eprintln!("corpora/eval-long/mixed.txt missing: build it with engine/tools/corpora/build_long_eval.py");
        return;
    };
    let all: Vec<Vec<String>> = text.lines().map(words_of).filter(|w| w.len() >= 2).collect();
    let (long, short) = evaluate_lines(all, 40);
    eprintln!(
        "benchmark: staged {:.1}% of {} words ({:.1}% of {} phrases exact); one at a time {:.1}% ({:.1}% exact)",
        pct(long.right, long.words),
        long.words,
        pct(long.exact, long.phrases),
        long.phrases,
        pct(short.right, short.words),
        pct(short.exact, short.phrases),
    );
}

/// How long the predictions of a long staged phrase take (shells ask after every key): printed,
/// not asserted (timing varies by machine).
#[test]
fn a_long_phrase_predicts_in_time() {
    let p = predictor();
    let words = words_of("당사는 인공지능 Artificial Intelligence 기반의 문서 자동화 솔루션인 DocuFlow를 개발하였으며 이를 통해 국내외 고객사에게 맞춤형 model tuning 서비스와 운영 관제 솔루션을 함께 제공하고 있습니다");
    for n in [4, 8, 16, words.len()] {
        let mut e = Engine::new();
        e.set_predictor(Some(p.clone()));
        for (i, w) in words[..n].iter().enumerate() {
            if i > 0 {
                e.key(Key::Space, Mods::NONE);
            }
            press_word(&mut e, w);
        }
        let start = std::time::Instant::now();
        let top = e.predictions().into_iter().next().map(|t| t.text).unwrap_or_default();
        eprintln!("{n:2} words: {:.1} ms  {top}", start.elapsed().as_secs_f64() * 1000.0);
    }
}

#[test]
fn the_owners_example_reads_as_a_phrase() {
    // The request that started docs/moonshot.md §5.5: each prefix of the phrase, typed with Space.
    let p = predictor();
    let want = ["오늘", "Claude의", "artificial", "intelligence", "모델에서", "발표한", "reasoning", "성능", "개선의", "경우"];
    let words: Vec<String> = want.iter().map(|w| w.to_string()).collect();
    for n in 2..=words.len() {
        assert_eq!(staged(&p, &words[..n]), words[..n].join(" "), "{n} words");
    }
}
