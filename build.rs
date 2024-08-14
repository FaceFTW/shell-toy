use std::{
    cell::LazyCell,
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
};

//This will likely always trigger because it just affects "pre-"compile time and not runtime
fn main() -> Result<(), io::Error> {
    create_fortune_db()
}

///
fn create_fortune_db() -> Result<(), io::Error> {
    if let Err(_) = fs::read_dir("target/resources") {
        fs::create_dir("target/resources")?
    }

    if let Ok(val) = std::env::var("FORTUNE_FILE") {
        println!("cargo::rerun-if-changed={val}");
        let _ = fs::copy(val, "target/resources/fortunes")?;
        Ok(())
    } else if let Ok(val) = std::env::var("FORTUNE_PATH") {
        gen_concat_fortune_files(val)
    } else if let Ok(val) = std::env::var("FORTUNEPATH") {
        gen_concat_fortune_files(val)
    } else {
        match std::env::consts::OS{
            "linux" => gen_concat_fortune_files(String::from("/usr/share/games/fortune")),
            _ => panic!("I don't know what the default path for fortunes are for this OS!.\nPlease provide a FORTUNEPATH or FORTUNE_PATH environment variable, or a single file with FORTUNE_FILE")
        }
    }
}

fn gen_concat_fortune_files(val: String) -> Result<(), io::Error> {
    println!("cargo::rerun-if-changed={val}");
    let (fortune_list, offensive_list) = fortune_list_iterate(&PathBuf::from(val), false);
    let concat_fortunes = concat_fortune_files(fortune_list.as_slice())?;
    let off_concat_fortunes = concat_fortune_files(offensive_list.as_slice())?;

    match File::create("target/resources/fortunes") {
        Ok(mut file) => {
            let _ = file.write_all(concat_fortunes.trim().as_bytes())?;
        }
        Err(err) => panic!("Could not concatenate fortunes into single file: {err}"),
    }
    match File::create("target/resources/off_fortunes") {
        Ok(mut file) => {
            let _ = file.write_all(off_concat_fortunes.trim().as_bytes())?;
        }
        Err(err) => panic!("Could not concatenate fortunes into single file: {err}"),
    }
    Ok(())
}

//NOTE the following is copied over from the respective modules in the source tree since we can't use that code in the build script directly iirca

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

//this is somewhat memory inefficient but again, it's compile-time only
fn fortune_list_iterate(path: &PathBuf, is_offensive: bool) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut fortune_list = vec![];
    let mut offensive_list = vec![];
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
                    if item.file_name() == "./off" {
                        offensive_list.append(&mut fortune_list_iterate(&item.path(), true).1)
                    } else {
                        let vecs = &mut fortune_list_iterate(&item.path(), false);
                        fortune_list.append(&mut vecs.0);
                        offensive_list.append(&mut vecs.1);
                    }
                }
                false => match is_offensive {
                    true => offensive_list.push(item.path()),
                    false => fortune_list.push(item.path()),
                },
            },
            Err(e) => panic!("Could not identify a file in the fortune directory {e}"),
        }
    }
    (fortune_list, offensive_list)
}

fn concat_fortune_files(list: &[PathBuf]) -> Result<String, io::Error> {
    let mut buffer = String::new();
    for path in list {
        match File::open(path) {
            Ok(mut file) => {
                let _ = file.read_to_string(&mut buffer)?;
            }
            Err(err) => panic!("Could not open file: {err}"),
        }
    }
    Ok(buffer)
}

// fn get_list_of_cows(path: &PathBuf) -> Result<Vec<String>, io::Error> {
//     let mut total_list = vec![];
//     let dir_list = fs::read_dir(path)?;
//     for entry in dir_list {
//         match entry {
//             Ok(item) => match item.metadata()?.is_dir() {
//                 true => total_list.append(get_list_of_cows(&item.path()).unwrap().as_mut()),
//                 false => {
//                     if item.path().extension().unwrap() == "cow" {
//                         total_list.push(item.path().to_str().unwrap().to_string());
//                     }
//                 }
//             },
//             Err(e) => return Err(e),
//         }
//     }

//     Ok(total_list)
// }

// fn identify_cow_path() -> PathBuf {
//     //Check if we have an environment variable defined:
//     let os = std::env::consts::OS;
//     if let Ok(val) = std::env::var("COWPATH") {
//         PathBuf::from(val.as_str())
//     } else if let Ok(val) = std::env::var("COW_PATH") {
//         PathBuf::from(val.as_str())
//     } else {
//         match os{
//             "linux" => PathBuf::from("/usr/share/cowsay/cows"),
//             _ => panic!("I don't know what the default path for cowfiles is for this OS!.\nPlease provide a COWPATH or COW_PATH environment variable")
//         }
//     }
// }
