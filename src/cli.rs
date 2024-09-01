use argh::FromArgs;

use crate::cowsay::BubbleType;

#[derive(FromArgs)]
/// various program options
pub(crate) struct Options {
    #[cfg(not(feature = "inline-cowsay"))]
    #[argh(option, short = 'c')]
    ///path to a direct cowfile
    pub cow_file: Option<String>,

    #[cfg(not(feature = "inline-cowsay"))]
    #[argh(option)]
    ///path to a folder containing multiple cows we should search.
    pub cow_path: Option<String>,

    #[argh(option, short = 'o', default = "false")]
    /// whether to include offensive fortunes
    pub include_offensive: bool,

    #[argh(
        option,
        short = 'b',
        long = "bubble",
        from_str_fn(parse_bubble_type),
        default = "BubbleType::Cowsay"
    )]
    ///the type of bubble to create. Options are "think", "round", and "cowsay"
    pub bubble_type: BubbleType,

    #[argh(positional)]
    pub message: Option<String>,

    #[cfg(not(feature = "inline-fortune"))]
    #[argh(option, short = 'f', long = "fortune-file")]
    ///instead of using internal fortunes, which file/dir to look in
    pub fortune_file: Option<String>,
}

fn parse_bubble_type(value: &str) -> Result<BubbleType, String> {
    match value {
        "think" => Ok(BubbleType::Think),
        "round" => Ok(BubbleType::Round),
        "cowsay" => Ok(BubbleType::Cowsay),
        _ => Err("Invalid bubble type".to_string()),
    }
}
