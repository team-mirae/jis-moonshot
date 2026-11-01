import Cocoa
import InputMethodKit

#if ENGLISH
/// The language keys, caught system-wide (docs/moonshot.md §4, spec §8.2.1), before any app sees them:
///
/// - **かな** (and **カタカナ**, which macOS turns into かな, and **the Korean Key**, F18 by default,
///   for keys remapped to it): to Mode A, JIS Moonshot's predictive mode, from an English mode or
///   another input source; in Mode A, forces the staged keys' Korean reading; from JIS Kana, back to
///   Mode A with the staged keys forced Korean. Some apps keep function keys to themselves (iTerm
///   keeps F1–F19), so it's caught here too.
/// - **英数**: to the last English mode. Left to macOS, it selects Apple's ABC when ABC is enabled
///   (#2), not JIS English.
///
/// The tap needs Accessibility permission. JIS Moonshot never asks for it on its own: only the input
/// menu's "Allow 英数 and かな Everywhere…" does. It runs only when one of JIS Moonshot's modes is
/// added and the permission is granted. It swallows a key (down and up, repeats included) only when
/// the mode it goes to is added. Keys with Ctrl, Option or Cmd held go through untouched.
enum KoreanKeyTap {
    private static var tap: CFMachPort?
    private static var polling = false
    /// Whether Mode A / an English mode of JIS Moonshot is added in System Settings, refreshed when
    /// the input sources change: the tap's callback reads these instead of asking macOS per key.
    private static var koreanAdded = false
    private static var englishAdded = false
    private static var lastCheck = Date.distantPast
    private static var lastUseful: Bool?
    private static let notices = EnabledSourcesNotices()

    /// Starts the tap if it's of use and its permission is granted, without asking. At launch, when
    /// the added input sources change, and now and then on activation (`checkSoon`).
    static func check() {
        notices.observe()
        lastCheck = Date()
        refreshAdded()
        let useful = englishAdded || koreanAdded
        if useful != lastUseful {
            lastUseful = useful
            DiagnosticLog.note(useful ? "language key tap: wanted (Accessibility \(AXIsProcessTrusted()))"
                                      : "language key tap: not needed (no mode added)")
        }
        if useful {
            start(ask: false)
        }
    }

    /// `check`, at most every 30 seconds.
    static func checkSoon() {
        guard Date().timeIntervalSince(lastCheck) > 30 else { return }
        check()
    }

    static func refreshAdded() {
        let ids = InputModes.enabledSourceIDs()
        koreanAdded = InputModes.all.contains { $0.korean && ids.contains($0.sourceID) }
        englishAdded = InputModes.all.contains { $0.english && ids.contains($0.sourceID) }
    }

    /// Starts the tap. `ask` (from the input menu): asks for Accessibility, and checks every few
    /// seconds until it's granted. Otherwise starts only if it's already allowed, and never asks.
    static func start(ask: Bool) {
        if ask {
            refreshAdded()
        }
        guard tap == nil else { return }
        let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: ask] as CFDictionary
        guard AXIsProcessTrustedWithOptions(options) else {
            if ask && !polling {
                polling = true
                poll()
            }
            return
        }
        let mask = (1 << CGEventType.keyDown.rawValue) | (1 << CGEventType.keyUp.rawValue)
        guard let port = CGEvent.tapCreate(tap: .cgSessionEventTap, place: .headInsertEventTap, options: .defaultTap,
                                           eventsOfInterest: CGEventMask(mask), callback: callback, userInfo: nil)
        else {
            DiagnosticLog.note("language key tap: couldn't create the tap")
            return
        }
        tap = port
        CFRunLoopAddSource(CFRunLoopGetMain(), CFMachPortCreateRunLoopSource(nil, port, 0), .commonModes)
        CGEvent.tapEnable(tap: port, enable: true)
        DiagnosticLog.note("language key tap: on")
    }

    /// After asking: waits for Accessibility, without asking again.
    private static func poll() {
        DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
            guard tap == nil else {
                polling = false
                return
            }
            if AXIsProcessTrusted() {
                polling = false
                start(ask: false)
            } else {
                poll()
            }
        }
    }

    private static let callback: CGEventTapCallBack = { _, type, event, _ in
        if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
            if let tap { CGEvent.tapEnable(tap: tap, enable: true) }
            return Unmanaged.passUnretained(event)
        }
        let code = Int(event.getIntegerValueField(.keyboardEventKeycode))
        let key = UserDefaults.standard.object(forKey: "KoreanKey") as? Int ?? kVK_F18
        if code == kVK_JIS_Kana || code == kVK_JIS_Eisu || code == key {
            DiagnosticLog.note("tap: key code \(code) \(type == .keyDown ? "down" : "up")")
        }
        guard event.flags.intersection([.maskCommand, .maskControl, .maskAlternate]).isEmpty else {
            return Unmanaged.passUnretained(event)
        }
        let press = type == .keyDown && event.getIntegerValueField(.keyboardEventAutorepeat) == 0
        if (code == kVK_JIS_Kana || code == key) && koreanAdded {
            if press { DispatchQueue.main.async { goToKana() } }
        } else if code == kVK_JIS_Eisu && englishAdded {
            if press { DispatchQueue.main.async { goToEnglish() } }
        } else {
            // Nothing to go to: the key goes on as usual (to the app, or to macOS).
            return Unmanaged.passUnretained(event)
        }
        return nil  // swallowed: no app sees it
    }

    /// かな, カタカナ or the Korean Key, from anywhere (docs/moonshot.md §4).
    static func goToKana() {
        // The field being typed in (JKInputController.typing). Any other field follows on its next key.
        if InputModes.selectedNow() != nil, let controller = JKInputController.typing {
            controller.kanaKey()
        } else if let target = InputModes.kanaTarget {
            // Another input source is selected (Japanese, ABC): select Mode A, and record it, so
            // the fields follow even if macOS doesn't tell their sessions.
            InputModes.becameCurrent(target)
            InputModes.select(target)
        }
        DiagnosticLog.note("かな (tap)")
    }

    /// 英数: to JIS English, from anywhere.
    static func goToEnglish() {
        guard let target = InputModes.eisuTarget else { return }
        if InputModes.selectedNow() != nil, let controller = JKInputController.typing {
            controller.switchFromTap(to: target)
        } else {
            InputModes.becameCurrent(target)
            InputModes.select(target)
        }
        DiagnosticLog.note("英数 (tap)")
    }
}

/// macOS's notice that input sources were added or removed (a selector needs an object).
private final class EnabledSourcesNotices: NSObject {
    private var observing = false

    func observe() {
        guard !observing else { return }
        observing = true
        // Delivered even while suspended, as PublicAPI's notices are: an input method is never the
        // active app.
        DistributedNotificationCenter.default().addObserver(
            self, selector: #selector(changed(_:)),
            name: Notification.Name(kTISNotifyEnabledKeyboardInputSourcesChanged as String), object: nil,
            suspensionBehavior: .deliverImmediately)
    }

    @objc func changed(_ note: Notification) {
        KoreanKeyTap.check()
    }
}
#endif
