//! The mode contract (spec §3.6, docs/modes.md), checked for every mode. JIS Moonshot is JIS only
//! (docs/moonshot.md §2): Modes B and C and their English modes are retired.

use jis_korean::{Engine, Key, Layout, Mode, Mods, Out, output_in, output_with};

const LETTERS: &str = "QWERTYUIOPASDFGHJKLZXCVBNM";

fn letter_keys() -> impl Iterator<Item = Key> {
    LETTERS.chars().map(|c| Key::from_code(&format!("Key{c}")))
}

#[test]
fn ids_are_unique_and_round_trip() {
    for (i, mode) in Mode::ALL.iter().enumerate() {
        assert_eq!(Mode::from_id(mode.id()), Some(*mode));
        assert!(
            Mode::ALL[..i].iter().all(|m| m.id() != mode.id() && m.name() != mode.name() && m.code() != mode.code()),
            "{mode:?} reuses an id, a name or a mode name"
        );
        assert!(!mode.name().is_empty());
    }
    #[cfg(not(feature = "english"))]
    assert_eq!(Mode::from_id(3), None, "JIS English is built only with the english feature");
    // Retired (B, C, B_QWERTY, CJ_QWERTY), reserved (4–6, A_ROMAJI) or never given.
    for id in [1, 2, 4, 9, 10, 12, 31, 99, 10_000, u32::MAX] {
        assert_eq!(Mode::from_id(id), None, "id {id} has no mode in this build: it's refused, not replaced");
    }
    assert_eq!(Mode::default(), Mode::A);
}

/// Spec §3.6.1: mode names are upper-case snake case, and each namespace has its id range.
#[test]
fn mode_names_and_ids_are_in_their_ranges() {
    for &mode in Mode::ALL {
        let (id, code) = (mode.id(), mode.code());
        let words: Vec<&str> = code.split('_').collect();
        assert!(
            words.iter().all(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()))
                && code.starts_with(|c: char| c.is_ascii_uppercase()),
            "{code} isn't upper-case snake case"
        );
        let in_range = match words.as_slice() {
            _ if (10_000..=11_000).contains(&id) => true, // a preview's temporary id
            [_] => id == 0,                              // A keeps its letter
            // A_ JIS, B_ US ANSI, C_ ISO, CJ_ ISO also on JIS: official (0–14) or contributed (15–30).
            ["A" | "B" | "C" | "CJ", ..] => id <= 30,
            _ => (31..=99).contains(&id),
        };
        assert!(in_range, "{code} can't have id {id}");
    }
}

#[test]
fn every_mode_keeps_standard_two_set_on_the_letter_keys() {
    for &mode in Mode::ALL.iter().filter(|m| m.is_korean()) {
        for layout in [Layout::JisKorean, Layout::Escaped] {
            for key in letter_keys() {
                for shift in [false, true] {
                    assert_eq!(output_in(mode, layout, key, shift), output_in(Mode::A, layout, key, shift), "{mode:?} {key:?}");
                }
            }
        }
    }
}

#[test]
fn every_mode_keeps_standard_two_set_with_its_symbol_keys_off() {
    for &mode in Mode::ALL.iter().filter(|m| m.is_korean()) {
        for key in letter_keys() {
            for shift in [false, true] {
                assert_eq!(output_with(mode, false, Layout::JisKorean, key, shift), output_in(Mode::A, Layout::JisKorean, key, shift), "{mode:?} {key:?}");
            }
        }
    }
}

#[test]
fn optional_keys_type_stock_when_off_and_nothing_else_changes() {
    for &mode in Mode::ALL.iter().filter(|m| m.is_korean()) {
        for &key in Key::ALL {
            for shift in [false, true] {
                let off = output_with(mode, false, Layout::JisKorean, key, shift);
                if mode.optional_keys().contains(&key) {
                    // Off, an optional key types the stock keyboard's character.
                    assert_eq!(off, output_in(mode, Layout::Escaped, key, shift), "{mode:?} {key:?}");
                    assert!(matches!(off, Some(Out::Char(_))), "{mode:?} {key:?} has no stock character");
                } else {
                    assert_eq!(off, output_in(mode, Layout::JisKorean, key, shift), "{mode:?} {key:?}");
                }
                // The stock keyboard is the same either way (English readings use it).
                assert_eq!(output_with(mode, false, Layout::Escaped, key, shift), output_in(mode, Layout::Escaped, key, shift));
            }
        }
    }
}

