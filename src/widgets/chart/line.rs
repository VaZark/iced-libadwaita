//! `LineChart`: series over categories as lines, smooth or stepped, or filled areas, stacked or
//! to 100%.

use super::{Cartesian, Formatter, State, assemble, finite, fmt_or_default, track};
use crate::Element;
use crate::Theme;
use crate::components::chart::path::to_path;
use crate::components::chart::style::{ChartStyle, style_methods};
use crate::components::chart::tooltip::{self, Row};
use crate::components::chart::{axis, ink};
use crate::primitives::chart::Series;
use crate::primitives::chart::scale::Band;
use crate::primitives::chart::shape::{Curve, Pt, area, line};
use crate::primitives::chart::stack::{Offset, extent, stack};
use iced::mouse::Cursor;
use iced::widget::canvas::{self, Fill, Frame, Geometry, Path, Stroke};
use iced::{Length, Point, Rectangle, Renderer, Size};
use std::rc::Rc;

/// See the [module](self).
pub struct LineChart {
    categories: Vec<String>,
    series: Vec<Series>,
    curve: Curve,
    filled: bool,
    stacked: bool,
    normalized: bool,
    points: Option<bool>,
    legend: bool,
    width: Length,
    height: Length,
    format: Option<Formatter>,
    style: ChartStyle,
}

impl LineChart {
    /// One label per category (the x axis) and one [`Series`] per line.
    pub fn new(
        categories: impl IntoIterator<Item = impl Into<String>>,
        series: impl IntoIterator<Item = Series>,
    ) -> Self {
        Self {
            categories: categories.into_iter().map(Into::into).collect(),
            series: series.into_iter().collect(),
            curve: Curve::Linear,
            filled: false,
            stacked: false,
            normalized: false,
            points: None,
            legend: true,
            width: Length::Fill,
            height: Length::Fixed(240.0),
            format: None,
            style: ChartStyle::default(),
        }
    }
    /// How points are joined: straight, stepped or smooth.
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }
    /// Fills under each line.
    pub fn area(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }
    /// Stacks the series on each other. Stacked series are filled.
    pub fn stacked(mut self, stacked: bool) -> Self {
        self.stacked = stacked;
        self
    }
    /// Stacks the series so every category adds up to 100%.
    pub fn normalized(mut self, normalized: bool) -> Self {
        self.normalized = normalized;
        self.stacked |= normalized;
        self
    }
    /// Dots on the data points. On by default up to 24 categories.
    pub fn points(mut self, points: bool) -> Self {
        self.points = Some(points);
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
    /// How values are written on the axis and in the tooltip.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.format = Some(Rc::new(f));
        self
    }
}

struct Geo {
    cart: Cartesian,
    band: Band,
    layers: Option<Vec<Vec<(f32, f32)>>>,
}

impl LineChart {
    fn geo(&self, size: Size) -> Geo {
        let values: Vec<Vec<f32>> = self
            .series
            .iter()
            .map(|s| s.values.iter().copied().map(finite).collect())
            .collect();
        let layers = self.stacked.then(|| {
            stack(
                &values,
                if self.normalized {
                    Offset::Expand
                } else {
                    Offset::Zero
                },
            )
        });
        let domain = match &layers {
            Some(l) => extent(l),
            None => {
                let all = values.iter().flatten().copied();
                let (lo, hi) = all.fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(v), b.max(v)));
                let (lo, hi) = if lo > hi { (0.0, 1.0) } else { (lo, hi) };
                if self.filled {
                    (lo.min(0.0), hi.max(0.0))
                } else {
                    (lo, hi)
                }
            }
        };
        let fmt = if self.normalized {
            Some(Rc::new(|v: f32| format!("{}%", (v * 100.0).round())) as Formatter)
        } else {
            self.format.clone()
        };
        let cart = Cartesian::new(size, domain, false, &[], &fmt, &self.style);
        let n = self.categories.len().max(
            self.series
                .iter()
                .map(|s| s.values.len())
                .max()
                .unwrap_or(0),
        );
        let band = Band::new(n, (cart.plot.x, cart.plot.x + cart.plot.width), 0.0);
        Geo { cart, band, layers }
    }
}

impl<M> canvas::Program<M, Theme> for LineChart {
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
            g.cart
                .plot
                .contains(p)
                .then(|| g.band.index_at(p.x))
                .flatten()
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
        let c = self;
        let g = c.geo(bounds.size());
        let mut frame = Frame::new(renderer, bounds.size());
        let plot = g.cart.plot;
        let show = axis::Show {
            grid: c.style.grid,
            labels: c.style.value_axis,
        };
        if show.grid || show.labels {
            axis::value_axis(
                &mut frame,
                theme,
                plot,
                &g.cart.scale,
                &g.cart.ticks,
                &g.cart.labels,
                show,
            );
        }
        if c.style.category_axis {
            axis::band_axis(&mut frame, theme, plot, &g.band, &c.categories);
        }

