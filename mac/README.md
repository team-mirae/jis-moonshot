# mac

The macOS input method for JIS Moonshot: an InputMethodKit app that wraps `/engine`. All behavior
is in the engine. This shell maps physical key codes to engine keys and applies the result as
inserted text and marked text. macOS is JIS Moonshot's first platform; like the rest of the project
it's an experiment ([../docs/moonshot.md](../docs/moonshot.md)).

Built without Xcode: the Command Line Tools are enough. Requirements: Rust with the
`aarch64-apple-darwin` and `x86_64-apple-darwin` targets.

```sh
make                   # build/full/JISMoonshot.app, universal (arm64 + x86_64)
make LEGACY_PREDICT=1  # build/full-legacy/JISMoonshot.app: predictions ranked by the rules only, no learned ranker
make install           # copy to ~/Library/Input Methods, register it, restart it, add the login agent
make uninstall
make dist              # build/<variant>/JISMoonshot-<version>[-legacy].zip
```

There is one build: predictions, the English modes and JIS Kana are always in. JIS Korean's
`PREDICT=0` and `ENGLISH=1` are gone. The bundle id is `me.sjun.inputmethod.JISMoonshot`, data is
in `~/Library/Application Support/JIS Moonshot/`, and it installs beside JIS Korean without
colliding.

## Input sources

Each mode is its own input source of the one input method, the way Japanese has Kana and Romaji
(spec §8.2):

| Input source ID | Shown as | Listed under |
|---|---|---|
| `me.sjun.inputmethod.JISMoonshot.2Set` | JIS Moonshot (Mode A: Korean, English and kana readings) | Korean |
| `me.sjun.inputmethod.JISMoonshot.A_KANA` | JIS Kana (kana typing) | Japanese |
| `me.sjun.inputmethod.JISMoonshot.A_QWERTY` | JIS English | English |
| `me.sjun.inputmethod.JISMoonshot.A_DVORAK` | JIS Dvorak | English |
| `me.sjun.inputmethod.JISMoonshot.A_COLEMAK` | JIS Colemak | English |

A first `make install` enables JIS Moonshot, JIS Kana and JIS English. Add the others in **System
Settings → Keyboard → Input Sources → Edit → +**. If they don't show up in the list, log out and
back in once. After later installs, newly opened apps use the new build at once, but apps that
were already running keep talking to the old one until you remove JIS Moonshot there and add it
back, quit and reopen the app, or log out and back in
([why](../docs/troubleshooting/macos-input-method-not-launching.md)).

## Predictive typing must be accepted

On the first activation JIS Moonshot shows an alert explaining predictive typing: what it reads (the
keys of the text not yet committed, to rank their readings), that it stays on this Mac, that
learning is separate and off, and that terminals and password managers start with predictions off.
Until you choose **Accept**, every key passes through untouched and the input menu has only
**Accept Predictive Typing…**. The answer is stored as `PredictiveAccepted` (the version of the
notice; a changed notice asks again).

The marked text shows the **top reading** of what you typed, Korean, English or kana. When more
than one reading is listed, ↓ and Tab move down the suggestion list and ↑ and Shift+Tab up, Enter
commits the chosen row, and Escape goes back to the top reading. Otherwise Space, Enter, Tab,
Escape, arrows, shortcuts and clicking away commit what's shown, and then act as they would on it.
The suggestion list never appears in screen sharing, recordings or screenshots.

The input menu also has **Learn from My Choices** (off by default; stays on this Mac, spec §10.11),
**Learned Weights** (what it learned, Reset What It Learned, Open Training Folder), per app
**Disable Predicting in …** and **Disable Learning from …** (terminals and some other apps start
with predictions off; there Mode A types forced Korean), **Manage Disabled Apps…**, **Enter Commits
Without a New Line**, **Korean Key (Same as かな)**, and **Let Other Apps Change the Mode**.

Tune the readings with multipliers on each one's learned probability (spec §10.5):

```sh
defaults write me.sjun.inputmethod.JISMoonshot Weight.Kana -float 0.5      # kana readings half as likely
defaults write me.sjun.inputmethod.JISMoonshot Weight.English -float 0.8
# Also Weight.Korean, Weight.Implied, Weight.Mixed, Weight.TwoSet, Weight.KeepFinals, Weight.Hanja,
# Weight.ImpliedL (default 1.0; 0 turns an alternative off)
```

The training file is `~/Library/Application Support/JIS Moonshot/training.json`
([../docs/training-contract.md](../docs/training-contract.md)); the interactive install prompt
([../docs/interactive-prompts](../docs/interactive-prompts/README.md)) ends with setting one up, and
that's strongly recommended.

## Keys

