//! `ComposedComponent`: a histogram and a range slider that edit one range. The page defines a
//! component of its own, `HistogramRangeSlider`, with it, and uses that.

use iced_adaptive_design as iad;

use iad::widget::{self as w, BarChart, ComposedComponent, Histogram, PreferencesGroup};
use iad::{Element, Metrics};
use iced::widget::{column, text};

#[derive(Debug, Clone)]
pub enum Msg {
    Range((f32, f32)),
}

/// A component defined with `ComposedComponent`: a histogram over a range slider, one range.
///
/// Defining one is a struct that holds the data and the callback, and a conversion into an
/// element that joins the parts. `ComposedComponent` owns the rest: building each part from
/// the shared range, routing whichever part changed it through one `on_change`, and applying
/// the rule.
pub struct HistogramRangeSlider<'a, M> {
    histogram: &'a Histogram,
    range: (f32, f32),
    on_change: Box<dyn Fn((f32, f32)) -> M + 'a>,
    full: bool,
}

impl<'a, M> HistogramRangeSlider<'a, M> {
    pub fn new(
        histogram: &'a Histogram,
        range: (f32, f32),
        on_change: impl Fn((f32, f32)) -> M + 'a,
    ) -> Self {
        Self {
            histogram,
            range,
            on_change: Box::new(on_change),
            full: false,
        }
    }
    /// With axes, grid and tooltip, in the theme's series color.
    pub fn full(mut self, full: bool) -> Self {
        self.full = full;
        self
    }
}

impl<'a, M: 'a> From<HistogramRangeSlider<'a, M>> for Element<'a, M> {
    fn from(c: HistogramRangeSlider<'a, M>) -> Self {
        let (h, full) = (c.histogram, c.full);
        ComposedComponent::new(c.range, c.on_change)
            // Part 1: the bars. Those inside the range are highlighted; pressing one picks it.
            .part(move |r| {
                let bars = BarChart::histogram(h)
                    .height(if full { 140.0 } else { 64.0 })
                    .selected(move |i| h.selected(i, *r))
                    .on_pick(move |i| h.bin_range(i));
                // Bare, like a range picker's histogram, or with every element.
                if full { bars } else { bars.minimal().gap(0.07) }.into()
            })
            // Part 2: the thumbs, for the exact ends.
            .part(|r| {
                w::range_slider(0.0..=100.0, *r, |moved| moved)
                    .step(1.0)
                    .into()
            })
            // The rule, once: whichever part sent it, lo <= hi.
            .normalize(|(lo, hi)| (lo.min(hi), lo.max(hi)))
            .spacing(6.0)
            .into()
    }
}

pub struct State {
    histogram: Histogram,
    range: (f32, f32),
}

impl Default for State {
    fn default() -> Self {
        // A price-like skew: many cheap items, a long tail.
        let values = (0..400).map(|i| {
            let x = (i as f32 * 0.618_034).fract();
            100.0 * x * x
        });
        Self {
            histogram: Histogram::from_values(values, 0.0, 100.0, 20),
            range: (20.0, 60.0),
        }
    }
}

impl State {
    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Range(r) => self.range = r,
        }
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let count = self.histogram.count_in(self.range);
        let readout = text(format!(
            "{:.0} to {:.0}: {} of {} values",
            self.range.0,
            self.range.1,
            count,
            self.histogram.counts().iter().sum::<usize>()
        ));
        let bare = HistogramRangeSlider::new(&self.histogram, self.range, Msg::Range);
        let full = HistogramRangeSlider::new(&self.histogram, self.range, Msg::Range).full(true);
        column![
            PreferencesGroup::<Msg>::new()
                .title("Bare")
                .description("Only the bars: accented inside the range, dim outside, a minimum height and a stub for an empty bin. The reference is the histogram range of the weave music app. Press a bar or drag a thumb: both edit the same range.")
                .metrics(m)
                .push(w::card(column![bare, readout].spacing(m.space_m))),
            PreferencesGroup::<Msg>::new()
                .title("Full")
                .description("The same component with every chart element: axes, gridlines, tooltip, hover and the series color. Same range, same state.")
                .metrics(m)
                .push(w::card(full)),
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
