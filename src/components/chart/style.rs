//! What a cartesian chart draws, and in which colors. Every element can be switched off and
//! every color replaced, so a chart can be as bare as a sparkline or as full as a report.

use crate::Theme;
use iced::Color;
use std::rc::Rc;

/// A color by theme and series (or bar) index.
pub type Paint = Rc<dyn Fn(&Theme, usize) -> Color>;
/// A color by theme.
pub type ThemePaint = Rc<dyn Fn(&Theme) -> Color>;

/// The elements and colors of a cartesian chart ([`LineChart`](crate::widgets::LineChart),
/// [`BarChart`](crate::widgets::BarChart)). The default draws everything with the series colors.
#[derive(Clone)]
pub struct ChartStyle {
    /// The value axis: its tick labels (and the room they take).
    pub value_axis: bool,
    /// The category axis: its baseline and labels (and the room they take).
    pub category_axis: bool,
    /// Gridlines at the value ticks.
    pub grid: bool,
    /// The tooltip that follows the pointer.
    pub tooltip: bool,
    /// The highlight of the category under the pointer.
    pub hover: bool,
    /// Replaces the series colors.
    pub paint: Option<Paint>,
    /// The color of marks outside a selection; by default the mark's own color at `dim`.
    pub unselected: Option<ThemePaint>,
    /// The opacity of marks outside a selection when `unselected` is not set.
    pub dim: f32,
    /// The corner radius of bars.
    pub radius: f32,
    /// A bar with a value is at least this tall (px), so small values stay visible.
    pub min_bar: f32,
    /// A bar with no value is this tall (px): a stub, or 0 for nothing.
    pub empty_bar: f32,
}

impl Default for ChartStyle {
    fn default() -> Self {
        Self {
            value_axis: true,
            category_axis: true,
            grid: true,
            tooltip: true,
            hover: true,
            paint: None,
            unselected: None,
            dim: 0.3,
            radius: 2.0,
            min_bar: 0.0,
            empty_bar: 0.0,
        }
    }
}

impl ChartStyle {
    /// Nothing but the marks: no axes, no grid, no tooltip, no hover highlight. Bars keep a
    /// minimum height and an empty stub, accented, with the unselected ones in a dim text color.
    pub fn minimal() -> Self {
        Self {
            value_axis: false,
            category_axis: false,
            grid: false,
            tooltip: false,
            hover: false,
            paint: Some(Rc::new(|t, _| t.palette.accent_bg_color)),
            unselected: Some(Rc::new(|t| crate::color::alpha(t.semantic.window.fg, 0.22))),
            min_bar: 4.0,
            empty_bar: 2.0,
            ..Self::default()
        }
    }
    /// Whether any axis is drawn.
    pub fn axes(&self) -> bool {
        self.value_axis || self.category_axis
    }
    /// The color of mark `i`.
    pub fn color(&self, theme: &Theme, i: usize) -> Color {
        match &self.paint {
            Some(p) => p(theme, i),
            None => super::color::series(theme, i),
        }
    }
    /// The color of mark `i` when it is outside the selection.
    pub fn dimmed(&self, theme: &Theme, i: usize) -> Color {
        match &self.unselected {
            Some(u) => u(theme),
            None => crate::color::alpha(self.color(theme, i), self.dim),
        }
    }
}

/// The builder methods every styled chart has, written once. Used inside an `impl` block of a
/// chart with a `style: ChartStyle` field.
macro_rules! style_methods {
    () => {
        /// Replaces the whole [`ChartStyle`].
        pub fn style(mut self, style: crate::components::chart::style::ChartStyle) -> Self {
            self.style = style;
            self
        }
        /// Both axes at once (labels and baseline). Gridlines are separate: [`grid`](Self::grid).
        pub fn axes(mut self, show: bool) -> Self {
            self.style.value_axis = show;
            self.style.category_axis = show;
            self
        }
        /// The value axis labels (on by default).
        pub fn value_axis(mut self, show: bool) -> Self {
            self.style.value_axis = show;
            self
        }
        /// The category axis baseline and labels (on by default).
        pub fn category_axis(mut self, show: bool) -> Self {
            self.style.category_axis = show;
            self
        }
        /// Gridlines at the value ticks (on by default).
        pub fn grid(mut self, show: bool) -> Self {
            self.style.grid = show;
            self
        }
        /// The tooltip that follows the pointer (on by default).
        pub fn tooltip(mut self, show: bool) -> Self {
            self.style.tooltip = show;
            self
        }
        /// The highlight of the category under the pointer (on by default).
        pub fn hover(mut self, show: bool) -> Self {
            self.style.hover = show;
            self
        }
        /// One color for every series, in place of the theme's series colors.
        pub fn color(mut self, color: iced::Color) -> Self {
            self.style.paint = Some(std::rc::Rc::new(move |_, _| color));
            self
        }
        /// The color of each series by theme and index, in place of the theme's series colors.
        pub fn colors(
            mut self,
            paint: impl Fn(&crate::Theme, usize) -> iced::Color + 'static,
        ) -> Self {
            self.style.paint = Some(std::rc::Rc::new(paint));
            self
        }
    };
}
pub(crate) use style_methods;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_draws_everything() {
        let s = ChartStyle::default();
        assert!(s.value_axis && s.category_axis && s.grid && s.tooltip && s.hover);
    }

    #[test]
    fn minimal_draws_only_marks() {
        let s = ChartStyle::minimal();
        assert!(!s.axes() && !s.grid && !s.tooltip && !s.hover);
        let t = Theme::light();
        assert_eq!(s.color(&t, 3), t.palette.accent_bg_color);
        assert!(s.dimmed(&t, 0).a < 0.3);
        assert!(s.min_bar > 0.0 && s.empty_bar > 0.0);
    }

    #[test]
    fn colors_default_to_the_series_colors() {
        let t = Theme::light();
        let s = ChartStyle::default();
        assert_eq!(s.color(&t, 1), super::super::color::series(&t, 1));
        assert!((s.dimmed(&t, 1).a - 0.3).abs() < 1e-6);
    }
}
