//! Prints the key presses a text takes on JIS Korean and on conventional 2-set, as JSON.
//! Usage: cargo run --release --example count -- <text file>


use jis_korean::plan::count;

fn main() {
    let path = std::env::args().nth(1).expect("usage: count <text file>");
    let text = std::fs::read_to_string(&path).expect("read the text file");
    let c = count(&text);
    println!(
        r#"{{"chars": {}, "jis": {}, "conventional": {}, "off_keyboard": {}, "dropped": {}}}"#,
        c.chars, c.jis, c.conventional, c.off_keyboard, c.dropped
    );
}
