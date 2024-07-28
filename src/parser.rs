///Imaging writing an entire parser for a Perl-like language just for a tiny little tool
/// couldn't be me hahahahahahaha
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_until},
    character::complete::digit1,
    combinator::map,
    error::ParseError,
    sequence::{delimited, preceded, tuple},
    IResult,
};

//For lack of a better name
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum TerminalCharacter {
    Space,
    DefaultForegroundColor,
    DefaultBackgroundColor,
    TerminalForegroundColor256(u8),
    TerminalForegroundColorTruecolor(u8, u8, u8),
    TerminalBackgroundColor256(u8),
    TerminalBackgroundColorTruecolor(u8, u8, u8),
    UnicodeCharacter(char),
    ThoughtPlaceholder,
    EyePlaceholder,
    TonguePlaceholder,
    Newline,
    Comment,
}

fn spaces_and_lines(input: &str) -> IResult<&str, TerminalCharacter> {
    alt((
        map(tag("\r\n"), |_| TerminalCharacter::Newline),
        map(tag("\n"), |_| TerminalCharacter::Newline),
        map(tag(" "), |_| TerminalCharacter::Space),
    ))(input)
}

fn misc_escapes<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, TerminalCharacter, E> {
    map(
        delimited(tag("\\e["), alt((tag("39"), tag("49"))), tag("m")),
        |esc: &str| match esc {
            "39" => TerminalCharacter::DefaultForegroundColor,
            "49" => TerminalCharacter::DefaultBackgroundColor,
            _ => panic!(), //TODO Change this to some error handling
        },
    )(i)
}

fn colors_256<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, TerminalCharacter, E> {
    map(
        delimited(
            tag("\\e["),
            tuple((
                alt((tag("38"), tag("48"))),
                tag(";"),
                tag("5"),
                tag(";"),
                digit1,
            )),
            tag("m"),
        ),
        |(color_type, _, _, _, color)| match color_type {
            "38" => TerminalCharacter::TerminalForegroundColor256(str::parse(color).unwrap()),
            "48" => TerminalCharacter::TerminalBackgroundColor256(str::parse(color).unwrap()),
            _ => panic!(), //TODO change this to some kind of error handling
        },
    )(i)
}

fn truecolor<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, TerminalCharacter, E> {
    map(
        delimited(
            tag("\\e["),
            tuple((
                alt((tag("38"), tag("48"))),
                tag(";"),
                tag("2"),
                tag(";"),
                digit1,
                tag(";"),
                digit1,
                tag(";"),
                digit1,
            )),
            tag("m"),
        ),
        |(color_type, _, _, _, red, _, green, _, blue)| {
            match color_type {
                "38" => TerminalCharacter::TerminalForegroundColorTruecolor(
                    str::parse(red).unwrap(),
                    str::parse(green).unwrap(),
                    str::parse(blue).unwrap(),
                ),
                "48" => TerminalCharacter::TerminalBackgroundColorTruecolor(
                    str::parse(red).unwrap(),
                    str::parse(green).unwrap(),
                    str::parse(blue).unwrap(),
                ),
                _ => panic!(), //TODO change this to some kind of error handling
            }
        },
    )(i)
}

fn unicode_char<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, TerminalCharacter, E> {
    //Turns out, there are different ways to parse unicode escapes.
    //This is my best attempt at covering them
    alt((
        //Xterm -\\N{U+xxxx}
        map(
            delimited(tag("\\N{U+"), take(4 as usize), tag("}")),
            |code: &str| {
                TerminalCharacter::UnicodeCharacter(
                    char::from_u32(u32::from_str_radix(code, 16).unwrap()).unwrap(),
                )
            },
        ),
        //ANSI - \\uxxxx
        map(preceded(tag("\\u"), take(4 as usize)), |code: &str| {
            TerminalCharacter::UnicodeCharacter(
                char::from_u32(u32::from_str_radix(code, 16).unwrap()).unwrap(),
            )
        }),
    ))(i)
}

fn comments<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, TerminalCharacter, E> {
    map(preceded(tag("#"), take_until("\n")), |_| {
        TerminalCharacter::Comment
    })(i)
}

///This parser is for random perl junk we see in files that we want to ignore since we aren't doing perl parsing
fn perl_junk<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, TerminalCharacter, E> {
    alt((map(
        alt((
            tag("EOC\n"),
            tag("EOC\r\n"),
            tag("\"@\n"),
            tag("\"@\r\n"),
            tag("$the_cow = @\"\n"),
            tag("$the_cow = @\"\r\n"),
            tag("$the_cow =<<EOC;\n"),
            tag("$the_cow =<<EOC;\r\n"),
            tag("$the_cow = <<\"EOC\";\n"),
            tag("$the_cow = <<\"EOC\";\r\n"),
            tag("binmode STDOUT, \":utf8\";\n"),
            tag("binmode STDOUT, \":utf8\";\r\n"),
        )),
        |_| TerminalCharacter::Comment,
    ),))(i)
}
fn placeholders<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, TerminalCharacter, E> {
    alt((
        map(tag("$thoughts"), |_| TerminalCharacter::ThoughtPlaceholder),
        map(tag("$eyes"), |_| TerminalCharacter::EyePlaceholder),
        map(tag("$tongue"), |_| TerminalCharacter::TonguePlaceholder),
    ))(i)
}

pub fn cow_parser(input: &str) -> IResult<&str, TerminalCharacter> {
    alt((
        comments,
        perl_junk,
        placeholders,
        spaces_and_lines,
        misc_escapes,
        colors_256,
        truecolor,
        unicode_char,
        map(take(1 as usize), |c: &str| {
            //TODO I don't like this
            TerminalCharacter::UnicodeCharacter(c.chars().into_iter().next().unwrap())
        }),
    ))(input)
}