#[test]
fn every_mode_has_a_stock_reading_for_each_key_it_maps() {
    for &mode in Mode::ALL {
        for &key in Key::ALL {
            for shift in [false, true] {
                if let Some(Out::Char(_)) = output_in(mode, Layout::JisKorean, key, shift) {
                    assert!(matches!(output_in(mode, Layout::Escaped, key, shift), Some(Out::Char(_))), "{mode:?} {key:?} has no stock character");
                }
            }
        }
    }
}

#[test]
fn every_mode_leaves_the_control_keys_alone() {
    for &mode in Mode::ALL {
        for key in [Key::Space, Key::Enter, Key::Backspace, Key::F6, Key::F10, Key::Escape, Key::Modifier, Key::Other] {
            for layout in [Layout::JisKorean, Layout::Escaped] {
                assert_eq!(output_in(mode, layout, key, false), None, "{mode:?} {key:?}");
            }
        }
    }
}

#[test]
fn backquote_and_the_iso_key_pass_through_in_mode_a() {
    let mut a = Engine::new();
    assert!(!a.key(Key::Backquote, Mods::NONE).consumed);
    assert!(!a.key(Key::IntlBackslash, Mods::NONE).consumed);
}

#[test]
fn mode_a_is_unchanged() {
    let mut e = Engine::new();
    for (code, shift) in [("Digit2", true), ("BracketLeft", false), ("Equal", false)] {
        e.key(Key::from_code(code), if shift { Mods::SHIFTED } else { Mods::NONE });
    }
    // docs/moonshot.md §9: @ is ㅡㄹ and ^ is 다; English still reads the stock keys.
    assert_eq!((e.preedit().as_str(), e.english().as_str()), ("2을다", "\"@^"));
}

/// Spec §3.6.1 over the C ABI: modes are listed by position, and an unknown id is refused.
#[test]
fn c_abi_lists_modes_by_position_and_refuses_unknown_ids() {
    use jis_korean::ffi::*;
    let ids: Vec<u32> = (0..jk_mode_count()).map(|i| jk_mode_id_at(i)).collect();
    assert_eq!(ids, Mode::ALL.iter().map(|m| m.id()).collect::<Vec<_>>());
    assert_eq!(jk_mode_id_at(jk_mode_count()), u32::MAX);
    assert_eq!(jk_mode_code_len(0), 1);
    assert_eq!(jk_mode_code_len(2), 0, "Mode C is retired");
    assert_eq!((jk_mode_name_len(9_999), jk_mode_code_len(9_999)), (0, 0));
    for mode in Mode::ALL {
        assert_eq!(jk_mode_is_english(mode.id()), mode.is_english() as u32, "{mode:?}");
    }
    assert_eq!(jk_mode_is_english(9_999), 0);
    let id = |key: Key| Key::ALL.iter().position(|&k| k == key).unwrap() as u32;
    assert_ne!(jk_legend_in(id(Key::Digit1), 0, 0, 0), 0);
    assert_eq!(jk_legend_in(id(Key::Digit1), 0, 0, 1), 0, "Mode B is retired");

    let engine = jk_new();
    assert_eq!(jk_set_mode(engine, 11), 1);
    assert_eq!(jk_mode(engine), 11);
    assert_eq!(jk_set_mode(engine, 1), 0, "Mode B is retired");
    // Still JIS Kana: T types か.
    jk_key(engine, id(Key::KeyT), 0);
    assert_eq!(unsafe { std::slice::from_raw_parts(jk_preedit_ptr(engine), jk_preedit_len(engine)) }, "か".as_bytes());
    // The かな key: back to Mode A, the key forced Korean.
    assert_eq!(jk_kana_key(engine), 1);
    assert_eq!((jk_mode(engine), jk_script(engine)), (0, 1));
    assert_eq!(unsafe { std::slice::from_raw_parts(jk_preedit_ptr(engine), jk_preedit_len(engine)) }, "ㅅ".as_bytes());
    unsafe { jk_free(engine) };
}
