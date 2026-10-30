//! Spacing, sizing and typography tokens.
//!
//! These are the "support vars" for layout: every widget in the crate reads
//! its padding, radii, spacing and heights from [`Metrics`], and its text
//! sizes from [`Typography`], so a whole app can be made denser or airier by
//! changing one value.
//!
//! ```
//! # use iced_libadwaita::{Theme, Metrics};
//! let theme = Theme::light().with_metrics(Metrics::compact());
//! ```

use iced::Padding;

/// What a row (or any [`FlexRow`](crate::widget::FlexRow)) does when its trailing
/// control can't share a line with the title.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CollapseBehavior {
    /// Wrap the control below the title and make it **100% width**: `Fill`-width
    /// controls (sliders, entries) stretch across the row. (default)
    #[default]
    Stacked,
    /// Wrap the control below the title but keep its own width (CSS `flex-wrap`
    /// with `flex-start`).
    Flex,
}

/// Layout tokens. Defaults follow libadwaita's stylesheet (logical px).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    // Spacing scale (use for `spacing(..)` / ad-hoc padding).
    pub space_xxs: f32,
    pub space_xs: f32,
    pub space_s: f32,
    pub space_m: f32,
    pub space_l: f32,
    pub space_xl: f32,
    pub space_xxl: f32,

    // Corner radii.
    pub radius_s: f32,
    /// Buttons, entries (`$button_radius`).
    pub radius_m: f32,
    /// Cards and boxed lists (`$card_radius`).
    pub radius_l: f32,
    /// Menu items (`$menu_radius`).
    pub radius_menu: f32,
    /// Popovers (`$popover_radius` = menu radius + 6).
    pub radius_popover: f32,
    /// Floating dialogs and windows (`--window-radius`).
    pub radius_window: f32,
    /// Pill buttons, circular widgets.
    pub radius_pill: f32,

    // Controls.
    pub button_padding: Padding,
    pub button_min_height: f32,
    pub pill_padding: Padding,
    pub entry_padding: Padding,
    pub entry_min_height: f32,
    pub border_width: f32,
    pub focus_ring_width: f32,
    pub icon_size: f32,
    pub switch_size: f32,
    pub checkbox_size: f32,
    /// Scrollbar rail (hover/hit area) width.
    pub scrollbar_width: f32,
    /// Visible slider width at rest (overlay indicator, 3px in Adwaita; we use
    /// 4 since iced cannot grow it on hover).
    pub scrollbar_slider_width: f32,
    /// Gap between the slider and the edge.
    pub scrollbar_margin: f32,
    /// `GtkProgressBar` trough thickness.
    pub progress_height: f32,

    // Rows & lists.
    pub row_padding: Padding,
    pub row_min_height: f32,
    pub row_spacing: f32,
    /// Minimum width a row's title area keeps on the first line. When the
    /// suffix control would squeeze it below this, the control wraps to its own
    /// line (CSS `flex-wrap`).
    pub row_wrap_basis: f32,
    /// Default [`CollapseBehavior`] for rows whose suffix wrapped.
    pub row_collapse: CollapseBehavior,
    pub list_spacing: f32,

    // Containers.
    pub headerbar_height: f32,
    pub headerbar_padding: Padding,
    pub toolbar_spacing: f32,
    pub card_padding: Padding,
    pub dialog_padding: Padding,
    pub dialog_width: f32,
    /// `popover > contents` padding.
    pub popover_padding: f32,

    // Pages.
    pub page_padding: Padding,
    /// Space between preferences groups.
    pub group_spacing: f32,
    /// `AdwClamp` default maximum size.
    pub clamp_maximum: f32,
    pub sidebar_width: f32,
    /// `.navigation-sidebar`: padding around the list.
    pub sidebar_padding: f32,
    /// Gap between sidebar rows.
    pub sidebar_spacing: f32,
    pub sidebar_row_height: f32,
    /// Horizontal padding inside a sidebar row.
    pub sidebar_row_padding: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            space_xxs: 2.0,
            space_xs: 4.0,
            space_s: 6.0,
            space_m: 12.0,
            space_l: 18.0,
            space_xl: 24.0,
            space_xxl: 36.0,

            radius_s: 4.0,
            radius_m: 9.0,
            radius_l: 12.0,
            radius_menu: 9.0,
            radius_popover: 15.0,
            radius_window: 15.0,
            radius_pill: 999.0,

            button_padding: Padding::from([5.0, 10.0]),
            button_min_height: 34.0,
            pill_padding: Padding::from([10.0, 32.0]),
            entry_padding: Padding::from([6.0, 9.0]),
            entry_min_height: 34.0,
            border_width: 1.0,
            focus_ring_width: 2.0,
            icon_size: 16.0,
            switch_size: 26.0,
            checkbox_size: 20.0,
            scrollbar_width: 8.0,
            scrollbar_slider_width: 4.0,
            scrollbar_margin: 2.0,
            progress_height: 8.0,

            row_padding: Padding::from([6.0, 12.0]),
            row_min_height: 50.0,
            row_spacing: 12.0,
            row_wrap_basis: 128.0,
            row_collapse: CollapseBehavior::Stacked,
            list_spacing: 0.0,

            headerbar_height: 47.0,
            headerbar_padding: Padding::from([0.0, 6.0]),
            toolbar_spacing: 6.0,
            card_padding: Padding::from(12.0),
            dialog_padding: Padding::from(24.0),
            dialog_width: 360.0,
            popover_padding: 8.0,

            page_padding: Padding::from([24.0, 12.0]),
            group_spacing: 24.0,
            clamp_maximum: 600.0,
            sidebar_width: 240.0,
            sidebar_padding: 6.0,
            sidebar_spacing: 2.0,
            sidebar_row_height: 36.0,
            sidebar_row_padding: 12.0,
        }
    }
}

