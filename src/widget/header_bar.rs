//! `AdwHeaderBar` and `AdwWindowTitle`.

use super::Element;
use crate::style::container as sc;
use crate::widget::typography;
use crate::{Icon, Metrics, Theme};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{column, container, mouse_area, row, space, text};
use iced::{Length, Padding};

/// Window-control messages shown on the right of a [`HeaderBar`]
/// (`show-end-title-buttons`).
pub struct WindowControls<M> {
    pub on_minimize: Option<M>,
    pub on_maximize: Option<M>,
    pub on_close: Option<M>,
}

impl<M> Default for WindowControls<M> {
    fn default() -> Self {
        Self {
            on_minimize: None,
            on_maximize: None,
            on_close: None,
        }
    }
}

/// `AdwWindowTitle`: centered title with optional subtitle.
pub fn window_title<'a, M: 'a>(title: &str, subtitle: Option<&str>) -> Element<'a, M> {
    let mut c = column![typography::heading(title.to_string())]
        .align_x(Horizontal::Center);
    if let Some(s) = subtitle {
        c = c.push(typography::caption(s.to_string()).style(crate::style::text::dim));
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
    on_drag: Option<M>,
    on_double_click: Option<M>,
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
            on_drag: None,
            on_double_click: None,
            metrics: Metrics::default(),
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

fn ctl<'a, M: Clone + 'a>(icon: Icon, msg: Option<M>) -> Element<'a, M> {
    let mut b = super::circular_button(icon);
    if let Some(m) = msg {
        b = b.on_press(m);
    }
    b.into()
}

impl<'a, M: Clone + 'a> From<HeaderBar<'a, M>> for Element<'a, M> {
    fn from(h: HeaderBar<'a, M>) -> Self {
        let m = h.metrics;
        let mut end = h.end;
        if let Some(c) = h.controls {
            if c.on_minimize.is_some() {
                end.push(ctl(Icon::WindowMinimize, c.on_minimize));
            }
            if c.on_maximize.is_some() {
                end.push(ctl(Icon::WindowMaximize, c.on_maximize));
            }
            if c.on_close.is_some() {
                end.push(ctl(Icon::WindowClose, c.on_close));
            }
        }
        let title: Element<'a, M> = match (h.title_widget, h.title) {
            (Some(w), _) => w,
            (None, Some(t)) => window_title(&t, h.subtitle.as_deref()),
            (None, None) => space().into(),
        };
        let side = |items: Vec<Element<'a, M>>, h: Horizontal| {
            container(row(items).spacing(m.space_s).align_y(Vertical::Center))
                .width(Length::FillPortion(1))
                .align_x(h)
        };
        let bar = container(
            row![
                side(h.start, Horizontal::Left),
                title,
                side(end, Horizontal::Right),
            ]
            .align_y(Vertical::Center)
            .spacing(m.space_m),
        )
        .padding(m.headerbar_padding)
        .center_y(Length::Fixed(m.headerbar_height))
        .width(Length::Fill)
        .style(sc::headerbar);

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
