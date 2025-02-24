use std::error::Error;
#[cfg(not(feature = "inline-fortune"))]
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read},
    path::PathBuf,
};
use tinyrand::Rand;

fn check_fortune_constraints(
    element: &&str,
    max_width: Option<u64>,
    max_lines: Option<u64>,
) -> bool {
    (match max_width {
        Some(val) =>
            element
                .split("\n")
                .reduce(|acc, e| if e.len() > acc.len(){e} else {acc})
                .expect("Could not split the chosen string for constraint validation")
                .len() <= val as usize,
        None => true,
    })
    //You can do this yes very cool
    &&(match max_lines {
        Some(val) => {
            element.chars().fold(0, |acc, e| match e == '\n' {
                true => acc + 1,
                false => acc,
            }) <= val
        }
        None => true,
    })
}

///default method of getting a fortune, without using the index file.
#[cfg(not(feature = "inline-fortune"))]
pub fn get_fortune(
    file_path: PathBuf,
    rng: &mut impl Rand,
    max_width: Option<u64>,
    max_lines: Option<u64>,
) -> Result<String, Box<dyn Error>> {
    match File::open(file_path) {
        Ok(mut file) => {
            let mut string_buf = String::new();
            let _result = file.read_to_string(&mut string_buf)?;
            let no_cr = string_buf.replace("\r", "");
            let split: Vec<&str> = no_cr
                .split("\n%\n")
                .filter(|element| check_fortune_constraints(element, max_width, max_lines))
                .collect();
            let chosen_idx = rng.next_lim_usize(split.len());
            Ok(split[chosen_idx].to_string())
        }
        Err(e) => panic!("Could not open Fortune file! {e}"),
    }
}

#[cfg(not(feature = "inline-fortune"))]
pub fn choose_fortune_file(
    include_offensive: bool,
    rng: &mut impl Rand,
    supplied_path: Option<String>,
) -> PathBuf {
    let os = std::env::consts::OS;
    if let Some(val) = supplied_path {
        match fs::metadata(&val).unwrap().is_dir() {
            true => choose_random_file(&PathBuf::from(val), include_offensive, rng)
                .expect("Could not read the specified directory for getting fortunes"),
            false => PathBuf::from(val),
        }
    } else if let Ok(val) = std::env::var("FORTUNE_FILE") {
        PathBuf::from(val.as_str())
    } else if let Ok(val) = std::env::var("FORTUNE_PATH") {
        choose_random_file(&PathBuf::from(val.as_str()), include_offensive, rng)
            .expect("Could not choose a random fortune file from the specified Fortune Path")
    } else {
        match os {
            "linux" => choose_fortune_file(
                include_offensive,
                rng,
                Some(String::from("/usr/share/games/fortunes")),
            ),
            _ => panic!(
                "I don't know what the default path for fortunes are for this OS!.\nPlease provide a FORTUNEPATH or FORTUNE_PATH environment variable, or a single file with FORTUNE_FILE"
            ),
        }
    }
}

#[cfg(not(feature = "inline-fortune"))]
macro_rules! illegal_file_suffixes {
    ($($ext:literal),*) => {
        [
            $(std::ffi::OsStr::new($ext)),*
        ]
    };
}

#[cfg(not(feature = "inline-fortune"))]
pub fn choose_random_file(
    path: &PathBuf,
    include_offensive: bool,
    rng: &mut impl Rand,
) -> Result<PathBuf, io::Error> {
    //Couple of rules when it comes to fortune files (defaults)
    //1. Default fortunes are in /usr/share/games/fortunes
    //2. Offensive fortunes are in /usr/share/games/fortunes/off. There is no prefix
    //3. Currently, we load the entire text into memory, so we don't care about DAT
    //4. Ignore Illegal suffixes in the ILLEGAL_SUFFIXES list

    fn iterate(path: &PathBuf, include_offensive: bool) -> Vec<PathBuf> {
        let illegal_file_suffixes: [&OsStr; 16] = illegal_file_suffixes!(
            "dat", "pos", "c", "h", "p", "i", "f", "pas", "ftn", "ins.c", "ins.pas", "ins.ftn",
            "sml", "sh", "pl", "csv"
        );
        let mut total_list = vec![];
        let dir_list = fs::read_dir(path).expect("Could not open directory");
        for entry in dir_list.filter(|item| {
            !illegal_file_suffixes.contains(
                &item
                    .as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .unwrap_or_default(),
            )
        }) {
            match entry {
                Ok(item) => match item.metadata().unwrap().is_dir() {
                    true => {
                        if (item.file_name() != "./off")
                            || (item.file_name() == "./off" && include_offensive)
                        {
                            total_list.append(&mut iterate(&item.path(), include_offensive))
                        }
                    }
                    false => total_list.push(item.path()),
                },
                Err(e) => panic!("Could not identify a file in the fortune directory {e}"),
            }
        }
        total_list
    }

    // let mut rng = thread_rng();
    let list = iterate(path, include_offensive);
    let chosen_idx = rng.next_lim_usize(list.len());

    Ok(list[chosen_idx].clone())
}

/************************************************/
/************Inline Feature Functions************/
/************************************************/
#[cfg(feature = "inline-fortune")]
include!("../target/generated_sources/fortune_db.rs");

#[cfg(feature = "inline-fortune")]
pub fn get_inline_fortune(
    rng: &mut impl Rand,
    include_offensive: bool,
    max_width: Option<u64>,
    max_lines: Option<u64>,
) -> Result<String, Box<dyn Error>> {
    macro_rules! choose_inline_fortune {
        ($rng_ident:ident, $list_ident:ident, $max_wid_ident:ident, $max_lines_ident:ident) => {{
            let list_iter: Vec<&'static str> = $list_ident
                .into_iter()
                .filter(|element| {
                    check_fortune_constraints(element, $max_wid_ident, $max_lines_ident)
                })
                .collect();
            let chosen_idx = $rng_ident.next_lim_usize(list_iter.len());
            Ok($list_ident[chosen_idx].to_string())
        }};
    }

    cfg_if::cfg_if! {
        if #[cfg(feature="inline-off-fortune")]{
            if include_offensive {
                let weight_off:f64 = OFF_FORTUNE_LIST.len() as f64/(FORTUNE_LIST.len() as f64 + OFF_FORTUNE_LIST.len() as f64);
                match rng.next_bool(weight_off.into()){
                    true => choose_inline_fortune!(rng, OFF_FORTUNE_LIST, max_width, max_lines),
                    false => choose_inline_fortune!(rng, FORTUNE_LIST, max_width, max_lines),
                }
            } else {
                choose_inline_fortune!(rng, FORTUNE_LIST, max_width, max_lines)
            }
        } else {
            let _ = include_offensive;
            choose_inline_fortune!(rng, FORTUNE_LIST, max_width, max_lines)
        }
    }
}
