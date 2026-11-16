//! The theme dialog: a filter, the themes by group (registered, bundled, folders, recent), the
//! global theme actions and the file actions.

use super::theme_item_props;
use crate::{Editor, Msg};
use iad::components as cmp;
use iad::widgets as w;
use iad::{Element, Icon};
use iced::widget::{column, container, responsive, row, space, text};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;

impl Editor {
    pub(super) fn theme_picker_view(&self) -> Element<'_, Msg> {
        responsive(move |size| self.theme_picker_at(size)).into()
    }

    /// The dialog's card for a window of `size`: as large as the toolkit's clamp, never more than
    /// the window less a margin.
    pub(crate) fn theme_picker_at(&self, size: iced::Size) -> Element<'_, Msg> {
        let m = iad::Metrics::desktop();
        let width = m.clamp_maximum.min((size.width - 2.0 * m.space_m).max(0.0));
        let height = m
            .clamp_maximum
            .min((size.height - 2.0 * m.space_m).max(0.0));
        let card = container(
            column![
                row![
                    text("Choose a theme"),
                    space().width(Length::Fill),
                    cmp::icon_button(Icon::WindowClose).on_press(Msg::ThemePicker(false)),
                ]
                .align_y(Alignment::Center),
                self.files_pane(m, (width - m.page_padding.x()).max(0.0)),
                self.global_row(m),
                row![
                    cmp::flat_button(cmp::typography::label("New")).on_press(Msg::New),
                    cmp::flat_button(cmp::typography::label("Open…")).on_press(Msg::Open),
                    cmp::flat_button(cmp::typography::label("Add folder…"))
                        .on_press(Msg::AddFolder),
                    cmp::icon_button(Icon::Reset).on_press(Msg::Refresh),
                ]
                .spacing(m.space_s)
                .wrap(),
            ]
            .spacing(m.space_m),
        )
        .padding(m.page_padding)
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .clip(true)
        .style(iad::styles::container::card);
        container(card)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    /// The global theme: whether the open theme is it, and the actions on it.
    fn global_row(&self, m: iad::Metrics) -> Element<'_, Msg> {
        let has_global = self
            .registry()
            .is_some_and(|d| crate::files::default_target(d).is_some());
        let is_global = self.is_global();
        row![
            text(if is_global {
                "This is the global theme"
            } else {
                "Apps do not use this theme yet"
            })
            .style(iad::styles::text::dim),
            space().width(Length::Fill),
            cmp::raised_button(cmp::typography::label("Use as global theme"))
                .on_press_maybe((!is_global).then_some(Msg::SetGlobal)),
            cmp::flat_button(cmp::typography::label("Remove global theme"))
                .on_press_maybe(has_global.then_some(Msg::ClearGlobal)),
        ]
        .spacing(m.space_s)
        .align_y(Alignment::Center)
        .wrap()
        .into()
    }

    /// One theme in the list: its name (marked when open or global) and where it lives.
    pub(crate) fn theme_option(
        &self,
        preset: &crate::files::Preset,
        width: f32,
    ) -> Element<'_, Msg> {
        let active = match preset.bundled {
            Some(key) => self.path.is_none() && self.source == Some(key),
            None => self
                .path
                .as_deref()
                .map(crate::files::canon)
                .is_some_and(|path| path == crate::files::canon(&preset.path)),
        };
        let place = match (&preset.target, preset.bundled) {
            (_, Some(_)) => "Bundled with the app".to_string(),
            (Some(target), None) => format!("{} → {}", preset.path.display(), target.display()),
            (None, None) => preset.path.display().to_string(),
        };
        let label = format!(
            "{}{}{}\n{place}",
            if active { self.dot() } else { "" },
            preset.name,
            if preset.is_default { " · default" } else { "" }
        );
        let on_press = match preset.bundled {
            Some(key) => Msg::PickBundled(key),
            None => Msg::Pick(preset.path.clone()),
        };
        w::menu_item_with_props(label, active, on_press, theme_item_props(width))
    }

    /// The filter and the grouped list, or an empty state.
    fn files_pane(&self, m: iad::Metrics, width: f32) -> Element<'_, Msg> {
        let filter = cmp::entry("Filter themes", self.files_filter.as_str())
            .left_section(iad::icon::EntryIcon::Search.view(16.0))
            .on_input(Msg::FilesFilter)
            .width(Length::Fill);
        let query = self.files_filter.trim().to_lowercase();
        let matches = |p: &crate::files::Preset| {
            query.is_empty()
                || p.name.to_lowercase().contains(&query)
                || p.path.display().to_string().to_lowercase().contains(&query)
        };

        let mut list = column![].spacing(m.space_xxs).width(Length::Fill);
        let (mut group, mut shown) = ("", 0);
        for preset in self.presets.iter().filter(|p| matches(p)) {
            shown += 1;
            if preset.group != group {
                group = preset.group;
                list = list.push(
                    text(group)
                        .size(m.space_m + 1.0)
                        .style(iad::styles::text::dim),
                );
            }
            list = list.push(self.theme_option(preset, (width - 2.0 * m.space_s).max(0.0)));
        }

        let body: Element<'_, Msg> = if shown == 0 {
            let none = self.presets.is_empty();
            w::status_page()
                .icon(Icon::Folder.view(48.0))
                .title(if none {
                    "No themes yet"
                } else {
                    "Nothing matches"
                })
                .description(if none {
                    "Open a theme file or start a new one. Save as registers it for every app."
                } else {
                    "No theme has that name or path."
                })
                .into()
        } else {
            iad::layouts::scrolled_window(container(list).padding([0.0, m.space_s]))
                .height(Length::Fill)
                .into()
        };
        column![container(filter).padding([m.space_xs, m.space_s]), body]
            .spacing(m.space_xs)
            .height(Length::Fill)
            .into()
    }
}
