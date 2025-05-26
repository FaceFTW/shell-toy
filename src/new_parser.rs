// use crate::parser::TerminalCharacter;
use winnow::{
    Parser,
    ascii::{alphanumeric1, digit1, space0},
    combinator::{alt, delimited, opt, preceded, repeat, repeat_till},
    error::{AddContext, ContextError, ParserError, StrContext},
    token::{literal, take, take_until},
};

type WResult<O, E> = winnow::Result<O, E>;
type Stream<'i> = &'i str;

//For lack of a better name
#[derive(Debug, PartialEq, Clone)]
pub enum TerminalCharacter {
    Space,
    DefaultForegroundColor,
    DefaultBackgroundColor,
    TerminalForegroundColor256(u8),
    TerminalForegroundColorTruecolor(u8, u8, u8),
    TerminalBackgroundColor256(u8),
    TerminalBackgroundColorTruecolor(u8, u8, u8),
    UnicodeCharacter(char),
    EscapedUnicodeCharacter(char),
    ThoughtPlaceholder,
    EyePlaceholder,
    TonguePlaceholder,
    Newline,
    Comment,
    VarBinding(String, Vec<TerminalCharacter>), //Think in terms of s-expr-like interpretation then this makes sense
    BoundVarCall(String),
    CowStart,
}

fn spaces_and_lines<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<TerminalCharacter, E> {
    alt((
        literal("\r\n").map(|_| TerminalCharacter::Newline),
        literal("\n").map(|_| TerminalCharacter::Newline),
        literal(" ").map(|_| TerminalCharacter::Space),
    ))
    .parse_next(input)
}

fn misc_escapes<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<TerminalCharacter, E> {
    delimited(
        literal("\\e["),
        alt((literal("39"), literal("49"))),
        literal("m"),
    )
    .map(|esc: &str| match esc {
        "39" => TerminalCharacter::DefaultForegroundColor,
        "49" => TerminalCharacter::DefaultBackgroundColor,
        _ => panic!(), //TODO Change this to some error handling
    })
    .parse_next(input)
}

fn colors_256<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<TerminalCharacter, E> {
    delimited(
        literal("\\e["),
        (
            alt((literal("38"), literal("48"))),
            literal(";"),
            literal("5"),
            literal(";"),
            digit1,
        ),
        literal("m"),
    )
    .map(|(color_type, _, _, _, color)| match color_type {
        "38" => TerminalCharacter::TerminalForegroundColor256(str::parse(color).unwrap()),
        "48" => TerminalCharacter::TerminalBackgroundColor256(str::parse(color).unwrap()),
        _ => panic!(), //TODO change this to some kind of error handling
    })
    .parse_next(input)
}

fn truecolor<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<TerminalCharacter, E> {
    delimited(
        literal("\\e["),
        (
            alt((literal("38"), literal("48"))),
            literal(";"),
            literal("2"),
            literal(";"),
            digit1,
            literal(";"),
            digit1,
            literal(";"),
            digit1,
        ),
        literal("m"),
    )
    .map(|(color_type, _, _, _, red, _, green, _, blue)| {
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
    })
    .parse_next(input)
}

fn unicode_char<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<TerminalCharacter, E> {
    //Turns out, there are different ways to parse unicode escapes.
    //This is my best attempt at covering them
    alt((
        //Xterm -\\N{U+xxxx}
        delimited(literal("\\N{U+"), take(4 as usize), literal("}")).map(|code: &str| {
            TerminalCharacter::UnicodeCharacter(
                char::from_u32(u32::from_str_radix(code, 16).unwrap()).unwrap(),
            )
        }),
        //ANSI - \\uxxxx
        preceded(literal("\\u"), take(4 as usize)).map(|code: &str| {
            TerminalCharacter::UnicodeCharacter(
                char::from_u32(u32::from_str_radix(code, 16).unwrap()).unwrap(),
            )
        }),
    ))
    .parse_next(input)
}

//Fallback for a character that has an explicit escape
fn escaped_char<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<TerminalCharacter, E> {
    preceded(literal("\\"), take(1usize))
        .map(|character: &str| {
            TerminalCharacter::EscapedUnicodeCharacter(character.chars().next().unwrap())
        })
        .parse_next(input)
}

fn comments<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<TerminalCharacter, E> {
    delimited(literal("#"), take_until(1.., "\n"), literal("\n"))
        .map(|_| TerminalCharacter::Comment)
        .parse_next(input)
}

///This parser is for random perl junk we see in files that we want to ignore since we aren't really a perl interpreter
/// Some of it *is* useful when it comes to acting as a "barrier" between actual text we want to parse
fn perl_junk<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<TerminalCharacter, E> {
    alt((
        literal("binmode STDOUT, \":utf8\";\n"),
        literal("binmode STDOUT, \":utf8\";\r\n"),
    ))
    .map(|_| TerminalCharacter::Comment)
    .parse_next(input)
}

fn placeholders<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<TerminalCharacter, E> {
    alt((
        literal("$thoughts").map(|_| TerminalCharacter::ThoughtPlaceholder),
        literal("$eyes").map(|_| TerminalCharacter::EyePlaceholder),
        literal("$tongue").map(|_| TerminalCharacter::TonguePlaceholder),
    ))
    .parse_next(input)
}

