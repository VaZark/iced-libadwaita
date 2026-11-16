//! Downstream-only API coverage. No private modules or source-text assertions.
//! See docs/lib/child-props.md for scope, exceptions, and unresolved core gaps.

use iad::components::{self as c, ButtonProps, ContainerProps, EntryProps, TextProps};
use iad::{Element, Theme, widgets as w};
use iced::advanced::{Shell, clipboard, layout, renderer::Headless, widget::Tree};
use iced::widget::{button, container, space, text};
use iced::{Event, Length, Point, Rectangle, Size, mouse};
use iced_adaptive_design as iad;

fn renderer() -> iced::Renderer {
    iced::futures::executor::block_on(iced::Renderer::new(
        iced::Font::DEFAULT,
        iced::Pixels(14.0),
        Some("tiny-skia"),
    ))
    .expect("headless software renderer")
}

fn layout<M>(element: &mut Element<'_, M>, renderer: &iced::Renderer) -> (Tree, layout::Node) {
    let mut tree = Tree::new(element.as_widget());
    let node = element.as_widget_mut().layout(
        &mut tree,
        renderer,
        &layout::Limits::new(Size::ZERO, Size::new(600.0, 400.0)),
    );
    (tree, node)
}

fn contains_size(node: &layout::Node, size: Size) -> bool {
    node.size() == size
        || node
            .children()
            .iter()
            .any(|child| contains_size(child, size))
}

fn sizes(node: &layout::Node) -> Vec<Size> {
    let mut result = vec![node.size()];
    for child in node.children() {
        result.extend(sizes(child));
    }
    result
}

#[test]
fn nine_widget_aggregates_preserve_custom_defaults_and_merge_partial_text_fields() {
    let renderer = renderer();
    macro_rules! check {
        ($make:expr, $props:path, $field:ident) => {{
            let make = $make;
            let baseline = || {
                make().props(<$props>::default().$field(TextProps::default().width(137).height(29)))
            };
            let mut base: Element<'_, ()> = baseline().into();
            let mut default: Element<'_, ()> = <$props>::default().apply(baseline()).into();
            let mut empty: Element<'_, ()> = <$props>::default()
                .$field(TextProps::default())
                .apply(baseline())
                .into();
            let mut partial: Element<'_, ()> = <$props>::default()
                .$field(TextProps::default().height(37))
                .apply(baseline())
                .into();
            let (_, a) = layout(&mut base, &renderer);
            let (_, b) = layout(&mut default, &renderer);
            let (_, c) = layout(&mut empty, &renderer);
            let (_, d) = layout(&mut partial, &renderer);
            assert!(
                contains_size(&a, Size::new(137.0, 29.0)),
                "{} baseline",
                stringify!($props)
            );
            assert_eq!(sizes(&a), sizes(&b), "{} default", stringify!($props));
            assert_eq!(
                sizes(&a),
                sizes(&c),
                "{} explicit empty child",
                stringify!($props)
            );
            assert!(
                contains_size(&d, Size::new(137.0, 37.0)),
                "{} partial overlay",
                stringify!($props)
            );
        }};
    }
    check!(
        || w::Form::<()>::new().title("Form"),
        w::FormWidgetProps,
        title
    );
    let state = iad::primitives::infinite::Infinite::<(), ()>::new(());
    check!(
        || w::InfiniteList::<()>::new(space(), &state)
            .trigger(w::LoadTrigger::LoadMore)
            .on_load_more(())
            .scrolled(false),
        w::InfiniteListWidgetProps,
        load_label
    );
    let items = [1];
    check!(
        || w::ReorderableList::<_, ()>::new(&items, |_| space().height(40).into()).title("Items"),
        w::ReorderableListWidgetProps,
        title
    );
    check!(
        || w::Spoiler::<()>::new(space().height(300)).max_height(80.0),
        w::SpoilerWidgetProps,
        toggle_label
    );
    check!(
        || w::SyncedText::new("empty", &[], |_| ()),
        w::SyncedTextWidgetProps,
        placeholder
    );
    check!(
        || w::Timeline::<()>::new(0).item(w::TimelineItem::new("Created")),
        w::TimelineWidgetProps,
        title
    );
    check!(
        || w::Stepper::<()>::new()
            .collapse(false)
            .step(w::StepperStep::new("Account")),
        w::StepperWidgetProps,
        label
    );
    let rows = [()];
    check!(
        || w::ResponsiveTable::<(), ()>::new(&rows)
            .column(w::Column::new("Name", |_| space().into())),
        w::ResponsiveTableWidgetProps,
        caption
    );
    let theme = Theme::light();
    let blocks = [iad::primitives::rich_text::Block::CodeBlock {
        lang: Some("Rust".into()),
        code: "let x = 1;".into(),
    }];
    check!(
        || w::RichText::<()>::new(&theme, &blocks),
        w::RichTextWidgetProps,
        code
    );
}

