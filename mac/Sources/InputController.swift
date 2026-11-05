import Cocoa
import InputMethodKit

private let enterCommitsDefault = "EnterCommits"
#if ENGLISH
/// The Korean Key (spec §8.2.1): a virtual key code that always selects Korean: from an English
/// mode the last Korean mode, in a Korean mode that mode again (so macOS shows it under the caret).
/// F18 by default; -1: none.
private let switchKeyDefault = "KoreanKey"

private func registerSwitchDefaults() {
    UserDefaults.standard.register(defaults: [switchKeyDefault: kVK_F18])
}
#endif
#if PREDICT
/// Learning from the user's choices (spec §10.11): opt-in, stays on this Mac.
private let learningDefault = "LearnFromChoices"
/// What was learned: one line of counts, no text or keys.
let learnedDefault = "Learned"
/// The scenarios' names (`Weight.<name>`), by engine id.
/// By weight id (`JK_WEIGHT_*`). Hanja (6) is retired: no reading uses it.
let weightNames = ["Korean", "English", "Implied", "Mixed", "TwoSet", "KeepFinals", "Hanja", "ImpliedL", "Kana"]
#endif

/// One controller per text input client; each owns its own engine. All behavior is in the engine
/// (docs/spec.md): this class only moves key events in and text out, and shows the suggestion list.
@objc(JKInputController)
final class JKInputController: IMKInputController {
    /// This controller's engine. The public API resolves keys on a copy of it (spec §8.3).
    let engine = jk_new()
    private var hasMarkedText = false
    /// The controller JIS Moonshot was last activated in: the focused app's, while JIS Moonshot is the
    /// input source. The public API answers for it.
    private(set) static weak var focused: JKInputController?
    /// The controller that last received a key: the field really being typed in. Chrome activates
    /// its fields (the omnibox, the page) often and out of order, so the last activated one isn't
    /// always it.
    private(set) static weak var keyed: JKInputController?
    /// The controller a switch key acts on, in place: the one that last got a key when it's in the
    /// app JIS Moonshot was last activated in (Chrome activates its fields out of order, so the last
    /// activated one isn't always the field typed in), else the last activated one (the user moved
    /// to another app and hasn't typed there yet).
    static var typing: JKInputController? {
        if let keyed, keyed.appID == focused?.appID { return keyed }
        return focused ?? keyed
    }
    /// Unmapped key codes already logged, per app (see `handle`).
    private static var unmappedNoted = Set<String>()
    /// The mode this controller's engine is in (`jk_set_mode`), nil before the first.
    private var modeID: UInt32?
    /// The app it types into, and whether predictions are on there (spec §10.11).
    private(set) var appID: String?
    private(set) var predicting = false
    #if PREDICT
    /// The ranked readings of what's staged (spec §10), shown in the suggestion list.
    private var predictions: [String] = []
    /// Which of them are Japanese readings (kanji, kana): shown in Hiragino Sans, so kanji take
    /// their Japanese (shinjitai) forms whatever the system language (docs/moonshot.md §5.1).
    private var japanese: [Bool] = []
    /// The row chosen with ↓ / Tab, or nil while nothing is chosen (the top reading applies).
    private var chosen: Int?
    /// The reading shown as the marked text in place of the Korean one: the chosen row, or the top
    /// prediction when it isn't the Korean reading as typed. Nil: the engine's own preedit shows.
    private var shown: String?
    /// What shows while no row is chosen: the top prediction when it isn't the Korean reading as typed.
    private var topShown: String?
    /// The bundle identifier of the app this controller types into (for the per-app settings).
    private var clientApp: String?
    #endif

    override init!(server: IMKServer!, delegate: Any!, client inputClient: Any!) {
        super.init(server: server, delegate: delegate, client: inputClient)
        // ⌥Space: a full-width space, or `_` in English (docs/moonshot.md §4). macOS only.
        jk_set_option_space(engine, 1)
        #if PREDICT
        UserDefaults.standard.register(defaults: [learningDefault: false])
        applyPredictionsSetting()
        #endif
    }

    deinit {
        jk_free(engine)
    }

