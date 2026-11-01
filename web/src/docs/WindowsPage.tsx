import { Code, PageHead, Source, Table } from "./Docs";

export function WindowsPage() {
  return (
    <>
      <PageHead title="Windows" folder="windows/">
        A Text Services Framework (TSF) text input processor, <code>jismoonshot.dll</code>, around the engine. It maps scan
        codes to engine keys and shows what the engine says as a TSF composition, the hanja and prediction list, and the
        taskbar's 한글 / かな / ABC indicator. It's behind the macOS shell.
      </PageHead>

      <h2>What it is</h2>
      <ul>
        <li>
          <strong>A thin shell</strong>, like the macOS input method: scan codes in, the engine's commit and composition
          text out, through the same C ABI (<Source path="engine/include/jis_korean.h" />).
        </li>
        <li>
          <strong>Inside every app's process</strong>, as every TSF input processor is. That's why the engine never panics
          and its C ABI catches any unwinding: a crash would take the app down with it. The DLL links the static C runtime,
          and embeds the hanja dictionary and the prediction model, the kana reading's lexicon included, parsed once per
          process.
        </li>
        <li>
          <strong>x64 and x86:</strong> 32-bit apps load the x86 DLL. The engine is built with its <code>english</code>{" "}
          feature, for ABC's English layouts.
        </li>
        <li>
          <strong>Beside JIS Korean:</strong> its own CLSID and profile, <code>Program Files\JIS Moonshot</code>,{" "}
          <code>HKCU\Software\JIS Moonshot</code> and <code>%APPDATA%\JIS Moonshot\</code>.
        </li>
      </ul>

      <h2>Build and install</h2>
      <p>
        Windows 10 or 11, Visual Studio 2022 Build Tools (C++ x64/x86), CMake 3.28+, Ninja, and Rust with the{" "}
        <code>x86_64-pc-windows-msvc</code> and <code>i686-pc-windows-msvc</code> targets. There is no lite build.
      </p>
      <Code lang="powershell">{`
cd windows
tools/build.ps1               # x64 + x86, runs the shell tests (-Legacy: ranked by the rules alone)
tools/package.ps1             # build/dist/JISMoonshot-<version>-setup.exe, the installer users download
tools/install.ps1             # development install, admin (UAC): Program Files\\JIS Moonshot,
                              # registers x64 + x86, adds JIS Moonshot to the Korean language's keyboards
tools/install.ps1 -Update     # copy a new build in, no UAC (development installs only)
tools/install.ps1 -Uninstall
tools/install.ps1 -Log        # turn the diagnostic log on and follow it
`}</Code>
      <p>
        The installer shows the predictive-typing notice as a page that must be accepted, like a licence. Then pick{" "}
        <strong>JIS Moonshot</strong> with Win+Space, and restart apps that were open: an app keeps the old DLL, or none,
        until it restarts.
      </p>

      <h2>Using it</h2>
      <Table
        head={["Feature", "How it works"]}
        rows={[
          [
            "Accept first",
            "Until the notice is accepted (in Setup, or the indicator's Accept Predictive Typing…), every key passes through untouched.",
          ],
          [
            "Mode A, JIS Kana",
            "Mode A (JIS Moonshot) reads Korean 2-set keys as Korean, English and kana at once. JIS Kana types kana on the JIS kana layout; it's in the indicator's menu. One setting for every app.",
          ],
          ["無変換", "Commits what's shown, then English (ABC, in the English layout). In ABC it stays there."],
          [
            "変換",
            "From ABC: back to Mode A. In Mode A: the staged keys forced Korean (nothing staged: nothing). In JIS Kana: back to Mode A, the staged keys forced Korean.",
          ],
          ["한/영", "Toggles ABC and 한글, as the Microsoft IME does, committing first. JIS keyboards have no 한/영 key; 無変換 and 変換 do its job."],
          ["F6 / F7", "Hiragana / katakana until the next commit; in JIS Kana they switch its script for good."],
          ["F8", "Kanji: the kana reading's conversions on top until the next commit (nothing staged: the next segment). In JIS Kana, the conversions above the kana."],
          ["Shift+F8", "The hanja list, numbered 1–9 per page."],
          ["F9 / F10", "The word's keys as full- / half-width letters. Shift+Z types ㅋ (っ in kana), not F10."],
          [
            "English Layout",
            "What ABC types: JIS English (the default), JIS Dvorak, JIS Colemak, or Windows' own layout.",
          ],
          [
            "Predictions",
            "Always on. The unconfirmed text shows the top reading. ↓ / Tab and ↑ / Shift+Tab move through the list, Space / Enter commit what's shown, Esc goes back to the top reading. Excluded by default in terminals, remote desktop, password managers, File Explorer and launchers: there, Mode A types forced Korean.",
          ],
        ]}
      />
      <p>
        <strong>Type English and kana in Mode A.</strong> 無変換 and 한/영 are an on/off switch and nothing more: English
        typed in ABC gets no predictions, F9/F10 or hanja, and switching mid-word splits the word. The recommended way is to
        stay in Mode A, let the predictions read English and kana words, and train them on your own words with a training
        file (<Source path="docs/training-contract.md" />) at <code>%APPDATA%\JIS Moonshot\training.json</code>.
      </p>

      <h2>Settings</h2>
      <p>
        In the indicator's menu, stored in <code>HKCU\Software\JIS Moonshot</code>:
      </p>
      <Table
        code
        head={["Value", "Default", "What"]}
        rows={[
          ["PredictiveAccepted", "unset", "The version of the notice accepted; unset, every key passes through"],
          ["Mode", "0", "0 Mode A (JIS Moonshot), 11 JIS Kana; a retired mode (1, 2, 9, 10) is Mode A"],
          ["EnglishMode", "3", "ABC's layout: 3 JIS English, 7 JIS Dvorak, 8 JIS Colemak, 0xFFFFFFFF for Windows' layout"],
          ["EnterCommits", "0", "Enter commits without a new line"],
          ["LearnFromChoices", "0", "Learning from your choices, stays on this PC"],
          ["Weight.<Scenario>", "", "Tuning: Korean, English, Implied, Mixed, TwoSet, KeepFinals, Hanja, ImpliedL, Kana"],
          ["Apps\\<program.exe>\\Predictions", "", "Predictions in one app (a safety exclusion: there is no global switch)"],
          ["Apps\\<program.exe>\\Off", "", "1: every key passes through in that program"],
          ["DiagnosticLog", "0", "Activations, focus and errors, never typed text, to %ProgramData%\\JIS Moonshot\\jismoonshot.log"],
        ]}
      />

      <h2>Passwords and screen recording</h2>
      <ul>
        <li>The list never appears in screen recordings, screenshots or screen sharing.</li>
        <li>Password boxes that say they are one get no input method at all.</li>
        <li>
          Password boxes that don't (common in banking and certificate programs) compose like text: use ABC there, or the
          menu's <strong>Off in &lt;program&gt;</strong>.
        </li>
      </ul>

      <h2>Known limits</h2>
      <ul>
        <li>Password fields and the secure desktop (UAC, sign-in) don't use input processors.</li>
        <li>Apps that read raw keys (games) see Windows' own layout.</li>
        <li>Browsers take F10 for their menu, and Shift+Z no longer stands in for it.</li>
        <li>
          Apps that end the composition at every key through the IMM layer still get loose jamo; TSF-native ones keep
          composing.
        </li>
      </ul>
      <p>
        More in <Source path="windows/README.md" />, and why the shell does what it does with TSF in{" "}
        <Source path="docs/troubleshooting/windows-tsf-notes.md" />.
      </p>
    </>
  );
}