#[test]
fn optional_row_and_card_overlays_preserve_omitted_child_dimensions() {
    let renderer = renderer();
    let make = || {
        w::ActionRow::<()>::new("Title")
            .subtitle("Subtitle")
            .label_props(TextProps::default().width(137).height(29))
            .subtitle_props(TextProps::default().width(151).height(31))
    };
    for props in [
        w::ActionRowProps::default(),
        w::ActionRowProps::default().label_props(TextProps::default()),
    ] {
        let mut before: Element<'_, ()> = make().into();
        let mut after: Element<'_, ()> = props.apply(make()).into();
        assert_eq!(
            sizes(&layout(&mut before, &renderer).1),
            sizes(&layout(&mut after, &renderer).1)
        );
    }
    let mut changed: Element<'_, ()> = w::ActionRowProps::default()
        .label_props(TextProps::default().height(37))
        .apply(make())
        .into();
    let (_, node) = layout(&mut changed, &renderer);
    assert!(contains_size(&node, Size::new(137.0, 37.0)));
    assert!(
        contains_size(&node, Size::new(151.0, 31.0)),
        "sibling subtitle is retained"
    );

    let card = w::Card::new(space().width(10).height(10))
        .child_props(ContainerProps::default().width(137).height(29));
    let card = w::CardProps::default().apply(card);
    let mut changed: Element<'_, ()> = w::CardProps::default()
        .child_props(ContainerProps::default().height(37))
        .apply(card)
        .into();
    let (_, node) = layout(&mut changed, &renderer);
    assert!(contains_size(&node, Size::new(137.0, 37.0)));
}

#[test]
fn empty_optional_presentation_does_not_add_page_wrappers() {
    let renderer = renderer();
    let make = || w::Page::<()>::new(space().width(137).height(29));
    let mut before: Element<'_, ()> = make().into();
    let mut default: Element<'_, ()> = w::PageProps::default().apply(make()).into();
    let mut explicit_empty: Element<'_, ()> = w::PageProps::default()
        .content_props(ContainerProps::default())
        .apply(make())
        .into();
    let a = sizes(&layout(&mut before, &renderer).1);
    assert_eq!(a, sizes(&layout(&mut default, &renderer).1));
    assert_eq!(
        a,
        sizes(&layout(&mut explicit_empty, &renderer).1),
        "empty optional props must preserve wrapper absence"
    );
}

fn click_sized<M>(mut element: Element<'_, M>, size: Size) -> Vec<M> {
    let renderer = renderer();
    let (mut tree, node) = layout(&mut element, &renderer);
    fn find(layout: layout::Layout<'_>, size: Size) -> Option<Rectangle> {
        if layout.bounds().size() == size {
            Some(layout.bounds())
        } else {
            layout.children().find_map(|child| find(child, size))
        }
    }
    let target = find(layout::Layout::new(&node), size).expect("retained native button geometry");
    let mut messages = Vec::new();
    for event in [
        mouse::Event::ButtonPressed(mouse::Button::Left),
        mouse::Event::ButtonReleased(mouse::Button::Left),
    ] {
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(event),
            layout::Layout::new(&node),
            mouse::Cursor::Available(target.center()),
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &Rectangle::with_size(Size::new(600.0, 400.0)),
        );
    }
    messages
}

