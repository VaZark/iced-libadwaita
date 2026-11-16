//! The header: the gallery launcher, the theme (a title button that opens the picker), then undo,
//! reset, save and the main menu, all icon buttons.

use super::{theme_item_props, tip};
use crate::{Editor, Msg};
use iad::components::{self as cmp, TooltipPosition};
use iad::widgets as w;
use iad::{Element, Icon};
use iced::Length;
use iced::widget::container;
use iced_adaptive_design as iad;

impl Editor {
    /// The theme's display name: the file's listed name, a bundled theme's, or "Untitled".
    fn theme_label(&self) -> String {
        match (&self.path, self.source) {
            (Some(path), _) => self
                .presets
                .iter()
                .find(|p| crate::files::canon(&p.path) == crate::files::canon(path))
                .map_or_else(|| crate::file_name(path), |p| p.name.clone()),
            (None, Some(key)) => crate::bundled::text(key)
                .map_or_else(|| key.to_string(), |t| crate::bundled::name_of(t, key)),
            (None, None) => "Untitled".to_string(),
        }
    }

    /// Where the values live, for the title's tooltip.
    fn location(&self) -> String {
        match (&self.path, self.source) {
            (Some(path), _) => path.display().to_string(),
            (None, Some(_)) => "Bundled theme: Save as writes your own copy".into(),
            (None, None) => "Not saved yet".into(),
        }
    }

    /// The title button: the theme's name (with the unsaved dot), wrapping within `width`.
    pub(crate) fn theme_selector(&self, width: f32) -> Element<'_, Msg> {
        let label = theme_item_props(width)
            .label
            .apply(cmp::typography::label(format!(
                "{}{}",
                self.dot(),
                self.theme_label()
            )));
        container(cmp::flat_button(label).on_press(Msg::ThemePicker(true)))
            .width(Length::Fixed(width))
            .max_width(width)
            .into()
    }

    pub(super) fn header(
        &self,
        collapsed: bool,
        chrome: &iad::windows::Chrome,
        m: iad::Metrics,
    ) -> Element<'_, Msg> {
        let dirty = self.dirty();
        let mut bar = w::header_bar()
            .chrome(&chrome.edges(collapsed, true), Msg::Window)
            .title_widget(cmp::tooltip(
                self.theme_selector(m.sidebar_width),
                self.location(),
                TooltipPosition::Bottom,
            ));
        if collapsed {
            bar = bar.pack_start(tip(
                cmp::icon_button(Icon::Sidebar).on_press(Msg::Drawer(true)),
                "Show sections",
            ));
        }
        bar = bar
            .pack_start(tip(
                cmp::icon_button(Icon::Play).on_press(Msg::Gallery),
                "Open the gallery with these values",
            ))
            .pack_start(
                w::ThemeToggle::new(self.launch_mode)
                    .on_toggle(Msg::LaunchMode)
                    .metrics(m),
            );
        // The menu trigger is padded like an icon button, so its glyph is not flush with the
        // hover shape.
        let side = (m.button_min_height - m.icon_size).max(0.0) / 2.0;
        // `pack_end` adds right to left: the menu is last, undo first from the left.
        bar.pack_end(
            w::primary_menu(self.main_menu())
                .tooltip("Menu")
                .open_initially(self.menu_open)
                .combobox_props(w::PopoverProps::default().padding([0.0, side]))
                .window_radius(chrome.radius()),
        )
        .pack_end(tip(
            cmp::icon_button(Icon::Save)
                .on_press_maybe((dirty || self.path.is_none()).then_some(Msg::Save)),
            "Save",
        ))
        .pack_end(tip(
            cmp::icon_button(Icon::Reset).on_press_maybe(dirty.then_some(Msg::Discard)),
            "Reset: drop the unsaved changes",
        ))
        .pack_end(tip(
            cmp::icon_button(Icon::GoPrevious)
                .on_press_maybe((!self.undo.is_empty()).then_some(Msg::Undo)),
            "Undo",
        ))
        .into()
    }
}
