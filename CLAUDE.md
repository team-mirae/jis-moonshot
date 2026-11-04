# JIS Moonshot

An experimental fork of JIS Korean (https://github.com/team-mirae/jis-korean): one JIS keyboard
read as Korean 2-set (두벌식), English and Japanese kana at once, with predictions picking the
reading. The owner doesn't expect it to become a usable product; it's an experiment.

## Source of truth

- **`docs/moonshot.md`** is the design of the fork: what changed from JIS Korean. Where it and
  `docs/spec.md` disagree, `moonshot.md` wins until the spec is updated.
- **`docs/spec.md`** defines everything else. Read both before changing anything in `/engine`.
- If code and the docs disagree, the docs win (`moonshot.md` first, then the spec), or the docs
  get updated in the same change. Never let them drift.
- Items marked **[decide]** or listed under "Open questions" are not settled. Don't implement a guess
  silently: ask, or implement the spec's stated default and say so.
- Origin: the owner's note "2-set on JIS layout spec (draft)" and its prototype artifact
  (https://claude.ai/artifact/5TpYiuCYgZk8grLq6Sp4Sa).

## Core principle (never break)

Every valid standard 2-set key and combination still types what it always did. Dedicated keys add
shortcuts; they never remove anything. ㅗ+ㅏ is still ㅘ, and double finals still compose. The
sanctioned exceptions are listed in spec §1: the implied ㅇ (ㅏ → 아) and Shift+vowel starting a
syllable on ㅇ. Shift+Z is no longer F10: it's 카 (Shift+consonant), and っ in kana.

JIS Moonshot moved what standard 2-set puts on Shift (docs/moonshot.md §9, the owner's call,
2026-11-02): ㅃ ㅉ ㄸ ㄲ ㅆ are on 1–5 only and ㅒ ㅖ on `:` and `;`; Shift on every vowel key, O and
P included, starts a syllable on ㅇ. Since 2026-11-03 (§9.2) Shift on a consonant key types the
consonant and ㅏ (Shift+R → 가), so Shift held over consonants no longer types 2-set. The baseline holds every other key to standard 2-set.

## Layout

| Path | What |
|---|---|
| `/engine` | Core engine: key map, Hangul automaton, kana, segment/commit logic, F6–F10, predictions. Platform-independent, no OS APIs, no I/O. |
| `/mac` | macOS input method (InputMethodKit, Swift) that wraps the engine. Built with `make` (no Xcode needed). |
| `/windows` | Windows TSF text input processor that wraps the engine. |
| `/web` | React playground that runs the engine in the browser so people can try it before installing. |
| `/docs` | The fork's design (`moonshot.md`), the spec and design notes. |

Build order: macOS → Windows. JIS only (Mode A, JIS Kana, the `A_` English modes). Android and
iPhone are out of scope.

## Architecture rules

- **All behavior lives in `/engine`.** Shells translate OS key events into canonical key codes
  (`KeyboardEvent.code` names, see spec §2) and apply the engine's output (commit text, preedit,
  candidates) to the host. A shell that decides what a key types is a bug.
- The engine API is **event in → actions out**: `(key code, modifiers) → { commit, preedit, consumed }`.
  No callbacks into the shell, no global state. A shell can then hold one engine instance per input
  context.
- The engine must be deterministic and never panic or crash on any input. On Windows it runs inside
  every app's process.

## Building

- Engine: `cargo test` in `/engine`.
- Web: `npm run dev` in `/web` (builds the engine to wasm first).
- macOS: `make` / `make install` in `/mac` (see `mac/README.md`). One build: predictions, the
  English modes and JIS Kana are always in; `make LEGACY_PREDICT=1` is the only variant.
- Engine builds (spec §10.1, §10.13): **full** (default: `predict` and `english`) ranks predictions
  with the learned ranker (`engine/predict/learned.bin`); **legacy** (`--features legacy-predict`)
  ranks by the rules alone. `cargo test --no-default-features` still has to pass (the engine builds
  without predictions or English), but no shell ships that build. macOS: `make` /
  `make LEGACY_PREDICT=1`; Windows: `tools/build.ps1` / `-Legacy` (no `-Lite`).
- Key model (docs/moonshot.md §5): `python3 -I engine/tools/corpora/build_keymodel.py [--eval]`
  rebuilds the three-language order-4 model in `engine/predict/ranker.bin`, keeping the
  Korean-vs-English tables the file starts with; it reads the gitignored corpora (fetch with
  `engine/tools/corpora/fetch_corpora.py`) and the Mozc cache (`build_kanji.py`).
- Learned ranker: `python3 engine/tools/train_learned.py` regenerates its training data (typing the
  corpora through the engine) and retrains `learned.bin` in one command; rerun it after
  `ranker.bin` or `common-*.txt` change. Never train on the evaluation texts (it excludes them).
- Kanji (docs/moonshot.md §5.1): `python3 engine/tools/kanji/build_kanji.py` fetches Mozc's
  open-source dictionary at a pinned commit (cached, gitignored) and writes `engine/predict/kanji.bin`,
  `LICENSE-mozc.txt` and `tests/data/kanji-evaluation.tsv`. `tests/kanji.rs` holds the accuracy floor.
- Brands (docs/moonshot.md §5.4): `python3 -I engine/tools/brands/build_brands.py` queries Wikidata
  (CC0) and writes `engine/predict/brands.tsv`, compiled into the engine; queries are cached in the
  gitignored `corpora/brands/`.
- Punctuation (docs/moonshot.md §5.6): `python3 -I engine/tools/corpora/build_punctuation.py`
  counts what follows each Korean syllable (the gitignored corpora) and each kana (Tatoeba's
  Japanese sentences, from `fetch_corpora.py`) and writes `engine/predict/punctuation.tsv`,
  compiled into the engine.
- Corpus tools (from the dropped Minecraft mod): `engine/tools/corpora/` (`fetch_corpora.py`,
  `build_ranker.py`, the Mirae word lists).
- Windows: `tools/build.ps1` in `/windows` (x64 + x86, runs the shell tests),
  `tools/install.ps1` (admin) / `-Update`, and `tools/type-test.ps1` types through
  the installed TIP (see `windows/README.md`). Read `docs/troubleshooting/windows-tsf-notes.md`
  before changing how keys or the composition reach the app.
- `cargo` lives in `~/.cargo/bin`, which may not be on PATH in non-interactive shells.

## Testing

- The engine gets **golden tables**: key sequences → expected commit and preedit, covering every
  automaton transition, backspace, each F-key, and every core-principle example in the spec.
- Every spec example (e.g. the F9/F10 cycle in §6.2) and every `moonshot.md` example must be a
  test. The fork's own behavior (kana typing, scripts, the switch keys, the kana reading) is in
  `tests/moonshot.rs`.
- **The robot** (`engine/src/plan.rs`, run by `engine/tests/robot.rs`) types real text key by key
  and checks the screen after every character. The web playground's key-press comparison uses the
  same planner (`plan::count`). It reads keys from the engine's layout tables, so a
  layout change that breaks typability shows up there. Its corpus, Samsung Electronics' latest
  "사업의 내용", is fetched by `tests/data/fetch_corpus.py` and gitignored (it's the filer's
  text). See the report with `cargo test --test robot -- --nocapture`.
- Run with `cargo test` in `/engine`. `tests/header.rs` fails if `include/jis_korean.h` drifts from
  the key list.
- **The baseline** (`engine/tests/baseline.rs`) is the core principle as a test: every syllable,
  every syllable before every initial, and a pangram covering every jamo, typed the conventional
  2-set way (and with sloppy Shift, and with the dedicated keys) in every mode. Its 2-set tables
  are written out independently of the engine's. Never loosen it to make a change pass.
- CI (`.github/workflows/test.yml`) runs the baseline and the full, legacy and lite engine tests,
  and builds the Windows shell and installer, on every pull request and branch push;
  `deploy-web.yml` runs the same tests before deploying `main` to jis-moonshot.sjun.me (Vercel
  project `jis-moonshot`, secret `VERCEL_TOKEN`).
- Shells are tested by hand against the app matrix (to be written per platform).

## Conventions

- User-facing name: **JIS Moonshot** (modes: JIS Moonshot for Mode A, JIS Kana, JIS English,
  JIS Dvorak, JIS Colemak). Code identifiers kept from JIS Korean (`jis_korean`, `jk_*`,
  `include/jis_korean.h`) stay.
- Write spec examples with jamo and JIS legends (`ㅎ ㅏ ㄴ`, `F6`), and write code-level tests with
  canonical key codes (`KeyG KeyK KeyS`).
- Unicode code points in the spec and in comments as `U+XXXX`.
