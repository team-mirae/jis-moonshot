#!/usr/bin/env python3
"""Builds engine/predict/punctuation.tsv (docs/moonshot.md §5.6): what follows a Korean
syllable, a punctuation mark or one of the syllables the symbol keys type, from the open corpora.

    python3 -I engine/tools/corpora/build_punctuation.py [--cap-mb N] [--min-count N]

Korean punctuation follows the ending closely: a final ending (다, 요, 까, 니다) takes . or ?, a
connective one (고, 며, 는데, 지만) a comma, a particle (을, 를, 의) almost never anything. Mode A's
symbol keys type syllables too (. is 을, / 서, , ㅢ, docs/moonshot.md §9.2), so the same counts
decide between 안녕하세요. and 안녕하세요을. For every syllable of the Korean texts
(corpora/derived/*/ko.txt, read up to --cap-mb MB a source, scaled by its meta.json weight) the
context is that syllable and the two ending there; what comes right after it is counted when it
is a mark in MARKS or a syllable in NEXT (a word's end before a space isn't either). Lines of the
evaluation texts are left out (as engine/tools/train_learned.py does).

Each line: language, context ("" for every syllable), how often it occurred, then one count per
mark in MARKS and per syllable in NEXT (weighted, rounded). Two-syllable contexts seen less than
--min-count times are dropped (one syllable: a tenth of that), and two-syllable ones that say no
more than their last syllable: the engine
backs off from two syllables to one to none. Deterministic: sorted output.
"""

import argparse
import glob
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CORPORA = os.path.join(REPO, "corpora")
ENGINE = os.path.join(REPO, "engine")
OUT = os.path.join(ENGINE, "predict", "punctuation.tsv")

# The marks Mode A's symbol keys stand for after a word (docs/moonshot.md §5.6); the engine reads
# the columns in this order.
MARKS = ".,?/;:-~"
# The first syllables the symbol keys type at a word's end (을 를 의 서 에서 로 으로 가 께서 왜 예 얘).
NEXT = "을를의서에로으가께왜예얘"
COLUMNS = MARKS + NEXT

HANGUL_RUN = re.compile(r"[가-힣]+")


def eval_files():
    files = glob.glob(os.path.join(ENGINE, "tests", "corpora-en-ko", "*.txt"))
    files += glob.glob(os.path.join(ENGINE, "tests", "corpora-en-ko", "extended", "*.txt"))
    samsung = os.path.join(ENGINE, "tests", "data", "samsung-business.txt")
    if os.path.exists(samsung):
        files.append(samsung)
    files += glob.glob(os.path.join(CORPORA, "eval", "**", "*.txt"), recursive=True)
    files += glob.glob(os.path.join(CORPORA, "eval-long", "*.txt"))
    return sorted(files)


def count_korean(cap, seen, follows, excluded):
    for meta in sorted(glob.glob(os.path.join(CORPORA, "derived", "*", "meta.json"))):
        path = os.path.join(os.path.dirname(meta), "ko.txt")
        if not os.path.exists(path):
            continue
        with open(meta, encoding="utf-8") as f:
            weight = float(json.load(f).get("weight", 1.0))
        budget = int(cap * 1024 * 1024 * max(weight, 0.25))
        read = 0
        syllables = 0
        with open(path, encoding="utf-8", errors="replace") as f:
            for line in f:
                read += len(line)
                if read > budget:
                    break
                if " ".join(line.split()) in excluded:
                    continue
                for word in line.split():
                    for m in HANGUL_RUN.finditer(word):
                        run = m.group()
                        after = word[m.end():m.end() + 1]
                        for i, c in enumerate(run):
                            nxt = run[i + 1] if i + 1 < len(run) else after
                            column = COLUMNS.find(nxt) if nxt else -1
                            syllables += 1
                            for ctx in ("", c, run[i - 1:i + 1] if i else None):
                                if ctx is None:
                                    continue
                                seen[ctx] = seen.get(ctx, 0.0) + weight
                                if column >= 0:
                                    row = follows.setdefault(ctx, [0.0] * len(COLUMNS))
                                    row[column] += weight
        print(f"  {os.path.basename(os.path.dirname(path))}: {syllables} syllables", file=sys.stderr)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--cap-mb", type=float, default=12.0, help="MB read per source (times its weight)")
    ap.add_argument("--min-count", type=float, default=1000.0, help="fewest times a two-syllable context must be seen (a tenth for one syllable)")
    ap.add_argument("--out", default=OUT)
    args = ap.parse_args()

    excluded = set()
    for path in eval_files():
        with open(path, encoding="utf-8", errors="replace") as f:
            excluded.update(" ".join(l.split()) for l in f if l.strip())

    seen, follows = {}, {}
    count_korean(args.cap_mb, seen, follows, excluded)

    lines = [
        "# Which punctuation follows a word's ending (docs/moonshot.md §5.6), from the open corpora.",
        "# Built by engine/tools/corpora/build_punctuation.py: language, context, times seen, then",
        "# the counts of " + " ".join(COLUMNS) + " right after it.",
    ]
    for ctx in sorted(seen, key=lambda c: (len(c), c)):
        n = seen[ctx]
        # One syllable is kept when seen at all often: one missing reads as rare (the engine
        # lowers a mark after it).
        if ctx and n < (args.min_count if len(ctx) == 2 else args.min_count / 10):
            continue
        row = follows.get(ctx, [0.0] * len(COLUMNS))
        # A two-syllable context only where it says more than its last syllable does: the engine
        # backs off to that.
        if len(ctx) == 2 and ctx[1:] in seen:
            base_n = seen[ctx[1:]]
            base = follows.get(ctx[1:], [0.0] * len(COLUMNS))
            if all(abs(r / n - b / base_n) < 0.02 for r, b in zip(row, base)):
                continue
        lines.append("\t".join(["ko", ctx, str(round(n))] + [str(round(r)) for r in row]))
    with open(args.out, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print(f"wrote {args.out}: {len(lines) - 3} contexts", file=sys.stderr)


if __name__ == "__main__":
    main()
