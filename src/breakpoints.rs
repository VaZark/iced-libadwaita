//! Window width classes.
//!
//! The scale follows the conventions of popular UI kits (Bootstrap 5, and Mantine whose names
//! are shifted by one) rather than a single phone width: `sm` 576, `md` 768, `lg` 992, `xl` 1200,
//! `xxl` 1400 are *minimum* widths. A width below `sm` is a phone, below `md` a tablet in
//! portrait or a small window: neither has room for a sidebar beside the content.
//!
//! The constants are **fallbacks**: the desktop config's `[breakpoints]` section overrides each
//! one (`sm`, `md`, `lg`, `xl`, `xxl`), and [`Breakpoints::current`] is what widgets read.
//!
//! The defaults built on it:
//!
//! * **mobile mode** (no window buttons, full-page menus): width below [`SM`];
//! * **sidebar collapse**: width below [`MD`], or whenever the content area would be narrower
//!   than the minimum content width (see `NavigationSplitView::min_content_width`). Content
//!   must never feel squeezed.

/// Small: landscape phones and up.
pub const SM: f32 = 576.0;
/// Medium: tablets in portrait and up. Split views are side by side from here.
pub const MD: f32 = 768.0;
/// Large.
pub const LG: f32 = 992.0;
/// Extra large.
pub const XL: f32 = 1200.0;
/// Extra extra large.
pub const XXL: f32 = 1400.0;

/// At or below this width (just under [`SM`]) the window is
/// in *mobile mode*: window decorations are hidden completely and menus become full pages.
/// Corners are unaffected: use [`windows::State`](crate::windows::State) (rounded unless actually maximized/fullscreen).
pub const MOBILE_MAX_WIDTH: f32 = SM - 1.0;

/// The mobile breakpoint in effect: the desktop config's `[window-decorations] mobile-max-width`
/// (default [`MOBILE_MAX_WIDTH`]).
pub fn mobile_max_width() -> f32 {
    crate::config::current().mobile_max_width
}

/// Whether a window of this width is in mobile mode.
pub fn is_mobile(width: f32) -> bool {
    width <= mobile_max_width()
}

/// The width scale in effect: the fallback constants unless the desktop config overrides them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Breakpoints {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

impl Default for Breakpoints {
    fn default() -> Self {
        Self {
            sm: SM,
            md: MD,
            lg: LG,
            xl: XL,
            xxl: XXL,
        }
    }
}

impl Breakpoints {
    /// The scale from the installed desktop config.
    pub fn current() -> Self {
        crate::config::current().breakpoints
    }

    /// Overlays optional `[breakpoints]` theme tokens on this base scale.
    /// Missing tokens inherit the base. The entire overlay is ignored if any supplied
    /// value is invalid (outside 1..=16384 logical pixels), or the result is unordered.
    pub fn with_tokens(self, tokens: &crate::theme_config::Tokens) -> Self {
        let mut scale = self;
        for (key, target) in [
            ("sm", &mut scale.sm),
            ("md", &mut scale.md),
            ("lg", &mut scale.lg),
            ("xl", &mut scale.xl),
            ("xxl", &mut scale.xxl),
        ] {
            if tokens.get("breakpoints", key).is_some() {
                match tokens.number("breakpoints", key) {
                    Some(value) if (1.0..=16384.0).contains(&value) => *target = value,
                    _ => return self,
                }
            }
        }
        if scale.is_ordered() { scale } else { self }
    }

    /// Whether the values are in increasing order (a config with a shuffled scale is ignored).
    pub fn is_ordered(&self) -> bool {
        self.xxl.is_finite()
            && 0.0 < self.sm
            && self.sm < self.md
            && self.md < self.lg
            && self.lg < self.xl
            && self.xl < self.xxl
    }
}

/// A width class, from [`WidthClass::of`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WidthClass {
    /// Below [`SM`].
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
}

impl WidthClass {
    /// The class of a window `width` in logical pixels, on the configured scale.
    pub fn of(width: f32) -> Self {
        Self::of_with(&Breakpoints::current(), width)
    }

