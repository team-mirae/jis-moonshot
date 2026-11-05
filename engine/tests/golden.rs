//! Golden tests: key sequences → document text and preedit. Every example in docs/spec.md is here.
//!
//! Keys are canonical codes separated by spaces. `S-` holds Shift and `C-` holds Ctrl.

use jis_korean::{Engine, Key, Mods};

/// A host document: committed text, plus what passed-through keys do in a plain text field.
struct Host {
    engine: Engine,
    doc: String,
    preedit: String,
}

impl Host {
    fn new() -> Self {
        Host { engine: Engine::new(), doc: String::new(), preedit: String::new() }
    }

    fn keys(&mut self, keys: &str) -> &mut Self {
        for token in keys.split_whitespace() {
            let mut mods = Mods::NONE;
            let mut code = token;
            loop {
                if let Some(rest) = code.strip_prefix("S-") {
                    mods.shift = true;
                    code = rest;
                } else if let Some(rest) = code.strip_prefix("C-") {
                    mods.ctrl = true;
                    code = rest;
                } else {
                    break;
                }
            }
            let key = Key::from_code(code);
            assert!(key != Key::Other || code == "Other" || code == "ArrowLeft", "unknown key {code}");
            let out = self.engine.key(key, mods);
            self.doc.push_str(&out.commit);
            self.preedit = out.preedit;
            if !out.consumed && !mods.is_shortcut() {
                match key {
                    Key::Enter => self.doc.push('\n'),
                    Key::Backspace => {
                        self.doc.pop();
                    }
                    _ => {}
                }
            }
        }
        self
    }

    fn flush(&mut self) -> &mut Self {
        let out = self.engine.flush();
        self.doc.push_str(&out.commit);
        self.preedit = out.preedit;
        self
    }
}

/// Types `keys` and checks the preedit; nothing may be committed.
#[track_caller]
fn pre(keys: &str, expected: &str) {
    let mut h = Host::new();
    h.keys(keys);
    assert_eq!((h.doc.as_str(), h.preedit.as_str()), ("", expected), "keys: {keys}");
}

/// Types `keys` and checks the document and the preedit.
#[track_caller]
fn doc(keys: &str, expected_doc: &str, expected_preedit: &str) {
    let mut h = Host::new();
    h.keys(keys);
    assert_eq!((h.doc.as_str(), h.preedit.as_str()), (expected_doc, expected_preedit), "keys: {keys}");
}

// §1 Core principle

#[test]
fn key_1_types_ssangbieup_and_shift_q_ba() {
    // docs/moonshot.md §9: the doubles are on 1–5; Shift on Q types 바 like Shift on any consonant.
    pre("Digit1 KeyK", "빠");
    pre("S-KeyQ", "바");
}

#[test]
fn o_then_a_and_key_0_both_type_wa() {
    pre("KeyR KeyH KeyK", "과");
    pre("KeyR Digit0", "과");
}

#[test]
fn double_finals_compose() {
    pre("KeyE KeyK KeyF KeyR", "닭");
    pre("KeyD KeyJ KeyQ KeyT", "없");
}

#[test]
fn shift_on_a_consonant_types_it_with_a() {
    // docs/moonshot.md §9.2: Shift+consonant is the consonant and ㅏ, as if both keys were pressed.
    for letter in ["KeyQ", "KeyW", "KeyE", "KeyR", "KeyT", "KeyA", "KeyS", "KeyD", "KeyF", "KeyG", "KeyZ", "KeyX", "KeyC", "KeyV"] {
        let mut a = Host::new();
        let mut b = Host::new();
        a.keys(&format!("KeyG {letter} KeyK KeyS"));
        b.keys(&format!("KeyG S-{letter} KeyS"));
        assert_eq!(a.preedit, b.preedit, "{letter}");
    }
    pre("S-KeyR S-KeyR", "가가");
    pre("KeyG KeyK S-KeyS", "하나");
    // A final before it moves, as before any ㅏ: 갑 + Shift+T → 갑사.
    pre("KeyR KeyK KeyQ S-KeyT", "갑사");
    // Backspace takes the key off whole.
    pre("KeyG S-KeyS Backspace", "ㅎ");
}

