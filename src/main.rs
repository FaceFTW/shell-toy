use std::{
    env::args,
    error::Error,
    fs::{self, metadata, read_to_string, DirEntry, File, Metadata},
    io::Read,
    path::PathBuf,
    process::exit,
};

use rand::{rngs::StdRng, seq::SliceRandom, thread_rng, Rng};

// mod file;
// mod strfile;

//A bunch of the static vars in the original source
// struct State {
//     pub found_one: bool,     // Did we find a match
//     pub find_files: bool,    // Just find a list of proper fortune files
//     pub fortunes_only: bool, // check only "fortunes" files
//     pub wait: bool,          // wait desired after fortune
//     pub short_only: bool,    // short fortune desired
//     pub long_only: bool,     // long fortune desired
//     pub offend: bool,        // offensive fortunes only
//     pub all_forts: bool,     // Any fortune allowed
//     pub equal_probs: bool,   // Scatter un-allocted prob equally
//     pub match_fortune: bool, // dump fortunes matching a pattern
//     pub write_to_disk: bool, // use files on disk to save state
//     pub fort_len: i64,
//     pub seekpts: [usize; 2], //seek pointers to fortunes
// }

pub const ILLEGAL_FILE_SUFFIXES: [&str; 13] = [
    "dat", "pos", "c", "h", "p", "i", "f", "pas", "ftn", "ins.c", "ins,pas", "ins.ftn", "sml",
];

//default method of getting a fortune, without using the index file.
fn get_fortune_no_index(
    fortune_path: &PathBuf,
    rng: &mut impl Rng,
) -> Result<String, Box<dyn Error>> {
    let path_metadata = metadata(fortune_path).unwrap();

    let mut file: File;
    if path_metadata.is_dir() {
        let mut file_list: Vec<DirEntry> = fs::read_dir(fortune_path)?
            .into_iter()
            .filter(|read_dir| {
                // let item = read_dir
                //     .expect("Error reading into the directory")
                //     .file_name()
                //     .into_string()
                //     .expect("Could not parse file name as a string");
                return !ILLEGAL_FILE_SUFFIXES.contains(
                    &read_dir
                        .as_ref()
                        .expect("error reading into the directory")
                        .file_name()
                        .into_string()
                        .expect("msg")
                        .as_str(),
                );
            })
            .map(|val| val.expect("Error"))
            .collect();
        file_list.shuffle(rng);
        file = File::open(file_list.get(0).expect("Should have a 0th element").path())?;
    } else {
        file = File::open(fortune_path)?
    }

    let mut string_buf = String::new();
    let _result = file.read_to_string(&mut string_buf)?;

    let fortunes: Vec<&str> = string_buf.split("%").collect();
    let rand_idx = rng.gen_range(0..fortunes.len());

    Ok(fortunes[rand_idx].to_string())
}

fn main() {
    let mut rng = thread_rng();
    let argv: Vec<String> = args().collect();

    if argv.len() < 2 {
        println!("No Path Argument was defined!");
        exit(1);
    }

    let path = PathBuf::from(argv[1].as_str());
    match get_fortune_no_index(&path, &mut rng) {
        Ok(fortune) => print!("{fortune}"),
        Err(err) => {
            println!("Error producing a fortune: {err}");
            exit(1);
        }
    }
}
