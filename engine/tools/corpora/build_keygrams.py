"""Lists the training words of the key model with Shift kept, for the dense 4-gram tables
(`cargo run --release --example keygrams`).

The same words and weights as build_keymodel.py (Korean, English, JIS kana), but each key press
keeps its Shift: 96 symbols (the 48 keys of build_keymodel.KEYS, then the same 48 with Shift) plus
96, the space between words. English keeps its case from the corpora (sentence-initial capitals
are typed too); kana takes Shift for the small kana, を and 「 」 、 。 ・ (docs/moonshot.md §3).
Korean is written as the word and its conventional route (explicit ㅇ, doubles on 1–5, ㅒ ㅖ on
: ;, no Shift); the example adds the cheapest JIS Moonshot way from the engine's own planner
(`plan::keys_for_word`: Shift+vowel, the implied ㅇ, the double press) and that way with
Shift+consonant for each consonant + ㅏ (docs/moonshot.md §9.2), and checks every route types the
word.

Writes corpora/keygrams/words.tsv: class (ko, en, ja; `test-` before held-out words), then
`keys` and comma-separated symbol ids, or `word` and `<Korean word>|<conventional route's ids>`,
then the weight.

Usage: python3 -I engine/tools/corpora/build_keygrams.py
"""

import math
import os
import random
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import build_keymodel as K  # noqa: E402

B = K.B
OUT = os.path.join(K.CORPORA, "keygrams")

# Kana typed with Shift on the JIS kana layout.
KANA_SHIFTED = set("ぁぃぅぇぉゃゅょっをゎゕゖ「」、。・")


def ids(keys, shifted=None):
    return ",".join(str(K.CODE[k] + (48 if shifted and shifted[i] else 0)) for i, k in enumerate(keys))


def english_ids(word):
    return ",".join(str(K.CODE[c.lower()] + (48 if c.isupper() else 0)) for c in word)


def kana_ids(word):
    out = []
    for c in K.unicodedata.normalize("NFD", K.hiragana(word)):
        k = K.KANA.get(c)
        if k is None:
            return None
        out.append(str(K.CODE[k] + (48 if c in KANA_SHIFTED else 0)))
    return ",".join(out)


def main():
    os.makedirs(OUT, exist_ok=True)
    corpora = K.CORPORA
    rng = random.Random(20261019)
    rows = []

    def add(cls, kind, payload, weight):
        if payload and weight > 0:
            rows.append(f"{cls}\t{kind}\t{payload}\t{weight:.6g}\n")

    # English: as build_keymodel, the corpora with case kept.
    nouns = set()
    with open(os.path.join(corpora, "wordnet", "WordNet-3.0", "dict", "index.noun"), encoding="utf-8") as f:
        for line in f:
            if not line.startswith(" "):
                lemma = line.split(" ", 1)[0]
                if lemma.isalpha() and lemma.isascii():
                    nouns.add(lemma.lower())
    with open(os.path.join(corpora, "dwyl", "words_alpha.txt"), encoding="utf-8") as f:
        english = [w.strip() for w in f if w.strip()]
    rng.shuffle(english)
    en_test, en_train = english[: len(english) // 10], english[len(english) // 10:]
    alpha = re.compile(r"[A-Za-z]+")
    for w in en_train:
        if alpha.fullmatch(w):
            add("en", "keys", english_ids(w.lower()), B.NOUN_WEIGHT if w.lower() in nouns else 1.0)
    for w in B.COMMON_EN:
        if alpha.fullmatch(w):
            add("en", "keys", english_ids(w), 200.0)
    en_counts, en_held = B.count_tokens(B.english_sentences(corpora), B.ENGLISH_TOKEN, lambda t: t)
    for w, n in en_counts.items():
        if w.isascii():
            add("en", "keys", english_ids(w), math.log2(1 + n))
    biz_ko, biz_en = B.business_terms(corpora)
    for w in biz_en:
        if alpha.fullmatch(w):
            add("en", "keys", english_ids(w), B.BUSINESS_WEIGHT)
    for w, n in en_held.items():
        if w.isascii():
            add("test-en", "keys", english_ids(w), n)

    # Korean: both routes.
    ko_counts, ko_held = B.count_tokens(B.korean_sentences(corpora), B.KOREAN_TOKEN, lambda t: t)
    ko_words = [(w, 200.0) for w in B.COMMON_KO]
    ko_words += [(w, math.log2(1 + n)) for w, n in ko_counts.items()]
    ko_words += [(w, B.BUSINESS_WEIGHT) for w in biz_ko]
    for w, weight in ko_words:
        k = K.korean_keys(w, False)
        if k:
            add("ko", "word", f"{w}|{ids(k)}", weight)
    for w, n in ko_held.items():
        if any("가" <= ch <= "힣" for ch in w):
            add("test-ko", "word", f"{w}|", n)

    # Kana: Mozc readings as build_keymodel (best cost per reading, 1 in 10 held out).
    best = {}
    for path in sorted(K.glob.glob(os.path.join(K.MOZC, "dictionary0*.txt"))):
        with open(path, encoding="utf-8") as f:
            for line in f:
                r, _, _, c = line.split("\t", 4)[:4]
                c = int(c)
                if r not in best or c < best[r]:
                    best[r] = c
    assert best, f"no Mozc dictionary in {K.MOZC}: run engine/tools/kanji/build_kanji.py first"
    particles, common_ja = [], []
    with open(os.path.join(K.PREDICT, "common-ja.txt"), encoding="utf-8") as f:
        for line in f:
            if line.startswith("#!particles"):
                particles += line.split()[1:]
            elif line.strip() and not line.startswith("#"):
                common_ja.append(line.split("\t")[0].strip())
    ja_rng = random.Random(20261102)
    for r, c in sorted(best.items()):
        k = kana_ids(r)
        if not k:
            continue
        if B.held_out(r):
            add("test-ja", "keys", k, 1)
            continue
        w = max(0.25, (9500 - c) / 347)
        add("ja", "keys", k, w)
        if ja_rng.random() < 0.3:
            add("ja", "keys", kana_ids(r + ja_rng.choice(particles)), w * 0.5)
    for w in common_ja:
        add("ja", "keys", kana_ids(w), 200.0)
    for p in particles:
        add("ja", "keys", kana_ids(p), 50.0)

    path = os.path.join(OUT, "words.tsv")
    with open(path, "w", encoding="utf-8") as f:
        f.writelines(rows)
    print(f"{len(rows)} rows → {path}")


if __name__ == "__main__":
    main()