#[test]
fn compound_vowel_keys_match_sequences() {
    pre("KeyR Digit6", "귀");
    pre("KeyR KeyN KeyL", "귀");
    pre("KeyR Digit7", "궈");
    pre("KeyR KeyN KeyJ", "궈");
    pre("KeyR Digit8", "괴");
    pre("KeyR KeyH KeyL", "괴");
    pre("KeyR Digit9", "긔");
    pre("KeyR KeyM KeyL", "긔");
}

// §3 Key map

#[test]
fn shift_number_row_types_digits() {
    pre("S-Digit1 S-Digit2 S-Digit3 S-Digit4 S-Digit5 S-Digit6 S-Digit7 S-Digit8 S-Digit9 S-Digit0", "1234567890");
}

#[test]
fn phone_numbers_type_with_shift_held() {
    pre("S-Digit0 S-Digit1 S-Digit0 S-Digit1 S-Digit2", "01012");
}

#[test]
fn the_keys_right_of_0_type_their_actions() {
    // docs/moonshot.md §9.2: each action as if its 2-set keys were pressed, real ㅇ included.
    pre("Minus S-Minus", "왜왜");
    pre("KeyE Minus", "돼");
    pre("KeyA KeyK KeyF Minus", "마뢔");
    pre("KeyA KeyK KeyF S-Minus", "말왜");
    pre("KeyR Semicolon", "계");
    pre("KeyA KeyK KeyF S-Semicolon", "말예");
    pre("KeyR Quote", "걔");
    pre("S-Quote KeyR KeyL", "얘기");
    pre("KeyG Comma", "희");
    // Korean keeps , on Shift.
    pre("S-Comma", ",");
}

#[test]
fn the_symbol_keys_type_their_jis_symbols() {
    // docs/moonshot.md §9.2 (2026-11-05): ^ ¥ @ [ ] . / ろ type their stock JIS symbols, alone and
    // with Shift, and end the syllable before them. Shift+`.` is a period (2026-11-06), as in JIS Kana.
    pre("Equal S-Equal IntlYen S-IntlYen BracketLeft S-BracketLeft", "^~¥|@`");
    pre("BracketRight S-BracketRight Backslash S-Backslash", "[{]}");
    pre("Period S-Period Slash S-Slash IntlRo S-IntlRo", "../?\\_");
    pre("KeyD KeyK KeyS KeyS KeyU KeyD KeyG KeyK KeyT KeyP KeyD KeyY Period", "안녕하세요.");
    pre("KeyA KeyN KeyJ KeyG KeyO KeyD KeyY S-Slash", "뭐해요?");
    pre("KeyG KeyK KeyS Period KeyR KeyK", "한.가");
}

#[test]
fn f10_reads_every_key_of_an_action_once() {
    // An action's parts belong to its key: Shift+; (예) converts to one +, Shift+− (왜) to one =.
    pre("KeyI S-KeyP KeyH KeyO KeyN KeyE F10", "iPhone");
    pre("KeyR Minus F10", "r-");
    pre("KeyR S-Semicolon F10", "r+");
    pre("KeyR S-Minus F10", "r=");
}

#[test]
fn backspace_takes_a_whole_key_off() {
    pre("KeyA KeyK KeyF S-Minus Backspace", "말");
    pre("KeyA KeyK KeyF S-Semicolon Backspace", "말");
    pre("KeyW KeyL KeyQ Period Backspace", "집");
}

#[test]
fn hankaku_zenkaku_converts_every_symbol_with_a_form_in_that_width() {
    pre("S-Comma S-Comma Backquote", "，，");
    // ¥ (the ¥ key) has a full-width form too.
    pre("S-Comma IntlYen S-Comma Backquote", "，￥，");
    pre("S-Comma IntlYen S-Comma Backquote S-Backquote", ",¥,");
    // A character with no other form stays: 한 has none.
    pre("KeyG KeyK KeyS S-Comma Backquote", "한，");
}

#[test]
fn shift_ignored_on_vowels_without_shifted_forms() {
    pre("KeyR S-KeyK", "가");
}

#[test]
fn shift_vowel_starts_on_ieung_after_a_final() {
    // No Shift is "still held" over (docs/moonshot.md §9.3): Shift on a consonant types its ㅏ.
    pre("KeyD KeyL Digit5 S-KeyJ", "있어");
    pre("KeyD KeyL Digit5 KeyJ", "이써");
    pre("KeyD KeyL S-KeyT S-KeyJ", "이사어");
    // A vowel that's part of a key's action isn't a press: the same vowel next is no double press.
    pre("KeyR KeyK S-KeyS KeyK", "가나아");
}

