"""Builds the key model in ranker.bin, format JKR4: JIS Moonshot's three-language key model
(Korean, English, JIS kana), one order-4 key n-gram table per language, all on the same 48
physical keys with and without Shift (docs/moonshot.md §5).

A staged run of keys is read three ways at once (docs/moonshot.md); this model says how well
each language explains the keys pressed. Every class is trained on the keys a typist presses:

- Korean (class 0): words typed on JIS Moonshot's keys (docs/moonshot.md §9) three ways, each
  checked to type the word through the engine: the conventional route (explicit ㅇ, ㅃ ㅉ ㄸ ㄲ ㅆ on
  1–5, ㅒ ㅖ on `:` `;`; a quarter of the weight), the cheapest way from the engine's planner
  (implied ㅇ, Shift+vowel, the double press; half) and that way with Shift+consonant for each
  consonant + ㅏ (a quarter). Words: common-ko.txt (hand-written part), the colloquial corpora
  (log2(1 + count)) and the business terms, as build_ranker.py.
- English (class 1): dwyl words_alpha (9 in 10, seeded split; WordNet nouns ×3), common-en.txt,
  corpus words with their case (log2(1 + count)) and business terms, as build_ranker.py.
- Kana (class 2): the readings of Mozc's dictionary (engine/tools/kanji/mozc, fetched by
  build_kanji.py; best cost per reading, weight max(0.25, (9500 − cost) / 347), 1 in 10 held out
  by hash, 3 in 10 also trained with a particle after them at half weight), common-ja.txt and its
  particles, typed on the JIS kana layout (§3): Shift for the small kana, を and 「 」 、 。 ・.

97 symbols: the 48 keys of KEYS below (`\\` is the ¥ key, `_` the ろ key), the same 48 with
Shift (48–95), and 96, the space before and after each word (never two in a row). Witten-Bell
interpolated, order 4, nothing pruned.

Two steps, both run here: build_keygrams.py lists the words with their keys and weights
(corpora/keygrams/words.tsv), and `cargo run --release --example keygrams` (engine/examples/
keygrams.rs) types the Korean ones, counts every n-gram, smooths every one of the 88,501,248
possible 4-grams densely (the reference, corpora/keygrams/cost.bin, 255 MB, gitignored), writes
the sparse model from the same numbers, checks it against the dense table on every n-gram and
prints the held-out three-way accuracy. The file format is in engine/src/keymodel.rs.

The shipped engine/predict/ranker.bin is the two-language JKRK tables (build_ranker.py, which still
score Korean against English: the rules are tuned on them) followed by this model, used for the
kana reading. Rebuilding keeps the JKRK tables the file starts with.

Usage (from the repository; reads the gitignored corpora/ fetched by fetch_corpora.py, dwyl
words_alpha.txt and WordNet in corpora/, and the Mozc cache):
  python3 -I engine/tools/corpora/build_keymodel.py [--out <file>]
      writes <file> (default engine/predict/ranker.bin). About 20 seconds.
Deterministic: fixed seeds and hash splits, sorted output; rebuilding gives identical bytes.
"""

import argparse
import glob
import os
import subprocess
import sys
import unicodedata

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CORPORA = os.path.join(REPO, "corpora")
MOZC = os.path.join(REPO, "engine", "tools", "kanji", "mozc")
PREDICT = os.path.join(REPO, "engine", "predict")

KEYS = "abcdefghijklmnopqrstuvwxyz0123456789-^\\@[;:],./_"  # ¥ as \, ろ as _
CODE = {k: i for i, k in enumerate(KEYS)}

_argv, sys.argv = sys.argv, [sys.argv[0]]  # build_ranker reads its own flags at import
sys.path.insert(0, HERE)
import build_ranker as B  # noqa: E402
sys.argv = _argv

# --- Korean on JIS Moonshot's physical keys (docs/moonshot.md §9) ---

