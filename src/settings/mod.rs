#[cfg(test)]
mod tests;

use crate::bit_flags::{BitFlags, FieldSize, WordTransform};
use crate::prelude::{Builder, PaddingResult, PaddingStrategy, Preset, Randomizer};
use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use std::cmp;
use std::collections::HashMap;
use std::ops::Range;
use std::result::Result;

const MIN_WORD_LENGTH_ERR: &str = "min word length must be 4 or higher";
const MAX_WORD_LENGTH_ERR: &str = "max word length must be 10 or lower";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    words_count: u8,
    word_lengths: (u8, u8),
    word_transforms: u8,
    separators: String,
    padding_digits: (u8, u8),
    padding_symbols: String,
    padding_symbol_lengths: (u8, u8),
    padding_strategy: PaddingStrategy,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            words_count: Self::DEFAULT_WORDS_COUNT,
            word_lengths: Self::DEFAULT_WORD_LENGTHS,
            word_transforms: Self::DEFAULT_WORD_TRANSFORMS,
            separators: Self::DEFAULT_SEPARATORS.to_string(),
            padding_digits: (0, Self::DEFAULT_PADDING_LENGTH),
            padding_symbols: Self::DEFAULT_SYMBOLS.to_string(),
            padding_symbol_lengths: (0, Self::DEFAULT_PADDING_LENGTH),
            padding_strategy: Self::DEFAULT_PADDING_STRATEGY,
        }
    }
}

impl Builder for Settings {
    fn with_words_count(&self, words_count: u8) -> Result<Self, &'static str> {
        if words_count == 0 {
            return Err("only positive integer is allowed for words count");
        }