| Key | Does |
|---|---|
| 英数 | To the last English mode (JIS English by default), committing what shows. |
| かな (and カタカナ, F18) | To Mode A. In Mode A with something staged: force Korean. From JIS Kana: Mode A, the staged keys forced Korean. |
| F6 / F7 | Hiragana / katakana: the staged keys read as kana until the next commit, then predictive again. With nothing staged, the next segment is kana. In JIS Kana (input menu) they switch its script for good. |
| ⌥Space | A full-width space (U+3000) in JIS Moonshot and JIS Kana, committing what's typed first; `_` in the English modes. |
| F8 | Kanji: the kana reading's conversions on top until the next commit (nothing staged: the next segment). In JIS Kana, the conversions above the kana. |
| Shift+F8 | The hanja list (Mode A). |
| F9 / F10 | Full-width / half-width letters, as in JIS Korean. |
| Shift+Z | ㅋ, っ or Z, like any other key. |

macOS turns カタカナ (the Windows JIS keyboards' key right of the space bar) into かな, and in JIS
Moonshot they do the same thing, so there's nothing to tell apart: no hardware listener, no Input
Monitoring. かな is no longer left to macOS's Japanese input method. The **Korean Key** (F18 by
default, F13–F19 or none in the input menu, any key code with `defaults write
me.sjun.inputmethod.JISMoonshot KoreanKey -int <code>`) also acts as かな, for a key remapped to it.

To catch 英数 and かな everywhere (before apps such as iTerm that keep F-keys, and from other input
sources), JIS Moonshot runs a system-wide key tap, which needs **Accessibility** only. It never asks
at launch: choose **Allow 英数 and かな Everywhere…** in the input menu. Without it, the keys work only
in apps that pass them on to the input method. The tap lives in JIS Moonshot's process, so
`make install` also installs a **login agent**
(`~/Library/LaunchAgents/me.sjun.inputmethod.JISMoonshot.login.plist`) that starts it in the
background at login; without it the keys would do nothing after a login until JIS Moonshot was
selected once. The permission follows the build's signature: signed with an Apple Development
certificate (the Makefile picks one from the keychain), it survives rebuilds; ad hoc, every build
has to be granted again in System Settings → Privacy & Security.

JIS English types JIS QWERTY with JIS symbols, and ⌥ + a letter offers its accented variants in a
row (⌥U then 2 → ü; 1–9 and 0 pick, Esc cancels; spec §3.6.3). There's no press and hold. In
terminals ⌥ goes to the app untouched. JIS Dvorak and JIS Colemak keep ⌘ and ⌥ shortcuts on
QWERTY (spec §3.6.4).

