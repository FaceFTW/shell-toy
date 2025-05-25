///Imaging writing an entire parser for a Perl-like language just for a tiny little tool
/// couldn't be me hahahahahahaha
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take, take_until, take_until1},
    character::complete::{alphanumeric1, char, digit1, space0},
    combinator::{map, opt},
    multi::many0,
    sequence::{delimited, preceded},
};

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

fn spaces_and_lines<'a>(input: &'a str) -> IResult<&'a str, TerminalCharacter> {
    alt((
        map(tag("\r\n"), |_| TerminalCharacter::Newline),
        map(tag("\n"), |_| TerminalCharacter::Newline),
        map(tag(" "), |_| TerminalCharacter::Space),
    ))
    .parse(input)
}

fn misc_escapes<'a>(i: &'a str) -> IResult<&'a str, TerminalCharacter> {
    map(
        delimited(tag("\\e["), alt((tag("39"), tag("49"))), tag("m")),
        |esc: &str| match esc {
            "39" => TerminalCharacter::DefaultForegroundColor,
            "49" => TerminalCharacter::DefaultBackgroundColor,
            _ => panic!(), //TODO Change this to some error handling
        },
    )
    .parse(i)
}

fn colors_256<'a>(i: &'a str) -> IResult<&'a str, TerminalCharacter> {
    map(
        delimited(
            tag("\\e["),
            (
                alt((tag("38"), tag("48"))),
                tag(";"),
                tag("5"),
                tag(";"),
                digit1,
            ),
            tag("m"),
        ),
        |(color_type, _, _, _, color)| match color_type {
            "38" => TerminalCharacter::TerminalForegroundColor256(str::parse(color).unwrap()),
            "48" => TerminalCharacter::TerminalBackgroundColor256(str::parse(color).unwrap()),
            _ => panic!(), //TODO change this to some kind of error handling
        },
    )
    .parse(i)
}

fn truecolor<'a>(i: &'a str) -> IResult<&'a str, TerminalCharacter> {
    map(
        delimited(
            tag("\\e["),
            (
                alt((tag("38"), tag("48"))),
                tag(";"),
                tag("2"),
                tag(";"),
                digit1,
                tag(";"),
                digit1,
                tag(";"),
                digit1,
            ),
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
    )
    .parse(i)
}

fn binding_name<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    map((char('$'), alphanumeric1), |(_, name)| name).parse(i)
}

fn binding_value<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    map(
        (
            char('"'),
            take_until1("\";"),
            tag("\";"),
            opt(alt((tag("\n"), tag("\r\n")))),
        ),
        |(_, binding_val, _, _)| binding_val,
    )
    .parse(i)
}

fn var_binding<'a>(i: &'a str) -> IResult<&'a str, TerminalCharacter> {
    //some assumptions of format we will be making here because perl has no BNF
    //1. Vars start with $, and are alphanumeric characters (at least one)
    //2. equals sign padded by any number of spaces on each side
    //3. The "binding's value" starts with a doublequote, then
    // any number of characters up until an ending doublequote AND semicolon
    //4. If the line contianing the binding ends with a newline, take it since this isn't an actual part of the COWFILE
    //5. We are intentionally ignoring any bound variable calls in the binding value since it would literally be a pain in the ass to set up an "environment" for expanding the values
    map(
        (binding_name, space0, char('='), space0, binding_value),
        |(binding_name, _, _, _, binding_value)| {
            let nom_it = nom::combinator::iterator(binding_value, cow_parser_no_vars);
            TerminalCharacter::VarBinding(
                binding_name.to_string(),
                nom_it.collect::<Vec<TerminalCharacter>>(),
            )
        },
    )
    .parse(i)
}

fn bound_var_call<'a>(i: &'a str) -> IResult<&'a str, TerminalCharacter> {
    map(binding_name, |name| {
        TerminalCharacter::BoundVarCall(name.to_string())
    })
    .parse(i)
}

fn unicode_char<'a>(i: &'a str) -> IResult<&'a str, TerminalCharacter> {
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
    ))
    .parse(i)
}

