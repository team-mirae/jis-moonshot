//! Resolving keys and text (spec §8.3), on the deterministic engine. With predictions on, see
//! tests/predict.rs.

use jis_korean::resolve::{MAX_PRESSES, key_input, key_input_after, keys_for_output, type_keys};
use jis_korean::{Engine, Mode};

fn in_mode(mode: Mode) -> Engine {
    let mut e = Engine::new();
    e.set_mode(mode);
    e
}

fn has(found: &[Vec<String>], keys: &str) -> bool {
    found.iter().any(|route| route.join(" ") == keys)
}

#[test]
fn a_key_alone_types_its_character_in_the_mode() {
    let a = in_mode(Mode::A);
    assert_eq!(key_input(&a, "KeyG").as_deref(), Some("ㅎ"));
    // docs/moonshot.md §9: Shift+Q is 바 (Shift+consonant), and the keys right of 0 show their
    // actions; the double finals as the jamo they make, the break as ㆍ.
    assert_eq!(key_input(&a, "S-KeyQ").as_deref(), Some("바"));
    assert_eq!(key_input(&a, "S-Minus").as_deref(), Some("왜"));
    assert_eq!(key_input(&a, "BracketRight").as_deref(), Some("\u{318D}"));
    assert_eq!(key_input(&a, "BracketLeft").as_deref(), Some("ㅄ"));
    assert_eq!(key_input(&a, "S-Equal").as_deref(), Some("께서"));
    // The one empty slot (docs/moonshot.md §9.2) has no legend.
    assert_eq!(key_input(&a, "S-IntlYen").as_deref(), Some(""));
    // Shift+Z is no longer F10 (docs/moonshot.md §1): 카, like Shift on any consonant.
    assert_eq!(key_input(&a, "S-KeyZ").as_deref(), Some("카"));
    // Not characters: F-keys, Space, unknown keys.
    for press in ["F6", "Space", "Bogus", "", "S-"] {
        assert_eq!(key_input(&a, press), None, "{press}");
    }
}

#[test]
fn a_key_after_context_shows_the_whole_field() {
    let e = Engine::new();
    assert_eq!(key_input_after(&e, "하", "KeyS").as_deref(), Some("한"));
    assert_eq!(key_input_after(&e, "한", "KeyK").as_deref(), Some("하나"));
    assert_eq!(key_input_after(&e, "", "KeyK").as_deref(), Some("아")); // the implied ㅇ (spec §4.2)
    assert_eq!(key_input_after(&e, "한", "Space").as_deref(), Some("한 "));
    assert_eq!(key_input_after(&e, "한", "F10").as_deref(), Some("gks"));
    assert_eq!(key_input_after(&e, "한", "Nope"), None);
    assert_eq!(key_input_after(&in_mode(Mode::A), "한", "F8").as_deref(), Some("한")); // reserved
}

#[test]
fn keys_for_a_character() {
    let e = Engine::new();
    // ㅃ is only on 1 in JIS Moonshot (docs/moonshot.md §9).
    let found = keys_for_output(&e, "ㅃ", 1, 1);
    assert!(has(&found, "Digit1") && !has(&found, "S-KeyQ"), "{found:?}");
    assert!(found.iter().all(|route| route.len() == 1));
    // Fewest presses, then fewest Shifts, first.
    assert_eq!(found[0], ["Digit1"]);
}

#[test]
fn keys_for_a_word_within_the_range() {
    let e = Engine::new();
    let found = keys_for_output(&e, "한", 3, 3);
    assert!(has(&found, "KeyG KeyK KeyS"), "{found:?}");
    // Two presses with Shift+consonant (docs/moonshot.md §9.2): 하 is Shift+G.
    assert!(has(&keys_for_output(&e, "한", 2, 2), "S-KeyG KeyS"));
    assert!(keys_for_output(&e, "한", 1, 1).is_empty());
    // F10 after the Hangul keys converts to letters.
    assert!(has(&keys_for_output(&e, "gks", 4, 4), "KeyG KeyK KeyS F10"));
}

#[test]
fn keys_follow_the_mode() {
    // 왜 is Shift+− in Mode A (docs/moonshot.md §9), and , is Shift+,.
    assert!(has(&keys_for_output(&in_mode(Mode::A), "왜", 1, 1), "S-Minus"));
    assert!(has(&keys_for_output(&in_mode(Mode::A), ",", 1, 1), "S-Comma"));
}

#[test]
fn every_combination_types_the_text() {
    // , is Shift+, in Mode A. (Bare vowels like ㅠㅠ aren't here: F6 no longer drops the implied
    // ㅇ, docs/moonshot.md §4.)
    for (e, symbol) in [(in_mode(Mode::A), "가,")] {
        for text in ["까", "와", "읽", "한 ", "1", symbol] {
            let found = keys_for_output(&e, text, 1, 4);
            assert!(!found.is_empty(), "{text}");
            for route in found {
                assert!((1..=4).contains(&route.len()));
                assert_eq!(type_keys(&e, &route).as_deref(), Some(text), "{route:?}");
            }
        }
    }
}

#[test]
fn nonsensical_ranges_give_nothing() {
    let e = Engine::new();
    for (min, max) in [(0, 3), (-1, 3), (3, 2), (-5, -1), (0, 0)] {
        assert!(keys_for_output(&e, "한", min, max).is_empty(), "{min}..{max}");
    }
    assert!(keys_for_output(&e, "", 1, 3).is_empty());
    // A huge max is cut to MAX_PRESSES, not refused.
    assert!(has(&keys_for_output(&e, "한", 3, i64::MAX), "KeyG KeyK KeyS"));
    assert!(keys_for_output(&e, "한", MAX_PRESSES + 1, i64::MAX).is_empty());
}
