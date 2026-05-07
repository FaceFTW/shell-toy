use std::{error::Error, ffi::OsStr, fs, io, path::PathBuf};

use argh::FromArgs;

use crate::cowsay::{BubbleType, CowVariant};

#[derive(FromArgs)]
/// various program options
pub(crate) struct Options {
    // #[cfg(not(feature = "inline-cowsay"))]
    #[argh(option, short = 'c')]
    /// path to a direct cowfile, a folder containing cow files, or the
    /// name of a cow file in the COW_PATH if set. Can be repeated to search through multiple paths
    pub cows: Vec<String>,

    #[cfg(feature = "inline-cowsay")]
    #[argh(switch, short = 'l', long = "list-cows")]
    /// lists the cows that are embedded in the executable
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

macro_rules! illegal_file_suffixes {
    ($($ext:literal),*) => {
        [
            $(std::ffi::OsStr::new($ext)),*
        ]
    };
}

impl Options {
    pub fn post_init(opts: Options) -> Options {
        // Cascade all options except for cow and fortunes which
        // should instead populate the individual files to operate on.
        Options {
            cows: match opts.cows.len() {
                0 => {
                    cfg_select! {
                        feature = "inline-cowsay" => { Vec::new() }
                        not(feature = "inline-cowsay") => { Vec::new() }
                    }
                }
                1 => {
                    cfg_select! {
                        feature = "inline-cowsay" => { opts.cows }
                        not(feature = "inline-cowsay") => { enumerate_files(&PathBuf::from(&opts.cows[0]), Some(&OsStr::new("cow")), None).expect("Could not enumerate cow files in specified path") }
                    }
                }
                _ => opts
                    .cows
                    .into_iter()
                    .flat_map(|path| {
                        enumerate_files(&PathBuf::from(path), Some(&OsStr::new("cow")), None)
                            .expect("Could not open some of the files listed")
                    })
                    .collect(),
            },
            fortunes: match opts.fortunes.len() {
                0 => cfg_select! {
                    feature = "inline-fortune" => {
                        Vec::new()
                    }
                    not(feature = "inline-fortune") => {
                        if let Ok(val) = std::env::var("FORTUNE_FILE") {
                            vec![val]
                        } else if let Ok(val) = std::env::var("FORTUNE_PATH") {
                            enumerate_files(
                                &PathBuf::from(val),
                                None,
                                Some(&illegal_file_suffixes!(
                                    "dat", "pos", "c", "h", "p", "i", "f", "pas", "ftn", "ins.c",
                                    "ins.pas", "ins.ftn", "sml", "sh", "pl", "csv"
                                )),
                            )
                            .expect("Could not open some of the files in the FORTUNE_PATH")
                        } else {
                            match std::env::consts::OS {
                                "linux" =>
                                    enumerate_files(
                                        &PathBuf::from("/usr/share/games/fortunes"),
                                        None,
                                        Some(&illegal_file_suffixes!(
                                            "dat", "pos", "c", "h", "p", "i", "f", "pas", "ftn", "ins.c",
                                            "ins.pas", "ins.ftn", "sml", "sh", "pl", "csv"
                                        )),
                                    )
                                    .expect("Could not open some of the files in the default fortunes directory")
                                _ => panic!(
                                    "I don't know what the default path for fortunes are for this OS!.\nPlease provide a FORTUNEPATH or FORTUNE_PATH environment variable, or a single file with FORTUNE_FILE"
                                ),
                            }
                        }
                    }
                },
                _ => opts
                    .fortunes
                    .into_iter()
                    .flat_map(|path| {
                        enumerate_files(
                            &PathBuf::from(path),
                            None,
                            Some(&illegal_file_suffixes!(
                                "dat", "pos", "c", "h", "p", "i", "f", "pas", "ftn", "ins.c",
                                "ins.pas", "ins.ftn", "sml", "sh", "pl", "csv"
                            )),
                        )
                        .expect("Could not open some of the files listed")
                    })
                    .collect(),
            },
            ..opts
        }
    }
}

fn enumerate_files(
    path: &PathBuf,
    extension: Option<&OsStr>,
    excluded_exts: Option<&[&OsStr]>,
) -> Result<Vec<String>, io::Error> {
    let mut total_list = vec![];
    match fs::metadata(path)?.file_type() {
        //TODO non-unicode handling
        ft if ft.is_file() => Ok(vec![path.to_string_lossy().to_string()]),
        ft if ft.is_dir() => {
            let dir_list = fs::read_dir(path)?.filter(|item| match excluded_exts {
                Some(excludes) => excludes.contains(
                    &item
                        .as_ref()
                        .unwrap()
                        .path()
                        .extension()
                        .unwrap_or_default(),
                ),
                None => true,
            });

            for entry in dir_list {
                match entry {
                    Ok(item) => match item.metadata()?.is_dir() {
                        true => total_list.append(
                            enumerate_files(&item.path(), extension, excluded_exts)
                                .unwrap()
                                .as_mut(),
                        ),
                        false => match extension {
                            Some(ext) => {
                                if item.path().extension().unwrap() == ext {
                                    total_list.push(item.path().to_str().unwrap().to_string());
                                }
                            }
                            None => total_list.push(item.path().to_str().unwrap().to_string()),
                        },
                    },
                    Err(e) => return Err(e),
                }
            }
            Ok(total_list)
        }
        //TODO probably don't want to panic, but it does the job
        _ => panic!("Encountered path {path:?} which was not a file or directory!"),
    }
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
