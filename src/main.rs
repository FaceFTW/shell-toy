mod cli;
mod cowsay;
mod fortune;
mod parser;

use cli::Options;
#[cfg(not(feature = "inline-cowsay"))]
use cowsay::identify_cow_path;
use cowsay::{choose_random_cow, print_cowsay, random_cow_variant, CowVariant, SpeechBubble};
#[cfg(not(feature = "inline-cowsay"))]
use std::{fs::File, io::Read};
use tinyrand::{Seeded, StdRand};

fn main() {
    //Init RNG
    let mut buf = [0u8; 8];
    getrandom::getrandom(&mut buf).expect("Could not open entropy source!");
    let mut rng = StdRand::seed(u64::from_le_bytes(buf));

    let options: Options = argh::from_env();

    cfg_if::cfg_if! {
        if #[cfg(feature="inline-cowsay")]{
            let cow_str = choose_random_cow(&mut rng);
        } else {
            let cow_str = match &options.cow_file {
                 Some(file_path)=> {
                    match File::open(file_path) {
                        Ok(mut file) => {
                            let mut cow_str = String::new();
                            file.read_to_string(&mut cow_str)
                                .expect("Error reading Cowfile");
                            cow_str
                        }
                        Err(e) => panic!("{e}"),
                    }
                }
                None => {
                    let cow_path = identify_cow_path(&options.cow_path);
                    choose_random_cow(&cow_path, &mut rng)
                }
            };
    }};

    let cow_msg = match options.message {
        Some(msg) => msg,
        None => {
            cfg_if::cfg_if! {
                if #[cfg(feature="inline-fortune")]{
                        fortune::get_inline_fortune(&mut rng, options.include_offensive)
                            .expect("Could not read internal fortune index, your future is shrouded in mystery...")
                } else {
                    let fortune_file = fortune::choose_fortune_file(options.include_offensive, &mut rng, options.fortune_file );
                    fortune::get_fortune(fortune_file, &mut rng)
                .expect("Could not get a fortune, your future is shrouded in mystery...")
                }
            }
        }
    };

    let cow_variant = match options.cow_variant {
        CowVariant::Random => random_cow_variant(&mut rng),
        _ => options.cow_variant,
    };

    print_cowsay(
        &cow_str,
        SpeechBubble::new(options.bubble_type),
        &cow_msg,
        &cow_variant,
    );
}
