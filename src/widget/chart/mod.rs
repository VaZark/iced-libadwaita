//! Chart widgets: the classics, composed from the headless layouts in
//! [`primitives::chart`](crate::primitives::chart) and the drawing pieces in
//! [`components::chart`](crate::components::chart).
//!
//! | Widget | Shows |
//! | --- | --- |
//! | [`LineChart`] | series over categories: lines, smooth or stepped, filled areas, stacked or 100% |
//! | [`BarChart`] | series over categories: grouped, stacked or 100%, vertical or horizontal; a [`Histogram`](crate::primitives::histogram::Histogram) |
//! | [`PieChart`] | parts of a whole; a donut with `.donut(..)` |
//! | [`Funnel`] | stages that narrow |
//! | [`Sankey`] | flows between nodes |
//! | [`Sunburst`] | a tree as rings |
//! | [`BubbleChart`] | points on two axes with a size (area) for a third value |
//! | [`DonutChart`] | parts of a whole as one or more rings, with the total in the middle |
//! | [`ComboChart`] | bars, lines and areas on one canvas and one scale |
//!
//! Every chart is built from its data and draws itself. The application owns the data; nothing is
//! kept between views except which mark the pointer is over, so a tooltip can follow it.
//! Colors come from the theme ([`color::series`](crate::components::chart::color::series)).
//! Charts take their width from the container (`Length::Fill`) and a fixed height.

mod bar;
mod bubble;
mod combo;
mod donut;
mod funnel;
mod line;
mod pie;
mod sankey;
mod sunburst;

pub use bar::{BarChart, BarLayout};
pub use bubble::{BubbleChart, BubblePoint, BubbleSeries};
pub use combo::ComboChart;
pub use donut::DonutChart;
pub use funnel::{Funnel, FunnelShape};
pub use line::LineChart;
pub use pie::PieChart;
pub use sankey::Sankey;
pub use sunburst::Sunburst;

use crate::components::chart::style::ChartStyle;
use crate::components::chart::{axis, legend::legend};
use crate::primitives::chart::Margin;
use crate::primitives::chart::scale::{Linear, format_tick, ticks};
use crate::widget::Element;
use iced::widget::canvas::{self, Program};
use iced::widget::{Action, column};
use iced::{Length, Rectangle, Size};
use std::rc::Rc;

/// What a chart remembers between frames: the mark the pointer is over.
#[derive(Default)]
pub struct State {
    pub(crate) hover: Option<usize>,
}

/// A function that turns a value into its label.
pub(crate) type Formatter = Rc<dyn Fn(f32) -> String>;

