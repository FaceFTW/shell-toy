mod cli;
mod cowsay;
mod fortune;
mod parser;

use cli::Options;
use cowsay::{CowVariant, SpeechBubble, get_cow_string, print_cowsay, random_cow_variant};

use tinyrand::{Seeded, StdRand};

fn main() {
    //Init RNG
    let mut buf = [0u8; 8];
    getrandom::fill(&mut buf).expect("Could not open entropy source!");
    let mut rng = StdRand::seed(u64::from_le_bytes(buf));

    let options: Options = argh::from_env();

    //Short Circuits for other things (aside from help)
    // if options.list_cows {
    //     cfg_select! {
    //         feature = "inline-cowsay" => { get_cow_names(); }
    //         not(feature = "inline-cowsay") => { get_cow_names(&options.cows); }
    //     };
    // } else {
    let cow_str = get_cow_string(&options.cows, &mut rng);

    let cow_msg = match options.message {
        Some(msg) => msg,
        None => fortune::get_fortune(
            &options.fortunes,
            &mut rng,
            options.include_offensive,
            options.fortune_width,
            options.fortune_lines,
        )
        .expect("Could not get a fortune, your future is shrouded in mystery..."),
    };

    let cow_variant = match options.cow_variant {
        CowVariant::Random => random_cow_variant(&mut rng),
        _ => options.cow_variant,
    };

    //TODO why did I do this
    let max_width = match options.max_width {
        Some(val) => val as usize,
        None => 64usize,
    };

    print_cowsay(
        &cow_str,
        SpeechBubble::new(options.bubble_type),
        &cow_msg,
        &cow_variant,
        max_width,
    );
    // }
}
