// pub struct TerminalColor {
//     red: u8,
//     green: u8,
//     blue: u8,
// }

//For lack of a better name
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum TerminalCharacter {
    Space,
    TerminalEscape(),
    TerminalForegroundColor8(u8),
    TerminalForegroundColor24(u32),
    TerminalBackgroundColor8(u8),
    TerminalBackgroundColor24(u32),
    UnicodeCharacter(char),
    ThoughtPlaceholder,
    Newline,
}
