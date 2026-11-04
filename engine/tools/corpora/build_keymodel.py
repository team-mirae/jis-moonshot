"""Builds ranker.bin in format JKR3: JIS Moonshot's three-language key model (Korean, English,
JIS kana), one order-4 key n-gram table per language, all on the same 48 physical keys.

A staged run of keys is read three ways at once (docs/moonshot.md); this model says how well
each language explains the keys pressed. Every class is trained on the keys a typist presses:

- Korean (class 0): words typed in 2-set on JIS Moonshot's keys (docs/moonshot.md §9): ㅃ ㅉ ㄸ ㄲ ㅆ
  on 1–5, ㅒ ㅖ on `:` `;`, with an explicit ㅇ, the cheapest JIS way (implied ㅇ, the double
  press), a seeded mix of the valid ways, and that mix with the slot keys (§9.2) half the time.
  Words: common-ko.txt (hand-written part), the colloquial corpora (log2(1 + count)) and the
  business terms, as build_ranker.py.
- English (class 1): dwyl words_alpha (9 in 10, seeded split; WordNet nouns ×3), common-en.txt,
  corpus words (log2(1 + count)) and business terms, as build_ranker.py.
- Kana (class 2): the readings of Mozc's dictionary (engine/tools/kanji/mozc, fetched by
  build_kanji.py; best cost per reading, weight max(0.25, (9500 − cost) / 347), 1 in 10 held out
  by hash, 3 in 10 also trained with a particle after them at half weight), common-ja.txt and its
  particles, typed on the JIS kana layout (§3) with Shift dropped (゛ is `@`, ゜ is `[`).

Shift is dropped everywhere: the 48 symbols are KEYS below (`\\` is the ¥ key, `_` the ろ key),
plus 48, the word boundary. Witten-Bell interpolated, order 4; n-grams of order 3 and 4 lighter
than weight 1 are pruned (the three-way experiment found no loss).

Format (little-endian):
  "JKR3", u32 version 3, u32 order 4, u32 alphabet 49, u32 classes 3; then per class (Korean,
  English, kana): u32 n, n × u32 key (ascending), n × u8 cost; u32 m, m × u32 history key
  (ascending), m × u8 backoff.
  key of symbols s_1..s_k = (k − 1) × 49^4 + fold(0, i → i × 49 + s). An entry is history ++ symbol
  (orders 1..4, all 49 unigrams present); cost = round(16 × −log2 P(w | h)), clamped 1..255, P
  interpolated. A history (1..3 symbols) stores round(16 × −log2(T(h) / (c(h) + T(h)))), clamped
  0..255. Scoring (`score_keys`): history starts as three boundaries; each key and the final
  boundary costs its entry if present, else backoff(h) (0 if absent) + the cost after h without
  its oldest symbol.

The shipped engine/predict/ranker.bin is the two-language JKRK tables (build_ranker.py, which still
score Korean against English: the rules are tuned on them) followed by this model, used for the
kana reading. Rebuilding keeps whatever JKRK tables the file starts with (or --base's).

Usage (one command, from the repository; reads the gitignored corpora/ fetched by
fetch_corpora.py, dwyl words_alpha.txt and WordNet in corpora/, and the Mozc cache):
  python3 -I engine/tools/corpora/build_keymodel.py [--out <dir>] [--eval]
      writes <dir>/ranker.bin (default engine/predict/ranker.bin); --eval also prints held-out
      3-way accuracy, from the float model and from the written file.
  python3 -I engine/tools/corpora/build_keymodel.py --score "<keys>" [--model <ranker.bin>]
      the three costs of a key string (bits), read back from the file: a reference for the engine.
Deterministic: fixed seeds and hash splits, sorted output; rebuilding gives identical bytes.
Rerun engine/tools/train_learned.py after ranker.bin changes.
"""

import argparse
import glob
import math
import os
import random
import re
import struct
import sys
import time
import unicodedata

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CORPORA = os.path.join(REPO, "corpora")
MOZC = os.path.join(REPO, "engine", "tools", "kanji", "mozc")
PREDICT = os.path.join(REPO, "engine", "predict")

