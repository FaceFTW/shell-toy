use crate::parser::{ParserIterator, TerminalCharacter};
use owo_colors::{DynColor, OwoColorize, Style, XtermColors};
use std::{collections::HashMap, error::Error, str::from_utf8};
#[cfg(not(feature = "inline-cowsay"))]
use std::{
    fs::{self, File},
    io::{self, Read},
    path::PathBuf,
};
use strip_ansi_escapes::strip;
use textwrap::fill;
use tinyrand::Rand;
use unicode_width::UnicodeWidthStr;
/***************************/
//The following code is derived and modified from latipun7/charasay (MIT Licensed Code)
//Original Source Link: https://github.com/latipun7/charasay/blob/main/src/bubbles.rs
/***************************/
#[derive(Debug, Clone, PartialEq)]
pub enum BubbleType {
    Think,
    Round,
    Cowsay,
}

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

macro_rules! speech_bubble {
    ($ctl:literal, $t:literal, $ctr:literal, $tr:literal, $r:literal, $br:literal, $cbr: literal, $b:literal, $cbl:literal,$bl:literal, $l:literal, $tl:literal, $sl:literal, $sr:literal ) => {
        SpeechBubble {
            corner_top_left: $ctl,
            top: $t,
            corner_top_right: $ctr,
            top_right: $tr,
            right: $r,
            bottom_right: $br,
            corner_bottom_right: $cbr,
            bottom: $b,
            corner_bottom_left: $cbl,
            bottom_left: $bl,
            left: $l,
            top_left: $tl,
            short_left: $sl,
            short_right: $sr,
        }
    };
}

const THINK_BUBBLE: SpeechBubble = speech_bubble!(
    "(", "⁀", ")\n", "  )\n", "  )\n", "  )\n", ")\n", "‿", "(", "(  ", "(  ", "(  ", "(  ",
    "  )\n"
);

const ROUND_BUBBLE: SpeechBubble = speech_bubble!(
    "╭", "─", "╮\n", "  │\n", "  │\n", "  │\n", "╯\n", "─", "╰", "│  ", "│  ", "│  ", "│  ",
    "  │\n"
);

const COWSAY_BUBBLE: SpeechBubble = speech_bubble!(
    " ", "_", " \n", "  \\\n", "  |\n", "  /\n", " \n", "-", " ", "\\  ", "|  ", "/  ", "<  ",
    "  >\n"
);

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
#[cfg(feature = "inline-cowsay")]
include!("../target/generated_sources/cow_literals.rs");

//Wrapper type to contain the Style struct so it can be passed recursively
struct StyleBuffer {
    inner: Style,
}

impl StyleBuffer {
    pub fn new() -> Self {
        Self {
            inner: owo_colors::Style::new().default_color().on_default_color(),
        }
    }

    pub fn default_color(&mut self) {
        self.inner = self.inner.default_color()
    }

    pub fn on_default_color(&mut self) {
        self.inner = self.inner.on_default_color()
    }

    pub fn color(&mut self, color: impl DynColor) {
        self.inner = self.inner.color(color)
    }

    pub fn on_color(&mut self, color: impl DynColor) {
        self.inner = self.inner.on_color(color)
    }

    pub fn truecolor(&mut self, red: u8, green: u8, blue: u8) {
        self.inner = self.inner.truecolor(red, green, blue)
    }

    pub fn on_truecolor(&mut self, red: u8, green: u8, blue: u8) {
        self.inner = self.inner.on_truecolor(red, green, blue)
    }
}

#[derive(Clone, Copy)]
pub enum CowVariant {
    Borg,
    Dead,
    Greedy,
    Paranoid,
    Stoned,
    Tired,
    Wired,
    Young,
    Default,
    Random,
}

impl CowVariant {
    fn get_eyes(&self) -> String {
        match self {
            CowVariant::Borg => String::from("=="),
            CowVariant::Dead => String::from("xx"),
            CowVariant::Greedy => String::from("$$"),
            CowVariant::Paranoid => String::from("@@"),
            CowVariant::Stoned => String::from("**"),
            CowVariant::Tired => String::from("--"),
            CowVariant::Wired => String::from("OO"),
            CowVariant::Young => String::from(".."),
            CowVariant::Default => String::from("oo"),
            CowVariant::Random => panic!("This option should not be directly chosen!"),
        }
    }

    fn get_toungue(&self) -> String {
        match self {
            CowVariant::Borg => String::from("  "),
            CowVariant::Dead => String::from("U "),
            CowVariant::Greedy => String::from("  "),
            CowVariant::Paranoid => String::from("  "),
            CowVariant::Stoned => String::from("U "),
            CowVariant::Tired => String::from("  "),
            CowVariant::Wired => String::from("  "),
            CowVariant::Young => String::from("  "),
            CowVariant::Default => String::from("  "),
            CowVariant::Random => panic!("This option should not be directly chosen!"),
        }
    }
}

