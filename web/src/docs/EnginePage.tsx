import { Code, PageHead, Source, Table } from "./Docs";

export function EnginePage() {
  return (
    <>
      <PageHead title="Engine" folder="engine/">
        The core of JIS Moonshot, in Rust, forked from JIS Korean's. Every platform runs this same engine: it decides what
        each key types, composes Hangul and kana, commits text, and ranks the Korean, English and kana readings. The
        platforms only move key events in and text out.
      </PageHead>

      <h2>What it is</h2>
      <ul>
        <li>
          <strong>Event in, actions out.</strong> A key press goes in as a physical key (its <code>KeyboardEvent.code</code>{" "}
          name) and modifiers; out come the text to commit, the unconfirmed text to show (the preedit), and whether the key
          was used. No callbacks, no global state, no I/O, no OS APIs. A shell keeps one engine per text field.
        </li>
        <li>
          <strong>Deterministic and crash-free.</strong> The same keys always give the same text, and no input makes it
          panic. On Windows it runs inside every app's process, so the C ABI also catches any unwinding.
        </li>
        <li>
          <strong>Two builds.</strong> The full build (the default: the <code>predict</code> and <code>english</code>{" "}
          features) ranks predictions with the learned ranker; the legacy build (<code>--features legacy-predict</code>)
          ranks by the rules alone. <code>--no-default-features</code> still builds and passes its tests, but no shell
          ships it: predictions are always on. The <code>english</code> feature is the English modes (JIS English, JIS
          Dvorak and JIS Colemak), which type English and compose nothing; 英数 needs them.
        </li>
        <li>
          <strong>Targets.</strong> A static library for macOS (arm64 and x86_64) and Windows (x64 and x86, linked into the
          input method's DLL), and WebAssembly for the web playground.
        </li>
        <li>
          <strong>The docs decide.</strong> What changed from JIS Korean is in <Source path="docs/moonshot.md" />, which
          wins over <Source path="docs/spec.md" /> where they disagree; the spec defines the rest. Every example in either
          is a test.
        </li>
      </ul>

      <h2>Core logic</h2>
      <p>Each key press goes through the same steps:</p>
      <ol>
        <li>
          <strong>Key map</strong> (<Source path="engine/src/layout.rs" />): the mode's table says what the key types, as a
          Hangul jamo or as a character, and every key also has its JIS kana legend (
          <Source path="engine/src/kana.rs" />). The 26 letter keys always type standard 2-set. JIS keyboards only: Mode A
          (JIS Moonshot), JIS Kana and the English modes; JIS Korean's Modes B and C are retired.
        </li>
        <li>
          <strong>Segment</strong> (<Source path="engine/src/engine.rs" />): the key joins the unconfirmed segment as a
          token, which keeps what it typed, what the same key types on the plain keyboard underneath (its English
          reading) and its kana. The keys are kept, not their text, so a forced script re-reads the same keystrokes.
        </li>
        <li>
          <strong>Composition</strong> (<Source path="engine/src/hangul.rs" />): the segment's jamo are composed into
          syllables by the 2-set automaton, with JIS Korean's additions: the implied ㅇ (ㅏ alone types 아), Shift+vowel
          starting a syllable on ㅇ, and the double press. Kana compose too: ゛ and ゜ attach to the kana before them
          (か゛ → が, は゜ → ぱ).
        </li>
        <li>
          <strong>Commit</strong>: Space, Enter, shortcuts, focus loss and switching input source commit the segment
          (spec §5). Space with kana showing types a full-width space (U+3000), and ⌥Space (an engine setting the macOS
          shell turns on) commits and types one in Mode A and JIS Kana, <code>_</code> in the English modes.
        </li>
        <li>
          <strong>Scripts and function keys</strong>: F6 and F7 show the staged keys as hiragana and katakana until the
          next commit (in JIS Kana, for good); the かな key (<code>jk_kana_key</code>) forces their Korean. F8 (and Shift+F8,
          the same key: there are no hanja) puts the kana reading's kanji conversions on top, F9 and F10 convert
          the word to full- and half-width letters, cycling the case. Shift+Z is a key like any other: ㅋ, っ or Z.
        </li>
        <li>
          <strong>Predictions</strong> (full build, <Source path="engine/src/predict.rs" />): the same keys are read several
          ways (Korean, English, plain 2-set, finals kept, English + particle) and ranked by rules over key-trigram
          models, word lists and the user's weights, then corrected by a learned model (<Source path="engine/src/learned.rs" />,
          spec §10.13; not in the legacy build). The kana reading (hiragana, or katakana for a known katakana word) is
          scored against the Japanese lexicon (<Source path="engine/predict/common-ja.txt" />) and merged in by score
          (docs/moonshot.md §5). At most 4 readings are offered, plus the kana one. Learning from the user's choices is
          opt-in and stays on the device.
        </li>
      </ol>

      <h2>Rust API</h2>
      <Code lang="rust">{`
use jis_korean::{Engine, Key, Mode, Mods};

let mut engine = Engine::new();            // Mode A, no predictor
for code in ["KeyG", "KeyK", "KeyS"] {
    let out = engine.key(Key::from_code(code), Mods::NONE);
    // out.commit: text to insert; out.preedit: text to show as marked;
    // out.consumed: false = pass the key on to the app
}
assert_eq!(engine.preedit(), "한");
let out = engine.flush();                  // focus loss: commits "한"
engine.set_mode(Mode::AKana);              // JIS Kana: kana typing
`}</Code>
      <Table
        code
        head={["Item", "What it does"]}
        rows={[
          ["Engine::key(key, mods) -> Output", "One key press. Output: commit, preedit, consumed, the candidate list (the English modes' ⌥ + letter variants), and the English reading."],
          ["flush() / clear()", "Commit everything unconfirmed / discard it."],
          ["activate()", "The input method was switched back to (nothing to reset since the F8 escape was retired)."],
          ["set_mode(Mode) / mode()", "Mode A (JIS Moonshot), JIS Kana (Mode::AKana) or an English mode. Mode::ALL lists them, with id(), code() and name(); Mode::from_id(id) is None for an id the build doesn't have, retired ids (1, 2, 9, 10) included. The engine changes its own mode (かな from JIS Kana): read mode() after a key."],
          ["script()", "How the staged keys are shown: Auto (the top prediction), Korean (forced), Hiragana or Katakana."],
          ["set_enter_commits(bool)", "Plain Enter commits without a new line, as Shift+Enter does."],
          ["set_option_space(bool)", "⌥Space: a full-width space (_ in the English modes). Off by default: on Windows Alt+Space is the window menu."],
          ["set_predictor(Option<Arc<Predictor>>)", "Turns predictions on (full build). predictions() ranks; commit_prediction(i) commits one."],
          ["key_after_prediction(i, key, mods) / flush_prediction(i)", "A key that commits while prediction i is shown (Space, Enter, a symbol): commits what's shown, then the key acts on it. flush_prediction commits it quietly, for focus loss."],
          ["preedit() / english()", "The unconfirmed text, and the same keys read on the plain keyboard."],
          ["output_in(mode, layout, key, shift)", "What a key types in a mode, for drawing legends."],
          ["resolve::*", "Resolving keys and text (below)."],
        ]}
      />

      <h2>C ABI</h2>
      <p>
        The shells use the C ABI in <Source path="engine/include/jis_korean.h" />. Conventions:
      </p>
      <ul>
        <li>
          An engine call stores its result in the engine handle; read it with the matching <code>_ptr</code> and{" "}
          <code>_len</code> functions until the next call. Strings are UTF-8 and not NUL-terminated.
        </li>
        <li>
          Keys are passed as stable ids (<code>JK_KEY_*</code>), one per <code>KeyboardEvent.code</code>. New keys are only
          appended, before <code>JK_KEY_MODIFIER</code>; a test fails if the header and the engine disagree.
        </li>
        <li>One handle per text field, used from one thread. Predictors are shared, reference-counted.</li>
        <li>Wasm hosts pass data in through <code>jk_alloc</code> / <code>jk_dealloc</code>.</li>
      </ul>
      <Table
        head={["Group", "Functions"]}
        rows={[
          ["Lifecycle", <code>jk_new, jk_free</code>],
          ["Keys", <code>jk_key, jk_kana_key, jk_flush, jk_clear, jk_activate, jk_set_enter_commits, jk_set_option_space, jk_key_commits, jk_is_escaped (always 0)</code>],
          ["Results", <code>jk_commit_*, jk_preedit_*, jk_english_*, jk_kana_*, jk_candidate_*</code>],
          ["Modes", <code>jk_set_mode, jk_mode, jk_script, jk_mode_count, jk_mode_id_at, jk_mode_name_*, jk_mode_code_*, jk_mode_is_english, jk_legend, jk_legend_in</code>],
          ["Keys by name", <code>jk_key_count, jk_key_code_ptr, jk_key_code_len</code>],
          ["Resolving", <code>jk_resolve_key_input, jk_resolve_key_input_after, jk_resolve_keys_for_output, jk_resolved_*</code>],
          ["Predictions", <code>jk_predictor_new, jk_predictor_with_learned, jk_predictor_with_japanese, jk_predictor_free, jk_set_predictor, jk_predict, jk_prediction_*, jk_commit_prediction, jk_key_after_prediction, jk_flush_prediction, jk_set_prediction_weight, jk_clear_prediction_weight, jk_set_ranker</code>],
          ["Learning", <code>jk_set_learning, jk_set_recording, jk_learned_weight, jk_learner_*, jk_learn_example, jk_predictor_with_words, jk_predictor_with_avoided, jk_predictor_with_avoided_japanese</code>],
          ["Key counts", <code>jk_plan, jk_plan_jis, jk_plan_conventional, jk_plan_chars, jk_plan_off_keyboard, jk_plan_dropped</code>],
        ]}
      />

      <h2>Resolving keys and text</h2>
      <p>
        Two questions, answered on a copy of a live engine so the answers match it exactly: its mode, and with predictions
        on, its predictor, weights and what it learned (<Source path="engine/src/resolve.rs" />, spec §8.3). A key press is
        written as its <code>KeyboardEvent.code</code>, with <code>S-</code> for Shift: <code>KeyG</code>,{" "}
        <code>S-Digit2</code>. One press is one key, with or without Shift.
      </p>
      <Table
        code
        head={["Function", "Returns", "Example"]}
        rows={[
          ["key_input(engine, press)", "The character the key types on its own, or none (F-keys, Space).", <code>KeyG → ㅎ</code>],
          ["key_input_after(engine, context, press)", "What the field shows after the unconfirmed context and the key, including anything it commits.", <code>하 + KeyS → 한</code>],
          [
            "keys_for_output(engine, text, min, max)",
            "Every combination of presses that types the text, min to max presses, fewest presses then fewest Shifts first, at most 1,000. Never an error: a nonsensical range gives an empty list, and max is cut to 16.",
            <code>ㅃ → [Digit1], [S-KeyQ]</code>,
          ],
          ["type_keys(engine, presses)", "What the field shows after typing the presses from an empty field.", <code>[KeyG, KeyK, KeyS] → 한</code>],
        ]}
      />
      <p>
        Keys apply as a shell applies them: with predictions on, the marked text shows the top prediction unless it's the
        Korean reading, and Space and Enter commit what's shown. The search tries only keys whose characters are part of
        the text, skips presses that change nothing, and follows routes heading toward the text: its jamo, its letters
        before F9/F10. Every combination is checked by typing it.
      </p>

      <h2>Performance</h2>
      <h3>Time</h3>
      <Table
        head={["Operation", "Cost", "Why"]}
        rows={[
          ["A key press (no predictor)", <>O(<i>n</i>), <i>n</i> = keys staged</>, "The segment is recomposed for the preedit; a segment is one word, usually under 20 keys."],
          ["A key press with predictions", <>O(<i>n</i> · <i>k</i>), <i>k</i> ≤ 8</>, "Each reading is composed (a handful), plus the English + particle splits over the last 8 keys; model and word-list lookups are hash lookups, O(1) each."],
          ["key_input", "O(1)", "One table lookup."],
          ["key_input_after, keys_for_output", "Bounded", "A search over key presses, capped at 50,000 engine steps and 16 presses: it always ends, and returns what it found."],
        ]}
      />
      <h3>Measured</h3>
      <p>
        Measured on JIS Korean v0.3.1, before the kana reading was added; not yet re-measured for JIS Moonshot.{" "}
        Native release build on an Apple M6, typing <Source path="web/public/long-sample-text.txt" /> word by word (4,313 key
        presses, Korean and English), with predictions ranked after every press as a shell does (by the rules: the
        benchmark doesn't load the learned ranker). Run it yourself with{" "}
        <code>cargo run --release --example bench [lite|full|resolve]</code>.
      </p>
      <Table
        head={["Measurement", "No predictor", "With predictions"]}
        rows={[
          ["Key press, mean", "0.2 µs", "3.6 µs"],
          ["Key press, median", "0.2 µs", "2.7 µs"],
          ["Key press, 99th percentile", "0.7 µs", "16.2 µs"],
          ["Key press, slowest", "1.3 µs", "54 µs"],
          ["Load the predictor", "–", "8–11 ms"],
        ]}
      />
      <Table
        head={["Resolving (with predictions)", "Result", "Time"]}
        rows={[
          [<code>keys_for_output("한", 3, 3)</code>, "8 combinations", "0.3 ms"],
          [<code>keys_for_output("hello", 5, 5)</code>, "1 combination", "3.6 ms"],
          [<code>key_input_after("대한민국의", KeyS)</code>, "대한민국읜", "0.2 ms"],
          [<code>keys_for_output("한글", 1, 8)</code>, "844 combinations", "91 ms"],
        ]}
      />
      <h3>Memory</h3>
      <Table
        head={["Part", "Size", "Notes"]}
        rows={[
          ["Engine per text field", "Small", "The segment (one word of tokens), settings and learned weights. Engines share the predictor."],
          ["Predictor", "1.3 MB on disk; about 8 MB in memory", "The key-trigram model (39 KB) and three word lists. The learned ranker adds learned.bin (2.1 MB)."],
          ["Wasm binary", "564 KB (176 KB gzipped)", "The full engine with predictions and resolving, without data files."],
        ]}
      />

      <h2>Tests</h2>
      <p>
        <code>cargo test</code> in <code>engine/</code>; <code>cargo test --features legacy-predict</code> for the legacy
        build and <code>cargo test --no-default-features</code> for the engine without predictions or English.
        Golden tables cover every automaton transition, Backspace, each F-key and every spec example; a robot types real
        text key by key and checks the screen after every character; <code>tests/modes.rs</code> checks every mode against
        the mode contract (<Source path="docs/modes.md" />), and <code>tests/moonshot.rs</code> the fork's own behavior
        (kana typing, scripts, the switch keys, the kana reading). <code>tests/baseline.rs</code> is the core principle as a
        test: every syllable, typed the conventional 2-set way, in every mode. CI runs the baseline and the full, legacy and
        no-default-features tests on every pull request and branch push.
      </p>
    </>
  );
}
