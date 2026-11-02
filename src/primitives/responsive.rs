//! Headless responsive logic: the conditions and the choice, with no widgets.
//!
//! [`MetaComponent`](crate::widget::MetaComponent) applies it to a container's size; any
//! other component (or plain code) can ask the same question of a width.

use crate::breakpoints::{Breakpoints, WidthClass};

/// When a variant applies to its container. All set conditions must hold.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct When {
    min_width: Option<f32>,
    from_class: Option<WidthClass>,
}

impl When {
    /// Always.
    pub const fn always() -> Self {
        Self {
            min_width: None,
            from_class: None,
        }
    }
    /// The container is at least `px` wide.
    pub const fn min_width(px: f32) -> Self {
        Self {
            min_width: Some(px),
            from_class: None,
        }
    }
    /// The container's width class (on the user's `[breakpoints]`) is `class` or larger.
    pub const fn from_class(class: WidthClass) -> Self {
        Self {
            min_width: None,
            from_class: Some(class),
        }
    }
    /// Also requires at least `px`.
    pub const fn and_min_width(mut self, px: f32) -> Self {
        self.min_width = Some(px);
        self
    }
    /// Also requires `class` or larger.
    pub const fn and_from_class(mut self, class: WidthClass) -> Self {
        self.from_class = Some(class);
        self
    }

    pub fn holds(&self, width: f32, scale: &Breakpoints) -> bool {
        self.min_width.is_none_or(|m| width >= m)
            && self
                .from_class
                .is_none_or(|c| WidthClass::of_with(scale, width) >= c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const B: Breakpoints = Breakpoints {
        sm: 576.0,
        md: 768.0,
        lg: 992.0,
        xl: 1200.0,
        xxl: 1400.0,
    };

    #[test]
    fn conditions_combine() {
        let w = When::from_class(WidthClass::Lg).and_min_width(1000.0);
        assert!(!w.holds(990.0, &B));
        assert!(w.holds(1000.0, &B));
        assert!(
            !When::min_width(1000.0)
                .and_from_class(WidthClass::Xl)
                .holds(1100.0, &B)
        );
    }
}
