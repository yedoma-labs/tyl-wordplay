//! End-to-end checks through the public API with the bundled demo lists.

use tyl::{
    carousel_of, nests_of, reparse, sample, swaps, wheel_of, zwitter, Form, HazardKind, NestConfig,
    Playground, ReparseConfig, SwapConfig, WordSet,
};

#[test]
fn hazards_come_first_and_explain_themselves() {
    let (en, block) = (sample::english(), sample::blocklist_en());
    let findings = Playground::new(&en)
        .with_blocklist(&block)
        .scan("Pen Island");
    assert_eq!(findings[0].form, Form::Hazard);
    assert_eq!(findings[0].text, "penis land");
    assert!(findings[0].explanation.contains("penis land"));
}

#[test]
fn therapist_is_caught_inside_one_word() {
    let (en, block) = (sample::english(), sample::blocklist_en());
    let hazards = tyl::hazard::scan(&en, &block, "Therapist", &Default::default());
    assert_eq!(hazards[0].kind, HazardKind::Inside);
    assert_eq!(
        hazards[0].reading.as_deref(),
        Some(&["the".to_string(), "rapist".into()][..])
    );
}

#[test]
fn clean_names_are_clean() {
    let (en, block) = (sample::english(), sample::blocklist_en());
    let findings = Playground::new(&en)
        .with_blocklist(&block)
        .scan("Yedoma Labs");
    assert!(findings.iter().all(|f| f.form != Form::Hazard));
}

#[test]
fn every_form_works_on_the_demo_lists() {
    let (en, de) = (sample::english(), sample::german());
    assert_eq!(
        nests_of(&en, "carton", &NestConfig::default())[0].host,
        "con"
    );
    assert_eq!(
        nests_of(&de, "wunder", &NestConfig::default())[0].host,
        "wer"
    );
    assert_eq!(
        reparse(&en, "nowhere", &ReparseConfig::default())[0].text(),
        "now here"
    );
    assert!(wheel_of(&en, "bit").unwrap().is_full());
    assert_eq!(
        carousel_of(&en, "tea").unwrap().words,
        ["ate", "eat", "tea"]
    );
    let gift = zwitter::across(&[("en", &en), ("de", &de)], "Gift").unwrap();
    assert_eq!(gift.meaning_differs(), Some(true));
    let theme = WordSet::from_words(["stove"]);
    assert_eq!(
        swaps(&theme, "stone", &SwapConfig::default())[0].phrase,
        "stove"
    );
}

#[test]
fn output_is_deterministic() {
    let (en, block) = (sample::english(), sample::blocklist_en());
    let p = Playground::new(&en).with_blocklist(&block);
    assert_eq!(p.scan("Experts Exchange"), p.scan("Experts Exchange"));
}

#[test]
fn long_ambiguous_input_stays_bounded() {
    let lex = WordSet::from_words(["aaa", "aaaa", "aaaaa"]);
    let cfg = ReparseConfig {
        max_scan: 500,
        max_results: 5,
        ..Default::default()
    };
    let r = reparse(&lex, &"a".repeat(400), &cfg);
    assert_eq!(r.len(), 5);
}

#[test]
fn unicode_input_does_not_panic() {
    let en = sample::english();
    let p = Playground::new(&en);
    for text in ["", "   ", "Straße ß ÄÖÜ", "日本語", "a\u{0301}bc", "🙂🙂"] {
        let _ = p.scan(text);
    }
}