impl Default for CowVariant {
    fn default() -> Self {
        CowVariant::Default
    }
}

const NON_RAND_VARIANTS: [CowVariant; 9] = [
    CowVariant::Borg,
    CowVariant::Dead,
    CowVariant::Greedy,
    CowVariant::Paranoid,
    CowVariant::Stoned,
    CowVariant::Tired,
    CowVariant::Wired,
    CowVariant::Young,
    CowVariant::Default,
];
pub fn random_cow_variant(rng: &mut impl Rand) -> CowVariant {
    NON_RAND_VARIANTS[rng.next_lim_usize(NON_RAND_VARIANTS.len())]
}

fn derive_cow_str(
    parsed_chars: &[TerminalCharacter],
    current_style: &mut StyleBuffer,
    cow_variant: &CowVariant,
) -> String {
    let mut environment: HashMap<String, Vec<TerminalCharacter>> = HashMap::new();

    let mut cow_started = false;
    //TODO Determine if we should pre-allocate the memory with an "estimate" for performance
    let mut cow_string = String::new();

    for term_char in parsed_chars {
        match term_char {
            TerminalCharacter::Space => {
                cow_string = cow_string + format!("{}", " ".style(current_style.inner)).as_str()
            }
            TerminalCharacter::DefaultForegroundColor => current_style.default_color(),
            TerminalCharacter::DefaultBackgroundColor => current_style.on_default_color(),
            TerminalCharacter::TerminalForegroundColor256(color) => {
                current_style.color(XtermColors::from(*color))
            }
            TerminalCharacter::TerminalForegroundColorTruecolor(red, green, blue) => {
                current_style.truecolor(*red, *green, *blue)
            }
            TerminalCharacter::TerminalBackgroundColor256(color) => {
                current_style.on_color(XtermColors::from(*color))
            }
            TerminalCharacter::TerminalBackgroundColorTruecolor(red, green, blue) => {
                current_style.on_truecolor(*red, *green, *blue);
            }
            TerminalCharacter::UnicodeCharacter(uchar) => {
                cow_string = cow_string + format!("{}", uchar.style(current_style.inner)).as_str()
            }
            TerminalCharacter::ThoughtPlaceholder => cow_string = cow_string + "\\",
            TerminalCharacter::EyePlaceholder => {
                cow_string = cow_string + cow_variant.get_eyes().as_str()
            }
            TerminalCharacter::TonguePlaceholder => {
                cow_string = cow_string + cow_variant.get_toungue().as_str()
            }
            TerminalCharacter::Newline => {
                if cow_started {
                    cow_string = cow_string + "\n";
                }
            }
            TerminalCharacter::Comment => (),
            TerminalCharacter::VarBinding(name, val) => {
                environment.insert(name.to_string(), val.to_vec());
            }
            TerminalCharacter::BoundVarCall(binding) => {
                let binding_val = environment
                    .get(binding)
                    .expect("Could not find a binding with the specified name");
                cow_string =
                    cow_string + derive_cow_str(&binding_val, current_style, cow_variant).as_str();
            }
            TerminalCharacter::CowStart => cow_started = true,
            TerminalCharacter::EscapedUnicodeCharacter(character) => {
                cow_string = cow_string + character.to_string().as_str();
            }
        }
    }

    cow_string
}

//Effectively a main function in the sense it does all the heavy lifting.
pub fn print_cowsay(mut cowsay: &str, bubble: SpeechBubble, msg: &str, cow_variant: &CowVariant) {
    let parser_it = ParserIterator::new(&mut cowsay);

    //Because colors will change before characters are created, we take an owo_colors style
    // and use it as the "current style under tracking". As we created the string, we apply the style necessary to each character
    let mut style_buffer = StyleBuffer::new();
    let cow_str = derive_cow_str(
        parser_it.collect::<Vec<TerminalCharacter>>().as_slice(),
        // scan_it.collect::<Vec<TerminalCharacter>>().as_slice(),
        &mut style_buffer,
        cow_variant,
    );
    let msg_str = bubble
        .create(msg, 64 as usize)
        .expect("Could not create message bubble");

    println!("{msg_str}{cow_str}")
}

#[cfg(feature = "inline-cowsay")]
pub fn choose_random_cow(rng: &mut impl Rand) -> String {
    let chosen_idx = rng.next_lim_usize(COW_DATA.len());
    COW_DATA[chosen_idx].1.to_string()
}

#[cfg(feature = "inline-cowsay")]
pub fn get_cow_by_name(name: &str) -> Option<&str> {
    COW_DATA
        .into_iter()
        .find_map(|item| if item.0 == name { Some(item.1) } else { None })
}