#[test]
fn aggregate_overlays_preserve_submit_busy_policy_and_row_actions() {
    for busy in [false, true] {
        let form = w::Form::new()
            .on_submit(7u8)
            .submitting(busy)
            .enter_submits(false)
            .props(
                w::FormWidgetProps::default()
                    .primary_button(ButtonProps::default().width(137).height(43)),
            );
        let form = w::FormWidgetProps::default().apply(form);
        let form = w::FormWidgetProps::default()
            .title(TextProps::default().size(21))
            .apply(form);
        assert_eq!(
            click_sized(form.into(), Size::new(137.0, 43.0)),
            if busy { vec![] } else { vec![7] }
        );
    }
    let row = w::ActionRow::new("Retained action")
        .on_activate(9u8)
        .button_props(ButtonProps::default().width(137).height(43));
    let row = w::ActionRowProps::default().apply(row);
    let row = w::ActionRowProps::default()
        .button_props(ButtonProps::default().height(47))
        .apply(row);
    assert_eq!(click_sized(row.into(), Size::new(137.0, 47.0)), vec![9]);
}

#[test]
fn theme_toggle_and_entry_row_children_are_public_native_props() {
    let renderer = renderer();
    let toggle = c::ThemeToggle::new(c::ThemeMode::System)
        .button_props(ButtonProps::default().width(97).height(43))
        .on_toggle(|_| ());
    let props: w::ThemeToggleProps = c::ThemeToggleProps::default()
        .button_props(ButtonProps::default().height(49))
        .container_props(ContainerProps::default().clip(true))
        .tooltip_props(
            c::TooltipProps::default()
                .label_props(TextProps::default().overflow(c::TextOverflow::Wrap)),
        );
    let mut element: Element<'_, ()> = props.apply(toggle).into();
    let (_, node) = layout(&mut element, &renderer);
    assert_eq!(node.size(), Size::new(97.0, 49.0));

    let row = w::EntryRow::new(
        "Team",
        w::EntryInput::Select {
            value: "Selected".into(),
            options: text("Option").into(),
            multiple: false,
        },
    )
    .value_props(TextProps::default().width(173));
    let row = w::EntryRowProps::default()
        .value_props(TextProps::default().height(37))
        .wrapper_props(
            c::InputWrapperProps::default()
                .description_props(TextProps::default().overflow(c::TextOverflow::Wrap)),
        )
        .text_view_props(
            c::TextViewProps::default().editor_props(c::TextEditorProps::default().padding(0)),
        )
        .apply(row);
    let mut element: Element<'_, ()> = row.into();
    let (_, node) = layout(&mut element, &renderer);
    assert!(contains_size(&node, Size::new(173.0, 37.0)));
}

#[test]
fn exported_owned_props_are_default_and_clone_from_downstream() {
    // An export smoke test, not evidence that every field reaches every branch.
    macro_rules! exported {
        ($($ty:path),+ $(,)?) => { $(let _ = <$ty>::default().clone();)+ };
    }
    exported!(
        c::TextProps,
        c::ContainerProps,
        c::ButtonProps,
        c::EntryProps,
        c::AvatarProps,
        c::BadgeProps,
        c::ChipProps,
        c::RoundBadgeProps,
        c::ButtonContentProps,
        c::CircularButtonProps,
        c::ToggleGroupProps,
        c::ColorSwatchProps,
        c::LinkButtonProps,
        c::TagProps,
        c::EntryInputProps,
        c::TextInputProps,
        c::TextEditorProps,
        c::TextViewProps,
        c::IconToggleButtonProps,
        c::ThemeToggleProps,
        w::ThemeToggleProps,
        c::InputWrapperProps,
        c::SearchBarProps,
        c::SectionButtonProps,
        c::SectionIconProps,
        c::TooltipProps,
        c::ScaleProps,
        c::RangeSliderProps,
        w::AccentPickerProps,
        w::AccordionItemProps,
        w::AccordionProps,
        w::BannerProps,
        w::ChipGroupProps,
        w::ChoiceDropdownProps,
        w::ChoiceToggleProps,
        w::CollapsibleProps,
        w::ColorInputProps,
        w::ColorPickerProps,
        w::ContextMenuProps,
        w::MenuProps,
        w::DateInputProps,
        w::DateRangeInputProps,
        w::AboutDialogProps,
        w::AlertDialogProps,
        w::DialogFrameProps,
        w::ShortcutsDialogProps,
        w::DropdownItemProps,
        w::EntryRowProps,
        w::CarouselIndicatorWidgetProps,
        w::PathBarWidgetProps,
        w::ShortcutLabelWidgetProps,
        w::TabButtonWidgetProps,
        w::ViewSwitcherBarWidgetProps,
        w::FieldProps,
        w::FormWidgetProps,
        w::HeaderBarProps,
        w::IconPickerProps,
        w::InfiniteListWidgetProps,
        w::ItemCardProps,
        w::ItemRowProps,
        w::SidebarRowProps,
        w::NumberInputProps,
        w::PopoverProps,
        w::PopoverSurfaceProps,
        w::PreferencesGroupProps,
        w::PreferencesPageProps,
        w::ProfileHeaderProps,
        w::RangeSliderProps,
        w::ReorderableListWidgetProps,
        w::ResponsiveTableWidgetProps,
        w::RichTextLineProps,
        w::RichTextWidgetProps,
        w::ActionRowProps,
        w::ComboRowProps,
        w::ToggleGroupRowProps,
        w::ScaleProps,
        w::SectionDividerProps,
        w::SectionHeadingProps,
        w::SpoilerWidgetProps,
        w::CardProps,
        w::ClampProps,
        w::PageProps,
        w::StatusPageProps,
        w::StepperWidgetProps,
        w::SyncedTextWidgetProps,
        w::TagFilterProps,
        w::TagGroupListProps,
        w::TimelineWidgetProps,
        w::ToastProps,
        w::ToolbarProps,
        w::TreeProps,
    );
}

