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
mod carousel;
mod dialog;
mod dropdown;
mod elevated;
mod extra;
mod forms;
mod flex;
mod header_bar;
mod menu;
mod motion;
mod navigation;
mod preferences;
mod rows;
mod segmented;
mod scrolled;
mod slots;
mod split;
mod spinner;
mod switch;
mod status_page;
mod toast;
mod toolbar;
pub mod typography;

pub use avatar::{avatar, Avatar};
pub use banner::{banner, Banner};
pub use carousel::{carousel, Carousel, CarouselIndicator};
pub use button::{
    circular_button, destructive_button, progress_bar, flat_button, icon_button, pill_button, raised_button,
    suggested_button, suggested_pill_button, toggle_group, ButtonContent, ToggleGroup,
    ToggleItem,
};
pub use extra::{
    bottom_sheet, bottom_sheet_in_window, breakpoint_bin, carousel_indicator_dots, carousel_indicator_lines,
    inline_view_switcher, password_entry_row, path_bar, preferences_dialog, shortcut_label, shortcuts_dialog,
    sidebar, tab_bar, tab_button, view_switcher, view_switcher_bar, window, wrap_box,
};
pub use dropdown::{dropdown, dropdown_disabled, menu_item, menu_item_disabled, popover, Align as PopoverAlign, Appearance as DropdownAppearance, Dropdown, Presentation as DropdownPresentation};
pub use elevated::{card_shadow, Elevated};
pub use split::{split_button, SplitAction, SplitButton, SplitButtonAction};
pub use switch::{switch, Switch};
pub use dialog::{dialog, dialog_in_window, present_dialog, AboutDialog, AlertDialog, ResponseAppearance};
pub use flex::FlexRow;
pub use iced::widget::text_editor::{Action as TextAction, Content as TextContent};
pub use forms::{
    accent_picker, card_button, check_button, color_swatch, entry, link_button, password_entry,
    radio_button, scale, status_swatch, tag, text_view, Entry, EntryState, TextView,
};
pub use header_bar::{header_bar, window_title, HeaderBar, WindowControls};
pub use motion::{drawer_layer, entrance, highlight, layered, page_transition, push_stack, reveal, DrawerLayer, Entrance, Highlight, PageTransition, PushStack, Reveal};
pub use menu::{menu_button, primary_menu, Menu, MenuButton, MenuItem, MenuPresentation};
pub use navigation::{
    navigation_sidebar, navigation_sidebar_rounded, CollapsedSidebar, navigation_view, sidebar_row, Breakpoint, NavigationPage,
    collapse_width, sidebar_width_for, SidebarItem, SidebarList, SidebarStyle, NavigationSplitView, OverlaySplitView, COLLAPSE_WIDTH,
};
pub use preferences::{preferences_group, preferences_page, PreferencesGroup, PreferencesPage};
pub use rows::{
    action_row, boxed_list, ActionRow, ButtonRow, ComboRow, EntryRow, ExpanderRow, SliderRow, SpinRow,
    SwitchRow, ToggleGroupRow,
};
pub use scrolled::{scrollbar, scrolled_window, scrolled_window_both, scrolled_window_horizontal};
pub use spinner::{spinner, Spinner};
pub use status_page::{bin, card, clamp, page, status_page, Card, Page, StatusPage};
pub use toast::{toast_overlay, toast_stack, toast_target, Toast, ToastPosition, ToastTarget};
pub use toolbar::{action_bar, toolbar_view, ActionBar, BarStyle, ToolbarView};

/// Shorthand for an iced element using this crate's [`Theme`](crate::Theme).
pub type Element<'a, Message> = iced::Element<'a, Message, crate::Theme>;
