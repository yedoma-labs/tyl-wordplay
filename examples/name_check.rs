//! Checks product names, domains or hashtags for unintended readings.
//!
//!     cargo run --example name_check -- "Pen Island" "Experts Exchange" Yedoma
//!     cargo run --example name_check -- --dict /usr/share/dict/words "Therapist Finder"
//!
//! Without `--dict` the tiny demo lists are used; a real dictionary finds far more.

use tyl::{sample, Playground, WordSet};

fn main() {
    let mut args = std::env::args().skip(1).peekable();
    let mut lexicon = sample::english();
    if args.peek().map(String::as_str) == Some("--dict") {
        args.next();
        let path = args.next().expect("--dict needs a path");
        lexicon = WordSet::from_file(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
    }
    let blocklist = sample::blocklist_en();
    let playground = Playground::new(&lexicon).with_blocklist(&blocklist);

    let names: Vec<String> = args.collect();
    if names.is_empty() {
        eprintln!("usage: name_check [--dict FILE] NAME...");
        std::process::exit(2);
    }
    let mut risky = false;
    for name in &names {
        let findings = playground.scan(name);
        if findings.is_empty() {
            println!("ok    {name}");
        }
        for f in findings {
            risky |= f.form == tyl::Form::Hazard;
            println!(
                "{:<5} {name}: {f}",
                if f.form == tyl::Form::Hazard {
                    "WARN"
                } else {
                    "note"
                }
            );
        }
    }
    std::process::exit(i32::from(risky));
}
