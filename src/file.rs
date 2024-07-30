///Functions for processing files and related things. Also checks environment variables for certain things
use rand::{seq::SliceRandom, thread_rng};
use std::{
    cell::LazyCell,
    ffi::OsStr,
    fs,
    io::{self, Read},
    path::PathBuf,
};

fn get_list_of_cows(path: &PathBuf) -> Result<Vec<String>, io::Error> {
    let mut total_list = vec![];
    let dir_list = fs::read_dir(path)?;
    for entry in dir_list {
        match entry {
            Ok(item) => match item.metadata()?.is_dir() {
                true => total_list.append(get_list_of_cows(&item.path()).unwrap().as_mut()),
                false => {
                    if item.path().extension().unwrap() == "cow" {
                        total_list.push(item.path().to_str().unwrap().to_string());
                    }
                }
            },
            Err(e) => return Err(e),
        }
    }

    Ok(total_list)
}

pub fn choose_random_cow(cow_path: &PathBuf) -> String {
    let mut rng = thread_rng();
    let cow_list = get_list_of_cows(cow_path).expect("Could not open the cow path");

    let chosen_path = cow_list
        .as_slice()
        .choose(&mut rng)
        .expect("List had no elements to choose from!");
    match fs::File::open(chosen_path) {
        Ok(mut file) => {
            let mut cow_str = String::new();
            file.read_to_string(&mut cow_str)
                .expect("Error reading cow string");
            cow_str
        }
        Err(e) => panic!("{e}"),
    }
}

pub fn identify_cow_path() -> PathBuf {
    //Check if we have an environment variable defined:
    let os = std::env::consts::OS;
    if let Ok(val) = std::env::var("COWPATH") {
        PathBuf::from(val.as_str())
    } else if let Ok(val) = std::env::var("COW_PATH") {
        PathBuf::from(val.as_str())
    } else {
        match os{
            "linux" => PathBuf::from("/usr/share/cowsay/cows"),
            _ => panic!("I don't know what the default path for cowfiles is for this OS!.\nPlease provide a COWPATH or COW_PATH environment variable")
        }
    }
}

pub fn choose_fortune_file(include_offensive: bool) -> PathBuf {
    let os = std::env::consts::OS;
    if let Ok(val) = std::env::var("FORTUNE_FILE") {
        PathBuf::from(val.as_str())
    } else if let Ok(val) = std::env::var("FORTUNE_PATH") {
        choose_random_fortune_file(&PathBuf::from(val.as_str()), include_offensive)
            .expect("Could not choose a random fortune file from the specified Fortune Path")
    } else if let Ok(val) = std::env::var("FORTUNEPATH") {
        choose_random_fortune_file(&PathBuf::from(val.as_str()), include_offensive)
            .expect("Could not choose a random fortune file from the specified Fortune Path")
    } else {
        match os{
            "linux" => PathBuf::from("/usr/share/games/fortune"),
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

    let mut rng = thread_rng();
    let list = iterate(path, include_offensive);

    match list.choose(&mut rng) {
        Some(val) => Ok(val.clone()),
        None => panic!("Could not choose a random fortune file!"),
    }
}
