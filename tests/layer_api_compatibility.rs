//! Public paths retained when shared atoms move to their owning lower layers.

use iad::{Element, components, layouts, widgets};
use iced_adaptive_design as iad;

#[test]
fn slider_paths_and_nested_props_keep_the_same_types() {
    let props: components::RangeSliderProps = widgets::RangeSliderProps::default().width(231);
    let scale_props: components::ScaleProps = widgets::ScaleProps::default().slider_props(props);
    let scale: widgets::Scale<'_, f32, f32> = components::scale(0.0..=100.0, 40.0, |v| v);
    let scale: components::Scale<'_, f32, f32> = scale_props.apply(scale);
    let element: Element<'_, f32> = scale.into();
    assert_eq!(element.as_widget().size().width, iced::Length::Fixed(231.0));

    let range: components::RangeSlider<'_, (f32, f32)> =
        widgets::range_slider(0.0..=100.0, (20.0, 80.0), |v| v);
    let range: widgets::RangeSlider<'_, (f32, f32)> = components::RangeSliderProps::default()
        .width(197)
        .apply(range);
    let element: Element<'_, (f32, f32)> = range.into();
    assert_eq!(element.as_widget().size().width, iced::Length::Fixed(197.0));
}

#[test]
fn appearance_paths_keep_the_same_controlled_button_and_mode() {
    let mode: widgets::ThemeMode = components::ThemeMode::Dark;
    let mode: iad::primitives::appearance::ThemeMode = mode;
    let toggle: widgets::ThemeToggle<'_, widgets::ThemeMode> =
        components::ThemeToggle::new(mode).on_toggle(|v| v);
    let toggle: components::ThemeToggle<'_, components::ThemeMode> = toggle;
    let _: Element<'_, components::ThemeMode> = toggle.into();
    assert_eq!(
        mode.icon().source(),
        widgets::ThemeMode::Dark.icon().source()
    );
}

#[test]
fn scale_keeps_oriented_input_disabled_and_release_callbacks() {
    use iad::primitives::scrub::Axis;
    use iced::advanced::{Shell, clipboard, layout, renderer::Headless, widget::Tree};
    use iced::{Event, Point, Size, mouse};

    #[derive(Clone, Debug, PartialEq)]
    enum Message {
        Changed(i32),
        Released,
    }

    let renderer = iced::futures::executor::block_on(iced::Renderer::new(
        iced::Font::DEFAULT,
        iced::Pixels(14.0),
        Some("tiny-skia"),
    ))
    .expect("software renderer");
    for axis in [Axis::Horizontal, Axis::Vertical] {
        for disabled in [false, true] {
            for range in [0..=100, 100..=0] {
                let scale: widgets::Scale<'_, i32, Message> =
                    components::scale(range, 40, Message::Changed)
                        .step(5)
                        .disabled(disabled)
                        .on_release(Message::Released);
                let mut element: Element<'_, Message> = widgets::ScaleProps::default()
                    .slider_props(
                        widgets::RangeSliderProps::default()
                            .orientation(axis)
                            .width(220)
                            .height(220),
                    )
                    .apply(scale)
                    .into();
                let mut tree = Tree::new(element.as_widget());
                let node = element.as_widget_mut().layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(220.0, 220.0)),
                );
                let layout = layout::Layout::new(&node);
                let bounds = layout.bounds();
                let radius = iad::styles::slider::HANDLE_RADIUS;
                let point = match axis {
                    Axis::Horizontal => {
                        Point::new(bounds.x + bounds.width - radius, bounds.center_y())
                    }
                    Axis::Vertical => Point::new(bounds.center_x(), bounds.y + radius),
                };
                let mut messages = vec![];
                for event in [
                    mouse::Event::ButtonPressed(mouse::Button::Left),
                    mouse::Event::ButtonReleased(mouse::Button::Left),
                ] {
                    element.as_widget_mut().update(
                        &mut tree,
                        &Event::Mouse(event),
                        layout,
                        mouse::Cursor::Available(point),
                        &renderer,
                        &mut clipboard::Null,
                        &mut Shell::new(&mut messages),
                        &bounds,
                    );
                }
                let expected = if disabled {
                    vec![]
                } else {
                    vec![Message::Changed(100), Message::Released]
                };
                assert_eq!(messages, expected, "{axis:?}, disabled={disabled}");
            }
        }
    }
}

#[test]
fn component_bevel_is_the_shared_layout_type() {
    let bevel: components::Bevel<'_, ()> = layouts::Bevel::new(
        iced::widget::space(),
        "frame.field",
        iad::primitives::bevel::BevelStyle::Sunken,
        2.0,
    );
    let bevel: layouts::Bevel<'_, ()> = bevel.padding(3).pressable(true);
    assert_eq!(bevel.inset(), 2.0);
    let _: Element<'_, ()> = bevel.into();
    assert_eq!(
        components::bevel_of("frame.field"),
        layouts::bevel_of("frame.field")
    );
    let _: Element<'_, ()> = components::framed(iced::widget::space(), "field");
    let _: Element<'_, ()> = layouts::framed(iced::widget::space(), "field");
}