ORDER = 4
KEYS = "abcdefghijklmnopqrstuvwxyz0123456789-^\\@[;:],./_"  # ¥ as \, ろ as _
CODE = {k: i for i, k in enumerate(KEYS)}
BOUNDARY = len(KEYS)  # 48
SIZE = len(KEYS) + 1  # 49
CLASSES = ["ko", "en", "ja"]  # file order: 0 Korean, 1 English, 2 kana
PRUNE = 1.0  # n-grams of order >= PRUNE_FROM lighter than this are dropped
PRUNE_FROM = 3
MAGIC, VERSION = b"JKR3", 3

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


# --- Data ---

def load(corpora=CORPORA, with_tests=False):
    """Training data {class: [(keys, weight)]}, info, and (with_tests) the held-out tests
    {name: (class, [(keys, count, word)])}."""
    t0 = time.time()
    rng = random.Random(20261019)
    data = {c: [] for c in CLASSES}

    # English, as build_ranker: dwyl split, nouns ×3, common-en, corpora, business terms.
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
    data["en"] += [(w, B.NOUN_WEIGHT if w in nouns else 1.0) for w in en_train]
    data["en"] += [(w, 200.0) for w in B.COMMON_EN]
    ko_counts, ko_held = B.count_tokens(B.korean_sentences(corpora), B.KOREAN_TOKEN, lambda t: t)
    en_counts, en_held = B.count_tokens(B.english_sentences(corpora), B.ENGLISH_TOKEN, str.lower)
    biz_ko, biz_en = B.business_terms(corpora)
    data["en"] += [(w, math.log2(1 + n)) for w, n in en_counts.items() if w.isascii()]
    data["en"] += [(w, B.BUSINESS_WEIGHT) for w in biz_en]
    data["en"] = [(w.lower(), x) for w, x in data["en"] if re.fullmatch(r"[a-zA-Z]+", w)]

    # Korean: common-ko (hand-written), corpora, business terms; build_ranker's four routes, the
    # last with Moonshot's slot keys half the time.
    ko_words = [(w, 200.0) for w in B.COMMON_KO]
    ko_words += [(w, math.log2(1 + n)) for w, n in ko_counts.items()]
    ko_words += [(w, B.BUSINESS_WEIGHT) for w in biz_ko]
    route_rng = random.Random(20261020)
    routes = [(False, None, 0.0, 0.25), (True, None, 0.0, 0.35), (True, route_rng, 0.0, 0.2), (True, route_rng, 0.5, 0.2)]
    for w, weight in ko_words:
        for jis, r, slots, share in routes:
            k = korean_keys(w, jis, r, slots)
            if k:
                data["ko"].append((k, weight * share))

    # Kana: Mozc readings (best cost per reading), 1 in 10 held out by hash; common-ja.
    best = {}
    for path in sorted(glob.glob(os.path.join(MOZC, "dictionary0*.txt"))):
        with open(path, encoding="utf-8") as f:
            for line in f:
                r, _, _, c = line.split("\t", 4)[:4]
                c = int(c)
                if r not in best or c < best[r]:
                    best[r] = c
    assert best, f"no Mozc dictionary in {MOZC}: run engine/tools/kanji/build_kanji.py first"
    particles, common_ja = [], []
    with open(os.path.join(PREDICT, "common-ja.txt"), encoding="utf-8") as f:
        for line in f:
            if line.startswith("#!particles"):
                particles += line.split()[1:]
            elif line.strip() and not line.startswith("#"):
                common_ja.append(line.split("\t")[0].strip())
    ja_rng = random.Random(20261102)
    ja_held = {}
    mozc_train = 0
    for r, c in sorted(best.items()):
        k = kana_keys(r)
        if not k:
            continue
        if B.held_out(r):
            ja_held[r] = c
            continue
        w = max(0.25, (9500 - c) / 347)  # Mozc cost ≈ -500 ln p: log2-count-like
        data["ja"].append((k, w))
        mozc_train += 1
        if ja_rng.random() < 0.3:  # a particle after it, as a typed segment often has
            kp = kana_keys(r + ja_rng.choice(particles))
            if kp:
                data["ja"].append((kp, w * 0.5))
    data["ja"] += [(kana_keys(w), 200.0) for w in common_ja if kana_keys(w)]
    data["ja"] += [(kana_keys(p), 50.0) for p in particles if kana_keys(p)]

    ko_types = [w for w in ko_held if any("가" <= ch <= "힣" for ch in w)]
    info = {"ko corpus types": len(ko_counts), "ko business": len(biz_ko), "ko common (hand)": len(B.COMMON_KO),
            "en dwyl train": len(en_train), "en corpus types": len(en_counts), "en business": len(biz_en),
            "ja Mozc readings train": mozc_train, "ja Mozc held out": len(ja_held), "ja common": len(common_ja),
            "sequences (count, weight)": {c: (len(d), round(sum(x for _, x in d), 1)) for c, d in data.items()}}
    print(f"data loaded in {time.time() - t0:.0f}s: {info}", flush=True)
    if not with_tests:
        return data, info, None

    # Tests, the three-way experiment's (fixed hash splits and seeded routes).
    tests = {}
    mix = random.Random(7)
    tests["ko: corpus eojeols, JIS cheapest"] = ("ko", [(korean_keys(w, True), ko_held[w], w) for w in ko_types])
    tests["ko: corpus eojeols, explicit ㅇ"] = ("ko", [(korean_keys(w, False), ko_held[w], w) for w in ko_types])
    tests["ko: corpus eojeols, mixed + slots"] = ("ko", [(korean_keys(w, True, mix, 0.5), ko_held[w], w) for w in ko_types])
    seen_ko = set(ko_counts) | set(B.COMMON_KO) | set(biz_ko)
    unseen_ko = [w for w in ko_types if w not in seen_ko]
    tests["ko: unseen eojeols, JIS cheapest"] = ("ko", [(korean_keys(w, True), ko_held[w], w) for w in unseen_ko])
    tests["en: dwyl held-out words"] = ("en", [(w, 1, w) for w in en_test])
    seen_en = set(en_counts) | set(en_train) | set(B.COMMON_EN) | set(biz_en)
    tests["en: corpus unseen words"] = ("en", [(w, n, w) for w, n in en_held.items() if w.isascii() and w not in seen_en])
    tests["en: corpus words"] = ("en", [(w, n, w) for w, n in en_held.items() if w.isascii()])
    tests["ja: Mozc held-out readings"] = ("ja", [(kana_keys(r), 1, r) for r in ja_held])
    tests["ja: Mozc held-out, cost<=4500"] = ("ja", [(kana_keys(r), 1, r) for r, c in ja_held.items() if c <= 4500])
    seen_common = sorted(r for r, c in best.items() if c <= 4500 and not B.held_out(r))
    tests["ja: Mozc trained readings, cost<=4500"] = ("ja", [(kana_keys(r), 1, r) for r in random.Random(11).sample(seen_common, 4000)])
    evals = []
    with open(os.path.join(MOZC, "evaluation.tsv"), encoding="utf-8") as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if len(parts) > 2 and not line.startswith("#") and kana_keys(parts[1]):
                evals.append(parts[1])
    tests["ja: Mozc evaluation phrases"] = ("ja", [(kana_keys(r), 1, r) for r in sorted(set(evals))])
    tests["ja: common-ja.txt (trained)"] = ("ja", [(kana_keys(w), 1, w) for w in common_ja if kana_keys(w)])
    for name in list(tests):
        cls, items = tests[name]
        tests[name] = (cls, [(k, n, w) for k, n, w in items if k])
    return data, info, tests


