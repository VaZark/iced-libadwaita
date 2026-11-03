//! `BarChart`: series over categories as bars, grouped, stacked or to 100%, vertical or
//! horizontal. A [`Histogram`] is a bar chart whose bars touch.

use super::{Cartesian, Formatter, State, assemble, finite, fmt_or_default, track};
use crate::Theme;
use crate::components::chart::tooltip::{self, Row};
use crate::components::chart::{axis, color, ink};
use crate::primitives::chart::Series;
use crate::primitives::chart::scale::{Band, format_tick};
use crate::primitives::chart::stack::{Offset, extent, stack};
use crate::primitives::histogram::Histogram;
use crate::widget::Element;
use iced::mouse::Cursor;
use iced::widget::canvas::{self, Fill, Frame, Geometry, Path};
use iced::{Length, Point, Rectangle, Renderer, Size};
use std::rc::Rc;

/// How the series of one category sit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BarLayout {
    /// Side by side.
    #[default]
    Grouped,
    /// On each other.
    Stacked,
    /// On each other, every category adding up to 100%.
    Normalized,
}

/// See the [module](self).
pub struct BarChart {
    categories: Vec<String>,
    series: Vec<Series>,
    layout: BarLayout,
    horizontal: bool,
    gap: f32,
    legend: bool,
    width: Length,
    height: Length,
    format: Option<Formatter>,
}

impl BarChart {
    /// One label per category and one [`Series`] per bar of a group.
    pub fn new(
        categories: impl IntoIterator<Item = impl Into<String>>,
        series: impl IntoIterator<Item = Series>,
    ) -> Self {
        Self {
            categories: categories.into_iter().map(Into::into).collect(),
            series: series.into_iter().collect(),
            layout: BarLayout::Grouped,
            horizontal: false,
            gap: 0.2,
            legend: true,
            width: Length::Fill,
            height: Length::Fixed(240.0),
            format: None,
        }
    }
    /// A histogram: one series of counts, bars that (almost) touch, labeled by where each bin
    /// starts.
    pub fn histogram(h: &Histogram) -> Self {
        let step = h.bin_width();
        let cats = (0..h.len())
            .map(|i| format_tick(h.bin_range(i).0, step))
            .collect::<Vec<_>>();
        let counts = h.counts().iter().map(|c| *c as f32).collect::<Vec<_>>();
        Self::new(cats, [Series::new("Count", counts)]).gap(0.04)
    }
    /// Grouped, stacked or normalized.
    pub fn layout(mut self, layout: BarLayout) -> Self {
        self.layout = layout;
        self
    }
    /// Bars grow to the right instead of upwards; the categories run down the left.
    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }
    /// The space between categories, as a fraction of a category (`0..=1`; default 0.2).
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.clamp(0.0, 0.95);
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
    /// How values are written on the axis and in the tooltip.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.format = Some(Rc::new(f));
        self
    }
}

struct Geo {
    cart: Cartesian,
    /// The categories, along the category axis.
    band: Band,
    layers: Option<Vec<Vec<(f32, f32)>>>,
}

impl BarChart {
    fn geo(&self, size: Size) -> Geo {
        let values: Vec<Vec<f32>> = self
            .series
            .iter()
            .map(|s| s.values.iter().copied().map(finite).collect())
            .collect();
        let layers = (self.layout != BarLayout::Grouped).then(|| {
            stack(
                &values,
                if self.layout == BarLayout::Normalized {
                    Offset::Expand
                } else {
                    Offset::Zero
                },
            )
        });
        let (lo, hi) = match &layers {
            Some(l) => extent(l),
            None => values
                .iter()
                .flatten()
                .fold((0.0f32, 0.0f32), |(a, b), v| (a.min(*v), b.max(*v))),
        };
        let fmt = if self.layout == BarLayout::Normalized {
            Some(Rc::new(|v: f32| format!("{}%", (v * 100.0).round())) as Formatter)
        } else {
            self.format.clone()
        };
        let cart = Cartesian::new(size, (lo, hi), self.horizontal, &self.categories, &fmt);
        let n = self
            .categories
            .len()
            .max(values.iter().map(Vec::len).max().unwrap_or(0));
        let range = if self.horizontal {
            (cart.plot.y, cart.plot.y + cart.plot.height)
        } else {
            (cart.plot.x, cart.plot.x + cart.plot.width)
        };
        // The gap is between bars; the outer padding is half of it, as in Chart.js.
        let band = Band::new(n, range, self.gap);
        Geo { cart, band, layers }
    }

    /// The bar of series `j` in category `i`, as a rectangle.
    fn bar(&self, g: &Geo, i: usize, j: usize) -> Rectangle {
        let (start, len) = match &g.layers {
            Some(_) => (g.band.start(i), g.band.bandwidth()),
            None => {
                let k = self.series.len().max(1);
                let sub = Band::new(
                    k,
                    (g.band.start(i), g.band.start(i) + g.band.bandwidth()),
                    0.0,
                );
                (sub.start(j), sub.bandwidth())
            }
        };
        let (v0, v1) = match &g.layers {
            Some(l) => l[j][i],
            None => (
                0.0,
                finite(self.series[j].values.get(i).copied().unwrap_or(0.0)),
            ),
        };
        let (a, b) = (g.cart.scale.map(v0), g.cart.scale.map(v1));
        let (lo, hi) = (a.min(b), a.max(b));
        if self.horizontal {
            Rectangle {
                x: lo,
                y: start,
                width: hi - lo,
                height: len,
            }
        } else {
            Rectangle {
                x: start,
                y: lo,
                width: len,
                height: hi - lo,
            }
        }
    }
}