fn binding_name<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<&'a str, E> {
    preceded(literal("$"), alphanumeric1).parse_next(input)
}

fn binding_value<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<Vec<TerminalCharacter>, E> {
    delimited(
        literal("\""),
        repeat_till(
            0..,
            alt((
                placeholders,
                spaces_and_lines,
                escaped_char,
                misc_escapes,
                colors_256,
                truecolor,
                unicode_char,
                take(1 as usize).map(|c: &str| {
                    TerminalCharacter::UnicodeCharacter(c.chars().into_iter().next().unwrap())
                }),
            )),
            literal("\";"),
        ),
        (literal("\";"), opt(alt((literal("\n"), literal("\r\n"))))),
    )
    .map(|(binding_val, _): (Vec<TerminalCharacter>, _)| binding_val)
    .parse_next(input)
}

fn var_binding<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<TerminalCharacter, E> {
    //some assumptions of format we will be making here because perl has no BNF
    //1. Vars start with $, and are alphanumeric characters (at least one)
    //2. equals sign padded by any number of spaces on each side
    //3. The "binding's value" starts with a doublequote, then
    // any number of characters up until an ending doublequote AND semicolon
    //4. If the line contianing the binding ends with a newline, take it since this isn't an actual part of the COWFILE
    //5. We are intentionally ignoring any bound variable calls in the binding value since it would literally be a pain in the ass to set up an "environment" for expanding the values
    (binding_name, space0, literal("="), space0, binding_value)
        .map(|(binding_name, _, _, _, binding_value)| {
            TerminalCharacter::VarBinding(binding_name.to_string(), binding_value)
        })
        .parse_next(input)
}

fn bound_var_call<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<TerminalCharacter, E> {
    preceded(literal("$"), alphanumeric1)
        .map(|name: &'a str| TerminalCharacter::BoundVarCall(name.to_string()))
        .parse_next(input)
}

fn cow_string<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<Vec<TerminalCharacter>, E> {
    //NOTE this makes a flawed assumption where the perl delimiters don't have to match. But FWIW
    //it is not a significant bug honestly, most of these scripts _should_ be functional in OG perl
    let start = (
        literal("$the_cow"),
        space0,
        literal("="),
        space0,
        literal("<<"),
        space0,
        //NOTE This is easier than trying to form a bunch of sub parsers honestly
        alt((
            literal("\"EOC\"\r\n"),
            literal("\"EOC\"\n"),
            literal("\"EOC\";\r\n"),
            literal("\"EOC\";\n"),
            literal("EOC\r\n"),
            literal("EOC\n"),
            literal("EOC;\r\n"),
            literal("EOC;\n"),
            literal("@\"\r\n"),
            literal("@\"\n"),
        )),
    );

    preceded(
        start,
        repeat_till(
            1..,
            alt((
                spaces_and_lines,
                placeholders,
                escaped_char,
                misc_escapes,
                colors_256,
                truecolor,
                unicode_char,
                bound_var_call,
                take(1 as usize).map(|c: &str| {
                    TerminalCharacter::UnicodeCharacter(c.chars().into_iter().next().unwrap())
                }),
            )),
            alt((
                literal::<Stream<'a>, Stream<'a>, E>("EOC\r\n"),
                literal::<Stream<'a>, Stream<'a>, E>("EOC\n"),
                literal::<Stream<'a>, Stream<'a>, E>("\"@\r\n"),
                literal::<Stream<'a>, Stream<'a>, E>("\"@\n"),
            )),
        ),
        // alt((
        //     literal::<Stream<'a>, Stream<'a>, E>("EOC\r\n"),
        //     literal::<Stream<'a>, Stream<'a>, E>("EOC\n"),
        //     literal::<Stream<'a>, Stream<'a>, E>("\"@\r\n"),
        //     literal::<Stream<'a>, Stream<'a>, E>("\"@\n"),
        // ))
        // .context(StrContext::Label("cowstring_end")),
    )
    .map(|(mut chars, _): (Vec<TerminalCharacter>, _)| {
        chars.push(TerminalCharacter::CowStart);
        chars
    })
    .parse_next(input)
}

pub fn cow_parser<'a, E: ParserError<Stream<'a>> + AddContext<Stream<'a>, StrContext>>(
    input: &mut Stream<'a>,
) -> WResult<Vec<TerminalCharacter>, E> {
    alt((
        comments.map(|comment| vec![comment]),
        spaces_and_lines.map(|whitespace| vec![whitespace]),
        perl_junk.map(|junk| vec![junk]),
        cow_string,
        var_binding.map(|binding| vec![binding]),
    ))
    // .parse(input)
    .parse_next(input)
}

pub struct ParserIterator<'i> {
    stream: Stream<'i>,
    // cow_started: bool,
    // prev_new_line: bool,
}

impl<'i> ParserIterator<'i> {
    pub fn new(input: &mut Stream<'i>) -> Self {
        Self {
            stream: &input,
            // cow_started: false,
            // prev_new_line: false,
        }
    }
}

impl<'i> Iterator for ParserIterator<'i> {
    type Item = Vec<TerminalCharacter>;

    fn next(&mut self) -> Option<Self::Item> {
        match cow_parser::<ContextError>.parse_next(&mut self.stream) {
            Ok(parsed) => Some(parsed),
            Err(_parse_err) => None, //NOTE this is probably flawed
        }
    }
}
