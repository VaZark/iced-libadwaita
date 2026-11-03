//! `BubbleChart`: points on two axes whose **size** says a third thing. The area of a bubble
//! follows its size.

use super::{Formatter, State, assemble, fmt_or_default, track};
use crate::Theme;
use crate::components::chart::style::{ChartStyle, style_methods};
use crate::components::chart::tooltip::{self, Row};
use crate::components::chart::{Anchor, axis, ink, label};
use crate::primitives::chart::bubble::{self, Bubble, Placed};
use crate::primitives::chart::scale::{Linear, format_tick, nice, ticks};
use crate::primitives::chart::shape::Pt;
use crate::widget::Element;
use iced::mouse::Cursor;
use iced::widget::canvas::{self, Fill, Frame, Geometry, Path, Stroke};
use iced::{Length, Point, Rectangle, Renderer, Size};
use std::rc::Rc;

pub use crate::primitives::chart::bubble::Bubble as BubblePoint;

/// A named run of bubbles: one color.
#[derive(Debug, Clone, PartialEq)]
pub struct BubbleSeries {
    pub name: String,
    pub points: Vec<Bubble>,
}

impl BubbleSeries {
    pub fn new(name: impl Into<String>, points: impl Into<Vec<Bubble>>) -> Self {
        Self {
            name: name.into(),
            points: points.into(),
        }
    }
}

/// See the [module](self).
pub struct BubbleChart {
    series: Vec<BubbleSeries>,
    radii: (f32, f32),
    x_title: Option<String>,
    y_title: Option<String>,
    size_name: String,
    opacity: f32,
    legend: bool,
    width: Length,
    height: Length,
    format: Option<Formatter>,
    style: ChartStyle,
}

impl BubbleChart {
    pub fn new(series: impl IntoIterator<Item = BubbleSeries>) -> Self {
        Self {
            series: series.into_iter().collect(),
            radii: (5.0, 26.0),
            x_title: None,
            y_title: None,
            size_name: "Size".into(),
            opacity: 0.55,
            legend: true,
            width: Length::Fill,
            height: Length::Fixed(320.0),
            format: None,
            style: ChartStyle::default(),
        }
    }
    /// The smallest and largest radius in px: the smallest bubble and the largest (default 5 and 26).
    pub fn radii(mut self, min: f32, max: f32) -> Self {
        let (a, b) = (min.max(1.0), max.max(1.0));
        self.radii = (a.min(b), a.max(b));
        self
    }
    /// A title under the x axis.
    pub fn x_title(mut self, t: impl Into<String>) -> Self {
        self.x_title = Some(t.into());
        self
    }
    /// A title above the y axis.
    pub fn y_title(mut self, t: impl Into<String>) -> Self {
        self.y_title = Some(t.into());
        self
    }
    /// What the size is called in the tooltip (default "Size").
    pub fn size_name(mut self, name: impl Into<String>) -> Self {
        self.size_name = name.into();
        self
    }
    /// The fill opacity of the bubbles, so overlaps show (default 0.55).
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
    /// The legend under the chart (on by default; hidden for a single series).
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
    style_methods!();
    /// How values are written on the axes and in the tooltip.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.format = Some(Rc::new(f));
        self
    }
}

struct Geo {
    plot: Rectangle,
    x: Linear,
    y: Linear,
    x_ticks: Vec<f32>,
    y_ticks: Vec<f32>,
    x_labels: Vec<String>,
    y_labels: Vec<String>,
    placed: Vec<Placed>,
}

impl BubbleChart {
    fn bubbles(&self) -> Vec<Vec<Bubble>> {
        self.series.iter().map(|s| s.points.clone()).collect()
    }

    fn geo(&self, size: Size) -> Geo {
        let data = self.bubbles();
        let ((x0, x1), (y0, y1)) = bubble::extent(&data).unwrap_or(((0.0, 1.0), (0.0, 1.0)));
        // Leave room for the bubbles at the edges: a tenth of the span on each side.
        let pad = |a: f32, b: f32| {
            let d = (b - a).max(f32::EPSILON) * 0.1;
            (a - d, b + d)
        };
        let (xd, yd) = (pad(x0, x1), pad(y0, y1));
        let (xd, yd) = (nice(xd.0, xd.1, 6), nice(yd.0, yd.1, 5));
        let (x_ticks, y_ticks) = (ticks(xd.0, xd.1, 6), ticks(yd.0, yd.1, 5));
        let label_of = |ts: &[f32]| -> Vec<String> {
            let step = ts.windows(2).map(|w| w[1] - w[0]).next().unwrap_or(1.0);
            ts.iter()
                .map(|v| match &self.format {
                    Some(f) => f(*v),
                    None => format_tick(*v, step),
                })
                .collect()
        };
        let (x_labels, y_labels) = (label_of(&x_ticks), label_of(&y_ticks));
        let (value, category) = (self.style.value_axis, self.style.category_axis);
        let left = if value {
            axis::label_margin(&y_labels)
        } else {
            0.0
        };
        let top = if self.y_title.is_some() {
            24.0
        } else if self.style.axes() {
            12.0
        } else {
            0.0
        };
        let bottom =
            (if category { 28.0 } else { 0.0 }) + if self.x_title.is_some() { 18.0 } else { 0.0 };
        let right = if self.style.axes() { 16.0 } else { 0.0 };
        let plot = Rectangle {
            x: left,
            y: top,
            width: (size.width - left - right).max(0.0),
            height: (size.height - top - bottom).max(0.0),
        };
        let x = Linear::new(xd, (plot.x, plot.x + plot.width));
        let y = Linear::new(yd, (plot.y + plot.height, plot.y));
        let mut placed = bubble::place(&data, &x, &y, self.radii);
        // Big ones first, so a small bubble is never hidden under a large one.
        placed.sort_by(|a, b| b.r.total_cmp(&a.r));
        Geo {
            plot,
            x,
            y,
            x_ticks,
            y_ticks,
            x_labels,
            y_labels,
            placed,
        }
    }
}

