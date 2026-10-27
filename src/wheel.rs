//! Vokalrad: words that differ only in their vowels.
//!
//! Replace every vowel by a blank and words with the same skeleton form a wheel:
//! `b_t` turns through `bat bet bit bot but`. A wheel with all five vowels is *full*.
//! Vowels are `a e i o u ä ö ü` (`y` counts as a consonant here).

use std::collections::BTreeMap;

use crate::finding::{Finding, Form};
use crate::lexicon::Lexicon;
use crate::text::{fold, is_vowel};

/// A family of words that share a vowel-less skeleton.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wheel {
    /// The word with every vowel replaced by `_`.
    pub skeleton: String,
    /// The words, sorted.
    pub members: Vec<String>,
}

impl Wheel {
    /// One blank that takes all of `a e i o u`.
    pub fn is_full(&self) -> bool {
        self.skeleton.matches('_').count() == 1
            && "aeiou".chars().all(|v| {
                let w = self.skeleton.replacen('_', &v.to_string(), 1);
                self.members.contains(&w)
            })
    }
}

impl From<&Wheel> for Finding {
    fn from(w: &Wheel) -> Self {
        Finding {
            form: Form::VowelWheel,
            text: w.members.join(" "),
            parts: w.members.clone(),
            score: (w.members.len() as f32 / 5.0).min(1.0),
            explanation: format!(
                "{} words share the skeleton \"{}\"{}",
                w.members.len(),
                w.skeleton,
                if w.is_full() {
                    " and turn through all five vowels"
                } else {
                    ""
                }
            ),
        }
    }
}

/// The skeleton of `word`: every vowel becomes `_`.
pub fn skeleton(word: &str) -> String {
    fold(word)
        .chars()
        .map(|c| if is_vowel(c) { '_' } else { c })
        .collect()
}

/// All wheels with at least `min_size` members, biggest first.
pub fn wheels<L: Lexicon + ?Sized>(lex: &L, min_size: usize) -> Vec<Wheel> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for w in lex.words() {
        let s = skeleton(w);
        if s.contains('_') {
            groups.entry(s).or_default().push(w.to_string());
        }
    }
    let mut out: Vec<Wheel> = groups
        .into_iter()
        .filter(|(_, m)| m.len() >= min_size.max(2))
        .map(|(skeleton, members)| Wheel { skeleton, members })
        .collect();
    out.sort_by(|a, b| {
        b.members
            .len()
            .cmp(&a.members.len())
            .then_with(|| a.skeleton.cmp(&b.skeleton))
    });
    out
}

/// The wheel `word` belongs to, if it has at least one sibling.
pub fn wheel_of<L: Lexicon + ?Sized>(lex: &L, word: &str) -> Option<Wheel> {
    let word = fold(word);
    if !lex.has(&word) {
        return None;
    }
    let target = skeleton(&word);
    let members: Vec<String> = lex
        .words()
        .filter(|w| w.chars().count() == word.chars().count() && skeleton(w) == target)
        .map(str::to_string)
        .collect();
    (members.len() >= 2).then_some(Wheel {
        skeleton: target,
        members,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::WordSet;

    fn lex() -> WordSet {
        WordSet::from_words(["bat", "bet", "bit", "bot", "but", "pan", "pen", "tea"])
    }

    #[test]
    fn b_t_is_a_full_wheel() {
        let w = wheel_of(&lex(), "bit").unwrap();
        assert_eq!(w.skeleton, "b_t");
        assert_eq!(w.members, ["bat", "bet", "bit", "bot", "but"]);
        assert!(w.is_full());
    }

    #[test]
    fn small_wheel_is_not_full() {
        let w = wheel_of(&lex(), "pan").unwrap();
        assert_eq!(w.members, ["pan", "pen"]);
        assert!(!w.is_full());
    }

    #[test]
    fn wheels_filters_by_size() {
        let all = wheels(&lex(), 4);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].skeleton, "b_t");
    }

    #[test]
    fn a_loner_has_no_wheel() {
        assert!(wheel_of(&lex(), "tea").is_none());
    }
}
