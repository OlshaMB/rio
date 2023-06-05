use font_kit::{properties::Style, source::SystemSource};
use glyph_brush::ab_glyph::{FontArc, FontVec};
use log::warn;

pub const DEFAULT_FONT_NAME: &str = "cascadiamono";

pub const FONT_CASCADIAMONO_REGULAR: &[u8; 308212] =
    include_bytes!("./resources/CascadiaMono/CascadiaMonoPL-Regular.otf");

pub const FONT_CASCADIAMONO_BOLD: &[u8; 312976] =
    include_bytes!("./resources/CascadiaMono/CascadiaMonoPL-Bold.otf");

pub const FONT_CASCADIAMONO_ITALIC: &[u8; 191296] =
    include_bytes!("./resources/CascadiaMono/CascadiaMonoPL-Italic.otf");

pub const FONT_CASCADIAMONO_BOLD_ITALIC: &[u8; 193360] =
    include_bytes!("./resources/CascadiaMono/CascadiaMonoPL-BoldItalic.otf");

pub const FONT_EMOJI: &[u8; 877988] =
    include_bytes!("./resources/NotoEmoji/static/NotoEmoji-Regular.ttf");

#[cfg(not(target_os = "macos"))]
pub const FONT_DEJAVU_MONO: &[u8; 340712] =
    include_bytes!("./resources/DejaVuSansMono.ttf");
#[derive(Debug, Clone)]
pub struct ComposedFontArc {
    pub regular: FontArc,
    pub bold: FontArc,
    pub italic: FontArc,
    pub bold_italic: FontArc,
}

pub struct Font {
    pub text: ComposedFontArc,
    pub symbol: FontArc,
    pub emojis: FontArc,
    pub unicode: FontArc,
}
fn font_arc_from_font(font: font_kit::font::Font) -> Option<FontArc> {
    let copied_font = font.copy_font_data();
    Some(FontArc::new(
        FontVec::try_from_vec_and_index(copied_font?.to_vec(), 0).unwrap(),
    ))
}
impl Font {
    // TODO: Refactor multiple unwraps in this code
    // TODO: Use FontAttributes bold and italic
    pub fn new(font_name: String) -> Font {
        let font_arc_unicode;
        let font_arc_symbol;

        #[cfg(target_os = "macos")]
        {
            let font_symbols = SystemSource::new()
                .select_by_postscript_name("Apple Symbols")
                .unwrap()
                .load()
                .unwrap();
            let copied_font_symbol = font_symbols.copy_font_data();
            let Some(copied_font_symbol) = copied_font_symbol else { todo!() };
            let font_vec_symbol =
                FontVec::try_from_vec_and_index(copied_font_symbol.to_vec(), 1).unwrap();
            font_arc_symbol = FontArc::new(font_vec_symbol);

            // TODO: Load native emojis
            // let font_emojis = SystemSource::new()
            //     .select_by_postscript_name("Apple Color Emoji")
            //     .unwrap()
            //     .load()
            //     .unwrap();
            // let copied_font_emojis = font_emojis.copy_font_data();
            // let Some(copied_font_emojis) = copied_font_emojis else { todo!() };
            // let font_vec_emojis = FontVec::try_from_vec_and_index(copied_font_emojis.to_vec(), 2).unwrap();

            let font_unicode = SystemSource::new()
                .select_by_postscript_name("Arial Unicode MS")
                .unwrap()
                .load()
                .unwrap();
            let copied_font_unicode = font_unicode.copy_font_data();
            let Some(copied_font_unicode) = copied_font_unicode else { todo!() };
            let font_vec_unicode =
                FontVec::try_from_vec_and_index(copied_font_unicode.to_vec(), 3).unwrap();
            font_arc_unicode = FontArc::new(font_vec_unicode);
        }

        #[cfg(not(target_os = "macos"))]
        {
            font_arc_unicode = FontArc::try_from_slice(FONT_DEJAVU_MONO).unwrap();
            font_arc_symbol = FontArc::try_from_slice(FONT_DEJAVU_MONO).unwrap();
        }

        let is_default_font = font_name.to_lowercase() == DEFAULT_FONT_NAME;
        if !is_default_font {
            if let Ok(system_fonts) =
                SystemSource::new().select_family_by_name(&font_name)
            {
                let fonts = system_fonts.fonts();
                if !fonts.is_empty() {
                    let mut text_fonts = ComposedFontArc {
                        regular: FontArc::try_from_slice(FONT_CASCADIAMONO_REGULAR)
                            .unwrap(),
                        bold: FontArc::try_from_slice(FONT_CASCADIAMONO_BOLD).unwrap(),
                        italic: FontArc::try_from_slice(FONT_CASCADIAMONO_ITALIC)
                            .unwrap(),
                        bold_italic: FontArc::try_from_slice(
                            FONT_CASCADIAMONO_BOLD_ITALIC,
                        )
                        .unwrap(),
                    };
                    for font in fonts.iter() {
                        let font = font.load();
                        if let Ok(font) = font {
                            let meta = font.properties();
                            match meta.style {
                                Style::Normal => {
                                    //TODO: Find a way to use struct Weight
                                    match meta.weight.0.round() as i32 {
                                        //NORMAL
                                        300 | 400 | 500 => {
                                            if let Some(font_arc) =
                                                font_arc_from_font(font)
                                            {
                                                text_fonts.regular = font_arc;
                                            }
                                        }
                                        //BOLD
                                        600 | 700 | 800 => {
                                            if let Some(font_arc) =
                                                font_arc_from_font(font)
                                            {
                                                text_fonts.bold = font_arc;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                Style::Italic => {
                                    match meta.weight.0.round() as i32 {
                                        //NORMAL
                                        400 => {
                                            if let Some(font_arc) =
                                                font_arc_from_font(font)
                                            {
                                                text_fonts.italic = font_arc;
                                            }
                                        }
                                        //BOLD
                                        700 => {
                                            if let Some(font_arc) =
                                                font_arc_from_font(font)
                                            {
                                                text_fonts.bold_italic = font_arc;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    return Font {
                        text: text_fonts,
                        symbol: font_arc_symbol,
                        emojis: FontArc::try_from_slice(FONT_EMOJI).unwrap(),
                        unicode: font_arc_unicode,
                    };
                }
            }

            warn!("failed to load font {font_name}");
        }

        Font {
            text: ComposedFontArc {
                regular: FontArc::try_from_slice(FONT_CASCADIAMONO_REGULAR).unwrap(),
                bold: FontArc::try_from_slice(FONT_CASCADIAMONO_BOLD).unwrap(),
                italic: FontArc::try_from_slice(FONT_CASCADIAMONO_ITALIC).unwrap(),
                bold_italic: FontArc::try_from_slice(FONT_CASCADIAMONO_BOLD_ITALIC)
                    .unwrap(),
            },
            symbol: font_arc_symbol,
            emojis: FontArc::try_from_slice(FONT_EMOJI).unwrap(),
            unicode: font_arc_unicode,
        }
    }
}
