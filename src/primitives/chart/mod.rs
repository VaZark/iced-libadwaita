//! Charts, headless: data in, geometry out. After d3 (scales, shape generators, layouts), Chart.js
//! (series and datasets, one chart type per layout) and TanStack Charts (the chart is state; the
//! drawing is a separate step).
//!
//! Nothing here draws and nothing here knows iced. A layout is a pure function from data and a
//! size to numbers: positions, angles, bands. The drawing pieces are in
//! [`components::chart`](crate::components::chart); the widgets
//! ([`LineChart`](crate::widget::LineChart), [`BarChart`](crate::widget::BarChart), …) compose the two.
//!
//! * [`scale`]: [`Linear`](scale::Linear) and [`Band`](scale::Band) scales, "nice" ticks;
//! * [`shape`]: path commands, lines and areas with a [`Curve`](shape::Curve);
//! * [`stack`]: stacked series (zero baseline or normalized to 100%);
//! * [`pie`]: slices as angles, annular sectors, hit testing;
//! * [`funnel`]: stages as trapezoids;
//! * [`sankey`]: nodes in layers, links as bands between them;
//! * [`hierarchy`]: a tree partitioned into rings (sunburst).

pub mod funnel;
pub mod hierarchy;
pub mod pie;
pub mod sankey;
pub mod scale;
pub mod shape;
pub mod stack;

/// A named run of values, one per category (Chart.js's dataset).
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    pub name: String,
    pub values: Vec<f32>,
}

impl Series {
    pub fn new(name: impl Into<String>, values: impl Into<Vec<f32>>) -> Self {
        Self {
            name: name.into(),
            values: values.into(),
        }
    }
}

/// The space kept free around the plot area for axes and labels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Margin {
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
    /// The plot rectangle `(x, y, width, height)` inside a `width` × `height` area.
    pub fn inner(&self, width: f32, height: f32) -> (f32, f32, f32, f32) {
        (
            self.left,
            self.top,
            (width - self.left - self.right).max(0.0),
            (height - self.top - self.bottom).max(0.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plot_is_what_the_margin_leaves() {
        let m = Margin::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(m.inner(200.0, 100.0), (40.0, 10.0, 140.0, 60.0));
        assert_eq!(m.inner(10.0, 10.0).2, 0.0);
    }
}
