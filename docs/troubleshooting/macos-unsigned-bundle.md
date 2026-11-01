# macOS: JIS Moonshot is selectable but types nothing (unsigned bundle)

Investigated 2026-11-02 on macOS 27.0 (Darwin 27.0.0), repo at `b886240`. Times are local (UTC+9);
the input method's own log writes UTC. **Status: cause confirmed and fixed by a rebuild.** The
missing signature itself isn't explained: see "Open" below.

## Symptoms

- JIS Moonshot shows in the input menu, and selecting it changes the menu-bar icon and its modes.
- Nothing types at all, in any mode (2-Set or the English modes), in every app. The only way to
  type is to switch to another input source (here: Ctrl+Space to ABC – Extended).
- The "Accept predictive typing?" alert never appears, and setting `PredictiveAccepted` by hand
  (`defaults write me.sjun.inputmethod.JISMoonshot PredictiveAccepted -int 1`) changes nothing.
  That's expected: the alert and the consent check both run only after an app has connected
  (`activateServer`, `handle(_:client:)`), and in this failure no app ever connects.
- Killing and relaunching `JISMoonshot` (even `kill -9`) changes nothing.
- The `JISMoonshot` process is running. With `DiagnosticLog` on, `~/Library/Logs/JISMoonshot.log`
  has `macOS selected …` / `mode …` lines (those come from the input-source change notification,
  which needs no client) but **no** `activated in …` and no `key in …` lines.

## Evidence

`imklaunchagent` gets a request from each app for the endpoint and turns it down:

```
imklaunchagent [com.apple.inputmethodkit:Server] Refusing connection name for bundle: unrecognized 'InputMethodConnectionName' value
imklaunchagent IMKLaunchAgent -getIMKXPCEndpointForBundle: NO Endpoint, Bail & Post to request completion queue!
imklaunchagent IMKLaunchAgent -requestIMKXPCEndpointInvalid: received notification Request for Endpoint Invalid (from pid_t: …)
```

The first `Refusing` line was at 2026-11-01 20:01, the minute the bundle was installed, and it
repeated on every attempt to type after that. JIS Korean, installed next to it and working, got
none.

The installed bundle had only the linker's ad-hoc signature, not the one `make` applies:

```
$ codesign -dv ~/Library/Input\ Methods/JISMoonshot.app
Identifier=JISMoonshot              # the executable's name, not the bundle id
Signature=adhoc
TeamIdentifier=not set
$ codesign --verify --deep --strict ~/Library/Input\ Methods/JISMoonshot.app
…: code object is not signed at all
```

`build/full/JISMoonshot.app` was the same, so the bad signature came from the build, not from
`make install`. A good bundle reads `Identifier=me.sjun.inputmethod.JISMoonshot`, a
`TeamIdentifier`, and verifies cleanly.

## Cause

The bundle wasn't sealed, so its `Info.plist` wasn't bound to the signature and the code
identifier wasn't the bundle id, and `imklaunchagent` rejected the connection name. (Inferred: the
log only says the value is "unrecognized". Signing correctly, with nothing else changed, made it
go away.) No app got an endpoint, so the input method never
got a client, and no key produced any text.

## Fix

```sh
cd mac
touch Info.plist     # forces the bundle rule (and its codesign) to run again
make
codesign --verify --deep --strict build/full/JISMoonshot.app && echo ok
make install
```

Then, as after any reinstall
([macos-input-method-not-launching.md](macos-input-method-not-launching.md)): remove JIS Moonshot
in System Settings → Keyboard → Input Sources and add it back, and quit and reopen the apps that
were open (or log out). Verified: after this, `imklaunchagent` logged no further `Refusing` lines,
and the log showed `activated in …` for the new process, and typing worked.

## Diagnosing it quickly

```sh
codesign --verify --deep --strict ~/Library/Input\ Methods/JISMoonshot.app
/usr/bin/log show --last 10m --info --style compact \
  --predicate 'process == "imklaunchagent" AND eventMessage CONTAINS "Refusing"'
```

If either one fails, rebuild as above. Restarting the process, re-registering it, or setting
`PredictiveAccepted` won't help.

## Open

- Why the 2026-11-01 build was left unsigned. The bundle rule ends in
  `codesign --force --sign "$(SIGN_IDENTITY)" $@`, and rebuilding with the same Makefile signed
  correctly. Possible causes, none confirmed: the codesign step failed (keychain locked or
  unavailable, e.g. an SSH session) and the half-built bundle was kept, or the bundle was
  assembled outside `make`. If `codesign` failing doesn't fail the build in some path, `install`
  should refuse a bundle that doesn't pass `codesign --verify --strict`.
