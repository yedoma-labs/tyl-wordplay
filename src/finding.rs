//! The common result type: every form can explain what it found.

use std::fmt;

/// The kinds of wordplay this crate knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Form {
    /// A word with a word inside whose removal leaves a word (`carton` = c·**art**·on → `con`).
    Nest,
    /// Letters that read as different words when the spaces move (`nowhere` = `now here`).
    Reparse,
    /// Words that differ only in their vowels (`bat bet bit bot but`).
    VowelWheel,
    /// A word that is valid in two languages (`Gift`).
    Zwitter,
    /// A phrase with one word swapped for a similar-sounding theme word.
    Swap,
    /// A name that contains a blocklisted word across or inside its words.
    Hazard,
    /// Letters rotated cyclically give other words (`tea` `eat` `ate`).
    Carousel,
}

impl Form {
    /// Every form.
    pub const ALL: [Form; 7] = [
        Form::Nest,
        Form::Reparse,
        Form::VowelWheel,
        Form::Zwitter,
        Form::Swap,
        Form::Hazard,
        Form::Carousel,
    ];

    /// English name.
    pub fn name(self) -> &'static str {
        match self {
            Form::Nest => "nest",
            Form::Reparse => "reparse",
            Form::VowelWheel => "vowel wheel",
            Form::Zwitter => "zwitter",
            Form::Swap => "swap",
            Form::Hazard => "hazard",
            Form::Carousel => "carousel",
        }
    }

    /// German name.
    pub fn german(self) -> &'static str {
        match self {
            Form::Nest => "Schachtelwort",
            Form::Reparse => "Neuschnitt",
            Form::VowelWheel => "Vokalrad",
            Form::Zwitter => "Zwitterwort",
            Form::Swap => "Austauschpointe",
            Form::Hazard => "Stolperwort",
            Form::Carousel => "Karussellwort",
        }
    }
}

impl fmt::Display for Form {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One piece of wordplay with the evidence for it.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    /// Which form this is.
    pub form: Form,
    /// The wordplay itself, as it would be shown.
    pub text: String,
    /// The words the wordplay is made of.
    pub parts: Vec<String>,
    /// Heuristic quality in `0.0..=1.0` (higher is better); only comparable within a form.
    pub score: f32,
    /// A sentence saying why this counts.
    pub explanation: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {} - {}", self.form, self.text, self.explanation)
    }
}
