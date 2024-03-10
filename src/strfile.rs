use std::{
    error::Error,
    fs::File,
    io::{self, BufReader, Read},
    os::windows::fs::FileExt,
    path::PathBuf,
};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use rand::Rng;

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
