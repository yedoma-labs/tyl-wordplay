//! A walk through all seven forms with the demo word lists.
//!
//!     cargo run --example tour

use tyl::{
    carousel_of, insert_everywhere, nests_of, reparse, sample, swaps, wheel_of, zwitter, Finding,
    NestConfig, ReparseConfig, SwapConfig, WordSet,
};

fn show<'a>(title: &str, findings: impl IntoIterator<Item = Finding> + 'a) {
    println!("\n== {title}");
    for f in findings {
        println!("  {f}");
    }
}

fn main() {
    let en = sample::english();
    let de = sample::german();

    show(
        "Nest (Schachtelwort)",
        nests_of(&en, "carton", &NestConfig::default())
            .iter()
            .chain(&nests_of(&de, "wunder", &NestConfig::default()))
            .chain(&insert_everywhere(&en, "con", "art"))
            .map(Finding::from),
    );
    show(
        "Reparse (Neuschnitt)",
        reparse(&en, "nowhere", &ReparseConfig::default())
            .iter()
            .map(Finding::from),
    );
    show(
        "Vowel wheel (Vokalrad)",
        wheel_of(&en, "bit").iter().map(Finding::from),
    );
    show(
        "Zwitter (Zwitterwort)",
        zwitter::across(&[("en", &en), ("de", &de)], "gift")
            .iter()
            .map(Finding::from),
    );
    let theme = WordSet::from_words(["stove", "kettle", "toast"]);
    show(
        "Swap (Austauschpointe)",
        swaps(
            &theme,
            "A rolling stone gathers no moss",
            &SwapConfig::default(),
        )
        .iter()
        .map(Finding::from),
    );
    show(
        "Carousel (Karussellwort)",
        carousel_of(&en, "tea").iter().map(Finding::from),
    );
    show(
        "Hazard (Stolperwort)",
        tyl::Playground::new(&en)
            .with_blocklist(&sample::blocklist_en())
            .scan("Experts Exchange"),
    );
}
