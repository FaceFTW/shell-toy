mod cli;
mod cowsay;
mod fortune;
mod parser;

use cli::Options;
use cowsay::{
    CowVariant, SpeechBubble, get_cow_names, get_cow_string, print_cowsay, random_cow_variant,
};

use tinyrand::{Seeded, StdRand};

fn main() {
    //Init RNG
    let mut buf = [0u8; 8];
    getrandom::fill(&mut buf).expect("Could not open entropy source!");
    let mut rng = StdRand::seed(u64::from_le_bytes(buf));

    let options: Options = argh::from_env();

    //Short Circuits for other things (aside from help)
    if options.list_cows {
        #[cfg(feature = "inline-cowsay")]
        get_cow_names();
        #[cfg(not(feature = "inline-cowsay"))]
        get_cow_names(&options.cow_path);
    } else {
        #[cfg(feature = "inline-cowsay")]
        let cow_str = get_cow_string(&options.cow_file, &mut rng);
        #[cfg(not(feature = "inline-cowsay"))]
        let cow_str = get_cow_string(&options.cow_file, &options.cow_path, &mut rng);

        let cow_msg = match options.message {
            Some(msg) => msg,
            None => {
                cfg_if::cfg_if! {
                    if #[cfg(feature="inline-fortune")]{
                            fortune::get_inline_fortune(&mut rng, options.include_offensive, options.fortune_width, options.fortune_lines)
                                .expect("Could not read internal fortune index, your future is shrouded in mystery...")
                    } else {
                        let fortune_file = fortune::choose_fortune_file(options.include_offensive, &mut rng, options.fortune_file );
                        fortune::get_fortune(fortune_file, &mut rng, options.fortune_width, options.fortune_lines)
                    .expect("Could not get a fortune, your future is shrouded in mystery...")
                    }
                }
            }
        };

        let cow_variant = match options.cow_variant {
            CowVariant::Random => random_cow_variant(&mut rng),
            _ => options.cow_variant,
        };

        //Useful little snippet for debugging cowfiles. Commented out usually
        // if options.enable_debug {
        //     let nom_it: Vec<_> =
        //         nom::combinator::iterator(cow_str.as_str(), parser::cow_parser).collect();
        //         dbg!(nom_it);
        // }

        print_cowsay(
            &cow_str,
            SpeechBubble::new(options.bubble_type),
            &cow_msg,
            &cow_variant,
        );
    }
}
