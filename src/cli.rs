use std::{ffi::OsStr, fs, path::PathBuf};

use argh::FromArgs;

use crate::cowsay::{BubbleType, CowVariant};

#[derive(FromArgs, Debug)]
/// various program options
pub(crate) struct Options {
    // #[cfg(not(feature = "inline-cowsay"))]
    #[argh(option, short = 'c')]
    /// path to a direct cowfile, a folder containing cow files, or the name of the cow
    /// present in the shell-toy binary. Can be repeated to search through multiple paths
    pub cows: Vec<String>,

    #[argh(switch, short = 'l', long = "list-cows")]
    /// lists the cows available
    pub list_cows: bool,

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

    #[argh(option, short = 'f', long = "fortunes")]
    /// what files/folders to look at for fetching fortunes. Multiple paths
    /// can be provided to expand the list
    pub fortunes: Vec<String>,

    #[argh(option, long = "max-fort-width")]
    /// limits the chosen fortunes to be a maximum number of characters per line
    pub fortune_width: Option<u64>,

    #[argh(option, long = "max-fort-lines")]
    /// limits the chosen fortunes to contain less than the specified number of lines
    pub fortune_lines: Option<u64>,

    #[argh(switch, short = 'o')]
    /// whether to include offensive fortunes
    pub include_offensive: bool,

    #[argh(option, short = 'w')]
    /// limits the length of speech bubbles. Default is 64
    pub max_width: Option<u64>,

    #[argh(positional)]
    pub message: Option<String>,
}

impl Options {
    pub fn post_init(self) -> Options {
        // Cascade all options except for cow and fortunes which
        // should instead populate the individual files to operate on.
        Options {
            cows: match self.cows.len() {
                0 => {
                    cfg_select! {
                        feature = "inline-cowsay" => { Vec::new() }
                        not(feature = "inline-cowsay") => {
                            if let Ok(val) = std::env::var("COW_PATH"){
                               enumerate_cows(&PathBuf::from(val))
                            } else {
                               match std::env::consts::OS {
                                   "linux" => vec![String::from("/usr/share/cowsay/cows")],
                                   _ => panic!("I don't know what the default path for cowsay files are for this OS! \
                                               Please set the COW_PATH environment variable to where the cow files are located.")
                               }
                            }
                        }
                    }
                }
                1 => {
                    cfg_select! {
                        // Provides the name of the cow to look for in the index
                        feature = "inline-cowsay" => { self.cows }
                        not(feature = "inline-cowsay") => { enumerate_cows(&PathBuf::from(&self.cows[0])) }
                    }
                }
                _ => self
                    .cows
                    .into_iter()
                    .flat_map(|path| enumerate_cows(&PathBuf::from(path)))
                    .collect(),
            },

            fortunes: match self.fortunes.len() {
                0 => cfg_select! {
                    feature = "inline-fortune" => {
                        Vec::new()
                    }
                    not(feature = "inline-fortune") => {
                        if let Ok(val) = std::env::var("FORTUNE_FILE") {
                            vec![val]
                        } else if let Ok(val) = std::env::var("FORTUNE_PATH") {
                           enumerate_fortunes(&PathBuf::from(val))
                        } else {
                            match std::env::consts::OS {
                                "linux" => enumerate_fortunes(&PathBuf::from("/usr/share/games/fortunes")),
                                _ => panic!(
                                    "I don't know what the default path for fortunes are for this OS!. \
                                    Please provide a FORTUNE_PATH environment variable, or a single file with FORTUNE_FILE"
                                ),
                            }
                        }
                    }
                },
                _ => self
                    .fortunes
                    .into_iter()
                    .flat_map(|path| enumerate_fortunes(&PathBuf::from(path)))
                    .collect(),
            },

            ..self
        }
    }
}

fn enumerate_files(
    path: &PathBuf,
    extension: Option<&OsStr>,
    excluded_exts: Option<&[&OsStr]>,
) -> Vec<String> {
    let mut total_list = vec![];
    match fs::metadata(path)
        .expect(&format!("Could not get metadata for path {path:?}"))
        .file_type()
    {
        //TODO non-unicode handling
        ft if ft.is_file() => vec![path.to_string_lossy().to_string()],
        ft if ft.is_dir() => {
            let dir_list = fs::read_dir(path)
                .expect(&format!("Could not open the directory {path:?}"))
                .filter(|item| match excluded_exts {
                    Some(excludes) => excludes.contains(
                        &item
                            .as_ref()
                            .unwrap() //TODO this is a bug waiting to happen
                            .path()
                            .extension()
                            .unwrap_or_default(),
                    ),
                    None => true,
                });

            for entry in dir_list {
                match entry {
                    Ok(item) => match item
                        .metadata()
                        .expect(&format!(
                            "Could not get metadata for file entry {}",
                            item.path().to_string_lossy(),
                        ))
                        .is_dir()
                    {
                        true => total_list.append(
                            enumerate_files(&item.path(), extension, excluded_exts).as_mut(),
                        ),
                        false => match extension {
                            Some(ext) => {
                                if let Some(file_ext) = item.path().extension()
                                    && file_ext == ext
                                {
                                    total_list.push(handle_path(item));
                                }
                            }
                            None => total_list.push(handle_path(item)),
                        },
                    },
                    Err(e) => panic!("Could not enumerate some file entries: {e}"),
                }
            }
            total_list
        }
        _ => panic!("Encountered path {path:?} which was not a file or directory!"),
    }
}

//TODO check effect of inlining
#[inline]
fn handle_path(item: std::fs::DirEntry) -> String {
    item.path()
        .to_str()
        .expect(
               &format!(
                    "Encounted a file with invalid unicode in it's path.\nThe path with invalid unicode removed: {}",
                    item.path().to_string_lossy())
        ).to_string()
}

macro_rules! illegal_file_suffixes {
    ($($ext:literal),*) => {
        [
            $(std::ffi::OsStr::new($ext)),*
        ]
    };
}

//TODO check effect of inlining
#[inline]
fn enumerate_fortunes(path: &PathBuf) -> Vec<String> {
    enumerate_files(
        &PathBuf::from(path),
        None,
        Some(&illegal_file_suffixes!(
            "dat", "pos", "c", "h", "p", "i", "f", "pas", "ftn", "ins.c", "ins.pas", "ins.ftn",
            "sml", "sh", "pl", "csv"
        )),
    )
}

//TODO check effect of inlining
#[inline]
fn enumerate_cows(path: &PathBuf) -> Vec<String> {
    enumerate_files(&PathBuf::from(path), Some(&OsStr::new("cow")), None)
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