#[test]
fn text_props_cover_native_builders_and_owned_styles() {
    let props = TextProps::default()
        .size(19)
        .line_height(iced::widget::text::LineHeight::Relative(1.5))
        .font(iced::Font::DEFAULT)
        .width(123)
        .height(37)
        .align_x(iced::widget::text::Alignment::Center)
        .align_y(iced::alignment::Vertical::Center)
        .shaping(iced::widget::text::Shaping::Advanced)
        .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
        .style(|_: &Theme| iced::widget::text::Style::default())
        .color(iced::Color::BLACK);
    let element: Element<'_, ()> = props
        .clone()
        .apply_to_text(text("日本語 🦀 e\u{301}"))
        .into();
    assert_eq!(element.as_widget().size(), Size::new(123.into(), 37.into()));
    assert!(Tree::new(&element).children.is_empty());
    // The widget retains an owned callback, not a borrow of the props builder.
    drop(props);
    let _: Element<'_, ()> = element;
}

#[test]
fn text_defaults_preserve_leaf_and_overflow_is_explicit() {
    let renderer = renderer();
    let label = "界🦀 e\u{301} ".repeat(80);
    let unchanged: Element<'_, ()> = TextProps::default()
        .apply_to_text(text(&label).width(91).height(29))
        .into();
    assert_eq!(
        unchanged.as_widget().size(),
        Size::new(91.into(), 29.into())
    );
    assert!(Tree::new(&unchanged).children.is_empty());

    let mut heights = Vec::new();
    for policy in [
        c::TextOverflow::Wrap,
        c::TextOverflow::Clip,
        c::TextOverflow::Grow,
    ] {
        let mut element: Element<'_, ()> = TextProps::default()
            .width(90)
            .overflow(policy)
            .apply(text(&label));
        let (_, node) = layout(&mut element, &renderer);
        heights.push(node.size().height);
        if policy != c::TextOverflow::Grow {
            assert_eq!(node.size().width, 90.0);
        }
    }
    assert!(
        heights[0] > heights[1],
        "wrapping must change actual layout"
    );
    assert_eq!(heights[1], heights[2]);
}

