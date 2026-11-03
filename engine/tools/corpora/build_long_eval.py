#!/usr/bin/env python3
"""Builds corpora/eval-long/mixed.txt: mixed Korean + English sentences for the long-context
benchmark (engine/tests/long_context.rs, docs/moonshot.md §5.5).

    python3 -I engine/tools/corpora/build_long_eval.py

Sentences from corpora/derived/<source>/ko.txt that mix Hangul and Latin words, 6 to 30 words,
none of them in the learned ranker's training plan (corpora/learned/plan.tsv); a fixed number per
source, picked deterministically. train_learned.py leaves corpora/eval-long/ out of training, so
these stay unseen. Gitignored, like the corpora: rebuild it here.
"""

import os
import random
import re

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
DERIVED = os.path.join(ROOT, "corpora", "derived")
PLAN = os.path.join(ROOT, "corpora", "learned", "plan.tsv")
OUT = os.path.join(ROOT, "corpora", "eval-long", "mixed.txt")
SOURCES = ["hike-code-switching", "kubernetes-docs-ko", "mdn-ko", "korean-petitions", "klue", "korfin-asc",
           "react-docs-ko", "koreascience-summarization"]
PER_SOURCE = 60


def main():
    plan = set()
    if os.path.exists(PLAN):
        with open(PLAN, encoding="utf-8") as f:
            for line in f:
                parts = line.rstrip("\n").split("\t")
                if len(parts) >= 4:
                    plan.add(parts[3])
    rng = random.Random(20261103)
    lines = []
    for source in SOURCES:
        path = os.path.join(DERIVED, source, "ko.txt")
        if not os.path.exists(path):
            print(f"  {source}: missing, skipped")
            continue
        pool = []
        with open(path, encoding="utf-8", errors="replace") as f:
            for line in f:
                line = " ".join(line.split())
                if line in plan or "http" in line or "@" in line:
                    continue
                if re.search(r"[가-힣]{2,}", line) and re.search(r"\b[A-Za-z]{3,}\b", line) and 6 <= len(line.split()) <= 30:
                    pool.append(line)
        pool.sort()
        picked = rng.sample(pool, min(PER_SOURCE, len(pool)))
        print(f"  {source}: {len(picked)} of {len(pool)}")
        lines += picked
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print(f"{len(lines)} sentences -> {OUT}")


if __name__ == "__main__":
    main()
