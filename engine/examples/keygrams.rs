//! Builds the three-language key model with Shift (`JKR4`, src/keymodel.rs) and, as its reference,
//! the dense table of every 4-gram: all 88,501,248 over 97 symbols (the 48 JIS keys, the same with
//! Shift, and the space between words, never two in a row), scored for Korean, English and kana.
//!
//! Reads corpora/keygrams/words.tsv (engine/tools/corpora/build_keygrams.py; run both with
//! build_keymodel.py). Korean words are typed three ways, each checked to type the word through
//! the engine (else left out): the conventional route (a quarter of the weight), the cheapest JIS
//! Moonshot way from the engine's planner (half) and that way with Shift+consonant for each
//! consonant + ㅏ (a quarter, docs/moonshot.md §9.2). Each word is counted with one space before
//! and after it, orders 1–4, and smoothed Witten-Bell (interpolated, as build_ranker.KeyModel).
//!
//! Writes:
//! - corpora/keygrams/cost.bin, the dense reference: "JKG4", u32 version 1, u32 alphabet 97, u32
//!   classes 3; then per class (Korean, English, kana) 97² bytes of order 2, 97³ of order 3 and
//!   97⁴ of order 4, each the cost of the n-gram's last symbol after the ones before it,
//!   round(16 × −log2 P), clamped 1..255; 0 where two spaces follow each other. Index: the
//!   symbols as base-97 digits, oldest first.
//! - `--out` (default engine/predict/ranker.bin): the two-language `JKRK` tables the file starts
//!   with, kept, then the sparse `JKR4` model: every seen n-gram and every unigram with its cost,
//!   every seen history with its backoff cost, round(16 × −log2 (T / (t + T))).
//!
//! Then it checks the sparse model against the dense table on every valid n-gram and prints the
//! held-out three-way accuracy. `cargo run --release --example keygrams [-- --out <file>]`.

use std::time::Instant;

use jis_korean::keymodel::{Class, KeyModel, SPACE, key};
use jis_korean::plan::{Press, keys_for_word};
use jis_korean::{Engine, Key, Mods};

const A: usize = 97;
const CLASSES: [&str; 3] = ["ko", "en", "ja"];
const KEY_CODES: [&str; 48] = [
    "KeyA", "KeyB", "KeyC", "KeyD", "KeyE", "KeyF", "KeyG", "KeyH", "KeyI", "KeyJ", "KeyK", "KeyL", "KeyM", "KeyN", "KeyO", "KeyP",
    "KeyQ", "KeyR", "KeyS", "KeyT", "KeyU", "KeyV", "KeyW", "KeyX", "KeyY", "KeyZ", "Digit0", "Digit1", "Digit2", "Digit3",
    "Digit4", "Digit5", "Digit6", "Digit7", "Digit8", "Digit9", "Minus", "Equal", "IntlYen", "BracketLeft", "BracketRight",
    "Semicolon", "Quote", "Backslash", "Comma", "Period", "Slash", "IntlRo",
];
/// Consonant keys whose Shift types the consonant and ㅏ (docs/moonshot.md §9.2).
const SHIFT_CONSONANTS: [Key; 14] = [
    Key::KeyQ, Key::KeyW, Key::KeyE, Key::KeyR, Key::KeyT, Key::KeyA, Key::KeyS, Key::KeyD, Key::KeyF, Key::KeyG, Key::KeyZ,
    Key::KeyX, Key::KeyC, Key::KeyV,
];

struct Row {
    class: usize,
    test: bool,
    symbols: Vec<u8>,
    weight: f64,
}

#[derive(Default)]
struct Typed {
    rows: Vec<Row>,
    dropped: usize,
    wrong_routes: usize,
}

/// What `presses` type in Mode A without predictions.
fn types(presses: &[Press]) -> String {
    let mut engine = Engine::new();
    let mut text = String::new();
    for p in presses {
        text += &engine.key(p.key, if p.shift { Mods::SHIFTED } else { Mods::NONE }).commit;
    }
    text + &engine.flush().commit
}

/// The planner's route with Shift+consonant for each consonant + ㅏ.
fn with_shift_consonants(presses: &[Press]) -> Option<Vec<Press>> {
    let mut out = Vec::with_capacity(presses.len());
    let mut changed = false;
    let mut i = 0;
    while i < presses.len() {
        let p = presses[i];
        if !p.shift && SHIFT_CONSONANTS.contains(&p.key) && presses.get(i + 1) == Some(&Press::new(Key::KeyK)) {
            out.push(Press { key: p.key, shift: true });
            changed = true;
            i += 2;
        } else {
            out.push(p);
            i += 1;
        }
    }
    changed.then_some(out)
}