// §4.1 Automaton

#[test]
fn final_moves_to_next_syllable() {
    pre("KeyG KeyK KeyS KeyK", "하나");
    pre("KeyE KeyK KeyF KeyR KeyK", "달가");
}

#[test]
fn double_final_splits_before_consonant_that_cannot_join() {
    pre("KeyD KeyK KeyS KeyW KeyD KeyK", "앉아");
    pre("KeyD KeyK KeyS KeyW KeyK", "안자");
    pre("KeyD KeyJ KeyQ KeyT KeyE KeyK", "없다");
}

#[test]
fn tense_consonants_that_cannot_be_finals_start_a_syllable() {
    pre("KeyR KeyK Digit3", "가ㄸ");
    pre("KeyR KeyK Digit3 KeyK", "가따");
    pre("KeyR KeyK Digit1 KeyK", "가빠");
}

#[test]
fn lone_consonants() {
    pre("KeyR", "ㄱ");
    pre("KeyR KeyS", "ㄱㄴ");
}

#[test]
fn dedicated_compound_vowel_does_not_combine_further() {
    pre("KeyR Digit0 KeyL", "과이");
}

// §4.2 Implied ㅇ

#[test]
fn vowel_with_no_initial_gets_implied_ieung() {
    pre("KeyK", "아");
    pre("KeyH KeyK", "와");
    pre("KeyK KeyS", "안");
    pre("Minus KeyK", "왜아");
}

#[test]
fn vowel_that_cannot_join_gets_implied_ieung() {
    pre("KeyR KeyK KeyJ", "가어");
    pre("KeyR KeyH Digit0", "고와");
    pre("KeyR KeyK Digit8", "가외");
}

#[test]
fn implied_ieung_moves_like_a_real_initial() {
    pre("KeyK KeyS KeyK", "아나");
}

// §4.2 Same vowel again keeps the final

#[test]
fn same_vowel_again_puts_the_final_back() {
    pre("KeyA KeyK KeyF KeyL", "마리");
    pre("KeyA KeyK KeyF KeyL KeyL", "말이");
    pre("KeyE KeyK KeyF KeyR KeyK KeyK", "닭아");
    pre("KeyR KeyK KeyD KeyK KeyK", "강아");
    pre("KeyA KeyK KeyF Digit0 Digit0", "말와");
}

#[test]
fn final_toggle_needs_the_same_vowel_right_away() {
    pre("KeyA KeyK KeyF KeyL KeyK", "마리아");
    pre("KeyA KeyK KeyF KeyH KeyK", "마롸");
    doc("KeyA KeyK KeyF KeyL S-Enter KeyL", "마리", "이");
    pre("KeyA KeyK KeyF KeyL KeyL KeyL", "말이이");
}

#[test]
fn final_toggle_then_commit_and_backspace() {
    // Committing keeps the double-press ㅇ as it is.
    doc("KeyA KeyK KeyF KeyL KeyL S-Enter", "말이", "");
    pre("KeyA KeyK KeyF KeyL KeyL Backspace", "마리");
}

// §4.3 Backspace

#[test]
fn backspace_removes_one_jamo_keystroke() {
    pre("KeyG KeyK KeyS Backspace", "하");
    pre("KeyG KeyK KeyS Backspace Backspace", "ㅎ");
    pre("KeyR KeyH KeyK Backspace", "고");
    pre("KeyR Digit0 Backspace", "ㄱ");
    pre("KeyR KeyK KeyJ Backspace", "가");
}

#[test]
fn backspace_with_nothing_unconfirmed_passes_through() {
    doc("KeyR KeyK S-Space Backspace Backspace", "", "");
}

// §5 Commit

#[test]
fn shift_space_commits_and_types_a_space() {
    // docs/moonshot.md §4 (2026-11-03): Shift+Space commits; plain Space stages a space.
    doc("KeyG KeyK KeyS S-Space", "한 ", "");
}

#[test]
fn enter_commits_then_passes_through() {
    doc("KeyG KeyK KeyS Enter", "한\n", "");
    doc("Enter", "\n", "");
}

#[test]
fn shift_enter_commits_only() {
    doc("KeyG KeyK KeyS S-Enter", "한", "");
}

