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
        Self { sm: SM, md: MD, lg: LG, xl: XL, xxl: XXL }
    }
}

impl Breakpoints {
    /// The scale from the installed desktop config.
    pub fn current() -> Self {
        crate::config::current().breakpoints
    }

    /// Whether the values are in increasing order (a config with a shuffled scale is ignored).
    pub fn is_ordered(&self) -> bool {
        0.0 < self.sm && self.sm < self.md && self.md < self.lg && self.lg < self.xl && self.xl < self.xxl
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let b = Breakpoints { sm: 500.0, md: 640.0, lg: 900.0, xl: 1100.0, xxl: 1300.0 };
        assert!(b.is_ordered());
        assert_eq!(WidthClass::of_with(&b, 520.0), WidthClass::Sm);
        assert_eq!(WidthClass::of_with(&b, 640.0), WidthClass::Md);
        assert!(!Breakpoints { sm: 800.0, ..b }.is_ordered());
    }
}
