//! The window: a `NavigationSplitView` with the sections on the side and, beside it, one centered
//! column holding the controls and the token page. The theme picker is a dialog over it.
//!
//! The layout follows `../organiser-apps`: a `responsive` body (a resize re-runs this closure and
//! the layout, never an update), toasts for what happened, shortcuts shown in the menu.
//!
//! * `header`: the title button, the toolbar icons and the main menu.
//! * `menu`: the main menu.
//! * `picker`: the theme dialog (list, global theme actions).
//! * `sidebar`: the section list.
//! * `content`: the centered column.

mod content;
mod header;
mod menu;
mod picker;
mod sidebar;

use crate::{Editor, Msg};
use iad::components::{self as cmp, TooltipPosition};
use iad::shortcut::Shortcuts;
use iad::widgets as w;
use iad::{Element, Role};
use iced::Length;
use iced::widget::responsive;
use iced_adaptive_design as iad;

/// Below this window width the section list is a drawer.
const SIDEBAR_COLLAPSES: f32 = 820.0;

/// Editor-local overflow policy for theme names and paths: they wrap within a bounded width.
/// Shared dropdown defaults stay unchanged.
pub(crate) fn theme_item_props(max_width: f32) -> w::DropdownItemProps {
    w::DropdownItemProps::default()
        .label_props(
            cmp::TextProps::default()
                .overflow(cmp::TextOverflow::Wrap)
                .width(Length::Fill),
        )
        .container_props(
            cmp::ContainerProps::default()
                .width(Length::Fill)
                .max_width(max_width),
        )
        .button_props(cmp::ButtonProps::default().width(Length::Fill))
}

/// `content` with a tooltip below it.
fn tip<'a>(content: impl Into<Element<'a, Msg>>, text: &'static str) -> Element<'a, Msg> {
    cmp::tooltip(content, text, TooltipPosition::Bottom).into()
}

impl Editor {
    pub(crate) fn view(&self) -> Element<'_, Msg> {
        // The window state is the default, as in the organiser apps: nothing listens to resizes.
        let chrome = iad::windows::Chrome::new(iad::windows::State::default());
        let body: Element<'_, Msg> = responsive(move |size| self.shell(size.width)).into();
        let body = if self.theme_picker {
            w::dialog(
                body,
                self.theme_picker_view(),
                Some(Msg::ThemePicker(false)),
            )
        } else {
            body
        };
        let framed = Shortcuts::new()
            .from_menu(&self.main_menu())
            .bind("Ctrl+Y", Msg::Redo)
            .scope(chrome.frame(body, Msg::Window));
        let toasts = match &self.toast {
            Some(n) => {
                let mut toast = w::Toast::new(n.text.clone())
                    .timeout(if n.error { 7000 } else { 3500 })
                    .on_dismiss(Msg::DismissToast);
                if n.error {
                    toast = toast.status(Role::Error);
                }
                vec![toast]
            }
            None => Vec::new(),
        };
        w::toast_stack(framed, toasts)
    }

    /// The split view for a window `width` wide.
    fn shell(&self, width: f32) -> Element<'_, Msg> {
        let m = iad::Metrics::desktop();
        let chrome = iad::windows::Chrome::new(iad::windows::State::default());
        let collapsed = width < SIDEBAR_COLLAPSES;

        let side = w::toolbar_view(self.sections_pane(m)).top_bar(
            w::header_bar()
                .chrome(&chrome.edges(true, false), Msg::Window)
                .title("Sections"),
        );
        let content = w::toolbar_view(self.content(m)).top_bar(self.header(collapsed, &chrome, m));

        w::NavigationSplitView::new(side, content)
            .adapt(width)
            .collapsed(collapsed)
            .show_sidebar(self.drawer)
            .on_close_drawer(Msg::Drawer(false))
            .window_radius(chrome.radius())
            .into()
    }
}
