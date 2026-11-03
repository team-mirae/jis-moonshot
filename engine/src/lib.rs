//! Core engine of JIS Moonshot. All behavior is defined in `docs/spec.md` and `docs/moonshot.md`.
//!
//! Rust callers use [`Engine`]. The macOS, Windows and web shells use the C ABI in [`ffi`]
//! (`include/jis_korean.h`).

mod engine;
#[cfg(feature = "predict")]
pub mod brands;
pub mod ffi;
mod hangul;
#[cfg(feature = "predict")]
pub mod kanji;
#[cfg(feature = "predict")]
pub mod keymodel;
#[cfg(feature = "english")]
mod hold;
pub mod kana;
mod key;
mod layout;
pub mod plan;
pub mod punctuation;
pub mod resolve;
pub mod travel;
pub mod width;
#[cfg(feature = "predict")]
pub mod learn;
#[cfg(all(feature = "predict", not(feature = "legacy-predict")))]
pub mod learned;
#[cfg(feature = "predict")]
pub mod predict;
#[cfg(feature = "predict")]
pub mod trust;

pub use engine::{Candidate, Candidates, Engine, HostEdit, Output, PAGE_SIZE, Script, full_width, half_width};
pub use key::{Key, Mods};
pub use layout::{Action, Layout, Mode, Out, Step, output, output_in, output_with};
