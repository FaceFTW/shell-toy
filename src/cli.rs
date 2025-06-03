use argh::FromArgs;

use crate::cowsay::{BubbleType, CowVariant};

#[derive(FromArgs)]
/// various program options
pub(crate) struct Options {
    // #[cfg(not(feature = "inline-cowsay"))]
    #[argh(option, short = 'c')]
    ///path to a direct cowfile OR the name of a cow that exists in the cow path
    pub cow_file: Option<String>,

    #[cfg(not(feature = "inline-cowsay"))]
    #[argh(option)]
    ///path to a folder containing cows that shell-toy should use.
    pub cow_path: Option<String>,

    #[argh(switch, short = 'l', long = "list-cows")]
    /// lists the cows that are embedded in the executable
    pub list_cows: bool,

    #[argh(option, short = 'i', long = "image")]
    ///uses an image as the "cow" file
    pub image: Option<String>,

    #[argh(
        option,
        short = 'b',
        long = "bubble",
        from_str_fn(parse_bubble_type),
        default = "BubbleType::Cowsay"
    )]
    ///the type of bubble to create. Options are "think", "round", and "cowsay"
    pub bubble_type: BubbleType,

    #[argh(
        option,
        short = 't',
        long = "cow-type",
        from_str_fn(parse_cow_variant),
        default = "CowVariant::Default"
    )]
    /// changes the eyes/tounge of the outputted cow. Values allowed are
    /// "default", "borg", "dead", "greedy", "paranoid", "stoned", "tired", "wired", "young".
    /// "random" is also an option to choose one of the aforementioned values at random.
    /// This only affects cowfiles like the default cowsay cow which use the $eyes and/or $toungue variable
    pub cow_variant: CowVariant,

    #[cfg(not(feature = "inline-fortune"))]
    #[argh(option, short = 'f', long = "fortune-file")]
    ///instead of using internal fortunes, which file/dir to look in
    pub fortune_file: Option<String>,

    #[argh(option, long = "max-fort-width")]
    /// limits the chosen fortunes to be a maximum number of characters per line
    pub fortune_width: Option<u64>,

    #[argh(option, long = "max-fort-lines")]
    /// limits the chosen fortunes to contain less than the specified number of lines
    pub fortune_lines: Option<u64>,

    #[argh(switch, short = 'o')]
    /// whether to include offensive fortunes
    pub include_offensive: bool,

    #[argh(positional)]
    pub message: Option<String>,
}

fn parse_bubble_type(value: &str) -> Result<BubbleType, String> {
    match value {
        "think" => Ok(BubbleType::Think),
        "round" => Ok(BubbleType::Round),
        "cowsay" => Ok(BubbleType::Cowsay),
        _ => Err("Invalid bubble type".to_string()),
    }
}

fn parse_cow_variant(value: &str) -> Result<CowVariant, String> {
    match value {
        "borg" => Ok(CowVariant::Borg),
        "dead" => Ok(CowVariant::Dead),
        "greedy" => Ok(CowVariant::Greedy),
        "paranoid" => Ok(CowVariant::Paranoid),
        "stoned" => Ok(CowVariant::Stoned),
        "tired" => Ok(CowVariant::Tired),
        "wired" => Ok(CowVariant::Wired),
        "young" => Ok(CowVariant::Young),
        "default" => Ok(CowVariant::Default),
        "random" => Ok(CowVariant::Random),
        _ => Err("Invalid Cow Variant".to_string()),
    }
}
