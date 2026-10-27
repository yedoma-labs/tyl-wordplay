//! Tiny demo word lists (a few hundred common words), enough for the documentation, the
//! examples and the tests. Use a real dictionary for real work, for example
//! `WordSet::from_file("/usr/share/dict/words")`.

use crate::lexicon::WordSet;

/// A few hundred English words, some with German glosses for the [zwitter](crate::zwitter) form.
pub fn english() -> WordSet {
    WordSet::from_lines(include_str!("../data/en.txt"))
}

/// A few hundred German words, some with English glosses.
pub fn german() -> WordSet {
    WordSet::from_lines(include_str!("../data/de.txt"))
}

/// A short English blocklist for the [hazard](crate::hazard) demo.
pub fn blocklist_en() -> WordSet {
    WordSet::from_lines(include_str!("../data/blocklist-en.txt"))
}
