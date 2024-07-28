use clap::ValueEnum;
use owo_colors::{OwoColorize, XtermColors};
use std::{error::Error, str::from_utf8};
use strip_ansi_escapes::strip;
use textwrap::fill;
use unicode_width::UnicodeWidthStr;

use crate::parser::TerminalCharacter;
pub fn parse_raw_cow(cow_str: &str, is_think: bool) -> String {
    //This is a really cut and dry method for parsing out the Perl bits
    //that originally existed in cow files
    //THIS WILL BREAK IF WE HAVE COWS USING SPECIAL VAR SUBSTITUTION
    //LIKE THE ONES HERE: https://charc0al.github.io/cowsay-files/converter/

    let thought_char = match is_think {
        true => "o",
        false => "\\",
    };

    cow_str
        .replace("$thoughts", thought_char)
        .replace("$eyes", "o o")
        .replace("$tongue", "  ")
        .split("\n")
        .fold(String::new(), |acc, x| {
            //this might be a "preemptive check"
            let cleaned_str = x.replace("\r", "");
            match cleaned_str.as_str() {
                //TODO this is incredibly brute-force
                "$the_cow = <<\"EOC\";" => acc,
                "$the_cow= <<\"EOC\";" => acc,
                "$the_cow=<<\"EOC\";" => acc,
                "$the_cow =<<\"EOC\";" => acc,
                "EOC" => acc,
                "$the_cow = @\"" => acc,
                "\"@" => acc,
                _ if cleaned_str.starts_with("#") => acc,
                _ => acc + x + "\n",
            }
        })
}

/***************************/
//The following code is derived and modified from latipun7/charasay (MIT Licensed Code)
//Original Source Link: https://github.com/latipun7/charasay/blob/main/src/bubbles.rs
/***************************/
#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum BubbleType {
    Think,
    Round,
    Cowsay,
}

const THINK_BUBBLE: SpeechBubble = SpeechBubble {
    corner_top_left: "(",
    top: "⁀",
    corner_top_right: ")\n",
    top_right: "  )\n",
    right: "  )\n",
    bottom_right: "  )\n",
    corner_bottom_right: ")\n",
    bottom: "‿",
    corner_bottom_left: "(",
    bottom_left: "(  ",
    left: "(  ",
    top_left: "(  ",
    short_left: "(  ",
    short_right: "  )\n",
};

const ROUND_BUBBLE: SpeechBubble = SpeechBubble {
    corner_top_left: "╭",
    top: "─",
    corner_top_right: "╮\n",
    top_right: "  │\n",
    right: "  │\n",
    bottom_right: "  │\n",
    corner_bottom_right: "╯\n",
    bottom: "─",
    corner_bottom_left: "╰",
    bottom_left: "│  ",
    left: "│  ",
    top_left: "│  ",
    short_left: "│  ",
    short_right: "  │\n",
};

const COWSAY_BUBBLE: SpeechBubble = SpeechBubble {
    corner_top_left: " ",
    top: "_",
    corner_top_right: " \n",
    top_right: "  \\\n",
    right: "  |\n",
    bottom_right: "  /\n",
    corner_bottom_right: " \n",
    bottom: "-",
    corner_bottom_left: " ",
    bottom_left: "\\  ",
    left: "|  ",
    top_left: "/  ",
    short_left: "<  ",
    short_right: "  >\n",
};

#[derive(Debug, Clone)]
pub struct SpeechBubble {
    corner_top_left: &'static str,
    top: &'static str,
    corner_top_right: &'static str,
    top_right: &'static str,
    right: &'static str,
    bottom_right: &'static str,
    corner_bottom_right: &'static str,
    bottom: &'static str,
    corner_bottom_left: &'static str,
    bottom_left: &'static str,
    left: &'static str,
    top_left: &'static str,
    short_left: &'static str,
    short_right: &'static str,
}

impl SpeechBubble {
    pub fn new(bubble_type: BubbleType) -> Self {
        match bubble_type {
            BubbleType::Think => THINK_BUBBLE,
            BubbleType::Round => ROUND_BUBBLE,
            BubbleType::Cowsay => COWSAY_BUBBLE,
        }
    }

    fn line_len(line: &str) -> Result<usize, Box<dyn Error>> {
        let stripped = strip(line);
        let text = from_utf8(stripped.as_slice());

        Ok(text.map(UnicodeWidthStr::width).unwrap_or(0))
    }

