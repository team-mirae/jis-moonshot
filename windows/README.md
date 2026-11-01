# windows

The Windows input method for JIS Moonshot, the fork of JIS Korean that types Korean, English and
Japanese kana from one JIS keyboard (`docs/moonshot.md`, which wins over `docs/spec.md`): a TSF
text input processor (`jismoonshot.dll`) that wraps `/engine`. All behavior is in the engine. This
shell maps scan codes to engine keys (spec §2) and applies the result to the app as a TSF
composition, the prediction list, and the taskbar's 한あ / かな / ABC indicator.

It installs beside JIS Korean without colliding: its own CLSID and profile (`src/guids.h`),
`jismoonshot.dll` in `Program Files\JIS Moonshot`, settings in `HKCU\Software\JIS Moonshot` and
data in `%APPDATA%\JIS Moonshot\`. The engine keeps JIS Korean's C names (`jis_korean.h`,
`jis_korean.lib`).

## Build

Windows 10 or 11, Visual Studio 2022 Build Tools (C++ x64/x86), CMake 3.28+, Ninja, and Rust
(rustup) with the `x86_64-pc-windows-msvc` and `i686-pc-windows-msvc` targets.

```powershell
rustup target add i686-pc-windows-msvc
tools/build.ps1               # x64 + x86 (32-bit apps load the x86 DLL), runs the shell tests
tools/build.ps1 -Legacy       # predictions ranked by the rules alone, no learned ranker (build/<arch>-legacy)
tools/build.ps1 -Arch x64     # one architecture
python -I tools/make-icons.py # only after the logo changes: icons/moonshot.ico from docs/logo
```

There is no lite build: predictions are always on (docs/moonshot.md §6). The DLL embeds the
prediction model, `predict/common-ja.txt` (the kana reading's lexicon) and `predict/kanji.bin`
included, so each process maps one copy from the image. It links the static CRT: it loads into every app
and can't count on a VC++ runtime being installed. The engine is built with its `english` feature,
for ABC's English layouts.

## Installer

```powershell
tools/package.ps1             # builds x64 + x86, then build/dist/JISMoonshot-<version>-setup.exe
tools/package.ps1 -SkipBuild  # package what's in build/ already
```

The installer users download: one wizard (Welcome, Licence, **Predictive Typing**, progress,
Finish) that does what `tools/install.ps1` does, plus the notice. The Predictive Typing page shows
the notice of docs/moonshot.md §6 (the same text as the TIP's and macOS's) and, like a licence,
must be accepted before Next is enabled; accepting stores `PredictiveAccepted` = 1 for the user who
ran Setup. It asks for UAC once, copies both DLLs to `Program Files\JIS Moonshot` (moving a DLL a
running app holds aside, as the script does), registers them, adds JIS Moonshot to the installing
user's Korean keyboards (adding Korean if it isn't there, without removing anything, JIS Korean
included), and registers an uninstaller in Apps & features. Running a newer installer over an
older one, or over a development install, upgrades it in place and keeps the user's settings.
The uninstaller keeps `%APPDATA%\JIS Moonshot` unless its box is ticked, and leaves
`HKCU\Software\JIS Moonshot` (the settings) as the script does. Setup changes the language list
as the user who ran it; the uninstaller can only do it as the account it was elevated as, so a
user who uninstalls with another administrator's password keeps a dead entry in their list.
Silent installs: `JISMoonshot-<version>-setup.exe /VERYSILENT /ACCEPTPREDICTIVE=yes`. Without
`/ACCEPTPREDICTIVE=yes` a silent install doesn't accept the notice: JIS Moonshot then passes every
key through until it's accepted from the indicator's menu.

It's [Inno Setup 6](https://jrsoftware.org/isinfo.php) (`installer/jismoonshot.iss`): free,
builds from the command line (`winget install JRSoftware.InnoSetup`), and its wizard pages, the
uninstall question and the steps that must run as the user rather than elevated (the language
list, `installer/keyboards.ps1`) are a few lines of script each, where an MSI would need custom
actions for all of them. The version comes from the engine's `Cargo.toml`, as in `CMakeLists.txt`.

**Signing** is off by default. Set `JK_SIGN_CERT` to a `.pfx` file (and `JK_SIGN_PASSWORD`, and
`JK_SIGN_TIMESTAMP` to change the timestamp server) and `package.ps1` signs the DLLs, the
installer and its uninstaller with `signtool`. Unsigned installers get a SmartScreen warning
("Windows protected your PC" → More info → Run anyway).

## Development install

```powershell
tools/install.ps1             # admin (UAC): copy build/<arch> to Program Files\JIS Moonshot,
                              # register x64 + x86, add JIS Moonshot to the Korean language's keyboards
