# JIS Moonshot: design

Status: **moonshot**. JIS Moonshot is a fork of JIS Korean (`upstream`, v0.3.1 + main at c0d2d49,
2026-11-01) that types Korean, English and Japanese kana from one JIS keyboard at once. Nobody
expects it to be a usable product; it's an experiment in how far one key stream can be read three
ways. Where this file and `spec.md` disagree, this file wins until the spec is updated.

## 1. What changed from JIS Korean

| # | JIS Korean | JIS Moonshot |
|---|---|---|
| 1 | Modes A, B (US ANSI), C (UK ISO) and the English modes on all three | **JIS only**: Mode A, the `A_` English modes, and the new `A_KANA` |
| 2 | Korean and English readings | Korean, English **and Japanese kana** (JIS kana layout), read from the same keys at once |
| 3 | Predictions are optional (`PREDICT=0`, a menu toggle) | **Always on.** The user accepts predictive typing before JIS Moonshot types anything. Learning stays optional |
| 4 | Training files are documented | The interactive install prompt **ends with an interactive training-file setup** (strongly recommended) |
| 5 | Minecraft mod | **Dropped** (its corpus tools moved to `engine/tools/corpora/`) |
| 6 | macOS, Windows, Minecraft, web | macOS first, Windows second. The web playground stays as a demo |
| 7 | 英数 → English, カタカナ/F18 → Korean, かな left to macOS | 英数 / 無変換 → English, かな / 変換 → predictive (Mode A); again in Mode A → force Korean |
| 8 | F6 commits Korean, F7 opens hanja; Shift+Z is F10 | **F6 → hiragana, F7 → katakana**, for the text being typed (`A_KANA` is the kana-only mode); **F8 → kanji**, **Shift+F8 → the hanja list**; F9/F10 as before. **Shift+Z is no longer F10**: it's ㅋ in Korean (standard 2-set again) and っ in kana |
| 9 | Hanja show as predictions | **No hanja predictions**; the kana reading's **kanji conversions** join the predictions instead (§5.1) |

Shift+Z was F10 in JIS Korean (F10 isn't reachable everywhere). In JIS Moonshot it types っ on the
kana layout, so it's a key like any other: ㅋ (standard 2-set's Shift+Z), っ, or `Z`. The core
principle's Shift+Z exception (spec §1) is gone. F10 itself is unchanged.

## 2. Modes

| Id | Mode name | Display name | What it is |
|---|---|---|---|
| 0 | `A` | JIS Moonshot | Predictive: Korean 2-set keys, readings in Korean, English and kana |
| 3 | `A_QWERTY` | JIS English | as in JIS Korean |
| 7 | `A_DVORAK` | JIS Dvorak | as in JIS Korean |
| 8 | `A_COLEMAK` | JIS Colemak | as in JIS Korean |
| 11 | `A_KANA` | JIS Kana | Kana typing on the JIS kana layout, hiragana or katakana |
| 12 | `A_ROMAJI` | (reserved) | Romaji kana typing. Not built: the id and the `KanaScheme::Romaji` slot are held for whoever writes it |

Ids 1, 2, 9 and 10 (B, C, B_QWERTY, CJ_QWERTY) are retired: never reused, refused by
`jk_set_mode`. A saved setting naming one falls back to Mode A. The English modes are always built
(the `english` feature is on in every shell build): 英数 needs them.

### 2.1 Kana typing is a mode, and its scheme is pluggable