Apple's ABC can't be removed from the input sources: macOS always keeps one plain keyboard layout
for password fields. With **ABC** itself enabled, macOS has been seen sending 英数 to ABC instead of
JIS English; the tap catches 英数 first, which is meant to make this unnecessary (not yet confirmed).
If 英数 still selects ABC, add **ABC – Extended**, then remove ABC
([#2](https://github.com/team-mirae/jis-korean/issues/2),
[troubleshooting](../docs/troubleshooting/macos-katakana-and-eisu.md)).

A Mac JIS keyboard has 英数 and かな, which is all JIS Moonshot needs. For other keyboards,
[keyboard-troubleshooting](https://github.com/themrsung/keyboard-troubleshooting) has remaps that
give them JIS's keys (with Karabiner-Elements, keep the keyboard configured as JIS).

## Logo and badges

The menu bar shows one badge per mode, drawn in [../docs/logo](../docs/logo/README.md) and shipped
as `Resources/icons/*.png` (16 and 32 px, packed into one `.tiff` each): `moonshot` (the input
method: the left half of the taegeuk beside a solid half disc), `mode-a` (with the JIS Enter),
`mode-a-kana` (KA), `mode-a-qwerty` (ABC), `mode-a-dvorak` (DV) and `mode-a-colemak` (CMK).

## The public API (spec §8.2)

Any program can read the mode, hear it change, and, if the user turned on **Let Other Apps Change
the Mode** in the input menu, change it:

```sh
defaults read me.sjun.inputmethod.JISMoonshot CurrentMode       # A, A_KANA, or an English mode (A_QWERTY…)
J=~/Library/"Input Methods"/JISMoonshot.app/Contents/MacOS/JISMoonshot
"$J" mode                                                       # A  JIS Moonshot  me.sjun…2Set
"$J" mode A_KANA                                                # needs the setting; prints the answer
```

```lua
-- Hammerspoon
hs.distributednotifications.new(function(_, _, info) print("JIS Moonshot mode", info.mode) end,
  "me.sjun.inputmethod.JISMoonshot.ModeDidChange"):start()
hs.distributednotifications.post("me.sjun.inputmethod.JISMoonshot.SetMode", nil, { mode = "A_KANA" })
```

A refused change is answered with `me.sjun.inputmethod.JISMoonshot.ModeChangeRefused` and a `reason`.

It also resolves keys and text (spec §8.3) on the engine of the app JIS Moonshot was last active in,
with that app's mode and predictions:

```sh
"$J" resolveKeyCodeInput KeyG              # {"result":"ㅎ", "app":…, "mode":"A", "predictions":true, …}
"$J" resolveKeyCodeInput KeyS 하           # the field after context 하 and the key: "한"
"$J" resolveKeyCodeInput 5 하              # macOS virtual key codes work too (5 is G)
"$J" resolveKeyCodeFromOutput ㅃ           # {"combinations":[["Digit1"],["S-KeyQ"]], …}
"$J" resolveKeyCodeFromOutput 한 3 3       # min and max key presses; a nonsensical range gives []
```

As notifications: post `me.sjun.inputmethod.JISMoonshot.ResolveKeyCodeInput` (`keyCode`, optional
`context`) or `…ResolveKeyCodeFromOutput` (`text`, `minKeypressCount`, `maxKeypressCount`) with a
`requestID`; the answer comes as the same name + `Result`, with the same `requestID`. Requests are
answered one at a time, with pauses so typing isn't held up; with more than 8 waiting, the oldest
is answered with an `error` instead.

## Files

| File | What |
|---|---|
| `Package.swift` | The shell as a Swift package, for CodeQL only (code scanning traces Swift through `swift build`); it makes no app. Build with `make`. |
| `Sources/main.swift` | Starts the `IMKServer`. |
| `Sources/InputController.swift` | One controller and engine per text client: key events in, insert/marked text out, focus-loss commit, the "Enter commits" menu item. |
| `Sources/KeyMap.swift` | macOS virtual key codes → engine key ids (spec §2). |
| `Sources/InputModes.swift` | The modes as input modes (Korean, kana, English), and the public API (spec §8.2). |
| `Sources/KoreanKeyTap.swift` | The system-wide key tap for 英数, かな (カタカナ) and the Korean Key (spec §8.2.1). Accessibility only. |
| `Sources/Consent.swift` | The predictive-typing notice and `PredictiveAccepted` (docs/moonshot.md §6). |
| `Sources/Training.swift` | The training file (docs/training-contract.md); its examples are learned in the background. |
| `Sources/SuggestionWindow.swift` | The suggestion list under the marked text (predictions, hanja candidates, letter variants). |
| `Sources/KeyboardViewerLayout.swift` | Names each mode's Keyboard Viewer layout to the app (spec §8.1), kept per app. |
| `Sources/DisabledApps.swift`, `Sources/DisabledAppsWindow.swift`, `Sources/DefaultOffApps.swift`, `Sources/KnownApps.swift` | Per-app predictions and learning (spec §10.11): the apps the user excluded, "Manage Disabled Apps…", and the apps where predictions start off. |
| `Sources/DiagnosticLog.swift` | `~/Library/Logs/JISMoonshot.log` (Notes below). |
| `Info.plist` | Bundle and input source metadata, with one input mode per engine mode. Bundle id `me.sjun.inputmethod.JISMoonshot`. |
| `Resources/icons/` | The logo and one badge per mode (16 px and 32 px PNGs), drawn in `docs/logo/` (MIT). Each pair ships as one `.tiff`. |
| `tools/register.swift` | `TISRegisterInputSource`, so a fresh install appears without logging out. Re-enables only the sources that were enabled. |
| `tools/make-keylayout` | One Keyboard Viewer layout per mode (spec §8.1). |
| `tools/add-english-mode.sh` | Adds the English modes to the built bundle's Info.plist (the checked-in one has Mode A and JIS Kana). |
| `tools/login-agent.plist` | The login agent `make install` puts in `~/Library/LaunchAgents`. |

## Notes

- Shortcuts (Ctrl, Option, Cmd) commit first, then go to the app unchanged.
- Debug logs: `/usr/bin/log stream --predicate 'process == "JISMoonshot"'` (in zsh, `log` alone is a shell builtin), and the input method's own `~/Library/Logs/JISMoonshot.log`, off by default: `defaults write me.sjun.inputmethod.JISMoonshot DiagnosticLog -bool true` turns it on immediately, `-bool false` off. It logs launches, activations and deactivations (with the input source macOS has selected and the mode JIS Moonshot is in), mode switches and the fields that resync their engine or layout to them, Keyboard Viewer layouts named, one line per key without the key (whether the engine took it, the field's engine mode and layout, and the keyboard type as the event and macOS's global report it), keys that took over 100 ms, commits the system or an app forced (their length only), and each unmapped key code once per app. Never any text.
- Bare jamo instead of syllables (ㅂㅏㄷㅏ for 바다) in one field means the app typed the keys through the mode's keyboard layout without asking the input method: password fields (macOS bypasses input methods there), and in Chrome 152–156 any field that once looked like a password (Chromium issue 567321058, fixed in 157). Apple's Korean input method types bare jamo there too. In an ordinary field, turn the log on and type a few keys there: the `key in …` lines show whether the keys reached the input method, and what the field's engine and layout were. Full diagnosis: [docs/troubleshooting/macos-bare-jamo-in-chrome.md](../docs/troubleshooting/macos-bare-jamo-in-chrome.md).

- Input method installed but not launching after a reinstall, or Keyboard Viewer showing ABC
  legends: see [docs/troubleshooting/macos-input-method-not-launching.md](../docs/troubleshooting/macos-input-method-not-launching.md).
