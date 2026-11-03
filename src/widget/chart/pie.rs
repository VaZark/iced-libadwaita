//! `PieChart`: parts of a whole as slices; a donut with a hole (and a text in it).

use super::{Formatter, State, assemble, fmt_or_default, on, track};
use crate::Theme;
use crate::components::chart::path::to_path;
use crate::components::chart::tooltip::{self, Row};
use crate::components::chart::{Anchor, color, label};
use crate::primitives::chart::pie::{self, Slice, arc, polar};
use crate::primitives::chart::shape::Pt;
use crate::widget::Element;
use iced::mouse::Cursor;
use iced::widget::canvas::{self, Fill, Frame, Geometry, Stroke};
use iced::{Length, Point, Rectangle, Renderer, Size};
use std::f32::consts::TAU;
use std::rc::Rc;

/// See the [module](self).
pub struct PieChart {
    items: Vec<(String, f32)>,
    inner: f32,
    pad: f32,
    labels: bool,
    center: Option<String>,
    legend: bool,
    width: Length,
    height: Length,
    format: Option<Formatter>,
    tooltip: bool,
}

impl PieChart {
    /// A `(name, value)` for each slice.
    pub fn new(items: impl IntoIterator<Item = (impl Into<String>, f32)>) -> Self {
        Self {
            items: items.into_iter().map(|(n, v)| (n.into(), v)).collect(),
            inner: 0.0,
            pad: 0.01,
            labels: true,
            center: None,
            legend: true,
            width: Length::Fill,
            height: Length::Fixed(260.0),
            format: None,
            tooltip: true,
        }
    }
    /// A donut: the hole is this fraction of the radius (`0..1`).
    pub fn donut(mut self, hole: f32) -> Self {
        self.inner = hole.clamp(0.0, 0.95);
        self
    }
    /// The gap between slices, in radians.
    pub fn pad(mut self, pad: f32) -> Self {
        self.pad = pad.max(0.0);
        self
    }
    /// Percentages on the slices big enough to hold them (on by default).
    pub fn labels(mut self, labels: bool) -> Self {
        self.labels = labels;
        self
    }
    /// Text in the middle of a donut.
    pub fn center(mut self, text: impl Into<String>) -> Self {
        self.center = Some(text.into());
        self
    }
    /// The legend under the chart (on by default).
    pub fn legend(mut self, legend: bool) -> Self {
        self.legend = legend;
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
    /// How values are written in the tooltip.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.format = Some(Rc::new(f));
        self
    }
}

struct Geo {
    center: Pt,
    inner: f32,
    outer: f32,
    slices: Vec<Slice>,
}

impl PieChart {
    fn geo(&self, size: Size) -> Geo {
        let outer = (size.width.min(size.height) / 2.0 - 8.0).max(0.0);
        let values: Vec<f32> = self.items.iter().map(|(_, v)| *v).collect();
        Geo {
            center: Pt::new(size.width / 2.0, size.height / 2.0),
            inner: outer * self.inner,
            outer,
            slices: pie::pie(&values, 0.0, TAU, self.pad),
        }
    }
}

impl<M> canvas::Program<M, Theme> for PieChart {
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
            let g = self.geo(bounds.size());
            pie::hit(g.center, g.inner, g.outer, &g.slices, Pt::new(p.x, p.y))
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
        let g = self.geo(bounds.size());
        let mut frame = Frame::new(renderer, bounds.size());
        let total: f32 = g.slices.iter().map(|s| s.value).sum();
        for s in &g.slices {
            if s.end <= s.start {
                continue;
            }
            let col = color::series(theme, s.index);
            let grow = if state.hover == Some(s.index) {
                4.0
            } else {
                0.0
            };
            let shape = to_path(&arc(g.center, g.inner, g.outer + grow, s.start, s.end));
            frame.fill(&shape, Fill::from(col));
            frame.stroke(
                &shape,
                Stroke::default()
                    .with_width(1.5)
                    .with_color(theme.semantic.view.bg),
            );
            let share = s.value / total;
            if self.labels && share >= 0.06 {
                let mid = polar(g.center, (g.inner + g.outer) / 2.0, s.middle());
                label(
                    &mut frame,
                    format!("{}%", (share * 100.0).round()),
                    Point::new(mid.x, mid.y),
                    Anchor::Middle,
                    on(col),
                );
            }
        }
        if let (Some(text), true) = (&self.center, g.inner > 0.0) {
            label(
                &mut frame,
                text.as_str(),
                Point::new(g.center.x, g.center.y),
                Anchor::Middle,
                theme.semantic.window.fg,
            );
        }
        if let (Some(i), Some(at), true) = (state.hover, cursor.position_in(bounds), self.tooltip) {
            let s = g.slices[i];
            let rows = [
                Row::new(
                    Some(color::series(theme, i)),
                    self.items[i].0.as_str(),
                    fmt_or_default(&self.format, s.value),
                ),
                Row::new(None, "Share", format!("{:.1}%", s.value / total * 100.0)),
            ];
            tooltip::draw(&mut frame, theme, at, None, &rows);
        }
        vec![frame.into_geometry()]
    }
}

impl<'a, M: 'a> From<PieChart> for Element<'a, M> {
    fn from(c: PieChart) -> Self {
        let names = c
            .legend
            .then(|| c.items.iter().map(|(n, _)| n.clone()).collect());
        let (w, h) = (c.width, c.height);
        assemble(c, w, h, names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_donut_hole_is_a_fraction_of_the_radius() {
        let c = PieChart::new([("a", 1.0), ("b", 3.0)]).donut(0.5);
        let g = c.geo(Size::new(216.0, 216.0));
        assert_eq!(g.outer, 100.0);
        assert_eq!(g.inner, 50.0);
        assert_eq!(g.slices.len(), 2);
    }

    #[test]
    fn a_small_area_does_not_go_negative() {
        let g = PieChart::new([("a", 1.0)]).geo(Size::new(4.0, 4.0));
        assert_eq!(g.outer, 0.0);
    }
}
