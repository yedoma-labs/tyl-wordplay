//! Austauschpointe: swap one word of a phrase for a similar-sounding theme word.
//!
//! `a rolling stone` + theme `stove` → `a rolling stove`. The candidates are ranked by a
//! coarse, language-agnostic sound distance (not real phonetics), and only words with the
//! same number of syllables are offered by default, so the rhythm of the phrase survives.

use crate::finding::{Finding, Form};
use crate::lexicon::Lexicon;
use crate::text::{fold, is_vowel};

/// Limits for [`swaps`].
#[derive(Debug, Clone)]
pub struct SwapConfig {
    /// Largest sound distance (edits) between the original and the theme word.
    pub max_distance: usize,
    /// Shortest word of the phrase that may be replaced.
    pub min_len: usize,
    /// Only offer theme words with as many syllables as the replaced word.
    pub keep_syllables: bool,
    /// How many puns to return.
    pub max_results: usize,
}

impl Default for SwapConfig {
    fn default() -> Self {
        Self {
            max_distance: 2,
            min_len: 3,
            keep_syllables: true,
            max_results: 25,
        }
    }
}

/// A phrase with one word replaced by a theme word.
#[derive(Debug, Clone, PartialEq)]
pub struct Swap {
    /// The phrase as given.
    pub original: String,
    /// The phrase with the swap, punctuation and capitalisation kept.
    pub phrase: String,
    /// The word that was replaced (folded).
    pub replaced: String,
    /// The theme word that took its place.
    pub with: String,
    /// Sound distance between the two, in edits.
    pub distance: usize,
    /// Higher is better; also rewards a shared first and last letter.
    pub score: f32,
}

impl From<&Swap> for Finding {
    fn from(s: &Swap) -> Self {
        Finding {
            form: Form::Swap,
            text: s.phrase.clone(),
            parts: vec![s.replaced.clone(), s.with.clone()],
            score: s.score.min(1.0),
            explanation: format!(
                "\"{}\" becomes \"{}\" ({} {} apart in sound)",
                s.replaced,
                s.with,
                s.distance,
                if s.distance == 1 { "edit" } else { "edits" }
            ),
        }
    }
}

/// Coarse sound folding: `ph`→`f`, `ck`→`k`, `qu`→`kw`, `x`→`ks`, `c`→`k`, `v`→`f`,
/// `z`→`s`, `y`→`i`, umlauts to their base vowel, doubled letters collapsed.
fn sound(word: &str) -> Vec<char> {
    let s = word
        .replace("ph", "f")
        .replace("ck", "k")
        .replace("qu", "kw")
        .replace('x', "ks")
        .replace('c', "k")
        .replace('v', "f")
        .replace('z', "s")
        .replace('y', "i")
        .replace('ä', "e")
        .replace('ö', "o")
        .replace('ü', "u");
    let mut out: Vec<char> = Vec::with_capacity(s.len());
    for c in s.chars() {
        if out.last() != Some(&c) {
            out.push(c);
        }
    }
    out
}

/// Optimal string alignment distance (insert, delete, substitute, swap neighbours).
fn distance(a: &[char], b: &[char]) -> usize {
    let (n, m) = (a.len(), b.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[n][m]
}

/// Vowel groups, counting `y`: a rough syllable count.
fn syllables(word: &str) -> usize {
    let mut count = 0;
    let mut prev = false;
    for c in word.chars() {
        let v = is_vowel(c) || c == 'y';
        if v && !prev {
            count += 1;
        }
        prev = v;
    }
    count.max(1)
}

fn match_case(raw: &str, word: &str) -> String {
    let letters: Vec<char> = raw.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.len() > 1 && letters.iter().all(|c| c.is_uppercase()) {
        word.to_uppercase()
    } else if letters.first().is_some_and(|c| c.is_uppercase()) {
        let mut cs = word.chars();
        cs.next()
            .map(|f| f.to_uppercase().chain(cs).collect())
            .unwrap_or_default()
    } else {
        word.to_string()
    }
}

/// Puns on `phrase` made from `theme`, best first.
pub fn swaps<T: Lexicon + ?Sized>(theme: &T, phrase: &str, cfg: &SwapConfig) -> Vec<Swap> {
    // (is_word, raw text) runs, so punctuation and spacing survive the swap
    let mut runs: Vec<(bool, String)> = Vec::new();
    for c in phrase.chars() {
        let w = c.is_alphabetic();
        match runs.last_mut() {
            Some((is_word, s)) if *is_word == w => s.push(c),
            _ => runs.push((w, c.to_string())),
        }
    }
    let mut out = Vec::new();
    for (idx, (is_word, raw)) in runs.iter().enumerate() {
        let word = fold(raw);
        if !is_word || word.chars().count() < cfg.min_len {
            continue;
        }
        let sw = sound(&word);
        for cand in theme.words() {
            if cand == word {
                continue;
            }
            if cfg.keep_syllables && syllables(cand) != syllables(&word) {
                continue;
            }
            let sc = sound(cand);
            let d = distance(&sw, &sc);
            let longer = sw.len().max(sc.len());
            if d > cfg.max_distance || d * 2 > longer {
                continue;
            }
            let mut score = 1.0 - d as f32 / longer as f32;
            if word.chars().next() == cand.chars().next() {
                score += 0.1;
            }
            if word.chars().last() == cand.chars().last() {
                score += 0.1;
            }
            let swapped: String = runs
                .iter()
                .enumerate()
                .map(|(i, (_, r))| {
                    if i == idx {
                        match_case(raw, cand)
                    } else {
                        r.clone()
                    }
                })
                .collect();
            out.push(Swap {
                original: phrase.to_string(),
                phrase: swapped,
                replaced: word.clone(),
                with: cand.to_string(),
                distance: d,
                score,
            });
        }
    }
    out.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.phrase.cmp(&b.phrase))
    });
    out.dedup_by(|a, b| a.phrase == b.phrase);
    out.truncate(cfg.max_results);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::WordSet;

    #[test]
    fn stone_becomes_stove_and_case_survives() {
        let theme = WordSet::from_words(["stove", "kettle"]);
        let s = swaps(&theme, "A Rolling Stone!", &SwapConfig::default());
        assert_eq!(s[0].phrase, "A Rolling Stove!");
        assert_eq!(s[0].distance, 1);
    }

    #[test]
    fn rhythm_is_kept_unless_asked_otherwise() {
        let theme = WordSet::from_words(["plant"]);
        assert!(swaps(&theme, "planet", &SwapConfig::default()).is_empty());
        let loose = SwapConfig {
            keep_syllables: false,
            ..Default::default()
        };
        assert_eq!(swaps(&theme, "planet", &loose)[0].phrase, "plant");
    }

    #[test]
    fn sound_folding_makes_spelling_variants_close() {
        let theme = WordSet::from_words(["fone"]);
        let s = swaps(&theme, "phone", &SwapConfig::default());
        assert_eq!(s[0].distance, 0);
    }

    #[test]
    fn upper_case_is_kept() {
        assert_eq!(match_case("STONE", "stove"), "STOVE");
        assert_eq!(match_case("stone", "stove"), "stove");
    }

    #[test]
    fn far_words_are_not_offered() {
        let theme = WordSet::from_words(["banana"]);
        assert!(swaps(&theme, "stone", &SwapConfig::default()).is_empty());
    }

    #[test]
    fn transposition_costs_one() {
        let a: Vec<char> = "form".chars().collect();
        let b: Vec<char> = "from".chars().collect();
        assert_eq!(distance(&a, &b), 1);
    }
}
