use rand::{seq::IteratorRandom, thread_rng, Rng};
use std::{error::Error, fs::File, io::Read, path::PathBuf};

// struct Flags {
//     pub sflag: bool, //silent run
//     pub oflag: bool, //ordering
//     pub iflag: bool, //ignore case flag
//     pub rflag: bool, //randomize order
//     pub xflag: bool, //set rotated bit
// }

//default method of getting a fortune, without using the index file.
fn get_fortune(file_path: &PathBuf) -> Result<String, Box<dyn Error>> {
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
        Err(e) => panic!("Could not open Fortune file!"),
    }
}

// fn fortune_main() {
// let mut rng = thread_rng();
// let argv: Vec<String> = args().collect();

// if argv.len() < 2 {
//     println!("No Path Argument was defined!");
//     exit(1);
// }

// let fortune_path = PathBuf::from(argv[1].as_str());
// let path_metadata = metadata(fortune_path).unwrap();

// let mut file: File;
// if path_metadata.is_dir() {
//     let mut file_list: Vec<DirEntry> = fs::read_dir(fortune_path)
//         .expect("Path was marked as a directory but could not enumerate it's files!")
//         .into_iter()
//         .filter(|read_dir| {
//             return !ILLEGAL_FILE_SUFFIXES.contains(
//                 &read_dir
//                     .as_ref()
//                     .expect("error reading into the directory")
//                     .file_name()
//                     .into_string()
//                     .expect("msg")
//                     .as_str(),
//             );
//         })
//         .map(|val| val.expect("Error"))
//         .collect();
//     file_list.shuffle(&mut rng);
//     file = File::open(file_list.get(0).expect("Should have a 0th element").path())
//         .expect("Could not open the chosen fortune file in the directory");
// } else {
//     file = File::open(fortune_path.clone()).expect("Could not open the specified file!");
// }

//check for a strfile
// let strfile_path = fortune_path.as_path().a

// match get_fortune_no_index(&, &mut rng) {
//     Ok(fortune) => print!("{fortune}"),
//     Err(err) => {
//         println!("Error producing a fortune: {err}");
//         exit(1);
//     }
// }
// }
