//! `ComposedComponent`: several parts that edit one shared value. Two range sliders of different
//! step and a readout all show and change the same range, so they cannot disagree.

use iced_adaptive_design as iad;

use iad::metas::{ComposeAxis, ComposedComponent};
use iad::widgets as w;
use iad::widgets::PreferencesGroup;
use iad::{Element, Metrics};
use iced::widget::{column, text};

#[derive(Debug, Clone)]
pub enum Msg {
    Range((f32, f32)),
    Horizontal(bool),
}

pub struct State {
    range: (f32, f32),
    /// Lay the parts out side by side instead of stacked.
    horizontal: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            range: (20.0, 60.0),
            horizontal: false,
        }
    }
}

impl State {
    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Range(r) => self.range = r,
            Msg::Horizontal(h) => self.horizontal = h,
        }
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let composed = ComposedComponent::new(self.range, Msg::Range)
            .part(|r| text(format!("{:.0} to {:.0}", r.0, r.1)).into())
            .part(|r| w::range_slider(0.0..=100.0, *r, |moved| moved).into())
            .part(|r| {
                w::range_slider(0.0..=100.0, *r, |moved| moved)
                    .step(10.0)
                    .into()
            })
            .normalize(|(lo, hi)| (lo.min(hi), lo.max(hi)))
            .axis(if self.horizontal {
                ComposeAxis::Horizontal
            } else {
                ComposeAxis::Vertical
            })
            .spacing(m.space_m);
        column![
            PreferencesGroup::<Msg>::new()
                .title("Shared range")
                .description("Three parts, one state: a readout, a free slider and a slider that steps by 10. Whichever the user touches, the application gets one value through one message, and every part is built from it on the next view.")
                .metrics(m)
                .push(w::card(composed)),
            PreferencesGroup::<Msg>::new()
                .title("Axis")
                .metrics(m)
                .push(w::SwitchRow::new("Side by side", self.horizontal, Msg::Horizontal).metrics(m)),
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