tools/install.ps1 -Update     # copy a new build in, no UAC (development installs only)
tools/install.ps1 -Uninstall
tools/install.ps1 -Log        # turn the diagnostic log on and follow it
```

Then pick **JIS Moonshot** with Win+Space. Nothing else in the language list changes: other Korean
input methods (JIS Korean too) stay where they are. The development install doesn't accept
predictive typing for you: until you do (right-click the indicator → Accept Predictive Typing…),
every key passes through.

**Restart apps after installing or updating.** An app that was running keeps the old DLL, or
none at all, until it restarts (FKeyIME `docs/issues/apps-dont-load-fkeyime.md`, the same on
Windows for any TIP).

## Using it

**Predictive typing must be accepted first** (docs/moonshot.md §6). Until
`HKCU\Software\JIS Moonshot\PredictiveAccepted` is set (Setup's notice page, or the indicator),
every key passes through untouched and the indicator's menu has only **Accept Predictive Typing…**,
which shows the notice: what is read (the keys of the text not yet committed, to rank its
readings), that it stays on this PC, that learning is a separate opt-in, and that the per-app
exclusions still apply. A left click on the indicator shows it too.

| | |
|---|---|
| Mode A: JIS Moonshot | The default. Korean 2-set keys, read at once as Korean, English and Japanese kana (JIS kana layout); the unconfirmed text shows the likeliest reading. |
| JIS Kana | Kana typing on the JIS kana layout, in hiragana or katakana (docs/moonshot.md §2.1, §3). It's in the indicator's menu; 変換 goes back to Mode A. |
| 無変換 | Commits what's shown, then English (ABC, in the English layout below). In ABC it stays there. |
| 変換 | From ABC: back to 한글, Mode A. In Mode A: the staged keys forced Korean (nothing staged: nothing). In JIS Kana: back to Mode A, the staged keys forced Korean. |
| 한/영 | Toggles ABC and 한글, as the Microsoft IME does, committing what's unconfirmed first. The state (with 無変換 / 変換) is one for the whole session, every app, and isn't stored: after a restart it's 한글. A click on the indicator does the same. JIS keyboards have no 한/영 key; 無変換 and 変換 do its job. |
| F6 / F7 | Hiragana (F6) or katakana (F7): the staged keys read as kana until the next commit, then predictive again; with nothing staged, the next segment is kana. In JIS Kana they switch its script for good. |
| F8 | Kanji: the kana reading's conversions on top until the next commit (nothing staged: the next segment). In JIS Kana, the conversions above the kana. |
| Shift+F8 | The same as F8 (kanji). JIS Moonshot has no hanja (docs/moonshot.md §4). |
| F9 / F10 | As in JIS Korean: the word's keys as full- / half-width letters. Shift+Z is no longer F10: it types ㅋ (っ on the kana layout). |
| English Layout (無変換, 한/영) | What ABC types, a selector of its own in the menu: JIS English (the default), JIS Dvorak, JIS Colemak, or **Windows' Keyboard Layout**, which passes every key to the app untouched. Shortcuts (Ctrl, Alt, Win) always go to the app; the Alt + letter variants of macOS's English modes are deliberately left out (spec §3.6.3). |
| Predictions (spec §10) | Always on. The unconfirmed text shows the top reading (`hello` shows as hello, きって as きって). ↓ / Tab and ↑ / Shift+Tab move through the list under it (up to five rows, the kana reading among them), Space / Enter commit what's shown (Enter the chosen row), Esc goes back to the top reading, a click picks a row. Excluded by default in terminals, remote desktop, password managers, File Explorer and launchers (spec §10.11.1): there, Mode A types forced Korean. |

The mode (Mode A or JIS Kana) is one setting for every app; when the engine switches it itself
(F6, F7, 変換) the setting and the indicator follow without committing anything. The indicator's
menu (right-click) also has: Off in this program, Predictions in this program (a safety exclusion,
not a switch for the feature), Learn from My Choices (Stays on This PC), Learned Weights
(read-only, with Reset What It Learned and Open Training Folder), and Enter Commits Without a New
Line.

### ABC and English: type in Mode A, and train it on your words

無変換 and 한/영 are there, and they work like the Microsoft IME's switch, but it's a plain on/off
switch, and that is all it can be. It's very hard to make it work the way people tend to expect:

- **It doesn't know what you were typing.** Pressing it commits what's unconfirmed as it is and
  then hands every key to the app. It can't go back and turn the last word into English, and
  pressing it mid-word splits the word in two.
- **English typed in ABC is invisible to JIS Moonshot's composition.** No predictions, no F9 / F10,
  no kanji, and nothing learned from it: ABC only types the English layout's characters.
- **One state for everything.** It's the same in every app and every field, and starts on 한글
  after a restart, so a field that "should" be English needs the key pressed again.
- **Mixed text costs two extra presses per switch**, where the predictions need none.

So we recommend **typing in Mode A all the time** and letting the predictions read the English
and kana words: `hello` shows as hello, `server를` as server를; F10 forces the English letters of
a word and 変換 its Korean when the prediction is wrong. Then **train the local model on the words
you use**: list them in a training file (`docs/training-contract.md`, format
`jis-moonshot-training`; JIS Korean's `jis-korean-training` files are read too) at
`%APPDATA%\JIS Moonshot\training.json` (menu → Learned Weights → Open Training Folder). Besides
Korean and English words it takes kana words (`vocabulary.japanese`, and `avoid` entries with
`"language": "japanese"`; hiragana, katakana and ー only) and a `kana` weight. It's applied the next
time JIS Moonshot activates, stays on this PC, and is forgotten again if you delete the file. Keep
ABC for typing that isn't words: passwords, code, commands, games.

**Example: `.co.kr`.** Out of the box `naver.com` types fine, but `naver.co.kr` comes out as
`naver.채.악`: `co` and `kr` aren't in the built-in English lists, and we keep them that way. Add
them yourself:

```json
{
  "format": "jis-moonshot-training",
  "version": 1,
  "vocabulary": { "english": [{ "word": "co" }, { "word": "kr" }] },
  "examples": [{ "text": "kr", "keys": "KeyK KeyR", "count": 20 }, { "text": "co.kr", "count": 20 }]
}
```

A word in `vocabulary` is enough for `co`. `kr` also needs `examples`: its keys spell 악, a
known Korean word, which a vocabulary count can't outrank (measured: still 악 at a count of
1,000,000). An example is learned as your own pick (spec §10.11), which can. Applying a file with
examples turns on Learn from My Choices.

## Settings

`HKCU\Software\JIS Moonshot` (read with `HKLM\Software\JIS Moonshot` as a fallback for apps that
can't see the user's key). The menu writes them; apps pick a change up at their next focus change.
Every write from JIS Moonshot stores a new `SettingsVersion`, and apps only re-read the settings when it
changed: a value edited by hand applies in programs started afterwards, or with the next change
from the menu.

| Value | Type | Default | |
|---|---|---|---|
| `PredictiveAccepted` | DWORD | unset | the version of the predictive-typing notice accepted (1); below 1 every key passes through (docs/moonshot.md §6) |
| `Mode` | DWORD | 0 | 0 = Mode A (JIS Moonshot), 11 = JIS Kana. The engine writes it too (変換 from JIS Kana). A mode it refuses (the retired 1, 2, 9, 10) is Mode A |
| `EnglishMode` | DWORD | 3 | ABC's layout: 3 JIS English, 7 JIS Dvorak, 8 JIS Colemak, `0xFFFFFFFF` = Windows' layout; anything else is JIS English |
| `EnterCommits` | DWORD | 0 | Enter commits without a new line (spec §5.1) |
| `LearnFromChoices` | DWORD | 0 | learning from your choices (spec §10.11) |
| `Learned` | SZ | | what was learned: one line of counts, no text or keys |
| `Weight.<Scenario>` | SZ | | user tuning (spec §10.5): `Korean` `English` `Implied` `Mixed` `TwoSet` `KeepFinals` `Hanja` `ImpliedL` `Kana`, e.g. `0.8` (`Hanja` is retired: it has no effect, but keeps its id) |
| `Apps\<program.exe>\Predictions` | DWORD | | predictions in one app, overriding the default-off list (a safety exclusion: there is no global switch) |
| `Apps\<program.exe>\Off` | DWORD | | 1: JIS Moonshot passes every key through in that program (menu: Off in <program>) |
| `DiagnosticLog` | DWORD | 0 | log activations, focus and errors (never typed text) to `%ProgramData%\JIS Moonshot\jismoonshot.log` |
| `TrainingFileDate`, `TrainingWords.*`, `TrainingAvoid.*` | SZ | | the training file last applied, its words and the words it avoids |
| `SettingsVersion` | DWORD | | changes with every write from JIS Moonshot (see above) |

## Testing

```powershell
tools/build.ps1          # shell tests: scan codes, keys typed into the engine, the JSON reader,
                         # training counts, repeated key tests, log truncation