impl Metrics {
    /// Upper bounds for corner radii, taken from the GNOME HIG / libadwaita
    /// stylesheet: 4px small, 9px controls and menus, 12px cards, 15px popovers,
    /// dialogs and windows.
    /// [`Theme::with_metrics`](crate::Theme::with_metrics) clamps to these.
    pub const MAX_RADIUS_S: f32 = 4.0;
    pub const MAX_RADIUS_M: f32 = 9.0;
    pub const MAX_RADIUS_L: f32 = 12.0;
    pub const MAX_RADIUS_MENU: f32 = 9.0;
    pub const MAX_RADIUS_POPOVER: f32 = 15.0;
    pub const MAX_RADIUS_WINDOW: f32 = 15.0;

    /// Returns a copy with radii limited to the HIG maxima.
    pub fn clamped(mut self) -> Self {
        self.radius_s = self.radius_s.clamp(0.0, Self::MAX_RADIUS_S);
        self.radius_m = self.radius_m.clamp(0.0, Self::MAX_RADIUS_M);
        self.radius_l = self.radius_l.clamp(0.0, Self::MAX_RADIUS_L);
        self.radius_menu = self.radius_menu.clamp(0.0, Self::MAX_RADIUS_MENU);
        self.radius_popover = self.radius_popover.clamp(0.0, Self::MAX_RADIUS_POPOVER);
        self.radius_window = self.radius_window.clamp(0.0, Self::MAX_RADIUS_WINDOW);
        self
    }

    /// A denser variant (roughly GNOME's "compact" / `.compact` toolbars).
    pub fn compact() -> Self {
        let d = Self::default();
        Self {
            button_padding: Padding::from([2.0, 8.0]),
            button_min_height: 28.0,
            entry_padding: Padding::from([4.0, 8.0]),
            entry_min_height: 28.0,
            row_padding: Padding::from([4.0, 10.0]),
            row_min_height: 40.0,
            headerbar_height: 38.0,
            group_spacing: 16.0,
            sidebar_row_height: 30.0,
            page_padding: Padding::from([16.0, 10.0]),
            ..d
        }
    }
}

/// Text sizes, expressed as in libadwaita's `.title-1` … `.caption` classes
/// (percent of the base size). `base` is the body size in logical px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Typography {
    pub base: f32,
    pub title_1: f32,
    pub title_2: f32,
    pub title_3: f32,
    pub title_4: f32,
    pub heading: f32,
    pub body: f32,
    pub caption_heading: f32,
    pub caption: f32,
    pub monospace: f32,
}

impl Typography {
    pub fn with_base(base: f32) -> Self {
        Self {
            base,
            title_1: base * 1.81,
            title_2: base * 1.36,
            title_3: base * 1.20,
            title_4: base * 1.10,
            heading: base,
            body: base,
            caption_heading: base * 0.82,
            caption: base * 0.82,
            monospace: base,
        }
    }
}

impl Default for Typography {
    /// 11pt ≈ 14.67px, GNOME's default interface font size.
    fn default() -> Self {
        Self::with_base(14.67)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_radius_is_15_and_capped() {
        assert_eq!(Metrics::default().radius_window, 15.0);
        let m = Metrics { radius_window: 40.0, ..Metrics::default() }.clamped();
        assert_eq!(m.radius_window, Metrics::MAX_RADIUS_WINDOW);
    }
}

#[cfg(test)]
mod collapse_tests {
    use super::*;

    #[test]
    fn rows_stack_by_default() {
        assert_eq!(CollapseBehavior::default(), CollapseBehavior::Stacked);
        assert_eq!(Metrics::default().row_collapse, CollapseBehavior::Stacked);
    }
}
