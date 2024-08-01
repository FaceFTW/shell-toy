mod cli;
mod cowsay;
mod fortune;
mod parser;

use std::{fs::File, io::Read, path::PathBuf};

use cli::Options;
use cowsay::{choose_random_cow, identify_cow_path, print_cowsay, SpeechBubble};
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
            &PathBuf::from(options.cow_path.as_deref().unwrap()),
            &mut rng,
        ),
        _ => {
            let cow_path = identify_cow_path();
            choose_random_cow(&cow_path, &mut rng)
        }
    };

    let cow_msg = match options.message {
        Some(msg) => msg,
        None => {
            let fortune_file = choose_fortune_file(options.include_offensive, &mut rng);
            get_fortune(&fortune_file, &mut rng)
                .expect("Could not get a fortune, your future is shrouded in mystery...")
        }
    };

    print_cowsay(
        &cow_str,
        SpeechBubble::new(options.bubble_type),
        &cow_msg,
    );
}
