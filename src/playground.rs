//! One call that runs every form over a text.

use crate::carousel::carousel_of;
use crate::finding::Finding;
use crate::hazard::{self, HazardConfig};
use crate::lexicon::WordSet;
use crate::nest::{nests_of, NestConfig};
use crate::reparse::{reparse, ReparseConfig};
use crate::text::tokens;

/// Runs the detectors of this crate against a text and returns [`Finding`]s.
///
/// Create it with the lexicon of the language of your text and, optionally, a blocklist
/// for the [hazard] check. Generators that need extra input
/// ([`swaps`](crate::swaps), [`insert_everywhere`](crate::insert_everywhere),
/// [`zwitter`](mod@crate::zwitter)) stay separate functions.
#[derive(Debug, Clone)]
pub struct Playground<'a> {
    lexicon: &'a WordSet,
    blocklist: Option<&'a WordSet>,
}

impl<'a> Playground<'a> {
    /// Detectors that need only a lexicon.
    pub fn new(lexicon: &'a WordSet) -> Self {
        Self {
            lexicon,
            blocklist: None,
        }
    }

    /// Adds a blocklist, which switches on the hazard check.
    pub fn with_blocklist(mut self, blocklist: &'a WordSet) -> Self {
        self.blocklist = Some(blocklist);
        self
    }

    /// Everything the detectors find in `text`, hazards first, then by score.
    pub fn scan(&self, text: &str) -> Vec<Finding> {
        let mut out: Vec<Finding> = Vec::new();
        if let Some(block) = self.blocklist {
            out.extend(
                hazard::scan(self.lexicon, block, text, &HazardConfig::default())
                    .iter()
                    .map(Finding::from),
            );
        }
        out.extend(
            reparse(self.lexicon, text, &ReparseConfig::default())
                .iter()
                .map(Finding::from),
        );
        for word in tokens(text) {
            out.extend(
                nests_of(self.lexicon, &word, &NestConfig::default())
                    .iter()
                    .map(Finding::from),
            );
            out.extend(carousel_of(self.lexicon, &word).iter().map(Finding::from));
        }
        out.sort_by(|a, b| {
            let hazard = |f: &Finding| f.form != crate::Form::Hazard;
            hazard(a)
                .cmp(&hazard(b))
                .then_with(|| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| a.text.cmp(&b.text))
        });
        out
    }

    /// The lexicon in use.
    pub fn lexicon(&self) -> &WordSet {
        self.lexicon
    }

    /// Whether a word is known to the lexicon.
    pub fn knows(&self, word: &str) -> bool {
        self.lexicon.contains(word)
    }
}