#[cfg(not(feature = "inline-cowsay"))]
fn get_list_of_cows(path: &PathBuf) -> Result<Vec<String>, io::Error> {
    let mut total_list = vec![];
    let dir_list = fs::read_dir(path)?;
    for entry in dir_list {
        match entry {
            Ok(item) => match item.metadata()?.is_dir() {
                true => total_list.append(get_list_of_cows(&item.path()).unwrap().as_mut()),
                false => {
                    if item.path().extension().unwrap() == "cow" {
                        total_list.push(item.path().to_str().unwrap().to_string());
                    }
                }
            },
            Err(e) => return Err(e),
        }
    }
    Ok(total_list)
}

#[cfg(not(feature = "inline-cowsay"))]
pub fn choose_random_cow(cow_path: &PathBuf, rng: &mut impl Rand) -> String {
    let cow_list = get_list_of_cows(&cow_path).expect("Could not open the cow path");

    let chosen_idx = rng.next_lim_usize(cow_list.len());

    let chosen_path = &cow_list[chosen_idx];
    match fs::File::open(chosen_path) {
        Ok(mut file) => {
            let mut cow_str = String::new();
            file.read_to_string(&mut cow_str)
                .expect("Error reading cow string");
            cow_str
        }
        Err(e) => panic!("{e}"),
    }
}

#[cfg(not(feature = "inline-cowsay"))]
pub fn identify_cow_path(defined_path: &Option<String>) -> PathBuf {
    //Check if we have an environment variable defined:
    let os = std::env::consts::OS;
    if let Some(path) = defined_path {
        PathBuf::from(path)
    } else if let Ok(val) = std::env::var("COW_PATH") {
        PathBuf::from(val.as_str())
    } else {
        match os {
            "linux" => PathBuf::from("/usr/share/cowsay/cows"),
            _ => panic!(
                "I don't know what the default path for cowfiles is for this OS!.\nPlease provide a COWPATH or COW_PATH environment variable"
            ),
        }
    }
}

/******************************************
 * Public Facing Methods
 *****************************************/
#[cfg(feature = "inline-cowsay")]
pub fn get_cow_string(cow_file: &Option<String>, rng: &mut impl tinyrand::Rand) -> String {
    if let Some(cow_name) = cow_file {
        get_cow_by_name(&cow_name.as_str())
            .expect("Could not find a cow with the specified name in the inlined data")
            .to_string()
    } else {
        choose_random_cow(rng)
    }
}
#[cfg(feature = "inline-cowsay")]
pub fn get_cow_names() {
    //TODO I wish there was a more expressive way of doing this
    let mut names_vec = COW_DATA.map(|val| val.0);
    names_vec.sort();
    let mut names_iter = names_vec.into_iter();
    let first_name = names_iter.next().unwrap();
    let names_string = names_iter.fold(String::from(first_name), |acc, e| format!("{acc}, {e}"));
    println!("Available Cow Files:\n{names_string}");
}

#[cfg(not(feature = "inline-cowsay"))]
pub fn get_cow_string(
    cow_file: &Option<String>,
    cow_path: &Option<String>,
    rng: &mut impl tinyrand::Rand,
) -> String {
    match cow_file {
        Some(cow_name_or_file) => {
            //do a simple check, if we see no slashes and no .cow suffix,
            // try looking in the cowpath if we can find it
            if !cow_name_or_file.contains(std::path::MAIN_SEPARATOR)
                && !cow_name_or_file.ends_with(".cow")
            {
                let cow_path = identify_cow_path(cow_path);
                let cow_list = get_list_of_cows(&cow_path)
                    .expect("Could not get a list of cows in the identified cow path");
                let file_path = cow_list
                    .into_iter()
                    .find(|item| item.contains(cow_name_or_file) && item.ends_with(".cow"))
                    .expect("Could not find a cow with the specified name in the defined cow path");
                match File::open(file_path) {
                    Ok(mut file) => {
                        let mut cow_str = String::new();
                        file.read_to_string(&mut cow_str)
                            .expect("Error reading Cowfile");
                        cow_str
                    }
                    Err(e) => panic!("{e}"),
                }
            } else {
                match File::open(cow_name_or_file) {
                    Ok(mut file) => {
                        let mut cow_str = String::new();
                        file.read_to_string(&mut cow_str)
                            .expect("Error reading Cowfile");
                        cow_str
                    }
                    Err(e) => panic!("{e}"),
                }
            }
        }
        None => {
            let cow_path = identify_cow_path(cow_path);
            choose_random_cow(&cow_path, rng)
        }
    }
}

#[cfg(not(feature = "inline-cowsay"))]
pub fn get_cow_names(provided_path: &Option<String>) {
    let cow_path = identify_cow_path(provided_path);
    let mut cow_list = get_list_of_cows(&cow_path)
        .expect("Could not get the cows listed in the identified cow path");
    cow_list.sort();
    let mut names_iter = cow_list.into_iter().map(|item| {
        let path = PathBuf::from(item);
        path.file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap()
            .replace(".cow", "")
    });
    let first_name = names_iter.next().unwrap_or_default();
    let names_string = names_iter.fold(String::from(first_name), |acc, e| format!("{acc}, {e}"));
    println!("Available Cow Files:\n{names_string}");
}
