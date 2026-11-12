//! Widgets: what the user sees and interacts with. Composed from components, layouts and primitives, with
//! widget-defined events, an internal layout, and a reaction to the global density.
//!
//! Each widget is a small builder that turns into an [`Element`](crate::Element). Colors and states are
//! resolved from the [`Theme`](crate::Theme) at draw time; sizes come from [`Metrics`](crate::Metrics) when the
//! view is built, so use `.metrics(..)` on a widget to deviate from the defaults.

pub(crate) mod accent_picker;
pub(crate) mod accordion;
pub(crate) mod autocomplete;
pub(crate) mod banner;
pub(crate) mod carousel;
pub(crate) mod chart;
pub(crate) mod chip_group;
pub(crate) mod collapsible;
pub(crate) mod color_picker;
pub(crate) mod date_input;
pub(crate) mod date_range_input;
pub(crate) mod dialog;
pub(crate) mod drag;
pub(crate) mod dropdown;
pub(crate) mod entries;
pub(crate) mod extra;
pub(crate) mod field;
pub(crate) mod floating_indicator;
pub(crate) mod form;
pub(crate) mod header_bar;
pub(crate) mod histogram_range_slider;
pub(crate) mod infinite;
pub(crate) mod item_card;
pub(crate) mod item_row;
pub(crate) mod menu;
pub(crate) mod menu_pages;
pub(crate) mod multi_select;
pub(crate) mod navigation;
pub(crate) mod number_input;
pub(crate) mod pagination;
pub(crate) mod popover;
pub(crate) mod preferences;
pub(crate) mod profile_header;
pub(crate) mod range_slider;
pub(crate) mod rich_text;
pub(crate) mod tag_group_list;
pub(crate) mod reorderable_list;
pub(crate) mod resizable;
pub(crate) mod responsive_table;
pub(crate) mod rows;
pub(crate) mod section_divider;
pub(crate) mod section_heading;
pub(crate) mod segmented;
pub(crate) mod sort_header;
pub(crate) mod split;
pub(crate) mod spoiler;
pub(crate) mod status_page;
pub(crate) mod stepper;
pub(crate) mod synced_text;
pub(crate) mod tabs;
mod tag_filter;
pub(crate) mod timeline;
pub(crate) mod toast;
pub(crate) mod toolbar;
pub(crate) mod tree;
pub(crate) mod tree_row;

pub use accent_picker::{AccentPicker, accent_picker};
pub use accordion::{Accordion, AccordionItem};
pub use autocomplete::Autocomplete;
pub use banner::{Banner, banner};
pub use carousel::{Carousel, CarouselIndicator, carousel};
pub use chart::{
    BarChart, BarLayout, Funnel, FunnelShape, LineChart, PieChart, Sankey, Sunburst, format_value,
};
pub use chart::{BubbleChart, BubblePoint, BubbleSeries, ComboChart, DonutChart};
pub use chip_group::{ChipGroup, ChipOption};
pub use collapsible::Collapsible;
pub use color_picker::{ColorInput, ColorPicker};
pub use date_input::DateInput;
pub use date_range_input::DateRangeInput;
pub use dialog::{
    AboutDialog, AlertDialog, ResponseAppearance, dialog, dialog_in_window, preferences_dialog,
    present_dialog, shortcuts_dialog,
};
pub use drag::{DragSource, DropZone, drag_source, drop_zone};
pub use dropdown::{DROPDOWN_MAX_WIDTH, DROPDOWN_MIN_WIDTH, DropdownOptions, dropdown, dropdown_disabled, dropdown_lead, dropdown_with, menu_item, menu_item_disabled, menu_item_lead};
pub use extra::{
    breakpoint_bin, carousel_indicator_dots, carousel_indicator_lines, inline_view_switcher,
    path_bar, shortcut_label, sidebar, tab_button, view_switcher, view_switcher_bar, wrap_box,
};
pub use field::{Field, FieldLayout, outlined as field_outline};
pub use floating_indicator::FloatingIndicator;
pub use form::{Form, FormActions, Step, StepIndicator};
pub use header_bar::{HeaderBar, WindowControlIcons, WindowControls, header_bar, window_title};
pub use histogram_range_slider::HistogramRangeSlider;
pub use infinite::{InfiniteList, LoadTrigger};
pub use item_card::ItemCard;
pub use item_row::{ItemColumns, ItemRow, PICTURE_WIDTH, SUFFIX_WIDTH, TIME_WIDTH};
pub use menu::{
    Menu, MenuButton, MenuCheckStyle, MenuIconSide, MenuItem, MenuPresentation, menu_button,
    primary_menu,
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
pub use number_input::{NumberDrag, NumberInput};
pub use pagination::label as pagination_label;
pub use pagination::{PageSize, Pagination, PaginationSize, page_size_dropdown};
pub use popover::{
    Align as PopoverAlign, Appearance as PopoverAppearance, Popover, PopoverBounds,
    Presentation as PopoverPresentation, popover, popover_bounds,
};
pub use preferences::{PreferencesGroup, PreferencesPage, preferences_group, preferences_page};
pub use profile_header::ProfileHeader;
pub use range_slider::{RangeSlider, range_slider};
pub use rich_text::RichText;
pub use tag_group_list::{TagGroupList, TagItem};
pub use reorderable_list::ReorderableList;
pub use resizable::Resizable;
pub use responsive_table::{
    Column, Mode as TableMode, Presentation as TablePresentation, ResponsiveTable,
};
pub use rows::{
    ActionRow, ButtonRow, ComboRow, ExpanderRow, RangeSliderRow, SliderRow, SpinRow, SwitchRow,
    ToggleGroupRow, action_row, boxed_list,
};
pub use section_divider::{CollapseState, SectionDivider};
pub use section_heading::SectionHeading;
pub use sort_header::SortHeader;
pub use split::{SplitAction, SplitButton, SplitButtonAction, split_button};
pub use spoiler::{Spoiler, SpoilerList};
pub use status_page::{Card, Page, StatusPage, bin, card, clamp, page, status_page};
pub use stepper::{Stepper, StepperIconPosition, StepperOrientation, StepperSize, StepperStep};
pub use synced_text::{SyncedText, follow_synced_text};
pub use tabs::{TabItem, Tabs, TabsVariant};
pub use tag_filter::TagFilter;
pub use timeline::{Timeline, TimelineAlign, TimelineItem};
pub use toast::{
    Toast, ToastPosition, ToastTarget, toast_overlay, toast_stack, toast_stack_from, toast_target,
};
pub use toolbar::{ActionBar, BarStyle, ToolbarView, action_bar, toolbar_view};
pub use tree::{DragMode, Payload as TreeNodePayload, ROW_HEIGHT as TREE_ROW_HEIGHT, Tree};
pub use tree_row::TreeRow;

pub use entries::{EntryInput, EntryRow};

pub use crate::components::InputWrapper;

pub(crate) mod theme_toggle;
pub use theme_toggle::{ThemeMode, ThemeToggle};

mod scale;
pub use scale::Scale;

mod toast_timeout;
