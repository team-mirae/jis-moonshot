<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/logo/moonshot-dark.svg">
    <img src="docs/logo/moonshot.svg" width="120" alt="The JIS Moonshot logo: the left half of a taegeuk, cut at the midpoint, beside a solid half disc">
  </picture>
</p>

<h1 align="center">JIS Moonshot</h1>

<p align="center">
  <b>Korean, English and Japanese kana from one JIS keyboard, read at the same time.</b>
</p>

> [!WARNING]
> **Dogfooded rolling release.** JIS Moonshot ships whatever its owner is typing with this week.
> There is no LTS, no stable branch, and no proper testing before a release: beyond the engine's
> automated tests, a release has been tried by one person, on their own machines. Versions started
> at JIS Korean's (v0.3.1) and will diverge from it. Anything can change or break between releases.
> If you need an input method that works, use [JIS Korean](https://github.com/team-mirae/jis-korean).

JIS Moonshot is an experimental fork of [JIS Korean](https://github.com/team-mirae/jis-korean), a
Korean 2-set (두벌식) input method for JIS keyboards. It takes every key you type and reads it
three ways at once: as Korean 2-set, as English (QWERTY), and as Japanese kana on the JIS kana
layout. Predictions pick the likeliest reading and show it; you commit it, or pick another.

**This is a moonshot.** In the owner's words: "I do not expect it to be a usable product. let's try
it anyway." One key stream is ambiguous between three languages far more often than between two,
and nothing here promises that the ranking is good enough to type with. Expect wrong guesses. If you
want a Korean input method that works, use JIS Korean.

The design, and everything that changed from JIS Korean, is in [docs/moonshot.md](docs/moonshot.md).

## Status

- **macOS first** (macOS 13 or later), then **Windows**. The Windows shell is behind the macOS one.
- **JIS keyboards only.** JIS Korean's Modes B (US ANSI) and C (UK ISO), and the US and UK English
  modes, are gone: the kana reading needs the JIS kana legends.
- **No Minecraft mod, no lite build, no official builds or releases.** You build it from a clone.
- The web playground (`web/`) still runs the engine in a browser, as a demo. It isn't deployed.

## Modes

| Mode | Name | What it does |
|---|---|---|
| A | **JIS Moonshot** | The predictive mode. JIS Korean's 2-set keys, read as Korean, English and kana; the top prediction shows. |
| `A_KANA` | **JIS Kana** | Kana typing: every key types its JIS kana legend, as hiragana or katakana. Nothing is predicted. |
| `A_QWERTY` | **JIS English** | English, JIS QWERTY with JIS symbols, as in JIS Korean. |
| `A_DVORAK`, `A_COLEMAK` | **JIS Dvorak**, **JIS Colemak** | JIS English with Dvorak's or Colemak's letters. |

On macOS each mode is its own input source. Romaji kana typing isn't built; a mode id is reserved
for it (`A_ROMAJI`).

## Keys

| Key (macOS / Windows) | What it does |
|---|---|
| 英数 / 無変換 | To English (the English mode used last), committing what shows. |
| かな / 変換 | To Mode A. In Mode A with something typed: **force Korean** (the keys read as JIS Korean, nothing predicted). |
| F6 | Hiragana: the typed keys read as kana, until you commit; then predictive again. |
| ⌥Space (macOS) | A full-width space in JIS Moonshot and JIS Kana; `_` in English. |
| F7 | Katakana, the same way. |
| Shift+F7 | The hanja list (it was F7 in JIS Korean). Hanja still show as predictions. |
| F9 / F10 | Full-width / half-width letters, as in JIS Korean. |
| Shift+Z | A key like any other: ㅋ in Korean, っ in kana, Z in English. In JIS Korean it was F10. |

The typed keys are kept, not their text, so switching re-reads the same keystrokes: `ㅎ ㅏ ㄴ`
(G K S) shows 한; F6 shows きのと; かな shows 한 again. On macOS, カタカナ and F18 act as かな.
Everything else on the Korean side is JIS Korean's layout ([docs/spec.md](docs/spec.md)): standard
2-set on the letter keys, double consonants and compound vowels on the number row, Korean
punctuation on the symbol keys.

## Predictive typing must be accepted

JIS Moonshot doesn't work without predictions, so there is no switch to turn them off. The first
time it would type, it asks you to accept predictive typing; until you do, every key goes to the app
untouched. What it reads is the keys of the text you haven't committed yet, to rank their readings,
and that stays on your computer. Terminals, password managers and similar apps start with
predictions off, and there Mode A types plain (forced) Korean.

**Learning from your choices is optional** and off until you turn it on in the input menu.

## Setting it up

In a coding assistant that can run commands (Claude Code, for example), started in a clone of this
repository, say:

```
Read docs/interactive-prompts/start.md and follow it.
```

It checks the clone, asks about your operating system and keyboards, builds and installs JIS
Moonshot, and asks before it scans, installs or changes anything. It ends with an **interactive
training-file setup**, which is **strongly recommended**: ranking three languages from one key
stream is hard, and your own words (vocabulary to prefer, readings to avoid) are what make it
bearable. See [docs/interactive-prompts](docs/interactive-prompts/README.md) and
[docs/training-contract.md](docs/training-contract.md).

By hand:

| Path | What | Build |
|---|---|---|
| [`docs/moonshot.md`](docs/moonshot.md) | The design of the fork. Wins over the spec where they differ. | |
| [`docs/spec.md`](docs/spec.md) | JIS Korean's spec: every key, the automaton, F9/F10, predictions. | |
| [`engine/`](engine) | The engine, shared by every shell. Rust, no crates. | `cargo test` |
| [`mac/`](mac) | macOS input method (InputMethodKit). No Xcode needed. | `make`, `make install` |
| [`windows/`](windows) | Windows input method (TSF) and its installer. | `tools/build.ps1`, `tools/install.ps1` |
| [`web/`](web) | The browser playground. | `npm run dev` |

Building needs Rust 1.85 or later; macOS needs the Command Line Tools for Xcode, and Windows Visual
Studio 2022 Build Tools, CMake and Ninja. Each folder's README has the details.

## Credits

JIS Moonshot is a fork of **[JIS Korean](https://github.com/team-mirae/jis-korean)** (v0.3.1 and
`main` at `c0d2d49`), by Mirae Research and Min Jun Sung. Almost all of it, the Korean layout, the
engine, the predictor and both shells, is JIS Korean's work. JIS Moonshot installs beside JIS Korean
without colliding.

- The **hanja dictionary** is derived from [libhangul](https://github.com/libhangul/libhangul)
  (BSD-3-Clause), with readings from the [Unicode Unihan database](https://www.unicode.org/reports/tr38/)
  (Unicode License v3).
- The **prediction word lists and ranker model** are word counts from public corpora and word
  lists, with no sentences shipped: [NSMC](https://github.com/e9t/nsmc) (CC0 1.0),
  [Chatbot_data](https://github.com/songys/Chatbot_data) (MIT), [Tatoeba](https://tatoeba.org)
  (CC BY 2.0 FR), [Taskmaster-1](https://github.com/google-research-datasets/Taskmaster)
  (CC BY 4.0), 행정안전부 공통표준용어 and 공통표준단어 ([공공데이터포털](https://www.data.go.kr)),
  [FIBO](https://github.com/edmcouncil/fibo) (MIT), [dwyl/english-words](https://github.com/dwyl/english-words)
  (Unlicense) and [Princeton WordNet 3.0](https://wordnet.princeton.edu) (WordNet license). The
  license files ship alongside the data (`engine/predict/`, `engine/dict/`).
- The Japanese word list (`engine/predict/common-ja.txt`) is hand-written for JIS Moonshot.
- Used for testing only, not shipped: Samsung Electronics' "사업의 내용" from
  [DART](https://dart.fss.or.kr), and texts on [Korean Wikisource](https://ko.wikisource.org).
- Keyboard setups: [keyboard-troubleshooting](https://github.com/themrsung/keyboard-troubleshooting)
  (CC0-1.0).

<p align="center">
  <sub><a href="LICENSE">MIT</a> &nbsp;·&nbsp; Copyright 2026 <b>Mirae Research</b> &amp; Min Jun Sung</sub>
</p>
