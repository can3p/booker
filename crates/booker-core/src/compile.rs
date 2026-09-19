use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::diagnostic::Diagnostic;
use crate::geometry::Length;
use crate::project::{ProjectRef, Revision};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub enum CompileTarget {
    /// Lay the book out; produce page geometry and diagnostics, no file.
    Layout,
    /// Lay it out and write a PDF.
    Pdf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct CompileRequest {
    pub project: ProjectRef,
    pub target: CompileTarget,
    /// The revision this request was made against, echoed back in the result
    /// so a caller can discard an answer to a question it no longer has.
    pub revision: Revision,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct CompileResult {
    pub revision: Revision,
    pub pages: Vec<PageInfo>,
    /// Always present, possibly empty. A compile that produced pages *and*
    /// problems is normal: a broken book still opens (`PLAN.md` §11.1).
    pub diagnostics: Vec<Diagnostic>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct PageInfo {
    /// 0-based index into the document.
    pub index: u32,
    /// What is printed on the page: usually the number, sometimes roman
    /// numerals in front matter, sometimes nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub width: Length,
    pub height: Length,
    /// Which chapter this page belongs to, for the sidebar and diagnostics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chapter: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub enum RenderFormat {
    Png,
    Svg,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct RenderRequest {
    pub project: ProjectRef,
    /// The revision this page was laid out at. A render returns bytes, so
    /// nothing echoes it back; staleness is caught by the revision in the
    /// `booker://` URL instead (see `ipc`). Kept because the renderer must
    /// refuse a page from a revision it no longer holds.
    pub revision: Revision,
    /// 0-based, matching `PageInfo::index`.
    pub page: u32,
    /// Pixels per CSS pixel: 1.0 is 96 dpi, 2.0 a retina display.
    ///
    /// Typst measures in points, not CSS pixels, so a renderer must convert
    /// by 96/72. Getting this wrong makes every preview 33% off and nothing
    /// complains, which is why the unit is spelled out here.
    pub scale: f32,
    pub format: RenderFormat,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{PageSize, Preset};

    #[test]
    fn a_compile_result_always_reports_its_revision() {
        let (width, height) = PageSize::Named(Preset::A5).dimensions();
        let result = CompileResult {
            revision: Revision(7),
            pages: vec![PageInfo {
                index: 0,
                label: Some("i".into()),
                width,
                height,
                chapter: None,
            }],
            diagnostics: Vec::new(),
            duration_ms: 12,
        };
        let value = serde_json::to_value(&result).unwrap();
        assert_eq!(value["revision"], 7);
        assert_eq!(value["pages"][0]["width"], "148mm");
    }
}
