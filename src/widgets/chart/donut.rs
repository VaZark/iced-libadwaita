//! `DonutChart`: parts of a whole as a ring, with a value in the middle. Add rings for more than
//! one breakdown of the same thing (by region and by product). [`PieChart::donut`](super::PieChart::donut)
//! is a pie with a hole; this is the ring-first chart: a thickness in px, several rings, and a
//! center that says the total and, under the pointer, the slice.

use super::{Formatter, State, assemble, fmt_or_default, on, track};
use crate::Element;
use crate::Theme;
use crate::components::chart::path::to_path;
use crate::components::chart::tooltip::{self, Row};
use crate::components::chart::{Anchor, color, label};
use crate::primitives::chart::pie::{self, Slice, arc, polar};
use crate::primitives::chart::shape::Pt;
use iced::mouse::Cursor;
use iced::widget::canvas::{self, Fill, Frame, Geometry, Stroke};
use iced::{Length, Point, Rectangle, Renderer, Size};
use std::f32::consts::TAU;
use std::rc::Rc;

/// See the [module](self).
pub struct DonutChart {
    rings: Vec<Vec<(String, f32)>>,
    thickness: f32,
    gap: f32,
    pad: f32,
    center: Option<(String, String)>,
    labels: bool,
    legend: bool,
    tooltip: bool,
    width: Length,
    height: Length,
    format: Option<Formatter>,
}

impl DonutChart {
    /// A `(name, value)` for each slice of the first ring, the outermost.
    pub fn new(items: impl IntoIterator<Item = (impl Into<String>, f32)>) -> Self {
        Self {
            rings: vec![collect(items)],
            thickness: 28.0,
            gap: 4.0,
            pad: 0.015,
            center: None,
            labels: true,
            legend: true,
            tooltip: true,
            width: Length::Fill,
            height: Length::Fixed(280.0),
            format: None,
        }
    }
    /// Another ring, inside the last one.
    pub fn ring(mut self, items: impl IntoIterator<Item = (impl Into<String>, f32)>) -> Self {
        self.rings.push(collect(items));
        self
    }
    /// How thick a ring is in px (default 28). Rings are as thin as needed to fit.
    pub fn thickness(mut self, px: f32) -> Self {
        self.thickness = px.max(2.0);
        self
    }
    /// The space between rings in px (default 4).
    pub fn ring_gap(mut self, px: f32) -> Self {
        self.gap = px.max(0.0);
        self
    }
    /// The gap between slices, in radians.
    pub fn pad(mut self, pad: f32) -> Self {
        self.pad = pad.max(0.0);
        self
    }
    /// The text in the middle: a title (large) and a subtitle (small). By default the total of
    /// the first ring and "Total"; under the pointer, the slice's name and share.
    pub fn center(mut self, title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        self.center = Some((title.into(), subtitle.into()));
        self
    }
    /// Percentages on slices big enough to hold them (on by default).
    pub fn labels(mut self, labels: bool) -> Self {
        self.labels = labels;
        self
    }
    /// The legend under the chart (on by default).
    pub fn legend(mut self, legend: bool) -> Self {
        self.legend = legend;
        self
    }
    /// The tooltip that follows the pointer (on by default).
    pub fn tooltip(mut self, show: bool) -> Self {
        self.tooltip = show;
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
    /// How values are written in the middle and the tooltip.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.format = Some(Rc::new(f));
        self
    }
}

fn collect(items: impl IntoIterator<Item = (impl Into<String>, f32)>) -> Vec<(String, f32)> {
    items.into_iter().map(|(n, v)| (n.into(), v)).collect()
}

/// A hover is a ring and a slice in it, in one number.
const STRIDE: usize = 10_000;

struct RingGeo {
    inner: f32,
    outer: f32,
    slices: Vec<Slice>,
}

struct Geo {
    center: Pt,
    rings: Vec<RingGeo>,
}

impl DonutChart {
    fn geo(&self, size: Size) -> Geo {
        let outer = (size.width.min(size.height) / 2.0 - 6.0).max(0.0);
        let n = self.rings.len().max(1) as f32;
        // Rings keep their thickness unless that would push them past the middle.
        let room = (outer - 12.0).max(0.0);
        let t = self
            .thickness
            .min(((room - self.gap * (n - 1.0)) / n).max(2.0));
        let rings = self
            .rings
            .iter()
            .enumerate()
            .map(|(k, items)| {
                let o = outer - k as f32 * (t + self.gap);
                let values: Vec<f32> = items.iter().map(|(_, v)| *v).collect();
                RingGeo {
                    inner: (o - t).max(0.0),
                    outer: o.max(0.0),
                    slices: pie::pie(&values, 0.0, TAU, self.pad),
                }
            })
            .collect();
        Geo {
            center: Pt::new(size.width / 2.0, size.height / 2.0),
            rings,
        }
    }

    fn at(&self, size: Size, p: Pt) -> Option<usize> {
        let g = self.geo(size);
        g.rings.iter().enumerate().find_map(|(k, r)| {
            pie::hit(g.center, r.inner, r.outer, &r.slices, p).map(|s| k * STRIDE + s)
        })
    }

    fn total(&self, ring: usize) -> f32 {
        self.rings[ring].iter().map(|(_, v)| v.max(0.0)).sum()
    }
}