#[test]
fn enter_commits_setting() {
    let mut h = Host::new();
    h.engine.set_enter_commits(true);
    h.keys("KeyG KeyK KeyS Enter");
    assert_eq!(h.doc, "한");
}

#[test]
fn shortcut_commits_and_passes_through() {
    doc("KeyG KeyK KeyS C-KeyC", "한", "");
}

#[test]
fn other_keys_commit_and_pass_through() {
    doc("KeyG KeyK KeyS ArrowLeft", "한", "");
}

#[test]
fn modifier_alone_changes_nothing() {
    pre("KeyG KeyK KeyS ShiftLeft", "한");
}

#[test]
fn flush_commits_everything() {
    let mut h = Host::new();
    h.keys("KeyG KeyK KeyS S-Comma").flush();
    assert_eq!((h.doc.as_str(), h.preedit.as_str()), ("한,", ""));
}

// §6 Function keys

#[test]
fn function_keys_with_nothing_unconfirmed_go_to_the_app() {
    // F6, F7 and F8 (Shift+F8 too: the same) are taken even then: kana or kanji for the next
    // segment (tests/moonshot.rs, tests/kanji.rs).
    for key in [Key::F9, Key::F10] {
        assert!(!Engine::new().key(key, Mods::NONE).consumed, "{key:?}");
    }
    assert!(Engine::new().key(Key::F8, Mods::SHIFTED).consumed);
    // Shift+Z is ㅋ, a key like any other: taken.
    assert!(Engine::new().key(Key::KeyZ, Mods::SHIFTED).consumed);
    // With something unconfirmed they act and are taken.
    let mut e = Engine::new();
    e.key(Key::KeyR, Mods::NONE);
    let out = e.key(Key::F10, Mods::NONE);
    assert!(out.consumed);
    assert_eq!((out.commit.as_str(), out.preedit.as_str()), ("", "r"));
}

// §6 F8: kanji conversions (docs/moonshot.md §4); Shift+F8 is the same

#[test]
fn f8_changes_no_keys() {
    // F8 asks for kanji predictions (none without a predictor): the keys around it type as ever.
    pre("KeyG KeyK F8 KeyS", "한");
    pre("F8 Digit1 S-Digit2", "ㅃ2");
    // Shift+F8 is F8: no hanja list any more.
    pre("KeyG KeyK S-F8 Digit1", "하ㅃ");
    let mut e = Engine::new();
    assert!(e.key(Key::F8, Mods::NONE).consumed);
    e.key(Key::KeyG, Mods::NONE);
    assert!(e.key(Key::F8, Mods::NONE).consumed);
    assert!(e.key(Key::F8, Mods::SHIFTED).consumed);
    assert!(!e.is_escaped());
}

// §6.2 F9 한글 / F10 ABC (docs/moonshot.md §4)

#[test]
fn f10_cycles_case_and_f9_goes_back_to_korean() {
    let mut h = Host::new();
    let mut step = |keys: &str, expected: &str| {
        h.keys(keys);
        assert_eq!(h.preedit, expected, "after {keys}");
    };
    step("KeyG KeyK KeyS", "한");
    step("F10", "gks");
    step("F10", "GKS");
    step("F10", "Gks");
    // F9 is the forced Korean reading (as 変換): it undoes F10. Nothing commits.
    step("F9", "한");
    assert_eq!(h.doc, "");
}

#[test]
fn f10_then_hankaku_zenkaku_is_full_width_letters() {
    // Full-width letters are 半角/全角 on F10's letters (F9 no longer converts).
    pre("KeyG KeyK KeyS F10 F10 Backquote", "ＧＫＳ");
}

#[test]
fn shift_f9_forces_the_bare_reading() {
    // ㅏ alone shows 아; Shift+F9 drops the implied ㅇ, as Shift+変換 does; F9 brings it back.
    pre("KeyK S-F9", "ㅏ");
    pre("KeyR KeyK KeyJ S-F9", "가ㅓ");
    pre("KeyK S-F9 F9", "아");
    // Typing goes on in the forced script, until the commit.
    pre("KeyK S-F9 KeyJ", "ㅏㅓ");
}

#[test]
fn f6_to_f10_never_commit() {
    for key in ["F6", "F7", "F8", "F9", "S-F9", "F10"] {
        doc(&format!("KeyG KeyK KeyS {key}"), "", match key {
            "F6" => "きのと",
            "F7" => "キノト",
            "F10" => "gks",
            _ => "한",
        });
    }
}

