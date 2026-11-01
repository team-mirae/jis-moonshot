import { Code, PageHead, Source, Table } from "./Docs";

export function MacPage() {
  return (
    <>
      <PageHead title="macOS" folder="mac/">
        An InputMethodKit input method, in Swift, around the engine. It turns macOS key events into engine keys and
        shows what the engine says: inserted text, marked text, hanja candidates and the suggestion list. macOS is JIS
        Moonshot's first platform.
      </PageHead>

      <h2>What it is</h2>
      <ul>
        <li>
          <strong>A thin shell.</strong> One controller per text field, each with its own engine. The controller moves key
          codes in and text out; it never decides what a key types.
        </li>
        <li>
          <strong>Built without Xcode.</strong> <code>make</code> builds the engine as a universal static library (arm64 and
          x86_64), compiles the Swift sources against its C header, and assembles <code>JISMoonshot.app</code>. It needs
          the Command Line Tools and Rust with the <code>aarch64-apple-darwin</code> and <code>x86_64-apple-darwin</code>{" "}
          targets. There is one build: predictions, the English modes and JIS Kana are always in.{" "}
          <code>make LEGACY_PREDICT=1</code> ranks predictions by the rules alone, without the learned ranker.
        </li>
        <li>
          <strong>Beside JIS Korean.</strong> The bundle id is <code>me.sjun.inputmethod.JISMoonshot</code> and its data is in{" "}
          <code>~/Library/Application Support/JIS Moonshot/</code>, so it installs beside JIS Korean without colliding.
        </li>
        <li>
          <strong>Signed with your certificate</strong> if the keychain has an Apple Development one, else ad hoc. The
          Accessibility permission the switch keys need follows the signature: with a certificate it survives rebuilds,
          and ad hoc every build has to be granted it again.
        </li>
        <li>
          <strong>Installed per user</strong> in <code>~/Library/Input Methods</code>. <code>make install</code> swaps the
          bundle in with renames and re-enables only the input sources that were enabled: a missing bundle, even briefly,
          leaves running apps unable to start it (<Source path="docs/troubleshooting/macos-input-method-not-launching.md">troubleshooting</Source>).
        </li>
      </ul>
      <Code lang="sh">{`
cd mac
make install             # build, install, register, add the login agent
make LEGACY_PREDICT=1    # predictions without the learned ranker
make dist                # build/<variant>/JISMoonshot-<version>[-legacy].zip
`}</Code>

      <h2>Modes are input sources</h2>
      <p>
        Each engine mode is its own input mode of the one input method, the way Japanese has Kana and Romaji. A first{" "}
        <code>make install</code> enables JIS Moonshot, JIS Kana and JIS English; add the others in System Settings →
        Keyboard → Input Sources → Edit → +.
      </p>
      <Table
        head={["Mode", "Input source ID", "Shown as"]}
        rows={[
          ["A", <code>me.sjun.inputmethod.JISMoonshot.2Set</code>, "JIS Moonshot (Korean, English and kana readings), under Korean"],
          ["A_KANA", <code>me.sjun.inputmethod.JISMoonshot.A_KANA</code>, "JIS Kana (kana typing)"],
          ["A_QWERTY", <code>me.sjun.inputmethod.JISMoonshot.A_QWERTY</code>, "JIS English, under English"],
          ["A_DVORAK, A_COLEMAK", <code>….A_DVORAK, ….A_COLEMAK</code>, "JIS Dvorak, JIS Colemak"],
        ]}
      />
      <p>
        The English modes type English with no composition. JIS English types JIS symbols on any keyboard, and ⌥ + a
        letter offers its accented variants in a numbered row. After the first install of a new mode, log out and back in
        if System Settings doesn't list it.
      </p>
      <p>
        <strong>Why input sources, and not a setting:</strong> the mode follows the keyboard, and switching apps doesn't
        change which keyboard you're typing on, so a per-app mode would mean nothing. As input sources, the modes are
        visible to everything that understands input sources: the input menu, System Settings, and tools such as
        Hammerspoon and Karabiner, which read and select them by ID.
      </p>

      <h2>Keys</h2>
      <Table
        head={["Key", "Does"]}
        rows={[
          ["英数", "To the last English mode (JIS English by default), committing what shows."],
          ["かな (and カタカナ, F18)", "To Mode A. In Mode A with something staged: force Korean. From JIS Kana: Mode A, the staged keys forced Korean."],
          ["F6 / F7", "Hiragana / katakana: the staged keys read as kana until the next commit, then predictive again. With nothing staged, the next segment is kana. In JIS Kana they switch its script for good."],
          ["F8", "Kanji: the kana reading's conversions on top until the next commit (nothing staged: the next segment). In JIS Kana, the conversions above the kana."],
          ["Shift+F8", "The hanja list (Mode A)."],
          ["F9 / F10", "Full-width / half-width letters, as in JIS Korean."],
          ["⌥Space", "A full-width space (U+3000) in JIS Moonshot and JIS Kana, committing what's typed first; _ in the English modes."],
          ["Shift+Z", "ㅋ, っ or Z, like any other key."],
        ]}
      />
      <ul>
        <li>
          <strong>No permission needed for かな.</strong> macOS's own かな selects an input mode of JIS Moonshot that
          reroutes itself to Mode A, and macOS turns カタカナ into かな, so they do the same thing. The{" "}
          <strong>Korean Key</strong> (F18 by default, F13–F19 or none in the input menu) also acts as かな, for a key
          remapped to it.
        </li>
        <li>
          <strong>Everywhere, with Accessibility.</strong> To catch 英数 and かな before apps such as iTerm that keep
          F-keys, and from other input sources, JIS Moonshot runs a system-wide key tap. It never asks at launch: choose{" "}
          <strong>Allow 英数 and かな Everywhere…</strong> in the input menu. The tap lives in JIS Moonshot's process, so{" "}
          <code>make install</code> also installs a login agent that starts it in the background.
        </li>
        <li>
          <strong>ABC – Extended.</strong> With Apple's ABC enabled, macOS has been seen sending 英数 to ABC instead of JIS
          English. If it still does with the tap, add ABC – Extended and remove ABC (JIS Korean{" "}
          <a href="https://github.com/team-mirae/jis-korean/issues/2" target="_blank" rel="noreferrer">
            #2
          </a>
          , <Source path="docs/troubleshooting/macos-katakana-and-eisu.md">troubleshooting</Source>).
        </li>
      </ul>
      <p>
        A Mac JIS keyboard has 英数 and かな, which is all JIS Moonshot needs. For other keyboards,{" "}
        <a href="https://github.com/themrsung/keyboard-troubleshooting" target="_blank" rel="noreferrer">
          keyboard-troubleshooting
        </a>{" "}
        has remaps that give them JIS's keys (with Karabiner-Elements, keep the keyboard configured as JIS). Setting up
        with an AI assistant: <Source path="docs/interactive-prompts/README.md">docs/interactive-prompts</Source> walks you
        through it and ends with setting up a training file.
      </p>

      <h2>Predictive typing must be accepted</h2>
      <p>
        Predictions are always on. On the first activation JIS Moonshot shows an alert explaining predictive typing: what
        it reads (the keys of the text not yet committed, to rank their readings), that it stays on this Mac, that learning
        is separate and off, and that terminals and password managers start with predictions off. Until you choose{" "}
        <strong>Accept</strong>, every key passes through untouched and the input menu has only{" "}
        <strong>Accept Predictive Typing…</strong>. The answer is stored as <code>PredictiveAccepted</code>.
      </p>
      <p>
        The marked text shows the top reading, Korean, English or kana. ↓ and Tab move down the suggestion list, ↑ and
        Shift+Tab up, Enter commits the chosen row, and Escape goes back to the top reading. The list never appears in
        screen sharing, recordings or screenshots.
      </p>

      <h2>Settings</h2>
      <p>In the input menu while JIS Moonshot is selected:</p>
      <Table
        head={["Menu item", "Stored as", "Default"]}
        rows={[
          ["Learn from My Choices (Stays on This Mac)", <code>LearnFromChoices</code>, "Off"],
          ["Learned Weights → Reset What It Learned, Open Training Folder", <code>Learned</code>, "–"],
          ["Disable Predicting / Learning in this app, Manage Disabled Apps…", <code>DisabledApps</code>, "Some apps start with predictions off; there Mode A types forced Korean"],
          ["Enter Commits Without a New Line", <code>EnterCommits</code>, "Off"],
          ["Korean Key (Same as かな): F13–F19 or None", <code>KoreanKey</code>, "F18"],
          ["Let Other Apps Change the Mode", <code>AllowOtherAppsToChangeMode</code>, "Off"],
        ]}
      />
      <p>
        And with <code>defaults write me.sjun.inputmethod.JISMoonshot …</code>: <code>Weight.Kana</code>,{" "}
        <code>Weight.English</code>, <code>Weight.Korean</code> and the other prediction weights (a multiplier, 1.0 = as
        learned), <code>KoreanKey -int &lt;key code&gt;</code> for any key code as the Korean Key (-1 for none), and{" "}
        <code>DiagnosticLog -bool true</code> for <code>~/Library/Logs/JISMoonshot.log</code> (off by default; it never
        logs typed text). The training file is <code>~/Library/Application Support/JIS Moonshot/training.json</code> (
        <Source path="docs/training-contract.md" />). Problems and their diagnosis are in{" "}
        <Source path="docs/troubleshooting">docs/troubleshooting</Source>.
      </p>

      <h2>Public API</h2>
      <p>
        Any program can read the mode, hear it change, resolve keys and text, and, if you allowed it, change the mode. It
        works by distributed notifications, which every language can post and observe, and from the command line through
        the input method's own executable:
      </p>
      <Code lang="sh">{`
J=~/Library/"Input Methods"/JISMoonshot.app/Contents/MacOS/JISMoonshot
`}</Code>

      <h3>Read the mode</h3>
      <Code lang="sh">{`
defaults read me.sjun.inputmethod.JISMoonshot CurrentMode    # A, A_KANA, or an English mode (A_QWERTY…)
"$J" mode                                                    # A  JIS Moonshot  me.sjun…2Set
`}</Code>
      <p>
        Also <code>CurrentModeID</code>, <code>CurrentModeInputSourceID</code> and <code>CurrentModeName</code>.{" "}
        <code>CurrentMode</code> is the mode last used; whether JIS Moonshot is the selected input source right now is the
        system's current input source.
      </p>

      <h3>Notifications</h3>
      <Table
        code
        head={["Name (me.sjun.inputmethod.JISMoonshot.…)", "Direction", "User info"]}
        rows={[
          ["ModeDidChange", "Posted by JIS Moonshot", <code>mode, id, inputSourceID, name</code>],
          ["SetMode", "Posted to JIS Moonshot", <code>mode</code>],
          ["ModeChangeRefused", "Posted by JIS Moonshot", <code>reason, requested</code>],
          ["ResolveKeyCodeInput", "Posted to JIS Moonshot", <code>requestID, keyCode, context?</code>],
          ["ResolveKeyCodeInputResult", "Posted by JIS Moonshot", <code>requestID, result?, app, mode, predictions, error?</code>],
          ["ResolveKeyCodeFromOutput", "Posted to JIS Moonshot", <code>requestID, text, minKeypressCount, maxKeypressCount</code>],
          ["ResolveKeyCodeFromOutputResult", "Posted by JIS Moonshot", <code>requestID, combinations, app, mode, predictions, error?</code>],
        ]}
      />

      <h3>Change the mode</h3>
      <p>
        Only when <strong>Let Other Apps Change the Mode</strong> is on. <code>mode</code> is A, A_KANA, an English mode, an
        engine mode id or an input source ID. The answer is <code>ModeDidChange</code>, or <code>ModeChangeRefused</code>{" "}
        with the reason: the setting is off, the mode is unknown, or it isn't added in System Settings (the API never adds
        one). Selecting the input source directly, from the input menu or another tool, is macOS's own switching and
        always works.
      </p>
      <Code lang="sh">{`
"$J" mode A_KANA
`}</Code>

      <h3>Resolve keys and text</h3>
      <p>
        The engine's resolving functions (see the Engine docs), run on the engine of the app JIS Moonshot was last active
        in, so they match that app exactly: its mode, and its predictions, weights and what was learned. Every answer names
        the app, the mode and whether predictions were on. Key codes are <code>KeyboardEvent.code</code> names (
        <code>KeyG</code>) or macOS virtual key codes (<code>5</code>), with <code>S-</code> for Shift.
      </p>
      <Code lang="sh">{`
"$J" resolveKeyCodeInput KeyG           # {"result":"ㅎ","app":…,"mode":"A","predictions":true,…}
"$J" resolveKeyCodeInput KeyS 하        # after the context 하: "한"
"$J" resolveKeyCodeFromOutput ㅃ        # {"combinations":[["Digit1"],["S-KeyQ"]],…}
"$J" resolveKeyCodeFromOutput 한 3 3    # 3 presses; a nonsensical range gives []
`}</Code>

      <h3>From Hammerspoon</h3>
      <Code lang="lua">{`
local api = "me.sjun.inputmethod.JISMoonshot."
hs.distributednotifications.new(function(_, _, info)
  print("JIS Moonshot is in mode " .. info.mode)
end, api .. "ModeDidChange"):start()

hs.distributednotifications.post(api .. "SetMode", nil, { mode = "A_KANA" })
`}</Code>

      <h3>From Swift</h3>
      <Code lang="swift">{`
let center = DistributedNotificationCenter.default()
let id = UUID().uuidString
center.addObserver(forName: .init("me.sjun.inputmethod.JISMoonshot.ResolveKeyCodeFromOutputResult"),
                   object: nil, queue: .main) { note in
    guard note.userInfo?["requestID"] as? String == id else { return }
    print(note.userInfo?["combinations"] ?? [])
}
center.postNotificationName(.init("me.sjun.inputmethod.JISMoonshot.ResolveKeyCodeFromOutput"), object: nil,
    userInfo: ["requestID": id, "text": "한", "minKeypressCount": 3, "maxKeypressCount": 3],
    deliverImmediately: true)
`}</Code>
    </>
  );
}