//Fallback for a character that has an explicit escape
fn escaped_char<'a>(i: &'a str) -> IResult<&'a str, TerminalCharacter> {
    map(preceded(tag("\\"), take(1usize)), |character: &str| {
        TerminalCharacter::EscapedUnicodeCharacter(character.chars().next().unwrap())
    })
    .parse(i)
}

fn comments<'a>(i: &'a str) -> IResult<&'a str, TerminalCharacter> {
    map((preceded(tag("#"), take_until("\n")), tag("\n")), |_| {
        TerminalCharacter::Comment
    })
    .parse(i)
}

///This parser is for random perl junk we see in files that we want to ignore since we aren't really a perl interpreter
/// Some of it *is* useful when it comes to acting as a "barrier" between actual text we want to parse
fn perl_junk<'a>(i: &'a str) -> IResult<&'a str, TerminalCharacter> {
    alt((
        map(
            alt((
                tag("EOC\n"),
                tag("EOC\r\n"),
                tag("\"@\n"),
                tag("\"@\r\n"),
                tag("binmode STDOUT, \":utf8\";\n"),
                tag("binmode STDOUT, \":utf8\";\r\n"),
            )),
            |_| TerminalCharacter::Comment,
        ),
        map(
            alt((
                tag("$the_cow = @\"\n"),
                tag("$the_cow = @\"\r\n"),
                tag("$the_cow =<<EOC;\n"),
                tag("$the_cow =<<EOC;\r\n"),
                tag("$the_cow = <<\"EOC\";\n"),
                tag("$the_cow = <<\"EOC\";\r\n"),
                tag("$the_cow = <<EOC;\n"),
                tag("$the_cow = <<EOC;\r\n"),
                tag("$the_cow = << EOC;\n"),
                tag("$the_cow = << EOC;\r\n"),
                tag("$the_cow = << EOC\n"),
                tag("$the_cow = << EOC\r\n"),
                tag("$the_cow = <<EOC\n"),
                tag("$the_cow = <<EOC\r\n"),
            )),
            |_| TerminalCharacter::CowStart,
        ),
    ))
    .parse(i)
}

fn placeholders<'a>(i: &'a str) -> IResult<&'a str, TerminalCharacter> {
    alt((
        map(tag("$thoughts"), |_| TerminalCharacter::ThoughtPlaceholder),
        map(tag("$eyes"), |_| TerminalCharacter::EyePlaceholder),
        map(tag("$tongue"), |_| TerminalCharacter::TonguePlaceholder),
    ))
    .parse(i)
}

fn cow_string<'a>(i: &'a str) -> IResult<&'a str, Vec<TerminalCharacter>> {
    //NOTE this makes a flawed assumption where the perl delimiters don't have to match. But FWIW
    //it is not a significant bug honestly, most of these scripts _should_ be functional in OG perl
    let start = (
        tag("$the_cow"),
        space0,
        tag("="),
        alt((
            map(
                (
                    space0,
                    tag("<<"),
                    space0,
                    alt((tag("\"EOC\""), tag("EOC"))),
                    opt(tag(";")),
                    opt(tag("\r")),
                    tag("\n"),
                ),
                |_| (),
            ),
            map((tag("@\""), opt(tag("\r")), tag("n")), |_| ()),
        )),
    );

    let valid_chars = alt((
        placeholders,
        spaces_and_lines,
        misc_escapes,
        colors_256,
        truecolor,
        unicode_char,
        map(take(1 as usize), |c: &str| {
            TerminalCharacter::UnicodeCharacter(c.chars().into_iter().next().unwrap())
        }),
    ));

    let end = (alt((tag("EOC"), tag("\"@"))), opt(tag("\r")), tag("\n"));
    delimited(start, many0(valid_chars), end)
        .map(|mut chars| {
            chars.insert(0usize, TerminalCharacter::CowStart);
            chars
        })
        .parse(i)
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
        escaped_char,
        map(take(1 as usize), |c: &str| {
            TerminalCharacter::UnicodeCharacter(c.chars().into_iter().next().unwrap())
        }),
    ))
    .parse(input)
}