    fn longest_line(lines: &[&str]) -> Result<usize, Box<dyn Error>> {
        let line_lengths = lines
            .iter()
            .map(|line| Self::line_len(line))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(line_lengths.into_iter().max().unwrap_or(0))
    }

    pub fn create(self, messages: &str, max_width: usize) -> Result<String, Box<dyn Error>> {
        const SPACE: &str = " ";
        let wrapped = fill(messages, max_width).replace('\t', "    ");
        let lines: Vec<&str> = wrapped.lines().collect();
        let line_count = lines.len();
        let actual_width = Self::longest_line(&lines)?;

        let total_size_buffer = (actual_width + 5) * 2 + line_count * (actual_width + 6);

        let mut write_buffer = Vec::with_capacity(total_size_buffer);

        // draw top box border
        write_buffer.push(self.corner_top_left);
        for _ in 0..(actual_width + 4) {
            write_buffer.push(self.top);
        }
        write_buffer.push(self.corner_top_right);

        // draw inner borders & messages
        for (i, line) in lines.into_iter().enumerate() {
            let left_border = match (line_count, i) {
                (1, _) => self.short_left,
                (_, 0) => self.top_left,
                (_, i) if i == line_count - 1 => self.bottom_left,
                _ => self.left,
            };
            write_buffer.push(left_border);

            let line_len = Self::line_len(line)?;
            write_buffer.push(line);
            write_buffer.resize(write_buffer.len() + actual_width - line_len, SPACE);

            let right_border = match (line_count, i) {
                (1, _) => self.short_right,
                (_, 0) => self.top_right,
                (_, i) if i == line_count - 1 => self.bottom_right,
                _ => self.right,
            };
            write_buffer.push(right_border);
        }

        // draw bottom box border
        write_buffer.push(self.corner_bottom_left);
        for _ in 0..(actual_width + 4) {
            write_buffer.push(self.bottom);
        }
        write_buffer.push(self.corner_bottom_right);

        Ok(write_buffer.join(""))
    }
}
/***************************/
//End Derived Code
/***************************/

//Effectively a main function in the sense it does all the heavy lifting.
pub fn print_cowsay(cowsay: &str, bubble: SpeechBubble, msg: &str) {
    let cow_str = parse_raw_cow(cowsay, false);
    let msg_str = bubble
        .create(msg, 64 as usize)
        .expect("Could not create message bubble");

    println!("{msg_str}{cow_str}")
}

pub fn derive_cow_str(parsed_chars: &[TerminalCharacter]) -> String {
    //Because colors will change before characters are created, we take an owo_colors style
    // and use it as the "current style under tracking". As we created the string, we apply the style necessary to each character
    let mut current_style = owo_colors::Style::new().default_color();
    //TODO Determine if we should pre-allocate the memory with an "estimate" for performance
    let mut cow_string = String::new();
    for term_char in parsed_chars {
        match term_char {
            TerminalCharacter::Space => {
                cow_string = cow_string + format!("{}", " ".style(current_style)).as_str()
            }
            TerminalCharacter::DefaultForegroundColor => {
                current_style = current_style.default_color()
            }
            TerminalCharacter::DefaultBackgroundColor => {
                current_style = current_style.on_default_color()
            } //TODO how to set owo_colors background default
            TerminalCharacter::TerminalForegroundColor256(color) => {
                current_style = current_style.color(XtermColors::from(*color))
            }
            TerminalCharacter::TerminalForegroundColorTruecolor(red, green, blue) => {
                current_style = current_style.truecolor(*red, *green, *blue)
            }
            TerminalCharacter::TerminalBackgroundColor256(color) => {
                current_style = current_style.on_color(XtermColors::from(*color))
            }
            TerminalCharacter::TerminalBackgroundColorTruecolor(red, green, blue) => {
                current_style = current_style.on_truecolor(*red, *green, *blue);
            }
            TerminalCharacter::UnicodeCharacter(uchar) => {
                cow_string = cow_string + format!("{}", uchar.style(current_style)).as_str()
            }
            TerminalCharacter::ThoughtPlaceholder => cow_string = cow_string + "\\",
            TerminalCharacter::EyePlaceholder => cow_string = cow_string + "o o",
            TerminalCharacter::TonguePlaceholder => cow_string = cow_string + "  ",
            TerminalCharacter::Newline => cow_string = cow_string + "\n",
            TerminalCharacter::Comment => (),
        }
    }

    cow_string
}
