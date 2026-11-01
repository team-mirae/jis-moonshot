# Set up JIS Moonshot on Windows

You come here from [start.md](start.md) with the session notes: the clone, the keyboards, and the
remap answer. [rules.md](rules.md) applies to every step: ask before each scan, and ask before each
install or change, one at a time.

Facts come from [docs/moonshot.md](../moonshot.md), [windows/README.md](../../windows/README.md) and
spec §8.4. Read them before you explain anything, and follow them where this prompt is less
precise. Before changing how keys reach apps, read
[windows-tsf-notes.md](../troubleshooting/windows-tsf-notes.md). Windows is JIS Moonshot's second
platform: say that its shell is less tried than macOS's.

## 1. Check what's there (with permission)

Each of these is one read-only check (PowerShell). Ask before running it, as rules.md says, or give
it to the user to run.

| To learn | Command |
|---|---|
| Windows version | `[Environment]::OSVersion.Version` or `winver` |
| Keyboards | `Get-PnpDevice -Class Keyboard -PresentOnly \| Select FriendlyName` |
| The hardware keyboard type (JIS: Japanese 106/109) | Settings → Time & language → Language & region → Japanese or Korean → Keyboard layout, or `Get-ItemProperty HKLM:\SYSTEM\CurrentControlSet\Services\i8042prt\Parameters` (`LayerDriver JPN`) |
| Languages and input methods | `Get-WinUserLanguageList` |
| Build tools | `cmake --version`, `ninja --version`, `rustup target list --installed` |
| Visual Studio Build Tools | `& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -products * -format text` |
| JIS Moonshot or JIS Korean already installed | `Test-Path "$env:ProgramFiles\JIS Moonshot", "$env:ProgramFiles\JIS Korean"` |

JIS Korean and JIS Moonshot install side by side. If JIS Korean is installed, say both will be in
the Win+Space list; removing it is the user's call.

## 2. Build

Follow [windows/README.md](../../windows/README.md). Each missing tool is its own question (rules.md
section 2): Visual Studio Build Tools (C++), CMake and Ninja, Rust with the MSVC targets, and Inno
Setup for the installer. `tools/build.ps1` builds x64 and x86 and runs the shell tests; there is
one build (predictions always on).

## 3. Install

Either the installer built by `tools/package.ps1`, or `tools/install.ps1` (administrator). Either is
a change: ask first, and say what to expect:
- **administrator rights (UAC)** once; an unsigned installer triggers SmartScreen (**More info →
  Run anyway**, for this file only; never turn SmartScreen off);
- the installer shows the **predictive typing notice**, which must be accepted to install
  (docs/moonshot.md §6). With `install.ps1`, JIS Moonshot asks from its indicator menu instead
  ("Accept Predictive Typing…"), and passes every key through until it's accepted;
- it adds JIS Moonshot to the user's Korean keyboards, adding Korean to the language list if it
  isn't there, without removing anything. Say this before: it changes their language list.

Pick **JIS Moonshot** with Win+Space and **restart apps** that were open.

## 4. The keyboard type

On a JIS keyboard, ¥ and ろ only arrive when Windows has the Japanese (106/109) hardware keyboard
type. If step 1 showed otherwise, offer the change (Settings → Time & language → Language & region →
Korean → Language options → Change layout → Japanese keyboard (106/109 key)). It needs a restart:
the user does it.

## 5. Switch keys

| Key | Goes to |
|---|---|
| 無変換 | English (ABC, typed with JIS English by default) |
| 変換 | back to JIS Moonshot (Mode A); in Mode A or JIS Kana, forces the staged text Korean |
| 한/영 | toggles English and JIS Moonshot, as the Microsoft Korean IME does |
| F6 / F7 | hiragana / katakana for the text being typed, until it's committed |

A JIS keyboard has 無変換 and 変換: nothing to remap. A keyboard without them: by the remap answer,
map keys to `VK_NONCONVERT` / `VK_CONVERT` (PowerToys Keyboard Manager, or firmware). The indicator
menu works on every keyboard.

## 6. Check it works

Ask the user to type, in Notepad: `G K S R M F` → 한글; `B Y I A F` → こんにちは; `H E L L O` →
hello; `G K S` then 変換 → 한; F6 → きのと; 無変換 → English; 変換 → back.

If something fails, read [windows/README.md](../../windows/README.md) and
[windows-tsf-notes.md](../troubleshooting/windows-tsf-notes.md) before guessing.

## 7. Wrap up, then the training setup

List what was installed and changed, and how to undo each: Apps → JIS Moonshot → Uninstall (or
`tools/install.ps1 -Uninstall`), the language list in Settings, `HKCU\Software\JIS Moonshot` for its
settings, PowerToys' remaps in PowerToys.

Then go on to **[training.md](training.md)**, and recommend it strongly. The training folder on
Windows is `%APPDATA%\JIS Moonshot\` (the indicator menu's Learned Weights → Open Training Folder).
