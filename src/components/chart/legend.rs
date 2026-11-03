//! The legend: a swatch and a name for each series, wrapping onto lines.

use super::color;
use crate::widget::Element;
use crate::{Metrics, Theme};
use iced::widget::{container, row, space, text};
use iced::{Alignment, Length};

/// The legend for `names`, colored by position like the chart's series.
pub fn legend<'a, M: 'a>(names: impl IntoIterator<Item = String>) -> Element<'a, M> {
    let m = Metrics::desktop();
    let items = names.into_iter().enumerate().map(|(i, name)| {
        row![
            container(space().width(10).height(10)).style(move |t: &Theme| container::Style {
                background: Some(color::series(t, i).into()),
                border: iced::Border::default().rounded(2),
                ..Default::default()
            }),
            text(name).size(super::LABEL_SIZE).style(sc_text),
        ]
        .spacing(m.space_xs + 2.0)
        .align_y(Alignment::Center)
        .into()
    });
    row(items)
        .spacing(m.space_m)
        .width(Length::Fill)
        .wrap()
        .into()
}

fn sc_text(t: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(t.semantic.window.fg),
    }
}
