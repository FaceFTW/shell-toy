use std::error::Error;
use std::{fs::File, io::Read};
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

fn get_external_fortune(
    rng: &mut impl Rand,
    list: &[String],
    include_off: bool,
    max_width: Option<u64>,
    max_lines: Option<u64>,
) -> Result<String, Box<dyn Error>> {
    let file_path = match include_off {
        true => {
            let idx = rng.next_lim_usize(list.len());
            list[idx].clone()
        }
        false => {
            let filtered: Vec<&String> = list.into_iter().filter(|x| !x.contains("off")).collect();
            let idx = rng.next_lim_usize(filtered.len());
            filtered[idx].clone()
        }
    };

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

///default method of getting a fortune, without using the index file.
pub fn get_fortune(
    _fortune_files: &[String],
    rng: &mut impl Rand,
    include_offensive: bool,
    max_width: Option<u64>,
    max_lines: Option<u64>,
) -> Result<String, Box<dyn Error>> {
    cfg_select! {
        feature = "inline-fortune" => {
            if !_fortune_files.is_empty(){
                get_external_fortune(
                     rng,
                     _fortune_files,
                     include_offensive,
                     max_width,
                     max_lines
                )
            } else {
                let list = match include_offensive {
                    true => {
                        cfg_select! {
                             feature = "inline-off-fortune" => {
                                 if include_offensive {
                                     let weight_off:f64 = OFF_FORTUNE_LIST.len() as f64/(FORTUNE_LIST.len() as f64 + OFF_FORTUNE_LIST.len() as f64);
                                     match rng.next_bool(weight_off.into()){
                                         true => OFF_FORTUNE_LIST.as_slice(),
                                         false => FORTUNE_LIST.as_slice(),
                                     }
                                 } else {
                                     FORTUNE_LIST.as_slice()
                                 }
                             }
                             not(feature = "inline-off-fortune") => { FORTUNE_LIST.as_slice() }
                         }
                    }
                    false => FORTUNE_LIST.as_slice(),
                };

                let list_iter: Vec<&'static str> = list
                    .into_iter()
                    .filter(|element| check_fortune_constraints(element, max_width, max_lines))
                    .collect();
                let chosen_idx = rng.next_lim_usize(list_iter.len());
                Ok(list[chosen_idx].to_string())
            }

        }
        not(feature = "inline-fortune") => {
           get_external_fortune(
                rng,
                _fortune_files,
                include_offensive,
                max_width,
                max_lines
            )
        }
    }
}

/************************************************/
/************Inline Feature Functions************/
/************************************************/
#[cfg(feature = "inline-fortune")]
include!("../target/generated_sources/fortune_db.rs");
