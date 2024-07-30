use rand::{seq::IteratorRandom, thread_rng};
use std::{error::Error, fs::File, io::Read, path::PathBuf};

// struct Flags {
//     pub sflag: bool, //silent run
//     pub oflag: bool, //ordering
//     pub iflag: bool, //ignore case flag
//     pub rflag: bool, //randomize order
//     pub xflag: bool, //set rotated bit
// }

//default method of getting a fortune, without using the index file.
pub fn get_fortune(file_path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let mut rng = thread_rng();
    match File::open(file_path) {
        Ok(mut file) => {
            let mut string_buf = String::new();
            let _result = file.read_to_string(&mut string_buf)?;

            Ok(string_buf
                .split("%")
                .choose(&mut rng)
                .expect("Could not choose an fortune for some reason")
                .to_string())
        }
        Err(e) => panic!("Could not open Fortune file! {e}"),
    }
}
