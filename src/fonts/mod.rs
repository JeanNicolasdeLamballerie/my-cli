use egui::epaint::text::{FontInsert, InsertFontFamily};

pub struct IconsCodePoints;
impl IconsCodePoints {
    pub const INFO: char = '\u{e88e}';
}

pub struct FONTS;
impl FONTS {
    const ICONS: &str = "ICONS";
    pub const ICONS_MATERIAL_ROUNDED: &str = "ICON_FONT_ROUNDED";
    pub fn icons_family() -> egui::FontFamily {
        egui::FontFamily::Name(Self::ICONS.into())
    }
    pub fn add_rounded_icons(ctx: &egui::Context) {
        let icons = FontInsert::new(
            Self::ICONS_MATERIAL_ROUNDED,
            egui::FontData::from_static(include_bytes!("./material_round_icons.otf")),
            vec![InsertFontFamily {
                family: Self::icons_family(),
                priority: egui::epaint::text::FontPriority::Highest,
            }],
        );
        ctx.add_font(icons);
    }
}

use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
pub enum UnicodeError {
    Int(ParseIntError),
    Unicode(u32),
}

pub fn parse_unicode(input: &str) -> Result<char, UnicodeError> {
    let unicode = u32::from_str_radix(input, 16).map_err(UnicodeError::Int)?;
    char::from_u32(unicode).ok_or_else(|| UnicodeError::Unicode(unicode))
}
