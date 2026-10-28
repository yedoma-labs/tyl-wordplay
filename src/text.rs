//! Normalisation shared by every form.

/// Lower-cases `word`, expands `ß` to `ss` and drops everything that is not a letter.
///
/// Umlauts stay distinct letters (`ä` is not `a`), so German and English words can share
/// one pipeline.
pub fn fold(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    for c in word.chars() {
        // `ẞ` (U+1E9E) lowercases to `ß`, so it must be expanded like `ß` itself.
        if c == 'ß' || c == 'ẞ' {
            out.push_str("ss");
        } else if c.is_alphabetic() {
            out.extend(c.to_lowercase());
        }
    }
    out
}

/// Splits `text` into folded words at everything that is not a letter.
pub fn tokens(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphabetic())
        .map(fold)
        .filter(|t| !t.is_empty())
        .collect()
}

/// Common English words of one or two letters. Dictionaries are full of fragments such as
/// `th` or `ra`, so readings only accept words shorter than three letters from such a list.
pub const SHORT_WORDS_EN: &[&str] = &[
    "a", "i", "am", "an", "as", "at", "be", "by", "do", "go", "he", "if", "in", "is", "it", "me",
    "my", "no", "of", "on", "or", "so", "to", "up", "us", "we",
];

/// Common German words of one or two letters, see [`SHORT_WORDS_EN`].
pub const SHORT_WORDS_DE: &[&str] = &[
    "ab", "am", "an", "da", "du", "er", "es", "im", "in", "ja", "ob", "so", "um", "zu",
];

pub(crate) fn short_words(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

pub(crate) fn collect(chars: &[char]) -> String {
    chars.iter().collect()
}

pub(crate) fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'ä' | 'ö' | 'ü')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_case_eszett_and_punctuation() {
        assert_eq!(fold("Straße!"), "strasse");
        assert_eq!(fold("Ärger"), "ärger");
    }

    #[test]
    fn capital_sharp_s_folds_like_eszett() {
        // U+1E9E (capital sharp s) must not survive as "ß": "STRAẞE" is "strasse"
        assert_eq!(fold("STRAẞE"), "strasse");
        assert_eq!(fold("straße"), fold("STRAẞE"));
    }

    #[test]
    fn tokens_split_on_non_letters() {
        assert_eq!(tokens("Ice-cream, 2 go"), ["ice", "cream", "go"]);
    }
}
