//! Schachtelwort: a word that holds a word, and what is left is a word too.
//!
//! `carton` = c·**art**·on, and `c`+`on` is `con`. `Wunder` = W·**und**·er, leaving `Wer`.
//! Both the inner word and the host that remains when it is lifted out must be words, and
//! the inner word must sit strictly inside (not at either end).

use crate::finding::{Finding, Form};
use crate::lexicon::Lexicon;
use crate::text::{collect, fold};

/// Limits for the nest functions.
#[derive(Debug, Clone)]
pub struct NestConfig {
    /// Shortest inner word.
    pub min_inner: usize,
    /// Shortest host (the letters left over).
    pub min_host: usize,
}

impl Default for NestConfig {
    fn default() -> Self {
        Self {
            min_inner: 3,
            min_host: 3,
        }
    }
}

/// `outer` = the first `at` letters of `host`, then `inner`, then the rest of `host`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nest {
    /// The whole word.
    pub outer: String,
    /// The word inside.
    pub inner: String,
    /// What remains without the inner word.
    pub host: String,
    /// Letter offset in `host` where `inner` was inserted.
    pub at: usize,
}

impl Nest {
    /// How balanced the nest is: the shorter of inner and host relative to the outer word.
    pub fn score(&self) -> f32 {
        let (i, h) = (self.inner.chars().count(), self.host.chars().count());
        i.min(h) as f32 / (i + h) as f32
    }
}

impl From<&Nest> for Finding {
    fn from(n: &Nest) -> Self {
        let head: String = n.host.chars().take(n.at).collect();
        let tail: String = n.host.chars().skip(n.at).collect();
        Finding {
            form: Form::Nest,
            text: format!("{head}({}){tail}", n.inner),
            parts: vec![n.outer.clone(), n.inner.clone(), n.host.clone()],
            score: n.score(),
            explanation: format!(
                "\"{}\" holds \"{}\"; without it \"{}\" remains",
                n.outer, n.inner, n.host
            ),
        }
    }
}

/// All nests of `word`, if `word` is in `lex`.
pub fn nests_of<L: Lexicon + ?Sized>(lex: &L, word: &str, cfg: &NestConfig) -> Vec<Nest> {
    let outer = fold(word);
    if !lex.has(&outer) {
        return Vec::new();
    }
    let w: Vec<char> = outer.chars().collect();
    let mut out = Vec::new();
    for i in 1..w.len() {
        for j in i + cfg.min_inner.max(1)..w.len() {
            let inner = collect(&w[i..j]);
            let mut rest = w[..i].to_vec();
            rest.extend_from_slice(&w[j..]);
            if rest.len() >= cfg.min_host && lex.has(&inner) && lex.has(&collect(&rest)) {
                out.push(Nest {
                    outer: outer.clone(),
                    inner,
                    host: collect(&rest),
                    at: i,
                });
            }
        }
    }
    out
}

/// Every nest in the lexicon, best first.
pub fn all_nests<L: Lexicon + ?Sized>(lex: &L, cfg: &NestConfig) -> Vec<Nest> {
    let mut out: Vec<Nest> = lex.words().flat_map(|w| nests_of(lex, w, cfg)).collect();
    out.sort_by(|a, b| {
        b.score()
            .partial_cmp(&a.score())
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.cmp(b))
    });
    out
}

/// Builds nests the other way round: every place where `inner` can be pushed into `host`
/// so that the result is a word (`con` + `art` → `carton`).
pub fn insert_everywhere<L: Lexicon + ?Sized>(lex: &L, host: &str, inner: &str) -> Vec<Nest> {
    let (host, inner) = (fold(host), fold(inner));
    let h: Vec<char> = host.chars().collect();
    (1..h.len())
        .filter_map(|at| {
            let outer = format!("{}{}{}", collect(&h[..at]), inner, collect(&h[at..]));
            lex.has(&outer).then(|| Nest {
                outer,
                inner: inner.clone(),
                host: host.clone(),
                at,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::WordSet;

    fn lex() -> WordSet {
        WordSet::from_words(["carton", "art", "con", "car", "ton", "wunder", "und", "wer"])
    }

    #[test]
    fn carton_holds_art_and_leaves_con() {
        let n = nests_of(&lex(), "Carton", &NestConfig::default());
        assert_eq!(n.len(), 1);
        assert_eq!(
            (n[0].inner.as_str(), n[0].host.as_str(), n[0].at),
            ("art", "con", 1)
        );
    }

    #[test]
    fn german_nest() {
        let n = nests_of(&lex(), "Wunder", &NestConfig::default());
        assert_eq!(n[0].inner, "und");
        assert_eq!(n[0].host, "wer");
    }

    #[test]
    fn insert_is_the_inverse() {
        let n = insert_everywhere(&lex(), "con", "art");
        assert_eq!(n.len(), 1);
        assert_eq!(n[0].outer, "carton");
    }

    #[test]
    fn all_nests_finds_both_languages() {
        let outers: Vec<_> = all_nests(&lex(), &NestConfig::default())
            .into_iter()
            .map(|n| n.outer)
            .collect();
        assert!(outers.contains(&"carton".to_string()) && outers.contains(&"wunder".to_string()));
    }

    #[test]
    fn unknown_words_have_no_nests() {
        assert!(nests_of(&lex(), "zzz", &NestConfig::default()).is_empty());
    }

    #[test]
    fn finding_shows_the_bracket() {
        let n = nests_of(&lex(), "carton", &NestConfig::default());
        assert_eq!(Finding::from(&n[0]).text, "c(art)on");
    }
}