DOUBLES = {"ㄲ": "4", "ㄸ": "3", "ㅃ": "1", "ㅆ": "5", "ㅉ": "2"}
INIT = {j: DOUBLES.get(j, k.lower()) for j, k in B.INITIAL_KEYS.items()}
MED = {j: {"ㅒ": ":", "ㅖ": ";"}.get(j, k.lower()) for j, k in B.MEDIAL_KEYS.items()}
FIN = {j: DOUBLES.get(j, k.lower()) for j, k in B.FINAL_KEYS.items()}
JAMO = {**INIT, **MED, **FIN}
# Slot keys (§9.2, 2026-11-05: only − ; : , type Korean), Shift dropped: a syllable on a real ㅇ
# (Shift+− 왜, Shift+; 예, Shift+: 얘), and the bare vowels.
SLOT_SYLLABLE = {("ㅇ", "ㅙ"): "-", ("ㅇ", "ㅖ"): ";", ("ㅇ", "ㅒ"): ":"}
SLOT_MEDIAL = {"ㅙ": "-", "ㅢ": ","}


def korean_keys(word, jis, rng=None, slots=0.0):
    """build_ranker.korean_keys on Moonshot's keys: doubles on 1-5, ㅒ ㅖ on : ;, and with
    `slots` > 0 each slot key used where it applies with that probability."""
    keys, prev = [], None
    for initial, medial, final in B.syllables(word):
        if initial is None:
            keys.append(JAMO[final])
            prev = None
            continue
        use = slots and rng and rng.random() < slots
        if use and not final and (initial, medial) in SLOT_SYLLABLE:
            keys.append(SLOT_SYLLABLE[(initial, medial)])
            prev = (initial, medial, final)
            continue
        vowel = MED[medial]
        if initial == "ㅇ" and jis:
            if prev and prev[2]:
                keys.append(rng.choice([vowel[0], "", "d"]) if rng else vowel[0])
            elif prev and (prev[1], vowel[0]) in B.COMBINES:
                keys.append(rng.choice(["d", ""]) if rng else "d")
            elif rng and rng.random() < 0.3:
                keys.append("d")
        else:
            keys.append(INIT[initial])
        keys.append(SLOT_MEDIAL[medial] if use and medial in SLOT_MEDIAL else vowel)
        if final:
            keys.append(FIN[final])
        prev = (initial, medial, final)
    return "".join(keys)


# --- Kana on the JIS kana layout (docs/moonshot.md §3), Shift dropped ---

KANA = dict(zip("ぬふあうえおやゆよわほへー" "たていすかんなにらせ" "ちとしはきくまのりれけむ" "つさそひこみもねるめろ",
                "1234567890-^\\" "qwertyuiop" "asdfghjkl;:]" "zxcvbnm,./_"))
KANA.update(dict(zip("ぁぃぅぇぉゃゅょっをゎゕゖ", "3e456789z00t:")))
KANA.update({"゙": "@", "゚": "[", "、": ",", "。": ".", "・": "/", "「": "[", "」": "]"})
assert set(KANA.values()) <= set(KEYS)


def hiragana(s):
    return "".join(chr(ord(c) - 0x60) if "ァ" <= c <= "ヶ" else c for c in s)


def kana_keys(word):
    """The JIS kana keys for a kana word (voiced kana as kana + ゛), or None."""
    out = []
    for c in unicodedata.normalize("NFD", hiragana(word)):
        k = KANA.get(c)
        if k is None:
            return None
        out.append(k)
    return "".join(out)


# --- Build ---

def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--out", default=os.path.join(PREDICT, "ranker.bin"), help="the ranker.bin to write")
    args = ap.parse_args()
    import build_keygrams  # imports this file

    build_keygrams.main()
    cargo = os.path.expanduser("~/.cargo/bin/cargo" + (".exe" if os.name == "nt" else ""))
    if not os.path.exists(cargo):
        cargo = "cargo"
    subprocess.run([cargo, "run", "--release", "--example", "keygrams", "--", "--out", os.path.abspath(args.out)],
                   cwd=os.path.join(REPO, "engine"), check=True)


if __name__ == "__main__":
    main()
