//! Word lists: the only data the forms need.

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use crate::text::fold;

/// A set of words the forms can ask "is this a word?".
///
/// Implement it for your own storage (an FST, a database, a spell checker). All words
/// passed to [`has`](Lexicon::has) are already [folded](crate::text::fold).
pub trait Lexicon {
    /// Whether the folded `word` is in the lexicon.
    fn has(&self, word: &str) -> bool;
    /// Length in characters of the longest word (bounds the search windows).
    fn max_word_len(&self) -> usize;
    /// All words, in a stable order (results of the forms are deterministic).
    fn words(&self) -> Box<dyn Iterator<Item = &str> + '_>;
}

/// An in-memory [`Lexicon`] with optional glosses (a short meaning per word).
///
/// Load a plain list with [`from_lines`](WordSet::from_lines) or
/// [`from_file`](WordSet::from_file) (for example `/usr/share/dict/words`).
#[derive(Debug, Clone, Default)]
pub struct WordSet {
    entries: BTreeMap<String, String>,
    max_len: usize,
}

impl WordSet {
    /// An empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a set from words.
    pub fn from_words<I, S>(words: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut set = Self::new();
        for w in words {
            set.insert(w.as_ref());
        }
        set
    }

    /// Parses one entry per line: `word` or `word<TAB>gloss`. Blank lines, lines starting
    /// with `#` and entries that are not a single word are skipped.
    pub fn from_lines(text: &str) -> Self {
        let mut set = Self::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (word, gloss) = line.split_once('\t').unwrap_or((line, ""));
            if word.split_whitespace().count() != 1 {
                continue;
            }
            set.insert_with_gloss(word, gloss.trim());
        }
        set
    }

    /// Reads a word list file, see [`from_lines`](WordSet::from_lines).
    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self::from_lines(&std::fs::read_to_string(path)?))
    }

    /// Adds a word (folded). Words without letters are ignored.
    pub fn insert(&mut self, word: &str) {
        self.insert_with_gloss(word, "");
    }

    /// Adds a word with a short meaning.
    pub fn insert_with_gloss(&mut self, word: &str, gloss: &str) {
        let word = fold(word);
        if word.is_empty() {
            return;
        }
        self.max_len = self.max_len.max(word.chars().count());
        let entry = self.entries.entry(word).or_default();
        if !gloss.is_empty() {
            *entry = gloss.to_string();
        }
    }

    /// Whether `word` (folded first) is in the set.
    pub fn contains(&self, word: &str) -> bool {
        self.has(&fold(word))
    }

    /// The gloss of `word`, if it has one.
    pub fn gloss(&self, word: &str) -> Option<&str> {
        self.entries
            .get(&fold(word))
            .map(String::as_str)
            .filter(|g| !g.is_empty())
    }

    /// Number of words.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the set has no words.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Lexicon for WordSet {
    fn has(&self, word: &str) -> bool {
        self.entries.contains_key(word)
    }

    fn max_word_len(&self) -> usize {
        self.max_len
    }

    fn words(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(self.entries.keys().map(String::as_str))
    }
}

impl<S: AsRef<str>> FromIterator<S> for WordSet {
    fn from_iter<I: IntoIterator<Item = S>>(iter: I) -> Self {
        Self::from_words(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_with_glosses_comments_and_phrases() {
        let set = WordSet::from_lines("# c\nGift\tpoison\n\nice cream\nrat\n");
        assert_eq!(set.len(), 2);
        assert_eq!(set.gloss("gift"), Some("poison"));
        assert_eq!(set.gloss("rat"), None);
        assert!(set.contains("GIFT"));
        assert_eq!(set.max_word_len(), 4);
    }
}
