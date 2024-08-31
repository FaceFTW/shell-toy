use std::{
    cell::LazyCell,
    error::Error,
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read},
    path::PathBuf,
};
use tinyrand::Rand;

//default method of getting a fortune, without using the index file.
pub fn get_fortune(file_path: PathBuf, rng: &mut impl Rand) -> Result<String, Box<dyn Error>> {
    match File::open(file_path) {
        Ok(mut file) => {
            let mut string_buf = String::new();
            let _result = file.read_to_string(&mut string_buf)?;
            let split: Vec<&str> = string_buf.split("\n%\n").collect();
            let chosen_idx = rng.next_lim_usize(split.len());
            Ok(split[chosen_idx].to_string())
        }
        Err(e) => panic!("Could not open Fortune file! {e}"),
    }
}
// #[cfg(feature = "inline-fortune")]
// const INLINE_FORTUNES: &'static str = include_str!("../target/resources/fortunes");
// #[cfg(all(feature = "inline-fortune", feature = "inline-off-fortune"))]
// const OFF_FORTUNES: &'static str = include_str!("../target/resources/off_fortunes");
// #[cfg(all(feature = "inline", not(feature = "inline-off")))]
// const OFF_FORTUNES: &'static str = "";

#[cfg(feature = "inline-fortune")]
include!("../target/generated_sources/fortune_db.rs");

macro_rules! choose_inline_fortune {
    ($rng_ident:ident, $list_ident:ident) => {{
        let chosen_idx = $rng_ident.next_lim_usize($list_ident.len());
        Ok($list_ident[chosen_idx].to_string())
    }};
}

#[cfg(feature = "inline-fortune")]
pub fn get_inline_fortune(
    rng: &mut impl Rand,
    include_offensive: bool,
) -> Result<String, Box<dyn Error>> {
    //This is a fun little test

    cfg_if::cfg_if! {
        if #[cfg(feature="inline-off-fortune")]{
            if include_offensive {
                let weight_off:f64 = OFF_FORTUNE_LIST.len() as f64/(FORTUNE_LIST.len() as f64 + OFF_FORTUNE_LIST.len() as f64);
                match rng.next_bool(weight_off.into()){
                    true => choose_inline_fortune!(rng, OFF_FORTUNE_LIST),
                    false => choose_inline_fortune!(rng, FORTUNE_LIST),
                }
            } else {
                choose_inline_fortune!(rng, FORTUNE_LIST)
            }
        } else {
            choose_inline_fortune!(rng, FORTUNE_LIST)
        }
    }
}

pub fn choose_fortune_file(
    include_offensive: bool,
    rng: &mut impl Rand,
    supplied_path: Option<String>,
) -> PathBuf {
    let os = std::env::consts::OS;
    if let Some(val) = supplied_path {
        match fs::metadata(&val).unwrap().is_dir() {
            true => choose_random_fortune_file(&PathBuf::from(val), include_offensive, rng)
                .expect("Could not read the specified directory for getting fortunes"),
            false => PathBuf::from(val),
        }
    } else if let Ok(val) = std::env::var("FORTUNE_FILE") {
        PathBuf::from(val.as_str())
    } else if let Ok(val) = std::env::var("FORTUNE_PATH") {
        choose_random_fortune_file(&PathBuf::from(val.as_str()), include_offensive, rng)
            .expect("Could not choose a random fortune file from the specified Fortune Path")
    } else {
        match os{
            "linux" => choose_fortune_file(include_offensive, rng,Some(String::from("/usr/share/games/fortunes"))),
            _ => panic!("I don't know what the default path for fortunes are for this OS!.\nPlease provide a FORTUNEPATH or FORTUNE_PATH environment variable, or a single file with FORTUNE_FILE")
        }
    }
}

//Used LazyCell because We use this in an OsStr comparison context
//Adds effectively an O(1) operation based on my understanding
pub const ILLEGAL_FILE_SUFFIXES: LazyCell<[&OsStr; 13]> = LazyCell::new(|| {
    [
        OsStr::new("dat"),
        OsStr::new("pos"),
        OsStr::new("c"),
        OsStr::new("h"),
        OsStr::new("p"),
        OsStr::new("i"),
        OsStr::new("f"),
        OsStr::new("pas"),
        OsStr::new("ftn"),
        OsStr::new("ins.c"),
        OsStr::new("ins.pas"),
        OsStr::new("ins.ftn"),
        OsStr::new("sml"),
    ]
});

pub fn choose_random_fortune_file(
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
        let mut total_list = vec![];
        let dir_list = fs::read_dir(path).expect("Could not open directory");
        for entry in dir_list.filter(|item| {
            !ILLEGAL_FILE_SUFFIXES.contains(
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