# --- Model ---

def train(data):
    """{class: KeyModel} at ORDER, pruned (orders >= PRUNE_FROM lighter than PRUNE dropped)."""
    models = {}
    for c in CLASSES:
        m = B.KeyModel(ORDER, SIZE, BOUNDARY)
        for keys, w in data[c]:
            m.add([CODE[k] for k in keys], w)
        for n in range(PRUNE_FROM, ORDER + 1):
            m.counts[n] = {k: w for k, w in m.counts[n].items() if w >= PRUNE}
        models[c] = m.finish()
    return models


def unpack(x, n):
    """KeyModel's packed int (6 bits per symbol, oldest first) as a list of n symbols."""
    return [(x >> (B.KeyModel.BITS * (n - 1 - i))) & 63 for i in range(n)]


def encode(symbols):
    """The file's key for a sequence of 1..ORDER symbols."""
    idx = 0
    for s in symbols:
        idx = idx * SIZE + s
    return (len(symbols) - 1) * SIZE ** ORDER + idx


def cost_byte(p, low):
    return max(low, min(255, round(-16 * math.log2(p))))


def tables(m):
    """(entries {key: cost}, backoffs {key: byte}) for one class."""
    bits = B.KeyModel.BITS
    entries, backoffs = {}, {}
    for n in range(1, ORDER + 1):
        for k in m.counts[n]:
            ctx, c = k >> bits, k & 63
            entries[encode(unpack(ctx, n - 1) + [c])] = cost_byte(m.prob(ctx, c, order=n), 1)
        if n >= 2:
            for ctx, t in m.total[n].items():
                y = m.types[n][ctx]
                backoffs[encode(unpack(ctx, n - 1))] = cost_byte(y / (t + y), 0)
    for c in range(SIZE):  # every unigram, so a lookup always ends
        if c not in entries:
            entries[c] = cost_byte(m.prob(0, c, order=1), 1)
    return entries, backoffs


