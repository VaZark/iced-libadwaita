//! `BarChart`: series over categories as bars, grouped, stacked or to 100%, vertical or
//! horizontal. A [`Histogram`] is a bar chart whose bars touch.

use super::{Cartesian, Formatter, State, assemble, finite, fmt_or_default, track};
use crate::Theme;
use crate::components::chart::style::{ChartStyle, style_methods};
use crate::components::chart::tooltip::{self, Row};
use crate::components::chart::{axis, ink};
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
pub struct BarChart<'a, M> {
    categories: Vec<String>,
    series: Vec<Series>,
    layout: BarLayout,
    horizontal: bool,
    gap: f32,
    legend: bool,
    width: Length,
    height: Length,
    format: Option<Formatter>,
    selected: Option<Vec<bool>>,
    style: ChartStyle,
    on_pick: Option<Box<dyn Fn(usize) -> M + 'a>>,
}

impl<'a, M> BarChart<'a, M> {
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
            selected: None,
            style: ChartStyle::default(),
            on_pick: None,
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
    /// Highlights the categories `i` for which `is_selected(i)` holds and dims the rest. For a
    /// histogram with a chosen range: `.selected(|i| h.selected(i, range))`.
    pub fn selected(mut self, is_selected: impl Fn(usize) -> bool) -> Self {
        let n = self.categories.len();
        self.selected = Some((0..n).map(is_selected).collect());
        self
    }
    /// Makes the bars pressable: `on_pick` gets the index of the pressed category.
    pub fn on_pick(mut self, on_pick: impl Fn(usize) -> M + 'a) -> Self {
        self.on_pick = Some(Box::new(on_pick));
        self
    }
    /// Draws only the bars: no axes, grid, tooltip or hover highlight, accented, the bars outside
    /// a [`selected`](Self::selected) range in a dim text color, a minimum bar height and a stub
    /// for an empty bin. [`BarChart::histogram`] plus this is a range picker's histogram. Any
    /// element can be switched back on after it.
    pub fn minimal(mut self) -> Self {
        self.style = ChartStyle::minimal();
        self
    }
    /// The color of the bars outside a selection, in place of the bar's color at
    /// [`dim`](Self::dim) opacity.
    pub fn unselected_color(mut self, paint: impl Fn(&Theme) -> iced::Color + 'static) -> Self {
        self.style.unselected = Some(Rc::new(paint));
        self
    }
    /// The opacity of bars outside a selection (default 0.3), when there is no
    /// [`unselected_color`](Self::unselected_color).
    pub fn dim(mut self, opacity: f32) -> Self {
        self.style.dim = opacity.clamp(0.0, 1.0);
        self
    }
    /// The corner radius of the bars (default 2).
    pub fn bar_radius(mut self, radius: f32) -> Self {
        self.style.radius = radius.max(0.0);
        self
    }
    /// A bar with a value is at least this tall in px, so small values stay visible.
    pub fn min_bar_height(mut self, px: f32) -> Self {
        self.style.min_bar = px.max(0.0);
        self
    }
    /// A bar with no value is this tall in px (a stub; 0 draws nothing).
    pub fn empty_bar_height(mut self, px: f32) -> Self {
        self.style.empty_bar = px.max(0.0);
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
    /// The categories, along the category axis.
    band: Band,
    layers: Option<Vec<Vec<(f32, f32)>>>,
}

impl<'a, M> BarChart<'a, M> {
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
        let cart = Cartesian::new(
            size,
            (lo, hi),
            self.horizontal,
            &self.categories,
            &fmt,
            &self.style,
        );
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
        // Small values stay visible and empty ones can keep a stub, growing from the baseline
        // in the direction the value points.
        let want = if v1 == v0 {
            self.style.empty_bar
        } else {
            self.style.min_bar
        };
        let dir = if self.horizontal { 1.0 } else { -1.0 } * if v1 >= v0 { 1.0 } else { -1.0 };
        let b = if (b - a).abs() < want {
            a + dir * want
        } else {
            b
        };
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

impl<'a, M> canvas::Program<M, Theme> for BarChart<'a, M> {
    type State = State;

