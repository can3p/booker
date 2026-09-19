use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ts_rs::TS;

/// A physical unit. Booker speaks millimetres internally; the other units
/// exist because people write books in the units their printer quotes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub enum Unit {
    Mm,
    Cm,
    In,
    Pt,
    Px,
}

impl Unit {
    /// How many millimetres one of these is. `Px` follows CSS: 96 per inch.
    pub fn in_mm(self) -> f64 {
        match self {
            Unit::Mm => 1.0,
            Unit::Cm => 10.0,
            Unit::In => 25.4,
            Unit::Pt => 25.4 / 72.0,
            Unit::Px => 25.4 / 96.0,
        }
    }

    fn suffix(self) -> &'static str {
        match self {
            Unit::Mm => "mm",
            Unit::Cm => "cm",
            Unit::In => "in",
            Unit::Pt => "pt",
            Unit::Px => "px",
        }
    }
}

/// A measurement, kept in the unit the user wrote it in.
///
/// Values are written back in the author's own unit, so a file a person
/// edited by hand does not come back in millimetres they never typed.
#[derive(Debug, Clone, Copy, PartialEq, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/", type = "string")]
pub struct Length {
    pub value: f64,
    pub unit: Unit,
}

impl Length {
    pub const ZERO: Length = Length {
        value: 0.0,
        unit: Unit::Mm,
    };

    pub fn new(value: f64, unit: Unit) -> Self {
        Self { value, unit }
    }

    pub fn mm(value: f64) -> Self {
        Self::new(value, Unit::Mm)
    }

    pub fn to_mm(self) -> f64 {
        self.value * self.unit.in_mm()
    }

    pub fn to_pt(self) -> f64 {
        self.to_mm() / Unit::Pt.in_mm()
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Round-tripping matters more than precision here: a dragged frame
        // writes one decimal (see PLAN §6), and a hand-written `20mm` must
        // come back as `20mm`, not `20.0mm`.
        let rounded = (self.value * 10.0).round() / 10.0;
        if (rounded - rounded.trunc()).abs() < f64::EPSILON {
            write!(f, "{}{}", rounded.trunc() as i64, self.unit.suffix())
        } else {
            write!(f, "{}{}", rounded, self.unit.suffix())
        }
    }
}

impl FromStr for Length {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let split = s
            .find(|c: char| c.is_ascii_alphabetic() || c == '%')
            .ok_or_else(|| format!("`{s}` has no unit; write for example `18mm`"))?;
        let (number, unit) = s.split_at(split);
        let value: f64 = number
            .trim()
            .parse()
            .map_err(|_| format!("`{number}` is not a number"))?;
        let unit = match unit.trim().to_ascii_lowercase().as_str() {
            "mm" => Unit::Mm,
            "cm" => Unit::Cm,
            "in" => Unit::In,
            "pt" => Unit::Pt,
            "px" => Unit::Px,
            other => {
                return Err(format!(
                    "unknown unit `{other}`; Booker understands mm, cm, in, pt and px"
                ))
            }
        };
        Ok(Length::new(value, unit))
    }
}

impl Serialize for Length {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Length {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(serde::de::Error::custom)
    }
}

/// Page margins. Named for facing pages: a book's margins are inside and
/// outside, not left and right, because they swap on every other page.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct Margins {
    pub top: Length,
    pub bottom: Length,
    pub inside: Length,
    pub outside: Length,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            top: Length::mm(18.0),
            bottom: Length::mm(20.0),
            inside: Length::mm(20.0),
            outside: Length::mm(15.0),
        }
    }
}

/// A page size, either one of the presets people ask for by name or an
/// explicit pair of measurements.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub enum PageSize {
    Named(Preset),
    Custom { width: Length, height: Length },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub enum Preset {
    A4,
    A5,
    Letter,
    /// 6 × 9 inches, the usual paperback.
    Trade,
    /// 5 × 8 inches.
    Digest,
    /// 8.5 × 8.5 inches, the usual square picture book.
    Square,
}

impl PageSize {
    pub fn dimensions(self) -> (Length, Length) {
        match self {
            PageSize::Custom { width, height } => (width, height),
            PageSize::Named(preset) => match preset {
                Preset::A4 => (Length::mm(210.0), Length::mm(297.0)),
                Preset::A5 => (Length::mm(148.0), Length::mm(210.0)),
                Preset::Letter => (Length::new(8.5, Unit::In), Length::new(11.0, Unit::In)),
                Preset::Trade => (Length::new(6.0, Unit::In), Length::new(9.0, Unit::In)),
                Preset::Digest => (Length::new(5.0, Unit::In), Length::new(8.0, Unit::In)),
                Preset::Square => (Length::new(8.5, Unit::In), Length::new(8.5, Unit::In)),
            },
        }
    }
}

impl Default for PageSize {
    fn default() -> Self {
        PageSize::Named(Preset::A5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_units_people_write() {
        assert_eq!("18mm".parse::<Length>().unwrap(), Length::mm(18.0));
        assert_eq!(
            "8.5in".parse::<Length>().unwrap(),
            Length::new(8.5, Unit::In)
        );
        assert_eq!(
            " 12 pt ".parse::<Length>().unwrap(),
            Length::new(12.0, Unit::Pt)
        );
    }

    #[test]
    fn explains_itself_when_the_input_is_wrong() {
        let err = "18".parse::<Length>().unwrap_err();
        assert!(err.contains("18mm"), "{err}");
        let err = "18furlongs".parse::<Length>().unwrap_err();
        assert!(err.contains("mm, cm, in, pt and px"), "{err}");
    }

    #[test]
    fn round_trips_in_the_unit_it_was_written_in() {
        for raw in ["20mm", "8.5in", "12pt", "1.5cm"] {
            let parsed: Length = raw.parse().unwrap();
            assert_eq!(parsed.to_string(), raw);
        }
    }

    #[test]
    fn converts_between_units() {
        assert!((Length::new(1.0, Unit::In).to_mm() - 25.4).abs() < 1e-9);
        assert!((Length::new(72.0, Unit::Pt).to_mm() - 25.4).abs() < 1e-9);
    }

    #[test]
    fn presets_have_the_sizes_printers_quote() {
        let (w, h) = PageSize::Named(Preset::Trade).dimensions();
        assert!((w.to_mm() - 152.4).abs() < 1e-9);
        assert!((h.to_mm() - 228.6).abs() < 1e-9);
    }
}