# The two-language tables (format JKRK, build_ranker.py) the shipped ranker.bin starts with: the
# engine keeps scoring Korean against English with them and uses this model for the kana reading
# (docs/moonshot.md §5). Their size per order, header included.
JKRK_SIZES = {1: 8 + 2 * 27 ** 3, 2: 8 + 2 * 27 ** 4}


def base_tables(path):
    """The JKRK tables at the start of `path`, if it starts with them; else b""."""
    try:
        with open(path, "rb") as f:
            data = f.read()
    except FileNotFoundError:
        return b""
    if data[:4] != b"JKRK":
        return b""
    size = JKRK_SIZES.get(data[5])
    assert size and len(data) >= size, "a JKRK file of unknown size"
    return data[:size]


def write(path, models, base=b""):
    """Writes `base` (JKRK tables, or nothing) then the JKR3 model."""
    out = bytearray(base) + MAGIC
    out += struct.pack("<4I", VERSION, ORDER, SIZE, len(CLASSES))
    counts = {}
    for c in CLASSES:
        entries, backoffs = tables(models[c])
        counts[c] = (len(entries), len(backoffs))
        for table in (entries, backoffs):
            keys = sorted(table)
            out += struct.pack("<I", len(keys))
            out += struct.pack(f"<{len(keys)}I", *keys)
            out += bytes(table[k] for k in keys)
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    with open(path, "wb") as f:
        f.write(out)
    return counts


# --- Reference scorer: reads the file back and scores exactly as the engine should ---

def read(path):
    """[(entries {key: cost}, backoffs {key: byte})] per class, in file order."""
    with open(path, "rb") as f:
        data = f.read()
    data = data[len(base_tables(path)):]  # the JKR3 model after any JKRK tables
    assert data[:4] == MAGIC, "not a JKR3 file"
    version, order, alphabet, classes = struct.unpack_from("<4I", data, 4)
    assert (version, order, alphabet) == (VERSION, ORDER, SIZE), (version, order, alphabet)
    pos, out = 20, []
    for _ in range(classes):
        tabs = []
        for _ in range(2):
            (n,) = struct.unpack_from("<I", data, pos)
            keys = struct.unpack_from(f"<{n}I", data, pos + 4)
            vals = data[pos + 4 + 4 * n: pos + 4 + 5 * n]
            assert all(a < b for a, b in zip(keys, keys[1:]))
            tabs.append(dict(zip(keys, vals)))
            pos += 4 + 5 * n
        out.append(tuple(tabs))
    assert pos == len(data)
    return out


def symbol_cost(table, h, w):
    """Cost (1/16 bits) of symbol w after history h (a list, oldest first, up to ORDER-1)."""
    entries, backoffs = table
    total = 0
    while True:
        k = encode(h + [w])
        if k in entries:
            return total + entries[k]
        total += backoffs.get(encode(h), 0)  # h is never empty here: unigrams are all present
        h = h[1:]