    /// The class of `width` on an explicit scale.
    pub fn of_with(b: &Breakpoints, width: f32) -> Self {
        match width {
            w if w >= b.xxl => Self::Xxl,
            w if w >= b.xl => Self::Xl,
            w if w >= b.lg => Self::Lg,
            w if w >= b.md => Self::Md,
            w if w >= b.sm => Self::Sm,
            _ => Self::Xs,
        }
    }

    /// The next wider class, if any.
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Xs => Some(Self::Sm),
            Self::Sm => Some(Self::Md),
            Self::Md => Some(Self::Lg),
            Self::Lg => Some(Self::Xl),
            Self::Xl => Some(Self::Xxl),
            Self::Xxl => None,
        }
    }

    /// The next narrower class, if any.
    pub fn prev(self) -> Option<Self> {
        match self {
            Self::Xs => None,
            Self::Sm => Some(Self::Xs),
            Self::Md => Some(Self::Sm),
            Self::Lg => Some(Self::Md),
            Self::Xl => Some(Self::Lg),
            Self::Xxl => Some(Self::Xl),
        }
    }

    /// The minimum width of this class on the configured scale (`0` for [`WidthClass::Xs`]).
    pub fn min_width(self) -> f32 {
        self.min_width_with(&Breakpoints::current())
    }

    /// The minimum width of this class on an explicit scale.
    pub fn min_width_with(self, b: &Breakpoints) -> f32 {
        match self {
            Self::Xs => 0.0,
            Self::Sm => b.sm,
            Self::Md => b.md,
            Self::Lg => b.lg,
            Self::Xl => b.xl,
            Self::Xxl => b.xxl,
        }
    }
}

/// A validated, order-independent set of `breakpoint:variant` rules.
///
/// Entries are comma-separated. Each class may occur once; names are case-sensitive.
/// Empty text is a valid no-op. Validation is atomic: an invalid entry rejects all rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rules<'a> {
    entries: Vec<(WidthClass, &'a str)>,
}

/// Why a breakpoint rule was rejected. Entry numbers are one-based.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleError {
    pub entry: usize,
    pub reason: &'static str,
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "breakpoint rule {}: {}", self.entry, self.reason)
    }
}

impl std::error::Error for RuleError {}

impl<'a> Rules<'a> {
    /// Parses rules, checking variant names against the widget's declarations.
    /// Names follow `Schema::variant`: ASCII letters, digits, hyphens and underscores.
    pub fn parse(
        text: &'a str,
        mut variant_exists: impl FnMut(&str) -> bool,
    ) -> Result<Self, RuleError> {
        let mut entries = Vec::new();
        if text.trim().is_empty() {
            return Ok(Self { entries });
        }
        for (index, entry) in text.split(',').enumerate() {
            let error = |reason| RuleError {
                entry: index + 1,
                reason,
            };
            let (class, variant) = entry
                .split_once(':')
                .ok_or_else(|| error("expected breakpoint:variant"))?;
            let class = match class.trim() {
                "xs" => WidthClass::Xs,
                "sm" => WidthClass::Sm,
                "md" => WidthClass::Md,
                "lg" => WidthClass::Lg,
                "xl" => WidthClass::Xl,
                "xxl" => WidthClass::Xxl,
                _ => return Err(error("unknown breakpoint")),
            };
            let variant = variant.trim();
            if variant.is_empty()
                || !variant
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
            {
                return Err(error("invalid variant name"));
            }
            if !variant_exists(variant) {
                return Err(error("undeclared variant"));
            }
            if entries.iter().any(|(existing, _)| *existing == class) {
                return Err(error("duplicate breakpoint"));
            }
            entries.push((class, variant));
        }
        entries.sort_by_key(|(class, _)| *class);
        Ok(Self { entries })
    }

