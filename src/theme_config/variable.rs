//! Declarative types for theme variables. Rounding snaps to the declared precision/step;
//! without rounding an off-grid value is invalid. Bounds are checked after rounding.
use super::{Kind, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    Ceil,
    Floor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Integer {
        min: Option<i64>,
        max: Option<i64>,
        step: u64,
        rounds: Option<Rounding>,
    },
    Decimal {
        no_decimals: u8,
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
        rounds: Option<Rounding>,
    },
    Color,
    Length {
        min: Option<f64>,
        max: Option<f64>,
    },
    Percentage {
        min: Option<f64>,
        max: Option<f64>,
        rounds: Option<Rounding>,
    },
    String {
        values: Vec<String>,
        accepts_random: bool,
    },
    /// Existing unrestricted toolkit numeric tokens.
    Number,
    Bool,
}
impl VariableType {
    /// The unconstrained type for a kind that has no parameters (color, number, bool, text).
    pub(crate) fn untyped(kind: Kind) -> Option<Self> {
        match kind {
            Kind::Color => Some(Self::Color),
            Kind::Number => Some(Self::Number),
            Kind::Bool => Some(Self::Bool),
            Kind::Text => Some(Self::String {
                values: vec![],
                accepts_random: true,
            }),
            _ => None,
        }
    }
    pub fn kind(&self) -> Kind {
        match self {
            Self::Integer { .. } => Kind::Integer,
            Self::Decimal { .. } => Kind::Decimal,
            Self::Length { .. } => Kind::Length,
            Self::Percentage { .. } => Kind::Percentage,
            Self::Color => Kind::Color,
            Self::String { .. } => Kind::Text,
            Self::Number => Kind::Number,
            Self::Bool => Kind::Bool,
        }
    }
    pub fn parse(&self, raw: &str) -> Option<Value> {
        let raw = raw.trim();
        match self {
            Self::Integer {
                min,
                max,
                step,
                rounds,
            } => {
                if *step == 0 || min.zip(*max).is_some_and(|(a, b)| a > b) {
                    return None;
                }
                let n = if let Ok(n) = raw.parse::<i64>() {
                    n
                } else {
                    let n = finite(raw)?;
                    let n = match rounds {
                        Some(Rounding::Ceil) => n.ceil(),
                        Some(Rounding::Floor) => n.floor(),
                        None => return None,
                    };
                    if n < i64::MIN as f64 || n >= -(i64::MIN as f64) {
                        return None;
                    }
                    n as i64
                };
                let origin = min.unwrap_or(0) as i128;
                let delta = n as i128 - origin;
                let step = *step as i128;
                let remainder = delta.rem_euclid(step);
                let n = if remainder == 0 {
                    n as i128
                } else {
                    match rounds {
                        Some(Rounding::Floor) => n as i128 - remainder,
                        Some(Rounding::Ceil) => n as i128 + step - remainder,
                        None => return None,
                    }
                };
                let n = i64::try_from(n).ok()?;
                if min.is_some_and(|v| n < v) || max.is_some_and(|v| n > v) {
                    return None;
                }
                Some(Value::Integer(n))
            }
            Self::Decimal {
                no_decimals,
                min,
                max,
                step,
                rounds,
            } => {
                if *no_decimals > 12 {
                    return None;
                }
                let quantum = 10f64.powi(-(*no_decimals as i32));
                let mut n = snap(finite(raw)?, quantum, 0.0, *rounds)?;
                if let Some(step) = step {
                    n = snap(n, *step, min.unwrap_or(0.0), *rounds)?;
                }
                n = snap(n, quantum, 0.0, None)?;
                bounds(n, *min, *max).map(Value::Decimal)
            }
            Self::Length { min, max } => {
                bounds(finite(raw.strip_suffix("px")?.trim())?, *min, *max).map(Value::Length)
            }
            Self::Percentage { min, max, rounds } => {
                let mut n = finite(raw.strip_suffix('%').unwrap_or(raw).trim())?;
                if rounds.is_some() {
                    n = snap(n, 1.0, 0.0, *rounds)?;
                }
                bounds(n, Some(min.unwrap_or(0.0)), Some(max.unwrap_or(100.0)))
                    .map(Value::Percentage)
            }
            Self::Color => parse_color(raw).map(Value::Color),
            Self::String {
                values,
                accepts_random,
            } => (!raw.contains(['\n', '\r'])
                && (*accepts_random || values.iter().any(|v| v == raw)))
            .then(|| Value::Text(raw.into())),
            Self::Number => raw
                .parse::<f32>()
                .ok()
                .filter(|n| n.is_finite())
                .map(Value::Number),
            Self::Bool => match raw.to_ascii_lowercase().as_str() {
                "true" | "yes" | "on" | "1" => Some(Value::Bool(true)),
                "false" | "no" | "off" | "0" => Some(Value::Bool(false)),
                _ => None,
            },
        }
    }
    pub(crate) fn well_formed(&self) -> bool {
        let range = |min: Option<f64>, max: Option<f64>| {
            min.is_none_or(f64::is_finite)
                && max.is_none_or(f64::is_finite)
                && min.zip(max).is_none_or(|(a, b)| a <= b)
        };
        match self {
            Self::Integer { min, max, step, .. } => {
                *step > 0 && min.zip(*max).is_none_or(|(a, b)| a <= b)
            }
            Self::Decimal {
                no_decimals,
                min,
                max,
                step,
                ..
            } => {
                *no_decimals <= 12
                    && range(*min, *max)
                    && step.is_none_or(|n| n.is_finite() && n > 0.0)
            }
            Self::Length { min, max } => range(*min, *max),
            Self::Percentage { min, max, .. } => {
                range(Some(min.unwrap_or(0.0)), Some(max.unwrap_or(100.0)))
            }
            _ => true,
        }
    }
}
fn finite(raw: &str) -> Option<f64> {
    raw.parse::<f64>().ok().filter(|n| n.is_finite())
}
fn bounds(n: f64, min: Option<f64>, max: Option<f64>) -> Option<f64> {
    (n.is_finite() && min.is_none_or(|m| n >= m) && max.is_none_or(|m| n <= m)).then_some(n)
}
fn snap(n: f64, step: f64, origin: f64, rounds: Option<Rounding>) -> Option<f64> {
    if !step.is_finite() || step <= 0.0 {
        return None;
    }
    let units = (n - origin) / step;
    if !units.is_finite() {
        return None;
    }
    let near = units.round();
    let tolerance = (f64::EPSILON * units.abs().max(1.0) * 16.0).min(1e-6);
    let snapped = if (units - near).abs() <= tolerance {
        near
    } else {
        match rounds {
            Some(Rounding::Ceil) => units.ceil(),
            Some(Rounding::Floor) => units.floor(),
            None => return None,
        }
    };
    Some(origin + snapped * step)
}
fn parse_color(raw: &str) -> Option<iced::Color> {
    if !raw.contains('(') {
        let raw = raw.strip_prefix('#').unwrap_or(raw);
        if ![3, 4, 6, 8].contains(&raw.len()) || !raw.bytes().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let c = crate::primitives::color_picker::Hsva::parse(raw)?;
        let (r, g, b, a) = c.to_rgba();
        return Some(iced::Color::from_rgba(r, g, b, a));
    }
    let raw = raw.to_ascii_lowercase().replace("hasva(", "hsva(");
    let (name, rest) = raw.split_once('(')?;
    let name = name.trim();
    let parts: Vec<_> = rest
        .strip_suffix(')')?
        .split(|c: char| c == ',' || c == '/' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .collect();
    let count = match name {
        "rgb" | "hsv" | "hsl" => 3,
        "rgba" | "hsva" | "hsla" => 4,
        _ => return None,
    };
    if parts.len() != count {
        return None;
    }
    for (i, part) in parts.iter().enumerate() {
        let percent = part.ends_with('%');
        let n = finite(part.trim_end_matches('%').trim_end_matches("deg"))?;
        if !(n as f32).is_finite() {
            return None;
        }
        let valid = if i == 3 {
            (0.0..=if percent { 100.0 } else { 1.0 }).contains(&n)
        } else if name.starts_with("rgb") {
            (0.0..=if percent { 100.0 } else { 255.0 }).contains(&n)
        } else if i == 0 {
            !percent
        } else {
            percent && (0.0..=100.0).contains(&n)
        };
        if !valid {
            return None;
        }
    }
    let canonical = format!("{name}({})", parts.join(", "));
    let c = crate::primitives::color_picker::Hsva::parse(&canonical)?;
    let (r, g, b, a) = c.to_rgba();
    Some(iced::Color::from_rgba(r, g, b, a))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integers_preserve_i64_and_enforce_steps_rounding_and_bounds() {
        let integer = |rounds| VariableType::Integer {
            min: Some(-10),
            max: Some(10),
            step: 2,
            rounds,
        };
        assert_eq!(integer(None).parse("4"), Some(Value::Integer(4)));
        for invalid in ["3", "1.5", "12", "NaN", "inf"] {
            assert!(integer(None).parse(invalid).is_none());
        }
        assert_eq!(
            integer(Some(Rounding::Ceil)).parse("-1.2"),
            Some(Value::Integer(0))
        );
        assert_eq!(
            integer(Some(Rounding::Floor)).parse("-1.2"),
            Some(Value::Integer(-2))
        );
        let wide = VariableType::Integer {
            min: None,
            max: None,
            step: 1,
            rounds: None,
        };
        assert_eq!(
            wide.parse(&i64::MAX.to_string()),
            Some(Value::Integer(i64::MAX))
        );
        assert!(wide.parse("9223372036854775808").is_none());
    }
    #[test]
    fn decimals_validate_precision_and_step_without_silently_clamping() {
        let ty = |rounds| VariableType::Decimal {
            no_decimals: 2,
            min: Some(0.0),
            max: Some(1.0),
            step: Some(0.25),
            rounds,
        };
        assert_eq!(ty(None).parse("0.5"), Some(Value::Decimal(0.5)));
        for invalid in ["0.501", "0.60", "2", "NaN", "-inf"] {
            assert!(ty(None).parse(invalid).is_none());
        }
        assert_eq!(
            ty(Some(Rounding::Ceil)).parse("0.501"),
            Some(Value::Decimal(0.75))
        );
        assert_eq!(
            ty(Some(Rounding::Floor)).parse("0.501"),
            Some(Value::Decimal(0.5))
        );
        assert!(ty(Some(Rounding::Ceil)).parse("1.001").is_none());
    }
    #[test]
    fn length_percentage_and_string_follow_the_declared_contract() {
        let length = VariableType::Length {
            min: Some(0.0),
            max: Some(100.0),
        };
        assert_eq!(length.parse("12.5px"), Some(Value::Length(12.5)));
        for invalid in ["12", "12em", "101px", "NaNpx", "-1px"] {
            assert!(length.parse(invalid).is_none());
        }
        let percentage = VariableType::Percentage {
            min: None,
            max: None,
            rounds: Some(Rounding::Floor),
        };
        assert_eq!(percentage.parse("42.7%"), Some(Value::Percentage(42.0)));
        assert!(percentage.parse("101%").is_none());
        let listed = VariableType::String {
            values: vec!["solid".into(), "outline".into()],
            accepts_random: false,
        };
        assert!(listed.parse("solid").is_some());
        assert!(listed.parse("other").is_none());
        let free = VariableType::String {
            values: vec![],
            accepts_random: true,
        };
        assert!(free.parse("anything").is_some());
        assert!(free.parse("a\nb").is_none());
    }
    #[test]
    fn all_color_formats_are_supported_and_invalid_channels_rejected() {
        for raw in [
            "#f00",
            "#f008",
            "#ff0000",
            "#ff000080",
            "rgb(255,0,0)",
            "rgba(255,0,0,0.5)",
            "hsv(0,100%,100%)",
            "hsva(0,100%,100%,0.5)",
            "hasva(0,100%,100%,0.5)",
            "hsl(0,100%,50%)",
            "hsla(0,100%,50%,0.5)",
            "rgb(100% 0% 0%)",
        ] {
            let Value::Color(c) = VariableType::Color
                .parse(raw)
                .unwrap_or_else(|| panic!("{raw}"))
            else {
                panic!()
            };
            assert!(
                (c.r - 1.0).abs() < 0.001 && c.g < 0.001 && c.b < 0.001,
                "{raw}: {c:?}"
            );
        }
        for raw in [
            "rgb(256,0,0)",
            "rgba(0,0,0,2)",
            "rgb(1,2)",
            "rgb(1,2,3,4,5)",
            "hsl(0,101%,50%)",
            "hsv(NaN,0%,0%)",
            "#zzz",
        ] {
            assert!(VariableType::Color.parse(raw).is_none(), "{raw}");
        }
    }
}