    fn update(
        &self,
        state: &mut State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<canvas::Action<M>> {
        if let (
            Some(pick),
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
            Some(i),
        ) = (&self.on_pick, event, state.hover)
        {
            return Some(canvas::Action::publish(pick(i)).and_capture());
        }
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
        let show = axis::Show {
            grid: self.style.grid,
            labels: self.style.value_axis,
        };
        if self.horizontal {
            if show.grid || show.labels {
                axis::value_axis_bottom(
                    &mut frame,
                    theme,
                    plot,
                    &g.cart.scale,
                    &g.cart.ticks,
                    &g.cart.labels,
                    show,
                );
            }
            if self.style.category_axis {
                axis::band_axis_left(&mut frame, theme, plot, &g.band, &self.categories);
            }
        } else {
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
            if self.style.category_axis {
                axis::band_axis(&mut frame, theme, plot, &g.band, &self.categories);
            }
        }
        if let (Some(i), true) = (state.hover, self.style.hover) {
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
                let col = if self.selected.as_ref().is_some_and(|s| !s[i]) {
                    self.style.dimmed(theme, j)
                } else {
                    self.style.color(theme, j)
                };
                frame.fill(
                    &Path::rounded_rectangle(r.position(), r.size(), self.style.radius.into()),
                    Fill::from(col),
                );
            }
        }
        if let (Some(i), Some(at), true) =
            (state.hover, cursor.position_in(bounds), self.style.tooltip)
        {
            let rows: Vec<Row> = self
                .series
                .iter()
                .enumerate()
                .filter_map(|(j, s)| {
                    s.values.get(i).map(|v| {
                        Row::new(
                            Some(self.style.color(theme, j)),
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

impl<'a, M: 'a> From<BarChart<'a, M>> for Element<'a, M> {
    fn from(c: BarChart<'a, M>) -> Self {
        let names = (c.legend && c.series.len() > 1)
            .then(|| c.series.iter().map(|s| s.name.clone()).collect());
        let (w, h) = (c.width, c.height);
        assemble(c, w, h, names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart() -> BarChart<'static, ()> {
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
    fn small_values_keep_a_minimum_and_empty_ones_a_stub() {
        let mut c: BarChart<'_, ()> =
            BarChart::new(["a", "b", "c"], [Series::new("x", [100.0, 0.5, 0.0])])
                .min_bar_height(4.0)
                .empty_bar_height(2.0);
        let g = c.geo(SIZE);
        let base = g.cart.scale.map(0.0);
        let (tall, small, empty) = (c.bar(&g, 0, 0), c.bar(&g, 1, 0), c.bar(&g, 2, 0));
        assert!(tall.height > 50.0);
        assert_eq!(small.height, 4.0);
        assert_eq!(empty.height, 2.0);
        // They all grow upwards from the baseline.
        assert!((small.y + small.height - base).abs() < 1e-3);
        assert!((empty.y + empty.height - base).abs() < 1e-3);
        // Without the options a zero is nothing.
        c.style = ChartStyle::default();
        assert_eq!(c.bar(&c.geo(SIZE), 2, 0).height, 0.0);
    }

    #[test]
    fn every_element_can_be_switched_off() {
        let c = chart().axes(false).grid(false).tooltip(false).hover(false);
        assert!(!c.style.axes() && !c.style.grid && !c.style.tooltip && !c.style.hover);
        let g = c.geo(SIZE);
        assert_eq!(g.cart.plot.width, 400.0);
        assert_eq!(g.cart.plot.height, 240.0);
        let c = chart().axes(false).value_axis(true);
        assert!(c.style.value_axis && !c.style.category_axis);
    }

    #[test]
    fn minimal_is_the_bare_style() {
        let c = chart().minimal();
        assert!(!c.style.axes() && c.style.min_bar > 0.0);
        // Elements come back one at a time.
        assert!(c.tooltip(true).style.tooltip);
    }

    #[test]
    fn selected_bins_are_remembered() {
        let c = chart().selected(|i| i == 1);
        assert_eq!(c.selected, Some(vec![false, true]));
    }

    #[test]
    fn a_histogram_is_one_series_of_counts() {
        let h = Histogram::from_counts(0.0, 30.0, [3, 5, 2]);
        let c: BarChart<'_, ()> = BarChart::histogram(&h);
        assert_eq!(c.series[0].values, vec![3.0, 5.0, 2.0]);
        assert_eq!(c.categories, vec!["0", "10", "20"]);
        assert!(c.gap < 0.1);
    }
}