        let mut cloned = self.clone();
        cloned.words_count = words_count;
        Ok(cloned)
    }

    fn with_word_lengths(
        &self,
        min_length: Option<u8>,
        max_length: Option<u8>,
    ) -> Result<Settings, &'static str> {
        let min_length = min_length.unwrap_or(self.word_lengths.0);
        let max_length = max_length.unwrap_or(self.word_lengths.1);

        let min = cmp::min(min_length, max_length);
        let max = cmp::max(min_length, max_length);

        if min < Self::MIN_WORD_LENGTH {
            return Err(MIN_WORD_LENGTH_ERR);
        }

        if max > Self::MAX_WORD_LENGTH {
            return Err(MAX_WORD_LENGTH_ERR);
        }

        let mut cloned = self.clone();
        cloned.word_lengths = (min, max);
        Ok(cloned)
    }

    fn with_separators(&self, separators: &str) -> Self {
        let mut cloned = self.clone();
        cloned.separators = separators.to_string();
        cloned
    }

    fn with_padding_digits(&self, prefix: Option<u8>, suffix: Option<u8>) -> Self {
        if prefix.is_none() && suffix.is_none() {
            return self.clone();
        }

        let mut cloned = self.clone();
        cloned.padding_digits = (
            prefix.unwrap_or(self.padding_digits.0),
            suffix.unwrap_or(self.padding_digits.1),
        );
        cloned
    }

    fn with_padding_symbols(&self, symbols: &str) -> Self {
        let mut cloned = self.clone();
        cloned.padding_symbols = symbols.to_string();
        cloned
    }

    fn with_padding_symbol_lengths(&self, prefix: Option<u8>, suffix: Option<u8>) -> Self {
        if prefix.is_none() && suffix.is_none() {
            return self.clone();
        }

        let mut cloned = self.clone();
        cloned.padding_symbol_lengths = (
            prefix.unwrap_or(self.padding_symbol_lengths.0),
            suffix.unwrap_or(self.padding_symbol_lengths.1),
        );
        cloned.padding_strategy = PaddingStrategy::Fixed;
        cloned
    }

    fn with_padding_strategy(&self, strategy: PaddingStrategy) -> Result<Settings, &'static str> {
        match strategy {
            PaddingStrategy::Adaptive(0) => Err("invalid adaptive padding number"),
            _ => {
                let mut cloned = self.clone();
                cloned.padding_strategy = strategy;
                cloned.padding_symbol_lengths = (0, 0);
                Ok(cloned)
            }
        }
    }

    fn with_word_transforms(&self, transforms: FieldSize) -> Result<Self, &'static str> {
        let mut cloned = self.clone();

        // handle group transforms first
        if transforms.has_flag(WordTransform::AltercaseLowerFirst) {
            cloned.word_transforms = WordTransform::AltercaseLowerFirst as FieldSize;
            return Ok(cloned);
        }

        if transforms.has_flag(WordTransform::AltercaseUpperFirst) {
            cloned.word_transforms = WordTransform::AltercaseUpperFirst as FieldSize;
            return Ok(cloned);
        }

        // no transform matched
        if !transforms.has_flag(WordTransform::Lowercase)
            && !transforms.has_flag(WordTransform::Titlecase)
            && !transforms.has_flag(WordTransform::Uppercase)
            && !transforms.has_flag(WordTransform::InversedTitlecase)
        {
            return Err("invalid transform");
        }

        let mut cloned = self.clone();
        cloned.word_transforms = transforms;
        Ok(cloned)
    }

    fn from_preset(preset: Preset) -> Self {
        match preset {
            Preset::AppleID => Settings {
                words_count: 3,
                word_lengths: (5, 7),
                word_transforms: WordTransform::Lowercase | WordTransform::Uppercase,
                separators: "-:.,".to_string(),
                padding_digits: (2, 2),
                padding_symbols: "!?@&".to_string(),
                padding_symbol_lengths: (1, 1),
                padding_strategy: PaddingStrategy::Fixed,
            },
            Preset::WindowsNtlmV1 => Settings {
                words_count: 2,
                word_lengths: (5, 5),
                word_transforms: FieldSize::from_flag(WordTransform::InversedTitlecase),
                separators: "-+=.*_|~,".to_string(),
                padding_digits: (1, 0),
                padding_symbols: "!@$%^&*+=:|~?".to_string(),
                padding_symbol_lengths: (0, 1),
                padding_strategy: PaddingStrategy::Fixed,
            },
            Preset::SecurityQuestions => Settings {
                words_count: 6,
                word_lengths: (4, 8),
                word_transforms: FieldSize::from_flag(WordTransform::Lowercase),
                separators: " ".to_string(),
                padding_digits: (0, 0),
                padding_symbols: ".!?".to_string(),
                padding_symbol_lengths: (0, 1),
                padding_strategy: PaddingStrategy::Fixed,
            },
            Preset::Web16 => Settings {
                words_count: 3,
                word_lengths: (4, 4),
                word_transforms: WordTransform::Lowercase | WordTransform::Uppercase,
                separators: "-+=.*_|~,".to_string(),
                padding_digits: (0, 0),
                padding_symbols: "!@$%^&*+=:|~?".to_string(),
                padding_symbol_lengths: (1, 1),
                padding_strategy: PaddingStrategy::Fixed,
            },
            Preset::Web32 => Settings {
                words_count: 4,
                word_lengths: (4, 5),
                word_transforms: FieldSize::from_flag(WordTransform::AltercaseUpperFirst),
                separators: "-+=.*_|~,".to_string(),
                padding_digits: (2, 2),
                padding_symbols: "!@$%^&*+=:|~?".to_string(),
                padding_symbol_lengths: (1, 1),
                padding_strategy: PaddingStrategy::Fixed,
            },
            Preset::Wifi => Settings {
                words_count: 6,
                word_lengths: (4, 8),
                word_transforms: WordTransform::Lowercase | WordTransform::Uppercase,
                separators: "-+=.*_|~,".to_string(),
                padding_digits: (4, 4),
                padding_symbols: "!@$%^&*+=:|~?".to_string(),
                padding_symbol_lengths: (0, 0),
                padding_strategy: PaddingStrategy::Adaptive(63),
            },
            Preset::Xkcd => Settings {
                words_count: 4,
                word_lengths: (4, 8),
                word_transforms: WordTransform::Lowercase | WordTransform::Uppercase,
                separators: "-".to_string(),
                padding_digits: (0, 0),
                padding_symbols: "".to_string(),
                padding_symbol_lengths: (0, 0),
                padding_strategy: PaddingStrategy::Fixed,
            },
            _ => Self::default(),
        }
    }
}

impl Randomizer for Settings {
    fn word_lengths(&self) -> Range<u8> {
        let (min, max) = self.word_lengths;
        min..(max + 1)
    }

    fn rand_words(&self, pool: &[&str]) -> Vec<String> {
        let words_list = self.build_words_list(pool);
        let transforms_list = self.build_transforms_list();

        words_list
            .iter()
            .zip(transforms_list.iter())
            .map(|(word, &transform)| transform_word(word, transform))
            .collect()
    }

    fn rand_separator(&self) -> String {
        rand_chars(&self.separators, 1)
    }

    fn rand_prefix(&self) -> (String, String) {
        let (prefix_digits, _) = self.padding_digits;
        let (prefix_symbols, _) = self.padding_symbol_lengths;
        (
            rand_chars(&self.padding_symbols, prefix_symbols),
            rand_digits(prefix_digits),
        )
    }

    fn rand_suffix(&self) -> (String, String) {
        let (_, suffix_digits) = self.padding_digits;
        let (_, suffix_symbols) = self.padding_symbol_lengths;
        (
            rand_digits(suffix_digits),
            rand_chars(&self.padding_symbols, suffix_symbols),
        )
    }

