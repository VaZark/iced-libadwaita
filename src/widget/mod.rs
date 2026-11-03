//! libadwaita-named widgets.
//!
//! Each widget is a small builder that turns into an
//! [`Element`](crate::Element). Colors and states are resolved from the
//! [`Theme`](crate::Theme) at draw time; sizes (padding, spacing, min heights)
//! come from [`Metrics`](crate::Metrics) and are resolved when the view is
//! built, so use `.metrics(..)` on a widget to deviate from the defaults.

mod accordion;
mod anchored;
mod autocomplete;
mod avatar;
mod badge;
mod banner;
mod button;
mod carousel;
mod collapsible;
mod color_picker;
mod content_area;
mod date_input;
mod date_range_input;
mod divider;
mod dropdown;
mod elevated;
mod extra;
mod field;
mod flex;
mod floating_indicator;
mod form;
mod forms;
mod header_bar;
mod infinite;
mod menu;
mod meta_component;
mod meta_parent_component;
mod motion;
mod multi_select;
mod navigation;
mod number_input;
mod overlay;
mod pagination;
mod popover;
mod preferences;
mod range_slider;
mod reorderable_list;
mod resizable;
mod responsive_table;
mod rows;
mod scrolled;
mod sections;
mod segmented;
mod shortcut_scope;
mod slots;
mod spinner;
mod split;
mod split_view;
mod splitter;
mod spoiler;
mod status_page;
mod stepper;
mod switch;
mod tabs;
mod timeline;
mod toast;
mod toolbar;
mod tree;
mod tree_row;
pub mod typography;

