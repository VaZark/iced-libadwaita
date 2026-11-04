//! `Funnel`: stages that narrow, each as wide as its value.

use super::{Formatter, State, assemble, fmt_or_default, on, track};
use crate::Element;
use crate::Theme;
use crate::components::chart::path::to_path;
use crate::components::chart::tooltip::{self, Row};
use crate::components::chart::{Anchor, color, label};
use crate::primitives::chart::funnel::{Shape, Stage, funnel};
use crate::primitives::chart::shape::{Cmd, Pt};
use iced::mouse::Cursor;
use iced::widget::canvas::{self, Fill, Frame, Geometry};
use iced::{Length, Point, Rectangle, Renderer, Size};
use std::rc::Rc;

pub use crate::primitives::chart::funnel::Shape as FunnelShape;

/// See the [module](self).
pub struct Funnel {
    items: Vec<(String, f32)>,
    shape: Shape,
    gap: f32,
    width: Length,
    height: Length,
    format: Option<Formatter>,
    tooltip: bool,
    labels: bool,
}

const INSET: f32 = 8.0;

impl Funnel {
    /// A `(name, value)` for each stage, widest first.
    pub fn new(items: impl IntoIterator<Item = (impl Into<String>, f32)>) -> Self {
        Self {
            items: items.into_iter().map(|(n, v)| (n.into(), v)).collect(),
            shape: Shape::Continuous,
            gap: 2.0,
            width: Length::Fill,
            height: Length::Fixed(260.0),
            format: None,
            tooltip: true,
            labels: true,
        }
    }
    /// One outline that narrows, or blocks as wide as their own value.
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }
    /// The space between stages.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }
    /// The tooltip that follows the pointer (on by default).
    pub fn tooltip(mut self, show: bool) -> Self {
        self.tooltip = show;
        self
    }
    /// The text on the marks (on by default).
    pub fn labels(mut self, show: bool) -> Self {
        self.labels = show;
        self
    }
    /// How values are written in the labels and tooltip.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.format = Some(Rc::new(f));
        self
    }

    /// The stages, positioned in the chart (inset by [`INSET`]).
    fn stages(&self, size: Size) -> Vec<Stage> {
        let values: Vec<f32> = self.items.iter().map(|(_, v)| *v).collect();
        let (w, h) = (
            (size.width - 2.0 * INSET).max(0.0),
            (size.height - 2.0 * INSET).max(0.0),
        );
        funnel(&values, w, h, self.gap, self.shape)
            .into_iter()
            .map(|mut s| {
                s.y0 += INSET;
                s.y1 += INSET;
                s.top = (s.top.0 + INSET, s.top.1 + INSET);
                s.bottom = (s.bottom.0 + INSET, s.bottom.1 + INSET);
                s
            })
            .collect()
    }
}

impl<M> canvas::Program<M, Theme> for Funnel {
    type State = State;

    fn update(
        &self,
        state: &mut State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<canvas::Action<M>> {
        track(&mut state.hover, event, || {
            let p = cursor.position_in(bounds)?;
            self.stages(bounds.size())
                .iter()
                .position(|s| s.contains(Pt::new(p.x, p.y)))
        })
    }

    fn draw(
        &self,
        state: &State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        for s in self.stages(bounds.size()) {
            let [a, b, c, d] = s.corners();
            let mut col = color::series(theme, s.index);
            if state.hover.is_some_and(|h| h != s.index) {
                col = crate::color::alpha(col, 0.45);
            }
            frame.fill(
                &to_path(&[
                    Cmd::Move(a),
                    Cmd::Line(b),
                    Cmd::Line(c),
                    Cmd::Line(d),
                    Cmd::Close,
                ]),
                Fill::from(col),
            );
            let mid = Point::new((bounds.width) / 2.0, (s.y0 + s.y1) / 2.0);
            let narrow = (s.top.1 - s.top.0).min(s.bottom.1 - s.bottom.0);
            let text = format!(
                "{}  {}",
                self.items[s.index].0,
                fmt_or_default(&self.format, s.value)
            );
            if self.labels
                && narrow > crate::components::chart::text_width(&text)
                && s.y1 - s.y0 >= 16.0
            {
                label(
                    &mut frame,
                    text,
                    mid,
                    Anchor::Middle,
                    on(color::series(theme, s.index)),
                );
            }
        }
        if let (Some(i), Some(at), true) = (state.hover, cursor.position_in(bounds), self.tooltip) {
            let stages = self.stages(bounds.size());
            let s = stages[i];
            let rows = [
                Row::new(
                    Some(color::series(theme, i)),
                    "Value",
                    fmt_or_default(&self.format, s.value),
                ),
                Row::new(None, "Of first", format!("{:.1}%", s.of_first * 100.0)),
                Row::new(
                    None,
                    "Of previous",
                    format!("{:.1}%", s.of_previous * 100.0),
                ),
            ];
            tooltip::draw(&mut frame, theme, at, Some(self.items[i].0.as_str()), &rows);
        }
        vec![frame.into_geometry()]
    }
}

impl<'a, M: 'a> From<Funnel> for Element<'a, M> {
    fn from(c: Funnel) -> Self {
        let (w, h) = (c.width, c.height);
        assemble(c, w, h, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_sit_inside_the_inset() {
        let f = Funnel::new([("a", 10.0), ("b", 5.0)]);
        let s = f.stages(Size::new(216.0, 116.0));
        assert_eq!(s[0].top, (INSET, 208.0));
        assert!(s[0].y0 == INSET && s[1].y1 <= 116.0 - INSET + 1e-3);
    }
}