impl<M> canvas::Program<M, Theme> for DonutChart {
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
            self.at(bounds.size(), Pt::new(p.x, p.y))
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
        for (k, ring) in g.rings.iter().enumerate() {
            let total = self.total(k);
            for s in &ring.slices {
                if s.end <= s.start {
                    continue;
                }
                let col = color::series(theme, s.index);
                let hot = state.hover == Some(k * STRIDE + s.index);
                let grow = if hot { 3.0 } else { 0.0 };
                let shape = to_path(&arc(
                    g.center,
                    ring.inner,
                    ring.outer + grow,
                    s.start,
                    s.end,
                ));
                frame.fill(&shape, Fill::from(col));
                frame.stroke(
                    &shape,
                    Stroke::default()
                        .with_width(1.5)
                        .with_color(theme.semantic.view.bg),
                );
                let share = if total > 0.0 { s.value / total } else { 0.0 };
                if self.labels && share >= 0.07 && ring.outer - ring.inner >= 18.0 {
                    let mid = polar(g.center, (ring.inner + ring.outer) / 2.0, s.middle());
                    label(
                        &mut frame,
                        format!("{}%", (share * 100.0).round()),
                        Point::new(mid.x, mid.y),
                        Anchor::Middle,
                        on(col),
                    );
                }
            }
        }

        // The middle: what the pointer is over, else the title given, else the total.
        let hole = g.rings.last().map_or(0.0, |r| r.inner);
        if hole > 28.0 {
            let (title, sub) = match state.hover {
                Some(h) => {
                    let (k, i) = (h / STRIDE, h % STRIDE);
                    let (name, v) = &self.rings[k][i];
                    let total = self.total(k);
                    let share = if total > 0.0 { v / total * 100.0 } else { 0.0 };
                    (
                        fmt_or_default(&self.format, *v),
                        format!("{name}  {share:.0}%"),
                    )
                }
                None => match &self.center {
                    Some((t, s)) => (t.clone(), s.clone()),
                    None => (fmt_or_default(&self.format, self.total(0)), "Total".into()),
                },
            };
            let fg = theme.semantic.window.fg;
            frame.fill_text(canvas::Text {
                content: title,
                position: Point::new(g.center.x, g.center.y - 6.0),
                color: fg,
                size: iced::Pixels((hole * 0.42).clamp(14.0, 28.0)),
                align_x: iced::widget::text::Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
            label(
                &mut frame,
                sub,
                Point::new(g.center.x, g.center.y + (hole * 0.3).clamp(10.0, 20.0)),
                Anchor::Middle,
                theme.semantic.dim(),
            );
        }
        if let (Some(h), Some(at), true) = (state.hover, cursor.position_in(bounds), self.tooltip) {
            let (k, i) = (h / STRIDE, h % STRIDE);
            let (name, v) = &self.rings[k][i];
            let total = self.total(k);
            let rows = [
                Row::new(
                    Some(color::series(theme, i)),
                    name.as_str(),
                    fmt_or_default(&self.format, *v),
                ),
                Row::new(
                    None,
                    "Share",
                    format!("{:.1}%", if total > 0.0 { v / total * 100.0 } else { 0.0 }),
                ),
            ];
            tooltip::draw(&mut frame, theme, at, None, &rows);
        }
        vec![frame.into_geometry()]
    }
}

impl<'a, M: 'a> From<DonutChart> for Element<'a, M> {
    fn from(c: DonutChart) -> Self {
        let names = c
            .legend
            .then(|| c.rings[0].iter().map(|(n, _)| n.clone()).collect());
        let (w, h) = (c.width, c.height);
        assemble(c, w, h, names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart() -> DonutChart {
        DonutChart::new([("a", 1.0), ("b", 3.0)]).ring([("x", 2.0), ("y", 2.0)])
    }
    const SIZE: Size = Size::new(300.0, 300.0);

    #[test]
    fn rings_nest_inwards() {
        let g = chart().geo(SIZE);
        assert_eq!(g.rings.len(), 2);
        assert!(g.rings[1].outer < g.rings[0].inner + 1e-3);
        assert!(g.rings[0].outer - g.rings[0].inner > 20.0);
    }

    #[test]
    fn rings_get_thinner_rather_than_overflow() {
        let g = chart().thickness(500.0).geo(SIZE);
        for r in &g.rings {
            assert!(r.inner >= 0.0 && r.outer >= r.inner);
        }
        assert!(g.rings[1].inner >= 0.0);
    }

    #[test]
    fn the_pointer_finds_its_ring_and_slice() {
        let c = chart();
        let g = c.geo(SIZE);
        let p = Pt::new(
            g.center.x + 3.0,
            g.center.y - (g.rings[0].inner + g.rings[0].outer) / 2.0,
        );
        // Just right of the top, in the outer ring: its first slice.
        assert_eq!(c.at(SIZE, p), Some(0));
        let q = Pt::new(
            g.center.x + 3.0,
            g.center.y - (g.rings[1].inner + g.rings[1].outer) / 2.0,
        );
        assert_eq!(c.at(SIZE, q), Some(STRIDE));
        assert_eq!(c.at(SIZE, g.center), None);
    }

    #[test]
    fn the_total_is_of_a_ring() {
        assert_eq!(chart().total(0), 4.0);
        assert_eq!(chart().total(1), 4.0);
    }
}