    /// Selects the widest matching minimum width, or `fallback` before any rule matches.
    /// Negative/non-finite widths and invalid scales are neutral, returning `fallback`.
    /// `xs` starts at zero; omitted classes inherit the preceding rule.
    pub fn select<'b>(&'b self, scale: &Breakpoints, width: f32, fallback: &'b str) -> &'b str {
        if !width.is_finite() || width < 0.0 || !scale.is_ordered() {
            return fallback;
        }
        self.entries
            .iter()
            .rev()
            .find(|(class, _)| width >= class.min_width_with(scale))
            .map_or(fallback, |(_, variant)| *variant)
    }
}

/// Resolves a widget's `breakpoint-rules` from explicit theme tokens and a base width scale.
/// Optional `[breakpoints]` tokens overlay the base atomically via [`Breakpoints::with_tokens`].
/// Invalid or absent rules preserve `fallback` (which need not be a declared variant).
/// This does not apply styling: pass the returned name to the widget's variant API.
pub fn variant_for_tokens(
    tokens: &crate::theme_config::Tokens,
    widget: &str,
    scale: &Breakpoints,
    width: f32,
    fallback: &str,
) -> String {
    let scale = scale.with_tokens(tokens);
    tokens
        .text(widget, "breakpoint-rules")
        .and_then(|text| Rules::parse(text, |name| tokens.variant(widget, name).exists()).ok())
        .map_or_else(
            || fallback.to_owned(),
            |rules| rules.select(&scale, width, fallback).to_owned(),
        )
}

