//! Karussellwort: turn the letters round and round, get other words.
//!
//! `tea` → `eat` → `ate`. A carousel is the set of cyclic rotations of a word that are
//! words too (at least two, counting the word itself).

use crate::finding::{Finding, Form};
use crate::lexicon::Lexicon;
use crate::text::{collect, fold};

/// The words reachable from each other by rotating the letters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Carousel {
    /// The words, sorted.
    pub words: Vec<String>,
}

impl From<&Carousel> for Finding {
    fn from(c: &Carousel) -> Self {
        Finding {
            form: Form::Carousel,
            text: c.words.join(" > "),
            parts: c.words.clone(),
            score: (c.words.len() as f32 / 3.0).min(1.0),
            explanation: format!(
                "rotating the letters of \"{}\" gives {}",
                c.words[0],
                c.words[1..].join(", ")
            ),
        }
    }
}

fn orbit<L: Lexicon + ?Sized>(lex: &L, w: &[char]) -> Vec<String> {
    let mut found: Vec<String> = (0..w.len())
        .map(|k| {
            let mut r = w[k..].to_vec();
            r.extend_from_slice(&w[..k]);
            collect(&r)
        })
        .filter(|r| lex.has(r))
        .collect();
    found.sort();
    found.dedup();
    found
}

/// The carousel `word` is on, if some other rotation is a word.
pub fn carousel_of<L: Lexicon + ?Sized>(lex: &L, word: &str) -> Option<Carousel> {
    let word = fold(word);
    if !lex.has(&word) {
        return None;
    }
    let words = orbit(lex, &word.chars().collect::<Vec<_>>());
    (words.len() >= 2).then_some(Carousel { words })
}

/// Every carousel of the lexicon once, longest first.
pub fn all_carousels<L: Lexicon + ?Sized>(lex: &L) -> Vec<Carousel> {
    let mut out: Vec<Carousel> = lex.words().filter_map(|w| carousel_of(lex, w)).collect();
    out.sort_by(|a, b| {
        b.words
            .len()
            .cmp(&a.words.len())
            .then_with(|| a.words.cmp(&b.words))
    });
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::WordSet;

    fn lex() -> WordSet {
        WordSet::from_words(["tea", "eat", "ate", "cat", "dog", "ale", "lea"])
    }

    #[test]
    fn tea_eat_ate() {
        let c = carousel_of(&lex(), "tea").unwrap();
        assert_eq!(c.words, ["ate", "eat", "tea"]);
        assert_eq!(Finding::from(&c).text, "ate > eat > tea");
    }

    #[test]
    fn needs_a_second_word() {
        assert!(carousel_of(&lex(), "cat").is_none());
    }

    #[test]
    fn ale_lea_is_a_carousel_and_listed_once() {
        let all = all_carousels(&lex());
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].words.len(), 3);
        assert_eq!(all[1].words, ["ale", "lea"]);
    }
}
