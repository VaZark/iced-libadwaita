//! libadwaita-named widgets.
//!
//! Each widget is a small builder that turns into an
//! [`Element`](crate::Element). Colors and states are resolved from the
//! [`Theme`](crate::Theme) at draw time; sizes (padding, spacing, min heights)
//! come from [`Metrics`](crate::Metrics) and are resolved when the view is
//! built, so use `.metrics(..)` on a widget to deviate from the defaults.

mod avatar;
mod banner;
mod button;
mod dialog;
mod dropdown;
mod extra;
mod flex;
mod header_bar;
mod menu;
mod navigation;
mod preferences;
mod rows;
mod scrolled;
mod slots;
mod spinner;
mod switch;
mod status_page;
mod toast;
mod toolbar;
pub mod typography;

pub use avatar::{avatar, Avatar};
pub use banner::{banner, Banner};
pub use button::{
    circular_button, destructive_button, progress_bar, flat_button, icon_button, pill_button, raised_button,
    split_button, suggested_button, suggested_pill_button, toggle_group, ButtonContent, ToggleGroup,
    ToggleItem,
};
pub use extra::{
    bottom_sheet, breakpoint_bin, carousel, carousel_indicator_dots, carousel_indicator_lines,
    inline_view_switcher, password_entry_row, preferences_dialog, shortcut_label, shortcuts_dialog,
    sidebar, tab_bar, tab_button, view_switcher, view_switcher_bar, window, wrap_box,
};
pub use dropdown::{dropdown, menu_item, popover, Align as PopoverAlign, Appearance as DropdownAppearance, Dropdown, Presentation as DropdownPresentation};
pub use switch::{switch, Switch};
pub use dialog::{dialog, AboutDialog, AlertDialog, ResponseAppearance};
pub use flex::FlexRow;
pub use header_bar::{header_bar, window_title, HeaderBar, WindowControls};
pub use menu::{menu_button, primary_menu, Menu, MenuButton, MenuItem, MenuPresentation};
pub use navigation::{
    navigation_sidebar, navigation_view, sidebar_row, Breakpoint, NavigationPage,
    NavigationSplitView, OverlaySplitView,
};
pub use preferences::{preferences_group, preferences_page, PreferencesGroup, PreferencesPage};
pub use rows::{
    action_row, boxed_list, ActionRow, ButtonRow, ComboRow, EntryRow, ExpanderRow, SpinRow,
    SwitchRow,
};
pub use scrolled::{scrollbar, scrolled_window, scrolled_window_both, scrolled_window_horizontal};
pub use spinner::{spinner, Spinner};
pub use status_page::{bin, card, clamp, status_page, StatusPage};
pub use toast::{toast_overlay, Toast};
pub use toolbar::{action_bar, toolbar_view, ActionBar, ToolbarView};

/// Shorthand for an iced element using this crate's [`Theme`](crate::Theme).
pub type Element<'a, Message> = iced::Element<'a, Message, crate::Theme>;
