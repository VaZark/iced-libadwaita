//! `ComboChart`: bars, lines and areas on **one canvas and one scale**. Revenue as bars with a
//! target as a line; counts as an area with a moving average over it.
//!
//! Layers are painted in the order they are added, so add the backdrop first: an area, then
//! bars, then lines. They share the categories and the value axis, and every series keeps its own
//! color, in the order added.

use super::{Cartesian, Formatter, State, assemble, finite, fmt_or_default, track};
use crate::Theme;
use crate::components::chart::path::to_path;
use crate::components::chart::style::{ChartStyle, style_methods};
use crate::components::chart::tooltip::{self, Row};
use crate::components::chart::{axis, ink};
use crate::primitives::chart::Series;
use crate::primitives::chart::scale::Band;
use crate::primitives::chart::shape::{Curve, Pt, area, line};
use crate::primitives::chart::stack::{Offset, extent, stack};
use crate::widget::Element;
use iced::mouse::Cursor;
use iced::widget::canvas::{self, Fill, Frame, Geometry, Path, Stroke};
use iced::{Length, Point, Rectangle, Renderer, Size};
use std::rc::Rc;

/// How a layer is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Bars,
    Line,
    Area,
}

struct Layer {
    kind: Kind,
    series: Series,
}

/// See the [module](self).
pub struct ComboChart {
    categories: Vec<String>,
    layers: Vec<Layer>,
    curve: Curve,
    stack_bars: bool,
    points: bool,
    legend: bool,
    width: Length,
    height: Length,
    format: Option<Formatter>,
    style: ChartStyle,
}

impl ComboChart {
    /// One label per category (the x axis).
    pub fn new(categories: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            categories: categories.into_iter().map(Into::into).collect(),
            layers: Vec::new(),
            curve: Curve::Linear,
            stack_bars: false,
            points: true,
            legend: true,
            width: Length::Fill,
            height: Length::Fixed(280.0),
            format: None,
            style: ChartStyle::default(),
        }
    }
    fn push(mut self, kind: Kind, series: Series) -> Self {
        self.layers.push(Layer { kind, series });
        self
    }
    /// A series drawn as bars. Several bar series sit side by side (or on each other with
    /// [`stack_bars`](Self::stack_bars)).
    pub fn bars(self, series: Series) -> Self {
        self.push(Kind::Bars, series)
    }
    /// A series drawn as a line through the middle of each category.
    pub fn line(self, series: Series) -> Self {
        self.push(Kind::Line, series)
    }
    /// A series drawn as a filled area down to zero.
    pub fn area(self, series: Series) -> Self {
        self.push(Kind::Area, series)
    }
    /// Stacks the bar series on each other instead of side by side.
    pub fn stack_bars(mut self, stacked: bool) -> Self {
        self.stack_bars = stacked;
        self
    }
    /// How lines and areas are joined: straight, stepped or smooth.
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }
    /// Dots on the lines' data points (on by default).
    pub fn points(mut self, points: bool) -> Self {
        self.points = points;
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
    /// The bar layers' indexes and their `(low, high)` per category.
    bars: Vec<(usize, Vec<(f32, f32)>)>,
}

impl ComboChart {
    fn values(&self, layer: &Layer) -> Vec<f32> {
        layer.series.values.iter().copied().map(finite).collect()
    }

