///Imaging writing an entire parser for a Perl-like language just for a tiny little tool
/// couldn't be me hahahahahahaha
use winnow::{
    Parser,
    ascii::{alphanumeric1, digit1, space0},
    combinator::{alt, delimited, fail, preceded, repeat_till, terminated},
    error::{AddContext, ContextError, ParserError, StrContext},
    token::{literal, take, take_until},
};

type ParserResult<O, E> = winnow::Result<O, E>;
type InputStream<'i> = &'i str;

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

fn spaces_and_lines<
    'a,
    E: ParserError<InputStream<'a>> + AddContext<InputStream<'a>, StrContext>,
>(
    input: &mut InputStream<'a>,
) -> ParserResult<TerminalCharacter, E> {
    alt((
        literal("\r\n").map(|_| TerminalCharacter::Newline),
        literal("\n").map(|_| TerminalCharacter::Newline),
        literal(" ").map(|_| TerminalCharacter::Space),
    ))
    .parse_next(input)
}

fn term_color<'a, E: ParserError<InputStream<'a>> + AddContext<InputStream<'a>, StrContext>>(
    input: &mut InputStream<'a>,
) -> ParserResult<TerminalCharacter, E> {
    let mut fg_color = winnow::dispatch! {delimited(literal(";"), take(1usize), literal(";"));
        "5" => terminated(digit1, literal("m")).map(|num|TerminalCharacter::TerminalForegroundColor256(str::parse(num).unwrap())),
        "2" => terminated((digit1 ,literal(";"), digit1, literal(";"), digit1), literal("m")).map(|(red, _ , green, _, blue)|{
             TerminalCharacter::TerminalForegroundColorTruecolor(
                str::parse(red).unwrap(),
                str::parse(green).unwrap(),
                str::parse(blue).unwrap(),
            )
        }),
        _ => fail
    };

    let mut bg_color = winnow::dispatch! {delimited(literal(";"), take(1usize), literal(";"));
        "5" => terminated(digit1, literal("m")).map(|num|TerminalCharacter::TerminalBackgroundColor256(str::parse(num).unwrap())),
        "2" => terminated((digit1 ,literal(";"), digit1, literal(";"), digit1), literal("m")).map(|(red, _ , green, _, blue)|{
             TerminalCharacter::TerminalBackgroundColorTruecolor(
                str::parse(red).unwrap(),
                str::parse(green).unwrap(),
                str::parse(blue).unwrap(),
            )
        }),
        _ => fail
    };

    winnow::dispatch! {preceded(literal("\\e["), take(2usize));
        "39" => literal("m").map(|_| TerminalCharacter::DefaultForegroundColor),
        "49" => literal("m").map(|_| TerminalCharacter::DefaultBackgroundColor),
        "38" => fg_color,
        "48" => bg_color,
        _ => fail
    }
    .parse_next(input)
}

fn unicode_char<'a, E: ParserError<InputStream<'a>> + AddContext<InputStream<'a>, StrContext>>(
    input: &mut InputStream<'a>,
) -> ParserResult<TerminalCharacter, E> {
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
fn escaped_char<'a, E: ParserError<InputStream<'a>> + AddContext<InputStream<'a>, StrContext>>(
    input: &mut InputStream<'a>,
) -> ParserResult<TerminalCharacter, E> {
    preceded(literal("\\"), take(1usize))
        .map(|character: &str| {
            TerminalCharacter::EscapedUnicodeCharacter(character.chars().next().unwrap())
        })
        .parse_next(input)
}

fn comments<'a, E: ParserError<InputStream<'a>> + AddContext<InputStream<'a>, StrContext>>(
    input: &mut InputStream<'a>,
) -> ParserResult<TerminalCharacter, E> {
    delimited(literal("#"), take_until(1.., "\n"), literal("\n"))
        .map(|_| TerminalCharacter::Comment)
        .parse_next(input)
}

///This parser is for random perl junk we see in files that we want to ignore since we aren't really a perl interpreter
/// Some of it *is* useful when it comes to acting as a "barrier" between actual text we want to parse
fn perl_junk<'a, E: ParserError<InputStream<'a>> + AddContext<InputStream<'a>, StrContext>>(
    input: &mut InputStream<'a>,
) -> ParserResult<TerminalCharacter, E> {
    alt((
        literal("binmode STDOUT, \":utf8\";\n"),
        literal("binmode STDOUT, \":utf8\";\r\n"),
    ))
    .map(|_| TerminalCharacter::Comment)
    .parse_next(input)
}

fn placeholders<'a, E: ParserError<InputStream<'a>> + AddContext<InputStream<'a>, StrContext>>(
    input: &mut InputStream<'a>,
) -> ParserResult<TerminalCharacter, E> {
    alt((
        literal("$thoughts").map(|_| TerminalCharacter::ThoughtPlaceholder),
        literal("$eyes").map(|_| TerminalCharacter::EyePlaceholder),
        literal("$tongue").map(|_| TerminalCharacter::TonguePlaceholder),
    ))
    .parse_next(input)
}

fn binding_name<'a, E: ParserError<InputStream<'a>> + AddContext<InputStream<'a>, StrContext>>(
    input: &mut InputStream<'a>,
) -> ParserResult<&'a str, E> {
    preceded(literal("$"), alphanumeric1).parse_next(input)
}

fn binding_value<'a, E: ParserError<InputStream<'a>> + AddContext<InputStream<'a>, StrContext>>(
    input: &mut InputStream<'a>,
) -> ParserResult<Vec<TerminalCharacter>, E> {
    preceded(
        literal("\""),
        repeat_till(
            0..,
            alt((
                placeholders,
                spaces_and_lines,
                term_color,
                unicode_char,
                escaped_char,
                take(1 as usize).map(|c: &str| {
                    TerminalCharacter::UnicodeCharacter(c.chars().into_iter().next().unwrap())
                }),
            )),
            literal("\";"),
        ),
    )
    .map(|(binding_val, _): (Vec<TerminalCharacter>, _)| binding_val)
    .parse_next(input)
}

