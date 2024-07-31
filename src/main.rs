mod cli;
mod cowsay;
mod file;
mod fortune;
mod parser;

use std::{fs::File, io::Read, path::PathBuf};

use cli::Options;
use cowsay::{print_cowsay, SpeechBubble};
use file::{choose_fortune_file, choose_random_cow, identify_cow_path};
use fortune::get_fortune;
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
        SpeechBubble::new(cowsay::BubbleType::Cowsay),
        &cow_msg,
    );

    // let test_cow = include_str!("../cows/default.cow");
    // let test_new_cow = include_str!("../cows/025_pikachu-alola-cap.cow");
    // let test_dragon_cow = include_str!("../cows/dragon.cow");

    // let mut it = nom::combinator::iterator(test_new_cow, cow_parser);
    // let parsed = it.collect::<Vec<TerminalCharacter>>();
    // println!("{:#?}\n", parsed);
    // print!("{:#}\n", derive_cow_str(parsed.as_slice()));

    // let mut it = nom::combinator::iterator(test_cow, cow_parser);
    // let parsed2 = it.collect::<Vec<TerminalCharacter>>();
    // println!("{:#?}\n", parsed2);
    // print!("{:#}\n", derive_cow_str(parsed2.as_slice()));

    // println!("{:#?}\n", parser::cow_parser(test_new_cow));

    // let test_msg = "Test\ntesting\ntesting.......";
    // let test_short_msg = "deez nuts";
    // let test_long_msg ="lmaooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo";
    // let bubble = SpeechBubble::new(cowsay::BubbleType::Cowsay);

    // print_cowsay(&test_cow, bubble.clone(), &test_msg);
    // print_cowsay(&test_cow, bubble.clone(), &test_short_msg);
    // print_cowsay(&test_cow, bubble.clone(), &test_long_msg);
    // print_cowsay(&test_new_cow, bubble.clone(), &test_long_msg);
    // print_cowsay(&test_dragon_cow, bubble.clone(), &test_long_msg);
}
