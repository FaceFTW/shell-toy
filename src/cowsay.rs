use std::fs::File;

///Holds the representation of the "cows" in a format that we can parse/use
enum Cowsay {
    ///From OG perl cowsay
    Cow {
        repr: Vec<(char, Option<owo_colors::Style>)>,
    },
    ///From charasay
    Chara { repr: Vec<String> },
}

impl Cowsay {
    pub fn from_cowsay_file(path: &str) -> Cowsay {
        todo!()
    }

    pub const fn from_cowsay_str(string: &str) -> Cowsay {
        //
    }
}