fn var_binding<'a, E: ParserError<InputStream<'a>> + AddContext<InputStream<'a>, StrContext>>(
    input: &mut InputStream<'a>,
) -> ParserResult<TerminalCharacter, E> {
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

fn bound_var_call<'a, E: ParserError<InputStream<'a>> + AddContext<InputStream<'a>, StrContext>>(
    input: &mut InputStream<'a>,
) -> ParserResult<TerminalCharacter, E> {
    preceded(literal("$"), alphanumeric1)
        .map(|name: &'a str| TerminalCharacter::BoundVarCall(name.to_string()))
        .parse_next(input)
}

fn cow_string<'a, E: ParserError<InputStream<'a>> + AddContext<InputStream<'a>, StrContext>>(
    input: &mut InputStream<'a>,
) -> ParserResult<Vec<TerminalCharacter>, E> {
    //NOTE this makes a flawed assumption where the perl delimiters don't have to match. But FWIW
    //it is not a significant bug honestly, most of these scripts _should_ have been proper in the OG perl
    let start = (
        literal("$the_cow"),
        space0,
        literal("="),
        space0,
        alt((
            alt((literal("@\"\r\n"), literal("@\"\n"))),
            (
                literal("<<"),
                space0,
                alt((
                    literal("EOC\r\n"),
                    literal("EOC\n"),
                    literal("EOC;\r\n"),
                    literal("EOC;\n"),
                    literal("\"EOC\"\r\n"),
                    literal("\"EOC\"\n"),
                    literal("\"EOC\";\r\n"),
                    literal("\"EOC\";\n"),
                )),
            )
                .value(""),
        )),
    );

    preceded(
        start,
        repeat_till(
            1..,
            alt((
                spaces_and_lines,
                placeholders,
                term_color,
                unicode_char,
                escaped_char,
                bound_var_call,
                take(1 as usize).map(|c: &str| {
                    TerminalCharacter::UnicodeCharacter(c.chars().into_iter().next().unwrap())
                }),
            )),
            alt((
                literal::<InputStream<'a>, InputStream<'a>, E>("EOC\r\n"),
                literal::<InputStream<'a>, InputStream<'a>, E>("EOC\n"),
                literal::<InputStream<'a>, InputStream<'a>, E>("\"@\r\n"),
                literal::<InputStream<'a>, InputStream<'a>, E>("\"@\n"),
            )),
        ),
    )
    .map(|(mut chars, _): (Vec<TerminalCharacter>, _)| {
        chars.insert(0usize, TerminalCharacter::CowStart);
        chars
    })
    .parse_next(input)
}

fn cow_parser<'a, E: ParserError<InputStream<'a>> + AddContext<InputStream<'a>, StrContext>>(
    input: &mut InputStream<'a>,
) -> ParserResult<Vec<TerminalCharacter>, E> {
    alt((
        comments.map(|comment| vec![comment]),
        spaces_and_lines.map(|whitespace| vec![whitespace]),
        perl_junk.map(|junk| vec![junk]),
        cow_string,
        var_binding.map(|binding| vec![binding]),
    ))
    .parse_next(input)
}

pub struct ParserIterator<'i> {
    stream: InputStream<'i>,
    cow_started: bool,
    prev_new_line: bool,
    //NOTE Technically, better is to use generics, but we know what the internal iterator is
    //composed of, so generic impl doesn't make too much sense
    parsed_iter: Option<std::vec::IntoIter<TerminalCharacter>>,
}

impl<'i> ParserIterator<'i> {
    pub fn new(input: &mut InputStream<'i>) -> Self {
        Self {
            stream: &input,
            cow_started: false,
            prev_new_line: false,
            parsed_iter: None,
        }
    }
}

impl<'i> Iterator for ParserIterator<'i> {
    type Item = TerminalCharacter;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parsed_iter {
            Some(ref mut iter) => match (iter).next() {
                Some(val) => Some(val),
                None => {
                    self.parsed_iter = None;
                    self.next() //NOTE this is obviously recursive. Should be a way to "fuse" iter here (effectively)
                }
            },
            None => {
                match cow_parser::<ContextError>.parse_next(&mut self.stream) {
                    Ok(parsed) => {
                        let iter = parsed.into_iter().filter(|parsed| match parsed {
                            TerminalCharacter::Newline => {
                                if self.prev_new_line {
                                    false
                                } else {
                                    self.prev_new_line = self.cow_started;
                                    true
                                }
                            }
                            TerminalCharacter::Comment => true, //Should not alter newline state since it's not interpreted
                            TerminalCharacter::CowStart => {
                                self.cow_started = true;
                                true
                            }
                            _ => {
                                self.prev_new_line = false;
                                true
                            }
                        });

                        //HACK This is _really_ bad code
                        //Basically since I can't really constrain the generic type of the filter predicate generic type
                        // since closures can't be calculated that way and _this_ closure requires messing with this
                        // iterator's internal state, I'm going to allocate a collected vector, then use _that vector's_
                        // iterator as the reference iterator.
                        //I can't think of a workaround. If you have a better solution, send patches and stop complaining
                        // because I hate it too.
                        let filtered: Vec<TerminalCharacter> = iter.collect();

                        self.parsed_iter = Some(filtered.into_iter());

                        self.next()
                    }
                    Err(_parse_err) => None,
                }
            }
        }
    }
}
