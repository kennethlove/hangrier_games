use rand::prelude::*;

use std::cell::RefCell;

/// Base witty phrase generator struct
///
/// Make a new generator using the default wordlists with new().
pub struct WPGen {
    rng: RefCell<ThreadRng>,
    words_intensifiers: Vec<&'static str>,
    words_adjectives: Vec<&'static str>,
    words_nouns: Vec<&'static str>,
}

impl WPGen {
    pub fn new() -> WPGen {
        let words_intensifiers = include_str!("intensifiers.txt");
        let words_adjectives = include_str!("adjectives.txt");
        let words_nouns = include_str!("nouns.txt");

        let words_intensifiers = words_intensifiers.lines().collect::<Vec<&'static str>>();
        let words_adjectives = words_adjectives.lines().collect::<Vec<&'static str>>();
        let words_nouns = words_nouns.lines().collect::<Vec<&'static str>>();

        WPGen {
            rng: RefCell::new(ThreadRng::default()),
            words_intensifiers,
            words_adjectives,
            words_nouns,
        }
    }

    /// Generate a witty phrase with either 1, 2, or 3 words
    ///
    /// returns None when no phrase could be generated (eg. if one of the wordlists is empty)
    pub fn with_words(&self, words: usize) -> Option<Vec<&'static str>> {
        let mut ret = vec![""; words];
        let mut n = 0;

        if words > 3 {
            ret[3] = self
                .words_nouns
                .iter()
                .choose(&mut *self.rng.borrow_mut())?;
        }

        if words > 2 {
            ret[n] = self
                .words_intensifiers
                .iter()
                .choose(&mut *self.rng.borrow_mut())?;
            n += 1;
        }
        if words > 1 {
            ret[n] = self
                .words_adjectives
                .iter()
                .choose(&mut *self.rng.borrow_mut())?;
            n += 1;
        }
        if words > 0 {
            ret[n] = self
                .words_nouns
                .iter()
                .choose(&mut *self.rng.borrow_mut())?;
        }

        Some(ret)
    }
}
