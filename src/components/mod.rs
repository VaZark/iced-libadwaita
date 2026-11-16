//! Components: themed, standalone UI elements that carry no information on their own.
//!
//! Each pulls [primitives](crate::primitives) (rules) and [binds](crate::binds) (event glue) and
//! composes them with its look. Widgets and third-party widgets are built from these.
//!

pub(crate) mod avatar;
pub(crate) mod badge;
pub(crate) mod bevel;
pub(crate) mod blocks;
pub(crate) mod button;
pub(crate) mod child_props;
pub(crate) mod classic;
pub(crate) mod classic_scroll;
pub(crate) mod controls;
pub(crate) mod divider;
pub(crate) mod elevated;
pub(crate) mod entry;
pub(crate) mod icon_toggle;
pub(crate) mod input_wrapper;
pub(crate) mod pixels;
pub mod props;
pub(crate) mod range_slider;
pub(crate) mod scale;
pub(crate) mod search_bar;
pub(crate) mod sections;
pub(crate) mod spectrum;
pub(crate) mod spinner;
pub(crate) mod switch;
pub(crate) mod theme_toggle;
pub(crate) mod tooltip;
pub mod typography;

pub use avatar::{Avatar, AvatarShape, avatar};
pub use badge::{Badge, Chip, ChipSize, RoundBadge};
pub use bevel::{Bevel, bevel_of, framed};
pub use blocks::{ProgressBlocks, blocks_style, themed_progress};
pub use button::{
    ButtonContent, ButtonVariant, CircularButton, SuggestedCircularButton, ToggleGroup, ToggleItem,
    circular_button, destructive_button, flat_button, icon_button, pill_button, progress_bar,
    raised_button, suggested_button, suggested_circular_button, suggested_pill_button,
    toggle_group, variant_button,
};
pub use classic::{check_map, disabled_label, drawn_marks, radio_map, themed_check, themed_radio};
pub use classic_scroll::{ClassicScroll, classic_scrollbar, themed_scrolled};
pub use controls::{
    check_button, color_swatch, link_button, radio_button, scale, status_swatch, tag,
};
pub use divider::{Divider, DividerTone, DividerVariant, LabelPosition, divider};
pub use elevated::{Elevated, card_shadow};
pub use entry::{Entry, EntryState, TextView, entry, password_entry, text_view};
pub use iced::widget::text_editor::{Action as TextAction, Content as TextContent};
pub use icon_toggle::IconToggleButton;
pub use pixels::Pixels;
pub use props::TextOverflow;
pub use props::{ButtonProps, ContainerProps, EntryProps, TextProps};
pub use range_slider::{RangeSlider, RangeSliderProps, range_slider};
pub use scale::{Scale, ScaleProps};
pub use search_bar::{SearchBar, search_bar};
pub use sections::{SectionButton, SectionIcon, section_button, section_icon, section_size};
pub use spectrum::{SpectrumBars, spectrum_bars};
pub use spinner::{Spinner, spinner};
pub use switch::{Switch, switch};
pub use theme_toggle::{ThemeMode, ThemeToggle, ThemeToggleProps};
pub use tooltip::{TooltipPosition, tooltip};

/// Label, description and validation wrapper for custom input components.
pub use input_wrapper::{InputWrapper, InputWrapperLayout, outlined as input_outline};

// Native presentation props are owned by the component they configure.
pub use avatar::AvatarProps;
pub use badge::{BadgeProps, ChipProps, RoundBadgeProps};
pub use button::{ButtonContentProps, CircularButtonProps, ToggleGroupProps};
pub use controls::{
    ColorSwatchProps, LinkButtonProps, TagProps, color_swatch_with_props, link_button_with_props,
    status_swatch_with_props, tag_with_props,
};
pub use entry::{EntryInputProps, TextEditorProps, TextInputProps, TextViewProps};
pub use icon_toggle::IconToggleButtonProps;
pub use input_wrapper::InputWrapperProps;
pub use search_bar::SearchBarProps;
pub use sections::{SectionButtonProps, SectionIconProps};
pub use tooltip::{TooltipProps, tooltip_with_props};
