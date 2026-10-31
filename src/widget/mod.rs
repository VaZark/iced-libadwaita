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
mod divider;
mod dropdown;
mod elevated;
mod extra;
mod flex;
mod forms;
mod header_bar;
mod menu;
mod motion;
mod navigation;
mod preferences;
mod rows;
mod scrolled;
mod segmented;
mod shortcut_scope;
mod slots;
mod spinner;
mod split;
mod split_view;
mod status_page;
mod switch;
mod toast;
mod toolbar;
pub mod typography;

pub use avatar::{Avatar, avatar};
pub use banner::{Banner, banner};
pub use button::{
    ButtonContent, ToggleGroup, ToggleItem, circular_button, destructive_button, flat_button,
    icon_button, pill_button, progress_bar, raised_button, suggested_button, suggested_pill_button,
    toggle_group,
};
pub use carousel::{Carousel, CarouselIndicator, carousel};
pub use dialog::{
    AboutDialog, AlertDialog, ResponseAppearance, dialog, dialog_in_window, present_dialog,
};
pub use divider::{Divider, DividerTone, divider};
pub use dropdown::{
    Align as PopoverAlign, Appearance as DropdownAppearance, Dropdown,
    Presentation as DropdownPresentation, dropdown, dropdown_disabled, menu_item,
    menu_item_disabled, popover,
};
pub use elevated::{Elevated, card_shadow};
pub use extra::{
    bottom_sheet, bottom_sheet_in_window, breakpoint_bin, carousel_indicator_dots,
    carousel_indicator_lines, inline_view_switcher, password_entry_row, path_bar,
    preferences_dialog, shortcut_label, shortcuts_dialog, sidebar, tab_bar, tab_button,
    view_switcher, view_switcher_bar, window, wrap_box,
};
pub use flex::FlexRow;
pub use forms::{
    Entry, EntryState, TextView, accent_picker, card_button, check_button, color_swatch, entry,
    link_button, password_entry, radio_button, scale, status_swatch, tag, text_view,
};
pub use header_bar::{HeaderBar, WindowControls, header_bar, window_title};
pub use iced::widget::text_editor::{Action as TextAction, Content as TextContent};
pub use menu::{Menu, MenuButton, MenuItem, MenuPresentation, menu_button, primary_menu};
pub use motion::{
    DrawerLayer, Entrance, Highlight, PageTransition, PushStack, Reveal, drawer_layer, entrance,
    highlight, layered, page_transition, push_stack, reveal,
};
pub use navigation::{
    Breakpoint, COLLAPSE_WIDTH, CollapsedSidebar, DRAWER_MAX_FRACTION, MIN_CONTENT_WIDTH,
    NavigationPage, NavigationSplitView, OverlaySplitView, SIDEBAR_FRACTION, SIDEBAR_MAX_WIDTH,
    SIDEBAR_MIN_WIDTH, SidebarItem, SidebarList, SidebarPadding, SidebarStyle, collapse_width,
    min_content_width, navigation_sidebar, navigation_sidebar_rounded,
    navigation_sidebar_rounded_padded, navigation_view, sidebar_row, sidebar_row_padded,
    sidebar_width_for, split_collapsed,
};
pub use preferences::{PreferencesGroup, PreferencesPage, preferences_group, preferences_page};
pub use rows::{
    ActionRow, ButtonRow, ComboRow, EntryRow, ExpanderRow, SliderRow, SpinRow, SwitchRow,
    ToggleGroupRow, action_row, boxed_list,
};
pub use scrolled::{scrollbar, scrolled_window, scrolled_window_both, scrolled_window_horizontal};
pub use shortcut_scope::shortcuts;
pub use spinner::{Spinner, spinner};
pub use split::{SplitAction, SplitButton, SplitButtonAction, split_button};
pub use split_view::{AdaptiveSplit, adaptive_split};
pub use status_page::{Card, Page, StatusPage, bin, card, clamp, page, status_page};
pub use switch::{Switch, switch};
pub use toast::{Toast, ToastPosition, ToastTarget, toast_overlay, toast_stack, toast_target};
pub use toolbar::{ActionBar, BarStyle, ToolbarView, action_bar, toolbar_view};

/// Shorthand for an iced element using this crate's [`Theme`](crate::Theme).
pub type Element<'a, Message> = iced::Element<'a, Message, crate::Theme>;
