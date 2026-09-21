//! Booker's attribute block: `{#id .class key=value key="a value"}`.
//!
//! Pandoc's syntax (`PLAN.md` §5.3), so a book written for Booker still reads
//! sensibly in any other Markdown tool. Parsing is strict on purpose: a
//! `{…}` that does not parse as attributes is left as text, because in a
//! novel a brace is far more likely to be prose than a typo in syntax.

use crate::model::Attributes;
use crate::span::Span;

/// The attribute keys something in this build reads. Anything else is kept,
/// and `booker check` warns about it with the nearest of these
/// (`BK-DOC-001`). Classes are not listed: a class with no meaning yet is
/// not a mistake — Wave 3 gives classes styles.
pub const ATTRIBUTE_KEYS: &[&str] = &[
    // On a heading or a `:::` div: start a new page here.
    "break-before",
    // On an image: how wide it is drawn, as a measurement or a percentage
    // of the text width.
    "width",
];

/// Parse `text`, which must be exactly one attribute block from `{` to `}`.
///
/// `span` is where `text` sits in the source, and becomes the attributes'
/// own span, so a diagnostic can point at the braces rather than at the
/// paragraph around them.
pub fn parse_block(text: &str, span: Span) -> Option<Attributes> {
    let inner = text.strip_prefix('{')?.strip_suffix('}')?;
    let mut attributes = parse_inner(inner)?;
    attributes.span = Some(span);
    Some(attributes)
}

/// The words after a `:::` fence: either an attribute block, or a single
/// bare word that is a class (`::: poem` means `::: {.poem}`, as in Pandoc).
pub fn parse_fence_info(text: &str, span: Span) -> Option<Attributes> {
    let text = text.trim();
    if text.starts_with('{') {
        return parse_block(text, span);
    }
    if !text.is_empty() && text.split_whitespace().count() == 1 && is_name(text) {
        return Some(Attributes {
            classes: vec![text.to_string()],
            span: Some(span),
            ..Attributes::default()
        });
    }
    None
}

fn parse_inner(inner: &str) -> Option<Attributes> {
    let mut attributes = Attributes::default();
    let mut rest = inner.trim();
    if rest.is_empty() {
        return None;
    }
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix('#') {
            let (name, next) = take_name(after)?;
            if attributes.id.is_some() {
                return None;
            }
            attributes.id = Some(name.to_string());
            rest = next;
        } else if let Some(after) = rest.strip_prefix('.') {
            let (name, next) = take_name(after)?;
            attributes.classes.push(name.to_string());
            rest = next;
        } else {
            let (key, after_key) = take_name(rest)?;
            let after_eq = after_key.strip_prefix('=')?;
            let (value, next) = take_value(after_eq)?;
            attributes.pairs.push((key.to_string(), value));
            rest = next;
        }
        rest = rest.trim_start();
    }
    Some(attributes)
}

fn is_name_char(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '-' | '_' | ':')
}

fn is_name(text: &str) -> bool {
    !text.is_empty() && text.chars().all(is_name_char)
}

/// A name, ending at whitespace, `=` or the end.
fn take_name(text: &str) -> Option<(&str, &str)> {
    let end = text
        .char_indices()
        .find(|(_, c)| !is_name_char(*c))
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    if end == 0 {
        return None;
    }
    let rest = &text[end..];
    // A name must be followed by a separator; `#id.class` without a space is
    // not something we guess at.
    match rest.chars().next() {
        None | Some('=') => Some((&text[..end], rest)),
        Some(c) if c.is_whitespace() => Some((&text[..end], rest)),
        _ => None,
    }
}

/// A value: `"quoted, with spaces"` or a bare word.
fn take_value(text: &str) -> Option<(String, &str)> {
    if let Some(quoted) = text.strip_prefix('"') {
        let end = quoted.find('"')?;
        let rest = &quoted[end + 1..];
        if rest.chars().next().is_some_and(|c| !c.is_whitespace()) {
            return None;
        }
        return Some((quoted[..end].to_string(), rest));
    }
    let end = text
        .char_indices()
        .find(|(_, c)| c.is_whitespace())
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    if end == 0 {
        return None;
    }
    Some((text[..end].to_string(), &text[end..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Option<Attributes> {
        parse_block(text, Span::new(0, text.len()))
    }

    #[test]
    fn reads_ids_classes_and_pairs() {
        let attributes = parse(r#"{#tulips .wide .photo width=80% caption="In May"}"#).unwrap();
        assert_eq!(attributes.id.as_deref(), Some("tulips"));
        assert_eq!(attributes.classes, ["wide", "photo"]);
        assert_eq!(attributes.get("width"), Some("80%"));
        assert_eq!(attributes.get("caption"), Some("In May"));
    }

    #[test]
    fn prose_in_braces_is_not_attributes() {
        for text in [
            "{she said so}",
            "{}",
            "{#}",
            "{#a #b}",
            "{.a.b}",
            "{key=}",
            "{x=\"open}",
        ] {
            assert!(parse(text).is_none(), "{text} should stay text");
        }
    }

    #[test]
    fn a_bare_word_after_a_fence_is_a_class() {
        let attributes = parse_fence_info("page-break", Span::new(0, 10)).unwrap();
        assert_eq!(attributes.classes, ["page-break"]);
        assert!(parse_fence_info("two words", Span::new(0, 9)).is_none());
    }
}