#[test]
fn native_props_preserve_actions_and_forward_geometry() {
    let renderer = renderer();
    let mut element: Element<'_, u8> = ButtonProps::default()
        .width(127)
        .height(43)
        .padding(3)
        .clip(true)
        .style(|_, _| iced::widget::button::Style::default())
        .apply(button(text("Keep action")).on_press(7))
        .into();
    let (mut tree, node) = layout(&mut element, &renderer);
    assert_eq!(node.size(), Size::new(127.0, 43.0));
    let point = Point::new(20.0, 20.0);
    let viewport = Rectangle::with_size(Size::new(600.0, 400.0));
    let mut messages = Vec::new();
    for event in [
        mouse::Event::ButtonPressed(mouse::Button::Left),
        mouse::Event::ButtonReleased(mouse::Button::Left),
    ] {
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(event),
            layout::Layout::new(&node),
            mouse::Cursor::Available(point),
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &viewport,
        );
    }
    assert_eq!(messages, vec![7]);

    let element: Element<'_, ()> = ContainerProps::default()
        .width(151)
        .height(53)
        .max_width(200)
        .max_height(100)
        .padding(2)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .clip(true)
        .style(|_| iced::widget::container::Style::default())
        .apply(container(text("Caller content")))
        .into();
    assert_eq!(element.as_widget().size(), Size::new(151.into(), 53.into()));
}

#[test]
fn entry_props_forward_native_input_and_defaults_do_not_replace_it() {
    let make = || {
        c::Entry::<String>::new("Hint", "Retained value")
            .on_input(|value| value)
            .input_props(c::TextInputProps::default().width(173))
    };
    let unchanged: Element<'_, String> = EntryProps::default().apply(make()).into();
    assert_eq!(unchanged.as_widget().size().width, Length::Fixed(173.0));
    let changed: Element<'_, String> = EntryProps::default()
        .input_props(c::TextInputProps::default().width(211).size(18).padding(4))
        .apply(make())
        .into();
    assert_eq!(changed.as_widget().size().width, Length::Fixed(211.0));
}

#[test]

fn entry_input_props_default_should_preserve_existing_nested_presentation() {
    let element: Element<'_, ()> = c::EntryInputProps::default()
        .apply(c::Entry::new("Hint", "Value").input_props(c::TextInputProps::default().width(173)))
        .into();
    assert_eq!(element.as_widget().size().width, Length::Fixed(173.0));
}

#[test]
fn search_bar_forwards_to_real_input_close_button_and_surface() {
    let renderer = renderer();
    let props = c::SearchBarProps::default()
        .input_props(c::TextInputProps::default().width(137))
        .button_props(ButtonProps::default().width(47).height(39))
        .container_props(ContainerProps::default().width(301).height(61));
    let mut element: Element<'_, String> = props
        .apply(c::search_bar("Search value", |value| value).on_close("Close".into()))
        .into();
    let (_, node) = layout(&mut element, &renderer);
    assert_eq!(node.size(), Size::new(301.0, 61.0));
    assert!(
        contains_size(&node, Size::new(47.0, 39.0)),
        "close-button props must reach the nested button"
    );
    fn has_width(node: &layout::Node, width: f32) -> bool {
        node.size().width == width || node.children().iter().any(|child| has_width(child, width))
    }
    assert!(
        has_width(&node, 137.0),
        "input props must reach the nested input"
    );
}

#[test]
fn entry_aggregate_partial_and_empty_input_overrides_keep_existing_width() {
    for input in [
        c::TextInputProps::default(),
        c::TextInputProps::default().size(21),
    ] {
        let make = || {
            c::Entry::<()>::new("Hint", "Secret")
                .password(true)
                .input_props(c::TextInputProps::default().width(173).padding(7))
        };
        let element: Element<'_, ()> = c::EntryInputProps::default()
            .input_props(input.clone())
            .apply(make())
            .into();
        assert_eq!(element.as_widget().size().width, Length::Fixed(173.0));
        let element: Element<'_, ()> = EntryProps::default()
            .input_props(input)
            .apply(make())
            .into();
        assert_eq!(element.as_widget().size().width, Length::Fixed(173.0));
    }
}

