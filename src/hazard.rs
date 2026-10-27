//! Stolperwort: does a name say something it should not?
//!
//! Brand names, domains and hashtags are written without spaces, so readers cut them
//! where they like: *Pen Island* (`penisland`), *Experts Exchange* (`expertsexchange`).
//! [`scan`] looks for blocklisted words that hide in the letters of a name and tells
//! whether the whole name can actually be read that way.

use crate::finding::{Finding, Form};
use crate::lexicon::Lexicon;
use crate::segment::Segmenter;
use crate::text::{collect, short_words, tokens, SHORT_WORDS_EN};

/// Where a blocklisted word sits in the name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HazardKind {
    /// A word of the name is itself blocklisted.
    Whole,
    /// Hidden inside one word of the name (`therapist` hides `rapist`; the "Scunthorpe problem").
    Inside,
    /// Spans the boundary between two or more words of the name.
    Crossing,
}

/// Limits for [`scan`].
#[derive(Debug, Clone)]
pub struct HazardConfig {
    /// Shortest blocklisted word to look for (short words match everywhere).
    pub min_len: usize,
    /// Also report words hidden inside a single word even when the name cannot be read
    /// with them (noisy: `assistant` contains `ass`). Inside words that complete to a full
    /// reading (`therapist` = `the rapist`) are always reported.
    pub include_inside: bool,
    /// Words shorter than three letters allowed when reading the rest of the name
    /// (default: [`SHORT_WORDS_EN`]).
    pub short_words: Vec<String>,
}

impl Default for HazardConfig {
    fn default() -> Self {
        Self {
            min_len: 3,
            include_inside: false,
            short_words: short_words(SHORT_WORDS_EN),
        }
    }
}

/// A blocklisted word found in a name.
#[derive(Debug, Clone, PartialEq)]
pub struct Hazard {
    /// The blocklisted word.
    pub word: String,
    /// Where it sits.
    pub kind: HazardKind,
    /// Letter offsets `start..end` into the name without spaces.
    pub span: (usize, usize),
    /// The full reading of the name that contains the word, if the rest can be read as words.
    pub reading: Option<Vec<String>>,
}

impl Hazard {
    /// Whole words and crossings that form a complete reading are the dangerous ones.
    pub fn is_confirmed(&self) -> bool {
        self.kind == HazardKind::Whole || self.reading.is_some()
    }
}

impl From<&Hazard> for Finding {
    fn from(h: &Hazard) -> Self {
        let (text, explanation) = match (&h.kind, &h.reading) {
            (HazardKind::Whole, _) => (
                h.word.clone(),
                "the name contains the blocklisted word".into(),
            ),
            (_, Some(r)) => (
                r.join(" "),
                format!("the name can be read as \"{}\"", r.join(" ")),
            ),
            (_, None) => (
                h.word.clone(),
                format!("\"{}\" is hidden in the letters of the name", h.word),
            ),
        };
        Finding {
            form: Form::Hazard,
            text,
            parts: h.reading.clone().unwrap_or_else(|| vec![h.word.clone()]),
            score: if h.is_confirmed() { 1.0 } else { 0.5 },
            explanation,
        }
    }
}