`A_KANA` types kana directly: every key types its JIS kana legend, nothing is predicted, and the
segment is shown as hiragana or katakana (its **script**, switched by F6 and F7). The mapping from
keys to kana is a `KanaScheme` (`engine/src/kana.rs`): `Jis` is the only one built. `Romaji` is a
named slot: a romaji scheme needs its own composer (`ka` → か, `kk` → っk, `nn` → ん), and the mode
`A_ROMAJI` (id 12) would select it. Everything else (scripts, F6/F7/F9/F10, commit rules, the
predictive mode's kana reading) is written against the scheme, not the JIS table.

The predictive mode's kana reading always uses the JIS scheme: it reads the same keystrokes Korean
reads, and romaji keystrokes would read as English.

## 3. The JIS kana layout

| Key | Kana | Shift | | Key | Kana | Shift |
|---|---|---|---|---|---|---|
| 1 | ぬ | ぬ | | A | ち | ち |
| 2 | ふ | ふ | | S | と | と |
| 3 | あ | ぁ | | D | し | し |
| 4 | う | ぅ | | F | は | は |
| 5 | え | ぇ | | G | き | き |
| 6 | お | ぉ | | H | く | く |
| 7 | や | ゃ | | J | ま | ま |
| 8 | ゆ | ゅ | | K | の | の |
| 9 | よ | ょ | | L | り | り |
| 0 | わ | を | | ; | れ | れ |
| − | ほ | ほ | | : | け | け |
| ^ | へ | へ | | ] | む | 」 |
| ¥ | ー | ー | | Z | つ | っ |
| Q | た | た | | X | さ | さ |
| W | て | て | | C | そ | そ |
| E | い | ぃ | | V | ひ | ひ |
| R | す | す | | B | こ | こ |
| T | か | か | | N | み | み |
| Y | ん | ん | | M | も | も |
| U | な | な | | , | ね | 、 |
| I | に | に | | . | る | 。 |
| O | ら | ら | | / | め | ・ |
| P | せ | せ | | ろ | ろ | ろ |
| @ | ゛ | ゛ | | [ | ゜ | 「 |

### 3.1 The kana automaton

Kana compose like this: a kana then ゛ becomes its voiced form (か゛ → が, う゛ → ゔ), and a kana of
the は row then ゜ becomes its semi-voiced form (は゜ → ぱ). A ゛ or ゜ that can't attach stays as
it is (゛). Backspace removes the last key, so が goes back to か. Katakana is the hiragana shifted
to the katakana block (ゔ → ヴ); ー and punctuation stay.

## 4. Scripts and the switch keys

Each engine has a **script**, how the staged keys are shown and committed:

| Script | Shown as | Predictions |
|---|---|---|
| Auto | the top prediction (Korean, English or kana) | yes |
| Korean | JIS Korean as composed (forced) | no |
| Hiragana / Katakana | the kana reading, `A_KANA` | no |

Mode A starts every segment in Auto. `A_KANA` remembers hiragana or katakana.

| Key (macOS / Windows) | From an English mode | From Mode A | From `A_KANA` |
|---|---|---|---|
| 英数 / 無変換 | (stays) | commit what shows, to the last English mode | commit, to the last English mode |
| かな / 変換 | to Mode A | staged text: **force Korean** (script Korean). Nothing staged: nothing | to Mode A, the staged keys forced Korean |
| F6 | goes to the app | hiragana: the staged keys read as kana, **until the next commit** (then Auto again) | hiragana, kept |
| F7 | goes to the app | katakana, until the next commit | katakana, kept |
| F8 | goes to the app | **kanji**: the kana reading's conversions on top, until the next commit (§5.1) | the conversions above the kana, until the next commit |
| Shift+F8 | goes to the app | the hanja list (staged text; nothing staged: to the app) | nothing |
| F9 / F10 | goes to the app | as in JIS Korean (the word's keys as full- / half-width letters) | the staged run's keys as full- / half-width characters |
| Shift+Z | Z | ㅋ / っ (a key like any other) | っ |

- **The staged keys are kept, not their text**, so forcing a script re-reads the same keystrokes:
  `ㅎ ㅏ ㄴ` (G K S) shows 한; F6 → きのと; かな → 한 again.
- **F6 and F7 are per segment in Mode A** (2026-11-01, after trying it: a sticky switch to `A_KANA`
  kept predictions off until a round trip through English). With nothing staged they're still taken:
  the next segment is kana (Chrome's F6 is lost, as in JIS Korean it wasn't). **[decide]**
  `A_KANA`, the kana-only mode, is selected from the input menu; there F6 / F7 switch its script
  for good.
- **F6 or F7 after F9/F10** undo the conversion first, then show kana.
- **F8** (2026-11-01) asks for kanji: the staged keys' kana reading is converted and the
  conversions (up to three) go on top, the kana reading right after them, then the other readings.
  Like F6 it lasts until the next commit, and with nothing staged it's taken and applies to the next
  segment. After F6 / F7 in Mode A it goes back to Auto (with the kanji); F6 / F7 after it show kana
  again. In `A_KANA` (which predicts nothing by itself) it lists the conversions above the kana as
  typed, in the mode's script. **[decide]** whether F8 should put the conversions on top or only
  make sure they're listed.
- **The hanja list** moved from F7 to Shift+F7 and then to **Shift+F8** (2026-11-01, next to F8's
  kanji), with something staged in Mode A; Shift+F8 again moves down the list. Shift+F6 and
  Shift+F7 are reserved. Hanja are **no longer predicted** (§5.1).
- **⌥Space** (macOS) commits what shows and types a full-width space (U+3000) in Mode A and
  `A_KANA`, and types `_` in the English modes. It's an engine setting (`jk_set_option_space`), off
  by default: on Windows Alt+Space is the window menu. In a terminal, ⌥ in an English mode still
  goes to the app untouched.
- **⌥ + a symbol or digit key** (macOS, the same setting, 2026-11-01) commits what shows and types
  the key's JIS legend in full-width in Mode A and `A_KANA`: ⇧⌥1 → ！, ⌥1 → １, ⇧⌥/ → ？. The
  punctuation a Japanese IME converts is typed its way: ⌥ `,` `.` `/` → 、 。 ・, and ⌥ `[` `]` →
  『 』 in Mode A, 「 」 in `A_KANA`. ⌥ + a letter is still a shortcut for the app. In the English
  modes (無変換) ⌥ `[` `]` type « » (wherever the layout puts [ and ]); other ⌥ symbols go to the
  app.
- **Space** commits and types a space; with kana showing it types a full-width space (U+3000), as
  Japanese IMEs do in hiragana mode. **Enter** with kana showing commits without a new line
  (Japanese IMEs confirm with Enter). **[decide]**
- The engine can change its own mode (かな from `A_KANA`): shells read `jk_mode` after each call
  and select the matching input source when it changed.

## 5. The kana reading (predictions)

Mode A's predictions get a **kana** reading, of kind `Kana` (hiragana) or `Katakana`:

- **Text:** the JIS kana of every staged key, composed (§3.1). Shift on the number row, E and Z
  gives small kana (ぁ ぃ ぅ ぇ ぉ ゃ ゅ ょ っ), Shift+0 を. A typist who skips Shift still gets
  **っ variants**: a つ before a kana of the か, さ, た or ぱ rows may be っ (きつて → きって), one
  variant per place and one with all of them, the lexicon choosing.
- **Katakana:** offered instead of hiragana when the katakana form is a known word (コーヒー).
- **Phrases:** a reading made entirely of known words and particles counts as a known word, and
  a word the lexicon knows only in katakana is written in katakana inside it
  (あなたのギターになりたい).
- **Lexicon:** `engine/predict/common-ja.txt` (hand-written, hiragana and katakana words with
  counts; particles and endings marked). Japanese is written without spaces, so a staged segment is
  often a whole phrase: the score uses the share of the kana covered by known words and particles
  (a longest-match segmentation), not only exact matches.
- **Score** (bits, the predictor's units): the best other reading's score, minus 3, plus the
  evidence: coverage by known words (up to +5 for a whole known word, scaled down for 2–3 kana),
  each **kana-only key** inside the run (− ^ ¥ @ [ ; : ] , . / ろ, which type symbols in Korean:
  +1.5, at most +4.5), each bare jamo in the Korean reading (+0.5), the previous word's language
  (Japanese +2, Korean or English −1), and the user's Kana weight. A ゛ or ゜ that attaches to
  nothing, or small kana / ん / ー at the start, cost 6 bits; a Korean reading that is a known word
  costs 3. The learned ranker doesn't score kana (it wasn't trained on them); the kana reading is
  merged into its list by score.
- It takes one extra slot: up to 4 other readings and the kana one.
- **Weight** `Kana` (id 8, `Weight.Kana`), learned like the others when learning is on.
- **Training files:** `vocabulary.japanese` and `avoid.japanese` (kana words), training contract
  version 1 stays, with the new keys optional.

### 5.1 Kanji

The kana reading's **kanji conversions** (kind `Kanji`, `JK_PREDICTION_KANJI` = 12) join the
predictions **only when a Japanese reading (kana or katakana) is among the top n**, n = 2
(`KANJI_TOP_N`, to be tuned): up to three, in slots of their own right after that reading
(わたしは → 私は), so the reading on top is never displaced. F8 puts them on top (§4).

- **Measured (2026-11-01):** of the 3,000 commonest Korean words in `common-ko.txt`, typed the
  fluent way, none has a Japanese reading on top and 862 (29%) have one second, so with n = 2
  kanji are listed (third and below) under about three Korean words in ten. With n = 1, none.
- **Converter:** `engine/src/kanji.rs`, a word lattice searched for the cheapest paths (word costs
  plus part-of-speech connection costs, a POS bigram), at most 40 kana; about 30 µs per keystroke.
- **Data:** `engine/predict/kanji.bin` (11 MB), built by `engine/tools/kanji/build_kanji.py` from
  **Mozc's open-source dictionary** (google/mozc `src/data/dictionary_oss`, pinned commit): IPAdic
  (NAIST's permissive licence, notice required), the Okinawa dictionary (public domain) and Google's
  additions (BSD-3-Clause). Notices in `engine/predict/LICENSE-mozc.txt`, shipped with the data.
  No MIT kana-kanji library small enough to embed was found; Mozc's data is permissive, so it's
  used with a converter of our own rather than an n-gram model trained from scratch.
- **Pruning:** every entry of cost ≤ 5500 plus each reading's two cheapest surfaces up to 7000
  (386k of 1.29M entries); the connection matrix kept whole, one byte per cell, run-length coded.
  On Mozc's evaluation sentences (`tests/data/kanji-evaluation.tsv`) it gets 70% exactly right
  (79% for the whole dictionary) and 79% within the top three; `tests/kanji.rs` holds the floor.
- **No hanja predictions:** they'd compete with kanji for the same slots and read the same
  characters differently. `Kind::Hanja` and `Weight::Hanja` keep their ids; the hanja dictionary
  still counts as evidence that a Korean reading is a word. Hanja come from the list (Shift+F8).
- **Display:** Japanese readings (kanji, kana, katakana) are drawn in a Japanese font so kanji
  take their Japanese (新字体) forms whatever the system language: **Hiragino Sans** in the macOS
  suggestion list and marked text (with the language tagged `ja`; apps that draw marked text in
  their own font ignore it), **Yu Gothic UI** in the Windows list (Hiragino isn't on Windows), and a
  Japanese font stack with `lang="ja"` on the web. Committed text takes the app's font.

## 6. Predictive typing must be accepted

Predictions are always on; there is no switch and no lite build in the shells. The first time JIS
Moonshot would type, it asks once:

- **macOS:** an alert on first activation, until accepted. Until then every key passes through
  untouched and the input menu has "Accept Predictive Typing…". Stored as `PredictiveAccepted`
  (with the version of the notice).
- **Windows:** the installer shows the notice as a page that must be accepted; the TIP also checks
  `HKCU\Software\JIS Moonshot\PredictiveAccepted` and passes every key through until it's set (the
  indicator menu has "Accept Predictive Typing…").

The notice: what is read (the keys of the text not yet committed, to rank its readings), that it
stays on the device, that learning is a separate opt-in, and that the per-app exclusions
(terminals, password managers) still apply.

Per-app "Disable Predicting in <app>" stays. It's a safety exclusion, not a way to turn the feature
off. In those apps Mode A types forced Korean.

## 7. Platforms

### macOS (priority)

- Bundle id `me.sjun.inputmethod.JISMoonshot`, app `JISMoonshot.app`, settings domain the bundle
  id, data in `~/Library/Application Support/JIS Moonshot/`. It installs beside JIS Korean without
  colliding.
- Input sources: `…JISMoonshot.2Set` (Mode A, Korean script), `…JISMoonshot.A_QWERTY` /
  `A_DVORAK` / `A_COLEMAK` (Roman script), `…JISMoonshot.A_KANA` (Japanese script).
- **The かな proxy, no permission needed:** an input mode `…JISMoonshot.KanaKey` ("JIS Moonshot
  (かな key)") is the primary Japanese-script source, so macOS's own かな selects it. It's never left
  selected: the controller reroutes it to Mode A in place (from English), or forces Korean and
  selects Mode A again (from Mode A or JIS Kana), and types as Mode A meanwhile. JIS Kana is
  declared in the Korean script: macOS's かな does nothing while a Japanese-script source is
  selected, so the proxy must be the only one. 英数 likewise selects JIS English natively, as the primary Roman
  mode, once Apple's ABC is swapped for ABC – Extended. **Untested:** whether the proxy flashes in
  the menu bar, and whether Ctrl+Space lands on it.
- The key tap (optional, Accessibility) catches 英数 and かな (and カタカナ, F18) system-wide: かな goes to Mode A (or forces
  Korean there), 英数 to the last English mode. カタカナ and F18 act as かな. Reading カタカナ at the
  hardware level is no longer needed, since かな and カタカナ do the same thing.
- `make` builds the full engine with the English modes; `PREDICT=0` and `ENGLISH=0` are gone.

### Windows

- New CLSID and profile GUIDs, display name JIS Moonshot, `HKCU\Software\JIS Moonshot`,
  `%APPDATA%\JIS Moonshot\`. Installs beside JIS Korean.
- 無変換 (`VK_NONCONVERT`) → English (ABC with the English layout), 変換 (`VK_CONVERT`) → Mode A,
  or force Korean there. 한/영 (`VK_HANGUL`) still toggles English ↔ Mode A.
- The mode menu lists Mode A and JIS Kana. `-Lite` is gone.

## 8. The logo

The left half is the taegeuk, cut at the vertical midpoint; the right half is a solid half disc.
A transparent gap as wide as the S separates them. Drawn by `docs/logo/make_logo.py` (`moonshot.svg`),
used for the app and the mode badges.
