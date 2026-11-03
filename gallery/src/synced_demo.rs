//! `SyncedText`: timed lines with the current one highlighted. A play button advances the
//! position; pressing a line seeks to it.

use iced_adaptive_design as iad;

use iad::widget::{self as w, PreferencesGroup, SyncedLine, SyncedText, synced};
use iad::{Element, Icon, Metrics};
use iced::widget::{Id, container, row, text};
use iced::{Length, Task};

const LIST: &str = "synced-demo";
const STEP: f32 = 4.0;

#[derive(Debug, Clone)]
pub enum Msg {
    Seek(f32),
    Step(f32),
}

pub struct State {
    lines: Vec<SyncedLine>,
    pos: f32,
}

impl Default for State {
    fn default() -> Self {
        let words = [
            "The clock begins to turn",
            "Every line has its hour",
            "Press one to go back there",
            "The list follows the current one",
            "Subtitles work the same way",
            "So does a transcript",
            "Or a changelog timeline",
            "Lines end when the next starts",
            "Before the first, none is current",
            "After the last, the last stays",
        ];
        Self {
            lines: words
                .iter()
                .enumerate()
                .map(|(i, t)| SyncedLine::new(i as f32 * STEP, *t))
                .collect(),
            pos: 0.0,
        }
    }
}

impl State {
    pub fn update(&mut self, m: Msg) -> Task<Msg> {
        let before = synced::current(&self.lines, self.pos);
        match m {
            Msg::Seek(at) => self.pos = at,
            Msg::Step(d) => self.pos = (self.pos + d).max(0.0),
        }
        let now = synced::current(&self.lines, self.pos);
        if now != before {
            w::follow_synced_text(Id::new(LIST), now, self.lines.len())
        } else {
            Task::none()
        }
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let list =
            container(SyncedText::new(Id::new(LIST), &self.lines, Msg::Seek).position(self.pos))
                .height(Length::Fixed(260.0));
        let controls = row![
            w::flat_button(Icon::GoPrevious.view(m.icon_size)).on_press(Msg::Step(-STEP)),
            text(format!("{:.0} s", self.pos)),
            w::flat_button(Icon::GoNext.view(m.icon_size)).on_press(Msg::Step(STEP)),
        ]
        .spacing(m.space_m)
        .align_y(iced::Alignment::Center);
        PreferencesGroup::<Msg>::new()
            .title("Synced text")
            .description("The current line is large; the rest are dim. Press a line to seek; the arrows move the position.")
            .metrics(m)
            .push(w::card(iced::widget::column![controls, list].spacing(m.space_m)))
            .into()
    }
}
