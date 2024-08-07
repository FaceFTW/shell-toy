///Imaging writing an entire parser for a Perl-like language just for a tiny little tool
/// couldn't be me hahahahahahaha
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_till, take_until, take_until1},
    character::complete::{alphanumeric1, char, digit1, space0},
    combinator::{map, opt},
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
    VarBinding(String, Vec<TerminalCharacter>), //Think in terms of s-expr-like interpretation then this makes sense
    BoundVarCall(String),
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

fn binding_name<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    map(tuple((char('$'), alphanumeric1)), |(_, name)| name)(i)
}

fn binding_value<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    map(
        tuple((
            char('"'),
            take_until1("\";"),
            tag("\";"),
            opt(alt((tag("\n"), tag("\r\n")))),
        )),
        |(_, binding_val, _, _)| binding_val,
    )(i)
}

fn var_binding<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, TerminalCharacter, E> {
    //some assumptions of format we will be making here because perl has no BNF
    //1. Vars start with $, and are alphanumeric characters (at least one)
    //2. equals sign padded by any number of spaces on each side
    //3. The "binding's value" starts with a doublequote, then
    // any number of characters up until an ending doublequote AND semicolon
    //4. If the line contianing the binding ends with a newline, take it since this isn't an actual part of the COWFILE
    //5. We are intentionally ignoring any bound variable calls in the binding value since it would literally be a pain in the ass to set up an "environment" for expanding the values
    map(
        tuple((binding_name, space0, char('='), space0, binding_value)),
        |(binding_name, _, _, _, binding_value)| {
            let mut nom_it = nom::combinator::iterator(binding_value, cow_parser_no_vars);
            TerminalCharacter::VarBinding(
                binding_name.to_string(),
                nom_it.collect::<Vec<TerminalCharacter>>(),
            )
        },
    )(i)
}

fn bound_var_call<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, TerminalCharacter, E> {
    map(binding_name, |name| {
        TerminalCharacter::BoundVarCall(name.to_string())
    })(i)
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

///This parser is for random perl junk we see in files that we want to ignore since we aren't really a perl interpreter
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
            tag("$the_cow = <<EOC;\n"),
            tag("$the_cow = <<EOC;\r\n"),
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

///This is a variant of the main parser that ignores bindings for the sake of preventing
/// recursive binding parsing since we have no "environment" to fetch from
fn cow_parser_no_vars(input: &str) -> IResult<&str, TerminalCharacter> {
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

pub fn cow_parser(input: &str) -> IResult<&str, TerminalCharacter> {
    alt((
        comments,
        perl_junk,
        placeholders,
        var_binding,
        bound_var_call,
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
