import Cocoa

/// Predictive typing must be accepted before JIS Moonshot types anything (docs/moonshot.md §6).
/// Until it is, every key passes through untouched, and the input menu offers "Accept Predictive
/// Typing…". Stored as `PredictiveAccepted`: the version of the notice accepted.
enum Consent {
    /// Bump when the notice changes in substance: everyone is asked again.
    static let version = 1
    private static let key = "PredictiveAccepted"
    /// Asked once per launch on its own; the input menu can ask again.
    private static var askedThisLaunch = false
    private static var asking = false

    static var accepted: Bool {
        UserDefaults.standard.integer(forKey: key) >= version
    }

    /// Asks once per launch, after the current event (an alert inside a key would hold the app).
    static func askSoon() {
        guard !accepted, !askedThisLaunch else { return }
        askedThisLaunch = true
        DispatchQueue.main.async { ask() }
    }

    static let notice = """
        JIS Moonshot reads every key you type as Korean, English and Japanese kana at once, and \
        ranks those readings to show the likeliest one. That is predictive typing, and JIS \
        Moonshot doesn't work without it.

        • What it reads: the keys of the text you haven't committed yet, to rank its readings.
        • Where: only on this Mac. Nothing is sent anywhere.
        • Learning from your choices is separate and off until you turn it on in the input menu.
        • Terminals, password managers and similar apps start with predictions off (Disable / \
        Resume Predicting in the input menu). There, keys type forced Korean.

        Until you accept, JIS Moonshot passes every key to the app untouched.
        """

    static func ask() {
        guard !accepted, !asking else { return }
        asking = true
        defer { asking = false }
        let alert = NSAlert()
        alert.messageText = "Accept predictive typing?"
        alert.informativeText = notice
        alert.addButton(withTitle: "Accept")
        alert.addButton(withTitle: "Not Now")
        NSApp.activate(ignoringOtherApps: true)
        if alert.runModal() == .alertFirstButtonReturn {
            UserDefaults.standard.set(version, forKey: key)
            DiagnosticLog.note("predictive typing accepted (notice \(version))")
        } else {
            DiagnosticLog.note("predictive typing not accepted yet")
        }
    }
}