def score_keys(table, keys):
    """Cost of a key string (1/16 bits): START, every key, then END."""
    h, total = [BOUNDARY] * (ORDER - 1), 0
    for w in [CODE[k] for k in keys.lower()] + [BOUNDARY]:
        total += symbol_cost(table, h, w)
        h = (h + [w])[1:]
    return total


# --- Evaluation (--eval) ---

def evaluate(models, path, tests):
    file_tables = read(path)
    lines = ["| test | weighting | float model | file (engine) |", "|---|---|---|---|"]
    confusion = {}
    for name, (cls, items) in tests.items():
        ok_f = {False: 0, True: 0}
        ok_b = {False: 0, True: 0}
        tot = {False: 0, True: 0}
        for keys, n, _ in items:
            codes = [CODE[k] for k in keys]
            lp = [models[c].logprob(codes) for c in CLASSES]
            pf = CLASSES[max(range(3), key=lambda i: lp[i])]
            sc = [score_keys(t, keys) for t in file_tables]
            pb = CLASSES[min(range(3), key=lambda i: sc[i])]
            if pb != cls:
                key = (name, cls, pb)
                confusion[key] = confusion.get(key, 0) + 1
            for weighted in (False, True):
                x = n if weighted else 1
                tot[weighted] += x
                ok_f[weighted] += x * (pf == cls)
                ok_b[weighted] += x * (pb == cls)
        for weighted in (False, True):
            if weighted and not name.startswith(("ko: corpus", "en: corpus", "ko: unseen")):
                continue
            lines.append(f"| {name} (n={len(items)}) | {'tokens' if weighted else 'types'} | "
                         f"{ok_f[weighted] / tot[weighted]:.1%} | {ok_b[weighted] / tot[weighted]:.1%} |")
    lines += ["", "file (engine), types: true -> predicted", ""]
    for (name, t, p), n in sorted(confusion.items()):
        lines.append(f"  {t}->{p} [{name}]: {n / len(tests[name][1]):.2%}")
    print("\n".join(lines), flush=True)


EXAMPLES = [("rkrj", "가거"), ("hello", "hello"), ("s@94v@", "どようび"), ("byiaf", "こんにちは"), ("gksrmf", "한글")]


def print_scores(path, examples):
    file_tables = read(path)
    print("costs in bits (lower = more likely):        Korean  English     kana")
    for keys, gloss in examples:
        sc = [score_keys(t, keys) / 16 for t in file_tables]
        print(f"  {keys:<10} {gloss:<12} " + " ".join(f"{s:8.3f}" for s in sc)
              + f"   -> {CLASSES[min(range(3), key=lambda i: sc[i])]}")


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--out", default=PREDICT, help="directory for ranker.bin (default engine/predict)")
    ap.add_argument("--corpora", default=CORPORA)
    ap.add_argument("--eval", action="store_true", help="also print held-out 3-way accuracy")
    ap.add_argument("--score", help="score a key string with an existing file and exit")
    ap.add_argument("--model", help="the file --score reads (default <out>/ranker.bin)")
    ap.add_argument("--base", help="JKRK tables to put first (default: the ones <out>/ranker.bin "
                    "starts with, if any; build_ranker.py writes them)")
    args = ap.parse_args()
    path = os.path.join(args.out, "ranker.bin")
    if args.score is not None:
        print_scores(args.model or path, [(args.score, "")])
        return
    data, info, tests = load(args.corpora, with_tests=args.eval)
    t0 = time.time()
    models = train(data)
    base = base_tables(args.base or path)
    counts = write(path, models, base)
    if base:
        print(f"kept the {len(base):,}-byte JKRK tables before the model", flush=True)
    print(f"{path}: {os.path.getsize(path):,} bytes, trained in {time.time() - t0:.0f}s; "
          f"(entries, histories) per class: {counts}", flush=True)
    print_scores(path, EXAMPLES)
    if args.eval:
        evaluate(models, path, tests)


if __name__ == "__main__":
    main()
