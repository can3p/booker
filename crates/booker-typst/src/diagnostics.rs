//! Turning what Typst says into what Booker shows.
//!
//! The rule from `PLAN.md` §11.2: a diagnostic without a source location is
//! not useful. Typst gives us a span for nearly everything, so nearly every
//! diagnostic here names a file and a line. The ones that cannot (an error
//! about the document as a whole) still say which project file was being
//! compiled, in the message.

use booker_core::diagnostic::{Diagnostic, Severity, SourceLocation};
use typst::diag::{Severity as TypstSeverity, SourceDiagnostic};
use typst::syntax::DiagSpan;
use typst::{World, WorldExt};

use crate::world::BookerWorld;

/// Typst refused to lay the book out.
pub const RULE_TYPST_ERROR: &str = "BK-TYPST-001";
/// Typst laid the book out but was unhappy about something.
pub const RULE_TYPST_WARNING: &str = "BK-TYPST-002";

/// Converts a batch of Typst diagnostics, keeping their order.
pub fn convert(world: &BookerWorld, diagnostics: &[SourceDiagnostic]) -> Vec<Diagnostic> {
    diagnostics.iter().map(|d| one(world, d)).collect()
}

fn one(world: &BookerWorld, diagnostic: &SourceDiagnostic) -> Diagnostic {
    let (rule, severity) = match diagnostic.severity {
        TypstSeverity::Error => (RULE_TYPST_ERROR, Severity::Error),
        TypstSeverity::Warning => (RULE_TYPST_WARNING, Severity::Warning),
    };

    let mut message = diagnostic.message.to_string();
    for hint in &diagnostic.hints {
        // Typst's hints are the part users act on ("did you mean ...?"), so
        // they travel with the message rather than being dropped. One line,
        // because this text is compared between the CLI and the app.
        message.push_str("; hint: ");
        message.push_str(&hint.v);
    }

    let mut converted = Diagnostic::error(rule, message).with_severity(severity);

    // Prefer the diagnostic's own span; fall back to the innermost call in
    // the trace, which is where an error raised inside a function shows up.
    let mut span = diagnostic.span;
    if span.is_detached() {
        if let Some(point) = diagnostic.trace.first() {
            span = DiagSpan::from(point.span);
        }
    }

    if let Some(location) = locate(world, span) {
        converted = converted.at(location);
    }

    converted
}

/// Where in the project a span points, in the 1-based line and column an
/// editor would show.
fn locate(world: &BookerWorld, span: DiagSpan) -> Option<SourceLocation> {
    let id = span.id()?;
    let range = world.range(span)?;
    let source = world.source(id).ok()?;
    let lines = source.lines();
    let (line, column) = lines.byte_to_line_column(range.start)?;
    Some(SourceLocation {
        file: world.display_path(id),
        line: u32::try_from(line.saturating_add(1)).unwrap_or(u32::MAX),
        column: u32::try_from(column.saturating_add(1)).unwrap_or(u32::MAX),
        span: Some((range.start, range.end)),
    })
}
