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
| 8 | F6 commits Korean, F7 opens hanja; Shift+Z is F10 | **F6 → hiragana, F7 → katakana**, for the text being typed (`A_KANA` is the kana-only mode); **F8 → kanji** (Shift+F8 the same), **the hanja list**; F9/F10 as before. **Shift+Z is no longer F10**: it's ㅋ in Korean (standard 2-set again) and っ in kana |
| 9 | Hanja show as predictions | **No hanja predictions**; the kana reading's **kanji conversions** join the predictions instead (§5.1) |
| 10 | Symbols on the keys right of 0 (! ( — ' [ ~ : ] , . · \\), ㅃ ㅉ ㄸ ㄲ ㅆ ㅒ ㅖ on Shift | Those 12 keys type syllables, compound vowels and finals instead (24 slots with Shift), and Shift on Q W E R T O P changes (§9) |

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
| Korean | JIS Korean as composed (forced) | only punctuation (below) |
| Hiragana / Katakana | the kana reading, `A_KANA` | no |

Mode A starts every segment in Auto. `A_KANA` remembers hiragana or katakana.

| Key (macOS / Windows) | From an English mode | From Mode A | From `A_KANA` |
|---|---|---|---|
| 英数 / 無変換 | (stays) | commit what shows, to the last English mode | commit, to the last English mode |
| かな / 変換 | to Mode A | staged text: **force Korean** (script Korean). Nothing staged: nothing | to Mode A, the staged keys forced Korean |
| Shift+かな / Shift+変換 | to Mode A | staged text: **force bare Korean** (script KoreanBare): no implied ㅇ, so ㅏ stays ㅏ (2026-11-03) | to Mode A, the staged keys forced bare |
| F6 | goes to the app | hiragana: the staged keys read as kana, **until the next commit** (then Auto again) | hiragana, kept |
| F7 | goes to the app | katakana, until the next commit | katakana, kept |
| F8 | goes to the app | **kanji**: the kana reading's conversions on top, until the next commit (§5.1) | the conversions above the kana, until the next commit |
| Shift+F8 | goes to the app | the same as F8 (the hanja list is gone) | the same as F8 |
| F9 (한글) | goes to the app | **force Korean**, the same as かな / 変換 (2026-11-03) | the same as かな / 変換 |
| Shift+F9 | goes to the app | **force bare Korean**, the same as Shift+かな / Shift+変換 | the same |
| F10 (ABC) | goes to the app | the whole staged text as its keys (English, half-width); again: UPPER, Capitalized | the same |
| Space | the app's | **a space, staged** (2026-11-03): the words stay unconfirmed and are read together. Nothing staged: the app's | a full-width space, staged |
| Shift+Space | the app's | **commit** what shows, then a space | commit, then a full-width space |
| Shift+Z | Z | 카 (Shift+consonant, §9.2) / っ | っ |
| Cmd / Ctrl + a key | the app's shortcut | the app's shortcut | the app's shortcut |

- **Committing** (2026-11-03): **Shift+Enter** commits with no side effect; **Shift+Space** and
  **Enter** commit and then do what they do (a space, a new line; in kana Enter only confirms);
  losing focus commits too (no side effect, though probably not meant). F6–F10 never commit: they
  only change the reading. Plain **Space** stages a space instead of committing, so a phrase stays
  unconfirmed longer and its words are read together (spec §10.10's pieces); with nothing staged
  it's the app's. A row chosen with ↓ / Tab is kept by Space, which stages the space after it
  (`jk_key_keeps_choice`); the top reading nobody chose stays open to the words after it. With nothing staged F6–F10 still go to the app (F6–F8 apply to the next
  segment, as before). The English modes are untouched: 英数 / 無変換 there does nothing.
- **Punctuation when forced** (2026-11-05): a forced Korean reading (かな / 変換, F9, and their
  Shift forms) gets a short list instead of none: the forced reading on top (what shows), then
  the symbol keys at the end read as their stock symbols, then every symbol key before a staged
  space or the end read so, each with its §5.2 variants. So 그렇고 `,` + 変換 shows 그렇고의, then
  그렇고, 그렇고、 그렇고，. A symbol typed as one (`.` and Shift+`,` type themselves) gets its
  variants on the forced reading itself: 안녕하세요. 안녕하세요。… With nothing to offer there's no list, as before. The rows aren't the
  predictor's: choosing one teaches the ranker nothing.
- **F9 한글 / F10 ABC.** F9 used to be full-width letters; full-width now comes from 半角/全角
  (§4.1), so F9 forces the Korean reading like 変換 (which stays), Shift+F9 the bare one. F10
  shows the whole staged text as its keys and cycles case on repeat; F9, F6 or F7 undo it.
- **Cmd and Ctrl shortcuts are the app's, at their QWERTY positions** (2026-11-03). The engine
  commits what shows and passes them on. On macOS, while either is held the app is given JIS
  English's layout, whose plain layer is QWERTY: apps that match shortcuts by
  `charactersIgnoringModifiers` (zenteru) read that layer, which is jamo in Mode A (Cmd+C read as
  ㅊ) and Dvorak's letters in JIS Dvorak. On release the mode's layout comes back (or, if Cmd+Tab
  released it in another app, before the next key without Cmd or Ctrl there). The Keyboard Viewer
  shows QWERTY while it's held.
- **The staged keys are kept, not their text**, so forcing a script re-reads the same keystrokes:
  `ㅎ ㅏ ㄴ` (G K S) shows 한; F6 → きのと; かな → 한 again.
- **F6 and F7 are per segment in Mode A** (2026-11-01, after trying it: a sticky switch to `A_KANA`
  kept predictions off until a round trip through English). With nothing staged they're still taken:
  the next segment is kana (Chrome's F6 is lost, as in JIS Korean it wasn't). **[decide]**
  `A_KANA`, the kana-only mode, is selected from the input menu; there F6 / F7 switch its script
  for good.
- **F6, F7 or F9 after F10** undo the conversion first, then show their reading.
- **F8** (2026-11-01) asks for kanji: the staged keys' kana reading is converted and the
  conversions (up to three) go on top, the kana reading right after them, then the other readings.
  Like F6 it lasts until the next commit, and with nothing staged it's taken and applies to the next
  segment. After F6 / F7 in Mode A it goes back to Auto (with the kanji); F6 / F7 after it show kana
  again. In `A_KANA` (which predicts nothing by itself) it lists the conversions above the kana as
  typed, in the mode's script. **[decide]** whether F8 should put the conversions on top or only
  make sure they're listed.
- **No hanja** (2026-11-02, §5.1): the hanja list (F7 in JIS Korean, then Shift+F7, then
  Shift+F8) is gone with the hanja dictionary. Shift+F8 is F8. Shift+F6 and Shift+F7 are reserved.
- **⌥Space** (macOS) commits what shows and types a full-width space (U+3000) in Mode A and
  `A_KANA`, and types `_` in the English modes. It's an engine setting (`jk_set_option_space`), off
  by default: on Windows Alt+Space is the window menu. In a terminal, ⌥ in an English mode still
  goes to the app untouched.
- **⌥ + a symbol or digit key** (macOS, the same setting, 2026-11-01) types the key's JIS legend
  in full-width in Mode A and `A_KANA`, **staged** after what shows (2026-11-03; it used to commit),
  so 半角/全角 (§4.1) can still change it and F10 converts it: ⇧⌥1 → ！, ⌥1 → １, ⇧⌥/ → ？. The
  punctuation a Japanese IME converts is typed its way: ⌥ `,` `.` `/` → 、 。 ・, and ⌥ `[` `]` →
  『 』 in Mode A, 「 」 in `A_KANA`. ⌥ + a letter is still a shortcut for the app. In the English
  modes (無変換) ⌥ `[` `]` type « » (wherever the layout puts [ and ]); other ⌥ symbols go to the
  app.
- **Space** commits and types a space; with kana showing it types a full-width space (U+3000), as
  Japanese IMEs do in hiragana mode. **Enter** with kana showing commits without a new line
  (Japanese IMEs confirm with Enter). **[decide]**
- The engine can change its own mode (かな from `A_KANA`): shells read `jk_mode` after each call
  and select the matching input source when it changed.

### 4.1 半角/全角

The 半角/全角 key (the `Backquote` code: left of 1 on a full JIS board; 2026-11-03) does what its
label says: **full-width**, and with **Shift half-width**. It no longer types `` ` `` and `~`
(on JIS they're Shift+`@` and Shift+`^` in the English modes). ⌥, ⌘ or Ctrl with it is a shortcut.

- **Staged text** (Mode A, `A_KANA`): what shows (the reading on screen, a prediction included)
  converts and stays staged, as plain text: ⇧⌥1 ⇧⌥/ → ！？, then ⇧半角/全角 → !?, then 半角/全角 →
  ！？ again; 한12 → 한１２; かな → ｶﾅ. Typing goes on after it.
- **Nothing staged, or an English mode** (無変換, which stages nothing): the app's text, in this
  order:
  1. the **selection**, if there is one;
  2. else the **word the caret is on**: the run of characters up to a blank (space, U+3000, any
     whitespace) or a change of script (Latin → Hangul, hiragana → katakana; digits, punctuation
     and ー go with the script they're in), holding the character right after the caret.
     `wo|rd` and `|word` convert `word`; at the end of a word (`word|`) there's none;
  3. else nothing (the key is still taken).
- **What converts:** every character with a form in that width (spec §5.3's pairs: ASCII and the
  space, ¥ and the like, 。「」、・), and kana: half-width makes hiragana and katakana half-width
  katakana (がぱ → ｶﾞﾊﾟ), full-width puts half-width katakana back (ｶﾞﾊﾟ → ガパ; hiragana stays
  katakana). Hangul and kanji have no other width.

The engine decides all of it: with nothing staged it asks for a host edit (`jk_convert_host`),
and the shell hands it the selection and the text around it (`jk_host_edit`), which say what to
replace with what. macOS and Windows (an edit session) read up to 256 characters either side, and
the caret ends after the replaced text. Windows' English is ABC, which never reaches the engine, so
there the key works in Mode A and JIS Kana only.

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
  (a longest-match segmentation), not only exact matches. A **common word in the kanji dictionary**
  counts as known too (2026-11-02): three kana or more, Mozc cost 4500 or less (どようび 3353 in,
  きのと 5766 out), so common words the hand-written list lacks read as Japanese and get their kanji.
- **Korean-only keys** (§9.3) lift Korean over English; a known Japanese word gets the same lift, so
  those keys (kana-only keys too) don't push Korean past it: どようび, typed with ゛ on `@`.
- **Key model** (2026-11-02): `predict/ranker.bin` carries a three-language key model after the
  Korean-vs-English tables: order-4 key n-grams over the 48 JIS keys (Shift dropped), one each
  for Korean on JIS Moonshot's keys, English and JIS kana typing (Mozc's readings and
  `common-ja.txt`), Witten-Bell smoothed, format `JKR3` (`engine/src/keymodel.rs`, built by
  `engine/tools/corpora/build_keymodel.py`, about 5 MB). Held out: Korean 99.5%, English 99.5%,
  kana 99.0% (97.6% for common words not trained on). Only the kana reading uses it so far: the
  rules are tuned on the older Korean-vs-English tables, which keep that job.
- **Score** (bits, the predictor's units): the best other reading's score, plus the **kana key
  model's** bits against the likelier of its Korean and English models (×0.5, −1, within ±6;
  without the model, a fixed −3), plus the evidence: coverage by known words (up to +5 for a
  whole known word, scaled down for 2–3 kana), each **kana-only key** inside the run (− ^ ¥ @ [ ;
  : ] , . / ろ: +1.5, at most +4.5; only without the key model, which already has them), each bare
  jamo in the Korean reading (+0.5), the previous word's language
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
- **No hanja at all** (2026-11-02, the owner's call; losses accepted): hanja predictions would
  compete with kanji for the same slots, and the hanja list and dictionary cost test time for little
  use. The dictionary (`engine/dict/`), the list, `jk_dict_*` / `jk_set_dictionary` and the
  prediction kind are gone; `Weight::Hanja` (id 6) stays as a retired slot so the weights after it
  keep their ids. Without the dictionary, Sino-Korean words it knew no longer count as known words
  (the legacy ranker's corpus limits rose a little, `tests/corpora.rs`).
- **Display:** Japanese readings (kanji, kana, katakana) are drawn in a Japanese font so kanji
  take their Japanese (新字体) forms whatever the system language: **Hiragino Sans** in the macOS
  suggestion list and marked text (with the language tagged `ja`; apps that draw marked text in
  their own font ignore it), **Yu Gothic UI** in the Windows list (Hiragino isn't on Windows), and a
  Japanese font stack with `lang="ja"` on the web. Committed text takes the app's font.

### 5.2 Symbol variants

A reading that ends in a symbol is also offered with that symbol's variants, right below it and of
its kind (2026-11-03), as a Japanese IME offers 、 and ， for a comma. Every reading gets them
(Korean, English, kana, mixed), except the standard 2-set reading, whose symbols are the stock
keys' (`<` for Shift+`,`). Only the last character counts, so they show while the symbol is the
last thing typed. The table is `engine/src/punctuation.rs`:

| Symbol | Variants |
|---|---|
| , | 、 ， |
| . | 。 … ... ． |
| / and · | ・ ／ (/) and ・ ･ (·) |
| ¥ | · ・ ￥ \ (· was on the ¥ key until 2026-11-05, §9.2) |
| [ ] | 「 『 【 ［ 〔 and 」 』 】 ］ 〕 |
| 「 」 『 』 | each other, 【 】, [ ] |
| ( ) { } < > | full-width, and 〈 《 « / 〉 》 » for < > |
| ! ? : ; ~ - ' " | full-width, 〜 for ~, ー — – for -, curly quotes |

Shift+`,` → , 、 ，; 한 + Shift+`,` → 한, 한、 한，; `¥` → ¥ · ・ ￥; `h e l l o .` → hello. hello。
hello… hello....

### 5.3 Japanese particles after English

As Claude에 (an English start and a Korean particle, spec §10.3), an English start can take a
Japanese particle read from the ending's keys as JIS kana (2026-11-03): Claude `k` → Claudeの,
Claude `f` → Claudeは, GitHub `w @` → GitHubで, Tokyoの, OpenAIの (`KANA_PARTICLES`: は が を に で
と も へ の や か ね よ から まで より and a few longer ones). The same gates hold: the start reads
clearly more English than the whole keys, or is a capitalized name (for a kana particle a
capitalized start that reads as English is enough, since `w @` doesn't look Korean), and a whole
that is a known English word isn't split. Where a Korean particle fits too, it wins, even over a
longer start (Claude `p` → Claude에; Claude `r k` → Claude가, not Clauder + `k` as の). The reading is of kind Mixed.

### 5.4 Brand names

A reading that spells a brand, in any script, also offers the brand in its **default script**
(2026-11-03, the owner's call), by the country of origin:

1. A name ending in "Korea" (a Korean branch, AWS Korea → 아마존웹서비시즈코리아): Korean.
2. A name ending in "Japan" (구글재팬 → Google Japan): English.
3. Else a company registered or operating mainly in Korea: Korean; in Japan: Japanese; anywhere
   else: English.

**Which brands** (2026-11-05, the owner's bar): companies and their brands only (not people,
places, schools, clubs, parties, governments or nonprofits; nothing dissolved), and of those only
ones that are

- **in a major stock index** (S&P 500, NASDAQ-100, Dow Jones, KOSPI 200, Nikkei 225, FTSE 100,
  DAX, CAC 40, EURO STOXX 50, SMI, Hang Seng) **or big enough to pass that bar easily** (market
  value or revenue over the country's bar: $20B / $10B in the US and elsewhere, $2B / $3B in Korea,
  $7B / $7B in Japan), or
- **known by the name alone**: searched without context, the name is the brand. Measured on
  Wikipedia: the article at the bare name (English, Korean or Japanese) is the brand's or
  redirects to it, or is read less than the brand's, and the brand's is widely read (60 days:
  30,000 views in English, 12,000 in Japanese, 3,000 in Korean; four times that when the bare name
  is a disambiguation page).

`engine/tools/brands/build_brands.py` and `admission.py` build it (2,021 brands, from 18,012
before); every candidate's verdict and reason is in the gitignored `corpora/brands/admission.tsv`.
Sola Municipality (そら → Sola, Norway) and Sora Aoi are out; Google, Uniqlo, IKEA and Coupang in.

So 스타벅스 offers Starbucks, 구글 Google, 소니 / sony ソニー, honda / 혼다 ホンダ, 도요타 トヨタ, kakao
카카오, and before a particle the particle stays (스타벅스에 → Starbucks에). It's **only a
suggestion** (kind `Brand`, id 14): it goes right below the reading that spells it, never above
what was typed, and nothing is added when the brand is already in what was typed or differs only
in case (apple → Apple is the English reading's own Capitalized).

- **Matching:** the Korean reading, the English reading (four letters or more, or capitals: LG)
  and the kana reading (only when a kana reading is offered) are matched without spaces or middle
  dots, so long Korean and Japanese names typed without spaces match. Before a particle the start
  keeps one script: a Korean start with a Korean particle, a kana one with a kana particle, an
  English start (read as English: a mixed reading with it) with either.
- **Which name:** a brand's names in its default script, without legal forms (Inc., 주식회사,
  株式会社), the main one unless an alias is clearly closer in length to what was typed (도요타 →
  トヨタ, not トヨタ自動車; honda → ホンダ, not 本田技研工業). Short forms without corporate words
  (소니 for 소니 그룹) are spellings too.
- **Data:** `engine/predict/brands.tsv`, about 18,000 companies and brands from Wikidata (CC0) with
  an English name and a Korean or Japanese one, built by `engine/tools/brands/build_brands.py`
  (queries cached in `corpora/brands/`). It's compiled into the engine, so no shell loads it.

### 5.5 Long context: staged phrases

Space stages a space (§4), so a whole phrase can stay unconfirmed. Such a phrase is **decoded word
by word** (2026-11-03): each word (a piece between spaces or punctuation, spec §10.10) is ranked
on its own with the language of the word before it as its context, as a committed word gives
the next one; its scores become probabilities, and a beam search (8 phrases, 4 readings a word)
keeps the likeliest phrases. The phrases typing straight through would give (every word Korean,
every word English) stay listed after them. Text staged without spaces (Research(이하) keeps the
pieces reading of spec §10.10. A 24-word phrase predicts in about a millisecond.

- **In a phrase**, a Japanese particle after English is read only in Japanese context: in a
  Korean or English phrase the last letter of an English word reads as one (PowerPoint as
  PowerPoin + `t` か).
- **Stacked particles** count as an ending after English: 들 + 에게, 부터 + 가, 들 + 에게 + 는
  (client들에게, naming부터가). Only particles that can start a stack come first; 이 가 을 를 은 는
  의 end one (model의 isn't mode + 이의).
- **The d of a name** (Claude + `d p` → Claude에) is only taken back after a capitalized start:
  a lowercase word keeps it (frontend에서).
- **No neighbour bonus:** rewarding neighbours for sharing a language cost words on real
  code-switching text, which switches often.

**Measured** by `engine/tests/long_context.rs`, typing each line of the evaluation texts in
8-word windows with Space between the words, against the same words typed and committed one at a
time. On the benchmark (480 mixed Korean + English sentences from the open corpora, none trained
on: `engine/tools/corpora/build_long_eval.py` → `corpora/eval-long/`, gitignored) staged phrases
went from 97.8% of 7,366 words right (86.9% of 1,081 phrases exact) to 98.3% (89.9%); on the
owner's mixed-script text from 98.5% to 99.9%, now as good as one word at a time.

**Next: bigger lexicons.** Rebuilding the word lists with lower minimum counts
(`build_ranker.py --lexicon-min-en 5 --lexicon-min-ko 4`: 40,000 English and 63,000 Korean
generated words, up from 14,000 and 34,000) lifted the benchmark to 98.7% of words (92.3% of
phrases exact) without retraining, but broke the owner's corpus limits, the avoid list and the
bare-vowel tests in both builds: the rarer words bring noise (keystrings, rare forms). **[decide]**
whether to clean a larger list, recalibrate the rules to it and retrain the learned ranker.

### 5.6 Punctuation in Korean and kana readings

Four of Mode A's symbol keys type Korean (§9.2: `−` ㅙ, `;` ㅖ, `:` ㅒ, `,` ㅢ), so their marks
have no key of their own in Korean: a comma after a word is Shift+`,`, but `,` alone is ㅢ. The
Korean readings read the mark from those keys too (2026-11-05, the owner's call): when the staged
keys end in one, the keys before it are read again and each of their Korean readings is offered
with the key's stock JIS symbol (the last key, and the last two). So 그렇고 `,` → **그렇고,**, with
그렇고의 below it; 했는데 `,` → 했는데,; and then each mark's variants (§5.2: 그렇고、). `.` and `/`
type themselves since §9.2's change of the same day (안녕하세요 `.` → 안녕하세요.); until then they
typed 을 and 서, and this section decided between 안녕하세요. and 안녕하세요을 the same way.

Which way it goes is learned from the corpora, since Korean punctuation follows the ending
closely: a final ending (다 요 까 니다 세요) takes . or ?, a connective one (고 는데 지만) a comma, a
particle (을 를 의) almost never anything. `engine/predict/punctuation.tsv` (compiled in; built by
`engine/tools/corpora/build_punctuation.py` from `corpora/derived/*/ko.txt`, the evaluation texts
left out) counts, after each syllable and each pair of syllables, how often each mark (`. , ? / ;
: - ~`) comes next and how often each syllable the symbol keys type (을 를 의 서 에 로 으 가 께 왜 예 얘).
A punctuated row's chance is its share of the readings of the keys before the mark, times how often
its ending takes the mark, against how often the syllable the key types follows the same reading
(고 is followed by , 9% of the time and by 의 0.6%; 회 by 의 10% and by , 1.4%). So nouns keep the
syllable the key types (회의) with the mark below, and endings take the mark. The table still has
the columns of the syllables the retired slots typed (을 를 서 …): rebuilding keeps them.

- Only readings with an eighth or more of the probability among the keys before the mark are
  punctuated (not 안녕핫에요., nor a stray kana reading of Korean keys), at most three a language,
  in the order they were ranked: a mark places them, it doesn't reorder them. A mark lifts a
  reading at most 8 bits over the typed one, however unlikely what the key typed is.
- A second mark costs 6 bits more (운송,, for 운송의 typed d `,` `,` isn't on top). After a
  syllable the table doesn't have (seen too rarely: 틍) a mark is an eighth as likely, after a
  bare jamo a sixty-fourth.
- In a staged phrase (§5.5) each word takes its own: 안녕하세요 `.` Space 반가워요 `.` → 안녕하세요.
  반가워요. A phrase of Korean words is of kind Korean.
- **The middle dot** (2026-11-05): the kana reading reads Shift+`/` as ・, and Korean writes a
  list with · (U+00B7, 가운뎃점: 가·나·다). So a Korean or mixed reading with `?` (Shift+`/`) or `/`
  between two letters (syllables, jamo, Latin: 가?ㄴ while 나 is typed) is also offered with · there: above the text as typed for `?` (가 Shift+`/` 나
  Shift+`/` 다 → 가·나·다, 가?나?다 below it), just below it when only `/` changes (남/여, then
  남·여). Forced Korean offers it too. `¥` isn't read so (가¥나 stays).
- The English reading needs none of this: its keys are the stock ones (hello.). Forced Korean
  (かな / 変換, §4) lists its own punctuation rows.

**Kana** (2026-11-05) works the same way. On the JIS kana layout `.` `,` `/` are る ね め, while a
romaji typist's `.` `,` `/` give 。、？, so the kana readings of the keys before such a key are also
offered with 。、？ (Shift+`.` and Shift+`,` are 。 and 、 already). The counts come from Tatoeba's
Japanese sentences (`corpora/jpn_sentences.tsv.bz2`, fetched by `fetch_corpora.py`; katakana
counted as hiragana): after each kana and pair of kana, how often 。、？・ come next and how often
る ね め. So ありがとう `.` → ありがとう。, ありがとうございます。 (る after ます never comes), こんにちは
`,` → こんにちは、; and the kana stays where it's the word: たべ `.` → たべる (る after べ 13%, 。
0.1%), そうです `,` → そうですね, each with the mark offered below. A kanji row takes the mark too
(有難う。), weighed by the kana typed.

**[decide] Kana against Korean.** The tests that measure Korean (`tests/corpora.rs`,
`tests/long_context.rs`) load no Japanese words or kanji, which the shells do. With them, as
shipped, the corpora's weighted failure rate is 12.1% against 0.04% (12 of the owner's 115 words:
사과 reads かのすわ, 회 きゆ, 책 そらす; measured after §9.2's change of 2026-11-05 and retraining). The marks above change none of it; the kana reading
does (§5).

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

## 9. Mode A without symbols

Built 2026-11-02 (`engine/src/layout.rs`, `jis_korean` and `moonshot_letter`); drawn in
`docs/layout/jis-moonshot-layouts.html`. It breaks standard 2-set on Shift+Q W E R T O P, so the
core principle in `CLAUDE.md` and spec §1, spec §3's key map and the baseline
(`engine/tests/baseline.rs`) say so.

Mode A types three languages, so its symbol keys are better spent on Korean: symbols come from
無変換 (English), ー from the kana reading (F6 / F7), and ⌥ `,` `.` `/` `[` `]` still type 、 。 ・ 『 』
(§4). Every JIS Korean symbol leaves Mode A, with and without Shift, which frees the 12 keys right
of 0 as 24 slots.

**Refilled** (2026-11-03, the owner's call): 다 도, 나 니, ㅡㄹ 을, 무 부, ㅂㅅ, 메 and 로 로゜ were
little used, so the slots were freed and refilled with particles, endings and finals (§9.2). Shift+`.`
and Shift+`/` went with them, so Mode A has **no `.` or `·` on Shift** any more: · is on `¥`, and
a period came from English (無変換) or ⌥`.` (。). Shift+`¥` was left empty.

**Symbols back** (2026-11-05, the owner's call): with `.` 을/를 and `/` 서/에서, Shift+`,` `.` `/`
couldn't type punctuation. So `^` `¥` `@` `[` `]` `.` `/` `ろ` type their **stock JIS symbols**
again, alone and with Shift (^ ~, ¥ |, @ `, [ {, ] }, . >, / ?, \ _), and Korean keeps only `−`
`;` `:` `,`. The slots that went: 가 께서, ·, ㅄ ㄶ, the ㆍ break, ㄴ ㄹ, 을 를, 서 에서, 로 으로, all
typed the 2-set way again (ㅂ ㅅ, ㄴ ㅎ, ㅇ ㅡ ㄹ …). · is a symbol variant of ¥ (§5.2). The kana
reading and JIS Kana still read every key by its kana (`.` is る).

**Shift+`.` is a period** (2026-11-06, the owner's call): it typed `>`, while JIS Kana's Shift+`.`
is 。 and Mode A's Shift+`,` is a comma. Now `.` and Shift+`.` both type `.`; `>` comes from
English (無変換).

### 9.1 Notation

An action is a short run of 2-set jamo, typed as if those keys were pressed. How a legend is
written says how it types:

| Mark | Meaning | Example |
|---|---|---|
| (none) | a bare vowel, written without ㅇ: it joins the consonant before it like a vowel key, and gets the usual implied ㅇ with nothing before it | ㄷ + ㅙ → 돼; 말 + ㅙ → 마뢔 |
| (written with ㅇ) | the ㅇ is real: typed as an initial, so the action starts its own syllable (what Shift+vowel already does, spec §4.2) | 말 + 왜 (Shift+`−`) → 말왜 |

### 9.2 The keys

| Key | Alone | Shift | Was (JIS Korean) |
|---|---|---|---|
| `−` | ㅙ | 왜 | ! ? |
| `^` | ^ | ~ | ( ) |
| `¥` | ¥ | \| | — ^ |
| `@` | @ | ` | ' " |
| `[` | [ | { | [ 「 |
| `;` | ㅖ | 예 | ~ - |
| `:` | ㅒ | 얘 | : ; |
| `]` | ] | } | ] 」 |
| `,` | ㅢ | , | , , |
| `.` | . | . | . . |
| `/` | / | ? | · · |
| `ろ` | \ | _ | \ _ |

The symbol keys are the stock JIS ones since 2026-11-05 (above); from 2026-11-03 to then they
typed 가/께서, ·/(empty), ㅄ/ㄶ, ㆍ, ㄴ/ㄹ, 을/를, 서/에서 and 로/으로.

Examples: ㄷ + `−` → 돼; 말 + Shift+`−` → 말왜; ㄱ + `;` → 계; 말 + Shift+`;` → 말예; ㅎ + `,` → 희;
안녕하세요 + `.` → 안녕하세요.; 뭐해요 + Shift+`/` → 뭐해요?.

The letter keys change on Shift only:

- **Shift on a consonant key** (2026-11-03, the owner's call) types the consonant and **ㅏ**, as if
  both keys were pressed: ⇧ㄱ → 가, 하 ⇧ㄴ → 하나, 갑 ⇧ㅅ → 갑사. ㅃ ㅉ ㄸ ㄲ ㅆ stay on 1–5. This
  replaces "Shift on a consonant changes nothing" (2026-11-02), so Shift held over consonants is
  no longer a way of typing 2-set.
- **Shift+O / Shift+P** are Shift+vowel like every other vowel key: 애, 에, a syllable on a real ㅇ
  (간 ⇧ㅔ → 간에; ㄱ ⇧ㅐ → 개). ㅒ ㅖ moved to `:` and `;`.
- Legends write a Shift+vowel as its syllable (아, 우, 애): no marks.

The number row, the other letter keys, F6–F10 and the switch keys stay as they are. The kana
reading still reads every key by its kana legend (`@` is ゛ and `[` is ゜ there too).

### 9.3 How the engine types an action

- **Keys, not jamo.** An action's first jamo carries the key (its stock JIS character, or the
  letter); the rest are parts of it. **Backspace takes the whole key off**,
  as the kana reading counts keys. F10 reads each key once (`i` ⇧P `h o n e` F10 → iPhone).
- **No Shift held over.** Shift on a consonant types its ㅏ, so a Shift+vowel after it starts its
  own syllable on ㅇ like any Shift+vowel (⇧ㄱ ⇧ㅏ → 가아). (Until 2026-11-03 Shift on a consonant
  changed nothing, and a Shift+vowel right after one was read as Shift still held.)
- **An action's vowel isn't a press.** The same vowel typed right after an action's vowel isn't a
  double press (spec §4.2): 하 + `^` (가) + ㅏ → 하가아, not 학아.
- **The English reading** reads a slot key as its stock JIS character (`-` for ㅙ), as before.
  The standard 2-set reading (spec §10.2) types that character too.
- **Korean-only keys** (spec §10.3): a slot key that typed Korean inside the word means the keys
  aren't an English word, so it lifts the Korean reading over the English reading's capital rules
  and adds 3 bits per key (의약품 typed `,` ⇧I r v n a isn't ,Irvna). The last key counts only
  when the Korean reading is a known word (의 typed `d` `,`), so a habitual comma after English
  (`hello,`) still reads as English.
- **Kana evidence** still counts the 12 keys by key: they type kana-only legends (§5).
- **Capitals.** Shift on every consonant key types Korean again (consonant + ㅏ), so all 14 are
  Korean Shifts (`KOREAN_SHIFTS`): a capital may be a Korean syllable (⇧R → 가), and acronyms (ESG,
  CRM) compete with that reading. The key model scores a capital consonant as its key and ㅏ's
  (`R` as `r k`) on the Korean side.
- **Legends.** `examples/legends.rs` (the macOS Keyboard Viewer) prints actions in full (왜, 예).
  `jk_legend` / `jk_legend_in` return one code point, the legend's first (왜, 예),
  so the web playground's board shows that until it gets a string legend.

### 9.4 Open questions

- **[decide]** A string legend in the C ABI (`jk_legend_text_in`) for the web board and Windows.
- The key model (`ranker.bin`, whose Korean class still typed the slots of before 2026-11-03) and the
  learned ranker (`learned.bin`) were retrained on these keys on 2026-11-05, after the brand table
  (§5.4). Validation top-1: 95.73% learned against 95.47% by the rules. The corpora's weighted
  failure rate went from 0.08% to 0.04%; the long-context benchmark from 98.3% of words (89.9% of
  phrases exact) to 98.2% (89.5%).

