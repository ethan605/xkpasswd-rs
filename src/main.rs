mod prelude;
pub mod settings;

use prelude::*;
use settings::*;

fn main() {
    let pass_generator = Xkpasswd::new();
    let settings = custom_settings().expect("Invalid settings");
    println!("Custom: {}", pass_generator.gen_pass(&settings));

    for preset in [
        Preset::AppleID,
        Preset::Default,
        Preset::WindowsNTLMv1,
        Preset::SecurityQuestions,
        Preset::Web16,
        Preset::Web32,
        Preset::Wifi,
        Preset::XKCD,
    ] {
        let settings = Settings::from_preset(preset);
        println!("{:?}: {}", preset, pass_generator.gen_pass(&settings));
    }
}

fn custom_settings() -> Result<Settings, &'static str> {
    Settings::default()
        .with_words_count(3)?
        .with_word_lengths(4, 8)?
        .with_separators(".")
        .with_padding_digits(0, 2)
        .with_padding_symbols("!@#$%^&*-_=+:|~?/;")
        .with_padding_symbol_lengths(0, 2)
        .with_padding_strategy(PaddingStrategy::Fixed)?
        .with_word_transforms(WordTransform::LOWERCASE | WordTransform::UPPERCASE)
}