impl<M> canvas::Program<M, Theme> for BubbleChart {
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
            bubble::hit(&g.placed, Pt::new(p.x, p.y))
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
        let (text, _) = ink(theme);
        let show = axis::Show {
            grid: self.style.grid,
            labels: self.style.value_axis,
        };
        if show.grid || show.labels {
            axis::value_axis(
                &mut frame,
                theme,
                g.plot,
                &g.y,
                &g.y_ticks,
                &g.y_labels,
                show,
            );
        }
        if self.style.category_axis || self.style.grid {
            axis::value_axis_bottom(
                &mut frame,
                theme,
                g.plot,
                &g.x,
                &g.x_ticks,
                &g.x_labels,
                axis::Show {
                    grid: self.style.grid,
                    labels: self.style.category_axis,
                },
            );
        }
        if let Some(t) = &self.x_title {
            label(
                &mut frame,
                t.as_str(),
                Point::new(g.plot.x + g.plot.width / 2.0, bounds.height - 8.0),
                Anchor::Middle,
                text,
            );
        }
        if let Some(t) = &self.y_title {
            label(
                &mut frame,
                t.as_str(),
                Point::new(g.plot.x, 10.0),
                Anchor::Start,
                text,
            );
        }
        for (i, b) in g.placed.iter().enumerate() {
            let col = self.style.color(theme, b.series);
            let c = Path::circle(Point::new(b.cx, b.cy), b.r);
            frame.fill(&c, Fill::from(crate::color::alpha(col, self.opacity)));
            let hot = state.hover == Some(i) && self.style.hover;
            frame.stroke(
                &c,
                Stroke::default()
                    .with_width(if hot { 3.0 } else { 1.5 })
                    .with_color(col),
            );
        }
        let hovered = match (state.hover, cursor.position_in(bounds)) {
            (Some(i), Some(at)) if self.style.tooltip => g.placed.get(i).map(|b| (b, at)),
            _ => None,
        };
        if let Some((b, at)) = hovered {
            let s = &self.series[b.series];
            let p = s.points[b.index];
            let rows = [
                Row::new(
                    Some(self.style.color(theme, b.series)),
                    "x",
                    fmt_or_default(&self.format, p.x),
                ),
                Row::new(None, "y", fmt_or_default(&self.format, p.y)),
                Row::new(
                    None,
                    self.size_name.as_str(),
                    fmt_or_default(&self.format, p.size),
                ),
            ];
            tooltip::draw(&mut frame, theme, at, Some(s.name.as_str()), &rows);
        }
        vec![frame.into_geometry()]
    }
}

impl<'a, M: 'a> From<BubbleChart> for Element<'a, M> {
    fn from(c: BubbleChart) -> Self {
        let names = (c.legend && c.series.len() > 1)
            .then(|| c.series.iter().map(|s| s.name.clone()).collect());
        let (w, h) = (c.width, c.height);
        assemble(c, w, h, names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart() -> BubbleChart {
        BubbleChart::new([
            BubbleSeries::new(
                "EU",
                [Bubble::new(10.0, 20.0, 5.0), Bubble::new(30.0, 60.0, 50.0)],
            ),
            BubbleSeries::new("US", [Bubble::new(20.0, 40.0, 25.0)]),
        ])
    }
    const SIZE: Size = Size::new(500.0, 320.0);

    #[test]
    fn every_bubble_is_placed_inside_the_plot() {
        let g = chart().geo(SIZE);
        assert_eq!(g.placed.len(), 3);
        for b in &g.placed {
            assert!(b.cx >= g.plot.x && b.cx <= g.plot.x + g.plot.width);
            assert!(b.cy >= g.plot.y && b.cy <= g.plot.y + g.plot.height);
        }
    }

    #[test]
    fn the_biggest_bubble_is_drawn_first() {
        let g = chart().geo(SIZE);
        assert!(g.placed.windows(2).all(|w| w[0].r >= w[1].r));
        assert_eq!(g.placed[0].r, 26.0);
        assert_eq!(g.placed.last().unwrap().r, 5.0);
    }

    #[test]
    fn radii_are_ordered_and_an_empty_chart_has_a_plot() {
        assert_eq!(chart().radii(30.0, 10.0).radii, (10.0, 30.0));
        let g = BubbleChart::new([]).geo(SIZE);
        assert!(g.placed.is_empty() && g.plot.width > 0.0);
    }

    #[test]
    fn titles_take_room() {
        let plain = chart().geo(SIZE).plot;
        let titled = chart().x_title("Spend").y_title("Sales").geo(SIZE).plot;
        assert!(titled.height < plain.height && titled.y > plain.y);
    }
}