#[test]
fn search_bar_aggregate_merges_individual_fields_and_preserves_close_action() {
    let renderer = renderer();
    for partial in [false, true] {
        let original = c::search_bar("Retained", |_: String| 1u8)
            .on_close(7)
            .input_props(c::TextInputProps::default().width(137).padding(6))
            .button_props(ButtonProps::default().width(47).height(39))
            .container_props(ContainerProps::default().width(301).height(71));
        let overrides = if partial {
            c::SearchBarProps::default()
                .input_props(c::TextInputProps::default().size(19))
                .button_props(ButtonProps::default().height(41))
                .container_props(ContainerProps::default().height(73))
        } else {
            c::SearchBarProps::default()
        };
        let mut element: Element<'_, u8> = overrides.apply(original).into();
        let (mut tree, node) = layout(&mut element, &renderer);
        assert_eq!(
            node.size(),
            Size::new(301.0, if partial { 73.0 } else { 71.0 })
        );
        fn find(layout: layout::Layout<'_>, size: Size) -> Option<Rectangle> {
            if layout.bounds().size() == size {
                Some(layout.bounds())
            } else {
                layout.children().find_map(|child| find(child, size))
            }
        }
        fn has_input_width(node: &layout::Node) -> bool {
            node.size().width == 137.0 || node.children().iter().any(has_input_width)
        }
        assert!(has_input_width(&node));
        let close = find(
            layout::Layout::new(&node),
            Size::new(47.0, if partial { 41.0 } else { 39.0 }),
        )
        .expect("omitted button width survives the aggregate overlay");
        let mut messages = Vec::new();
        for event in [
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
        ] {
            element.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(event),
                layout::Layout::new(&node),
                mouse::Cursor::Available(close.center()),
                &renderer,
                &mut clipboard::Null,
                &mut Shell::new(&mut messages),
                &Rectangle::with_size(Size::new(600.0, 400.0)),
            );
        }
        assert_eq!(messages, vec![7]);
    }
}

#[test]
fn text_aggregate_overlays_preserve_omitted_dimensions() {
    let renderer = renderer();
    let baseline = TextProps::default().width(113).height(31).size(17);
    let content = c::ButtonContent::new("Retained label").label_props(baseline.clone());
    let content = c::ButtonContentProps::default().apply(content);
    let mut element: Element<'_, ()> = c::ButtonContentProps::default()
        .label_props(TextProps::default().height(37))
        .apply(content)
        .into();
    let (_, node) = layout(&mut element, &renderer);
    assert!(contains_size(&node, Size::new(113.0, 37.0)));

    let wrapper = c::InputWrapper::<()>::new("Label", space())
        .description("Description")
        .label_props(baseline.clone())
        .description_props(TextProps::default().width(127).height(29));
    let wrapper = c::InputWrapperProps::default().apply(wrapper);
    let mut element: Element<'_, ()> = c::InputWrapperProps::default()
        .label_props(TextProps::default().height(37))
        .description_props(TextProps::default().size(18))
        .apply(wrapper)
        .into();
    let (_, node) = layout(&mut element, &renderer);
    assert!(contains_size(&node, Size::new(113.0, 37.0)));
    assert!(contains_size(&node, Size::new(127.0, 29.0)));
}

#[test]
fn icon_toggle_aggregate_preserves_controlled_state_and_disabled_policy() {
    let renderer = renderer();
    for disabled in [false, true] {
        let toggle = c::IconToggleButton::new(iad::Icon::Search, true)
            .disabled(disabled)
            .on_toggle(|next| next)
            .button_props(ButtonProps::default().width(83).height(43));
        let toggle = c::IconToggleButtonProps::default().apply(toggle);
        let mut element: Element<'_, bool> = c::IconToggleButtonProps::default()
            .button_props(ButtonProps::default().height(47))
            .apply(toggle)
            .into();
        let (mut tree, node) = layout(&mut element, &renderer);
        assert_eq!(node.size(), Size::new(83.0, 47.0));
        let mut messages = Vec::new();
        for event in [
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
        ] {
            element.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(event),
                layout::Layout::new(&node),
                mouse::Cursor::Available(Point::new(20.0, 20.0)),
                &renderer,
                &mut clipboard::Null,
                &mut Shell::new(&mut messages),
                &Rectangle::with_size(Size::new(600.0, 400.0)),
            );
        }
        assert_eq!(messages, if disabled { vec![] } else { vec![false] });
    }
}

