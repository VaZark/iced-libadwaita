//! `ComposedComponent`: a histogram and a range slider that edit one range. The library's
//! `HistogramRangeSlider` is one; the Code tab shows how it is defined.

use iced_adaptive_design as iad;

use iad::widget::{self as w, Histogram, HistogramRangeSlider, PreferencesGroup};
use iad::{Element, Metrics};
use iced::widget::{column, text};

#[derive(Debug, Clone)]
pub enum Msg {
    Range((f32, f32)),
    Filter(bool),
}

pub struct State {
    histogram: Histogram,
    range: (f32, f32),
    /// Only the first half of the values, to show a histogram built in the view.
    cheap_only: bool,
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
            cheap_only: false,
        }
    }
}

impl State {
    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Range(r) => self.range = r,
            Msg::Filter(f) => self.cheap_only = f,
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
        let bare = HistogramRangeSlider::new(&self.histogram, self.range, Msg::Range).step(1.0);
        let full = HistogramRangeSlider::new(&self.histogram, self.range, Msg::Range)
            .step(1.0)
            .full(true);
        // A histogram that depends on a filter is built in the view and handed over owned: nothing
        // to keep in step in the application's state.
        let filtered = Histogram::from_values(
            (0..400)
                .map(|i| {
                    let x = (i as f32 * 0.618_034).fract();
                    100.0 * x * x
                })
                .filter(|v| !self.cheap_only || *v < 50.0),
            0.0,
            100.0,
            20,
        );
        let owned = HistogramRangeSlider::new(filtered, self.range, Msg::Range).step(1.0);
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
            PreferencesGroup::<Msg>::new()
                .title("Owned: built in the view")
                .description("A histogram that depends on a filter is built where it is shown and passed by value (`Histogram::from_values(..)`), so the application keeps only the filter.")
                .metrics(m)
                .push(w::SwitchRow::new("Only values under 50", self.cheap_only, Msg::Filter).metrics(m))
                .push(w::card(owned)),
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
