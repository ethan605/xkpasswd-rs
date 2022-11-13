use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub const DEFAULT_WORDS_COUNT: u8 = 3;
pub const DEFAULT_DIGITS_COUNT: u8 = 3;
pub const DEFAULT_WORD_LENGTHS: (u8, u8) = (4, 10);
pub const DEFAULT_SYMBOLS: &str = "!@#$%^&*-_=+:|~?/;";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub words_count: u8,
    pub word_lengths: (u8, u8),
    separators: Vec<char>,
    padding_digits: (u8, u8),
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            words_count: DEFAULT_WORDS_COUNT,
            word_lengths: DEFAULT_WORD_LENGTHS,
            separators: DEFAULT_SYMBOLS.chars().collect(),
            padding_digits: (0, DEFAULT_DIGITS_COUNT),
        }
    }
}

impl Settings {
    pub fn words_count(&self, words_count: u8) -> Settings {
        Settings {
            words_count,
            separators: self.separators.clone(),
            ..(*self)
        }
    }

    pub fn word_lengths(&self, min_length: u8, max_length: u8) -> Settings {
        let word_lengths = if min_length > max_length {
            (max_length, min_length)
        } else {
            (min_length, max_length)
        };

        Settings {
            word_lengths,
            separators: self.separators.clone(),
            ..(*self)
        }
    }

    pub fn separators(&self, separators: &str) -> Settings {
        Settings {
            separators: separators.chars().collect(),
            ..(*self)
        }
    }

    pub fn padding_digits(&self, prefix: u8, suffix: u8) -> Settings {
        Settings {
            separators: self.separators.clone(),
            padding_digits: (prefix, suffix),
            ..(*self)
        }
    }

    pub fn rand_separator(&self) -> char {
        if self.separators.is_empty() {
            return '\0';
        }

        let len = self.separators.len();
        let mut rng = rand::thread_rng();
        self.separators[rng.gen_range(0..len)]
    }

    pub fn rand_prefix(&self) -> String {
        let (prefix, _) = self.padding_digits;
        self.rand_digits(prefix)
    }

    pub fn rand_suffix(&self) -> String {
        let (_, suffix) = self.padding_digits;
        self.rand_digits(suffix)
    }

    fn rand_digits(&self, count: u8) -> String {
        if count == 0 {
            return "".to_string();
        }

        let mut rng = rand::thread_rng();
        let padding_digits: u8 = Uniform::from(10..100).sample(&mut rng);
        padding_digits.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();

        assert_eq!(DEFAULT_WORDS_COUNT, settings.words_count);
        assert_eq!(DEFAULT_WORD_LENGTHS, settings.word_lengths);
        assert_eq!(
            DEFAULT_SYMBOLS.chars().collect::<Vec<char>>(),
            settings.separators
        );
        assert_eq!((0, DEFAULT_DIGITS_COUNT), settings.padding_digits);
    }

    #[test]
    fn test_words_count_builder() {
        let settings = Settings::default().words_count(1);
        // only words_count updated
        assert_eq!(1, settings.words_count);

        // other fields remain unchanged
        assert_eq!(DEFAULT_WORD_LENGTHS, settings.word_lengths);
        assert_eq!(
            DEFAULT_SYMBOLS.chars().collect::<Vec<char>>(),
            settings.separators
        );
        assert_eq!((0, DEFAULT_DIGITS_COUNT), settings.padding_digits);

        let settings = Settings::default().words_count(123);
        assert_eq!(123, settings.words_count);
    }

    #[test]
    fn test_word_lengths_builder() {
        let settings = Settings::default().word_lengths(2, 3);
        // only word_lengths updated
        assert_eq!((2, 3), settings.word_lengths);

        // other fields remain unchanged
        assert_eq!(DEFAULT_WORDS_COUNT, settings.words_count);
        assert_eq!(
            DEFAULT_SYMBOLS.chars().collect::<Vec<char>>(),
            settings.separators
        );
        assert_eq!((0, DEFAULT_DIGITS_COUNT), settings.padding_digits);

        let settings = Settings::default().word_lengths(5, 5);
        assert_eq!((5, 5), settings.word_lengths); // equal values

        let settings = Settings::default().word_lengths(6, 4);
        assert_eq!((4, 6), settings.word_lengths); // min/max corrected
    }

    #[test]
    fn test_separators_builder() {
        let settings = Settings::default().separators("abc123");
        // only separators updated
        assert_eq!(vec!['a', 'b', 'c', '1', '2', '3'], settings.separators);

        // other fields remain unchanged
        assert_eq!(DEFAULT_WORDS_COUNT, settings.words_count);
        assert_eq!(DEFAULT_WORD_LENGTHS, settings.word_lengths);
        assert_eq!((0, DEFAULT_DIGITS_COUNT), settings.padding_digits);

        let settings = Settings::default().separators("");
        assert_eq!(vec![] as Vec<char>, settings.separators);
    }

    #[test]
    fn test_rand_separator() {
        let symbols = "abc123";
        let settings = Settings::default().separators(symbols);
        let separator_chars = symbols.chars().collect::<Vec<char>>();

        for _ in 1..10 {
            let separator = settings.rand_separator();
            assert_eq!(true, separator_chars.contains(&separator));
        }

        // Empty separators list
        let settings = Settings::default().separators("");

        for _ in 1..10 {
            let separator = settings.rand_separator();
            assert_eq!('\0', separator);
        }
    }

    #[test]
    fn test_padding_digits_builder() {
        let settings = Settings::default().padding_digits(1, 3);
        // only padding_digits updated
        assert_eq!((1, 3), settings.padding_digits);

        // other fields remain unchanged
        assert_eq!(DEFAULT_WORDS_COUNT, settings.words_count);
        assert_eq!(DEFAULT_WORD_LENGTHS, settings.word_lengths);
        assert_eq!(
            DEFAULT_SYMBOLS.chars().collect::<Vec<char>>(),
            settings.separators
        );

        let settings = Settings::default().padding_digits(0, 0);
        assert_eq!((0, 0), settings.padding_digits);
    }
}
