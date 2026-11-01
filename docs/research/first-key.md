# A first-key prior: which language a word's first key press starts

Research note, 2026-11-02. Data: [`first-key.tsv`](first-key.tsv), made by
`cargo run --release --example first_key > ../docs/research/first-key.tsv` in `engine/`
(`engine/examples/first_key.rs`). Not used by the engine.

## Question

Would a weight that depends on the staged text's first key press (how likely each language is to
start on that key) help JIS Moonshot rank its Korean, English and kana readings?

## Method

- **Words:** the 5,000 likeliest of each language. Korean from `predict/common-ko.txt` (Hangul
  words), English from `predict/common-en.txt`, both by count; Japanese half from
  `predict/common-ja.txt` and half from the common words of Mozc's dictionary (`predict/kanji.bin`,
  hiragana words, cost ≤ 4500, weighted by e^(−cost / 500)). Each language's words share its weight
  by frequency.
- **First presses and weights** (the owner's, 2026-11-02):
  - Korean typed efficiently, the robot's cheapest route (implied ㅇ, the dedicated keys,
    Shift+vowel): **2**.
  - Korean typed diligently, 초성 first (ㅇ typed, the doubles on 1–5): **1**, only when that first
    press differs from the efficient one. When they're the same press the word counts once, at 2.
  - English, the first letter: **2**.
  - Japanese, the first kana's key on the JIS kana layout (が on か's key, small kana with Shift):
    **2**.
- The table gives each press's weight per language and the language given the press.

## Results

| | |
|---|---|
| Korean efficient = diligent first press | 72.8% of the Korean weight (27.2% differ: mostly ㅇ-initial words, D against the vowel key) |
| Prior over the three languages | ko 0.362, en 0.319, ja 0.319: 1.58 bits |
| Uncertainty left after the first press | 1.24 bits: the first press tells **0.34 bits, 21%** |
| Weight starting on a press that's ≥ 80% one language | **9%** |

The decisive presses are nearly all kana-only starts: the number row (お 6, あ 3, ふ 2, や 7, ゆ 8,
よ 9, ほ −), the keys right of 0 (ね , る . め / へ ^ ろ ろ む ]), Shift+Z (っ) and Shift+0 (を).
Since 2026-11-02 those keys also type Korean (docs/moonshot.md §9), so they're kana-leaning rather
than kana-only. On the letter keys the three languages overlap heavily (T: ko 0.26 en 0.50 ja 0.23;
D: 0.54 / 0.27 / 0.19).

## Is it useful as a first-key weight?

**For Korean against English: no.** Their key models (`predict/ranker.bin`) already score every
word from a start-of-word context: `log_probability` pads the history with `START`, so
P(first key | language) is part of each model's score. A separate first-key weight would count that
evidence twice.

**For kana: only as a stopgap.** The kana reading has no key model; it's scored from word lists.
A first-key prior would add some of what a kana key model knows (that お on 6 or ね on , starts
Japanese) but only 0.34 bits, against whole-word evidence (a known word is worth up to 5 bits).
A kana key n-gram model, under test at the time of writing, would include the first key the same way
the Korean and English models do, and much more.

**Where it would matter:** one- and two-key segments, where whole-word evidence is thin. That's
also where readings are rarely worth predicting.

**Recommendation:** don't add a first-key weight. Fold the first key into a kana key model (if
one ships), which conditions on the word start like the other two. If context-dependent weights
are wanted, the previous word's language (spec §10.3) carries more than the first key: the
first key leaves 1.24 of 1.58 bits.