#[test]
fn space_stages_a_space() {
    // docs/moonshot.md §4: the space is staged, the words stay unconfirmed; Shift+Space commits all.
    doc("KeyG KeyK KeyS Space KeyR KeyM KeyF", "", "한 글");
    doc("KeyG KeyK KeyS Space KeyR KeyM KeyF S-Space", "한 글 ", "");
    // Shift+Enter commits with no side effect; Enter commits and goes on to the app.
    doc("KeyG KeyK KeyS Space KeyR KeyM KeyF S-Enter", "한 글", "");
    // Backspace takes the staged space off; F10 reads the whole staged text.
    pre("KeyG KeyK KeyS Space Backspace KeyK", "하나");
    pre("KeyG KeyK KeyS Space KeyR KeyM KeyF F10", "gks rmf");
    // With nothing staged it's the app's.
    let mut e = Engine::new();
    assert!(!e.key(Key::Space, Mods::NONE).consumed);
    // It doesn't commit what shows: hosts send it to jk_key, not jk_key_after_prediction.
    assert!(!e.commits(Key::Space, Mods::NONE));
    assert!(e.commits(Key::Space, Mods::SHIFTED));
}

#[test]
fn conversion_uses_raw_keys() {
    pre("S-KeyQ KeyK F10", "Qk");
    pre("Digit1 KeyK F10", "1k");
    pre("KeyK F10", "k");
}

#[test]
fn conversion_takes_the_whole_staged_text() {
    pre("KeyG KeyK KeyS S-Comma KeyG KeyK KeyS F10", "gks,gks");
    doc("KeyG KeyK S-Enter KeyG KeyK F10", "하", "gk");
}

#[test]
fn conversion_with_no_word_is_a_no_op() {
    pre("F10", "");
    doc("KeyG KeyK KeyS S-Enter F10", "한", "");
}

#[test]
fn input_after_conversion_ends_it() {
    pre("KeyG KeyK KeyS F10 KeyK", "gks아");
    doc("KeyG KeyK KeyS F10 KeyR S-Enter", "gksㄱ", "");
}

#[test]
fn space_commits_converted_text() {
    doc("KeyG KeyK KeyS F10 S-Space", "gks ", "");
}

// English reading and clear (for hosts that rank Korean against English)

#[test]
fn english_reads_the_same_keys_on_stock_jis() {
    let mut e = Engine::new();
    let mut last = None;
    for (code, shift) in [("KeyH", false), ("KeyE", false), ("KeyL", false), ("KeyL", false), ("KeyO", false), ("Digit1", true)] {
        last = Some(e.key(Key::from_code(code), if shift { Mods::SHIFTED } else { Mods::NONE }));
    }
    let out = last.unwrap();
    // h e l → 오디, then the second l is the same vowel right after a move: 옫이 (double press).
    assert_eq!(out.preedit, "옫이애1");
    assert_eq!(out.english, "hello!");
    assert_eq!(e.english(), "hello!");
}

#[test]
fn english_keeps_case_dedicated_digits_and_symbols() {
    let mut e = Engine::new();
    for (code, shift) in [("KeyQ", true), ("Digit1", false), ("Minus", false), ("Slash", false), ("IntlYen", false)] {
        e.key(Key::from_code(code), if shift { Mods::SHIFTED } else { Mods::NONE });
    }
    // Shift+Q is 바, 1 ㅃ, − ㅙ, / and ¥ themselves (docs/moonshot.md §9); English reads the keys.
    assert_eq!(e.preedit(), "바뽸/¥");
    assert_eq!(e.english(), "Q1-/¥");
}

#[test]
fn clear_discards_without_committing() {
    let mut e = Engine::new();
    e.key(Key::from_code("KeyG"), Mods::NONE);
    e.key(Key::from_code("KeyK"), Mods::NONE);
    e.clear();
    assert_eq!((e.preedit().as_str(), e.english().as_str()), ("", ""));
    assert_eq!(e.flush().commit, "");
}

// §4.2 Shift+vowel starts a syllable on ㅇ

#[test]
fn shift_vowel_starts_a_syllable_and_leaves_the_final() {
    pre("KeyA KeyK KeyF S-KeyL", "말이");
    pre("KeyR KeyK KeyS S-KeyK", "간아");
    pre("KeyE KeyK KeyF KeyR S-KeyK", "닭아");
    pre("KeyR KeyK KeyS S-KeyJ KeyS", "간언");
}

