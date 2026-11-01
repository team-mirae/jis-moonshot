//! Prints each key's legends in one mode, unshifted and with Shift, by C ABI key id (an action's in
//! full: 다, ㅡㄹ, 왜, docs/moonshot.md §9), for
//! shells that draw or bundle the layout (macOS builds its Keyboard Viewer layouts from this; spec
//! §3, §3.6).
//! Usage: cargo run --example legends [mode id]   (default 0, Mode A; lines of
//! `id<TAB>unshifted<TAB>shifted`)

use jis_korean::{Key, Layout, Mode, Out, output_in};

fn main() {
    let id = std::env::args().nth(1).and_then(|id| id.parse().ok()).unwrap_or(0);
    let Some(mode) = Mode::from_id(id) else {
        eprintln!("no mode {id} in this build");
        std::process::exit(1);
    };
    for (id, &key) in Key::ALL.iter().enumerate() {
        let legend = |shift: bool| {
            output_in(mode, Layout::JisKorean, key, shift).map_or(String::new(), Out::legend)
        };
        let (base, shifted) = (legend(false), legend(true));
        if !base.is_empty() || !shifted.is_empty() {
            println!("{id}\t{base}\t{shifted}");
        }
    }
}
