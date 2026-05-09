use ratatui::style::{Color, Style};

pub const BG: Color = Color::Rgb(0x21, 0x21, 0x21);
pub const SIDEBAR_BG: Color = Color::Rgb(0x25, 0x25, 0x25);
pub const FG: Color = Color::Rgb(0xe0, 0xe0, 0xe0);
pub const COMMENT: Color = Color::Rgb(0x6a, 0x6a, 0x6a);
pub const PRIMARY: Color = Color::Rgb(0xfa, 0xb2, 0x83);
pub const SECONDARY: Color = Color::Rgb(0x5c, 0x9c, 0xf5);
pub const ACCENT: Color = Color::Rgb(0x9d, 0x7c, 0xd8);
pub const BORDER: Color = Color::Rgb(0x4b, 0x4c, 0x5c);
pub const GREEN: Color = Color::Rgb(0x7f, 0xd8, 0x8f);
pub const YELLOW: Color = Color::Rgb(0xe5, 0xc0, 0x7b);
pub const BTN_BORDER_ACTIVE: Color = Color::Rgb(0xa0, 0xcc, 0xff);
pub const BTN_BORDER: Color = Color::Rgb(0x7b, 0xb8, 0xff);
pub const BTN_BG: Color = Color::Rgb(0x5c, 0x9c, 0xf5);
pub const BTN_TEXT: Color = Color::Rgb(0xe8, 0xee, 0xff);

pub fn base() -> Style {
    Style::default().bg(BG)
}

pub fn sidebar_base() -> Style {
    Style::default().bg(SIDEBAR_BG)
}