/// A value as short text: whole numbers without a decimal point, otherwise up to two decimals.
pub fn format_value(v: f32) -> String {
    if (v - v.round()).abs() < 1e-4 {
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{v:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

pub(crate) fn fmt_or_default(f: &Option<Formatter>, v: f32) -> String {
    match f {
        Some(f) => f(v),
        None => format_value(v),
    }
}

/// Tracks which mark the pointer is over: sets `state` and asks for a redraw when it changed.
pub(crate) fn track<M>(
    state: &mut Option<usize>,
    event: &canvas::Event,
    now: impl FnOnce() -> Option<usize>,
) -> Option<Action<M>> {
    if !matches!(event, canvas::Event::Mouse(_)) {
        return None;
    }
    let now = now();
    (*state != now).then(|| {
        *state = now;
        Action::request_redraw()
    })
}

/// A canvas of `width` × `height` with an optional legend of `names` under it.
pub(crate) fn assemble<'a, M: 'a, P>(
    program: P,
    width: Length,
    height: Length,
    names: Option<Vec<String>>,
) -> Element<'a, M>
where
    P: Program<M, crate::Theme> + 'a,
{
    let chart = iced::widget::canvas(program).width(width).height(height);
    match names {
        Some(n) => column![chart, legend(n)].spacing(8).width(width).into(),
        None => chart.into(),
    }
}

/// The plot of a chart with one value axis: where the plot is, and the scale and ticks of the value axis.
pub(crate) struct Cartesian {
    pub plot: Rectangle,
    pub scale: Linear,
    pub ticks: Vec<f32>,
    pub labels: Vec<String>,
}

impl Cartesian {
    /// For `domain`, widened to round values. A vertical value axis is on the left; a horizontal
    /// one (`horizontal`) runs along the bottom, and `category_labels` take the left margin.
    pub fn new(
        size: Size,
        domain: (f32, f32),
        horizontal: bool,
        category_labels: &[String],
        fmt: &Option<Formatter>,
        style: &ChartStyle,
    ) -> Self {
        let count = if horizontal {
            size.width / 80.0
        } else {
            size.height / 40.0
        };
        let count = (count as usize).clamp(2, 10);
        let nice = Linear::new(domain, (0.0, 1.0)).nice(count);
        let domain = nice.domain();
        let t = ticks(domain.0, domain.1, count);
        let step = t.windows(2).map(|w| w[1] - w[0]).next().unwrap_or(1.0);
        let labels: Vec<String> = t
            .iter()
            .map(|v| match fmt {
                Some(f) => f(*v),
                None => format_tick(*v, step),
            })
            .collect();
        // Room only for the axes that are drawn: with none, the plot fills the chart.
        let (value, category) = (style.value_axis, style.category_axis);
        let (left, bottom) = if horizontal {
            (
                if category {
                    axis::label_margin(category_labels)
                } else {
                    0.0
                },
                if value { 28.0 } else { 0.0 },
            )
        } else {
            (
                if value {
                    axis::label_margin(&labels)
                } else {
                    0.0
                },
                if category { 28.0 } else { 0.0 },
            )
        };
        let (top, right) = if style.axes() {
            (12.0, 16.0)
        } else {
            (0.0, 0.0)
        };
        let (x, y, w, h) = Margin::new(top, right, bottom, left).inner(size.width, size.height);
        let plot = Rectangle {
            x,
            y,
            width: w,
            height: h,
        };
        let range = if horizontal { (x, x + w) } else { (y + h, y) };
        Self {
            plot,
            scale: Linear::new(domain, range),
            ticks: t,
            labels,
        }
    }
}

/// The ink for text drawn over `fill`.
pub(crate) fn on(fill: iced::Color) -> iced::Color {
    crate::color::readable_on(fill)
}

/// Dropped values: a chart treats a missing or non-finite value as zero.
pub(crate) fn finite(v: f32) -> f32 {
    if v.is_finite() { v } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_print_short() {
        assert_eq!(format_value(3.0), "3");
        assert_eq!(format_value(2.50), "2.5");
        assert_eq!(format_value(0.126), "0.13");
        assert_eq!(format_value(-0.0), "0");
    }

    #[test]
    fn the_plot_leaves_room_for_the_axis() {
        let c = Cartesian::new(
            Size::new(400.0, 200.0),
            (0.0, 97.0),
            false,
            &[],
            &None,
            &ChartStyle::default(),
        );
        assert_eq!(c.scale.domain(), (0.0, 100.0));
        assert!(c.plot.x > 20.0 && c.plot.width < 400.0);
        assert_eq!(c.ticks.len(), c.labels.len());
        // The value 0 is at the bottom of the plot.
        assert_eq!(c.scale.map(0.0), c.plot.y + c.plot.height);
    }

    #[test]
    fn a_bare_chart_fills_its_area() {
        let c = Cartesian::new(
            Size::new(400.0, 200.0),
            (0.0, 10.0),
            false,
            &[],
            &None,
            &ChartStyle::minimal(),
        );
        assert_eq!(
            c.plot,
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 400.0,
                height: 200.0
            }
        );
    }

    #[test]
    fn a_hover_change_asks_for_a_redraw() {
        let mut s = None;
        let ev = canvas::Event::Mouse(iced::mouse::Event::CursorLeft);
        assert!(track::<()>(&mut s, &ev, || Some(2)).is_some());
        assert_eq!(s, Some(2));
        assert!(track::<()>(&mut s, &ev, || Some(2)).is_none());
        assert!(
            track::<()>(
                &mut s,
                &canvas::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(
                    Default::default()
                )),
                || None
            )
            .is_none()
        );
    }
}
