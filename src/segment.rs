//! Splitting a run of letters into dictionary words.

use crate::text::collect;

pub(crate) struct Segmenter<'a> {
    pub has: &'a dyn Fn(&str) -> bool,
    pub min_word: usize,
    pub short: &'a [String],
    pub max_len: usize,
}

impl Segmenter<'_> {
    fn word_ok(&self, part: &[char]) -> bool {
        let n = part.len();
        if n == 0 || n > self.max_len.max(2) {
            return false;
        }
        let word = collect(part);
        if n < self.min_word {
            return self.short.contains(&word);
        }
        (self.has)(&word)
    }

    /// `ok[i]`: the letters from `i` on can be split into words.
    fn reachable(&self, chars: &[char]) -> Vec<bool> {
        let n = chars.len();
        let mut ok = vec![false; n + 1];
        ok[n] = true;
        for i in (0..n).rev() {
            ok[i] = (i + 1..=n.min(i + self.max_len)).any(|j| ok[j] && self.word_ok(&chars[i..j]));
        }
        ok
    }

    /// One split with as few words as the greedy longest-first walk finds; `Some(vec![])` for no letters.
    pub fn one(&self, chars: &[char]) -> Option<Vec<String>> {
        let ok = self.reachable(chars);
        if !ok[0] {
            return None;
        }
        let n = chars.len();
        let (mut i, mut out) = (0, Vec::new());
        while i < n {
            let j = (i + 1..=n.min(i + self.max_len))
                .rev()
                .find(|&j| ok[j] && self.word_ok(&chars[i..j]))?;
            out.push(collect(&chars[i..j]));
            i = j;
        }
        Some(out)
    }

    /// Up to `cap` different splits.
    pub fn all(&self, chars: &[char], cap: usize) -> Vec<Vec<String>> {
        let ok = self.reachable(chars);
        let mut out = Vec::new();
        if ok[0] {
            self.walk(chars, &ok, 0, &mut Vec::new(), &mut out, cap);
        }
        out
    }

    fn walk(
        &self,
        chars: &[char],
        ok: &[bool],
        i: usize,
        path: &mut Vec<String>,
        out: &mut Vec<Vec<String>>,
        cap: usize,
    ) {
        if out.len() >= cap {
            return;
        }
        if i == chars.len() {
            out.push(path.clone());
            return;
        }
        for j in i + 1..=chars.len().min(i + self.max_len) {
            if ok[j] && self.word_ok(&chars[i..j]) {
                path.push(collect(&chars[i..j]));
                self.walk(chars, ok, j, path, out, cap);
                path.pop();
            }
        }
    }
}
