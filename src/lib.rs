//! # tyl-wordplay
//!
//! A wordplay engine. Most crates implement one classic trick (anagrams, palindromes,
//! spoonerisms). This one defines **seven forms as measurable types**, runs them through
//! one lexicon-agnostic engine and makes every result **explain itself**:
//!
//! | Form | German | Example |
//! |---|---|---|
//! | [`nest`] | Schachtelwort | `carton` = c(**art**)on → `con` |
//! | [`reparse`](mod@reparse) | Neuschnitt | `nowhere` = `now here` = `no where` |
//! | [`wheel`] | Vokalrad | `bat bet bit bot but` |
//! | [`zwitter`] | Zwitterwort | `Gift`: present (en) / poison (de) |
//! | [`swap`] | Austauschpointe | `a rolling stone` → `a rolling stove` |
//! | [`hazard`] | Stolperwort | `Experts Exchange` → `expert sex change` |
//! | [`carousel`] | Karussellwort | `tea` > `eat` > `ate` |
//!
//! ```
//! use tyl::{sample, Playground};
//!
//! let en = sample::english();
//! let block = sample::blocklist_en();
//! let findings = Playground::new(&en).with_blocklist(&block).scan("Experts Exchange");
//! assert!(findings[0].explanation.contains("expert sex change"));
//! ```
//!
//! No dependencies, no `unsafe`, deterministic output. You bring the words: anything that
//! implements [`Lexicon`], or a [`WordSet`] loaded from a plain list.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod carousel;
pub mod finding;
pub mod hazard;
pub mod lexicon;
pub mod nest;
pub mod playground;
pub mod reparse;
pub mod sample;
mod segment;
pub mod swap;
pub mod text;
pub mod wheel;
pub mod zwitter;

pub use carousel::{all_carousels, carousel_of, Carousel};
pub use finding::{Finding, Form};
pub use hazard::{Hazard, HazardConfig, HazardKind};
pub use lexicon::{Lexicon, WordSet};
pub use nest::{all_nests, insert_everywhere, nests_of, Nest, NestConfig};
pub use playground::{Playground, PlaygroundConfig, ScanError};
pub use reparse::{reparse, Reparse, ReparseConfig};
pub use swap::{swaps, Swap, SwapConfig};
pub use text::{SHORT_WORDS_DE, SHORT_WORDS_EN};
pub use wheel::{wheel_of, wheels, Wheel};
pub use zwitter::Zwitter;
