//! The section list: one flat button per section, the open one raised, each with how many tokens
//! the file sets in it.

use crate::{Editor, Msg};
use iad::Element;
use iad::components as cmp;
use iced::Length;
use iced::widget::{column, row, space, text};
use iced_adaptive_design as iad;

impl Editor {
    pub(super) fn sections_pane(&self, m: iad::Metrics) -> Element<'_, Msg> {
        let current = self.effective_section();
        let list = column(self.section_choices().map(|s| {
            let set = self.set_count(s);
            let mut label = row![text(crate::tokens::section_label(s))]
                .spacing(m.space_xs)
                .width(Length::Fill);
            if set > 0 {
                label = label.push(space().width(Length::Fill)).push(
                    text(set.to_string())
                        .size(m.space_m)
                        .style(iad::styles::text::dim),
                );
            }
            let button = if s == current {
                cmp::raised_button(label)
            } else {
                cmp::flat_button(label)
            };
            button
                .on_press(Msg::Section(s.to_string()))
                .width(Length::Fill)
                .into()
        }))
        .spacing(m.space_xxs)
        .width(Length::Fill);
        iad::layouts::scrolled_window(list)
            .height(Length::Fill)
            .into()
    }
}