    fn adjust_padding(&self, pass_length: usize) -> PaddingResult {
        match self.padding_strategy {
            PaddingStrategy::Fixed => PaddingResult::Unchanged,
            PaddingStrategy::Adaptive(len) => {
                let length = len as usize;

                if length > pass_length {
                    let padded_symbols =
                        rand_chars(&self.padding_symbols, (length - pass_length) as u8);
                    PaddingResult::Pad(padded_symbols)
                } else {
                    PaddingResult::TrimTo(len)
                }
            }
        }
    }
}

impl Settings {
    const MIN_WORD_LENGTH: u8 = 4;
    const MAX_WORD_LENGTH: u8 = 10;
    const DEFAULT_PADDING_LENGTH: u8 = 2;
    const DEFAULT_PADDING_STRATEGY: PaddingStrategy = PaddingStrategy::Fixed;
    const DEFAULT_SEPARATORS: &str = ".-_~";
    const DEFAULT_SYMBOLS: &str = "~@$%^&*-_+=:|~?/.;";
    const DEFAULT_WORDS_COUNT: u8 = 3;
    const DEFAULT_WORD_LENGTHS: (u8, u8) = (Self::MIN_WORD_LENGTH, Self::MAX_WORD_LENGTH);
    const DEFAULT_WORD_TRANSFORMS: FieldSize = 0b00000101; // WordTransform::Lowercase | WordTransform::Uppercase

    const ALL_SINGLE_WORD_TRANSFORMS: [WordTransform; 4] = [
        WordTransform::Lowercase,
        WordTransform::Titlecase,
        WordTransform::Uppercase,
        WordTransform::InversedTitlecase,
    ];

    fn build_words_list<'a>(&self, pool: &[&'a str]) -> Vec<&'a str> {
        if pool.is_empty() {
            return vec![];
        }

        let mut rng = rand::thread_rng();
        let word_indices = Uniform::from(0..pool.len());

        // not enough words to distinguishably randomize
        if pool.len() < self.words_count as usize {
            return (0..self.words_count)
                .map(|_| {
                    let index: usize = word_indices.sample(&mut rng);
                    pool[index]
                })
                .collect();
        }

        // enough words, ensure no duplicates
        let mut index_marker: HashMap<usize, bool> = HashMap::new();
        (0..self.words_count)
            .map(|_| loop {
                let index: usize = word_indices.sample(&mut rng);
                let word = pool[index];

                if index_marker.get(&index).is_none() {
                    index_marker.insert(index, true);
                    break word;
                }
            })
            .collect()
    }

    fn build_transforms_list(&self) -> Vec<WordTransform> {
        if self
            .word_transforms
            .has_flag(WordTransform::AltercaseLowerFirst)
        {
            return (0..self.words_count)
                .map(|idx| {
                    if idx % 2 == 0 {
                        WordTransform::Lowercase
                    } else {
                        WordTransform::Uppercase
                    }
                })
                .collect();
        }

        if self
            .word_transforms
            .has_flag(WordTransform::AltercaseUpperFirst)
        {
            return (0..self.words_count)
                .map(|idx| {
                    if idx % 2 == 0 {
                        WordTransform::Uppercase
                    } else {
                        WordTransform::Lowercase
                    }
                })
                .collect();
        }

        let whitelisted_transforms: Vec<&WordTransform> = Self::ALL_SINGLE_WORD_TRANSFORMS
            .iter()
            .filter(|&&transform| self.word_transforms & transform)
            .collect();

        let mut rng = rand::thread_rng();
        let transform_indices = Uniform::from(0..whitelisted_transforms.len());

        (0..self.words_count)
            .map(|_| {
                let index: usize = transform_indices.sample(&mut rng);
                *whitelisted_transforms[index]
            })
            .collect()
    }
}

fn rand_digits(count: u8) -> String {
    if count == 0 {
        return "".to_string();
    }

    let affordable_count = 20u32.min(count as u32);

    let lower_bound = 10u64.pow(affordable_count - 1);
    let upper_bound = if affordable_count == 20 {
        u64::MAX
    } else {
        10u64.pow(affordable_count)
    };

    let mut rng = rand::thread_rng();
    let padding_digits: u64 = Uniform::from(lower_bound..upper_bound).sample(&mut rng);
    padding_digits.to_string()
}

fn rand_chars(pool: &str, count: u8) -> String {
    if pool.is_empty() {
        return "".to_string();
    }

    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..pool.len());
    pool.chars()
        .nth(idx)
        .unwrap()
        .to_string()
        .repeat(count as _)
}

fn transform_word(word: &str, transform: WordTransform) -> String {
    match transform {
        WordTransform::Titlecase => word[..1].to_uppercase() + &word[1..],
        WordTransform::Uppercase => word.to_uppercase(),
        WordTransform::InversedTitlecase => word[..1].to_lowercase() + &word[1..].to_uppercase(),
        // lowercase by default
        _ => word.to_lowercase(),
    }
}