        let zero = g
            .cart
            .scale
            .map(0.0f32.clamp(g.cart.scale.domain().0, g.cart.scale.domain().1));
        let dots = c.points.unwrap_or(g.band.len() <= 24);
        for (j, s) in c.series.iter().enumerate() {
            let col = c.style.color(theme, j);
            let n = g.band.len().min(s.values.len());
            let (top, bottom): (Vec<Pt>, Vec<Pt>) = (0..n)
                .map(|i| {
                    let x = g.band.center(i);
                    match &g.layers {
                        Some(l) => (
                            Pt::new(x, g.cart.scale.map(l[j][i].1)),
                            Pt::new(x, g.cart.scale.map(l[j][i].0)),
                        ),
                        None => (
                            Pt::new(x, g.cart.scale.map(finite(s.values[i]))),
                            Pt::new(x, zero),
                        ),
                    }
                })
                .unzip();
            if c.filled || c.stacked {
                let alpha = if c.stacked { 0.8 } else { 0.18 };
                frame.fill(
                    &to_path(&area(&top, &bottom, c.curve)),
                    Fill::from(crate::color::alpha(col, alpha)),
                );
            }
            frame.stroke(
                &to_path(&line(&top, c.curve)),
                Stroke::default()
                    .with_width(2.0)
                    .with_color(col)
                    .with_line_join(canvas::LineJoin::Round),
            );
            if dots {
                for p in &top {
                    frame.fill(&Path::circle(Point::new(p.x, p.y), 3.0), col);
                }
            }
        }

        if let (Some(i), Some(at), true) = (
            state.hover,
            cursor.position_in(bounds),
            c.style.tooltip || c.style.hover,
        ) {
            let x = g.band.center(i);
            let (text, _) = ink(theme);
            frame.stroke(
                &Path::line(Point::new(x, plot.y), Point::new(x, plot.y + plot.height)),
                Stroke::default()
                    .with_width(1.0)
                    .with_color(crate::color::alpha(text, 0.5)),
            );
            let mut rows = Vec::new();
            for (j, s) in c.series.iter().enumerate() {
                let Some(v) = s.values.get(i) else { continue };
                let col = c.style.color(theme, j);
                let y = match &g.layers {
                    Some(l) => l[j][i].1,
                    None => finite(*v),
                };
                frame.fill(
                    &Path::circle(Point::new(x, g.cart.scale.map(y)), 4.5),
                    theme.semantic.view.bg,
                );
                frame.fill(&Path::circle(Point::new(x, g.cart.scale.map(y)), 3.5), col);
                rows.push(Row::new(
                    Some(col),
                    s.name.as_str(),
                    fmt_or_default(&c.format, *v),
                ));
            }
            let title = c.categories.get(i).map(String::as_str);
            if c.style.tooltip {
                tooltip::draw(&mut frame, theme, at, title, &rows);
            }
        }
        vec![frame.into_geometry()]
    }
}

impl<'a, M: 'a> From<LineChart> for Element<'a, M> {
    fn from(c: LineChart) -> Self {
        let names = (c.legend && c.series.len() > 1)
            .then(|| c.series.iter().map(|s| s.name.clone()).collect());
        let (w, h) = (c.width, c.height);
        assemble(c, w, h, names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart() -> LineChart {
        LineChart::new(
            ["a", "b", "c"],
            [
                Series::new("x", [1.0, 5.0, 3.0]),
                Series::new("y", [2.0, 2.0, 2.0]),
            ],
        )
    }

    #[test]
    fn the_axis_covers_the_data() {
        let g = chart().geo(Size::new(400.0, 200.0));
        let (lo, hi) = g.cart.scale.domain();
        assert!(lo <= 1.0 && hi >= 5.0);
    }

    #[test]
    fn stacking_raises_the_axis_to_the_total() {
        let g = chart().stacked(true).geo(Size::new(400.0, 200.0));
        assert!(g.cart.scale.domain().1 >= 7.0);
        assert!(g.layers.is_some());
    }

    #[test]
    fn a_normalized_chart_runs_to_one() {
        let g = chart().normalized(true).geo(Size::new(400.0, 200.0));
        assert_eq!(g.cart.scale.domain(), (0.0, 1.0));
        assert_eq!(g.cart.labels.last().map(String::as_str), Some("100%"));
    }

    #[test]
    fn an_empty_chart_has_a_domain() {
        let g = LineChart::new(Vec::<String>::new(), []).geo(Size::new(100.0, 100.0));
        assert!(g.cart.scale.domain().1 > g.cart.scale.domain().0);
    }
}
