# macOS: 英数 selects ABC (and カタカナ, now the same as かな)

Seen 2026-10-22 on macOS 27.0.1, `main` at `cd71e0d` built with `make ENGLISH=1`, an HHKB Studio JIS
over Bluetooth through Karabiner-Elements 16.3.0 (remapping only Pause → Play/Pause), on a MacBook
with a US ANSI built-in keyboard, with JIS Korean. Both problems are about the switch keys of spec
§8.2.1. Names, log lines and facts below are updated to JIS Moonshot (`JISMoonshot`,
`me.sjun.inputmethod.JISMoonshot`), where the first problem no longer exists.

Turn on the input method's own log first; every step below reads it:

```sh
defaults write me.sjun.inputmethod.JISMoonshot DiagnosticLog -bool true   # off again: -bool false
tail -f ~/Library/Logs/JISMoonshot.log
```

## カタカナ acts as かな

Not a problem any more. JIS Korean sent カタカナ to Korean and left かな to macOS's Japanese input
method, so it had to tell the two apart: macOS turns カタカナ into かな (key code 104), and JIS Korean
read the keyboard itself (HID usage 0x88, with Input Monitoring) from keyboards configured as JIS.
That needed the keyboard, and Karabiner's virtual keyboard, stored as JIS in
`/Library/Preferences/com.apple.keyboardtype` (type 42), which they often weren't.

In JIS Moonshot カタカナ and かな do the same thing (to Mode A, or force Korean there; spec §8.2.1),
so there is nothing to tell apart: no hardware listener and no Input Monitoring. Both are caught by
the key tap, which needs Accessibility only (the input menu's "Allow 英数 and かな Everywhere…"). A
press logs:

```
tap: key code 104 down
かな (tap)
```

If nothing happens, check that Mode A (JIS Moonshot) is added in System Settings and that
Accessibility is granted; without it the key works only in apps that pass it on to the input method.

## 英数 selects ABC instead of JIS English

[#2](https://github.com/team-mirae/jis-korean/issues/2). With Apple's **ABC** enabled, 英数 goes
to ABC, although JIS English (`me.sjun.inputmethod.JISMoonshot.A_QWERTY`) is added and the input
method's key tap sees the key (`tap: key code 102 down`). This answers the open question of spec §8.2.1:
with ABC present, macOS picks ABC.

**Workaround:** replace ABC with **ABC – Extended**. In System Settings → Keyboard → Input Sources →
Edit…, first add English → ABC – Extended, then remove ABC (macOS keeps at least one plain keyboard
layout, so ABC can't be removed while it's the only one). 英数 then goes to JIS English, and
ABC – Extended still types in password fields.

## Scripting input sources on macOS 27

`TISEnableInputSource` and `TISDisableInputSource` return `noErr` on macOS 27.0.1 but the change
doesn't persist (the source stays as it was, also in `com.apple.HIToolbox`). Add and remove input
sources in System Settings. `TISSelectInputSource` works.