/// Blocklisted words hiding in `name`, confirmed readings first.
///
/// `lex` reads the rest of the name, `blocklist` lists what must not appear.
pub fn scan<L, B>(lex: &L, blocklist: &B, name: &str, cfg: &HazardConfig) -> Vec<Hazard>
where
    L: Lexicon + ?Sized,
    B: Lexicon + ?Sized,
{
    let toks = tokens(name);
    let letters: Vec<char> = toks.concat().chars().collect();
    let n = letters.len();
    let mut starts = Vec::with_capacity(toks.len());
    let mut at = 0;
    for t in &toks {
        starts.push(at);
        at += t.chars().count();
    }
    let boundaries: Vec<usize> = starts.iter().copied().skip(1).collect();

    let has = |w: &str| lex.has(w) || blocklist.has(w);
    let seg = Segmenter {
        has: &has,
        min_word: 3,
        short: &cfg.short_words,
        max_len: lex.max_word_len().max(blocklist.max_word_len()),
    };

    let mut out = Vec::new();
    for i in 0..n {
        for j in i + cfg.min_len.max(1)..=n {
            let word = collect(&letters[i..j]);
            if !blocklist.has(&word) {
                continue;
            }
            let crossing = boundaries.iter().any(|&b| i < b && b < j);
            let whole = starts
                .iter()
                .zip(toks.iter())
                .any(|(&s, t)| s == i && s + t.chars().count() == j);
            let kind = if crossing {
                HazardKind::Crossing
            } else if whole {
                HazardKind::Whole
            } else {
                HazardKind::Inside
            };
            let reading = if kind != HazardKind::Whole {
                seg.one(&letters[..i])
                    .zip(seg.one(&letters[j..]))
                    .map(|(mut l, r)| {
                        l.push(word.clone());
                        l.extend(r);
                        l
                    })
            } else {
                None
            };
            if kind == HazardKind::Inside && reading.is_none() && !cfg.include_inside {
                continue;
            }
            out.push(Hazard {
                word,
                kind,
                span: (i, j),
                reading,
            });
        }
    }
    out.sort_by(|a, b| {
        b.is_confirmed()
            .cmp(&a.is_confirmed())
            .then_with(|| (b.span.1 - b.span.0).cmp(&(a.span.1 - a.span.0)))
            .then_with(|| a.span.cmp(&b.span))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::WordSet;

    fn lex() -> WordSet {
        WordSet::from_words([
            "expert",
            "experts",
            "exchange",
            "change",
            "pen",
            "island",
            "land",
            "assistant",
        ])
    }

    fn block() -> WordSet {
        WordSet::from_words(["sex", "penis", "ass"])
    }

    #[test]
    fn experts_exchange_reads_as_expert_sex_change() {
        let h = scan(
            &lex(),
            &block(),
            "Experts Exchange",
            &HazardConfig::default(),
        );
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].word, "sex");
        assert_eq!(h[0].kind, HazardKind::Crossing);
        assert_eq!(
            h[0].reading.as_deref(),
            Some(&["expert".to_string(), "sex".into(), "change".into()][..])
        );
    }

    #[test]
    fn pen_island() {
        let h = scan(&lex(), &block(), "Pen Island", &HazardConfig::default());
        assert_eq!(h[0].word, "penis");
        assert!(h[0].is_confirmed());
    }

    #[test]
    fn inside_matches_are_opt_in() {
        assert!(scan(&lex(), &block(), "assistant", &HazardConfig::default()).is_empty());
        let cfg = HazardConfig {
            include_inside: true,
            ..Default::default()
        };
        let h = scan(&lex(), &block(), "assistant", &cfg);
        assert_eq!(h[0].kind, HazardKind::Inside);
        assert!(!h[0].is_confirmed());
    }

    #[test]
    fn inside_a_word_with_a_full_reading_is_always_reported() {
        let lex = WordSet::from_words(["the", "therapist"]);
        let block = WordSet::from_words(["rapist"]);
        let h = scan(&lex, &block, "Therapist", &HazardConfig::default());
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].kind, HazardKind::Inside);
        assert_eq!(
            h[0].reading.as_deref(),
            Some(&["the".to_string(), "rapist".into()][..])
        );
        assert!(h[0].is_confirmed());
    }

    #[test]
    fn a_blocklisted_word_of_its_own_is_whole() {
        let h = scan(&lex(), &block(), "Sex Shop", &HazardConfig::default());
        assert_eq!(h[0].kind, HazardKind::Whole);
    }

    #[test]
    fn clean_names_report_nothing() {
        assert!(scan(&lex(), &block(), "Yedoma Labs", &HazardConfig::default()).is_empty());
    }
}
