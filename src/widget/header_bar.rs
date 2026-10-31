//! `AdwHeaderBar` and `AdwWindowTitle`.

use super::Element;
use crate::style::container as sc;
use crate::widget::typography;
use crate::{DecorationLayout, Icon, Metrics, Theme, WindowButton};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{column, container, mouse_area, row, space, text};
use iced::{Length, Padding};

/// Window controls for a [`HeaderBar`].
///
/// Which buttons appear, and on which side, follows the user's GNOME
/// `button-layout` (`DecorationLayout::detect`, GNOME's default is a lone close
/// button at the end). A button is shown only if the layout includes it *and* you
/// provide its message. The icons come from the current icon theme.
///
/// In mobile mode ([`window::is_mobile`](crate::window::is_mobile)) don't add
/// controls at all.
pub struct WindowControls<M> {
    pub on_minimize: Option<M>,
    pub on_maximize: Option<M>,
    pub on_close: Option<M>,
    /// Shows the restore icon on the maximize button.
    pub maximized: bool,
    /// Overrides the detected layout (e.g. for tests or non-GNOME setups).
    pub layout: Option<DecorationLayout>,
}

impl<M> Default for WindowControls<M> {
    fn default() -> Self {
        Self {
            on_minimize: None,
            on_maximize: None,
            on_close: None,
            maximized: false,
            layout: None,
        }
    }
}

/// `AdwWindowTitle`: centered title with optional subtitle.
pub fn window_title<'a, M: 'a>(title: &str, subtitle: Option<&str>) -> Element<'a, M> {
    let mut c = column![typography::heading(title.to_string())].align_x(Horizontal::Center);
    if let Some(s) = subtitle {
        c = c.push(
            typography::caption(s.to_string())
                .wrapping(text::Wrapping::None)
                .style(crate::style::text::dim),
        );
    }
    c.into()
}

/// `AdwHeaderBar`.
pub struct HeaderBar<'a, M> {
    start: Vec<Element<'a, M>>,
    end: Vec<Element<'a, M>>,
    title: Option<String>,
    subtitle: Option<String>,
    title_widget: Option<Element<'a, M>>,
    controls: Option<WindowControls<M>>,
    show_start_buttons: bool,
    show_end_buttons: bool,
    on_drag: Option<M>,
    on_double_click: Option<M>,
    corners: (f32, f32),
    flat: Option<bool>,
    metrics: Metrics,
}

/// Builds an empty [`HeaderBar`].
pub fn header_bar<'a, M>() -> HeaderBar<'a, M> {
    HeaderBar::new()
}

