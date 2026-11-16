//! The content pane: one centered column. The controls (variant, search, filters) stay at the top
//! and the token page scrolls under them; both sit in a container of the same maximum width, so
//! they line up at any window size.

use crate::{Editor, Msg, Variant};
use iad::Element;
use iad::components::{self as cmp, TooltipPosition};
use iced::widget::{column, container, row, text};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;

/// How wide the column is at most, beyond the clamp the rest of the toolkit uses.
fn column_width(m: iad::Metrics) -> f32 {
    m.clamp_maximum + 160.0
}

/// `content` in the centered column: at most [`column_width`] wide, padded at the sides.
fn centered<'a>(content: impl Into<Element<'a, Msg>>, m: iad::Metrics) -> Element<'a, Msg> {
    container(
        container(content)
            .width(Length::Fill)
            .max_width(column_width(m)),
    )
    .padding([0.0, m.space_m])
    .center_x(Length::Fill)
    .into()
}

impl Editor {
    pub(super) fn content(&self, m: iad::Metrics) -> Element<'_, Msg> {
        column![
            // The page beside it gives up the scrollbar and its gutter at the right while it
            // scrolls: the controls do the same, so both stay on one center line.
            container(centered(self.controls(m), m)).padding(iced::Padding {
                top: m.space_m,
                bottom: m.space_m,
                left: 0.0,
                right: m.scrollbar_width + m.scrollbar_gutter,
            }),
            self.tokens_page(m),
        ]
        .height(Length::Fill)
        .into()
    }

    /// Variant and search on one row, the filters under it.
    fn controls(&self, m: iad::Metrics) -> Element<'_, Msg> {
        let base = self.variant == Variant::Base;
        let variants = cmp::toggle_group(
            Variant::ALL
                .iter()
                .map(|v| cmp::ToggleItem::new(*v, v.label()))
                .collect(),
            Some(self.variant),
            Msg::Variant,
        );
        let search = cmp::entry("Search tokens", self.search.as_str())
            .left_section(iad::icon::EntryIcon::Search.view(16.0))
            .on_input(Msg::Search)
            .width(Length::Fill);

        let switch = |label: &'static str, on: bool, msg: Option<fn(bool) -> Msg>| {
            let mut s = cmp::switch(on);
            if let Some(msg) = msg {
                s = s.on_toggle(msg);
            }
            row![text(label), s]
                .spacing(m.space_s)
                .align_y(Alignment::Center)
        };
        let mut filters = row![
            switch("Modified only", self.modified_only, Some(Msg::ModifiedOnly)),
            tip_text(
                switch(
                    "Overrides only",
                    self.overrides_only,
                    (!base).then_some(Msg::OverridesOnly as fn(bool) -> Msg),
                ),
                if base {
                    "Choose Light or Dark to show only tokens explicitly set in that scheme."
                } else {
                    "Only values explicitly set in the active scheme; inherited base values are hidden."
                },
            ),
        ]
        .spacing(m.space_l)
        .align_y(Alignment::Center);
        if !base {
            filters = filters.push(tip_text(
                cmp::flat_button(cmp::typography::label("Copy from Base"))
                    .on_press(Msg::CopyFromBase),
                "Copy what the base sets into this variant, where it sets nothing",
            ));
        }
        let mut controls = column![
            row![variants, search]
                .spacing(m.space_m)
                .align_y(Alignment::Center),
            filters.wrap().vertical_spacing(m.space_s),
        ]
        .spacing(m.space_m);
        if !base {
            controls = controls.push(
                text(format!(
                    "Editing the {} variant: values here win over the base on that color scheme.",
                    self.variant.label().to_lowercase()
                ))
                .size(m.space_m + 1.0)
                .style(iad::styles::text::dim),
            );
        }
        controls.into()
    }

    /// The token page, scrolling under the controls in the same centered column.
    fn tokens_page(&self, m: iad::Metrics) -> Element<'_, Msg> {
        iad::layouts::scrolled_window(centered(
            container(self.page(m)).padding([m.space_s, 0.0]),
            m,
        ))
        .id(iced::widget::Id::new(crate::tokens::TOKENS_SCROLL))
        .on_scroll(|v| {
            Msg::TokensScrolled(iad::layouts::virtualize::Viewport {
                offset: v.absolute_offset().y,
                height: v.bounds().height,
            })
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

/// `content` with a tooltip below it.
fn tip_text<'a>(content: impl Into<Element<'a, Msg>>, text: &'static str) -> Element<'a, Msg> {
    cmp::tooltip(content, text, TooltipPosition::Bottom).into()
}