tools/type-test.ps1      # types through the installed TIP; don't touch the keyboard meanwhile
build/x64/jk_unloadtest.exe   # loads, activates and unloads the DLL in rounds: memory mustn't grow
```

`type-test.ps1` runs `build/x64/jk_testpad.exe`, which selects JIS Moonshot for itself, types key
sequences with `SendInput` by scan code and reads the result back, in three kinds of control: a
classic Edit and a RichEdit through the IMM compatibility layer, and a RichEdit on TSF itself
(`SES_USECTF`), plus targets that misbehave the way some apps do (no input context, a composition
ended at every key, a read-only field; see the top of `tests/testpad.cpp`). `jk_testpad` alone
opens the same window to type in by hand. `type-test.ps1` accepts predictive typing for the run
and turns predictions off in `jk_testpad.exe` (`Apps\jk_testpad.exe\Predictions`) for the cases
without them, restoring both after. CI builds both architectures and runs the shell tests; typing
through the TIP needs a desktop.

What the shell does differently from what TSF's documentation suggests, and why, is in
[docs/troubleshooting/windows-tsf-notes.md](../docs/troubleshooting/windows-tsf-notes.md).

## Files

| File | What |
|---|---|
| `src/text_service.*` | One per thread: key events in, composition out, the list, focus-loss commits, 한/영 / 無変換 / 変換, the consent gate, the settings menu |
| `src/key_map.*` | Scan codes → engine keys (spec §2), and the switch keys. No Windows headers; unit-tested |
| `src/lang_bar.*` | The taskbar indicator (한あ / かな / ABC, drawn at run time) and its menu |
| `src/candidate_window.*` | The prediction list |
| `src/settings.*` | The registry settings, the consent value and the default-off apps |
| `src/training.*`, `src/training_lines.*`, `src/json.*` | Training files (spec §10.12), applied off the UI thread; `training_lines` is unit-tested |
| `src/shared_data.*` | The predictor (with `common-ja.txt` and `kanji.bin`), parsed once per process, off the UI thread, unless predictions are excluded in the program |
| `src/edit_session.*` | TSF edit sessions, the only way to change the document |
| `src/log.*` | The diagnostic log (`DiagnosticLog`) |
| `src/registration.*`, `src/dllmain.cpp` | COM and TSF registration (`regsvr32`) |
| `src/guids.h` | **Permanent** GUIDs, JIS Moonshot's own (not JIS Korean's): they end up in users' registries and language lists. Never change them |
| `tests/main.cpp`, `tests/testpad.cpp`, `tests/unload.cpp` | Shell tests, the test window, and the unload test |
| `installer/` | The installer (`jismoonshot.iss`, with the notice page) and its per-user step (`keyboards.ps1`) |
| `tools/` | `build.ps1`, `package.ps1`, `install.ps1`, `type-test.ps1`, `dev.ps1` (a VS developer shell), `make-icons.py` |

## Passwords and screen recording

The prediction list shows readings of what's being typed, so in a password box it would
show the password in other forms. JIS Moonshot protects it in layers:

- **The list never appears in screen recordings, screenshots or screen sharing**
  (`WDA_EXCLUDEFROMCAPTURE`), whatever the field.
- **Password boxes that say they are one get no input method at all.** Windows itself keeps TIPs
  out of `ES_PASSWORD` edits and fields with a password input scope (measured with JIS Korean: it
  never receives a key there). JIS Moonshot also checks for them itself (input scope, disabled keyboard,
  a password character on an Edit or RichEdit) and passes keys through if one slips by *(not
  measured: Windows caught every case we could build)*.
- **Password boxes that don't say so can't be detected.** Banking and certificate programs often
  draw their own (●●● in a plain field), and the keys then compose like any text. Use **ABC**
  (無変換, 한/영) there, or right-click the indicator → **Off in <program>**, which passes every key through
  in that program only, from then on. Measured: UniSign accepts its certificate password in ABC.
  JIS Moonshot starts off in the local agents of certificate programs found on a test PC
  (CrossCert UniSign, DreamSecurity MagicLine4NX, SignKorea; `kOffByDefault` in
  `src/settings.cpp`). Certificate windows drawn inside a web page are in the browser: use ABC.

## Known limits

- Password fields and the secure desktop (UAC, sign-in) don't use TIPs (spec §8).
- Apps that read raw keys (games) see Windows' own layout, not JIS Moonshot.
- Browsers take F10 for their menu, and Shift+Z no longer stands in for it (it types ㅋ).
- 無変換 and 変換 are caught by virtual key (`VK_NONCONVERT`, `VK_CONVERT`) or by their JIS scan
  codes (0x7B, 0x79). Which virtual keys Windows' Korean layout gives them on a JIS board isn't
  measured yet (#9, P4).
- Apps that end the composition at every key through the IMM layer still get loose jamo; TSF-native
  ones keep composing (#6, [windows-tsf-notes.md](../docs/troubleshooting/windows-tsf-notes.md)).
- Classic Win32 controls (through the IMM layer: the Run dialog, older programs' text boxes) show
  only the first character of what's being typed (きのと shows き, hello shows h) until it
  commits; what commits is whole. TSF-native apps show all of it
  ([windows-tsf-notes.md](../docs/troubleshooting/windows-tsf-notes.md)).
- Store apps that run sandboxed (AppContainer: Store apps, the Start menu's search) read the
  settings only because JIS Moonshot grants ALL APPLICATION PACKAGES read access on
  `HKCU\Software\JIS Moonshot` (at its first write or consent check in an ordinary program):
  without it they never saw the accepted notice and asked again every time (measured). They
  can't write it, so a choice made from the menu there doesn't last. Whether they can read the
  training file is *not measured yet*.