impl<'a, M> HeaderBar<'a, M> {
    pub fn new() -> Self {
        Self {
            start: Vec::new(),
            end: Vec::new(),
            title: None,
            subtitle: None,
            title_widget: None,
            controls: None,
            show_start_buttons: true,
            show_end_buttons: true,
            on_drag: None,
            on_double_click: None,
            corners: (0.0, 0.0),
            flat: None,
            metrics: Metrics::desktop(),
        }
    }
    /// `pack_start`.
    pub fn pack_start(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.start.push(e.into());
        self
    }
    /// `pack_end` (added right-to-left like GTK: first call is rightmost).
    pub fn pack_end(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.end.insert(0, e.into());
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn subtitle(mut self, t: impl Into<String>) -> Self {
        self.subtitle = Some(t.into());
        self
    }
    /// `title_widget`: replaces the title (e.g. a [`ToggleGroup`](super::ToggleGroup)).
    pub fn title_widget(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.title_widget = Some(e.into());
        self
    }
    /// `show-start-title-buttons`: window buttons the layout puts at the start.
    /// In split views only the pane at that window edge shows them.
    pub fn show_start_title_buttons(mut self, show: bool) -> Self {
        self.show_start_buttons = show;
        self
    }
    /// `show-end-title-buttons`: window buttons the layout puts at the end.
    pub fn show_end_title_buttons(mut self, show: bool) -> Self {
        self.show_end_buttons = show;
        self
    }
    pub fn window_controls(mut self, c: WindowControls<M>) -> Self {
        self.controls = Some(c);
        self
    }
    /// Message sent on press of the empty header area (use `window::drag`).
    pub fn on_drag(mut self, m: M) -> Self {
        self.on_drag = Some(m);
        self
    }
    /// Message for double-click (use `window::toggle_maximize`).
    pub fn on_double_click(mut self, m: M) -> Self {
        self.on_double_click = Some(m);
        self
    }
    /// Rounds the bar's top corners to match the window corners (pass
    /// [`window::State::radius`](crate::window::State::radius); 0 when maximized).
    pub fn window_radius(mut self, r: f32) -> Self {
        self.corners = (r, r);
        self
    }
    /// Flat header (`%headerbar-flat`: no background, no shadow): the bar shows
    /// whatever is behind it, so it flows into the content. Used in dialogs
    /// (Nautilus-style) and inline contexts.
    ///
    /// Without a call to this, the bar follows [`Theme::flat_headerbars`](crate::Theme::flat_headerbars).
    pub fn flat(mut self, flat: bool) -> Self {
        self.flat = Some(flat);
        self
    }
    /// Rounds only the outer corners: `left` and `right` radii. Use in split views
    /// where each pane has its own header (left pane: `(r, 0)`, right pane: `(0, r)`).
    pub fn window_corners(mut self, left: f32, right: f32) -> Self {
        self.corners = (left, right);
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> Default for HeaderBar<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

/// A window button: a 24px circle with the icon theme's symbol.
fn ctl<'a, M: Clone + 'a>(icon: Icon, msg: M) -> Element<'a, M> {
    let m = Metrics::desktop();
    iced::widget::button(container(icon.view(m.icon_size)).center(Length::Fixed(24.0)))
        .padding(0)
        .style(crate::style::button::circular)
        .on_press(msg)
        .into()
}

impl<'a, M: Clone + 'a> From<HeaderBar<'a, M>> for Element<'a, M> {
    fn from(h: HeaderBar<'a, M>) -> Self {
        let m = h.metrics;
        let (corner_l, corner_r) = h.corners;
        let flat = h.flat;
        let mut start = h.start;
        let mut end = h.end;
        if let Some(c) = h.controls {
            let layout = c.layout.clone().unwrap_or_else(DecorationLayout::detect);
            let make = |b: WindowButton| -> Option<Element<'a, M>> {
                match b {
                    WindowButton::Minimize => {
                        c.on_minimize.clone().map(|m| ctl(Icon::WindowMinimize, m))
                    }
                    WindowButton::Maximize => c.on_maximize.clone().map(|m| {
                        ctl(
                            if c.maximized {
                                Icon::WindowRestore
                            } else {
                                Icon::WindowMaximize
                            },
                            m,
                        )
                    }),
                    WindowButton::Close => c.on_close.clone().map(|m| ctl(Icon::WindowClose, m)),
                }
            };
            if h.show_start_buttons {
                let mut buttons: Vec<_> = layout.start.iter().copied().filter_map(&make).collect();
                buttons.append(&mut start);
                start = buttons;
            }
            if h.show_end_buttons {
                end.extend(layout.end.iter().copied().filter_map(&make));
            }
        }
        let title: Element<'a, M> = match (h.title_widget, h.title) {
            (Some(w), _) => w,
            (None, Some(t)) => window_title(&t, h.subtitle.as_deref()),
            (None, None) => space().into(),
        };
        let side = |items: Vec<Element<'a, M>>| -> Element<'a, M> {
            row(items)
                .spacing(m.space_s)
                .align_y(Vertical::Center)
                .into()
        };
        let bar = container(super::slots::slots(
            side(start),
            title,
            side(end),
            m.headerbar_height,
            m.space_m,
        ))
        .padding(m.headerbar_padding)
        .width(Length::Fill)
        .style(move |t: &Theme| {
            if flat.unwrap_or(t.flat_headerbars) {
                // Text color only; the background comes from what is behind the bar.
                return iced::widget::container::Style {
                    text_color: Some(t.semantic.headerbar.fg),
                    ..Default::default()
                };
            }
            iced::widget::container::Style {
                border: iced::Border::default()
                    .rounded(iced::border::top_left(corner_l).top_right(corner_r)),
                ..sc::headerbar(t)
            }
        });

        let mut area = mouse_area(bar);
        if let Some(d) = h.on_drag {
            area = area.on_press(d);
        }
        if let Some(d) = h.on_double_click {
            area = area.on_double_click(d);
        }
        area.into()
    }
}

#[allow(dead_code)]
fn _assert_types(_: Padding, _: &Theme, _: text::Text<'static, Theme>) {}
