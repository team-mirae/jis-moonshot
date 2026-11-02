# JIS Korean: layout and behavior spec

> **JIS Moonshot.** This is JIS Korean's spec, kept for JIS Moonshot. What the fork changed (kana
> typing and the kana reading, JIS only, F6/F7, the switch keys, predictions always on, Shift+Z) is
> in [moonshot.md](moonshot.md), which **wins where the two differ**. The sections below that were
> plainly wrong for the fork are updated; the rest may still describe JIS Korean.

Status: **draft**. Source: the note "2-set on JIS layout spec (draft)" and its interactive
prototype (https://claude.ai/artifact/5TpYiuCYgZk8grLq6Sp4Sa). Where the note is silent, this
spec writes down what the prototype does. Those sections are marked *(from prototype)*. Decisions
this spec had to make that neither source settles are marked **[decide]** and also appear in
[Open questions](#9-open-questions).

JIS Korean is a Korean 2-set (두벌식, KS X 5002) input method for a JIS physical keyboard. It
extends 2-set over the full area that JIS kana typing uses (1 to ¥, down to ろ) and over F6–F10.
That is **Mode A**, the original layout, and everything below describes it unless it says
otherwise. (JIS Korean's Modes B and C, for US ANSI and UK ISO, are retired in JIS Moonshot, §3.6.)

---

## 1. Core principle

Every valid standard 2-set key and combination still types what it always did. A dedicated key
adds a shortcut; it never takes anything away.

- ㅗ then ㅏ is still ㅘ, even though 0 also types ㅘ.
- Double finals (ㄹ+ㄱ → ㄺ etc.) compose as usual.

JIS Moonshot moves the jamo standard 2-set puts on Shift (moonshot.md §9, 2026-11-02, the owner's
call): ㅃ ㅉ ㄸ ㄲ ㅆ are on 1–5 only, and ㅒ ㅖ on `:` and `;`. Shift on a consonant key types the
consonant and ㅏ (Shift+R → 가; moonshot.md §9.2, 2026-11-03, the owner's call), and Shift on every
vowel key, O and P included, starts a syllable on ㅇ (§4.2). Everything else on the letter keys is
standard 2-set, and `engine/tests/baseline.rs` holds it to that (Shift held over consonants is no
longer tested: it no longer types 2-set).

Two deliberate exceptions (§4.2):

- **Implied ㅇ:** a vowel with no initial types a full syllable (ㅏ → 아) instead of a bare jamo. The
  bare vowels are offered as a prediction (§10.2). (In JIS Korean F6 right after committed the bare
  jamo; in JIS Moonshot F6 is hiragana, moonshot.md §4.)
- **Shift+vowel** (vowel keys whose Shift types nothing else: all but O and P) after a syllable
  starts a new syllable on ㅇ instead of acting like the plain vowel (마ㄹ+⇧ㅣ → 말이, not 마리).

JIS Korean had a third, Shift+Z as F10. JIS Moonshot drops it: Shift+Z is 카, as Shift on any
consonant (and っ in kana, moonshot.md §1).

Any change that breaks this principle is a spec change, not an implementation detail.

---

## 2. Physical keys

Keys are identified by **physical position**, never by the character the OS layout would produce.
This spec uses W3C `KeyboardEvent.code` names as the canonical IDs. Every platform shell maps its
native key codes to these.

| JIS legend | Canonical code | macOS virtual key | Windows scan code |
|---|---|---|---|
| 1 … 0 | `Digit1` … `Digit0` | ANSI positions | 0x02 … 0x0B |
| − | `Minus` | `kVK_ANSI_Minus` | 0x0C |
| ^ | `Equal` | `kVK_ANSI_Equal` | 0x0D |
| ¥ (left of Backspace) | `IntlYen` | `kVK_JIS_Yen` (0x5D) | 0x7D |
| Q … P | `KeyQ` … `KeyP` | ANSI positions | |
| @ | `BracketLeft` | `kVK_ANSI_LeftBracket` | 0x1A |
| [ | `BracketRight` | `kVK_ANSI_RightBracket` | 0x1B |
| A … L | `KeyA` … `KeyL` | ANSI positions | |
| ; | `Semicolon` | `kVK_ANSI_Semicolon` | 0x27 |
| : | `Quote` | `kVK_ANSI_Quote` | 0x28 |
| ] | `Backslash` | `kVK_ANSI_Backslash` | 0x2B |
| Z … M | `KeyZ` … `KeyM` | ANSI positions | |
| , . / | `Comma` `Period` `Slash` | ANSI positions | 0x33 0x34 0x35 |
| ろ | `IntlRo` | `kVK_JIS_Underscore` (0x5E) | 0x73 |
| F6 … F10 | `F6` … `F10` | `kVK_F6` … `kVK_F10` | 0x40 … 0x44 |

The platform columns are a starting point. **Verify** them on real hardware during each shell's
first spike.

---

## 3. Key map

Unless stated otherwise, every output below is a **character** (it goes into the text as-is),
except Hangul jamo, which go into the composition automaton (§4).

### 3.1 Number row

| Key | Unshifted | Shift | Notes |
|---|---|---|---|
| 1 | ㅃ | 1 | |
| 2 | ㅉ | 2 | |
| 3 | ㄸ | 3 | |
| 4 | ㄲ | 4 | |
| 5 | ㅆ | 5 | |
| 6 | ㅟ | 6 | rarest of the five; on the key some typists reach with the left hand |
| 7 | ㅝ | 7 | |
| 8 | ㅚ | 8 | most frequent of ㅟ ㅝ ㅚ; middle finger |
| 9 | ㅢ | 9 | placed for ergonomics |
| 0 | ㅘ | 0 | JIS kana わ is on this key |
| − ^ ¥ | ㅙ 가 · | 왜, 께서, nothing | moonshot.md §9.2 (Shift+¥ is empty) |

Digits are on Shift, so numbers type with Shift held. The 12 keys right of 0 type the actions of
moonshot.md §9.2 (short runs of jamo, real-ㅇ syllables and ㆍ breaks, §9.1); JIS Korean's symbols on them are gone.

### 3.2 Q row

| Key | Unshifted | Shift | Notes |
|---|---|---|---|
| Q W E R T | ㅂ ㅈ ㄷ ㄱ ㅅ | 바 자 다 가 사 | Shift+consonant: the consonant and ㅏ (moonshot.md §9.2; the doubles are on 1–5, §1) |
| Y U I | ㅛ ㅕ ㅑ | ㅛ ㅕ ㅑ | standard 2-set (Shift: §4.2) |
| O P | ㅐ ㅔ | ㅐ ㅔ | Shift: §4.2 (ㅒ ㅖ are on : and ;, moonshot.md §9.2) |
| @ [ | ㅄ, ㆍ (break) | ㄶ, ㆍ | moonshot.md §9.2 (ㅄ ㄶ type their two jamo) |

### 3.3 A row

| Key | Unshifted | Shift | Notes |
|---|---|---|---|
| A S D F G | ㅁ ㄴ ㅇ ㄹ ㅎ | 마 나 아 라 하 | Shift+consonant (moonshot.md §9.2) |
| H J K L | ㅗ ㅓ ㅏ ㅣ | same | standard 2-set |
| ; : ] | ㅖ ㅒ ㄴ | 예 얘 ㄹ | moonshot.md §9.2 |

### 3.4 Z row

| Key | Unshifted | Shift | Notes |
|---|---|---|---|
| Z | ㅋ | 카 | Shift+consonant. JIS Korean made Shift+Z F10; JIS Moonshot doesn't (it's っ on the kana layout, moonshot.md §1) |
| X C V | ㅌ ㅊ ㅍ | 타 차 파 | Shift+consonant (moonshot.md §9.2) |
| B N M | ㅠ ㅜ ㅡ | same | standard 2-set |
| , . / | ㅢ 을 서 | , 를 에서 | moonshot.md §9.2; Korean keeps only , on Shift (· is on ¥) |
| ろ | 로 | 으로 | moonshot.md §9.2 |

### 3.5 Keys outside this map

Keys not listed (Space, Enter, Backspace, Tab, arrows, modifiers, other F-keys, keypad) keep their
normal behavior, except as §5 and §6 define. Any keystroke with Ctrl, Alt/Option or Cmd/Win held
**commits the composition first** and then passes through to the app unchanged **[decide]**.

### 3.6 Modes

A **mode** is one keyboard's key map. The Korean mode keeps standard 2-set on the 26 letter keys
(§1), F6–F10, the automaton (§4) and everything after §4. A mode only says what the other keys
type, and which stock keyboard sits under it. Mode A is the default.

| Mode | Name | Keyboard | Stock keyboard |
|---|---|---|---|
| A | JIS Moonshot | JIS | JIS |
| `A_KANA` | JIS Kana | JIS | JIS (the kana legends, moonshot.md §3) |

JIS Moonshot is **JIS only** (moonshot.md §2). JIS Korean's **Mode B** (US ANSI) and **Mode C** (UK
ISO), and their English modes `B_QWERTY` and `CJ_QWERTY`, are retired: their ids are never reused,
`jk_set_mode` refuses them, and a saved setting naming one falls back to Mode A. The kana reading
needs the JIS kana legends, which no other keyboard has. `Backquote` (the HHKB's `` ` `` key; 半角/全角
on a full JIS board) is 半角/全角 in every mode: full-width, Shift half-width (moonshot.md §4.1). `IntlBackslash`
isn't in Mode A: it commits and passes through (§3.5).

The stock keyboard (JIS) drives everything that reads keys as plain characters: the **English
readings** (`Engine::english`) and the **standard 2-set reading** (§10.2).

**JIS Kana** (`A_KANA`) isn't a 2-set mode: every key types its JIS kana legend, nothing is
predicted, and the staged keys show as hiragana or katakana (moonshot.md §2.1, §3).

#### 3.6.1 Mode ids and names

Every mode has three identifiers:

- **Id:** a number in the ranges below (0–99, and 10000–11000 for previews). Ids are saved in settings and passed over the
  C ABI (`jk_set_mode`), so an id is never renumbered or reused, even when its mode is removed.
- **Mode name:** what tools, settings and the public API (§8.2) use. Upper-case snake case: words of
  `A`–`Z` and `0`–`9` joined by `_`, starting with a letter (`A_QWERTY`,
  `SPLIT_ORTHO_LAYOUTDESIGNERNAME`). Unique, and never renamed once released.
- **Display name:** what a menu shows (`JIS English`). Free text, unique, and it can be translated.

Ids are in ranges; they don't have to be given in order:

| Ids | For |
|---|---|
| 0–14 | Official modes. 0–12 are taken, retired or reserved (below); 13–14 are held for future official designations. |
| 15–30 | Official modes contributed from outside, each accepted by a majority of the contributors. |
| 31–99 | Custom layouts: any free id, chosen by whoever makes the layout. Nothing coordinates them, so two forks can pick the same one. |
| 10000–11000 | Development and preview builds: a temporary id for a mode still being made. When it's done it gets a permanent id in its range (0–30 for an official mode), and the temporary one is dropped. Never saved by a release build's settings. |

Every other id is invalid for now (0–99 may grow later). Every platform can hold ids up to
2,147,483,647, so the ranges have room to grow.

**Ids are looked up, never counted.** Ids have gaps (0, 3, 7, 8, 11), and a build has only the
modes it was built with (the English modes need the `english` feature, on in every shell build;
retired and reserved ids have no mode). So the
engine keeps an explicit table of the modes it has, each with its id, and nothing assumes ids run
from 0 without gaps:

- **Listing:** `jk_mode_count` says how many modes this build has, and `jk_mode_id_at(index)` gives
  each one's id, in the table's order (`Mode::ALL` in Rust). The index is only a position in that
  list, never an id.
- **An unknown id is refused, not replaced.** `jk_set_mode` returns 1 and switches, or returns 0 and
  keeps the current mode for an id this build doesn't have. `jk_legend_in` gives no legend (0), and
  the name and mode-name calls give an empty string. A shell that loads a saved id this build
  doesn't have (say, Mode B saved by JIS Korean) falls back to Mode A itself, visibly.
- **Names:** `jk_mode_name_ptr`/`_len` give the display name, `jk_mode_code_ptr`/`_len` the mode name
  (`A`, `A_QWERTY`).

A mode name's first word is its **namespace**, and says which keyboard family the mode is for, as
Modes A, B and C do; the **id range** says who maintains it:

| Namespace | Keyboard family | Ids |
|---|---|---|
| `A_` | JIS (Mode A's) | 0–14 official, 15–30 contributed |
| `B_` | US ANSI (Mode B's): also minor adjustments on an ANSI board, such as Korean ANSI or a modified laptop keyboard | 0–14 official, 15–30 contributed |
| `C_` | ISO (Mode C's, UK): also other ISO locales | 0–14 official, 15–30 contributed |
| `CJ_` | ISO, also supporting JIS: designed for an ISO board, with JIS's extra keys bound (below) | 0–14 official, 15–30 contributed |
| Anything else (`COMPANYNAME_QWERTZ`, `COMPANYNAME_CUSTOMLAYOUTNAME`) | any | 31–99 |

- **Official** (0–14): Mirae Research's, shipped and maintained here.
- **Contributed** (15–30): accepted into this repository under the stricter guidelines in
  `docs/modes.md`: review, tests and docs as for an official mode, and a majority of the
  contributors in favour.
- **Custom** (31–99, any other prefix): whoever makes it, in a fork or their own build. Only the
  naming rules and §3.6's "what every mode keeps"; not reviewed here. Use a prefix of your own, such
  as your company or your name.

**Mode A keeps its one-letter name**, `A` (id 0), as released: tools and the public API already use
it. `B` and `C` (ids 1, 2) are retired. A one-letter name has no `_`, so it's in no namespace, and
no new mode gets one.

**How a `CJ_` mode is designed.** The European layouts are `CJ_` modes: their people mostly have
ISO boards, and JIS users can use them too. Every `CJ_` mode follows two rules:

1. **Build it for ISO.** The layout is the ISO one, as its people know it.
2. **Map the JIS keys.** A JIS board has two keys ISO lacks, ¥ (`IntlYen`) and ろ (`IntlRo`), and
   lacks two ISO keys: the extra key left of Z (`IntlBackslash`) and the key left of 1
   (`Backquote`, 半角/全角 on a JIS board). The mode gives ¥ and ろ a mapping; any mapping is
   allowed, and the mode's section says which. The default, used by `CJ_QWERTY`: **ろ types the
   ISO extra key's characters and ¥ those of the key left of 1**, so every character of the layout
   is reachable on JIS.

**Scope.** `A_` modes are for JIS boards, `B_` for US ANSI, and `C_` and `CJ_` for ISO (`CJ_` on
JIS too). No mode forbids any
keyboard, but support covers only the board family a mode is for: a `B_` mode that doesn't work on
an ISO board, for example, isn't an issue this repository will fix.

| Id | Mode name | Display name | Badge text | Status |
|---|---|---|---|---|
| 0 | `A` | JIS Moonshot | (keys) | Shipped |
| 1 | `B` | JIS Korean for US ANSI | | Retired |
| 2 | `C` | JIS Korean for UK ISO | | Retired |
| 3 | `A_QWERTY` | JIS English | ABC | Specified (§3.6.2) |
| 4 | `CJ_QWERTZ` | **[decide]** | QZ | Reserved |
| 5 | `CJ_SWISS_QZ` | **[decide]** | CH | Reserved |
| 6 | `CJ_AZERTY` | **[decide]** | AY | Reserved |
| 7 | `A_DVORAK` | JIS Dvorak | DV | Specified (§3.6.4) |
| 8 | `A_COLEMAK` | JIS Colemak | CMK | Specified (§3.6.4) |
| 9 | `B_QWERTY` | US English | | Retired |
| 10 | `CJ_QWERTY` | UK English | | Retired |
| 11 | `A_KANA` | JIS Kana | KA | Specified (moonshot.md §2.1, §3) |
| 12 | `A_ROMAJI` | (none yet) | | Reserved: romaji kana typing, the `KanaScheme::Romaji` slot (moonshot.md §2.1) |

A **retired** mode's id and name are never reused, and `jk_set_mode` refuses them. A **reserved**
mode has its id and name held for it, and nothing else yet: no key table, no build.
Ids 4–8 are Mirae Research's Latin-layout modes in the family of JIS English (§3.6.2): the European
layouts as `CJ_` modes (4–6), and Dvorak and Colemak as `A_` modes (7–8), which aren't tied to
European ISO boards. Each has its layout's letters, with the symbols still to be decided per mode
(Dvorak's and Colemak's are, §3.6.4), and a badge of the taegeuk with its badge text over the lower
right, as `ABC` on JIS English's. Ids 13–14 are held for the next official modes.

#### 3.6.2 `A_QWERTY`: JIS English

**JIS English** types English: the standard JIS QWERTY layout, with JIS English legends and JIS
symbols, on any keyboard (Shift+2 is `"`, the key right of P is `@`, `IntlYen` is `¥`, `IntlRo` is
`\`). It's the Mac's ABC for JIS Korean users: an English input source in the same input method as
the Korean modes, so switching between them never leaves JIS Korean (§8.2.1).

It isn't a 2-set mode (nor are ids 4–8): none of §3.1–3.5 or §4 onward applies. It behaves like
the Japanese input method's Romaji mode without its Japanese conversions: the engine consumes no key
(every key goes to the app, typed by the mode's keyboard layout, §8.1) and composes nothing; no
predictions, hanja or F6–F10: F9 and F10 go to the app, Shift+Z types Z and Shift+X types X.
Caps Lock types capitals. The one thing it adds is letter variants on ⌥ (§3.6.3). Switching in commits whatever a Korean mode left unconfirmed. The core principle (§1) is about the Korean modes and
is unaffected. The engine still names the layout: its key table is the JIS stock table (Mode A's
stock keyboard), which the macOS keyboard layout is generated from, so the symbols can't drift from
Mode A's stock keyboard.

JIS English, JIS Dvorak and JIS Colemak (§3.6.4) are the **English modes**. JIS Korean's US English
(`B_QWERTY`) and UK English (`CJ_QWERTY`) are retired with Modes B and C.

They're **always built** in JIS Moonshot (the engine's `english` feature is on by default and in
every shell build): 英数 / 無変換 need them (moonshot.md §2, §4).

#### 3.6.3 Letter variants (⌥ + letter)

**macOS only, deliberately.** Other platforms leave it out: on Windows Alt + letter is the app's
menu and shortcut key, used far too much to take (§8.4).

In the English modes, **⌥ + a letter** offers the letter's accented and other variants, macOS's
press-and-hold set. Nothing is typed first: the letter shows as the preedit with its list beside
it, none selected. The list runs horizontally, since every variant is a single letter. With Shift
(or Caps Lock), the capitals:

| Letter | Variants (1, 2, …) | With Shift (or Caps Lock) |
|---|---|---|
| a | à á â ä æ ã å ā | À Á Â Ä Æ Ã Å Ā |
| c | ç ć č | Ç Ć Č |
| e | è é ê ë ē ė ę | È É Ê Ë Ē Ė Ę |
| i | î ï í ī į ì | Î Ï Í Ī Į Ì |
| l | ł | Ł |
| n | ñ ń | Ñ Ń |
| o | ô ö ò ó œ ø ō õ | Ô Ö Ò Ó Œ Ø Ō Õ |
| s | ß ś š | Ś Š |
| u | û ü ù ú ū | Û Ü Ù Ú Ū |
| y | ÿ | Ÿ |
| z | ž ź ż | Ž Ź Ż |

While the list is open:

- **1–9 and 0** pick the first ten variants (0 is the tenth). A variant past the tenth has no
  digit: choose it with the arrows. A digit past the list keeps the letter and is typed after it.
- **The same ⌥ + letter again** chooses the next variant, wrapping. Its auto-repeat
  (`JK_MOD_REPEAT`) does nothing.
- **The arrows** choose (the preedit shows the choice); from none, → and ↓ go to the first, ← and ↑
  to the last, wrapping. **Enter** commits the choice, or the letter with none chosen, and stops
  there.
- **Esc** and **Backspace** cancel: nothing is typed.
- **Any other key** (Space, a letter, a shortcut, another ⌥ + letter) keeps what shows, the choice
  or the letter, and goes on to the app (another ⌥ + letter opens its own list).

⌥ on a letter without variants types what ABC's ⌥ layer has there, and ⌥ with ⌘ or Ctrl is a
shortcut. Holding a key repeats it as usual: there's no press and hold, which would need the app to
take back the letter already typed, and many apps (terminals, Discord) can't. In a terminal, ⌥ goes
to the app untouched (readline's ⌥C, ⌥U and ⌥L, with Option as Meta). The Korean modes ignore ⌥ and
`JK_MOD_REPEAT`.

Examples (key codes; `A-` is ⌥): `A-KeyU Digit2` → ü; `A-S-KeyU Digit2` → Ü; `A-KeyE A-KeyE A-KeyE
Enter` → é; `A-KeyE ArrowLeft Enter` → ę; `A-KeyA KeyB` → ab; `A-KeyO Escape` → nothing.

Examples: `KeyG KeyK KeyS` → `gks`; `S-Digit2` → `"`; `BracketLeft` → `@`; `IntlYen` → `¥`.

#### 3.6.4 `A_DVORAK` and `A_COLEMAK`: JIS Dvorak and JIS Colemak

**JIS Dvorak** (id 7) and **JIS Colemak** (id 8) are JIS English (§3.6.2) with Dvorak's and
Colemak's letters. They only move JIS English's keys: every symbol keeps its JIS Shift pair, and a
symbol key moves only when a letter takes its place. The number row, `@ [ : ]`, ¥ and ろ stay where
they are.

| Key (JIS English) | JIS Dvorak | JIS Colemak |
|---|---|---|
| Q row: `q w e r t y u i o p` | `/ , . p y f g c r l` | `q w f p g j l u y ;` |
| A row: `a s d f g h j k l ;` | `a o e u i d h t n s` | `a r s t d h n e i o` |
| Z row: `z x c v b n m , . /` | `; q j k x b m w v z` | `z x c v b k m , . /` |

- **Dvorak**: the four JIS symbol keys its letters displace go to its punctuation slots: `, <` on
  W, `. >` on E, `; +` on Z, and `/ ?` on Q, where Dvorak has `' "` (JIS types those on Shift+7 and
  Shift+2).
- **Colemak**: `; +` goes to P, Colemak's semicolon. Caps Lock stays Caps Lock (§8.2.1);
  Colemak's Backspace on Caps Lock isn't adopted.
- **⌘ and ⌥ use JIS English's keys**, so shortcuts stay where they are on JIS QWERTY: ⌘C is the
  key JIS English types c on, whichever letter the mode types there. ⌥ types what it does in JIS
  English, except on the letters with variants (§3.6.3), which follow the letter the key types. **Ctrl** does the same by default. **[decide]** Apple's Dvorak – QWERTY ⌘ keeps Ctrl on
  Dvorak's letters instead (Ctrl+A, Ctrl+E in a terminal).

Everything else is JIS English: the engine consumes no key and composes nothing, and ⌥ (§3.6.3)
offers the variants of the letter the key types, not of its JIS English letter (in JIS Dvorak, ⌥ on
the F key, which types u, offers ü). They're English modes, built with the `english` feature like the others.

---

## 4. Hangul composition

### 4.1 Automaton

The standard 2-set automaton, with initial (초성), medial (중성) and final (종성) slots:

- **Initials:** ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ
- **Medials:** ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ
- **Finals:** ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ (ㄸ ㅃ ㅉ are never finals)
- **Compound medials by sequence:** ㅗㅏ→ㅘ, ㅗㅐ→ㅙ, ㅗㅣ→ㅚ, ㅜㅓ→ㅝ, ㅜㅔ→ㅞ, ㅜㅣ→ㅟ, ㅡㅣ→ㅢ
- **Double finals by sequence:** ㄱㅅ→ㄳ, ㄴㅈ→ㄵ, ㄴㅎ→ㄶ, ㄹㄱ→ㄺ, ㄹㅁ→ㄻ, ㄹㅂ→ㄼ, ㄹㅅ→ㄽ,
  ㄹㅌ→ㄾ, ㄹㅍ→ㄿ, ㄹㅎ→ㅀ, ㅂㅅ→ㅄ

Transitions:

| State | Input | Result |
|---|---|---|
| empty | consonant | start syllable with it as initial |
| empty | vowel | new syllable on an implied ㅇ (§4.2): ㅏ → 아 |
| initial only | vowel | set medial |
| initial only | consonant | flush the lone initial, start a new syllable |
| initial + medial | vowel that combines | compound medial |
| initial + medial | vowel that doesn't combine | new syllable on an implied ㅇ (§4.2): 가+ㅓ → 가어 |
| initial + medial | consonant that can be a final | set final |
| initial + medial | consonant that cannot be a final (ㄸ ㅃ ㅉ) | flush, start new syllable |
| with final | consonant that forms a double final | double final |
| with final | vowel | **move the (last) final to the next syllable:** 닭+ㅏ → 달가, 한+ㅏ → 하나 |
| with final | other consonant | flush, start new syllable |
| any | non-jamo character | flush, then emit the character |

A dedicated compound key (6–0) inputs that compound medial **directly**. It behaves exactly like
the same medial built by sequence, except that nothing combines further with it: ㅘ+ㅣ does not
become ㅙ (ㅙ is reached as ㅗ+ㅐ, as usual) *(from prototype)*, so a dedicated compound key after a
vowel always starts a new syllable (고+0 → 고와). A dedicated double-consonant key
(1–5) is identical to Shift+Q/W/E/R/T.

Output is precomposed Hangul syllables (U+AC00–U+D7A3) when an initial and a medial are present,
and compatibility jamo (U+3131–U+318E) otherwise.

### 4.2 Implied ㅇ

A vowel that can't attach to an initial gets an implied ㅇ and starts a syllable: at the start of
the segment, after a symbol, or after a syllable whose vowel it can't combine with.

| Keys | Result |
|---|---|
| ㅏ | 아 |
| ㅗ ㅏ | 와 (the vowels still combine) |
| ㄱ ㅏ ㅓ | 가어 |
| ㄱ ㅗ 0 | 고와 |
| ㅏ ㄴ ㅏ | 아나 (the implied ㅇ behaves like a typed one) |

**F6 right after** commits with the implied ㅇ removed: ㅏ F6 → ㅏ, 가어 F6 → 가ㅓ,
와 F6 → ㅘ. This works while the syllable has no final yet. Once it has one (안), F6 commits it as
it is. Being a commit, F6 ends everything: the next vowel gets an implied ㅇ again
(ㅏ F6 ㅏ → ㅏ, then 아 unconfirmed).

**Same vowel again keeps the final.** A vowel after a final takes it (말 ㅣ → 마리), but the
writer may have meant 말이. Pressing the **same vowel key again right away** puts the final back
and starts the vowel's own syllable on an implied ㅇ:

| Keys | Result |
|---|---|
| ㅁ ㅏ ㄹ ㅣ | 마리 |
| ㅁ ㅏ ㄹ ㅣ ㅣ | 말이 |
| ㄷ ㅏ ㄹ ㄱ ㅏ ㅏ | 닭아 (a double final goes back whole) |
| ㅁ ㅏ ㄹ 0 0 | 말와 |
| ㅁ ㅏ ㄹ ㅣ ㅣ ㅣ | 말이이 (only the first repeat toggles) |
| ㅁ ㅏ ㄹ ㅣ ㅏ | 마리아 (a different vowel doesn't toggle) |
| ㅁ ㅏ ㄹ ㅣ F6 ㅣ | 마리 committed, then 이 (F6 commits in between) |

The ㅇ from a double press was asked for, so F6 commits it as it is (말이 F6 → 말이), unlike a
plain implied ㅇ. Backspace removes the
repeat and gives the first reading back (말이 → 마리).

**Shift+vowel starts a syllable on ㅇ.** Shift on a vowel key (all of them in JIS Moonshot: Y U I O
P H J K L B N M; JIS Korean left out O and P, which typed ㅒ ㅖ) finishes the syllable before it as it is, final
included, and starts a new syllable with ㅇ and that vowel. It's the single-keystroke alternative
to the double press, and it also keeps two vowels apart that would otherwise combine.

| Keys | Result |
|---|---|
| ㅁ ㅏ ㄹ ⇧ㅣ | 말이 (plain ㅣ: 마리) |
| ㄷ ㅏ ㄹ ㄱ ⇧ㅏ | 닭아 (a double final stays whole) |
| ㅗ ⇧ㅣ | 오이 (plain ㅣ: 외) |
| ㄱ ㅏ ㄴ ⇧ㅗ ㅏ | 간와 (the new vowel can still combine with the next one) |
| ⇧ㅏ | 아 (same as the implied ㅇ) |
| ⇧ㄱ ⇧ㅏ | 가: an initial waiting for its vowel takes it as usual, so Shift held over from a consonant is harmless |
| ㄱ ㅏ ⇧ㄱ ⇧ㅏ ㅇ ㅣ | 가가이: right after a consonant typed with Shift (which changes nothing in JIS Moonshot, §1), Shift on a vowel may just be held over, so the vowel takes the consonant as it takes any final. Also `ㅂ ㅣ ⇧ㅅ ⇧ㅏ` → 비사 |
| ㅇ ㅣ 5 ⇧ㅓ, ㅇ ㅣ 5 ㅓ | 있어, 이써: after the dedicated ㅆ key (5), a Shift is new |
| ㄱ ⇧ㅐ, 간 ⇧ㅔ | 개, 간에: O and P are vowel keys like the rest (ㅒ ㅖ are on : and ;) |

The ㅇ was asked for, so F6 commits it as it is (unlike an implied ㅇ). Backspace removes the
whole Shift+vowel in one press. The English reading keeps the shifted letter (⇧L → `L`).

### 4.3 Backspace

While composing, Backspace removes the **last jamo keystroke** and recomposes:
간 → 가 → ㄱ → (empty). A compound medial typed as a sequence comes apart one key at a time
(과 → 고), and one typed with a dedicated key goes in one press (과 via 0 → ㄱ) *(from prototype)*.
An implied ㅇ isn't a keystroke, so it goes with its vowel (가어 → 가). A slot key (moonshot.md §9)
comes off whole, as one key. None types several jamo since the 2026-11-03 slots were freed.
With nothing unconfirmed, Backspace passes through to the app.

---

## 5. Unconfirmed text and commit

The engine keeps an **unconfirmed segment**: the text typed since the last commit, shown in the
app as preedit (marked text / composition string). It holds composed Hangul, the syllable still
being composed, and pending symbols (§5.3). This is closer to the Japanese IME than to a per-syllable
Korean IME, and it is what F9/F10 need **[decide]**.

### 5.1 Commit triggers

| Input | Effect |
|---|---|
| Space | Commit the segment, then type a space. Space **never converts** (unlike the Japanese kana IME). |
| F6 / F7 | No commit: the staged keys show as hiragana / katakana, in JIS Kana (§6, moonshot.md §4). |
| Shift+Enter | Commit the segment. Nothing else is typed. *(planned)* |
| Enter | Default: commit the segment, then pass Enter to the app. Setting "Enter commits": commit only, as Shift+Enter does. *(planned)* **[decide]** |
| Focus loss, app switch, IME switch | Commit the segment. |
| Ctrl/Alt/Cmd/Win shortcuts | Commit, then pass through (§3.5). |
| ↓ / Tab, ↑ / Shift+Tab | With predictions showing more than one reading (§10): move through them (↓ and Tab next, ↑ and Shift+Tab previous, wrapping); Enter then commits the chosen one and Esc goes back to none chosen. Otherwise: commit, then pass through. As the macOS shell does; Windows matches. |
| ← / →, Esc | Commit, then pass through. |

### 5.2 Words

A **word**, for F9/F10, is the trailing run of jamo keystrokes in the segment that each came from a
letter or digit key. A symbol, a digit typed with Shift or a conversion ends the
run *(from prototype)*. Each keystroke keeps its **raw character**: the Latin letter of a letter
key (uppercase when typed with Shift), or the digit of a number-row key (so key 1 = ㅃ has raw
character `1`).

### 5.3 Symbols and digits with F9/F10

Typed symbols and Shift+digits stay in the segment like everything else. When the segment **ends in
a run of typed symbols or digits**, F9/F10 convert that run instead of a word: `2026` F9 →
`２０２６`, 한! F9 → 한！. The run and a word never mix: a word ends at the first symbol, and a
symbol run ends at the first jamo. Case cycling and F6 (restore and commit) work the same way.

Symbols convert **as typed**, never as the stock key's character: F9 makes every character that
has a full-width form full-width, and F10 every character that has a half-width form half-width.
A character without one stays as it is (· and — have none: `!·!` F9 → `！·！`). The full-width form
of U+0021–U+007E is the code point + 0xFEE0; space becomes U+3000; the Halfwidth and Fullwidth
Forms pairs ¢ £ ¬ ¯ ¦ ¥ ₩ and ｡ ｢ ｣ ､ ･ (。「」、・ full-width) convert both ways, so 「 F10 → ｢.

---

## 6. Function keys

| Key | Label | Behavior | Status |
|---|---|---|---|
| F6 | ひらがな | **Hiragana:** to JIS Kana (`A_KANA`), the staged keys read as kana and shown in hiragana; nothing is committed. Right after F9/F10 it undoes the conversion first; an open hanja list closes. With nothing staged it still switches (consumed). Shift+F6 with something staged does nothing (reserved). | moonshot.md §4 |
| F7 | カタカナ | **Katakana:** the same, shown in katakana. | moonshot.md §4 |
| F8 | 漢字 | **Kanji:** the staged keys' kana reading converted, the conversions on top of the predictions until the next commit (§6.1). With nothing staged it's still taken (the next segment). | moonshot.md §4, §5.1 |
| Shift+F8 | 漢字 | The same as F8 (the hanja list is gone, §7). | moonshot.md §4 |
| F9 | 전각영숫자 | Convert the current word to full-width alphanumerics, from the raw characters (§5.2). | done in prototype |
| F10 | 반각영숫자 | Same, half-width. (In JIS Korean Shift+Z did the same; in JIS Moonshot it's ㅋ, §3.4.) | done |

**With nothing unconfirmed, F9, F10 and Shift+F6–F8 go to the app**: they act only on
unconfirmed text. F6, F7 and F8 are the exception: they apply to the next segment even with nothing
staged, so kana and kanji are reachable without typing something first (moonshot.md §4,
**[decide]**). In an
English mode every F-key goes to the app.

### 6.1 F8 (kanji)

F8 used to escape to standard 2-set over the stock keys, for the symbols JIS Korean has no key for.
It was retired on 2026-10-27, before release: it wasn't standard 2-set (the implied ㅇ, the double
press and F6 still applied), and symbols off the legends are typed in an English mode (§3.6.2) or
the platform's English input source. The engine keeps each mode's stock keyboard for the English
and standard 2-set readings (§10.2). Since 2026-11-01 F8 is **kanji** (Shift+F8 too since the hanja list
went, §7): moonshot.md §4 and §5.1 define it.

### 6.2 F9 / F10 cycling *(from prototype)*

- The first press converts the word as typed (raw case kept).
- Repeated presses of the same key cycle **as typed → UPPER → Capitalized → as typed …**
- Pressing the other key switches width and keeps the current case step.
- F6 or F7 right after either key undoes the conversion, then shows the keys as kana (moonshot.md
  §4); かな / 変換 undoes it and forces Korean.
- Any other input ends the conversion state. The converted text stays in the segment as plain
  characters.
- With no word to convert, F9/F10 do nothing (with nothing unconfirmed they go to the app, §6).

Example: ㅎ ㅏ ㄴ (keys G K S) → 한. F10 → `gks`. F10 → `GKS`. F10 → `Gks`. F9 → `Ｇｋｓ`. F6 → きのと.

---

## 7. Hanja (removed)

JIS Moonshot has no hanja (2026-11-02, moonshot.md §5.1): no hanja dictionary, no hanja list and no
hanja readings. Shift+F8 does what F8 does (kanji, §6.1). JIS Korean's hanja support (a libhangul /
Unihan dictionary in `engine/dict/`, a list on F7 for the longest dictionary word at the end of the
unconfirmed text) lives on upstream.

---|---|---|
| no list | Shift+F8 | In Mode A, open a list for the **longest dictionary word at the end of the unconfirmed text**: 나는학교 → 학교. Only whole syllables count. No match, or no dictionary: nothing happens. (F7 in JIS Korean; F7 is now katakana, §6, and Shift+F7 reserved.) |
| list open | Shift+F8, ↓ | Next candidate (wraps) |
| | ↑ | Previous candidate (wraps) |
| | → / ← | Next / previous page of 9 |
| | 1–9 (number row, no Shift) | Pick that candidate on the current page |
| | Enter | Pick the selected candidate. No new line. |
| | Esc, Backspace | Close the list; the Hangul stays unconfirmed and editable |
| | F6, F7 | Close the list, then hiragana / katakana as usual (§6) |
| | anything else | Pick the selected candidate, then handle the key as usual (가 Shift+F8 Space → 可 committed + space) |
| | focus loss | Commit the selected candidate |

While the list is open, the preedit shows the **selected candidate in place** of the reading. A
pick stays unconfirmed like any other text. Earlier syllables keep their jamo, so Backspace still
works on them by jamo. When the reading doesn't start on a keystroke boundary (말이 typed with
the double press, then Shift+F8 on 이), the text before it becomes plain characters first.

Shells show the page that holds the selected candidate, numbered 1–9, with comments. The web
playground draws it under the caret; the macOS shell for now shows only the inline preview.

---

## 8. Platforms

All behavior above lives in `/engine`. Platform shells only translate between OS events and the
engine:

- **macOS** (`/mac`): an InputMethodKit input method.
- **Windows** (`/windows`): a TSF text input processor. It reads scan codes, so the keys are the
  same whatever layout Windows has loaded; on a JIS keyboard, the Japanese (106/109) hardware
  keyboard setting makes ¥ and ろ arrive. Password fields and the secure desktop do not load TIPs.
  That is a known limit. Modes, 한/영 and settings: §8.4.
- **Web** (`/web`): a React playground (a demo) that runs the same engine in the browser, keyed on
  `KeyboardEvent.code`. On non-JIS keyboards, `IntlYen` and `IntlRo` don't exist, so the page needs
  on-screen keys for them.

Android and iPhone are out of scope. JIS Korean's Minecraft mod is dropped in JIS Moonshot (its
corpus tools moved to `engine/tools/corpora/`).

**Language keys** (moonshot.md §4). Two keys of the JIS keyboard select where typing goes, whatever
is selected now:

| Key | macOS / Windows | Goes to |
|---|---|---|
| `LANG_EN` | 英数 / 無変換 | English: the English mode used last (§3.6.2), committing what shows |
| `LANG_KO` | かな / 変換 | Mode A, the predictive mode. Pressed again in Mode A with something staged: **force Korean** (the staged keys' Korean reading, nothing predicted); nothing staged: nothing. From JIS Kana: to Mode A, the staged keys forced Korean |

There is no `LANG_JA` any more: kana is part of JIS Moonshot (F6 hiragana, F7 katakana, §6), not
handed to the platform's Japanese input method. Each platform says how it catches the keys (macOS:
§8.2.1; Windows: §8.4). On Windows 한/영 (`VK_HANGUL`) still toggles English ↔ Mode A. The engine
handles `LANG_KO` itself (`Engine::kana_key`, `jk_kana_key`), and the engine can change its own mode: shells read
`jk_mode` after each call and select the matching input source; switching to English is the
shell's, like Ctrl+Space.

---

## 9. Open questions

From the note:

1. **Symbols with no key.** Compared with stock JIS, these are now unreachable:
   # $ % & * + = | @ ` { } < > / and ¥. Most urgent: @ and / (email addresses, URLs, dates). They're
   typed in an English mode since the F8 escape was retired (§6.1); in Mode B, Shift+1–0 type
   digits, so ! @ # $ % ^ & * need English too. ~~( )~~ Settled: on the ^ key (( unshifted, ) with Shift).
2. ~~**Free key.**~~ Settled: the ^ key types ( and ); ^ moved to Shift+¥.
3. ~~**~ vs -.**~~ Settled: ~ stays on the ; key unshifted; the ^ key went to ( ) instead.
4. ~~**F8 and composition.**~~ Settled: the F8 escape is retired; F8 is reserved (§6.1).
5. ~~**F6 vs Shift+Enter.**~~ Settled: F6 commits everything unconfirmed (after dropping a plain
   implied ㅇ or undoing F9/F10). Shift+Enter commits as-is.
6. **Shift on the jamo number keys.** Shift on 1–0 types digits, and the jamo on those keys exist
   only unshifted. Confirm this is final.

Raised by this spec:

7. ~~**Segment model (§5).**~~ Settled 2026-10-27: the whole segment stays unconfirmed. Wrong
   readings it commits are fixed at the source, by the user's negative vocabulary (§10.12 `avoid`).
   Keep the whole segment unconfirmed (Japanese style, needed for F9 over
   whole words and for pending symbols), or commit each finished syllable (Korean style, what apps
   and users of Korean IMEs expect)?
8. **Plain Enter default (§5.1).** Commit + newline by default, with "Enter commits" as the opt-in.
   Is that the intended reading?
9. **Digits inside a word (§5.2).** Shift+digit ends the F9 word, so `abc123` converts as `abc`
   then `123` (each with its own F9/F10, §5.3). Is that enough?
10. ~~**Arrows, Tab, Esc (§5.1).**~~ Settled 2026-10-27: with predictions showing, ↓ ↑ Tab move
    through them as on macOS; otherwise they commit and pass through, and Esc does too (it doesn't
    cancel). A top reading the user never means is avoided with §10.12 `avoid`.
11. ~~**Non-JIS hardware.**~~ Settled: Mode B (§3.6) for US ANSI. It keeps Mode A's number row
    and brackets and leaves every other key as US ANSI has it.
12. ~~**`B_` and `C_` namespaces.**~~ Settled (§3.6.1): `B_` varies Mode B on US ANSI boards,
    `C_` varies Mode C on ISO boards; each is supported on its board type only.
13. **JIS English on other platforms (§3.6.2).** Partly settled for Windows: no English input
    mode of its own. JIS English is a macOS workaround (switching English → Korean through
    Hammerspoon could switch back by itself); Windows has 한/영 inside the one input method, and
    the English modes are only the layout 한/영 types (§8.4), without letter variants (§3.6.3,
    macOS only). Minecraft maps an English mode to the Korean mode on the same keyboard (§8).
    Still open: should the web playground offer `A_QWERTY`?

---

## 10. Prediction *(draft)*

A layer on top of the deterministic engine that ranks **readings of the staged, uncommitted
text**. It never changes what the keys do; it only offers alternatives to commit. In JIS Moonshot
it's **always on** in the shells (§10.1).

### 10.1 Lite and full

| | Lite | Legacy | Full |
|---|---|---|---|
| Build | `cargo build --no-default-features` | `--features legacy-predict` | default (`predict` feature) |
| macOS / Windows | none (engine only) | `make LEGACY_PREDICT=1` / `build.ps1 -Legacy` | `make` / `build.ps1` |
| Ranking | — | the rules (§10.3) | the learned ranker (§10.13) on top of the rules |
| Model files | — | `ranker.bin`, word lists | also `learned.bin` (2 MB) |
| Run time | — | prediction is on only after the host loads a predictor (`jk_predictor_new`, `jk_set_predictor`); with none set, behaves exactly like lite | the same; without `jk_predictor_with_learned` the rules rank, as in legacy |
| What the user gets | §1–§7 only: Space commits the JIS Korean reading | ranked readings to choose from, in addition | the same readings, ranked better |

**JIS Moonshot's shells always load the predictor** (moonshot.md §3, §6): there is no Predictions
switch, no `PREDICT=0` and no `-Lite`. The user accepts predictive typing once, before JIS Moonshot
types anything; until then every key passes through. Learning (§10.11) stays a separate opt-in, and
the per-app exclusions (§10.11.1) stay: in those apps Mode A types forced Korean. The lite engine
remains a build the tests and CI check (`cargo test --no-default-features`), and with no predictor
set the engine still behaves exactly like it.

### 10.2 Readings (candidates)

| Kind | Reading | Example (keys → text) |
|---|---|---|
| Korean | JIS Korean, as composed | `gksrmf` → 한글 |
| English | the same keys on QWERTY (the mode's stock symbols, §3.6) | `gksrmf` → gksrmf |
| English, Capitalized / UPPER | case variants, **only when English ranks first** | `hello` → Hello, HELLO |
| Standard 2-set | plain 2-set on the mode's stock keyboard (JIS): no implied ㅇ, no double press, Shift+vowel is just the vowel, number-row keys type digits | `bb` → ㅠㅠ (JIS Korean: 유유); `rk1` → 가1 |
| Korean, finals kept | each vowel after a final starts its own syllable on ㅇ | `akfl` → 말이 (JIS Korean: 마리) |
| Bare vowels | JIS Korean without the implied ㅇ (§10.3) | `bb` → ㅠㅠ, `nn` → ㅜㅜ |
| English + particle | an English start with a Korean particle or ending (§10.3); after an acronym, a Korean word; or a Korean start with an acronym | Anthropic에서, report에, RLHF에, server를; LG에너지솔루션은; 한전KPS |
| Implied ㄹ after ㄴ | an ㄹ before a vowel a ㄴ final moved onto (§10.5) | `wjsur` → 전력 (JIS Korean: 저녁) |
| ㄶ by sound | ㄶ and ㄱ / ㄷ / ㅈ where a ㄴ final met ㅋ / ㅌ / ㅊ (§10.5) | `dkszh` → 않고 (JIS Korean: 안코) |
| Hanja | **Not in JIS Moonshot** (moonshot.md §5.1: kanji conversions instead; hanja from Shift+F8). Was: the top dictionary form of the **whole** Korean reading, 2+ syllables only; plus the everyday exceptions (§10.4) | 학교 → 學校, 2026년 → 2026年 |
| Kana | the JIS kana of every staged key, composed (゛ ゜, moonshot.md §3.1), in hiragana. Shift on the number row, E and Z gives small kana, Shift+0 を; a つ before a kana of the か, さ, た or ぱ rows may also be read as っ | `G K S` → きのと |
| Katakana | the kana reading in katakana, offered instead of hiragana when that is the known word | コーヒー |
| Kanji | the kana reading converted to kanji, up to three, when a Japanese reading is in the top 2 (moonshot.md §5.1) | わたしは → 私は |

Duplicates are removed (most readings coincide for most words), and **at most 4** are offered, plus
the kana reading in one extra slot. The kana reading always uses the JIS kana layout, and it's scored
as moonshot.md §5 says (its own lexicon, `engine/predict/common-ja.txt`; the learned ranker doesn't
score it, and it's merged into the list by score).

### 10.3 Ranking

Scores are in bits per key, positive leaning Korean:

1. **Model:** the same keys under two key-trigram models, English spelling vs JIS Korean typing
   (`engine/predict/ranker.bin`, built by `engine/tools/corpora/build_ranker.py`; held-out accuracy
   about 99% on words, 96% on real Korean text). The same file then carries JIS Moonshot's
   three-language order-4 key model (`JKR3`, moonshot.md §5), which so far scores only the kana
   reading. English nouns train at 3× weight.
   **Nouns weigh more than verbs:** an English reading that is a noun in its normalized form
   (Princeton WordNet 3.0 lemmas, 3+ letters: report, server, model) gets a bonus; verbs,
   adjectives and inflected or other forms (published, reports) don't. In mixed Korean text English
   words are mostly nouns. Cost on 8,774 real Korean words: 0.73% → 0.76% ranked as English (0.81% after the corpus retrain, §10.9).
2. **Rules:** a capital at the start of a word on a key Korean never shifts means English. On a
   vowel key it may be Shift+vowel starting the word (있습니다 as `L T t m q s l e k`): unless the
   keys spell a known English word, it then leans English by 2 bits instead of settling it, and a
   known Korean word cancels the lean. Two capitals in a row (an acronym) still settle it: a word
   typed in capitals only, two letters or more, is English (KB, not 아유; OEM, not 얘뜨), unless
   its Korean reading is a common word (a count of 20 or more, §10.9: 쓰 typed `5 ㅡ`, 빠 typed
   `1 ㅏ`; in JIS Moonshot ⇧T ⇧M is TM, since Shift on Q W E R T types no double, moonshot.md §9),
   and so is an English start of two capitals before a particle (KPI와).
   **Shift+consonant** (moonshot.md §9.2, 2026-11-03) types the consonant and ㅏ, so a capital on a
   consonant key can be Korean too (`KOREAN_SHIFTS`; the key model reads `R` as `r k` on the Korean
   side). Two capitals in a row with a consonant among them are still an acronym (ESG, KPS), unless
   the Korean reading is a known word; Shift+vowels in a row (분야의 as `q n s ⇧I ⇧M l`) stay Korean.
   A Shift+vowel start that is no English word splits off as English (before a particle) only when
   it's longer than two letters and the Korean reading isn't a known word: 오픈 typed `⇧H v m s` isn't
   Hv은, while Kotlin은 still splits.
   **Korean-only keys:** a key right of 0 that typed Korean inside the word (ㅢ on `,`,
   moonshot.md §9.3) lifts Korean over these capital rules and adds 3 bits per key; the last key
   counts only when the Korean reading is a known word, or when it typed a consonant first (ㄴ on
   `]`, 가 on `^`): no habit types those characters after English the way `,` (hello,) or `.` is.
   After any key of the word, a number-row key included (있 as `l 5`), a capital on a vowel key is
   Shift+vowel. **Digits then a Korean counter** (개 년 월 일 원 명 건 억 조 배 개월 년간 분기 …,
   maybe with a particle: 3년간, 2개의) or a known Korean word are Korean, at least one step: 2개,
   not `2ro`; 3년, not `3sus`. Exact
   matches in `predict/common-ko.txt` / `common-en.txt` lean their way; leftover bare jamo count
   against Korean; a command name (after `/`) is English; the previous committed word's language
   tips close calls. A known Korean word against keys that spell no known English word or noun is
   Korean (몇 over `auc`, 어디야 over `jeli`), unless the Shift rule settled it.
3. **Alternatives** rank one step below the reading they vary, unless they're a known word (so ㅠㅠ
   beats 유유). When both are words, their counts decide (§10.9): an alternative needs to be more
   than twice as common to pass the reading the keys spell (저녁 stays 저녁, not 전역).
4. **Implied ㅇ:** each implied ㅇ is weighed against its bare vowel. It's likely in real words
   (아이, 이거) and unlikely in runs of one vowel that chat doubles as jamo (유유 → ㅠㅠ: the
   lexicon counts ㅠㅠ above 유유; 워워 and 오오 stay syllables) or in known chat words, which the
   bare reading then outranks. **A final kept before a vowel** (한글 맞춤법 제14·15·19항: a final is spelled where it is
   when a particle or ending follows the word it ends, and moves within one morpheme): the engine
   offers one variant per place the keys moved a final, with that one kept and the vowel on ㅇ
   (기준입니다 for 기주닙니다 typed `rlwnslqslek`, 당신이, 어린이). Each kept final scores:
   - at least +1.5 bits when the reading up to it is a known word of two syllables or more (not a
     하다/되다 modifier such as 반영한) and the rest is a particle or ending on ㅇ (이 은 을
     의 에 에서 으로 입니다 이에요 아 어 았 었 으면 음 …), maybe followed by a short ending (다 요
     니다 습니다 서 는 도 고 며 면 의 에 를 …). A rest that only starts like one doesn't count:
     엔지니어링, not 엔진 + 이어링; 트레이딩, not 틀 + 에이딩;
   - else the corpus odds learned for ㄴ and ㄹ (`#!implied-after` lines in common-ko.txt, by the
     next syllable's vowel: 입 counts as 이; 의 strongly kept, 아 and 와 mostly moved), or −2 bits
     for other finals;
   - a double final (값이, 많이) +2 bits when its stem is a known word, −2 otherwise (지난주 typed
     `wlskswn` isn't 지낝우);
   - a ㅇ final followed by a particle that only follows consonants, as the rest of the word (은 을
     과 으로 이랑 이나; not 이, which ends many words: 아이), +2 bits: a ㅇ final that moves leaves
     the next syllable as it is, and that particle can't follow a vowel (소비량은 typed
     `thqlfidms`, not 소비랴은).
   Only what a variant keeps beyond the typed reading counts, and word counts decide between two
   known words (아니 over 안이). A test types 31 edge cases from the 국립국어원 rules the lazy way
   (드리겠습니다, 지난주, 사람 move; 기준입니다, 웃음, 많이, 소비량은 keep; 산이/사니, 너머/넘어
   both offered), and 6 more for the ㅇ final: 35 of 37 pass, the misses being stems the corpora
   don't have (값, 바둑). When the bare reading is also the plain 2-set one (ㅏㅣ for `kl`), it's
   one row weighed by both. A single vowel key is its syllable, never a letter: ㅗ alone is 오, not
   `h`, since a lone lowercase letter is hardly ever an English word.
5. **English + particle:** the keys split into an English start and a Korean ending that is a known
   particle (에, 에서, 를, 이라는, 부터는, and business text's 로부터, 로서, 와의, 이자, 이란, 등 …:
   FDA로부터, Licensee로서, Dynamics와의) or a known word made of 하다 / 되다 forms (push했어). The split counts when the start reads clearly more English than
   the whole keys (0.5 bits per key; a capitalized start such as Anthropic or RLHF always
   qualifies) and the whole isn't a known English word or noun, so `small` never becomes smal이 and
   Goodwill never Goodwil이. A start is letters, or letters and digits typed as digits with a
   capital in them (APR1400은); never a number-row jamo key (Dynamics와의, not Dynamics0의 with ㅘ
   as `0`), and not capitals that only go on with more capitals (CCL, not CC이). Among capitalized starts a known English word wins (Frontier를, not Frontierf을), else the longer start (Android에, not Androi에 with `d p` read as 에; SSD에, Docker와), except an acronym trailing off in lowercase keys, which go to the ending (API를, not APIf을), and among such starts the shorter wins (GWh를, not GWhf을).
   - **An acronym, then a Korean word:** after a start of two capitals or more (digits allowed),
     the split is also made where the case changes, however long the rest, and the rest counts as
     the ending when it reads as a Korean word: two lowercase keys or more (then capitals Korean
     typing makes too: ㅖ as `⇧P`), no bare jamo, two syllables or a particle, reading as Korean
     on the key model or a known word: LG에너지솔루션은, KB국민카드의, IT서비스 (not 얐서비스),
     KB증권은, ESG경영, AI플랫폼. Not the lowercase end of a unit or name, alone or before a
     particle (MMBtu로, not MMB셔로).
   - **A Korean word, then an acronym:** where lowercase keys first meet two capitals, the keys
     before them read as Korean, the capitals as typed, and any keys after them as Korean
     (한전KPS, 에너지ICT, 삼성SDI의), one step above a split of the same keys, when the start reads
     as Korean (or is a known word) and the rest is empty, a particle or a Korean word. Not when
     every capital is one Korean typing makes too (`QWERTOP`, or a Shift+vowel) and the Korean
     reading has no bare jamo: 등록되어있으며 typed with 있 as `⇧L ⇧T` stays Korean.
   - **A known Korean start holds a split back:** a split with a lowercase start isn't offered when
     the Korean reading is a known word and the split's ending (소요되는, not thy되는).
   The start must also read as English on its own (1 bit per key), and when the whole keys make a
   Korean word (in the lexicon: `kanxms` is 아무튼, not kanx은) the split is
   only an alternative below it; junk Korean (slack에 as 님차에) doesn't hold it back. Otherwise it
   ranks above the whole-word English reading. A regression test types 8,700+ real Korean words the
   JIS Korean way, with the hanja dictionary: 0.07% rank as mixed (fragments such as 노드는; 0.13%
   before the business-text rules, with 년까지).
6. **Hanja** were lean in JIS Korean (one dictionary candidate, kept last). JIS Moonshot has none
   (§7).

### 10.4 Everyday hanja (not penalized)

*Removed in JIS Moonshot with the other hanja readings (moonshot.md §5.1); kept here as JIS Korean
defined them.* These tie with the Korean reading and follow it right after:

- **Country names in the news, for OECD members** (38 as of 2026-10): 美 미, 日 일, 韓 한, 英 영,
  佛 불, 獨 독, 伊 이 (Italy), 以 이 (Israel), 加 가, 濠 호, 土 토. Alone or in runs (韓美). The
  other members' traditional forms (丁 諾 芬 匈 波 白 墺 葡 希 墨 西) aren't in everyday news use and
  are left out; 中 is everyday but China isn't an OECD member.
- **Dates:** 年 (년, 연), 月, 日, and digits before them (2026年, 10月, 19日), also with a particle
  after them (1日까지, 3日부터); 曰 after a dictionary
  stem (孔子曰).
- **Weekdays:** 月火水木金土日, alone or in runs.

### 10.5 User tuning

Each scenario has a weight, a multiplier on its learned probability: 1.0 is as learned, English
0.8 makes English readings 0.8 times as likely, and Korean 1.2 makes Korean ones 1.2 times as
likely. A weight adds log2(weight) bits to that reading's score (per key for Korean and English,
per implied ㅇ for the implied ㅇ). Scenarios: Korean, English, implied ㅇ, English + particle,
plain 2-set, finals kept, hanja, implied ㄹ after ㄴ, and **kana** (the kana reading, moonshot.md §5:
`Weight.Kana`, id 8; 0 turns it off; learned like the others when learning is on). Weights are clamped to 0.1–10; 0 turns an
alternative scenario off entirely (its readings aren't offered), but not Korean or English, which
always show. They never override the hard rules (a capital at the start of a word, command names).

**Implied ㄹ after ㄴ:** ㄴ then ㄹ sounds ㄹㄹ (전력 [절력]), so a typist going by sound may leave the
ㄹ out: the keys of 저녁 (`wjsur`). After a ㄴ final, each vowel the composition put a moved ㄴ on
also gets a reading with an ㄹ before it (전력). It's an alternative: one step below the typed
reading unless it's a known word, with word counts deciding between two known words. Both are
often valid (저녁, 전력), so it's a slider (default 1.0; 0.00 turns it off; macOS
`Weight.ImpliedL`).

**ㄶ by sound (retired 2026-11-03):** a typist going by sound types 않고 as 안코 (`dkszh`), and
the reading used to offer ㄶ where a ㄴ final met ㅋ, ㅌ or ㅊ (안코 → 않고, kind `Aspirated`, id 13).
ㄶ now has its own key (Shift+`@`, moonshot.md §9.2), so the rule and the kind are gone; id 13
isn't reused.

Hosts store them per user: the web playground has sliders under "Tune predictions" (remembered
in the browser); macOS reads `defaults write me.sjun.inputmethod.JISMoonshot Weight.<Scenario>`;
Windows reads `HKCU\Software\JIS Moonshot`, value `Weight.<Scenario>` (§8.4).

### 10.6 Open questions

- Should the hosts' Space commit the top prediction (as the Minecraft mod does) or keep committing
  the JIS Korean reading, with predictions as a picker only?
- The Minecraft mod still has its own Java copy of the ranker; it should move to this layer
  (through the wasm engine) so there's one implementation.
- Hosts so far: the web playground (predictions on by default, picker under the caret, Space
  commits the top reading) and macOS (marked text shows the top reading, committing keys take it
  and then act on it as on the Korean reading (`Engine::key_after_prediction`): Space adds a space,
  Shift+Enter and the "Enter commits" setting (§5.1) only commit, plain Enter goes on to the app;
  "Predictions" menu item; `make PREDICT=0` strips the layer). macOS has no picker yet: the other
  readings need an IMK candidate window.
- Learning from the user's picks (session or persistent) isn't in the draft.

### 10.7 Travel ranker (experimental, side by side)

A second way to rank the implied-ㅇ alternatives (bare vowels: ㅠㅠ against 유유; a kept final: 말이
against 마리), chosen per engine (`jk_set_ranker`: `JK_RANKER_RULES` by default, `JK_RANKER_TRAVEL`,
`JK_RANKER_ROUTE` for §10.8). Everything else ranks the same.

- **Finger travel** (`engine/src/travel.rs`): key centers on a JIS 109 board in key units, each key
  pressed by its touch-typing finger (F6–F10 by the finger under them). A press moves its finger to
  the key and every other finger back home. Shift and other modifiers move nothing; thumbs rest on
  Space. The planner (`plan.rs`) can minimize travel instead of presses (`Metric::Travel`) and
  reports it either way. Consequence: ㅇ (`d`) is a home key, so the implied ㅇ saves a press but no
  travel; a dedicated number-row key saves Shift but costs travel.
- **Ranking:** an alternative scores the Korean reading's score plus the lexicon's evidence
  (§10.9), less the effort a typist who meant it skipped: the presses and travel of the cheapest way
  to type it (planned for least travel) beyond those of the staged keys, at `press` bits per press
  and `travel` bits per key unit. No one-step-below penalty, no run-of-one-vowel rule. Retrained
  (2026-10-19), both costs are best at 0: the effort term adds nothing, and only the lexicon
  decides.

### 10.8 Route-preference ranker (experimental, side by side)

The question: given typing each jamo against JIS Korean's optimized route, how strongly would a
typist prefer the optimized one? Each reading R has an **each-jamo route** (plain 2-set, one jamo at
a time: ㅇ as `d`, Shift for doubles, two keys for compound vowels and double finals) and an
**optimized route** (the cheapest JIS Korean route: implied ㅇ, dedicated keys, Shift+vowel, double
press). A typist takes the optimized one with odds p = σ(press × presses saved + travel × travel
saved − bias), clamped to 2–98%.

- **The Korean reading:** if the keys are its each-jamo route, they were taken with 1 − p (or for
  sure when no cheaper route exists, as for 마리); otherwise they're a route with shortcuts, taken
  with p for what they save over each jamo.
- **An alternative:** if the keys are its each-jamo route (ㅠㅠ as `b b`), likewise; otherwise the
  keys leave out (or add) presses it needs, at `omission` bits per edit to the nearer of its two
  routes (말이 typed `akfl` is one press short).
- **Score:** Korean score + lexicon evidence (§10.9) + a prior against bare jamo (one step per
  jamo inside a word, ㅗㅗ백년; one step for a word of mixed jamo, ㅓㅏㅂ; none for a run of one
  jamo, the chat genre ㅠㅠㅠ) + log2 P(keys | alternative) − log2 P(keys | Korean).
- **Parameters** (near the best of the grid): press 0, travel −0.5, bias −1, omission 3. Presses
  saved don't move the preference, travel saved lowers it slightly, and a typist takes JIS Korean's
  route about 3 times in 4.

### 10.9 Lexicon counts and the side-by-side evaluation

- **Counts:** `common-*.txt` hold a hand-written list, then (below a marker) words generated from
  colloquial corpora with their counts (`word<TAB>count`), and business terms, each counted the same
  (20, or more if chat uses them more) whatever its frequency. Generated Korean words need 8 uses
  (one-syllable words 20, and a lone jamo never counts); English ones 30 and 3+ letters. Sources and notices:
  `engine/predict/LICENSE-corpora.txt`; fetch with `engine/tools/corpora/fetch_corpora.py`, train with
  `build_ranker.py --corpora corpora`. 1 in 10 sentences is held out. Lexicon evidence for an
  alternative against the Korean reading: +4 bits if only the alternative is a word, −2 if only the
  Korean one is, and when both are, 2 bits plus log2 of their count ratio, within ±4. All three
  rankers share it (the rules ranker otherwise unchanged).
- **Evaluation:** `cargo run --release --example sxs` scores the three on words typed properly with
  an implied ㅇ (A), ㅇ after a final typed lazily (B), held-out bare-vowel chat words (C), held-out
  chat eojeols (D), held-out English words (E) and Korean business terms (F), with grids over the
  travel and route parameters. 2026-10-19, errors on A + C + D′ (implied ㅇ in chat): rules 82,
  travel 86, route 77 (75 at omission 5, at the cost of B). E and F are the same for all. Per
  prediction (native, release): rules ≈5 µs, travel ≈14 µs, route ≈17 µs.
- `examples/abc.rs` types texts into all three behind blind labels for rater evaluations.
- The web playground can show all three lists (Tune predictions → Implied ㅇ ranking → Side by
  side).

### 10.10 Punctuation inside a word, and the trusting typist

- **Typed punctuation stays as typed** in the English reading (a `;` typed is `;`). F9/F10 convert
  it as typed too (§5.3): number-row jamo keys convert to their digits, typed symbols to their
  other width where they have one.
- **Pieces:** punctuation typed inside a staged word (anything that isn't a letter or digit) splits
  it into pieces, each ranked on its own. The pieces' top readings, joined with their punctuation,
  form one more reading (Research(이하, AI)를, 기여(contribute)하는, SaaS(Software-as-a-Service)에),
  ranked above the whole-word readings. A piece that is one capital, maybe with digits, next to a
  piece whose top reading starts with a Latin letter is that letter as typed, not the jamo its key
  types: K-IFRS (not 아-IFRS), CT-P13 (not CT-예13). A lowercase one stays Korean: (USD)에.
- **The trusting typist** (`engine/src/trust.rs`, `tests/corpora.rs`) types a text with the fewest
  presses and lets the predictions do the rest: Hangul by JIS Korean's optimized route, Latin
  letter by letter (a capital Z as z, since Shift+Z is F10), hanja by their reading, symbols by their
  key or pasted (a flat 3 presses, as in the planner); Space or Enter commits the top
  prediction. Hanja are the exception: never the top prediction, they're picked with ↓ (each a
  press) and fail only when not listed. A test asserts no hanja is ever on top. Measures: failed
  words, characters wrong, and **effort**: presses plus the cost of fixing failures (↓ to the word
  when listed, else Backspace and the word again without predictions), against the planner's
  presses without predictions.
- **Typing styles:** real typists don't always take the cheapest way. The planner (`plan::Style`)
  can type each syllable the cheapest way, every jamo (ㅇ typed, Shift for doubles, two keys for
  compound vowels and double finals), or a deterministic mix of all the valid ways (의 as `9`, `d9`
  or `dml`; 당신이 as `ekdtlsL`, `ekdtlsll` or `ekdtlsdl`). The tests type the owner's texts in all
  of them (0 failures each) and the AI-extended ones with per-style ratchets. Shift+vowel is a
  deliberate signal that the ㅇ was wanted, so with one in the keys the plain 2-set reading gets no
  lexicon advantage (열어 typed `d u f ⇧J` isn't 여러), and a word started with one holds back an
  English + particle split like a known Korean word does (오픈 typed `⇧H v m s` isn't Hv은).
- **Training on mixed routes:** the key model learns each Korean word typed every jamo (25%), the
  cheapest way (35%) and two random mixes of the valid ways (20% each). Number-row keys are read as
  their 2-set keys, as the predictor maps them. Held-out accuracy on mixed routes: 99.0%.
- **Texts:** `engine/tests/corpora-en-ko/` holds the owner's texts verbatim (ratchet: 0 failures)
  and `extended/` their continuations by AI agents, marked "human corpus extended by AI" on the
  first line (not typed). They weigh ¼ in the weighted failure rate and have their own ratchet.
  2026-10-19: owner's texts 0/115 failed (effort 2.7% below no predictions on the mixed one),
  AI-extended 3/2008, weighted 0.12%; 2026-10-27, with the business-text rules (§10.3): AI-extended
  2/2008, weighted 0.08%. `examples/trust.rs` runs any texts with all three rankers.

### 10.11 Learning from the user's choices (opt-in, on the device)

Off by default. When on, gentle multipliers on the §10.5 scenario weights follow what the user
picks. All logic is in the engine (`engine/src/learn.rs`, pure and deterministic); hosts only store
one line of counts and show the setting.

- **Signals:** the engine records them itself when a key commits (a prediction, or the Korean
  reading). A commit the host forces (focus loss, a click elsewhere: `flush`,
  `flush_prediction`) and F6 (Korean, forced) are no choice and record nothing.
  - **Override:** a lower prediction chosen (↓ or a click) over the top one.
  - **Accept:** the top one committed while other kinds were shown, also the Korean reading on top
    committed by a plain commit key (Space, Enter).
  - **Regret:** Backspace right after an accept, counted as half a demotion.
  - Only kinds that were shown count, so rarely shown kinds don't look unpopular. A choice between
    two readings of the same kind says nothing.
- **Attribution:** a choice between the Korean reading and one of its alternatives (bare vowels,
  kept finals, 2-set, English + particle, hanja) moves only that alternative's scenario. Only
  Korean against English moves those two. The implied-ㅇ scenario's counts are about the
  bare-vowel reading, so its learned weight is the inverse (picking ㅠㅠ makes the implied ㅇ less
  likely).
- **Update:** per scenario, the rate at which it's chosen when shown below the top, and the rate at
  which it's passed over when on top, each with 100 pseudo-counts at a 5% prior. The weight is
  exp(0.35 × (log-odds up − log-odds down)), clamped to 0.5–2× (a user's own weights may go
  0.1–10). One choice moves a weight about 7%; ten about 45%; around thirty reach the clamp. Counts
  halve every 400 events.
- **Precedence:** a weight the user set by hand wins over the learned one until cleared
  (`jk_clear_prediction_weight`).
- **Hard guards that learning can't cross:**
  - A hanja reading is never the top prediction, whatever any weight says; the first other reading
    moves up. (JIS Moonshot predicts no hanja at all, moonshot.md §5.1.)
  - The Shift rule and the chat-run rule are not weights and don't learn.
- **Stored:** no text, no keys, no times, no app names, only the counts (`jk_learner_export`, one
  line). Nothing leaves the device.
  - **macOS:** the app's preferences, keys `LearnFromChoices` and `Learned`. The input menu has
    "Learn from My Choices (Stays on This Mac)" and "Learned Weights" (read-only, with "Reset What
    It Learned").
  - **Web:** the browser's local storage only, forgotten 24 hours after learning started, whether or
    not it's the same session. The age is checked when the page loads, on every save, and once a
    minute.
  - **Windows:** `HKCU\Software\JIS Moonshot`, values `LearnFromChoices` and `Learned`. The
    indicator's menu has "Learn from My Choices (Stays on This PC)" and "Learned Weights"
    (read-only, with "Reset What It Learned").
  - **Minecraft:** not yet; it would need the mod's config.
- **Validation** (`tests/learn.rs`): simulated users with a habit the defaults don't match.
  - A bare-vowel chatter: half the habit is on top within three rounds. ㅗㅗ and ㅏㅏ stay a ↓ away,
    because the lexicon knows 오오 and 아아 and the clamp holds.
  - A Korean-first typist: all on top by round two.
  - A hanja lover: the hanja weight reaches the 2× clamp, and hanja still never come first.
  - Learning off learns nothing, and a trained profile leaves ordinary words alone.
  - The owner's texts, typed attentively with learning on five times over, stay at 0 failures with
    no extra ↓.

### 10.11.1 Per app (macOS)

- **Disable / Resume Learning from <app>** (input menu): choices made in that app aren't recorded
  (`jk_set_recording`); what was learned still applies there.
- **Disable / Resume Predicting in <app>**: no predictions there (the deterministic engine).
- **Off by default:** predictions start off, until the user turns them on, in apps where typing is
  commands, paths, passwords or a game's chat:
  - terminals;
  - file transfer and SSH (Transmit, Cyberduck, FileZilla, ForkLift, Termius…);
  - containers and VMs (Docker, OrbStack, Parallels, UTM…);
  - remote desktop;
  - password managers;
  - Finder and launchers;
  - Minecraft Java Edition, where the JIS Korean mod is the dedicated integration;
  - any bundle id with "terminal", "ssh", "sftp" or ".ftp".
  It's detected when JIS Korean is first used there: nothing is added to the settings until the
  user changes it. The first time, a small notice under the input menu's icon gives the reason
  (sensitive app, or dedicated mod recommended).
- **Manage Disabled Apps…**: a list of the apps with a setting, each with its canonical full path,
  fetched when the list opens. An uninstalled app stays listed, marked with an ⓘ saying it was
  deleted, until the user removes it (with a confirmation).

### 8.1 Keyboard Viewer (macOS)

While JIS Moonshot is active, macOS uses a private layout bundled with the input method, one per
mode: `JISMoonshot.keylayout` for Mode A (JIS Kana uses it too: it consumes every key it maps), and
`JISMoonshotEnglish`, `JISMoonshotDvorak` and `JISMoonshotColemak` for the English modes. The input method
names the mode's layout to the client on each activation (`overrideKeyboard(withKeyboardNamed:)`).
- **Generated at build time** by `mac/tools/make-keylayout` from the engine's own legends
  (`cargo run --example legends <mode id>`), so it can't drift from §3.
- **Its plain and Shift layers** carry the mode's legends.
- **Its Cmd, Ctrl and Option layers, and unlabelled keys,** are ABC's (JIS), so shortcuts and
  Option characters are unchanged.
- **The engine reads key codes,** not characters, so the layout never changes what a key JIS Korean
  consumes types. A key it passes on (letters in an English mode, shortcuts) is typed by this layout,
  and so is every key an app types without asking the input method: password fields, where macOS
  bypasses input methods, and in Chrome 152–156 any field that once looked like a password
  (Chromium issue 567321058, fixed in 157). In a Korean mode those come out as bare jamo, as with
  Apple's Korean input method and its 2SetHangul layout.
- **The layout named is the app's, not the field's:** every field of an app (Chrome's omnibox and
  its pages) is typed through the layout named last, and macOS drops it when the input method is
  deactivated there. So the input method keeps the name last given to each app and, before a key,
  names the field's mode's layout again when it differs, as it does the engine's mode (a switch can
  reach another field than the one typing, §8.2.1). Otherwise an English engine's letters, passed on
  to the app, would be typed by a Korean layout as bare jamo.

### 8.2 Modes as input sources, and the public API (macOS)

Each mode (§3.6) is its own **input mode** of the one input method, the way Japanese has Kana and
Romaji:

| Mode | Input source ID | Shown as | Script |
|---|---|---|---|
| A | `me.sjun.inputmethod.JISMoonshot.2Set` | JIS Moonshot | Korean |
| `A_KANA` | `me.sjun.inputmethod.JISMoonshot.A_KANA` | JIS Kana | Japanese |
| `A_QWERTY` | `me.sjun.inputmethod.JISMoonshot.A_QWERTY` | JIS English | Roman |
| `A_DVORAK` | `me.sjun.inputmethod.JISMoonshot.A_DVORAK` | JIS Dvorak | Roman |
| `A_COLEMAK` | `me.sjun.inputmethod.JISMoonshot.A_COLEMAK` | JIS Colemak | Roman |

Modes after A take their mode name as the last part of the input source ID (§3.6.1). The bundle id
is `me.sjun.inputmethod.JISMoonshot` (`JISMoonshot.app`), so JIS Moonshot installs beside JIS Korean
(`me.sjun.inputmethod.JISKorean`) without colliding.

The user adds the ones they want in System Settings → Keyboard → Input Sources. A first
`make install` enables JIS Moonshot, JIS Kana and JIS English.

**Why input sources, and not a setting.** The mode follows the keyboard, and switching apps
doesn't change which keyboard the user is typing on, so a per-app mode would mean nothing. As input
sources, the modes are visible to everything that already understands input sources: the input
menu, System Settings, and tools JIS Moonshot has no integration with, such as Hammerspoon and
Karabiner, which can read and select them by ID. Never rename an ID: those tools match on it.

**Public API.** The JIS Moonshot program also answers any other program, without an input-source
API:

| | How | Allowed |
|---|---|---|
| Read the mode | `defaults read me.sjun.inputmethod.JISMoonshot CurrentMode` → the mode name, `A`, `A_KANA` or an English mode (`A_QWERTY`…) (also `CurrentModeID`, `CurrentModeInputSourceID`, `CurrentModeName`), or `JISMoonshot.app/Contents/MacOS/JISMoonshot mode` | always |
| Hear changes | distributed notification `me.sjun.inputmethod.JISMoonshot.ModeDidChange`, user info `mode`, `id`, `inputSourceID`, `name` | always |
| Change the mode | distributed notification `me.sjun.inputmethod.JISMoonshot.SetMode`, user info `mode` (a mode name, an engine id or an input source ID), or `JISMoonshot mode A_KANA` | only with **Let Other Apps Change the Mode** on, in the input menu with JIS Moonshot's other settings (off by default) |

A change request is answered with `ModeDidChange`, or with
`me.sjun.inputmethod.JISMoonshot.ModeChangeRefused` and a `reason`: the setting is off, the mode is
unknown, or the mode isn't added in System Settings (the API never adds one). `CurrentMode` is the
mode last used, whether or not JIS Moonshot is the selected input source now; the selected source
says that. Selecting the input source directly (the input menu, `TISSelectInputSource`,
Hammerspoon's `hs.keycodes.currentSourceID`) is macOS's own switching and isn't gated by the
setting.

### 8.2.1 Switching between Korean and English (macOS)

The language keys (§8, moonshot.md §4) on macOS:

| Key | macOS keys |
|---|---|
| `LANG_KO` | かな; also カタカナ (which macOS turns into かな) and the Korean Key (F18 by default), which act as かな: Mode A if it's added; in Mode A, force Korean; from JIS Kana, Mode A with the staged keys forced Korean |
| `LANG_EN` | 英数: a JIS English mode (`A_`: the one used last if it was one, else `A_QWERTY`, else another), committing what shows |

On Windows the same keys are 無変換 and 変換 (§8.4). On macOS English is an input mode, JIS English
(`A_QWERTY`, §3.6.2), instead of a hidden on/off flag. Where this section says JIS English, it means
**the English mode used last** (or, if that one isn't added in System Settings, the first English
mode that is).

- **Why a mode, not a flag.** A flag inside the controller would switch fastest, but macOS wouldn't
  know about it: the menu bar would keep the Korean icon, 英数 and Ctrl+Space would go to Apple's
  ABC, and the state would be lost whenever macOS reselects an input source.
- **Why a mode, not a separate input method.** Each input method is its own process. Switching
  between two (JIS Moonshot and ABC) has macOS deselect one, select the other and reconnect the app,
  asynchronously, so a key pressed right after can land in the wrong one. Switching between modes of
  one input method stays in the same process and the same session with the app, which is how
  Japanese switches between Kana and Romaji.
- **Selecting is asynchronous.** The field switches in place first (its engine and its app's
  layout), then the mode is selected: through the field's own session (`selectInputMode:`, IMK's
  way for an input method to change its mode) when one of JIS Moonshot's modes is selected, and with
  `TISSelectInputSource` only when coming from another input source. Selected through TIS from the
  input method's own process, Chrome's session was cut off and Chrome typed every key through the
  layout itself, as bare jamo. The selection counts as made from the moment of the switch: until
  macOS reports it, it still names the old source, so a field activated meanwhile (Chrome activates
  its fields out of order) counts the selection as made instead of reverting to the old mode, which
  every other field would then follow. macOS's notice of a selection is taken even while the input
  method's notifications are suspended (AppKit suspends them whenever the app isn't active, and an
  input method never is).
- **The engine changes its own mode.** F6 and F7 switch to JIS Kana, and かな from JIS Kana back to
  Mode A, inside the engine: the shell reads `jk_mode` after each key and selects the matching input
  source.
- **It's macOS's English input source.** It's declared with the Roman script (JIS English as the
  script's primary mode, JIS Dvorak and JIS Colemak beside it), so 英数 selects it (as with Japanese
  input methods' own Romaji modes), and the menu bar shows its badge
  (`docs/logo/mode-a-qwerty.svg`). JIS Kana is declared with the Japanese script. Unlike Windows'
  ABC the mode is **remembered**, like any input source.
- **かな and カタカナ are one key.** macOS turns カタカナ (the カタカナ/ひらがな key of Windows JIS
  keyboards, right of the space bar) into かな (key code 104). JIS Korean read the keyboard at the
  hardware level to tell them apart; JIS Moonshot doesn't need to, since both go to Mode A. There is
  no hardware listener and no Input Monitoring permission. かな is no longer left to macOS: with the
  key tap running and Mode A added, it never selects Apple's Japanese input method.
- **Korean Key:** F18 by default, so a key remapped to F18 (Karabiner, or a keyboard's own firmware)
  acts as かな without further setup. The input menu offers None and F13 to F19 (Korean Key (Same as
  かな)); any other key code is set with `defaults write me.sjun.inputmethod.JISMoonshot KoreanKey
  -int <code>` (-1 for none). It never reaches the app.
- **Caught system-wide.** かな (with カタカナ and the Korean Key) and 英数, without ⌘, Ctrl or ⌥, are
  caught by a system-wide key tap before any app sees them, so they work in apps that keep keys to
  themselves (iTerm keeps F1–F19) and from other input sources, Japanese included. The tap needs
  **Accessibility** only. It runs when one of JIS Moonshot's modes is added and the permission is
  already granted: JIS Moonshot never asks at launch, only from the input menu's "Allow 英数 and かな
  Everywhere…". It checks again when input sources change and, at most every 30 seconds, when a
  field activates. A key whose target mode isn't added passes through (かな with Mode A not added, 英数
  with no English mode). Without the permission, the keys work only in apps that pass them on to
  the input method. Caps Lock was tried and dropped: macOS toggles its light before an input method
  hears of it.
- **Starting at login.** The tap lives in the input method's process, which macOS otherwise starts
  only once JIS Moonshot is selected somewhere, so `make install` adds a login agent
  (`~/Library/LaunchAgents/me.sjun.inputmethod.JISMoonshot.login.plist`) that starts it in the
  background.
- Switching commits what's unconfirmed first (§5.1), except かな in Mode A and JIS Kana, which
  re-read the staged keys (moonshot.md §4).

A Mac JIS keyboard (a MacBook's built-in one) has 英数 and かな, which is all JIS Moonshot needs.

**ABC stays.** macOS always keeps at least one plain keyboard layout enabled (ABC, or a country
layout), and an input method's modes don't count as one, so Apple's ABC can't be removed from the
input sources. macOS needs it for secure input (password fields), where it bypasses input methods
and types through a plain layout. JIS English sits beside it.

Seen on macOS 27.0.1 with JIS Korean: with Apple's ABC enabled, 英数 selected ABC, not JIS English,
although the tap saw the key. The tap now swallows 英数 and selects JIS English itself, which is
meant to make the swap unnecessary (not yet confirmed on that setup). With ABC – Extended as the
plain layout instead of ABC, 英数 goes to JIS English; that swap stays the documented workaround
([#2](https://github.com/team-mirae/jis-korean/issues/2),
[troubleshooting](troubleshooting/macos-katakana-and-eisu.md)).

### 8.3 Resolving keys and text

Two functions answer "what does this key type" and "which keys type this", for programs that drive
or check JIS Korean from outside (`engine/src/resolve.rs`, the C ABI `jk_resolve_*`, and on macOS
the public API of §8.2). They run on a copy of a live engine, so they match it exactly: its mode,
and with predictions on, its predictor, weights and what it learned. On macOS that is the engine of
the app JIS Korean was last active in, and every answer names the app, the mode and whether
predictions were on.

A key press is a `KeyboardEvent.code` with `S-` for Shift held (`KeyG`, `S-Digit2`); the macOS API
also takes a virtual key code (`5`, `S-5`). One press is one key, with or without Shift.

- **resolveKeyCodeInput(keyCode)** → the character the key types on its own in the mode (`KeyG` →
  ㅎ, `S-BracketLeft` → " in Mode A). None for keys that type no character: F-keys, Space.
- **resolveKeyCodeInput(keyCode, context)** → what the field shows after `context` (unconfirmed
  text typed so far) and then the key: the context with the key's effect, including anything it
  commits (`하` + `KeyS` → 한; `한` + `Space` → `한 `; with predictions on, `hello` + `Space` →
  `hello `). The context is staged by keys whose own Korean reading is the context, as a person types it
  (국의 with its ㅇ); only when there are none, by keys that show it through the top prediction.
  None when no keys type the context.
- **resolveKeyCodeFromOutput(text, minKeypressCount = 1, maxKeypressCount = 1)** → every
  combination of key presses that types `text` from an empty field, with that many presses: `ㅃ`
  → `[Digit1]`, `[S-KeyQ]`. Fewest presses first, then fewest Shifts; at most 1000. **Never an
  error:** a minimum below 1, a maximum below the minimum or an empty text give an empty array, and
  a maximum above 16 is cut to 16.

Keys apply as the shell applies them (§10): with predictions on, the marked text shows the top
prediction unless that is the Korean reading, and Space and Enter commit what is shown. The search
tries the keys whose characters are part of the text (in the layout), F6,
and F7, F9, F10, Space and Enter only where the text needs them; it skips presses that change
nothing, and follows only key routes whose text heads toward the target (its jamo, its letters
before F9/F10, a hanja's reading before F7). Every combination returned is checked by typing it.

### 8.4 Windows: modes, 無変換 / 変換, 한/영 and settings

Windows comes after macOS in JIS Moonshot (moonshot.md §7); this is the design the shell follows.

**One input method, the mode as a setting.** Windows lists one keyboard, **JIS Moonshot**, with new
CLSID and profile GUIDs, so it installs beside JIS Korean. The mode (§3.6) is chosen in its menu
(right-click the taskbar indicator): **Mode A** or **JIS Kana**, one setting for every app, Mode A by
default. This differs from macOS (§8.2) on purpose: on Windows each input method is one more entry
in the Win+Space list and in the language settings, and none of the tools that read input sources
there select them by ID the way Hammerspoon and Karabiner do on macOS, so a mode per input method
would cost more than it gives. The engine can change the mode itself (F6, F7, 変換 from JIS Kana);
the shell reads `jk_mode` after each key and follows it.

**The language keys** (§8):

| Key | Does |
|---|---|
| 無変換 (`VK_NONCONVERT`) | English (**ABC**, typed with the English layout below), committing what shows |
| 変換 (`VK_CONVERT`) | Mode A; in Mode A with something staged, force Korean (`jk_kana_key`) |
| 한/영 (`VK_HANGUL`) | Toggles English ↔ Mode A, as the Microsoft Korean IME does, committing first (§5.1, IME switch) |

The English state is one for the whole session, shared by every app, and never stored: it starts in
Mode A after a restart or a crash. The taskbar shows the state where the Microsoft IME shows 가 / A,
and a click there switches too. JIS keyboards have 無変換 and 変換 next to the space bar; a 한/영
key can still be mapped with the keyboard's own software or a remapper such as PowerToys.

**ABC's layout** is an English mode (§3.6.2, §3.6.4), chosen in the menu's **English Layout**:

| Setting | ABC types |
|---|---|
| Default (unset) | JIS English (`A_QWERTY`) |
| Any English mode | that mode (JIS English, JIS Dvorak, JIS Colemak) |
| Windows' keyboard layout | every key goes to the app untouched, typed by Windows' own layout |

Windows doesn't switch between three input sources, so the English modes are only this: the layout
ABC types, not input methods of their own. As on macOS the engine consumes no key in an English
mode; the shell types each key's character from the engine's table (`jk_legend_in`), as macOS's
keyboard layout is generated from it, so the symbols can't drift. Caps Lock types capitals.
Shortcuts (Ctrl, Alt, Win), Space, Enter, Tab, the arrows and the F-keys go to the app. **Letter
variants (§3.6.3) are deliberately left out on Windows**: Alt + letter is Windows' menu and shortcut
key, used far too much to take.

**Predictive typing must be accepted** (moonshot.md §6). The installer shows the notice as a page
that must be accepted, and the TIP checks `HKCU\Software\JIS Moonshot\PredictiveAccepted`: until
it's set, every key passes through, and the indicator menu has "Accept Predictive Typing…". There is
no Predictions switch and no lite build (`build.ps1` has no `-Lite`).

**Private fields.** The list (§7, §10) shows readings of what's typed, so it's excluded from
screen capture (recordings, screenshots, screen sharing). In a field marked as a password or PIN
(input scope, disabled keyboard, a password character) every key types ABC's layout, composing
nothing: JIS on any board, not Windows' Korean layout under the TIP (US ANSI symbols, Shift+2 → @).
Windows already keeps TIPs out of most of them, and there its Korean layout types. Password boxes that aren't marked (common in
banking and certificate programs) can't be told apart from text: there the user switches to ABC,
or turns JIS Moonshot off for that program in the menu (`Apps\<program.exe>\Off`), which types
ABC's layout there only.

**Settings** are the macOS ones, stored in `HKCU\Software\JIS Moonshot` (`Mode` by engine id;
`EnglishMode`, ABC's layout: an English mode's id, `0xFFFFFFFF` for Windows' layout, unset for the
default; `PredictiveAccepted`, `EnterCommits`, `LearnFromChoices`, `Learned`, `Weight.<Scenario>`,
per app `Apps\<program.exe>\Predictions`); the default-off apps (§10.11.1) are matched by program
file name. A saved `Mode` naming a retired id (JIS Korean's B or C) falls back to Mode A. The
training file is `%APPDATA%\JIS Moonshot\training.json`.

---

### 10.12 Training files (the training contract)

A training file tunes the predictions to one person from their own data, prepared by any tool or an
AI agent: `docs/training-contract.md` (the contract, with instructions and questions for agents),
`docs/training-contract.schema.json`, `docs/training-example.json`. A JSON object with `format`
`"jis-korean-training"`, `version` 1, and optionally:

- `weights`: the §10.5 sliders, applied as set by hand (0 turns an alternative off);
- `vocabulary`: the user's Korean and English words with counts, added to the word lists
  (`jk_predictor_with_words`: a new predictor, the base unchanged);
- `examples`: words as meant, optionally with the keys typed: each is typed into a scratch engine
  and, where it's listed, learned as a choice (`jk_learn_example`; §10.11's learner, so it counts
  while learning is on, and applying a file with examples turns learning on);
- `avoid`: words the user never means, Korean or English, each soft or `strong`
  (`jk_predictor_with_avoided`: a new predictor, the base unchanged). Wherever a reading is an
  avoided word, alone or followed by a particle (§10.3 rule 5's list: god, god을, god에서, 마리가),
  it drops 4 bits; a strong one is also never on top, like hanja (the next reading moves up). It
  stays listed, so it can be picked on purpose. English matches ignore case, except when the keys
  start with a capital (§10.3 rule 2 settled English: God typed with Shift+G stays). An avoided word
  wins over the same word in `vocabulary`. Learning is unchanged: picking an avoided reading is
  learned like any pick, and the avoid list still applies afterwards. Hanja readings are never
  matched;
- `typos` and `preferences`: kept, not acted on in version 1.

Applied on the device only: the web playground has a drop zone under "Tune predictions" (words kept
in the browser until "Forget training words"; learned examples expire with the 24-hour rule);
macOS reads `~/Library/Application Support/JIS Korean/training.json` on activation when it changed
(input menu → Learned Weights → Open Training Folder), and forgets its words when it's removed;
Windows does the same with `%APPDATA%\JIS Korean\training.json` (§8.4).

### 10.13 Learned ranker

The full build ranks the readings (§10.2, unchanged) with a linear model learned from text typed
through the engine. It corrects the rules' order rather than replacing it: the rules still make
every reading and score it, the model nudges, and the hard guarantees are rules applied after it.

- **Score:** a reading's score is its rules' score (§10.3, computed with neutral weights, within
  ±20 bits) plus the model's correction, `reach · tanh(w·x / reach)`: at most `reach` (2) bits either
  way. Then the user's weights (§10.5) and the learned ones (§10.11) add their bits for the
  reading's scenario, times twice the reach: the model can move two readings apart by twice its
  reach, and a weight at the learner's clamp (2×) can always undo that. Duplicates are one
  reading, with the kind of the copy the weights lift most.
- **Features** (`engine/src/learned.rs`; about 130 per reading, hashed with FNV-1a into 2^20
  weights): the rules' score, rank and gap to the top; where the reading came from (Korean,
  English, case variants, 2-set, kept finals, implied ㄹ, hanja, bare vowels, English + particle,
  Korean + acronym, pieces) and its other copies; the key model's margin; lexicon counts of the
  reading, of the Korean reading and of the keys as English, and WordNet nouns; the keys' capitals
  (initial, acronym, all capitals, capitals Korean shifts too, Shift+vowel inside), digits, digits
  then a counter, punctuation; the previous word's language; lengths; implied ㅇ, kept-final and
  implied-ㄹ counts; bare jamo; for English + particle, the split's shape (ending, its length,
  particle or 하다/되다 ending, acronym tail, the start's lexicon count and margin); for Korean +
  acronym, its parts; the keys' n-grams (1–5) and case shape crossed with the reading's class; the
  reading's character classes (1–4), syllables (1–2) and last syllables. Deterministic: a fixed
  hash, no hash-map order.
- **Still rules, after scoring:** a reading a rule ruled out (rules' score −50 or less: a capital
  that settles English, an acronym in capitals, a command name) stays below the rest; one vowel key
  alone is its syllable, never a letter; the dictionary's hanja come last and an everyday hanja
  (§10.4) right after the Korean reading; hanja and strongly avoided words are never on top; the
  avoid list's 4 bits (§10.12); a scenario weighed 0 is off; at most 4. The experimental travel and
  route rankers (§10.7, §10.8) aren't affected.
- **Model file:** `engine/predict/learned.bin`: `JKLR`, format version, feature-set version, table
  bits, scale and reach, then one 16-bit weight per entry. A host loads it after the predictor with
  `jk_predictor_with_learned` (macOS, Windows and the web do); a model trained on other features,
  or malformed, is refused (the rules rank). The legacy build ignores it.
- **Training** (`python3 engine/tools/train_learned.py`, one command, deterministic; rerun after
  `ranker.bin` or the word lists change): samples about 1.8 million words from `corpora/derived/*/`
  (each source by its `weight`) and `corpora/` chat: Korean and English sentences, word lists,
  business terms and mixed Latin + Hangul tokens. `examples/learn_data.rs` types them through the
  engine the trusting way (§10.10) in the planner's styles (cheapest 40%, every jamo 20%, mixed
  40%), and a third of the words with a final before a syllable on ㅇ also lazily (당신이 typed as
  당시니, rule 4), recording each word's readings, features and which one it is. Words with hanja,
  words with one reading, and **every word of the evaluation texts** are left out. Identical
  records merge (816,000 lists, 4.4 million readings). Listwise logistic loss (softmax over each
  word's readings), weighted by source, L2 300, L-BFGS. 1 word in 20 (by a hash of the word) is
  held out.
- **Evaluation** (never trained on): the owner's texts and the AI-extended ones (§10.10), the
  Samsung robot corpus, and the DART audit corpus (10 KOSPI 200 business reports, 76,600 words,
  never picking from the list). 2026-10-27, words wrong (rules → learned): owner's 0 → 0 of 115 in
  every style; AI-extended 2 → 2 of 2,008 (styles: every jamo 1 → 1, mixed 5/5/4 → 4/4/4);
  Samsung 47 → 41 of 4,713 (every jamo 60 → 55, mixed 63 → 57); DART 584 → 499 (every jamo
  579 → 492, mixed 728 → 594). Held-out words: top-1 95.75% → 95.84% (lazily typed finals included).
- **Legacy:** `cargo build --features legacy-predict` (macOS `make LEGACY_PREDICT=1`, Windows
  `build.ps1 -Legacy`) leaves the model out: the rules rank, as before this section.

