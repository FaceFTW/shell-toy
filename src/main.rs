mod cli;
mod cowsay;
mod fortune;
mod parser;

use cowsay::{print_cowsay, SpeechBubble};

fn main() {
    let test_cow = include_str!("../cows/default.cow");
    let test_new_cow = include_str!("../cows/025_pikachu-alola-cap.cow");
    let test_dragon_cow = include_str!("../cows/dragon.cow");

    // let mut it = nom::combinator::iterator(test_new_cow, cow_parser);
    // let parsed = it.collect::<Vec<TerminalCharacter>>();
    // println!("{:#?}\n", parsed);
    // print!("{:#}\n", derive_cow_str(parsed.as_slice()));

    // let mut it = nom::combinator::iterator(test_cow, cow_parser);
    // let parsed2 = it.collect::<Vec<TerminalCharacter>>();
    // println!("{:#?}\n", parsed2);
    // print!("{:#}\n", derive_cow_str(parsed2.as_slice()));

    println!("{:#?}\n", parser::cow_parser(test_new_cow));

    let test_msg = "Test\ntesting\ntesting.......";
    let test_short_msg = "deez nuts";
    let test_long_msg ="lmaooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo";
    let bubble = SpeechBubble::new(cowsay::BubbleType::Cowsay);

    print_cowsay(&test_cow, bubble.clone(), &test_msg);
    print_cowsay(&test_cow, bubble.clone(), &test_short_msg);
    print_cowsay(&test_cow, bubble.clone(), &test_long_msg);
    print_cowsay(&test_new_cow, bubble.clone(), &test_long_msg);
    print_cowsay(&test_dragon_cow, bubble.clone(), &test_long_msg);
}
