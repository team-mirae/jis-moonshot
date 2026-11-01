# Set up JIS Moonshot on macOS

You come here from [start.md](start.md) with the session notes: the clone, the keyboards, and the
remap answer. [rules.md](rules.md) applies to every step: ask before each scan, and ask before each
install or change, one at a time.

Facts come from [docs/moonshot.md](../moonshot.md), [mac/README.md](../../mac/README.md) and spec
§8.1–§8.2.1. Read them before you explain anything, and follow them where this prompt is less
precise. macOS is JIS Moonshot's first platform.

## 1. Check what's there (with permission)

Each of these is one read-only check. Ask before running it, as rules.md says, or give it to the
user to run.

| To learn | Command |
|---|---|
| macOS version | `sw_vers -productVersion` |
| Command Line Tools | `xcode-select -p` |
| Rust | `rustup target list --installed` (or `~/.cargo/bin/cargo --version`) |
| Connected keyboards | `ioreg -r -c IOHIDInterface -l` (the entries with usage page 1 and usage 6: `Product`, `Transport`) |
| The type macOS stored for each keyboard | `defaults read /Library/Preferences/com.apple.keyboardtype` (40 ANSI, 41 ISO, 42 JIS) |
| Karabiner-Elements running | `pgrep -fl Karabiner` |
| JIS Moonshot or JIS Korean already installed | `ls ~/Library/Input\ Methods/ ~/Library/LaunchAgents/` |

JIS Korean and JIS Moonshot install side by side (different bundle ids, input sources and
settings). If JIS Korean is installed, say that both will be in the input menu, and that both
catch 英数 system-wide if both have their key tap on: suggest keeping only one of them added in
System Settings while trying JIS Moonshot. Removing JIS Korean is the user's call; don't suggest it
unprompted.

## 2. Build

It needs, each its own question (rules.md section 2):
1. **Command Line Tools** (`xcode-select --install`, Apple's, about 1 GB). No Xcode needed.
2. **Rust** through rustup, with the `aarch64-apple-darwin` and `x86_64-apple-darwin` targets.
   Rust can come from rustup.rs or from Homebrew. If Homebrew isn't installed, that's a third
   question of its own. `cargo` lives in `~/.cargo/bin`, which may not be on `PATH`.

Then, in `mac/`: `make` builds `build/full/JISMoonshot.app` (one build: predictions, JIS Kana and the
English modes always in; `LEGACY_PREDICT=1` ranks without the learned ranker). Running the engine's
tests first is a good check: `cargo test` in `engine/` (a few minutes).

**Signing:** without an Apple Development certificate in the keychain, the build is signed ad hoc,
and every rebuild counts as a new app, so Accessibility (step 5) has to be granted again after each.
Say this before the first build.

## 3. Install

`make install` in `mac/` is a change: ask first, and say what it does:
- copies `JISMoonshot.app` to `~/Library/Input Methods` (no administrator password);
- registers it, and on a first install enables **JIS Moonshot**, **JIS Kana** and **JIS English**
  as input sources;
- adds a login agent (`~/Library/LaunchAgents/me.sjun.inputmethod.JISMoonshot.login.plist`) that
  starts it at login, so 英数 and かな work before it's been selected.

If the input sources don't appear: log out and back in once
([troubleshooting](../troubleshooting/macos-input-method-not-launching.md)). Apps that were already
open use it after they're reopened.

## 4. Accept predictive typing

The first time JIS Moonshot is selected in a text field, it shows **"Accept predictive typing?"**
(docs/moonshot.md §6). Until the user accepts, every key passes through untouched, and the input
menu shows only "Accept Predictive Typing…". Tell them this before they select it, so the alert
isn't a surprise, and remind them of what they agreed to in start.md. The user clicks Accept
themselves: never script it or write `PredictiveAccepted` for them.

## 5. Accessibility, for 英数 and かな everywhere

JIS Moonshot catches 英数, かな (and カタカナ, and the Korean Key, F18 by default) system-wide with a
key tap, so they work in apps that keep keys to themselves and from other input sources. That needs
**Accessibility**. It never asks at launch: the prompt appears when the user chooses **Allow 英数
and かな Everywhere…** in the input menu. Explain it before that; the user grants it in System
Settings → Privacy & Security → Accessibility. Never script it. No Input Monitoring is needed.

If Apple's Japanese input method is enabled, かな no longer reaches it while JIS Moonshot's tap is
on: JIS Moonshot takes かな for its predictive mode. Say so. F6 / F7 (and JIS Kana, in the input menu) are JIS Moonshot's own
kana typing.

## 6. Switch keys, by keyboard and remap answer

| Key | Goes to |
|---|---|
| 英数 | the last English mode (JIS English by default) |
| かな (also カタカナ, F18) | Mode A, JIS Moonshot (Korean, English and kana readings); in Mode A, forces the staged text Korean |
| F6 / F7 | hiragana / katakana for the text being typed, until it's committed |

- **A Mac JIS keyboard** (MacBook built-in, Magic Keyboard JIS) and **a Windows JIS keyboard**:
  nothing to remap.
- **A JIS board without 英数 / かな** (some HHKB modes): by the remap answer. Karabiner: a simple
  modification to `japanese_eisuu` and `japanese_kana`. Firmware: LANG2 (英数) and LANG1 (かな).
- **No remapping at all:** Ctrl+Space and the input menu select the modes; F6 / F7 still reach
  JIS Kana.

## 7. Check it works

Ask the user to type, in TextEdit or any text field, in JIS Moonshot (Mode A):
- `G K S R M F` → 한글 shows; Space commits it;
- `B Y I A F` → こんにちは shows (the kana reading on top); Space commits it;
- `H E L L O` → hello;
- `G K S`, then かな → 한 (forced Korean); then F6 → きのと; then かな → 한 again; Space, then
  `G K S` → 한 (predictive again after the commit);
- 英数 → JIS English; かな → back.

If something fails, turn on the diagnostic log (ask first: it's a change: `defaults write
me.sjun.inputmethod.JISMoonshot DiagnosticLog -bool true`, read `~/Library/Logs/JISMoonshot.log`) and
read [mac/README.md](../../mac/README.md) and [docs/troubleshooting](../troubleshooting/) before
guessing.

## 8. Wrap up, then the training setup

List what was installed, what was changed, and how to undo each:
- `make uninstall` in `mac/` removes JIS Moonshot and its login agent;
- System Settings → Keyboard → Input Sources removes the modes;
- `defaults delete me.sjun.inputmethod.JISMoonshot` forgets its settings, including the acceptance
  and what it learned;
- Karabiner's rules are undone in Karabiner-Elements.

Then go on to **[training.md](training.md)**, and recommend it strongly. The training folder on
macOS is `~/Library/Application Support/JIS Moonshot/` (the input menu's Learned Weights → Open
Training Folder).
