# engine

Core engine of JIS Moonshot (forked from JIS Korean's): key map, Hangul automaton, kana, commit
logic, F6–F10 and predictions. No OS APIs, no I/O. See [../docs/moonshot.md](../docs/moonshot.md)
(the fork's design, which wins) and [../docs/spec.md](../docs/spec.md). The crate is
`jis-moonshot-engine`; the library, the C header and the `jk_*` API keep JIS Korean's names
(`jis_korean`, `include/jis_korean.h`).

| Path | What |
|---|---|
| `src/` | The engine. `ffi.rs` is the C ABI in `include/jis_korean.h`. `kana.rs` is the JIS kana layout, its automaton (゛ ゜) and the `KanaScheme` slot for romaji. |
| `tests/golden.rs` | Key sequences → expected text, for every spec example. |
| `tests/baseline.rs` | The core principle as a test: every syllable typed the conventional 2-set way, in every mode. |
| `tests/modes.rs` | Every mode against the mode contract ([../docs/modes.md](../docs/modes.md)); the retired ids are refused. |
| `tests/moonshot.rs` | JIS Moonshot's own behavior (docs/moonshot.md): kana typing, scripts, the switch keys and the kana reading. |
| `src/predict.rs`, `predict/` | The predictive layer (spec §10, `predict` feature, always on in the shells): ranks Korean, English, kana and other readings of the staged text. `predict/` holds its models and word lists (`common-ja.txt` for kana). |
| `src/learned.rs`, `predict/learned.bin` | The learned ranker (spec §10.13), the default ranking; the `legacy-predict` feature leaves it out and ranks by the rules alone. |
| `tools/train_learned.py` | Regenerates the learned ranker's training data (the corpora typed through the engine, the evaluation texts excluded) and retrains `learned.bin`. |
| `tools/corpora/` | Corpus tools, moved from JIS Korean's Minecraft mod: `fetch_corpora.py`, `build_ranker.py` (builds `predict/ranker.bin`), the Mirae word lists. |
| `src/plan.rs` | The typing robot and key-press counts (JIS Korean vs conventional 2-set), used by the robot test and the web playground. |
| `tests/robot.rs` | The robot types a real document key by key; see the report with `--nocapture`. |
| `tests/data/fetch_corpus.py` | Fetches the robot's corpus (gitignored). |

```sh
cargo test
cargo test --no-default-features      # the engine without predictions or English (no shell ships it)
cargo test --features legacy-predict   # the legacy build: predictions ranked by the rules alone
cargo test --test robot -- --nocapture
python3 -I tests/data/fetch_corpus.py   # from tests/data
python3 engine/tools/train_learned.py   # from the repo root, after ranker.bin or common-*.txt change
```