    fn geo(&self, size: Size) -> Geo {
        let bar_layers: Vec<usize> = (0..self.layers.len())
            .filter(|i| self.layers[*i].kind == Kind::Bars)
            .collect();
        let bar_values: Vec<Vec<f32>> = bar_layers
            .iter()
            .map(|i| self.values(&self.layers[*i]))
            .collect();
        let bars: Vec<(usize, Vec<(f32, f32)>)> = if self.stack_bars {
            let stacked = stack(&bar_values, Offset::Zero);
            bar_layers.iter().copied().zip(stacked).collect()
        } else {
            bar_layers
                .iter()
                .copied()
                .zip(
                    bar_values
                        .iter()
                        .map(|v| v.iter().map(|x| (0.0f32.min(*x), 0.0f32.max(*x))).collect()),
                )
                .collect()
        };
        // One scale for every layer, always including zero (bars and areas rest on it).
        let (mut lo, mut hi) = extent(&bars.iter().map(|(_, b)| b.clone()).collect::<Vec<_>>());
        for l in self.layers.iter().filter(|l| l.kind != Kind::Bars) {
            for v in self.values(l) {
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        let cart = Cartesian::new(size, (lo, hi), false, &[], &self.format, &self.style);
        let n = self.categories.len().max(
            self.layers
                .iter()
                .map(|l| l.series.values.len())
                .max()
                .unwrap_or(0),
        );
        let band = Band::new(n, (cart.plot.x, cart.plot.x + cart.plot.width), 0.3);
        Geo { cart, band, bars }
    }
}

impl<M> canvas::Program<M, Theme> for ComboChart {
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
        let g = self.geo(bounds.size());
        let mut frame = Frame::new(renderer, bounds.size());
        let plot = g.cart.plot;
        let show = axis::Show {
            grid: self.style.grid,
            labels: self.style.value_axis,
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
        if self.style.category_axis {
            axis::band_axis(&mut frame, theme, plot, &g.band, &self.categories);
        }
        if let (Some(i), true) = (state.hover, self.style.hover) {
            let (text, _) = ink(theme);
            frame.fill_rectangle(
                Point::new(g.band.start(i), plot.y),
                Size::new(g.band.bandwidth(), plot.height),
                crate::color::alpha(text, 0.08),
            );
        }
        let zero = g.cart.scale.map(0.0);
        let centers = |n: usize| -> Vec<f32> { (0..n).map(|i| g.band.center(i)).collect() };

        // Layers in the order they were added.
        let k = g.bars.len().max(1);
        for (idx, layer) in self.layers.iter().enumerate() {
            let col = self.style.color(theme, idx);
            let vals = self.values(layer);
            let n = vals.len().min(g.band.len());
            match layer.kind {
                Kind::Bars => {
                    let slot = g.bars.iter().position(|(l, _)| *l == idx).unwrap_or(0);
                    let ranges = &g.bars[slot].1;
                    for (i, (lo, hi)) in ranges.iter().enumerate().take(n) {
                        let (a, b) = (g.cart.scale.map(*lo), g.cart.scale.map(*hi));
                        let (x, w) = if self.stack_bars {
                            (g.band.start(i), g.band.bandwidth())
                        } else {
                            let sub = Band::new(
                                k,
                                (g.band.start(i), g.band.start(i) + g.band.bandwidth()),
                                0.0,
                            );
                            (sub.start(slot), sub.bandwidth())
                        };
                        let (top, h) = (a.min(b), (a - b).abs());
                        if h > 0.0 {
                            frame.fill(
                                &Path::rounded_rectangle(
                                    Point::new(x, top),
                                    Size::new(w, h),
                                    self.style.radius.into(),
                                ),
                                Fill::from(col),
                            );
                        }
                    }
                }
                Kind::Line | Kind::Area => {
                    let xs = centers(n);
                    let top: Vec<Pt> = (0..n)
                        .map(|i| Pt::new(xs[i], g.cart.scale.map(vals[i])))
                        .collect();
                    if layer.kind == Kind::Area {
                        let base: Vec<Pt> = (0..n).map(|i| Pt::new(xs[i], zero)).collect();
                        frame.fill(
                            &to_path(&area(&top, &base, self.curve)),
                            Fill::from(crate::color::alpha(col, 0.25)),
                        );
                    }
                    frame.stroke(
                        &to_path(&line(&top, self.curve)),
                        Stroke::default()
                            .with_width(2.0)
                            .with_color(col)
                            .with_line_join(canvas::LineJoin::Round),
                    );
                    if self.points && layer.kind == Kind::Line {
                        for p in &top {
                            frame.fill(&Path::circle(Point::new(p.x, p.y), 3.5), col);
                        }
                    }
                }
            }
        }

        if let (Some(i), Some(at), true) =
            (state.hover, cursor.position_in(bounds), self.style.tooltip)
        {
            let rows: Vec<Row> = self
                .layers
                .iter()
                .enumerate()
                .filter_map(|(j, l)| {
                    l.series.values.get(i).map(|v| {
                        Row::new(
                            Some(self.style.color(theme, j)),
                            l.series.name.as_str(),
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

impl<'a, M: 'a> From<ComboChart> for Element<'a, M> {
    fn from(c: ComboChart) -> Self {
        let names = (c.legend && c.layers.len() > 1)
            .then(|| c.layers.iter().map(|l| l.series.name.clone()).collect());
        let (w, h) = (c.width, c.height);
        assemble(c, w, h, names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart() -> ComboChart {
        ComboChart::new(["Q1", "Q2", "Q3"])
            .bars(Series::new("Revenue", [40.0, 55.0, 30.0]))
            .bars(Series::new("Costs", [20.0, 25.0, 22.0]))
            .line(Series::new("Target", [45.0, 60.0, 80.0]))
    }
    const SIZE: Size = Size::new(480.0, 280.0);

    #[test]
    fn every_layer_shares_one_scale() {
        let g = chart().geo(SIZE);
        // The line's 80 is the highest value anywhere: the axis reaches it.
        assert!(g.cart.scale.domain().1 >= 80.0);
        assert!(g.cart.scale.domain().0 <= 0.0);
        assert_eq!(g.bars.len(), 2);
    }

    #[test]
    fn stacked_bars_raise_the_axis_to_the_total() {
        let g = ComboChart::new(["a"])
            .bars(Series::new("x", [60.0]))
            .bars(Series::new("y", [50.0]))
            .stack_bars(true)
            .geo(SIZE);
        assert!(g.cart.scale.domain().1 >= 110.0);
        assert_eq!(g.bars[1].1[0], (60.0, 110.0));
    }

    #[test]
    fn layers_keep_the_order_added() {
        let c = ComboChart::new(["a"])
            .area(Series::new("back", [1.0]))
            .bars(Series::new("mid", [2.0]))
            .line(Series::new("front", [3.0]));
        let kinds: Vec<Kind> = c.layers.iter().map(|l| l.kind).collect();
        assert_eq!(kinds, vec![Kind::Area, Kind::Bars, Kind::Line]);
    }

    #[test]
    fn an_empty_combo_has_a_scale() {
        let g = ComboChart::new(Vec::<String>::new()).geo(SIZE);
        assert!(g.cart.scale.domain().1 > g.cart.scale.domain().0);
    }
}