fn main() {
    let t0 = Instant::now();
    let manifest = env!("CARGO_MANIFEST_DIR");
    let root = format!("{manifest}/../corpora/keygrams");
    let args: Vec<String> = std::env::args().collect();
    let out_path = args.iter().position(|a| a == "--out").map_or(format!("{manifest}/predict/ranker.bin"), |i| args[i + 1].clone());
    let text = std::fs::read_to_string(format!("{root}/words.tsv")).expect("run engine/tools/corpora/build_keygrams.py first");
    let keys: Vec<Key> = KEY_CODES.iter().map(|c| Key::from_code(c)).collect();
    let symbol_of = |p: &Press| keys.iter().position(|&k| k == p.key).map(|i| i as u8 + if p.shift { 48 } else { 0 });
    let symbols_of = |presses: &[Press]| presses.iter().map(symbol_of).collect::<Option<Vec<u8>>>();
    let press_of = |s: u8| Press { key: keys[(s % 48) as usize], shift: s >= 48 };
    let (symbols_of, press_of) = (&symbols_of, &press_of);

    // Every row as symbols, Korean typed and checked, in parallel.
    let lines: Vec<&str> = text.lines().collect();
    let threads = std::thread::available_parallelism().map_or(8, |n| n.get());
    let parts: Vec<Typed> = std::thread::scope(|s| {
        let handles: Vec<_> = lines
            .chunks(lines.len().div_ceil(threads))
            .map(|part| {
                s.spawn(move || {
                    let mut t = Typed::default();
                    for line in part {
                        let f: Vec<&str> = line.split('\t').collect();
                        let [cls, kind, payload, w] = f[..] else { continue };
                        let test = cls.starts_with("test-");
                        let class = CLASSES.iter().position(|c| *c == cls.trim_start_matches("test-")).unwrap();
                        let weight: f64 = w.parse().unwrap();
                        if kind == "keys" {
                            t.rows.push(Row { class, test, symbols: payload.split(',').map(|x| x.parse().unwrap()).collect(), weight });
                            continue;
                        }
                        let (word, explicit) = payload.split_once('|').unwrap();
                        let Some(cheapest) = keys_for_word(word).filter(|k| !k.is_empty() && types(k) == word) else {
                            t.dropped += 1;
                            continue;
                        };
                        let shifted = with_shift_consonants(&cheapest).filter(|k| types(k) == word);
                        let explicit: Option<Vec<Press>> = (!explicit.is_empty())
                            .then(|| explicit.split(',').map(|x| press_of(x.parse().unwrap())).collect::<Vec<_>>())
                            .filter(|k| types(k) == word);
                        if !test && explicit.is_none() {
                            t.wrong_routes += 1;
                        }
                        if test {
                            t.rows.extend(symbols_of(&cheapest).map(|symbols| Row { class, test, symbols, weight }));
                            continue;
                        }
                        let routes = [(explicit, 0.25), (shifted, 0.25), (Some(cheapest), 0.5)];
                        let found: f64 = routes.iter().filter(|(r, _)| r.is_some()).map(|(_, share)| share).sum();
                        for (route, share) in routes {
                            if let Some(symbols) = route.as_deref().and_then(symbols_of) {
                                t.rows.push(Row { class, test, symbols, weight: weight * share / found });
                            }
                        }
                    }
                    t
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let dropped: usize = parts.iter().map(|t| t.dropped).sum();
    let wrong: usize = parts.iter().map(|t| t.wrong_routes).sum();
    let rows: Vec<Row> = parts.into_iter().flat_map(|t| t.rows).collect();
    eprintln!(
        "{} sequences ({dropped} Korean words the planner can't type, {wrong} conventional routes that don't type their word) in {:.1}s",
        rows.len(),
        t0.elapsed().as_secs_f64()
    );

    // Counts per class and order: c[n][index of the n-gram], index = base-97 digits.
    let mut counts: Vec<[Vec<f32>; 5]> =
        (0..3).map(|_| [vec![0.0f32; 1], vec![0.0; A], vec![0.0; A * A], vec![0.0; A * A * A], vec![0.0; A * A * A * A]]).collect();
    for row in rows.iter().filter(|r| !r.test) {
        let c = &mut counts[row.class];
        let seq: Vec<usize> = std::iter::once(SPACE).chain(row.symbols.iter().copied()).chain([SPACE]).map(usize::from).collect();
        for i in 1..seq.len() {
            let mut index = 0;
            for n in 1..=4.min(i + 1) {
                index += seq[i + 1 - n] * A.pow(n as u32 - 1);
                c[n][index] += row.weight as f32;
            }
        }
    }

    // Witten-Bell, dense: p[n][h ++ w] for every history h of n − 1 symbols. The sparse model's
    // entries come from the same numbers.
    let byte = |p: f64, low: f64| (-16.0 * p.log2()).round().clamp(low, 255.0) as u8;
    let double_space = |mut i: usize, n: usize| {
        let mut last = usize::MAX;
        for _ in 0..n {
            if i % A == SPACE as usize && last == SPACE as usize {
                return true;
            }
            last = i % A;
            i /= A;
        }
        false
    };
    let mut dense: Vec<[Vec<u8>; 3]> = Vec::new();
    let mut sparse: Vec<(Vec<(u32, u8)>, Vec<(u32, u8)>)> = Vec::new();
    let digits = |mut i: usize, n: usize| {
        let mut s = vec![0u8; n];
        for d in s.iter_mut().rev() {
            *d = (i % A) as u8;
            i /= A;
        }
        s
    };
    for c in &counts {
        let (mut entries, mut histories) = (Vec::new(), Vec::new());
        let total: f64 = c[1].iter().map(|&x| x as f64).sum();
        let types = c[1].iter().filter(|&&x| x > 0.0).count() as f64;
        let mut p: Vec<f32> = c[1].iter().map(|&x| ((x as f64 + types / A as f64) / (total + types)) as f32).collect();
        for (w, &q) in p.iter().enumerate() {
            entries.push((key(&[w as u8]), byte(q as f64, 1.0)));
        }
        let mut tables: Vec<Vec<u8>> = Vec::new();
        for n in 2..=4 {
            let rows_n = A.pow(n as u32 - 1);
            let mut next = vec![0f32; rows_n * A];
            let (lower, cn) = (&p, &c[n]);
            let per = rows_n.div_ceil(threads);
            // Per history: (total, types) when seen.
            let seen: Vec<Vec<(usize, f64, f64)>> = std::thread::scope(|s| {
                let handles: Vec<_> = next
                    .chunks_mut(per * A)
                    .enumerate()
                    .map(|(part, out)| {
                        s.spawn(move || {
                            let mut seen = Vec::new();
                            for (r, slot) in out.chunks_mut(A).enumerate() {
                                let h = part * per + r;
                                let ch = &cn[h * A..(h + 1) * A];
                                let total: f64 = ch.iter().map(|&x| x as f64).sum();
                                let types = ch.iter().filter(|&&x| x > 0.0).count() as f64;
                                let below = &lower[(h % (rows_n / A)) * A..][..A];
                                for w in 0..A {
                                    slot[w] = if total > 0.0 { ((ch[w] as f64 + types * below[w] as f64) / (total + types)) as f32 } else { below[w] };
                                }
                                if total > 0.0 {
                                    seen.push((h, total, types));
                                }
                            }
                            seen
                        })
                    })
                    .collect();
                handles.into_iter().map(|h| h.join().unwrap()).collect()
            });
            for (h, total, types) in seen.into_iter().flatten() {
                histories.push((key(&digits(h, n - 1)), byte(types / (total + types), 0.0)));
                for w in 0..A {
                    if c[n][h * A + w] > 0.0 {
                        entries.push((key(&digits(h * A + w, n)), byte(next[h * A + w] as f64, 1.0)));
                    }
                }
            }
            tables.push(next.iter().enumerate().map(|(i, &q)| if double_space(i, n) { 0 } else { byte(q as f64, 1.0) }).collect());
            p = next;
        }
        entries.sort_unstable();
        histories.sort_unstable();
        sparse.push((entries, histories));
        dense.push(tables.try_into().unwrap());
    }
    eprintln!("smoothed every n-gram at {:.1}s", t0.elapsed().as_secs_f64());

    // The dense reference.
    let mut file = b"JKG4".to_vec();
    for x in [1u32, A as u32, 3] {
        file.extend_from_slice(&x.to_le_bytes());
    }
    dense.iter().flatten().for_each(|t| file.extend_from_slice(t));
    std::fs::write(format!("{root}/cost.bin"), &file).unwrap();
    eprintln!("wrote {root}/cost.bin ({} MB)", file.len() >> 20);

    // The sparse model, after the JKRK tables the output file starts with.
    let old = std::fs::read(&out_path).unwrap_or_default();
    // JKRK version 1 holds key trigrams, version 2 4-grams: two tables of 27^order bytes each.
    let base = match old.get(..6) {
        Some([b'J', b'K', b'R', b'K', 0, v @ (1 | 2)]) => &old[..(8 + 2 * 27usize.pow(*v as u32 + 2)).min(old.len())],
        _ => &[][..],
    };
    let mut model = Vec::new();
    model.extend_from_slice(b"JKR4");
    for x in [4u32, 4, A as u32, 3] {
        model.extend_from_slice(&x.to_le_bytes());
    }
    for (entries, histories) in &sparse {
        for list in [entries, histories] {
            model.extend_from_slice(&(list.len() as u32).to_le_bytes());
            list.iter().for_each(|(k, _)| model.extend_from_slice(&k.to_le_bytes()));
            list.iter().for_each(|(_, b)| model.push(*b));
        }
    }
    let parsed = KeyModel::parse(&model).expect("the written model parses");
    let mut out = base.to_vec();
    out.extend_from_slice(&model);
    std::fs::write(&out_path, &out).unwrap();
    for (name, (e, h)) in CLASSES.iter().zip(&sparse) {
        eprintln!("{name}: {} n-grams, {} histories", e.len(), h.len());
    }
    eprintln!("wrote {out_path}: {} MB, the key model {:.1} MB", out.len() >> 20, model.len() as f64 / 1048576.0);

    // The sparse model against the dense table, every valid n-gram of orders 2–4 under 255 (16
    // bits; past it the dense byte is clamped): the difference in 1/16 bits, from rounding each
    // backoff step on its own.
    let t3 = Instant::now();
    for (class, cls) in [Class::Korean, Class::English, Class::Kana].into_iter().enumerate() {
        let mut worst = 0i32;
        let mut sum = 0i64;
        let mut checked = 0u64;
        for n in 2..=4usize {
            let table = &dense[class][n - 2];
            let per = table.len().div_ceil(threads);
            let results: Vec<(i32, i64, u64)> = std::thread::scope(|s| {
                let handles: Vec<_> = table
                    .chunks(per)
                    .enumerate()
                    .map(|(part, chunk)| {
                        let parsed = &parsed;
                        s.spawn(move || {
                            let (mut worst, mut sum, mut checked) = (0i32, 0i64, 0u64);
                            for (j, &b) in chunk.iter().enumerate() {
                                // Two spaces, or at the byte's limit (the sparse sum may go past it).
                                if b == 0 || b == 255 {
                                    continue;
                                }
                                let d = digits(part * per + j, n);
                                let diff = parsed.cost(cls, &d[..n - 1], d[n - 1]) as i32 - b as i32;
                                worst = worst.max(diff.abs());
                                sum += diff.abs() as i64;
                                checked += 1;
                            }
                            (worst, sum, checked)
                        })
                    })
                    .collect();
                handles.into_iter().map(|h| h.join().unwrap()).collect()
            });
            for (w, s, c) in results {
                worst = worst.max(w);
                sum += s;
                checked += c;
            }
        }
        eprintln!(
            "{}: sparse against dense on {checked} n-grams under 16 bits: mean |difference| {:.4}, worst {} (1/16 bits)",
            CLASSES[class],
            sum as f64 / checked as f64,
            worst
        );
    }
    eprintln!("checked in {:.1}s", t3.elapsed().as_secs_f64());

    // Held-out: which class explains each word best (weighted by its count), from the shipped model.
    let classes = [Class::Korean, Class::English, Class::Kana];
    for (class, name) in CLASSES.iter().enumerate() {
        let (mut right, mut total, mut bits, mut presses) = (0.0, 0.0, 0.0, 0.0);
        for row in rows.iter().filter(|r| r.test && r.class == class) {
            let scores: Vec<f64> = classes.iter().map(|&k| parsed.log_probability(k, &row.symbols)).collect();
            let best = (0..3).max_by(|&a, &b| scores[a].total_cmp(&scores[b])).unwrap();
            right += row.weight * (best == class) as u8 as f64;
            total += row.weight;
            bits -= row.weight * scores[class];
            presses += row.weight * (row.symbols.len() + 1) as f64;
        }
        eprintln!("held-out {name}: {:.2}% read as {name} (weighted), {:.3} bits per key", 100.0 * right / total, bits / presses);
    }
    eprintln!("total {:.1}s", t0.elapsed().as_secs_f64());
}