/// Resolves a widget's rules using the installed theme config and desktop breakpoint scale.
/// For a particular `Theme`, use [`variant_for_tokens`] with `&theme.tokens` instead.
pub fn variant_for(widget: &str, width: f32, fallback: &str) -> String {
    variant_for_tokens(
        &crate::theme_config::current().tokens(),
        widget,
        &Breakpoints::current(),
        width,
        fallback,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(text: &str) -> crate::theme_config::Tokens {
        crate::theme_config::ThemeConfig::builder("breakpoint-tests")
            .files(false)
            .toolkit_text(text)
            .build()
            .tokens()
    }

    #[test]
    fn token_scale_partially_overrides_and_inherits_base() {
        let base = Breakpoints {
            sm: 500.0,
            ..Breakpoints::default()
        };
        assert_eq!(base.with_tokens(&tokens("")), base);
        let scale = base.with_tokens(&tokens(r##"{
    "breakpoints": {
        "md": 700,
        "xl": 1250,
    },
}
"##));
        assert_eq!(
            scale,
            Breakpoints {
                md: 700.0,
                xl: 1250.0,
                ..base
            }
        );
        assert_eq!(
            base.with_tokens(&tokens(r##"{
    "breakpoints": {
        "sm": 1,
        "xxl": 16384,
    },
}
"##)),
            Breakpoints {
                sm: 1.0,
                xxl: 16384.0,
                ..base
            }
        );
    }

    #[test]
    fn unordered_token_scale_falls_back_atomically() {
        let base = Breakpoints::default();
        for text in [
            "sm=600px\nmd=600px",
            "sm=600px\nmd=1000px",
            "sm=600px\nxxl=1100px",
        ] {
            assert_eq!(
                base.with_tokens(&tokens(&format!("[breakpoints]\n{text}"))),
                base
            );
        }
        // The schema discards out-of-range/non-numeric inputs before they become tokens.
        for value in ["0", "16385", "NaN", "bad"] {
            assert_eq!(
                base.with_tokens(&tokens(&format!("[breakpoints]\nmd={value}"))),
                base
            );
        }
    }

    #[test]
    fn variant_selection_uses_local_token_scale() {
        let base = Breakpoints::default();
        let local = tokens(r##"{
    "breakpoints": {
        "md": 700,
    },
    "tabs": {
        "breakpoint-rules": "md:outline",
    },
}
"##);
        assert_eq!(
            variant_for_tokens(&local, "tabs", &base, 699.9, "default"),
            "default"
        );
        assert_eq!(
            variant_for_tokens(&local, "tabs", &base, 700.0, "default"),
            "outline"
        );
        let invalid = tokens(r##"{
    "breakpoints": {
        "md": 1100,
    },
    "tabs": {
        "breakpoint-rules": "md:outline",
    },
}
"##);
        assert_eq!(
            variant_for_tokens(&invalid, "tabs", &base, base.md, "default"),
            "outline"
        );
    }

    #[test]
    fn rules_cascade_at_inclusive_boundaries() {
        let b = Breakpoints::default();
        let rules = Rules::parse("xxl:wide, md:medium, xs:small", |_| true).unwrap();
        for (width, expected) in [
            (0.0, "small"),
            (575.9, "small"),
            (768.0, "medium"),
            (1399.9, "medium"),
            (1400.0, "wide"),
        ] {
            assert_eq!(rules.select(&b, width, "neutral"), expected);
        }
        for class in [
            WidthClass::Xs,
            WidthClass::Sm,
            WidthClass::Md,
            WidthClass::Lg,
            WidthClass::Xl,
            WidthClass::Xxl,
        ] {
            let name = match class {
                WidthClass::Xs => "xs",
                WidthClass::Sm => "sm",
                WidthClass::Md => "md",
                WidthClass::Lg => "lg",
                WidthClass::Xl => "xl",
                WidthClass::Xxl => "xxl",
            };
            let text = format!("{name}:chosen");
            let rules = Rules::parse(&text, |_| true).unwrap();
            let min = class.min_width_with(&b);
            assert_eq!(rules.select(&b, min, "neutral"), "chosen");
            if min > 0.0 {
                assert_eq!(rules.select(&b, min - 0.1, "neutral"), "neutral");
            }
        }
    }

    #[test]
    fn rules_reject_malformed_and_unknown_entries() {
        for text in [
            "md",
            "md:",
            ":known",
            "600:known",
            "MD:known",
            "md:bad name",
            "md:known:extra",
            "md:known,",
            "md:known,md:known",
            "md:unknown",
        ] {
            assert!(
                Rules::parse(text, |name| name == "known").is_err(),
                "{text}"
            );
        }
        assert!(Rules::parse(" sm : known ", |name| name == "known").is_ok());
    }

    #[test]
    fn invalid_widths_and_scales_are_neutral() {
        let rules = Rules::parse("xs:chosen", |_| true).unwrap();
        let b = Breakpoints::default();
        for width in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(rules.select(&b, width, "neutral"), "neutral");
        }
        for scale in [
            Breakpoints {
                xxl: f32::INFINITY,
                ..b
            },
            Breakpoints { sm: f32::NAN, ..b },
            Breakpoints { md: b.sm, ..b },
        ] {
            assert_eq!(rules.select(&scale, 800.0, "neutral"), "neutral");
        }
        assert_eq!(
            Rules::parse("", |_| false)
                .unwrap()
                .select(&b, 900.0, "neutral"),
            "neutral"
        );
        assert_eq!(
            variant_for_tokens(&Default::default(), "button", &b, 900.0, "neutral"),
            "neutral"
        );
    }

    #[test]
    fn classes_start_at_their_minimum_width() {
        assert_eq!(WidthClass::of(360.0), WidthClass::Xs);
        assert_eq!(WidthClass::of(575.9), WidthClass::Xs);
        assert_eq!(WidthClass::of(576.0), WidthClass::Sm);
        assert_eq!(WidthClass::of(768.0), WidthClass::Md);
        assert_eq!(WidthClass::of(1000.0), WidthClass::Lg);
        assert_eq!(WidthClass::of(1400.0), WidthClass::Xxl);
        assert!(WidthClass::of(800.0) > WidthClass::of(600.0));
    }

    #[test]
    fn the_scale_can_be_replaced() {
        let b = Breakpoints {
            sm: 500.0,
            md: 640.0,
            lg: 900.0,
            xl: 1100.0,
            xxl: 1300.0,
        };
        assert!(b.is_ordered());
        assert_eq!(WidthClass::of_with(&b, 520.0), WidthClass::Sm);
        assert_eq!(WidthClass::of_with(&b, 640.0), WidthClass::Md);
        assert!(!Breakpoints { sm: 800.0, ..b }.is_ordered());
    }
}
