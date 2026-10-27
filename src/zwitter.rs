//! Zwitterwort: one spelling, two languages.
//!
//! `Gift` is a present in English and poison in German; `Kind` is a child, `Rat` is advice.
//! For a product name that is the difference between a joke and a recall. Give each
//! language its [`WordSet`] with glosses and ask which words they share.

use crate::finding::{Finding, Form};
use crate::lexicon::WordSet;
use crate::text::fold;

/// A word that is valid in more than one language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Zwitter {
    /// The folded spelling.
    pub word: String,
    /// `(language, gloss)` per language that has the word; the gloss is empty when unknown.
    pub senses: Vec<(String, String)>,
}

impl Zwitter {
    /// `Some(true)` when every language has a gloss and they are not all the same,
    /// `Some(false)` when they agree, `None` when glosses are missing.
    pub fn meaning_differs(&self) -> Option<bool> {
        if self.senses.iter().any(|(_, g)| g.is_empty()) {
            return None;
        }
        let first = self.senses[0].1.to_lowercase();
        Some(self.senses.iter().any(|(_, g)| g.to_lowercase() != first))
    }
}

impl From<&Zwitter> for Finding {
    fn from(z: &Zwitter) -> Self {
        let senses: Vec<String> = z
            .senses
            .iter()
            .map(|(lang, gloss)| {
                if gloss.is_empty() {
                    lang.clone()
                } else {
                    format!("{lang}: {gloss}")
                }
            })
            .collect();
        Finding {
            form: Form::Zwitter,
            text: z.word.clone(),
            parts: z.senses.iter().map(|(l, _)| l.clone()).collect(),
            score: match z.meaning_differs() {
                Some(true) => 1.0,
                Some(false) => 0.2,
                None => 0.5,
            },
            explanation: format!("\"{}\" is a word in {}", z.word, senses.join(" / ")),
        }
    }
}

/// How `word` reads in each of `langs`, if it is a word in at least two of them.
pub fn across(langs: &[(&str, &WordSet)], word: &str) -> Option<Zwitter> {
    let word = fold(word);
    let senses: Vec<(String, String)> = langs
        .iter()
        .filter(|(_, set)| set.contains(&word))
        .map(|(name, set)| (name.to_string(), set.gloss(&word).unwrap_or("").to_string()))
        .collect();
    (senses.len() >= 2).then_some(Zwitter { word, senses })
}

/// Every word the languages share. Words whose meaning differs come first.
pub fn shared(langs: &[(&str, &WordSet)]) -> Vec<Zwitter> {
    use crate::lexicon::Lexicon;
    let Some((_, first)) = langs.first() else {
        return Vec::new();
    };
    let mut out: Vec<Zwitter> = first.words().filter_map(|w| across(langs, w)).collect();
    out.sort_by(|a, b| {
        let rank = |z: &Zwitter| match z.meaning_differs() {
            Some(true) => 0,
            None => 1,
            Some(false) => 2,
        };
        rank(a).cmp(&rank(b)).then_with(|| a.word.cmp(&b.word))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sets() -> (WordSet, WordSet) {
        (
            WordSet::from_lines("gift\ta present\nhand\thand\nhouse\n"),
            WordSet::from_lines("gift\tpoison\nhand\tHand\nhaus\n"),
        )
    }

    #[test]
    fn gift_means_different_things() {
        let (en, de) = sets();
        let z = across(&[("en", &en), ("de", &de)], "Gift").unwrap();
        assert_eq!(z.meaning_differs(), Some(true));
        assert!(Finding::from(&z).explanation.contains("de: poison"));
    }

    #[test]
    fn cognates_agree_and_rank_last() {
        let (en, de) = sets();
        let all = shared(&[("en", &en), ("de", &de)]);
        let words: Vec<_> = all.iter().map(|z| z.word.as_str()).collect();
        assert_eq!(words, ["gift", "hand"]);
        assert_eq!(all[1].meaning_differs(), Some(false));
    }

    #[test]
    fn single_language_words_are_not_zwitters() {
        let (en, de) = sets();
        assert!(across(&[("en", &en), ("de", &de)], "house").is_none());
    }

    #[test]
    fn missing_glosses_are_unknown() {
        let a = WordSet::from_words(["rat"]);
        let b = WordSet::from_words(["rat"]);
        assert_eq!(
            across(&[("en", &a), ("de", &b)], "rat")
                .unwrap()
                .meaning_differs(),
            None
        );
    }
}
