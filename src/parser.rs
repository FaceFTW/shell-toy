use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{digit1, newline},
    combinator::map,
    number::complete::u8,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

//For lack of a better name
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum TerminalCharacter {
    Space,
    DefaultForegroundColor,
    DefaultBackgroundColor,
    TerminalForegroundColor8(u8),
    TerminalForegroundColor24(u32),
    TerminalBackgroundColor8(u8),
    TerminalBackgroundColor24(u32),
    UnicodeCharacter(char),
    ThoughtPlaceholder,
    Newline,
    Comment,
}

fn terminal_chars(input: &str) -> IResult<&str, TerminalCharacter> {
    alt((
        map(
            delimited(tag("\\e["), digit1, tag("m")),
            |esc: &str| match esc {
                "39" => Ok(TerminalCharacter::DefaultForegroundColor),
                "49" => Ok(TerminalCharacter::DefaultBackgroundColor),
                _ => Err("Unknown Escape!"),
            },
        ),
        // map(
        //     delimited(
        //         tag("\\e["),
        //         tuple(digit1, tag(";"), digit1, tag(";"), digit1),
        //         tag("m"),
        //     ),
        //     |esc: &str| todo!(),
        // ),
    ))(input);
    todo!()
}

fn unicode_char(input: &str) -> IResult<&str, TerminalCharacter> {
    todo!()
}

fn spaces_and_lines(input: &str) -> IResult<&str, TerminalCharacter> {
    alt((
        map(tag("\n"), |_| TerminalCharacter::Newline),
        map(tag(" "), |_| TerminalCharacter::Space),
    ))(input)
}

fn terminal_foreground_8(input: &str) -> IResult<&str, TerminalCharacter> {
    todo!()
}