pub use crate::primitives::anchor::{self, Align as AnchorAlign, Placed, Placement, Side, place};
pub use crate::primitives::calendar::{CalendarView, Dir as NavDir, Level};
pub use crate::primitives::color_picker::{Format as ColorFormat, Hsva};
pub use crate::primitives::date::{Date, days_in_month, is_leap_year, validate as validate_date};
pub use crate::primitives::form::{FormSpec, FormState, Outcome as FormOutcome, StepStatus};
pub use crate::primitives::meta_parent::Registry as MetaRegistry;
pub use crate::primitives::meta_parent::{ChildEvent, Emitted};
pub use crate::primitives::numeric::{Decimal, Numeric};
pub use crate::primitives::range::{Collide, DateRange, DayRole, Edge, Range, Thumb, Track};
pub use crate::primitives::reorder::{Event as ReorderEvent, Mode, ReorderState};
pub use crate::primitives::splitter::{
    Event as SplitterEvent, Limit as SplitterLimit, Pane as SplitterPane, Redistribute,
    Size as SplitterSize, SplitterState,
};
pub use crate::primitives::spoiler::{Event as SpoilerEvent, SpoilerState};
pub use crate::primitives::stepper::{Fill as StepperFill, Status as StepperStatus, StepperState};
pub use crate::primitives::tree::{
    Effect as TreeEffect, Event as TreeEvent, Position as DropPosition, TreeNode, TreeState,
};
pub use crate::primitives::validation::{
    self, Interaction, Presentation as ValidationState, Validity,
};
pub use crate::primitives::{Selection, Trigger, When};
pub use crate::window::dialog::{
    AboutDialog, AlertDialog, ResponseAppearance, dialog, dialog_in_window, preferences_dialog,
    present_dialog, shortcuts_dialog,
};
pub use accordion::{Accordion, AccordionItem};
pub use anchored::{
    Anchored, Indicator, dock_banner, dock_center, dock_drawer, dock_footer, docked_anchored,
};
pub use autocomplete::Autocomplete;
pub use avatar::{Avatar, avatar};
pub use badge::{Badge, Chip};
pub use banner::{Banner, banner};
pub use button::{
    ButtonContent, ToggleGroup, ToggleItem, circular_button, destructive_button, flat_button,
    icon_button, pill_button, progress_bar, raised_button, suggested_button, suggested_pill_button,
    toggle_group,
};
pub use carousel::{Carousel, CarouselIndicator, carousel};
pub use collapsible::Collapsible;
pub use color_picker::{ColorInput, ColorPicker};
pub use content_area::{ContentArea, content_area};
pub use date_input::DateInput;
pub use date_range_input::DateRangeInput;
pub use divider::{Divider, DividerTone, DividerVariant, LabelPosition, divider};
pub use dropdown::{dropdown, dropdown_disabled, menu_item, menu_item_disabled};
pub use elevated::{Elevated, card_shadow};
pub use extra::{
    bottom_sheet, bottom_sheet_in_window, breakpoint_bin, carousel_indicator_dots,
    carousel_indicator_lines, inline_view_switcher, password_entry_row, path_bar, shortcut_label,
    sidebar, tab_button, view_switcher, view_switcher_bar, window, wrap_box,
};
pub use field::{Field, FieldLayout, outlined as field_outline};
pub use flex::FlexRow;
pub use floating_indicator::FloatingIndicator;
pub use form::{Form, FormActions, Step, StepIndicator};
pub use forms::{
    Entry, EntryState, TextView, accent_picker, card_button, check_button, color_swatch, entry,
    link_button, password_entry, radio_button, scale, status_swatch, tag, text_view,
};
pub use header_bar::{HeaderBar, WindowControls, header_bar, window_title};
pub use iced::widget::text_editor::{Action as TextAction, Content as TextContent};
pub use infinite::{InfiniteList, LoadTrigger};
pub use menu::{Menu, MenuButton, MenuItem, MenuPresentation, menu_button, primary_menu};
pub use meta_component::MetaComponent;
pub use meta_parent_component::{MetaParentComponent, Reporter};
pub use motion::{
    DrawerLayer, Entrance, Highlight, PageTransition, PushStack, Reveal, drawer_layer, entrance,
    highlight, layered, page_transition, push_stack, reveal,
};
pub use multi_select::MultiSelect;
pub use navigation::{
    Breakpoint, COLLAPSE_WIDTH, CollapsedSidebar, DRAWER_MAX_FRACTION, MIN_CONTENT_WIDTH,
    NavigationPage, NavigationSplitView, OverlaySplitView, SIDEBAR_FRACTION, SIDEBAR_MAX_WIDTH,
    SIDEBAR_MIN_WIDTH, SidebarItem, SidebarList, SidebarPadding, SidebarStyle, collapse_width,
    min_content_width, navigation_sidebar, navigation_sidebar_rounded,
    navigation_sidebar_rounded_padded, navigation_view, sidebar_row, sidebar_row_padded,
    sidebar_width_for, split_collapsed,
};
pub use number_input::NumberInput;
pub use overlay::{Layer as OverlayLayer, Overlay, stacked as overlay_stack};
pub use pagination::label as pagination_label;
pub use pagination::{PageSize, Pagination, PaginationSize, page_size_dropdown};
pub use popover::{
    Align as PopoverAlign, Appearance as PopoverAppearance, Popover, PopoverBounds,
    Presentation as PopoverPresentation, popover, popover_bounds,
};
pub use preferences::{PreferencesGroup, PreferencesPage, preferences_group, preferences_page};
pub use range_slider::{RangeSlider, range_slider};
pub use reorderable_list::ReorderableList;
pub use resizable::Resizable;
pub use responsive_table::{
    Column, Mode as TableMode, Presentation as TablePresentation, ResponsiveTable,
};
pub use rows::{
    ActionRow, ButtonRow, ComboRow, EntryRow, ExpanderRow, RangeSliderRow, SliderRow, SpinRow,
    SwitchRow, ToggleGroupRow, action_row, boxed_list,
};
pub use scrolled::{scrollbar, scrolled_window, scrolled_window_both, scrolled_window_horizontal};
pub use sections::{SectionButton, SectionIcon, section_button, section_icon, section_size};
pub use shortcut_scope::shortcuts;
pub use spinner::{Spinner, spinner};
pub use split::{SplitAction, SplitButton, SplitButtonAction, split_button};
pub use split_view::{AdaptiveSplit, adaptive_split};
pub use splitter::{Splitter, SplitterOrientation};
pub use spoiler::{Spoiler, SpoilerList};
pub use status_page::{Card, Page, StatusPage, bin, card, clamp, page, status_page};
pub use stepper::{Stepper, StepperIconPosition, StepperOrientation, StepperSize, StepperStep};
pub use switch::{Switch, switch};
pub use tabs::{TabItem, Tabs, TabsVariant};
pub use timeline::{Timeline, TimelineAlign, TimelineItem};
pub use toast::{
    Toast, ToastPosition, ToastTarget, toast_overlay, toast_stack, toast_stack_from, toast_target,
};
pub use toolbar::{ActionBar, BarStyle, ToolbarView, action_bar, toolbar_view};
pub use tree::{DragMode, Payload as TreeNodePayload, ROW_HEIGHT as TREE_ROW_HEIGHT, Tree};
pub use tree_row::TreeRow;

/// Shorthand for an iced element using this crate's [`Theme`](crate::Theme).
pub type Element<'a, Message> = iced::Element<'a, Message, crate::Theme>;