#[test]
fn shift_vowel_does_not_combine_with_the_vowel_before() {
    pre("KeyH S-KeyL", "오이");
    pre("KeyH KeyL", "외");
    pre("KeyR KeyN S-KeyJ", "구어");
}

#[test]
fn shift_vowel_can_still_combine_with_the_vowel_after() {
    pre("KeyR KeyK KeyS S-KeyH KeyK", "간와");
}

#[test]
fn shift_vowel_on_its_own_is_like_the_implied_ieung() {
    pre("S-KeyK", "아");
    pre("Minus S-KeyK", "왜아");
}

#[test]
fn backspace_removes_the_shift_vowel_in_one_press() {
    pre("KeyA KeyK KeyF S-KeyL Backspace", "말");
}

#[test]
fn shift_on_o_and_p_starts_a_syllable_on_ieung() {
    // Shift+O / Shift+P are Shift+vowel like the rest (spec §4.2); ㅒ ㅖ are on : and ;.
    pre("KeyR S-KeyO", "개");
    pre("KeyR S-KeyO KeyS KeyK", "개나");
    pre("KeyR KeyK KeyS S-KeyP", "간에");
    pre("KeyA KeyK KeyF S-KeyO", "말애");
    pre("KeyR KeyK KeyS Quote", "가냬");
}

#[test]
fn shift_on_consonants_adds_a() {
    pre("KeyR KeyK S-KeyS", "가나");
    pre("KeyR KeyK S-KeyR", "가가");
}

#[test]
fn english_reading_keeps_the_shifted_vowel_letter() {
    let mut e = Engine::new();
    for (code, shift) in [("KeyA", false), ("KeyK", false), ("KeyF", false), ("KeyL", true)] {
        e.key(Key::from_code(code), if shift { Mods::SHIFTED } else { Mods::NONE });
    }
    assert_eq!((e.preedit().as_str(), e.english().as_str()), ("말이", "akfL"));
}

#[test]
fn shift_vowel_after_shift_consonant_starts_on_ieung() {
    // Shift+consonant already has its ㅏ (docs/moonshot.md §9.2), so a Shift+vowel after it starts
    // its own syllable on ㅇ.
    pre("S-KeyR S-KeyK", "가아");
    pre("Digit4 S-KeyK", "까");
    pre("KeyR S-KeyK", "가");
}

// Shift+Z is ㅋ (docs/moonshot.md §1: no longer F10)

#[test]
fn shift_z_types_ka() {
    // Shift+Z is 카 like Shift on any consonant (docs/moonshot.md §9.2), not F10.
    pre("KeyZ KeyZ", "ㅋㅋ");
    pre("S-KeyZ", "카");
    pre("KeyG KeyK KeyS S-KeyZ", "한카");
    pre("KeyG KeyK KeyS S-KeyZ F10", "gksZ");
}

#[test]
fn plain_z_still_types_kieuk() {
    pre("KeyZ KeyZ", "ㅋㅋ");
    pre("KeyZ KeyK", "카");
}

#[test]
fn the_engine_says_which_keys_commit_what_stands() {
    let e = Engine::new();
    for key in [Key::Enter, Key::Escape, Key::ArrowDown, Key::ArrowLeft, Key::Other, Key::IntlBackslash] {
        assert!(e.commits(key, Mods::NONE), "{key:?}");
    }
    // Shift+Space commits; Space stages a space (docs/moonshot.md §4).
    assert!(e.commits(Key::Space, Mods::SHIFTED));
    assert!(!e.commits(Key::Space, Mods::NONE));
    assert!(e.commits(Key::KeyG, Mods { ctrl: true, ..Mods::NONE }));
    for key in [Key::KeyG, Key::Digit1, Key::Minus, Key::Backspace, Key::F6, Key::F10, Key::Modifier] {
        assert!(!e.commits(key, Mods::NONE), "{key:?}");
    }
    // 半角/全角 takes the reading on screen, to convert it (docs/moonshot.md §4.1).
    assert!(e.commits(Key::Backquote, Mods::NONE));
    // Shift+Z is ㅋ, not a commit.
    assert!(!e.commits(Key::KeyZ, Mods::SHIFTED));
}
