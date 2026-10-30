//! `AdwToolbarView` and `GtkActionBar`.

use super::Element;
use crate::style::container as sc;
use crate::{Metrics, Theme};
use iced::alignment::Vertical;
use iced::gradient::Linear;
use iced::widget::{column, container, row, space, stack};
use iced::{Background, Color, Length, Radians};

/// `AdwToolbarView:top-bar-style` / `bottom-bar-style`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarStyle {
    /// No shadow (the Adwaita default; libadwaita draws a scroll undershoot instead).
    #[default]
    Flat,
    /// Persistent shadow: `0 1px` and `0 2px 4px` of `--headerbar-shade-color` at 50%.
    Raised,
    /// A 1px line in `--headerbar-darker-shade-color`.
    RaisedBorder,
}

/// The shadow cast by a raised bar onto the content, drawn as a fading strip
/// (iced draws one shadow per quad and later siblings would paint over it).
fn shade_strip<'a, M: 'a>(top: bool) -> Element<'a, M> {
    container(space())
        .width(Length::Fill)
        .height(Length::Fixed(6.0))
        .style(move |t: &Theme| {
            let s = &t.semantic;
            let k = if t.shadows.enabled { t.shadows.intensity } else { 0.0 };
            let c = Color { a: (s.headerbar_shade.a * 0.5 * k).min(1.0), ..s.headerbar_shade };
            let clear = Color { a: 0.0, ..c };
            // iced angles: 0 = left→right; π/2 = top→bottom.
            let (from, to) = if top { (c, clear) } else { (clear, c) };
            let g = Linear::new(Radians(std::f32::consts::FRAC_PI_2)).add_stop(0.0, from).add_stop(1.0, to);
            iced::widget::container::Style {
                background: Some(Background::Gradient(g.into())),
                ..Default::default()
            }
        })
        .into()
}

fn border_line<'a, M: 'a>() -> Element<'a, M> {
    container(space().height(1))
        .width(Length::Fill)
        .style(|t: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(t.semantic.headerbar_darker_shade)),
            ..Default::default()
        })
        .into()
}

/// `AdwToolbarView`: top bars, content, bottom bars.
pub struct ToolbarView<'a, M> {
    top: Vec<Element<'a, M>>,
    bottom: Vec<Element<'a, M>>,
    content: Element<'a, M>,
    top_style: BarStyle,
    bottom_style: BarStyle,
}

pub fn toolbar_view<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> ToolbarView<'a, M> {
    ToolbarView::new(content)
}

impl<'a, M: 'a> ToolbarView<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            top: Vec::new(),
            bottom: Vec::new(),
            content: content.into(),
            top_style: BarStyle::Flat,
            bottom_style: BarStyle::Flat,
        }
    }
    /// `add_top_bar`.
    pub fn top_bar(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.top.push(e.into());
        self
    }
    /// `add_bottom_bar`.
    pub fn bottom_bar(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.bottom.push(e.into());
        self
    }
    /// `top-bar-style`.
    pub fn top_bar_style(mut self, s: BarStyle) -> Self {
        self.top_style = s;
        self
    }
    /// `bottom-bar-style`.
    pub fn bottom_bar_style(mut self, s: BarStyle) -> Self {
        self.bottom_style = s;
        self
    }
}

impl<'a, M: 'a> From<ToolbarView<'a, M>> for Element<'a, M> {
    fn from(t: ToolbarView<'a, M>) -> Self {
        let mut col = column![];
        let has_top = !t.top.is_empty();
        let has_bottom = !t.bottom.is_empty();
        for b in t.top {
            col = col.push(b);
        }
        if has_top && t.top_style == BarStyle::RaisedBorder {
            col = col.push(border_line());
        }

        let mut layers = vec![container(t.content).width(Length::Fill).height(Length::Fill).into()];
        if has_top && t.top_style == BarStyle::Raised {
            layers.push(container(shade_strip(true)).align_top(Length::Fill).into());
        }
        if has_bottom && t.bottom_style == BarStyle::Raised {
            layers.push(container(shade_strip(false)).align_bottom(Length::Fill).into());
        }
        col = col.push(stack(layers).width(Length::Fill).height(Length::Fill));

        if has_bottom && t.bottom_style == BarStyle::RaisedBorder {
            col = col.push(border_line());
        }
        for b in t.bottom {
            col = col.push(b);
        }
        col.width(Length::Fill).height(Length::Fill).into()
    }
}

/// `GtkActionBar`.
pub struct ActionBar<'a, M> {
    start: Vec<Element<'a, M>>,
    center: Option<Element<'a, M>>,
    end: Vec<Element<'a, M>>,
    metrics: Metrics,
}

pub fn action_bar<'a, M>() -> ActionBar<'a, M> {
    ActionBar::new()
}

impl<'a, M> ActionBar<'a, M> {
    pub fn new() -> Self {
        Self {
            start: Vec::new(),
            center: None,
            end: Vec::new(),
            metrics: Metrics::default(),
        }
    }
    pub fn pack_start(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.start.push(e.into());
        self
    }
    pub fn pack_end(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.end.insert(0, e.into());
        self
    }
    pub fn center(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.center = Some(e.into());
        self
    }
}

impl<'a, M: 'a> Default for ActionBar<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, M: 'a> From<ActionBar<'a, M>> for Element<'a, M> {
    fn from(a: ActionBar<'a, M>) -> Self {
        let m = a.metrics;
        let side = |v: Vec<Element<'a, M>>, h| {
            container(row(v).spacing(m.space_s).align_y(Vertical::Center))
                .width(Length::FillPortion(1))
                .align_x(h)
        };
        container(
            row![
                side(a.start, iced::alignment::Horizontal::Left),
                a.center.unwrap_or_else(|| space().into()),
                side(a.end, iced::alignment::Horizontal::Right),
            ]
            .align_y(Vertical::Center)
            .spacing(m.space_m),
        )
        .padding(m.space_s)
        .width(Length::Fill)
        .style(sc::headerbar)
        .into()
    }
}
