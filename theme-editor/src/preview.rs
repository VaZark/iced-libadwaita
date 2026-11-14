//! The live preview: a small sample UI drawn with the theme being edited, inside the editor,
//! while the editor's own chrome keeps its fixed look.
//!
//! The preview's `Theme` (colors, radii, shadows) comes from the values as they are now, through
//! `iced::widget::Themer`. Its layout metrics (paddings, heights, gaps) are passed to the widgets
//! that take them (`.metrics(m)`). Widgets that read the global config for their text size or
//! internal gaps keep the editor's own values: the gallery and the toy windows show those.

use crate::Msg;
use iced::widget::{column, container, row, space, text};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;
use iad::components as cmp;
use iad::theme_config::ThemeConfig;
use iad::widgets as w;
use iad::{Icon, Metrics, Role, Scheme};

/// What the preview is built from: the theme and the layout metrics the values imply.
#[derive(Clone)]
pub(crate) struct Preview {
    pub(crate) theme: iad::Theme,
    pub(crate) metrics: Metrics,
    /// The text of the file this was built from.
    pub(crate) source: String,
    pub(crate) dark: bool,
}

impl Preview {
    /// The preview of `text` (a theme file) in the light or the dark scheme.
    pub(crate) fn build(text: &str, dark: bool) -> Self {
        let cfg = ThemeConfig::builder("org.iad.ThemeEditor.Preview")
            .files(false)
            .toolkit_text(text)
            .app_text("")
            .build();
        let mut metrics = Metrics::default();
        cfg.apply_to_metrics(&mut metrics);
        let scheme = if dark { Scheme::Dark } else { Scheme::Light };
        Self {
            theme: iad::Theme::new(scheme).with_theme_config(&cfg),
            metrics,
            source: text.to_string(),
            dark,
        }
    }

    /// The sample UI, wrapped so it is drawn with the preview's theme.
    pub(crate) fn view(&self) -> iad::Element<'_, Msg> {
        let sample = self.sample();
        iced::widget::Themer::new(Some(self.theme.clone()), sample).into()
    }

    fn sample(&self) -> iad::Element<'_, Msg> {
        let m = self.metrics;
        let ty = iad::Typography::desktop();
        let noop = || Msg::Noop;

        let header = w::header_bar()
            .title("Preview")
            .subtitle("This is how the values look")
            .pack_start(cmp::icon_button(Icon::Sidebar).on_press(noop()))
            .pack_end(cmp::icon_button(Icon::Search).on_press(noop()))
            .metrics(m);

        let buttons = row![
            cmp::raised_button(cmp::typography::label("Default")).on_press(noop()),
            cmp::flat_button(cmp::typography::label("Flat")).on_press(noop()),
            cmp::suggested_button(cmp::typography::label("Suggested")).on_press(noop()),
            cmp::destructive_button(cmp::typography::label("Destructive")).on_press(noop()),
            cmp::pill_button(cmp::typography::label("Pill")).on_press(noop()),
            cmp::circular_button(Icon::Play).on_press(noop()),
            cmp::raised_button(cmp::typography::label("Disabled")),
        ]
        .spacing(m.space_s)
        .align_y(Alignment::Center)
        .wrap();

        let rows = w::preferences_group()
            .title("Rows")
            .description("A group of rows in a card")
            .push(
                w::ActionRow::new("Action row")
                    .subtitle("With a subtitle and a suffix")
                    .suffix(cmp::switch(true).on_toggle(|_| Msg::Noop))
                    .metrics(m),
            )
            .push(w::SwitchRow::new("Switch row", false, |_| Msg::Noop).metrics(m))
            .push(
                w::ActionRow::new("Entry")
                    .suffix(
                        cmp::entry("Placeholder", "Some text")
                            .on_input(|_| Msg::Noop)
                            .width(Length::Fixed(160.0))
                            .metrics(m),
                    )
                    .metrics(m),
            )
            .metrics(m);

        let controls = row![
            cmp::check_button("Check", true).on_toggle(|_| Msg::Noop),
            cmp::check_button("Unchecked", false).on_toggle(|_| Msg::Noop),
            cmp::progress_bar(0.0..=1.0, 0.6).length(Length::Fixed(120.0)),
        ]
        .spacing(m.space_m)
        .align_y(Alignment::Center)
        .wrap();

        let tags = row![
            cmp::tag("Accent", Role::Accent),
            cmp::tag("Success", Role::Success),
            cmp::tag("Warning", Role::Warning),
            cmp::tag("Error", Role::Error),
            cmp::tag("Destructive", Role::Destructive),
        ]
        .spacing(m.space_s)
        .wrap();

        let surfaces = {
            let s = &self.theme.semantic;
            column![
                row![
                    cmp::color_swatch("window", s.window),
                    cmp::color_swatch("view", s.view),
                    cmp::color_swatch("headerbar", s.headerbar),
                ]
                .spacing(m.space_xs),
                row![
                    cmp::color_swatch("card", s.card),
                    cmp::color_swatch("sidebar", s.sidebar),
                    cmp::color_swatch("popover", s.popover),
                ]
                .spacing(m.space_xs),
                row![
                    cmp::status_swatch("accent", s.accent),
                    cmp::status_swatch("primary", s.primary),
                    cmp::status_swatch("success", s.success),
                ]
                .spacing(m.space_xs),
            ]
            .spacing(m.space_xs)
        };

        let card = w::card(
            column![
                text("Text styles").size(ty.heading),
                ty.title_3("Title"),
                ty.body("Body text sits on a card, on the window, on a view."),
                ty.caption("A caption, dimmer and smaller."),
                controls,
                tags,
            ]
            .spacing(m.space_s),
        );

        let page = column![buttons, rows, card, surfaces]
            .spacing(m.group_spacing)
            .padding(m.page_padding);

        let body = container(iad::layouts::scrolled_window(page))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|t: &iad::Theme| iced::widget::container::Style {
                background: Some(t.semantic.window.bg.into()),
                text_color: Some(t.semantic.window.fg),
                ..Default::default()
            });
        column![header, body, space().height(0)].into()
    }
}
