//! `ComposedComponent`: a histogram and a range slider that edit one range.

use iced_adaptive_design as iad;

use iad::widget::{self as w, ComposedComponent, Histogram, HistogramRange, PreferencesGroup};
use iad::{Element, Metrics};
use iced::widget::{column, text};

#[derive(Debug, Clone)]
pub enum Msg {
    Range((f32, f32)),
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
        let composed = ComposedComponent::new(self.range, Msg::Range)
            .part(|r| HistogramRange::new(&self.histogram, *r, |picked| picked).into())
            .part(|r| {
                w::range_slider(0.0..=100.0, *r, |moved| moved)
                    .step(1.0)
                    .into()
            })
            .normalize(|(lo, hi)| (lo.min(hi), lo.max(hi)))
            .spacing(m.space_s);
        let count = self.histogram.count_in(self.range);
        PreferencesGroup::<Msg>::new()
            .title("Histogram and slider")
            .description(
                "Press a bar or drag a thumb: both edit the same range, and both redraw from it.",
            )
            .metrics(m)
            .push(w::card(
                column![
                    composed,
                    text(format!(
                        "{:.0} to {:.0}: {} of {} values",
                        self.range.0,
                        self.range.1,
                        count,
                        self.histogram.counts().iter().sum::<usize>()
                    )),
                ]
                .spacing(m.space_m),
            ))
            .into()
    }
}