impl<M> canvas::Program<M, Theme> for BarChart {
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
            if !g.cart.plot.contains(p) {
                return None;
            }
            g.band.index_at(if self.horizontal { p.y } else { p.x })
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
        let plot = g.cart.plot;
        if self.horizontal {
            axis::value_axis_bottom(
                &mut frame,
                theme,
                plot,
                &g.cart.scale,
                &g.cart.ticks,
                &g.cart.labels,
            );
            axis::band_axis_left(&mut frame, theme, plot, &g.band, &self.categories);
        } else {
            axis::value_axis(
                &mut frame,
                theme,
                plot,
                &g.cart.scale,
                &g.cart.ticks,
                &g.cart.labels,
            );
            axis::band_axis(&mut frame, theme, plot, &g.band, &self.categories);
        }
        if let Some(i) = state.hover {
            let (text, _) = ink(theme);
            let (p, s) = if self.horizontal {
                (
                    Point::new(plot.x, g.band.start(i)),
                    Size::new(plot.width, g.band.bandwidth()),
                )
            } else {
                (
                    Point::new(g.band.start(i), plot.y),
                    Size::new(g.band.bandwidth(), plot.height),
                )
            };
            frame.fill_rectangle(p, s, crate::color::alpha(text, 0.08));
        }
        for i in 0..g.band.len() {
            for (j, _) in self.series.iter().enumerate() {
                let r = self.bar(&g, i, j);
                if r.width <= 0.0 || r.height <= 0.0 {
                    continue;
                }
                let col = color::series(theme, j);
                frame.fill(
                    &Path::rounded_rectangle(r.position(), r.size(), 2.0.into()),
                    Fill::from(col),
                );
            }
        }
        if let (Some(i), Some(at)) = (state.hover, cursor.position_in(bounds)) {
            let rows: Vec<Row> = self
                .series
                .iter()
                .enumerate()
                .filter_map(|(j, s)| {
                    s.values.get(i).map(|v| {
                        Row::new(
                            Some(color::series(theme, j)),
                            s.name.as_str(),
                            fmt_or_default(&self.format, *v),
                        )
                    })
                })
                .collect();
            tooltip::draw(
                &mut frame,
                theme,
                at,
                self.categories.get(i).map(String::as_str),
                &rows,
            );
        }
        vec![frame.into_geometry()]
    }
}

impl<'a, M: 'a> From<BarChart> for Element<'a, M> {
    fn from(c: BarChart) -> Self {
        let names = (c.legend && c.series.len() > 1)
            .then(|| c.series.iter().map(|s| s.name.clone()).collect());
        let (w, h) = (c.width, c.height);
        assemble(c, w, h, names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart() -> BarChart {
        BarChart::new(
            ["a", "b"],
            [Series::new("x", [4.0, 2.0]), Series::new("y", [1.0, 3.0])],
        )
    }
    const SIZE: Size = Size::new(400.0, 240.0);

    #[test]
    fn grouped_bars_share_a_category() {
        let c = chart();
        let g = c.geo(SIZE);
        let (x, y) = (c.bar(&g, 0, 0), c.bar(&g, 0, 1));
        assert!(x.x + x.width <= y.x + 1e-3);
        assert!(x.height > y.height);
        // They rest on the zero line.
        assert!((x.y + x.height - g.cart.scale.map(0.0)).abs() < 1e-3);
    }

    #[test]
    fn stacked_bars_sit_on_each_other() {
        let c = chart().layout(BarLayout::Stacked);
        let g = c.geo(SIZE);
        let (x, y) = (c.bar(&g, 0, 0), c.bar(&g, 0, 1));
        assert_eq!(x.x, y.x);
        assert!((y.y + y.height - x.y).abs() < 1e-3);
        assert!(g.cart.scale.domain().1 >= 5.0);
    }

    #[test]
    fn horizontal_bars_grow_to_the_right() {
        let c = chart().horizontal(true);
        let g = c.geo(SIZE);
        let r = c.bar(&g, 0, 0);
        assert!((r.x - g.cart.scale.map(0.0)).abs() < 1e-3);
        assert!(r.width > 0.0 && r.height > 0.0);
    }

    #[test]
    fn a_histogram_is_one_series_of_counts() {
        let h = Histogram::from_counts(0.0, 30.0, [3, 5, 2]);
        let c = BarChart::histogram(&h);
        assert_eq!(c.series[0].values, vec![3.0, 5.0, 2.0]);
        assert_eq!(c.categories, vec!["0", "10", "20"]);
        assert!(c.gap < 0.1);
    }
}
