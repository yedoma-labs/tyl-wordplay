//! Neuschnitt: the same letters, cut into different words.
//!
//! `nowhere` is `now here`, `therapist` is `the rapist`, `a part` is `apart`. A reparse is
//! every other way to split the letters of a phrase into dictionary words. Unlike a
//! word-splitting tool that returns the single most probable split, this returns *all*
//! readings, ranked, because the second reading is the pun (or the accident).

use crate::finding::{Finding, Form};
use crate::lexicon::Lexicon;
use crate::segment::Segmenter;
use crate::text::{short_words, tokens, SHORT_WORDS_EN};

/// Limits for [`reparse`].
#[derive(Debug, Clone)]
pub struct ReparseConfig {
    /// Shortest word taken from the lexicon in a reading.
    pub min_word: usize,
    /// Words shorter than `min_word` that may still appear (default: [`SHORT_WORDS_EN`]).
    pub short_words: Vec<String>,
    /// How many readings to return.
    pub max_results: usize,
    /// How many splits to enumerate before ranking (bounds the work on long inputs).
    pub max_scan: usize,
}

impl Default for ReparseConfig {
    fn default() -> Self {
        Self {
            min_word: 3,
            short_words: short_words(SHORT_WORDS_EN),
            max_results: 20,
            max_scan: 10_000,
        }
    }
}

/// An alternative reading of a phrase.
#[derive(Debug, Clone, PartialEq)]
pub struct Reparse {
    /// The words as written.
    pub original: Vec<String>,
    /// The words of the new reading.
    pub reading: Vec<String>,
    /// Heuristic: few, long words rank higher.
    pub score: f32,
}

impl Reparse {
    /// The new reading as text.
    pub fn text(&self) -> String {
        self.reading.join(" ")
    }
}

impl From<&Reparse> for Finding {
    fn from(r: &Reparse) -> Self {
        Finding {
            form: Form::Reparse,
            text: r.text(),
            parts: r.reading.clone(),
            score: r.score,
            explanation: format!(
                "\"{}\" has the same letters as \"{}\"",
                r.original.join(" "),
                r.text()
            ),
        }
    }
}

/// All other ways to read the letters of `phrase` as words of `lex`, best first.
pub fn reparse<L: Lexicon + ?Sized>(lex: &L, phrase: &str, cfg: &ReparseConfig) -> Vec<Reparse> {
    let original = tokens(phrase);
    let letters: Vec<char> = original.concat().chars().collect();
    if letters.is_empty() {
        return Vec::new();
    }
    let has = |w: &str| lex.has(w);
    let seg = Segmenter {
        has: &has,
        min_word: cfg.min_word,
        short: &cfg.short_words,
        max_len: lex.max_word_len(),
    };
    let mut readings: Vec<Reparse> = seg
        .all(&letters, cfg.max_scan)
        .into_iter()
        .filter(|s| *s != original)
        .map(|reading| {
            let shortest = reading.iter().map(|w| w.chars().count()).min().unwrap_or(0);
            let score = (shortest.min(4) as f32 / 4.0) / (reading.len() as f32 / 2.0).max(1.0);
            Reparse {
                original: original.clone(),
                reading,
                score,
            }
        })
        .collect();
    readings.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.reading.cmp(&b.reading))
    });
    readings.truncate(cfg.max_results);
    readings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::WordSet;

    fn lex() -> WordSet {
        WordSet::from_words([
            "no", "where", "now", "here", "nowhere", "a", "part", "apart", "i",
        ])
    }

    #[test]
    fn nowhere_has_two_other_readings() {
        let r = reparse(&lex(), "nowhere", &ReparseConfig::default());
        let texts: Vec<_> = r.iter().map(Reparse::text).collect();
        assert_eq!(texts, ["now here", "no where"]);
    }

    #[test]
    fn spaces_moving_back_is_a_reading() {
        let r = reparse(&lex(), "a part", &ReparseConfig::default());
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].text(), "apart");
    }

    #[test]
    fn the_original_split_is_not_a_reading() {
        let r = reparse(&lex(), "no where", &ReparseConfig::default());
        assert!(r.iter().all(|x| x.text() != "no where"));
        assert!(r.iter().any(|x| x.text() == "now here"));
    }

    #[test]
    fn nothing_for_unsplittable_or_empty() {
        assert!(reparse(&lex(), "zzz", &ReparseConfig::default()).is_empty());
        assert!(reparse(&lex(), "  ", &ReparseConfig::default()).is_empty());
    }
}
