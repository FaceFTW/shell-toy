use argh::FromArgs;

#[derive(FromArgs)]
/// various program options
pub(crate) struct Options {
    #[argh(option)]
    ///path to a direct cowfile
    pub cow_file: Option<String>,

    #[argh(option)]
    ///path to a folder containing multiple cows we should search.
    pub cow_path: Option<String>,

    #[argh(option, default = "false")]
    /// whether to include offensive fortunes
    pub include_offensive: bool,

    #[argh(positional)]
    pub message: Option<String>,
}
