use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{
    env::args,
    error::Error,
    fs::{self, metadata, DirEntry, File},
    io::{self, Read, Seek, SeekFrom},
    path::PathBuf,
    process::exit,
};

pub const ILLEGAL_FILE_SUFFIXES: [&str; 13] = [
    "dat", "pos", "c", "h", "p", "i", "f", "pas", "ftn", "ins.c", "ins,pas", "ins.ftn", "sml",
];

// struct StrStruct {
//     pub first: char,
//     pub pos: i32,
// }

// struct Flags {
//     pub sflag: bool, //silent run
//     pub oflag: bool, //ordering
//     pub iflag: bool, //ignore case flag
//     pub rflag: bool, //randomize order
//     pub xflag: bool, //set rotated bit
// }

pub struct StrFileHeader {
    pub str_version: u32,
    pub str_numstr: u32,
    pub str_longlen: u32,
    pub str_shortlen: u32,
    pub random_order_flag: bool,
    pub ordered_order_flag: bool,
    pub rotated_flag: bool,
    pub delimiting_char: char,
}

const DELIMITER_CHAR: char = '%';

pub struct StrFile {
    pub header: StrFileHeader,
    pub offsets: Vec<u32>,
}

pub fn parse_strfile(file: &PathBuf) -> Result<StrFile, Box<dyn Error>> {
    let mut datfile = File::open(file)?;

    let str_version = datfile.read_u32::<LittleEndian>()?;
    let numstr = datfile.read_u32::<LittleEndian>()?;
    let longlen = datfile.read_u32::<LittleEndian>()?;
    let shortlen = datfile.read_u32::<LittleEndian>()?;
    //here we breakdown the flags
    let flags = datfile.read_u32::<LittleEndian>()?;
    let random_order = flags & 0x01u32 == 1;
    let ordered_order = flags & 0x02u32 == 1;
    let rotated = flags & 0x04u32 == 1;
    //Here we identify the delimiter, in most cases
    //it is likely going to be DELIMITER_CHAR, but we check just in case
    //We also use big endian to ensure the ascii is not treated as some kind of
    //unicode char
    let delimiter_raw = datfile.read_u32::<BigEndian>()?;
    let delimiter =
        char::from_u32(delimiter_raw).ok_or("Could not parse the delimiter in the strfile")?;

    let header = StrFileHeader {
        str_version,
        str_numstr: numstr,
        str_longlen: longlen,
        str_shortlen: shortlen,
        random_order_flag: random_order,
        ordered_order_flag: ordered_order,
        rotated_flag: rotated,
        delimiting_char: delimiter,
    };

    let mut offsets: Vec<u32> = Vec::new();

    while let Ok(val) = datfile.read_u32::<LittleEndian>() {
        offsets.push(val);
    }

    //A quick validation to ensure we got everything
    if offsets.len() != header.str_numstr as usize {
        return Err(Box::new(io::Error::new(io::ErrorKind::UnexpectedEof, "")));
    }

    Ok(StrFile { header, offsets })
}

pub fn choose_fortune_offset(offsets: &[u32], rng: &mut impl Rng) -> u32 {
    let idx: usize = rng.gen_range(0..offsets.len()).into();
    offsets[idx]
}

//default method of getting a fortune, without using the index file.
fn get_fortune_no_index(file: &mut File, rng: &mut impl Rng) -> Result<String, Box<dyn Error>> {
    // let path_metadata = metadata(fortune_path).unwrap();

    // let mut file: File;
    // if path_metadata.is_dir() {
    //     let mut file_list: Vec<DirEntry> = fs::read_dir(fortune_path)?
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
    //     file_list.shuffle(rng);
    //     file = File::open(file_list.get(0).expect("Should have a 0th element").path())?;
    // } else {
    //     file = File::open(fortune_path)?
    // }

    let mut string_buf = String::new();
    let _result = file.read_to_string(&mut string_buf)?;

    let fortunes: Vec<&str> = string_buf.split("%").collect();
    let rand_idx = rng.gen_range(0..fortunes.len());

    Ok(fortunes[rand_idx].to_string())
}

fn get_fortune_using_index(
    file: &mut File,
    strfile: &StrFile,
    rng: &mut impl Rng,
) -> Result<String, Box<dyn Error>> {
    // let mut file = File::open(fortune_path)?;

    let fortune_offset = choose_fortune_offset(&strfile.offsets.as_slice(), rng);
    file.seek(SeekFrom::Start(fortune_offset.into()))?;
    //
    let mut fortune_string = String::new();
    let mut char_buf: [u8; 16] = Default::default(); //A fairly reasonable buffer size
    while let Ok(_) = file.read_exact(&mut char_buf) {
        let mut short_circuit = false;
        for char_val in char_buf.map(|val| val as char) {
            if char_val != strfile.header.delimiting_char {
                fortune_string.push(char_val);
            } else {
                short_circuit = true;
                break;
            }
        }
        if short_circuit {
            break;
        }
    }

    Ok(fortune_string)
}

fn fortune_main() {
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
}
