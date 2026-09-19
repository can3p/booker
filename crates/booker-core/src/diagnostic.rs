use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Something wrong with a book, said precisely enough to act on.
///
/// The rule that matters (`PLAN.md` §11.2): a diagnostic without a source
/// location is not useful. "Text overflows on page 12" cannot be acted on;
/// "content/03-flowers.md:145 overflows the caption frame on page 12" can,
/// by a person or by an agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct Diagnostic {
    /// Stable identifier, e.g. `BK-LAYOUT-003`. Stable means: usable in
    /// `[check]` configuration and in an ignore comment, for years.
    pub rule: String,
    pub severity: Severity,
    /// Written for a person. An agent reads it too, so no jargon we invented.
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutLocation>,
    /// Present when the repair is mechanical; this is what `fix --safe` applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix: Option<Fix>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Where in the files the author wrote it. Paths are relative to the project
/// root so that output is identical on every machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct SourceLocation {
    /// Where the file is in the project, relative to its root and spelled
    /// with forward slashes on every platform — build it with
    /// [`crate::display_path`] when it comes from the filesystem rather than
    /// from a project file. This string reaches the problems panel, the CLI
    /// and, from Wave 7, an agent comparing our output with the CLI's, so
    /// one file has to print as one string wherever Booker runs.
    #[ts(type = "string")]
    pub file: PathBuf,
    /// 1-based, as editors count.
    pub line: u32,
    pub column: u32,
    /// Byte range in the file, when the producer knows it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<(usize, usize)>,
}

/// Where in the laid-out book it showed up.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct LayoutLocation {
    /// 1-based page number as printed, not an index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chapter: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct Fix {
    /// What it would do, in a sentence, for a confirmation prompt.
    pub description: String,
    /// A unified diff against the project. Absent means the fix needs the
    /// layout engine in the loop rather than a text edit (`PLAN.md` §11.4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch: Option<String>,
}

impl Diagnostic {
    pub fn error(rule: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            rule: rule.into(),
            severity: Severity::Error,
            message: message.into(),
            source: None,
            layout: None,
            fix: None,
        }
    }

    pub fn at(mut self, source: SourceLocation) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialises_without_the_fields_it_does_not_have() {
        let json = serde_json::to_string(&Diagnostic::error(
            "BK-REF-001",
            "image `assets/images/cat.png` does not exist",
        ))
        .unwrap();
        assert_eq!(
            json,
            r#"{"rule":"BK-REF-001","severity":"error","message":"image `assets/images/cat.png` does not exist"}"#,
            "output is compared between runs and between the CLI and MCP, so it stays terse and stable"
        );
    }

    #[test]
    fn carries_a_source_location_when_it_has_one() {
        let diagnostic = Diagnostic::error("BK-LAYOUT-003", "text has nowhere left to continue")
            .at(SourceLocation {
                file: PathBuf::from("content/03-flowers.md"),
                line: 145,
                column: 1,
                span: None,
            });
        let value = serde_json::to_value(&diagnostic).unwrap();
        assert_eq!(value["source"]["file"], "content/03-flowers.md");
        assert_eq!(value["source"]["line"], 145);
    }

    #[test]
    fn severities_sort_most_serious_first() {
        let mut severities = vec![Severity::Info, Severity::Error, Severity::Warning];
        severities.sort();
        assert_eq!(
            severities,
            vec![Severity::Error, Severity::Warning, Severity::Info]
        );
    }
}
