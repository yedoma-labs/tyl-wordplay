//! One call that runs every form over a text.

use std::fmt;

use crate::carousel::carousel_of;
use crate::finding::Finding;
use crate::hazard::{self, HazardConfig};
use crate::lexicon::WordSet;
use crate::nest::{nests_of, NestConfig};
use crate::reparse::{reparse, ReparseConfig};
use crate::text::tokens;

/// Limits for a [`Playground`]: the configuration of each detector, and the input size
/// that [`Playground::try_scan`] accepts.
///
/// The defaults are the defaults of the detectors. Tighten `max_input_chars` for untrusted
/// input, and the detector limits (`reparse.max_scan`, `reparse.max_results`, …) to bound
/// the run time.
#[derive(Debug, Clone)]
pub struct PlaygroundConfig {
    /// Settings for the [hazard] check (used when a blocklist is set).
    pub hazard: HazardConfig,
    /// Settings for [reparse](mod@crate::reparse).
    pub reparse: ReparseConfig,
    /// Settings for the search of nests in every word ([`nests_of`]).
    pub nest: NestConfig,
    /// Longest text, in characters, that [`Playground::try_scan`] accepts.
    pub max_input_chars: usize,
}

impl Default for PlaygroundConfig {
    fn default() -> Self {
        Self {
            hazard: HazardConfig::default(),
            reparse: ReparseConfig::default(),
            nest: NestConfig::default(),
            max_input_chars: 256,
        }
    }
}

/// Why [`Playground::try_scan`] refused a text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanError {
    /// The text is longer than [`PlaygroundConfig::max_input_chars`].
    TooLong {
        /// Length of the text in characters.
        chars: usize,
        /// The configured limit.
        max: usize,
    },
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::TooLong { chars, max } => {
                write!(f, "text has {chars} characters, the limit is {max}")
            }
        }
    }
}

impl std::error::Error for ScanError {}

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
    config: PlaygroundConfig,
}

impl<'a> Playground<'a> {
    /// Detectors that need only a lexicon, with the default [`PlaygroundConfig`].
    pub fn new(lexicon: &'a WordSet) -> Self {
        Self {
            lexicon,
            blocklist: None,
            config: PlaygroundConfig::default(),
        }
    }

    /// Adds a blocklist, which switches on the hazard check.
    pub fn with_blocklist(mut self, blocklist: &'a WordSet) -> Self {
        self.blocklist = Some(blocklist);
        self
    }

    /// Replaces the configuration of the detectors and the input limit.
    pub fn with_config(mut self, config: PlaygroundConfig) -> Self {
        self.config = config;
        self
    }

    /// The configuration in use.
    pub fn config(&self) -> &PlaygroundConfig {
        &self.config
    }

    /// Everything the detectors find in `text`, hazards first, then by score.
    ///
    /// There is no input limit here; use [`try_scan`](Playground::try_scan) for untrusted text.
    pub fn scan(&self, text: &str) -> Vec<Finding> {
        let cfg = &self.config;
        let mut out: Vec<Finding> = Vec::new();
        if let Some(block) = self.blocklist {
            out.extend(
                hazard::scan(self.lexicon, block, text, &cfg.hazard)
                    .iter()
                    .map(Finding::from),
            );
        }
        out.extend(
            reparse(self.lexicon, text, &cfg.reparse)
                .iter()
                .map(Finding::from),
        );
        for word in tokens(text) {
            out.extend(
                nests_of(self.lexicon, &word, &cfg.nest)
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

    /// Like [`scan`](Playground::scan), but refuses text longer than
    /// [`PlaygroundConfig::max_input_chars`] instead of doing the work.
    pub fn try_scan(&self, text: &str) -> Result<Vec<Finding>, ScanError> {
        let chars = text.chars().count();
        let max = self.config.max_input_chars;
        if chars > max {
            return Err(ScanError::TooLong { chars, max });
        }
        Ok(self.scan(text))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Form;
    use crate::hazard::HazardConfig;
    use crate::reparse::ReparseConfig;
    use crate::sample;

    #[test]
    fn default_config_keeps_the_demo_result() {
        let (en, block) = (sample::english(), sample::blocklist_en());
        let p = Playground::new(&en).with_blocklist(&block);
        let found = p.scan("Pen Island");
        assert_eq!(found[0].form, Form::Hazard);
        assert_eq!(found[0].text, "penis land");
        assert_eq!(p.config().max_input_chars, 256);
    }

    #[test]
    fn hazard_min_len_is_configurable() {
        // "sex" is three letters: found by default, not with a four-letter minimum.
        let lex = WordSet::from_words(["expert", "change", "exchange", "experts"]);
        let block = WordSet::from_words(["sex"]);
        let default = Playground::new(&lex).with_blocklist(&block);
        assert!(default
            .scan("Experts Exchange")
            .iter()
            .any(|f| f.form == Form::Hazard));

        let cfg = PlaygroundConfig {
            hazard: HazardConfig {
                min_len: 4,
                ..HazardConfig::default()
            },
            ..PlaygroundConfig::default()
        };
        let strict = Playground::new(&lex)
            .with_blocklist(&block)
            .with_config(cfg);
        assert!(strict
            .scan("Experts Exchange")
            .iter()
            .all(|f| f.form != Form::Hazard));
    }

    #[test]
    fn reparse_limits_are_configurable() {
        let lex = WordSet::from_words(["no", "where", "now", "here", "nowhere"]);
        let cfg = PlaygroundConfig {
            reparse: ReparseConfig {
                max_results: 1,
                ..ReparseConfig::default()
            },
            ..PlaygroundConfig::default()
        };
        let count = |p: &Playground| {
            p.scan("nowhere")
                .iter()
                .filter(|f| f.form == Form::Reparse)
                .count()
        };
        assert_eq!(count(&Playground::new(&lex)), 2);
        assert_eq!(count(&Playground::new(&lex).with_config(cfg)), 1);
    }

    #[test]
    fn try_scan_refuses_long_text_and_accepts_the_limit() {
        let lex = WordSet::from_words(["no", "where", "now", "here", "nowhere"]);
        let cfg = PlaygroundConfig {
            max_input_chars: 7,
            ..PlaygroundConfig::default()
        };
        let p = Playground::new(&lex).with_config(cfg);

        // exactly at the limit: accepted
        assert!(p.try_scan("nowhere").is_ok());
        // one character over: refused, with both numbers
        assert_eq!(
            p.try_scan("nowhere!"),
            Err(ScanError::TooLong { chars: 8, max: 7 })
        );
        // counts characters, not bytes: "nöwhere" is 7 chars but 8 bytes
        assert!(p.try_scan("nöwhere").is_ok());
    }

    #[test]
    fn try_scan_matches_scan_when_accepted() {
        let (en, block) = (sample::english(), sample::blocklist_en());
        let p = Playground::new(&en).with_blocklist(&block);
        assert_eq!(p.try_scan("Pen Island").unwrap(), p.scan("Pen Island"));
    }

    #[test]
    fn scan_error_explains_itself() {
        let e = ScanError::TooLong {
            chars: 300,
            max: 256,
        };
        assert_eq!(e.to_string(), "text has 300 characters, the limit is 256");
    }
}
