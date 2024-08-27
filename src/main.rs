mod cli;
mod cowsay;
mod fortune;
mod parser;

use std::{fs::File, io::Read, path::PathBuf};

use cli::Options;
#[cfg(not(feature = "inline-cowsay"))]
use cowsay::identify_cow_path;
use cowsay::{choose_random_cow, print_cowsay, SpeechBubble};
#[cfg(feature = "inline-fortune")]
use fortune::get_inline_fortune;
use fortune::{choose_fortune_file, get_fortune};
use tinyrand::{Seeded, StdRand};

fn main() {
    //Init RNG
    let mut buf = [0u8; 8];
    getrandom::getrandom(&mut buf).expect("Could not open entropy source!");
    let mut rng = StdRand::seed(u64::from_le_bytes(buf));

    let options: Options = argh::from_env();

    let cow_str = match &options {
        options if options.cow_file.is_some() => {
            match File::open(options.cow_file.as_deref().unwrap()) {
                Ok(mut file) => {
                    let mut cow_str = String::new();
                    file.read_to_string(&mut cow_str)
                        .expect("Error reading cow string");
                    cow_str
                }
                Err(e) => panic!("{e}"),
            }
        }
        options if options.cow_path.is_some() => choose_random_cow(
            &Some(PathBuf::from(options.cow_path.as_deref().unwrap())),
            &mut rng,
        ),
        _ => {
            cfg_if::cfg_if! {
                if #[cfg(feature="inline-cowsay")]{
                    choose_random_cow(&None, &mut rng)
                } else {
                    let cow_path = identify_cow_path();
                    choose_random_cow(Some(&cow_path), &mut rng)

                }
            }
        }
    };

    let cow_msg = match options.message {
        Some(msg) => msg,
        None => {
            cfg_if::cfg_if! {
                if #[cfg(feature="inline-fortune")]{
                    if let Some(path) = options.fortune_file{
                        let fortune_file = choose_fortune_file(options.include_offensive, &mut rng, Some(path) );
                        get_fortune(fortune_file, &mut rng)
                            .expect("Could not get a fortune, your future is shrouded in mystery...")
                    } else {
                        get_inline_fortune(&mut rng, options.include_offensive)
                            .expect("Could not read internal fortune index, your future is shrouded in mystery...")
                    }
                } else {
                    let fortune_file = choose_fortune_file(options.include_offensive, &mut rng, options.fortune_file );
                    get_fortune(fortune_file, &mut rng)
                .expect("Could not get a fortune, your future is shrouded in mystery...")
                }
            }
        }
    };

    print_cowsay(&cow_str, SpeechBubble::new(options.bubble_type), &cow_msg);
}
