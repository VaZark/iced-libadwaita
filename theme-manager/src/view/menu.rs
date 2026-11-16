//! The main menu. Its accelerators work while it is in the view.

use crate::{Editor, Msg};
use iad::widgets as w;
use iced_adaptive_design as iad;

impl Editor {
    /// File actions, edit, registering, the gallery and folders.
    pub(crate) fn main_menu(&self) -> w::Menu<Msg> {
        let registered = match (self.registry(), &self.path) {
            (Some(dir), Some(file)) => Some((
                crate::files::is_registered(dir, file),
                !crate::files::links_to(dir, file).is_empty(),
            )),
            _ => None,
        };
        let item = |label: &str, msg: Msg, accel: Option<&str>, enabled: bool| {
            let mut i = w::MenuItem::action(label, msg);
            if let Some(a) = accel {
                i = i.accel(a);
            }
            if !enabled {
                i = i.disabled();
            }
            i
        };
        w::Menu::new("Menu")
            .section([
                item("New", Msg::New, Some("Ctrl+N"), true),
                item("Open…", Msg::Open, Some("Ctrl+O"), true),
                item("Choose theme…", Msg::ThemePicker(true), None, true),
            ])
            .section([
                item("Save", Msg::Save, Some("Ctrl+S"), true),
                item("Save as…", Msg::SaveAs, Some("Ctrl+Shift+S"), true),
            ])
            .section([
                item("Undo", Msg::Undo, Some("Ctrl+Z"), !self.undo.is_empty()),
                item(
                    "Redo",
                    Msg::Redo,
                    Some("Ctrl+Shift+Z"),
                    !self.redo.is_empty(),
                ),
            ])
            .section([
                item(
                    "Register for all apps",
                    Msg::Register,
                    None,
                    registered.is_some_and(|(is, _)| !is),
                ),
                item(
                    "Unregister",
                    Msg::Unregister,
                    None,
                    registered.is_some_and(|(_, linked)| linked),
                ),
            ])
            .section([
                item("Open the gallery", Msg::Gallery, None, true),
                item("Add folder…", Msg::AddFolder, None, true),
                item("Refresh the list", Msg::Refresh, Some("F5"), true),
            ])
    }
}