#[test]
fn choice_compatibility_names_are_real_native_row_props() {
    let row = w::ActionRowProps::default()
        .label_props(TextProps::default().size(18))
        .subtitle_props(TextProps::default().overflow(c::TextOverflow::Wrap));
    let toggle: w::ToggleGroupRowProps = w::ChoiceToggleProps::default()
        .row_props(row.clone())
        .group_props(c::ToggleGroupProps::default().label_props(TextProps::default().size(17)));
    let dropdown: w::ComboRowProps = w::ChoiceDropdownProps::default()
        .row_props(row.clone())
        .dropdown_props(w::DropdownItemProps::default().label_props(TextProps::default().size(16)));
    let _: Element<'_, &str> =
        w::ChoiceRow::new("Style", &["One", "Two"], Some("One"), |value| value)
            .toggle_props(toggle)
            .dropdown_props(dropdown)
            .row_props(row)
            .into();
    let _: c::InputWrapperProps = w::FieldProps::default();
}

#[test]
fn composed_rows_content_and_workflow_props_build_publicly() {
    let label = TextProps::default()
        .size(18)
        .overflow(c::TextOverflow::Wrap);
    let surface = ContainerProps::default().width(280);
    let action = ButtonProps::default().padding(5);
    let _: Element<'_, ()> = w::HeaderBarProps::default()
        .title_props(label.clone())
        .subtitle_props(label.clone())
        .controls_props(action.clone())
        .container_props(surface.clone())
        .apply(w::HeaderBar::new().title("Title"))
        .into();
    let _: Element<'_, String> = w::NumberInputProps::default()
        .entry_props(EntryProps::default().input_props(c::TextInputProps::default().size(18)))
        .button_props(action.clone())
        .apply(w::NumberInput::new("42", |value| value))
        .into();
    let _: Element<'_, String> = w::EntryRowProps::default()
        .label_props(label.clone())
        .entry_props(EntryProps::default().padding(4))
        .apply(w::EntryRow::new(
            "Name",
            w::EntryInput::Text(c::Entry::new("Hint", "Value").on_input(|value| value)),
        ))
        .into();
    let _: Element<'_, ()> = w::PreferencesGroupProps::default()
        .title_props(label.clone())
        .description_props(label.clone())
        .card_props(w::CardProps::default().child_props(surface.clone()))
        .apply(w::PreferencesGroup::new())
        .into();
    let _: Element<'_, ()> = w::StatusPageProps::default()
        .title_props(label.clone())
        .description_props(label.clone())
        .body_props(surface.clone())
        .apply(w::StatusPage::new())
        .into();
    let _: Element<'_, ()> = w::SectionHeadingProps::default()
        .title_props(label.clone())
        .action_label_props(label.clone())
        .action_props(action.clone())
        .apply(w::SectionHeading::new("Section"))
        .into();
    let _: Element<'_, ()> = w::FormWidgetProps::default()
        .title(label.clone())
        .primary_label(label.clone())
        .primary_button(action.clone())
        .indicator(w::StepperWidgetProps::default().label(label.clone()))
        .apply(w::Form::new().title("Form"))
        .into();
    let _: Element<'_, ()> = w::TreeRow::new("Node")
        .title_props(label)
        .row_props(w::ActionRowProps::default().button_props(action.clone()))
        .handle_button_props(action)
        .children_props(surface)
        .into();
}

#[test]
fn menu_text_trigger_and_item_presentation_build_publicly() {
    let menu = w::Menu::new("Commands")
        .title_props(TextProps::default().size(20))
        .subtitle_props(TextProps::default().size(14))
        .dropdown_props(w::PopoverSurfaceProps::default())
        .section([w::MenuItem::action("Open 日本語", ())
            .accel("Ctrl+O")
            .label_props(TextProps::default().overflow(c::TextOverflow::Wrap))
            .accelerator_props(TextProps::default().size(12))
            .state_props(TextProps::default().size(12))
            .container_props(ContainerProps::default().padding(3))
            .button_props(ButtonProps::default().height(41))]);
    let _: Element<'_, ()> = menu.sheet(Some("Sheet".into()));
    let _: Element<'_, ()> = w::menu_button(iad::Icon::Search, menu)
        .text_trigger("File 日本語 🦀")
        .label_props(
            TextProps::default()
                .size(17)
                .overflow(c::TextOverflow::Wrap),
        )
        .trigger_props(ContainerProps::default().width(133))
        .combobox_props(w::PopoverProps::default().popover_width(240.0))
        .into();
    let _: Element<'_, ()> = w::menu_button(iad::Icon::Search, w::Menu::new("Public aggregate"))
        .text_trigger("Commands")
        .props(w::MenuProps {
            label: TextProps::default().overflow(c::TextOverflow::Wrap),
            trigger: ContainerProps::default().width(133),
            popover: w::PopoverProps::default().popover_width(240.0),
        })
        .into();
}