    override func recognizedEvents(_ sender: Any!) -> Int {
        Int(NSEvent.EventTypeMask.keyDown.rawValue | NSEvent.EventTypeMask.flagsChanged.rawValue)
    }

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        // Cmd or Ctrl held: shortcuts at their QWERTY positions in every app (KeyboardViewerLayout).
        // The modifier itself always goes on to the app.
        if let event, event.type == .flagsChanged, let client = sender as? IMKTextInput {
            let flags = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
            KeyboardViewerLayout.shortcutModifiers(down: !flags.isDisjoint(with: [.command, .control]), for: client)
            return false
        }
        guard let event, event.type == .keyDown, let client = sender as? IMKTextInput else { return false }
        // A key without Cmd or Ctrl: an app left on the QWERTY layout (Cmd+Tab released Cmd
        // elsewhere) gets its own back before the key.
        let shortcutFlags = event.modifierFlags.intersection([.command, .control])
        KeyboardViewerLayout.shortcutModifiers(down: !shortcutFlags.isEmpty, for: client)
        // Predictive typing must be accepted before JIS Moonshot types anything (docs/moonshot.md §6):
        // until then every key goes to the app untouched.
        guard Consent.accepted else {
            Consent.askSoon()
            return false
        }
        let started = Date()
        defer { noteSlowKey(started, client) }
        Self.keyed = self
        syncMode(client)
        if engineKey(event.keyCode) == UInt32(JK_KEY_OTHER) {
            // Keys JIS Moonshot doesn't map, with the app: shows which keys an app passes on at all
            // (iTerm keeps F1–F19 to itself) and what code a remapped key arrives as. Once per code
            // and app, so what's typed on unmapped keys (a PIN on the keypad) isn't in the log.
            let note = "unmapped key code \(event.keyCode) in \(client.bundleIdentifier() ?? "?")"
            if Self.unmappedNoted.insert(note).inserted {
                DiagnosticLog.note(note)
            }
        }
        #if ENGLISH
        if let handled = handleSwitchKey(event, client) {
            return handled
        }
        #endif
        // In a terminal, ⌥ in an English mode goes to the app untouched: readline's ⌥C, ⌥U and ⌥L
        // with Option as Meta, not letter variants (spec §3.6.3).
        if InputModes.current.english && event.modifierFlags.contains(.option)
            && KnownApps.isTerminal(client.bundleIdentifier()) {
            return false
        }
        // The auto-repeat of a held key, and Caps Lock: the English modes' variants list reads
        // them (spec §3.6.3); the Korean modes ignore them.
        var extra = event.isARepeat ? UInt32(JK_MOD_REPEAT) : 0
        if event.modifierFlags.contains(.capsLock) { extra |= UInt32(JK_MOD_CAPS) }
        let mods = modifiers(event.modifierFlags) | extra
        #if PREDICT
        if let handled = handleList(event, client) {
            return handled
        }
        // A key that commits takes the reading on screen, not the Korean one underneath, and then
        // does to it what it does to any text (Shift+Space adds a space, Shift+Enter only commits).
        // A row chosen with ↓ / Tab is kept by Space too, which stages a space after it
        // (jk_key_keeps_choice, docs/moonshot.md §4).
        let key = engineKey(event.keyCode)
        let keepsChoice = chosen != nil && jk_key_keeps_choice(engine, key, mods) != 0
        if keepsChoice || (shown != nil && jk_key_commits(engine, key, mods) != 0) {
            let consumed = jk_key_after_prediction(engine, chosen ?? 0, engineKey(event.keyCode), mods)
            chosen = nil
            apply(to: client)
            saveLearned()
            return consumed != 0
        }
        chosen = nil
        #endif
        let consumed = jk_key(engine, engineKey(event.keyCode), mods)
        apply(to: client)
        convertHostText(client)
        // F6 and F7 switch the engine to JIS Kana itself (docs/moonshot.md §4).
        followEngineMode(client)
        #if PREDICT
        saveLearned()
        #endif
        noteKey(consumed: consumed != 0, event, client)
        return consumed != 0
    }

    /// One line per key, without the key: whether the engine took it, and the state that decides
    /// what the app types when it didn't. Shows whether keys reach the input method at all (an app
    /// can type through the layout without asking it).
    /// Also the keyboard type, as the event carries it and as macOS's global has it (`typingOnISO`
    /// reads the global): whether the event's can replace it on Macs with several kinds of keyboard (#12).
    private func noteKey(consumed: Bool, _ event: NSEvent, _ client: IMKTextInput) {
        guard DiagnosticLog.isOn else { return }
        let app = client.bundleIdentifier()
        let eventType = event.cgEvent.map { String($0.getIntegerValueField(.keyboardEventKeyboardType)) } ?? "none"
        DiagnosticLog.note("key in \(app ?? "?"): consumed \(consumed), engine mode \(modeID.map(String.init) ?? "none"), layout \(KeyboardViewerLayout.named(to: app) ?? "none"), current \(InputModes.current.letter), keyboard type \(eventType) (event) \(LMGetKbdType()) (global)")
    }

    /// A key that held the app for long: logged with its time.
    private func noteSlowKey(_ started: Date, _ client: IMKTextInput) {
        let ms = Date().timeIntervalSince(started) * 1000
        if ms > 100 {
            DiagnosticLog.note(String(format: "a key took %.0f ms in %@", ms, client.bundleIdentifier() ?? "?"))
        }
    }

    override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        appID = (sender as? IMKTextInput)?.bundleIdentifier()
        Self.focused = self
        // Which mode's input source is selected (spec §8.2): macOS's answer, unless a selection of
        // ours is still in flight. setValue(_:forTag:client:) also reports it, but not always
        // before the first key.
        let mode = InputModes.selected() ?? InputModes.current
        Consent.askSoon()
        DiagnosticLog.note("activated in \(appID ?? "?"): selected \(InputModes.selectedNow()?.letter ?? "another input source"), current \(InputModes.current.letter), using \(mode.letter)")
        #if PREDICT
        clientApp = (sender as? IMKTextInput)?.bundleIdentifier()
        DefaultOffApps.noticeIfFirst(clientApp)
        #endif
        jk_activate(engine)
        applyMode(mode, client: sender as? IMKTextInput)
        #if PREDICT
        applyPredictionsSetting()
        // A new or changed training file (docs/training-contract.md): its weights and words now,
        // then settings again; its examples in the background (trainingApplied).
        Training.applyIfChanged()
        applyPredictionsSetting()
        #endif
        jk_set_enter_commits(engine, UserDefaults.standard.bool(forKey: enterCommitsDefault) ? 1 : 0)
        #if ENGLISH
        // A keyboard configured as JIS or a permission granted since launch: the key tap starts
        // now (at most one check every 30 s, after this call).
        DispatchQueue.main.async { KoreanKeyTap.checkSoon() }
        #endif
    }

    /// The system names the input mode the user picked: JIS Moonshot's modes are input modes.
    override func setValue(_ value: Any!, forTag tag: Int, client sender: Any!) {
        // Switching commits what's unconfirmed first (spec §8.2.1): applyMode does it.
        if tag == kTextServiceInputModePropertyTag, let id = value as? String, id == InputModes.kanaProxySourceID {
            // かな, through macOS's own routing: the proxy is never left selected.
            DiagnosticLog.note("かな (proxy)")
            kanaKey(client: sender as? IMKTextInput, reselect: true)
        } else if tag == kTextServiceInputModePropertyTag, let id = value as? String, let mode = InputModes.mode(sourceID: id) {
            applyMode(mode, client: sender as? IMKTextInput)
        }
        super.setValue(value, forTag: tag, client: sender)
    }

    private func applyMode(_ mode: InputModes.Mode, client: IMKTextInput?) {
        InputModes.becameCurrent(mode)
        setEngineMode(mode, client: client)
    }

    /// Puts this controller's engine and its client's keyboard layout on `mode`, committing what
    /// the old mode left unconfirmed.
    private func setEngineMode(_ mode: InputModes.Mode, client: IMKTextInput?) {
        if let modeID, modeID != mode.id {
            flush(client)
        }
        jk_set_mode(engine, mode.id)
        modeID = mode.id
        KeyboardViewerLayout.select(mode.keyboardViewerLayout, for: client)
    }

    /// The engine changed its own mode (F6 and F7 to JIS Kana, かな back to Mode A): the field and
    /// macOS follow, without committing (the engine kept what's staged).
    private func followEngineMode(_ client: IMKTextInput) {
        let id = jk_mode(engine)
        guard id != modeID, let mode = InputModes.mode(id: id) else { return }
        DiagnosticLog.note("engine switched to \(mode.letter)")
        modeID = id
        InputModes.becameCurrent(mode)
        KeyboardViewerLayout.select(mode.keyboardViewerLayout, for: client)
        selectLater(mode, client)
    }

    /// かな, カタカナ or the Korean Key (docs/moonshot.md §4), from the key tap or a key that reached
    /// the input method: from an English mode to Mode A; otherwise the engine's かな key (force
    /// Korean in Mode A, back to Mode A from JIS Kana).
    /// `reselect`: the かな proxy was selected (macOS's own かな), so Mode A is selected again
    /// whatever happens, never leaving the proxy selected.
    func kanaKey(client given: IMKTextInput? = nil, reselect: Bool = false, bare: Bool = false) {
        guard let client = given ?? (client() as IMKTextInput?) else { return }
        if InputModes.current.english || modeID.flatMap(InputModes.mode(id:))?.english == true {
            let target = InputModes.defaultMode
            applyMode(target, client: client)
            selectLater(target, client)
            return
        }
        #if PREDICT
        chosen = nil
        #endif
        // Shift+かな: the staged keys forced bare, with no implied ㅇ (docs/moonshot.md §4).
        if bare { jk_kana_key_bare(engine) } else { jk_kana_key(engine) }
        apply(to: client)
        if reselect || jk_mode(engine) == modeID {
            // Nothing changed mode, or the proxy is selected: Mode A again, shown under the caret.
            InputModes.becameCurrent(InputModes.defaultMode)
            selectLater(InputModes.defaultMode, client)
        }
        followEngineMode(client)
    }

    /// Keeps this field's engine, and its app's keyboard layout, on the mode JIS Moonshot is in. A
    /// switch can reach another controller than the one typing (カタカナ and F18 from the key tap,
    /// the public API, a selection macOS didn't report to this session); this field then catches up
    /// on its next key. Otherwise its engine stays in the old mode, and an English engine passes
    /// letters on to the Korean mode's layout, which types them as loose jamo (ㄷㅏㄹㅏ for 다락방).
    /// The layout is the app's, not the field's (KeyboardViewerLayout): another field of the app
    /// may have named its own mode's layout since, so it's checked on its own, engine or no engine.
    private func syncMode(_ client: IMKTextInput) {
        let mode = InputModes.current
        let app = client.bundleIdentifier()
        if modeID != mode.id {
            DiagnosticLog.note("mode resync \(modeID.map { String($0) } ?? "none") -> \(mode.letter) in \(app ?? "?")")
            setEngineMode(mode, client: client)
        } else if KeyboardViewerLayout.named(to: app) != mode.keyboardViewerLayout {
            DiagnosticLog.note("layout resync \(KeyboardViewerLayout.named(to: app) ?? "none") -> \(mode.keyboardViewerLayout) in \(app ?? "?")")
            KeyboardViewerLayout.select(mode.keyboardViewerLayout, for: client)
        }
    }

    override func deactivateServer(_ sender: Any!) {
        noteEarlyCommit("deactivateServer", sender)
        flush(sender)
        hideList()
        // macOS drops the layout named to the app when the input method is deactivated there.
        let app = (sender as? IMKTextInput)?.bundleIdentifier() ?? appID
        KeyboardViewerLayout.forget(app)
        DiagnosticLog.note("deactivated in \(app ?? "?")")
        super.deactivateServer(sender)
    }

    /// Called by the system on focus changes, mouse clicks in the text and similar.
    override func commitComposition(_ sender: Any!) {
        noteEarlyCommit("commitComposition", sender)
        flush(sender)
    }

    /// Logs a commit the app or the system asked for while something was unconfirmed: how many
    /// characters, never the text.
    private func noteEarlyCommit(_ what: String, _ sender: Any!) {
        let length = jk_preedit_len(engine)
        guard length > 0 else { return }
        let app = (sender as? IMKTextInput)?.bundleIdentifier() ?? appID ?? "?"
        DiagnosticLog.note("\(what) in \(app) with \(length) unconfirmed byte(s)")
    }

    override func menu() -> NSMenu! {
        let menu = NSMenu()
        guard Consent.accepted else {
            menu.addItem(NSMenuItem(title: "Accept Predictive Typing…", action: #selector(askConsent(_:)), keyEquivalent: ""))
            return menu
        }
        #if PREDICT
        let learning = NSMenuItem(title: "Learn from My Choices (Stays on This Mac)", action: #selector(toggleLearning(_:)), keyEquivalent: "")
        learning.state = UserDefaults.standard.bool(forKey: learningDefault) ? .on : .off
        menu.addItem(learning)
        // What was learned, read-only: one line per scenario that moved.
        let learned = NSMenuItem(title: "Learned Weights", action: nil, keyEquivalent: "")
        let submenu = NSMenu()
        // By weight id; 6 (hanja) is retired and not shown.
        let labels = ["Korean", "English", "Implied ㅇ", "English + particle", "Plain 2-set", "Finals kept", "", "Implied ㄹ after ㄴ", "Kana"]
        for (id, label) in labels.enumerated() where !label.isEmpty {
            let value = jk_learned_weight(engine, UInt32(id))
            let item = NSMenuItem(title: String(format: "%@  ×%.2f", label, value), action: nil, keyEquivalent: "")
            item.isEnabled = false
            submenu.addItem(item)
        }
        submenu.addItem(.separator())
        submenu.addItem(NSMenuItem(title: "Reset What It Learned", action: #selector(resetLearned(_:)), keyEquivalent: ""))
        submenu.addItem(NSMenuItem(title: "Open Training Folder", action: #selector(openTrainingFolder(_:)), keyEquivalent: ""))
        learned.submenu = submenu
        menu.addItem(learned)
        // Per app (spec §10.11): what this app does, and the list of excluded apps.
        menu.addItem(.separator())
        let app = clientApp ?? (client() as IMKTextInput?)?.bundleIdentifier()
        if let app {
            let name = DisabledApps.displayName(app)
            let entry = DisabledApps.entry(app)
            let learning = NSMenuItem(title: entry?.noLearning == true ? "Resume Learning from \(name)" : "Disable Learning from \(name)",
                                      action: #selector(toggleAppLearning(_:)), keyEquivalent: "")
            learning.representedObject = app
            menu.addItem(learning)
            let on = DefaultOffApps.predicting(app)
            let why = DefaultOffApps.reason(app).map { " (off by default: \($0.notice))" } ?? ""
            let predicting = NSMenuItem(title: on ? "Disable Predicting in \(name)" : "Resume Predicting in \(name)\(why)",
                                        action: #selector(toggleAppPredictions(_:)), keyEquivalent: "")
            predicting.representedObject = app
            menu.addItem(predicting)
        }
        menu.addItem(NSMenuItem(title: "Manage Disabled Apps…", action: #selector(manageDisabledApps(_:)), keyEquivalent: ""))
        #endif
        let item = NSMenuItem(title: "Enter Commits Without a New Line", action: #selector(toggleEnterCommits(_:)), keyEquivalent: "")
        item.state = UserDefaults.standard.bool(forKey: enterCommitsDefault) ? .on : .off
        menu.addItem(item)
        #if ENGLISH
        // The Korean Key (spec §8.2.1); any other key code can be set with
        // `defaults write me.sjun.inputmethod.JISMoonshot KoreanKey -int <code>`.
        let switching = NSMenuItem(title: "Korean Key (Same as かな)", action: nil, keyEquivalent: "")
        let keys = NSMenu()
        registerSwitchDefaults()
        let current = UserDefaults.standard.integer(forKey: switchKeyDefault)
        for (title, code) in [("None", -1), ("F13", kVK_F13), ("F14", kVK_F14), ("F15", kVK_F15), ("F16", kVK_F16),
                              ("F17", kVK_F17), ("F18", kVK_F18), ("F19", kVK_F19)] {
            let key = NSMenuItem(title: title, action: #selector(setSwitchKey(_:)), keyEquivalent: "")
            key.tag = code
            key.state = code == current ? .on : .off
            keys.addItem(key)
        }
        switching.submenu = keys
        menu.addItem(switching)
        // 英数, かな and the Korean Key everywhere (iTerm, other input sources) need Accessibility.
        if !AXIsProcessTrusted() {
            menu.addItem(NSMenuItem(title: "Allow 英数 and かな Everywhere…", action: #selector(askForKeyPermissions(_:)), keyEquivalent: ""))
        }
        #endif
        // The public API's one setting (spec §8.2): reading the mode is always open, changing it isn't.
        let allow = NSMenuItem(title: "Let Other Apps Change the Mode", action: #selector(toggleAllowModeChange(_:)), keyEquivalent: "")
        allow.state = PublicAPI.allowed ? .on : .off
        menu.addItem(allow)
        return menu
    }

    #if ENGLISH
    @objc func askForKeyPermissions(_ sender: Any?) {
        KoreanKeyTap.start(ask: true)
    }

    @objc func setSwitchKey(_ sender: Any?) {
        guard let code = (sender as? NSMenuItem)?.tag else { return }
        UserDefaults.standard.set(code, forKey: switchKeyDefault)
    }

    /// 英数, かな and the Korean Key (docs/moonshot.md §4), when they reach the input method (without
    /// the key tap, or in apps that pass them on): 英数 always goes to English (`eisuTarget`), かな and
    /// the Korean Key to Mode A, or force Korean there (`kanaKey`). Nil: not a switch key here, or
    /// the mode to switch to isn't added in System Settings (then the key goes on as usual).
    private func handleSwitchKey(_ event: NSEvent, _ client: IMKTextInput) -> Bool? {
        let mods = modifiers(event.modifierFlags)
        guard !InputModes.english.isEmpty, mods & UInt32(JK_MOD_CTRL | JK_MOD_ALT | JK_MOD_META) == 0
        else { return nil }
        registerSwitchDefaults()
        let defaults = UserDefaults.standard
        let code = Int(event.keyCode)
        let target: InputModes.Mode?
        if code == defaults.integer(forKey: switchKeyDefault) || code == kVK_JIS_Kana {
            // かな (and the Korean Key): docs/moonshot.md §4.
            guard InputModes.kanaTarget != nil else { return nil }
            if !event.isARepeat { kanaKey(bare: mods & UInt32(JK_MOD_SHIFT) != 0) }
            return true
        } else if code == kVK_JIS_Eisu {
            target = InputModes.eisuTarget
            // Already there: nothing to do, and ABC isn't wanted.
            if let target, target.sourceID == InputModes.current.sourceID { return true }
        } else {
            return nil
        }
        guard let target, InputModes.isEnabled(target) else { return nil }
        if !event.isARepeat {
            // The field being typed in switches now, in place: its engine and keyboard layout. Asking
            // macOS first left it on the old mode (the open session isn't always told) and cost a
            // round trip; macOS is told afterwards, for the menu bar and the next field.
            flush(client)
            applyMode(target, client: client)
            selectLater(target, client)
        }
        return true
    }

    /// A key caught by the key tap (KoreanKeyTap, spec §8.2.1: the Korean Key, カタカナ, 英数), in the
    /// field this controller types into: to `target`, in place. Already in it, the mode is selected
    /// again, so macOS shows it under the caret, and nothing unconfirmed is committed.
    func switchFromTap(to target: InputModes.Mode) {
        guard let client = client() as IMKTextInput? else { return }
        if InputModes.current.sourceID == target.sourceID && modeID == target.id {
            selectLater(target, client)
            return
        }
        // Another mode, or this field's engine didn't follow an earlier switch: in place.
        // applyMode commits what the old mode left unconfirmed.
        guard InputModes.isEnabled(target) else { return }
        applyMode(target, client: client)
        selectLater(target, client)
    }

    /// Selects `mode`'s input source once the key being handled is done: selecting it from inside
    /// handle(_:client:) can cut the open session off (Terminal then gets the keys itself).
    private func selectLater(_ mode: InputModes.Mode, _ client: IMKTextInput?) {
        InputModes.expect(mode)
        DispatchQueue.main.async { InputModes.select(mode, in: client) }
    }
    #endif

    @objc func askConsent(_ sender: Any?) {
        Consent.ask()
    }

    @objc func toggleAllowModeChange(_ sender: Any?) {
        UserDefaults.standard.set(!PublicAPI.allowed, forKey: PublicAPI.allowKey)
    }

    @objc func toggleEnterCommits(_ sender: Any?) {
        let on = !UserDefaults.standard.bool(forKey: enterCommitsDefault)
        UserDefaults.standard.set(on, forKey: enterCommitsDefault)
        jk_set_enter_commits(engine, on ? 1 : 0)
    }

    #if PREDICT
    @objc func toggleLearning(_ sender: Any?) {
        UserDefaults.standard.set(!UserDefaults.standard.bool(forKey: learningDefault), forKey: learningDefault)
        applyPredictionsSetting()
    }

    @objc func toggleAppLearning(_ sender: Any?) {
        guard let app = (sender as? NSMenuItem)?.representedObject as? String ?? clientApp else { return }
        DisabledApps.update(app) { $0.noLearning.toggle() }
        applyPredictionsSetting()
    }

    @objc func toggleAppPredictions(_ sender: Any?) {
        guard let app = (sender as? NSMenuItem)?.representedObject as? String ?? clientApp else { return }
        let defaultOff = DefaultOffApps.reason(app) != nil
        if DefaultOffApps.predicting(app) {
            // Turning off: back to the default where it starts off, else an explicit "off".
            DisabledApps.update(app) {
                $0.allowPredictions = false
                $0.noPredictions = !defaultOff
            }
        } else {
            DisabledApps.update(app) {
                $0.noPredictions = false
                $0.allowPredictions = defaultOff
            }
        }
        applyPredictionsSetting()
    }

    @objc func manageDisabledApps(_ sender: Any?) {
        DisabledAppsWindow.shared.show()
    }

    @objc func openTrainingFolder(_ sender: Any?) {
        Training.openFolder()
    }

    @objc func resetLearned(_ sender: Any?) {
        jk_learner_reset(engine)
        UserDefaults.standard.removeObject(forKey: learnedDefault)
    }

    /// The shared predictor, always on (docs/moonshot.md §6), except in apps excluded from
    /// predictions (spec §10.11.1), where Mode A types forced Korean. Also applies the user's
    /// tuning (spec §10.5), e.g. `defaults write me.sjun.inputmethod.JISMoonshot Weight.English -float 0.8`
    /// (`Weight.ImpliedL -float 0` turns the implied ㄹ after ㄴ off)
    /// (a weight set this way wins over the learned one), and what was learned (spec §10.11).
    private func applyPredictionsSetting() {
        let defaults = UserDefaults.standard
        // Per app (spec §10.11): no predictions, or no learning recorded, where the user said so.
        let app = DisabledApps.entry(clientApp)
        let predicting = DefaultOffApps.predicting(clientApp)
        let predictor = predicting ? Training.predictor() : nil
        jk_set_predictor(engine, predictor)
        self.predicting = predictor != nil
        jk_set_recording(engine, app?.noLearning == true ? 0 : 1)
        for (id, name) in weightNames.enumerated() {
            if let value = defaults.object(forKey: "Weight.\(name)") as? Double {
                jk_set_prediction_weight(engine, UInt32(id), value)
            } else {
                jk_clear_prediction_weight(engine, UInt32(id))
            }
        }
        // The latest of what every engine learned: each saves after a change, each loads here.
        if let learned = defaults.string(forKey: learnedDefault) {
            let bytes = Array(learned.utf8)
            _ = jk_learner_import(engine, bytes, bytes.count)
        } else {
            jk_learner_reset(engine)
        }
        jk_set_learning(engine, defaults.bool(forKey: learningDefault) ? 1 : 0)
    }

    /// A training file's examples were learned (Training, in the background): the field being typed
    /// in loads what they taught now; the others when they're activated.
    static func trainingApplied() {
        focused?.applyPredictionsSetting()
        if let keyed, keyed !== focused {
            keyed.applyPredictionsSetting()
        }
    }

    /// Stores what was learned when it changed: counts only, no text or keys.
    private func saveLearned() {
        guard jk_learner_changed(engine) != 0 else { return }
        let length = jk_learner_export(engine)
        UserDefaults.standard.set(text(jk_learner_ptr(engine), length), forKey: learnedDefault)
    }

    /// Keys for the suggestion list, while it shows predictions: ↓ / Tab and ↑ move through it
    /// (the chosen row replaces the marked text), Escape leaves it, Space / Enter commit the chosen
    /// row. Nil: not a list key, handle as usual.
    private func handleList(_ event: NSEvent, _ client: IMKTextInput) -> Bool? {
        guard predictions.count > 1, jk_candidate_count(engine) == 0 else { return nil }
        let mods = modifiers(event.modifierFlags)
        guard mods & UInt32(JK_MOD_CTRL | JK_MOD_ALT | JK_MOD_META) == 0 else { return nil }
        let count = predictions.count
        let shift = mods & UInt32(JK_MOD_SHIFT) != 0
        let key = Int(event.keyCode)
        if key == kVK_DownArrow || (key == kVK_Tab && !shift) {
            chosen = ((chosen ?? -1) + 1) % count
        } else if key == kVK_UpArrow || (key == kVK_Tab && shift) {
            chosen = ((chosen ?? 0) - 1 + count) % count
        } else if key == kVK_Escape && chosen != nil {
            chosen = nil
        } else if (key == kVK_Return || key == kVK_ANSI_KeypadEnter) && chosen != nil {
            commitPrediction(chosen ?? 0, to: client)
            return true
        } else {
            return nil
        }
        refresh(client)
        return true
    }

    private func commitPrediction(_ index: Int, to client: IMKTextInput) {
        jk_commit_prediction(engine, index)
        chosen = nil
        apply(to: client)
        saveLearned()
    }
    #endif

    private func flush(_ sender: Any!) {
        let client = (sender as? IMKTextInput) ?? (client() as IMKTextInput?)
        #if PREDICT
        if shown != nil, let client {
            // Committed because the system or the app asked: not a choice to learn from.
            jk_flush_prediction(engine, chosen ?? 0)
            chosen = nil
            apply(to: client)
            return
        }
        #endif
        jk_flush(engine)
        if let client {
            apply(to: client)
        }
    }

    /// Inserts the engine's commit text, then shows its preedit (or the chosen reading) as marked
    /// text, and updates the suggestion list.
    private func apply(to client: IMKTextInput) {
        let commit = text(jk_commit_ptr(engine), jk_commit_len(engine))
        if !commit.isEmpty {
            // Inserting replaces any marked text.
            client.insertText(commit, replacementRange: NSRange(location: NSNotFound, length: NSNotFound))
            hasMarkedText = false
        }
        #if PREDICT
        predictions = []
        japanese = []
        let preedit = text(jk_preedit_ptr(engine), jk_preedit_len(engine))
        if !preedit.isEmpty && jk_candidate_count(engine) == 0 {
            let count = jk_predict(engine)
            predictions = (0..<count).map { text(jk_prediction_ptr(engine, $0), jk_prediction_len(engine, $0)) }
            let japaneseKinds = [JK_PREDICTION_KANA, JK_PREDICTION_KATAKANA, JK_PREDICTION_KANJI].map { UInt32($0) }
            japanese = (0..<count).map { japaneseKinds.contains(jk_prediction_kind(engine, $0)) }
            // The top row shows unless it's the Korean reading as typed: a Korean row can differ
            // from it too (안녕하세요. for 안녕하세요을, 한·미·일 for 한?미?일, docs/moonshot.md §5.6).
            let topIsPreedit = count == 0 || predictions[0] == preedit
            if let chosen, chosen >= count { self.chosen = nil }
            topShown = topIsPreedit ? nil : predictions.first
        } else {
            chosen = nil
            topShown = nil
        }
        #endif
        refresh(client)
    }

    /// The marked text and the suggestion list, from the current state.
    private func refresh(_ client: IMKTextInput) {
        let notFound = NSRange(location: NSNotFound, length: NSNotFound)
        var preedit = text(jk_preedit_ptr(engine), jk_preedit_len(engine))
        #if PREDICT
        // The chosen row, else the top reading: Escape out of a row shows (and commits) the top
        // reading again, not the row left behind.
        shown = chosen.flatMap { $0 < predictions.count ? predictions[$0] : nil } ?? topShown
        if let shown {
            preedit = shown
        }
        #endif
        if !preedit.isEmpty {
            var attributes: [NSAttributedString.Key: Any] = [
                .underlineStyle: NSUnderlineStyle.single.rawValue,
                .markedClauseSegment: 0,
            ]
            #if PREDICT
            // A Japanese reading on screen: Hiragino Sans and the language tagged Japanese, for the
            // apps that draw marked text with its attributes (many use their own font).
            let index = chosen ?? 0
            if shown != nil, index < japanese.count, japanese[index] {
                attributes[.font] = SuggestionWindow.japaneseFont(size: NSFont.systemFontSize)
                attributes[NSAttributedString.Key(kCTLanguageAttributeName as String)] = "ja"
            }
            #endif
            let marked = NSAttributedString(string: preedit, attributes: attributes)
            let caret = NSRange(location: preedit.utf16.count, length: 0)
            client.setMarkedText(marked, selectionRange: caret, replacementRange: notFound)
            hasMarkedText = true
        } else if hasMarkedText {
            client.setMarkedText("", selectionRange: NSRange(location: 0, length: 0), replacementRange: notFound)
            hasMarkedText = false
        }
        showList(client, empty: preedit.isEmpty)
    }

    /// Which list was asked for last: a show scheduled for an earlier one is dropped.
    private var listGeneration = 0

    /// The engine's list (an English mode's letter variants), or the predictions while typing (when
    /// there's more than one).
    private func showList(_ client: IMKTextInput, empty: Bool) {
        let window = SuggestionWindow.shared
        listGeneration += 1
        let generation = listGeneration
        // The caret's rectangle is a synchronous round trip into the app, asked once the key is
        // done rather than inside it: inside, the app may not have the marked text yet (Chrome
        // hands it to the page's process after the key, and asks that process for the rectangle,
        // blocking), and the key would wait on it.
        func show(_ items: [String], selected: Int?, horizontal: Bool = false, japanese: [Bool] = []) {
            DispatchQueue.main.async { [weak self] in
                guard let self, generation == self.listGeneration else { return }
                var caret = NSRect.zero
                _ = client.attributes(forCharacterIndex: 0, lineHeightRectangle: &caret)
                window.show(items, selected: selected, below: caret, horizontal: horizontal, japanese: japanese)
            }
        }
        let count = jk_candidate_count(engine)
        if count > 0 {
            let items = (0..<count).map { i -> String in
                let candidate = text(jk_candidate_ptr(engine, i), jk_candidate_len(engine, i))
                let comment = text(jk_candidate_comment_ptr(engine, i), jk_candidate_comment_len(engine, i))
                return comment.isEmpty ? candidate : "\(candidate)  \(comment)"
            }
            let selected = jk_candidate_selected(engine)
            window.onPick = { [weak self] index in self?.pickCandidate(index, from: selected, client) }
            // An English mode's list is letter variants (spec §3.6.3): one row.
            show(items, selected: selected < count ? selected : nil, horizontal: InputModes.current.english)
            return
        }
        #if PREDICT
        if !empty && predictions.count > 1 {
            window.onPick = { [weak self] index in self?.commitPrediction(index, to: client) }
            show(predictions, selected: chosen, japanese: japanese)
            return
        }
        #endif
        window.hide()
    }

    /// Hides the list, dropping any show still scheduled.
    private func hideList() {
        listGeneration += 1
        SuggestionWindow.shared.hide()
    }

    /// A click on a row of the engine's list: move the engine's selection there, then pick it.
    private func pickCandidate(_ index: Int, from selected: Int, _ client: IMKTextInput) {
        let key = UInt32(index > selected ? JK_KEY_ARROWDOWN : JK_KEY_ARROWUP)
        for _ in 0..<abs(index - selected) {
            jk_key(engine, key, 0)
        }
        jk_key(engine, UInt32(JK_KEY_ENTER), 0)
        apply(to: client)
    }

    /// An edit of the app's own text the engine asked for (docs/moonshot.md §4.1): 半角/全角 with
    /// nothing staged. The engine says what to replace, from the selection and the text around it,
    /// which only the app has.
    private func convertHostText(_ client: IMKTextInput) {
        guard jk_convert_host(engine) != 0 else { return }
        let selection = client.selectedRange()
        guard selection.location != NSNotFound else { return }
        // Enough either side for any word; a longer one converts in part.
        let reach = 256
        let end = selection.location + selection.length
        let length = client.length()
        func read(_ range: NSRange) -> String {
            guard range.length > 0 else { return "" }
            var actual = NSRange(location: NSNotFound, length: 0)
            return client.string(from: range, actualRange: &actual) ?? ""
        }
        let start = max(0, selection.location - reach)
        let before = Array(read(NSRange(location: start, length: selection.location - start)).utf8)
        let selected = Array(read(selection).utf8)
        let after = Array(read(NSRange(location: end, length: max(0, min(reach, length - end)))).utf8)
        guard jk_host_edit(engine, before, before.count, selected, selected.count, after, after.count) != 0 else { return }
        let replacement = text(jk_resolved_ptr(engine), jk_resolved_len(engine))
        let back = jk_host_edit_before(engine), forward = jk_host_edit_after(engine)
        let range = NSRange(location: selection.location - back, length: back + selection.length + forward)
        client.insertText(replacement, replacementRange: range)
    }

    private func text(_ pointer: UnsafePointer<UInt8>?, _ length: Int) -> String {
        guard let pointer, length > 0 else { return "" }
        return String(decoding: UnsafeBufferPointer(start: pointer, count: length), as: UTF8.self)
    }

    private func modifiers(_ flags: NSEvent.ModifierFlags) -> UInt32 {
        var bits: UInt32 = 0
        if flags.contains(.shift) { bits |= UInt32(JK_MOD_SHIFT) }
        if flags.contains(.control) { bits |= UInt32(JK_MOD_CTRL) }
        if flags.contains(.option) { bits |= UInt32(JK_MOD_ALT) }
        if flags.contains(.command) { bits |= UInt32(JK_MOD_META) }
        return bits
    }
}
