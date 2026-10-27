# tyl-wordplay

<picture>
  <source media="(max-width: 640px)" srcset="https://raw.githubusercontent.com/yedoma-labs/assets/main/resized/banner-resized-mobile.png">
  <img src="https://raw.githubusercontent.com/yedoma-labs/assets/main/resized/banner-resized.png" alt="Project Header">
</picture>

> **A wordplay engine for Rust** — finds, explains and generates forms of wordplay that no other crate covers, and checks your names for the readings you did not intend

[![CI](https://github.com/yedoma-labs/tyl-wordplay/actions/workflows/ci.yml/badge.svg)](https://github.com/yedoma-labs/tyl-wordplay/actions/workflows/ci.yml)
[![Security Audit](https://github.com/yedoma-labs/tyl-wordplay/actions/workflows/audit.yml/badge.svg)](https://github.com/yedoma-labs/tyl-wordplay/actions/workflows/audit.yml)
[![Crates.io](https://img.shields.io/crates/v/tyl-wordplay.svg)](https://crates.io/crates/tyl-wordplay)
[![Documentation](https://docs.rs/tyl-wordplay/badge.svg)](https://docs.rs/tyl-wordplay)
[![License](https://img.shields.io/crates/l/tyl-wordplay.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-stable-blue.svg)](https://www.rust-lang.org)
[![No Unsafe](https://img.shields.io/badge/unsafe-forbidden-success)](src/lib.rs)
[![Dependencies](https://img.shields.io/badge/dependencies-0-success)](Cargo.toml)
[![Conventional Commits](https://img.shields.io/badge/commits-conventional-fe5196?logo=conventionalcommits&logoColor=white)](https://www.conventionalcommits.org/)

*tyl* (тыл; Yakutian/Sakha) = "word, language" — small, exact, and it says what it means

## Why tyl-wordplay?

**The Problem:** Wordplay crates implement one classic trick each (`anagram`, `palindrome`, `portmanteau`, `zummi` for spoonerisms), return bare strings, and say nothing about *why* a result counts. And nobody checks the thing that costs real money: the name you are about to print on a box, register as a domain or use as a hashtag, read with the spaces in a different place.

**The Solution:** One engine, seven forms defined as testable types, any word list you like, and every result carries an explanation.

```rust
use tyl::{sample, Playground};

let en = sample::english();          // tiny demo list; load a real dictionary in production
let block = sample::blocklist_en();
let findings = Playground::new(&en).with_blocklist(&block).scan("Experts Exchange");

println!("{}", findings[0]);
// [hazard] expert sex change - the name can be read as "expert sex change"
```

## What makes it unique

| | |
|---|---|
| **Name hazards** | `hazard::scan` finds blocklisted words that hide *across* the boundaries of a name (`Pen Island`, `Experts Exchange`) or *inside* a word when the rest completes to a full reading (`Therapist` = `the rapist`). Findings that form a complete reading are flagged as confirmed, so you see the dangerous ones first instead of every `ass` in `assistant`. |
| **All readings, not the best guess** | Word splitters return the one most probable split. `reparse` returns every other way to cut the same letters into words, ranked, because the second reading is the pun (or the accident). |
| **Forms as types** | A nest, a vowel wheel, a zwitter, a carousel are structs with fields (`host`, `at`, `skeleton`, `senses`, …), not strings. You can sort, filter and render them yourself; `Finding` is the uniform view on top. |
| **Explainable** | Every `Finding` has `parts`, a `score` and a sentence saying why it counts. |
| **Bring your own words** | Everything is generic over the `Lexicon` trait. Load `/usr/share/dict/words`, a frequency-filtered list, an FST or a database. German and English work out of the box (umlauts stay distinct letters, `ß` folds to `ss`). |
| **Cross-language aware** | `zwitter` finds spellings that are words in two languages with different meanings (`Gift`, `Rat`, `Kind`, `Rock`), using glosses from your word lists. |
| **Deterministic, bounded, dependency-free** | Same input, same output. Search windows and result counts are capped, so adversarial input cannot blow up the run time. `#![forbid(unsafe_code)]`, zero dependencies. |

## The seven forms

The names and exact definitions are this crate's; some phenomena are known from puzzle culture (rotations, container clues, segmentation puns). What is new is having them as one measurable, explainable, language-agnostic API.

| Form | German | Definition | Example |
|---|---|---|---|
| `nest` | Schachtelwort | A word holds a word strictly inside, and what is left is a word too | `carton` = c(**art**)on → `con`; `Wunder` = W(**und**)er → `Wer` |
| `reparse` | Neuschnitt | The same letters, cut into other words | `nowhere` = `now here` = `no where` |
| `wheel` | Vokalrad | Words that differ only in their vowels; *full* when all of `a e i o u` fit | `bat bet bit bot but` |
| `zwitter` | Zwitterwort | One spelling that is a word in two languages | `Gift`: a present (en) / poison (de) |
| `swap` | Austauschpointe | Replace one word of a phrase by a theme word that sounds close, with the same syllable count | `a rolling stone` → `a rolling stove` |
| `hazard` | Stolperwort | A name that says something it should not | `Experts Exchange` → `expert sex change` |
| `carousel` | Karussellwort | Rotating the letters gives other words | `tea` > `eat` > `ate` |

## Usage

```toml
[dependencies]
tyl-wordplay = "0.1"
```

The library is imported as `tyl`.

```rust
use tyl::{nests_of, reparse, swaps, wheel_of, NestConfig, ReparseConfig, SwapConfig, WordSet};

// your own words: one per line, optionally `word<TAB>meaning`
let words = WordSet::from_file("/usr/share/dict/words")?;

for n in nests_of(&words, "carton", &NestConfig::default()) {
    println!("{} holds {}, leaving {}", n.outer, n.inner, n.host);
}

for r in reparse(&words, "therapist", &ReparseConfig::default()) {
    println!("{}", r.text()); // the rapist
}

// puns: swap one word for a theme word
let kitchen = WordSet::from_words(["stove", "kettle", "toast"]);
for s in swaps(&kitchen, "a rolling stone", &SwapConfig::default()) {
    println!("{}", s.phrase); // a rolling stove
}
```

For untrusted text, bound the work with a `Playground` configuration and `try_scan`, which refuses text longer than `max_input_chars` (256 by default). `PlaygroundConfig` also sets the limits of each detector:

```rust
use tyl::{sample, Playground, PlaygroundConfig, ScanError};

let mut cfg = PlaygroundConfig::default();
cfg.max_input_chars = 64;
cfg.reparse.max_results = 5;
let en = sample::english();
match Playground::new(&en).with_config(cfg).try_scan(&input) {
    Ok(findings) => { /* ... */ }
    Err(ScanError::TooLong { chars, max }) => eprintln!("{chars} characters, limit {max}"),
}
```

Check names from the command line (exit code 1 when a hazard is found, handy in CI):

```bash
cargo run --example name_check -- --dict /usr/share/dict/words "Pen Island" "Therapist Finder" Yedoma
cargo run --example tour        # every form once
```

## Choosing word lists

The bundled lists (`tyl::sample`) are a few hundred words, enough for the docs and tests. A raw system dictionary works but is full of fragments (`ssi`, `pon`), which shows up as noise in nests and long readings. For production use a frequency-filtered list of the language of your text. `WordSet::from_frequency_file(path, 1000)` reads the common `word count` layout and keeps the words seen at least 1000 times. Set `HazardConfig::short_words` / `ReparseConfig::short_words` (`SHORT_WORDS_EN`, `SHORT_WORDS_DE`) because words under three letters are only accepted from that allowlist.

You decide what goes on the blocklist. The bundled `blocklist_en` is a ten-word demo, not a moderation list.

## Limitations

- **Text-based, not phonetic.** `swap` uses a coarse language-agnostic sound folding (`ph`→`f`, `ck`→`k`, …) and counts vowel groups as syllables. It is a ranking heuristic, not a pronunciation model; homophones that are spelled very differently (`ice cream` / `I scream`) are not found.
- **No meaning.** Apart from the glosses you provide for `zwitter`, the crate does not know what words mean, so it cannot tell whether a pun is funny. Scores rank candidates; a human picks.
- **Quality follows your lexicon.** Garbage in the word list becomes garbage readings.
- **Novelty claim.** We searched crates.io (2026-10) for wordplay, anagram, palindrome, spoonerism, portmanteau, segmentation and name-checking crates and found only single-form crates and word splitters. We cannot prove that nothing similar exists elsewhere.

## Development

```bash
cargo test
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
git config core.hooksPath .githooks   # Conventional Commits
```

See [CONTRIBUTING.md](CONTRIBUTING.md). Releases are cut from Conventional Commits by release-plz.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
