//! The built-in themes: one bundle of typographic defaults per template.
//!
//! A theme is the answer to "what should this book look like if the author
//! says nothing at all" (`AGENTS.md`: defaults must produce a good-looking
//! book with no configuration). `book.toml` picks one by name with `theme`;
//! Wave 3's `styles.toml` overrides any value in it, which is why these are
//! plain values rather than Typst code — they become the bottom layer of the
//! style cascade (`PLAN.md` §5.4).
//!
//! Every font named here is bundled (`fonts/NOTICE.txt`), so a theme renders
//! the same on every machine. Libertinus Serif follows each one as a
//! fallback for characters a face lacks.

use booker_core::{ChapterStart, DEFAULT_THEME};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Paragraphs {
    /// Novels: no space between paragraphs, the first line indented, except
    /// after a heading or a break — the way printed prose has always looked.
    Indented { indent_em: f64 },
    /// Everything else: a gap between paragraphs and no indent. The gap is
    /// *added* to the space between lines — Typst's paragraph spacing
    /// replaces the line gap rather than adding to it, so a gap smaller
    /// than the leading would be invisible.
    Spaced { gap_em: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageNumbers {
    None,
    /// Centred in the footer.
    Centre,
    /// In the footer, on the outer edge — right on a right-hand page.
    Outside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleAlign {
    Left,
    Centre,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    pub body_font: &'static str,
    pub body_size_pt: f64,
    /// Space between lines, as Typst's `leading`: the gap, not the pitch.
    pub leading_em: f64,
    pub paragraphs: Paragraphs,
    /// Justified text, which also switches hyphenation on (Typst hyphenates
    /// justified text by default, in the book's language).
    pub justify: bool,
    pub heading_font: &'static str,
    /// Sizes of heading levels 1–3, in points. Deeper levels use level 3.
    pub heading_size_pt: [f64; 3],
    pub heading_bold: bool,
    /// How a chapter title (a level-1 heading) sits.
    pub title_align: TitleAlign,
    /// How far down the page a chapter title starts, in ems of body text.
    /// Printed books drop chapter openings about a quarter of the way.
    pub title_sink_em: f64,
    /// Space between a chapter title and the text under it.
    pub title_below_em: f64,
    pub chapter_start: ChapterStart,
    /// Whether a book with more than one chapter gets a table of contents
    /// when `[toc] enabled` says nothing.
    pub contents: bool,
    pub page_numbers: PageNumbers,
    /// Width of an image that does not say, as a percentage of the text.
    pub image_width_percent: u32,
}

pub const NOVEL: Theme = Theme {
    name: "novel",
    body_font: "EB Garamond",
    body_size_pt: 11.5,
    leading_em: 0.62,
    paragraphs: Paragraphs::Indented { indent_em: 1.3 },
    justify: true,
    heading_font: "EB Garamond",
    heading_size_pt: [22.0, 14.0, 12.0],
    heading_bold: false,
    title_align: TitleAlign::Centre,
    title_sink_em: 7.0,
    title_below_em: 2.2,
    chapter_start: ChapterStart::RightPage,
    contents: true,
    page_numbers: PageNumbers::Outside,
    image_width_percent: 80,
};

pub const PICTURE_BOOK: Theme = Theme {
    name: "picture-book",
    body_font: "Andika",
    body_size_pt: 17.0,
    leading_em: 0.75,
    paragraphs: Paragraphs::Spaced { gap_em: 0.8 },
    // Ragged text for young readers: even word spacing, no hyphens.
    justify: false,
    heading_font: "Andika",
    heading_size_pt: [30.0, 22.0, 18.0],
    heading_bold: true,
    title_align: TitleAlign::Centre,
    title_sink_em: 1.0,
    title_below_em: 1.2,
    chapter_start: ChapterStart::NewPage,
    contents: false,
    page_numbers: PageNumbers::None,
    image_width_percent: 100,
};

pub const POETRY: Theme = Theme {
    name: "poetry",
    body_font: "EB Garamond",
    body_size_pt: 12.0,
    leading_em: 0.6,
    paragraphs: Paragraphs::Spaced { gap_em: 0.9 },
    // A line of verse ends where the poet ended it.
    justify: false,
    heading_font: "EB Garamond",
    heading_size_pt: [18.0, 13.0, 12.0],
    heading_bold: false,
    title_align: TitleAlign::Left,
    title_sink_em: 4.0,
    title_below_em: 1.8,
    chapter_start: ChapterStart::NewPage,
    contents: true,
    page_numbers: PageNumbers::Centre,
    image_width_percent: 70,
};

pub const PAPER: Theme = Theme {
    name: "paper",
    body_font: "Source Serif 4",
    body_size_pt: 10.5,
    leading_em: 0.62,
    paragraphs: Paragraphs::Spaced { gap_em: 0.55 },
    justify: true,
    heading_font: "Inter",
    heading_size_pt: [16.0, 12.5, 10.5],
    heading_bold: true,
    title_align: TitleAlign::Left,
    title_sink_em: 0.0,
    title_below_em: 0.9,
    chapter_start: ChapterStart::Continue,
    contents: false,
    page_numbers: PageNumbers::Centre,
    image_width_percent: 80,
};

/// Every built-in theme, in the order `booker_core::THEMES` names them.
pub const ALL: [Theme; 4] = [NOVEL, PICTURE_BOOK, POETRY, PAPER];

/// The theme called `name`, or the default one. An unknown name is reported
/// by the project loader (`BK-FORMAT-006`); here it must still produce a
/// book.
pub fn by_name(name: &str) -> Theme {
    ALL.into_iter()
        .find(|theme| theme.name == name)
        .or_else(|| ALL.into_iter().find(|theme| theme.name == DEFAULT_THEME))
        .unwrap_or(NOVEL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_themes_are_the_ones_the_contract_names() {
        let names: Vec<&str> = ALL.iter().map(|theme| theme.name).collect();
        assert_eq!(names, booker_core::THEMES);
    }

    #[test]
    fn an_unknown_name_gets_the_default_theme() {
        assert_eq!(by_name("novle").name, DEFAULT_THEME);
    }
}