#[test]
fn menu_sheet_forwards_item_button_geometry() {
    let renderer = renderer();
    let menu = w::Menu::new("Commands")
        .section([w::MenuItem::action("Run", ())
            .button_props(ButtonProps::default().width(117).height(79))]);
    let mut element = menu.sheet(None);
    let (_, node) = layout(&mut element, &renderer);
    assert!(contains_size(&node, Size::new(117.0, 79.0)));
}

#[test]
fn context_menu_surface_props_preserve_target_and_open_a_native_overlay() {
    let renderer = renderer();
    let menu = w::Menu::new("Context")
        .section([w::MenuItem::action("Run", 9u8).label_props(TextProps::default().size(18))]);
    let mut element: Element<'_, u8> = w::context_menu(
        button(text("Target")).width(120).height(40).on_press(3),
        menu,
    )
    .props(w::ContextMenuProps {
        popover: w::PopoverProps::default()
            .padding(30)
            .appearance(w::PopoverAppearance::Button)
            .popover_width(200.0),
    })
    .into();
    let (mut tree, node) = layout(&mut element, &renderer);
    assert_eq!(node.size(), Size::new(120.0, 40.0));
    let viewport = Rectangle::with_size(Size::new(600.0, 400.0));
    let mut messages = Vec::new();
    for event in [
        mouse::Event::ButtonPressed(mouse::Button::Left),
        mouse::Event::ButtonReleased(mouse::Button::Left),
    ] {
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(event),
            layout::Layout::new(&node),
            mouse::Cursor::Available(Point::new(20.0, 20.0)),
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &viewport,
        );
    }
    assert_eq!(
        messages,
        vec![3],
        "context wrapper must preserve the target action"
    );
    element.as_widget_mut().update(
        &mut tree,
        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)),
        layout::Layout::new(&node),
        mouse::Cursor::Available(Point::new(20.0, 20.0)),
        &renderer,
        &mut clipboard::Null,
        &mut Shell::new(&mut messages),
        &viewport,
    );
    assert!(
        element
            .as_widget_mut()
            .overlay(
                &mut tree,
                layout::Layout::new(&node),
                &renderer,
                &viewport,
                iced::Vector::ZERO,
            )
            .is_some(),
        "right click must open a native overlay"
    );
    assert_eq!(messages, vec![3]);
    let _: Element<'_, ()> = w::ContextMenu::new(space(), w::Menu::new("Empty")).into();
}

#[test]
fn moved_scale_and_theme_toggle_keep_component_and_widget_type_identity() {
    let slider: c::RangeSliderProps = w::RangeSliderProps::default().width(213).height(39);
    let props: c::ScaleProps = w::ScaleProps::default().slider_props(slider);
    let scale: w::Scale<'_, f32, f32> = c::scale(0.0..=100.0, 30.0, |value| value);
    let scale: c::Scale<'_, f32, f32> = props.apply(scale);
    let element: Element<'_, f32> = scale.into();
    assert_eq!(element.as_widget().size().width, Length::Fixed(213.0));
    // Height controls the major axis only for a vertical slider.
    let vertical: Element<'_, f32> = props
        .clone()
        .slider_props(
            w::RangeSliderProps::default()
                .orientation(iad::primitives::scrub::Axis::Vertical)
                .height(139),
        )
        .apply(c::Scale::new(0.0..=100.0, 30.0, |value| value))
        .into();
    assert_eq!(vertical.as_widget().size().height, Length::Fixed(139.0));
    let mode: c::ThemeMode = w::ThemeMode::Dark;
    let mode: iad::primitives::appearance::ThemeMode = mode;
    let toggle: w::ThemeToggle<'_, c::ThemeMode> = c::ThemeToggle::new(mode).on_toggle(|mode| mode);
    let toggle: c::ThemeToggle<'_, w::ThemeMode> = toggle.padding(4).disabled(false);
    let _: Element<'_, w::ThemeMode> = toggle.into();
}
