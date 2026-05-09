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
    _fortune_files: &[String],
    rng: &mut impl Rand,
    include_offensive: bool,
    max_width: Option<u64>,
    max_lines: Option<u64>,
) -> Result<String, Box<dyn Error>> {
    cfg_select! {
        feature = "inline-fortune" => {
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
        not(feature = "inline-fortune") => {
            // TODO Choose the file to open, exclude offensive as necessary

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
    }
}

/************************************************/
/************Inline Feature Functions************/
/************************************************/
#[cfg(feature = "inline-fortune")]
include!("../target/generated_sources/fortune_db.rs");
