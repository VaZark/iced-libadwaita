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
    /// A `linear-gradient(<angle>deg, <color> <pos>%, ...)`, see [`Gradient`](super::Gradient).
    Gradient,
    /// A font family name: 1 to 64 characters, no control characters.
    Font,
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
            Kind::Gradient => Some(Self::Gradient),
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
            Self::Gradient => Kind::Gradient,
            Self::Font => Kind::Text,
            Self::String { .. } => Kind::Text,
            Self::Number => Kind::Number,
            Self::Bool => Kind::Bool,
        }
    }
    /// Parses text typed by a person (a field in the theme manager): lengths end in `px`,
    /// percentages may end in `%`, gradients are `linear-gradient(...)`. Files do not go through
    /// text: see [`from_ron`](Self::from_ron).
    pub fn parse(&self, raw: &str) -> Option<Value> {
        let raw = raw.trim();
        let input = match self {
            Self::Integer { .. } => match raw.parse::<i64>() {
                Ok(n) => Input::Int(n),
                Err(_) => Input::Num(finite(raw)?),
            },
            Self::Decimal { .. } => Input::Num(finite(raw)?),
            Self::Length { .. } => Input::Num(finite(raw.strip_suffix("px")?.trim())?),
            Self::Percentage { .. } => {
                Input::Num(finite(raw.strip_suffix('%').unwrap_or(raw).trim())?)
            }
            Self::Number => Input::Num(raw.parse::<f32>().ok().filter(|n| n.is_finite())? as f64),
            Self::Bool => match raw.to_ascii_lowercase().as_str() {
                "true" | "yes" | "on" | "1" => Input::Bool(true),
                "false" | "no" | "off" | "0" => Input::Bool(false),
                _ => return None,
            },
            Self::Gradient => return super::Gradient::parse(raw).map(Value::Gradient),
            Self::Color | Self::Font | Self::String { .. } => Input::Text(raw),
        };
        self.check(input)
    }

    /// Reads a value from a file's RON: numbers are numbers (a length is a plain number of px),
    /// booleans are booleans, colors, fonts and choices are strings, and a gradient is the
    /// structure `(angle: 90, stops: [(0.0, "#0a246a"), (1.0, "#a6caf0")])`. `Err` says what
    /// was expected.
    pub fn from_ron(&self, v: &ron::Value) -> Result<Value, String> {
        use ron::Value as R;
        let input = match (self, v) {
            (Self::Gradient, v) => return super::Gradient::from_ron(v).map(Value::Gradient),
            (_, R::Number(n)) => match (self, n) {
                (Self::Integer { .. }, ron::Number::I8(_) | ron::Number::I16(_) | ron::Number::I32(_)
                    | ron::Number::I64(_) | ron::Number::U8(_) | ron::Number::U16(_)
                    | ron::Number::U32(_) | ron::Number::U64(_)) => {
                    Input::Int(n.into_f64() as i64)
                }
                _ => Input::Num(n.into_f64()),
            },
            (Self::Bool, R::Bool(b)) => Input::Bool(*b),
            (Self::Color | Self::Font | Self::String { .. }, R::String(s)) => Input::Text(s.trim()),
            _ => return Err(format!("expected {}", self.expectation())),
        };
        self.check(input)
            .ok_or_else(|| format!("{v:?} is not valid: expected {}", self.expectation()))
    }

    /// Checks a value built in code (a hardcoded token) the way a file's value is checked: the
    /// right kind, inside the range, on the step.
    pub fn from_value(&self, v: &Value) -> Option<Value> {
        if self.kind() != v.kind() {
            return None;
        }
        match v {
            Value::Integer(n) => self.check(Input::Int(*n)),
            Value::Decimal(n) | Value::Length(n) | Value::Percentage(n) => {
                self.check(Input::Num(*n))
            }
            Value::Number(n) => self.check(Input::Num(*n as f64)),
            Value::Bool(b) => self.check(Input::Bool(*b)),
            Value::Text(t) => self.check(Input::Text(t.trim())),
            Value::Color(_) => Some(v.clone()),
            Value::Gradient(g) => (matches!(self, Self::Gradient)
                && (2..=super::gradient::MAX_STOPS).contains(&g.stops.len()))
            .then(|| v.clone()),
        }
    }

    /// What a value of this type looks like, for error messages.
    pub fn expectation(&self) -> String {
        match self {
            Self::Integer { min, max, step, .. } => format!(
                "an integer{}{}",
                range_text(min.map(|n| n as f64), max.map(|n| n as f64)),
                if *step > 1 { format!(" in steps of {step}") } else { String::new() }
            ),
            Self::Decimal { min, max, .. } => format!("a number{}", range_text(*min, *max)),
            Self::Length { min, max } => format!("a length in px (a number){}", range_text(*min, *max)),
            Self::Percentage { min, max, .. } => format!(
                "a percentage (a number){}",
                range_text(Some(min.unwrap_or(0.0)), Some(max.unwrap_or(100.0)))
            ),
            Self::Color => "a color string (\"#rrggbb\", \"rgba(..)\", \"hsl(..)\")".into(),
            Self::Gradient => "a gradient: (angle: 90, stops: [(0.0, \"#fff\"), (1.0, \"#000\")])".into(),
            Self::Font => "a font family name".into(),
            Self::String { values, accepts_random } => {
                if *accepts_random || values.is_empty() {
                    "a string".into()
                } else {
                    format!("one of {}", values.join(", "))
                }
            }
            Self::Number => "a number".into(),
            Self::Bool => "true or false".into(),
        }
    }

    /// Checks a typed input against this type's range, step and choices.
    fn check(&self, input: Input<'_>) -> Option<Value> {
        match (self, input) {
            (
                Self::Integer {
                    min,
                    max,
                    step,
                    rounds,
                },
                input @ (Input::Int(_) | Input::Num(_)),
            ) => {
                if *step == 0 || min.zip(*max).is_some_and(|(a, b)| a > b) {
                    return None;
                }
                let n = match input {
                    Input::Int(n) => n,
                    Input::Num(n) => {
                        let n = match rounds {
                            Some(Rounding::Ceil) => n.ceil(),
                            Some(Rounding::Floor) => n.floor(),
                            None if n.fract() == 0.0 => n,
                            None => return None,
                        };
                        if !n.is_finite() || n < i64::MIN as f64 || n >= -(i64::MIN as f64) {
                            return None;
                        }
                        n as i64
                    }
                    _ => return None,
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
            (
                Self::Decimal {
                    no_decimals,
                    min,
                    max,
                    step,
                    rounds,
                },
                input @ (Input::Int(_) | Input::Num(_)),
            ) => {
                if *no_decimals > 12 {
                    return None;
                }
                let quantum = 10f64.powi(-(*no_decimals as i32));
                let mut n = snap(input.num()?, quantum, 0.0, *rounds)?;
                if let Some(step) = step {
                    n = snap(n, *step, min.unwrap_or(0.0), *rounds)?;
                }
                n = snap(n, quantum, 0.0, None)?;
                bounds(n, *min, *max).map(Value::Decimal)
            }
            (Self::Length { min, max }, input @ (Input::Int(_) | Input::Num(_))) => {
                bounds(input.num()?, *min, *max).map(Value::Length)
            }
            (Self::Percentage { min, max, rounds }, input @ (Input::Int(_) | Input::Num(_))) => {
                let mut n = input.num()?;
                if rounds.is_some() {
                    n = snap(n, 1.0, 0.0, *rounds)?;
                }
                bounds(n, Some(min.unwrap_or(0.0)), Some(max.unwrap_or(100.0)))
                    .map(Value::Percentage)
            }
            (Self::Color, Input::Text(t)) => parse_color(t).map(Value::Color),
            (Self::Font, Input::Text(name)) => (!name.is_empty()
                && name.chars().count() <= 64
                && !name.chars().any(char::is_control))
            .then(|| Value::Text(name.into())),
            (
                Self::String {
                    values,
                    accepts_random,
                },
                Input::Text(t),
            ) => (!t.contains(['\n', '\r']) && (*accepts_random || values.iter().any(|v| v == t)))
                .then(|| Value::Text(t.into())),
            (Self::Number, input @ (Input::Int(_) | Input::Num(_))) => {
                let n = input.num()? as f32;
                n.is_finite().then_some(Value::Number(n))
            }
            (Self::Bool, Input::Bool(b)) => Some(Value::Bool(b)),
            _ => None,
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
/// What a value is checked from: an integer, a number, text or a flag.
#[derive(Clone, Copy)]
enum Input<'a> {
    Int(i64),
    Num(f64),
    Text(&'a str),
    Bool(bool),
}
impl Input<'_> {
    fn num(self) -> Option<f64> {
        match self {
            Input::Int(n) => Some(n as f64),
            Input::Num(n) if n.is_finite() => Some(n),
            _ => None,
        }
    }
}
fn range_text(min: Option<f64>, max: Option<f64>) -> String {
    match (min, max) {
        (Some(a), Some(b)) => format!(" from {a} to {b}"),
        (Some(a), None) => format!(" of at least {a}"),
        (None, Some(b)) => format!(" of at most {b}"),
        (None, None) => String::new(),
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
pub(super) fn parse_color(raw: &str) -> Option<iced::Color> {
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
