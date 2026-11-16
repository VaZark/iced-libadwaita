//! Downstream layout, transformed-input and overlay regression tests.
use iced_adaptive_design::{Element, Theme};

use iced::advanced::renderer::Headless;
use iced::advanced::{
    Clipboard, Shell, clipboard, layout, mouse, overlay, renderer,
    widget::{Operation, Tree, Widget},
};
use iced::{Event, Length, Point, Rectangle, Size, Vector, touch};
use iced_adaptive_design::layouts::{Overflow, OverflowMode, overflow};

fn renderer() -> iced::Renderer {
    iced::futures::executor::block_on(iced::Renderer::new(
        iced::Font::DEFAULT,
        iced::Pixels(14.0),
        Some("tiny-skia"),
    ))
    .expect("headless software renderer")
}

fn measure<M>(
    element: &mut Element<'_, M>,
    renderer: &iced::Renderer,
    max: Size,
) -> (Tree, layout::Node) {
    let mut tree = Tree::new(element.as_widget());
    let node =
        element
            .as_widget_mut()
            .layout(&mut tree, renderer, &layout::Limits::new(Size::ZERO, max));
    (tree, node)
}

#[test]
fn grow_reports_natural_size_under_finite_limits_and_scene_floor_is_not_a_cap() {
    let renderer = renderer();
    let mut element: Element<'_, ()> = overflow(iced::widget::space().width(600).height(300))
        .content_size(Size::new(400.0, 400.0))
        .width(Length::Fill)
        .into();
    let (_, node) = measure(&mut element, &renderer, Size::new(200.0, 100.0));
    assert_eq!(node.size(), Size::new(600.0, 400.0));
}

#[test]
fn scale_down_contains_preserves_aspect_and_never_upscales() {
    let renderer = renderer();
    for (available, expected) in [
        (Size::new(200.0, 200.0), 0.5),
        (Size::new(800.0, 600.0), 1.0),
    ] {
        let mut element: Element<'_, Seen> = Overflow::new(Element::new(Probe))
            .mode(OverflowMode::ScaleDown)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        let (mut tree, node) = measure(&mut element, &renderer, available);
        assert_eq!(node.size(), available);
        let mut bounds = Bounds::default();
        element.as_widget_mut().operate(
            &mut tree,
            layout::Layout::new(&node),
            &renderer,
            &mut bounds,
        );
        assert_eq!(
            bounds.0,
            vec![Rectangle::new(
                Point::ORIGIN,
                Size::new(400.0 * expected, 200.0 * expected)
            )]
        );
    }
}

#[test]
fn actual_text_font_growth_increases_natural_size_with_fill_compression() {
    let renderer = renderer();
    let mut dimensions = Vec::new();
    for font_size in [12, 36] {
        let mut element: Element<'_, ()> = overflow(
            iced::widget::column![iced::widget::text("Intrinsic text grows").size(font_size)]
                .width(Length::Fill),
        )
        .min_size(Size::new(1.0, 1.0))
        .into();
        let (_, node) = measure(&mut element, &renderer, Size::new(40.0, 20.0));
        assert!(node.size().width.is_finite() && node.size().height.is_finite());
        dimensions.push(node.size());
    }
    assert!(dimensions[1].width > dimensions[0].width);
    assert!(dimensions[1].height > dimensions[0].height);
    assert!(dimensions[1].width > 40.0);
}

#[derive(Debug, PartialEq)]
struct Seen {
    cursor: Option<Point>,
    touch: Option<Point>,
    bounds: Rectangle,
}
struct Probe;
impl Widget<Seen, Theme, iced::Renderer> for Probe {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }
    fn layout(&mut self, _: &mut Tree, _: &iced::Renderer, _: &layout::Limits) -> layout::Node {
        layout::Node::new(Size::new(400.0, 200.0))
    }
    fn draw(
        &self,
        _: &Tree,
        renderer: &mut iced::Renderer,
        _: &Theme,
        _: &renderer::Style,
        layout: layout::Layout<'_>,
        _: mouse::Cursor,
        _: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                ..Default::default()
            },
            iced::Color::from_rgb(1.0, 0.0, 0.0),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle::new(
                    layout.position() + Vector::new(100.0, 40.0),
                    Size::new(100.0, 60.0),
                ),
                ..Default::default()
            },
            iced::Color::from_rgb(0.0, 1.0, 0.0),
        );
    }
    fn update(
        &mut self,
        _: &mut Tree,
        event: &Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        _: &iced::Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, Seen>,
        _: &Rectangle,
    ) {
        shell.publish(seen(event, cursor, layout.bounds()));
    }
    fn operate(
        &mut self,
        _: &mut Tree,
        layout: layout::Layout<'_>,
        _: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.traverse(&mut |operation| operation.text(None, layout.bounds(), "probe"));
    }
    fn mouse_interaction(
        &self,
        _: &Tree,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
        _: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
    fn overlay<'a>(
        &'a mut self,
        _: &'a mut Tree,
        layout: layout::Layout<'a>,
        _: &iced::Renderer,
        _: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Seen, Theme, iced::Renderer>> {
        Some(overlay::Element::new(Box::new(Popup {
            position: layout.position() + Vector::new(100.0, 40.0) + translation,
            nested: true,
        })))
    }
}
fn seen(event: &Event, cursor: mouse::Cursor, bounds: Rectangle) -> Seen {
    Seen {
        cursor: cursor.position(),
        touch: match event {
            Event::Touch(touch::Event::FingerPressed { position, .. }) => Some(*position),
            _ => None,
        },
        bounds,
    }
}
#[derive(Default)]
struct Bounds(Vec<Rectangle>);
impl Operation for Bounds {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }
    fn text(&mut self, _: Option<&iced::advanced::widget::Id>, bounds: Rectangle, _: &str) {
        self.0.push(bounds);
    }
}
struct Popup {
    position: Point,
    nested: bool,
}
impl overlay::Overlay<Seen, Theme, iced::Renderer> for Popup {
    fn layout(&mut self, _: &iced::Renderer, _: Size) -> layout::Node {
        layout::Node::new(Size::new(100.0, 60.0)).move_to(self.position)
    }
    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        _: &Theme,
        _: &renderer::Style,
        layout: layout::Layout<'_>,
        _: mouse::Cursor,
    ) {
        use iced::advanced::Renderer as _;
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                ..Default::default()
            },
            iced::Color::from_rgb(0.0, 0.0, 1.0),
        );
    }
    fn update(
        &mut self,
        event: &Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        _: &iced::Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, Seen>,
    ) {
        shell.publish(seen(event, cursor, layout.bounds()));
    }
    fn mouse_interaction(
        &self,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        _: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
    fn operate(
        &mut self,
        layout: layout::Layout<'_>,
        _: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.text(None, layout.bounds(), "popup");
    }
    fn overlay<'a>(
        &'a mut self,
        _: layout::Layout<'a>,
        _: &iced::Renderer,
    ) -> Option<overlay::Element<'a, Seen, Theme, iced::Renderer>> {
        self.nested.then(|| {
            overlay::Element::new(Box::new(Popup {
                position: self.position + Vector::new(20.0, 10.0),
                nested: false,
            }))
        })
    }
    fn index(&self) -> f32 {
        7.0
    }
}

#[test]
fn inverse_pointer_touch_mouse_interaction_and_operations_at_nonzero_origin() {
    let renderer = renderer();
    let mut element: Element<'_, Seen> = overflow(Element::new(Probe))
        .mode(OverflowMode::ScaleDown)
        .width(200)
        .height(100)
        .into();
    let (mut tree, node) = measure(&mut element, &renderer, Size::new(200.0, 100.0));
    let node = node.move_to(Point::new(30.0, 50.0));
    let layout = layout::Layout::new(&node);
    let cursor = mouse::Cursor::Available(Point::new(80.0, 75.0));
    let viewport = Rectangle::with_size(Size::new(800.0, 600.0));
    let mut messages = Vec::new();
    element.as_widget_mut().update(
        &mut tree,
        &Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(0),
            position: Point::new(80.0, 75.0),
        }),
        layout,
        cursor,
        &renderer,
        &mut clipboard::Null,
        &mut Shell::new(&mut messages),
        &viewport,
    );
    assert_eq!(
        messages,
        vec![Seen {
            cursor: Some(Point::new(130.0, 100.0)),
            touch: Some(Point::new(130.0, 100.0)),
            bounds: Rectangle::new(Point::new(30.0, 50.0), Size::new(400.0, 200.0))
        }]
    );
    assert_eq!(
        element
            .as_widget()
            .mouse_interaction(&tree, layout, cursor, &viewport, &renderer),
        mouse::Interaction::Pointer
    );
    assert_eq!(
        element.as_widget().mouse_interaction(
            &tree,
            layout,
            mouse::Cursor::Available(Point::new(240.0, 60.0)),
            &viewport,
            &renderer
        ),
        mouse::Interaction::None
    );
    let mut bounds = Bounds::default();
    element
        .as_widget_mut()
        .operate(&mut tree, layout, &renderer, &mut bounds);
    assert_eq!(
        bounds.0,
        vec![Rectangle::new(
            Point::new(30.0, 50.0),
            Size::new(200.0, 100.0)
        )]
    );
}

#[test]
fn overlays_and_nested_overlays_transform_layout_input_bounds_and_interaction() {
    let renderer = renderer();
    let mut element: Element<'_, Seen> = overflow(Element::new(Probe))
        .mode(OverflowMode::ScaleDown)
        .width(200)
        .height(100)
        .into();
    let (mut tree, node) = measure(&mut element, &renderer, Size::new(200.0, 100.0));
    let node = node.move_to(Point::new(30.0, 50.0));
    let viewport = Rectangle::with_size(Size::new(800.0, 600.0));
    let mut popup = element
        .as_widget_mut()
        .overlay(
            &mut tree,
            layout::Layout::new(&node),
            &renderer,
            &viewport,
            Vector::new(10.0, 20.0),
        )
        .expect("forwarded overlay");
    assert_eq!(popup.as_overlay().index(), 7.0);
    let popup_node = popup.as_overlay_mut().layout(&renderer, viewport.size());
    assert_eq!(
        popup_node.bounds(),
        Rectangle::new(Point::new(90.0, 90.0), Size::new(50.0, 30.0))
    );
    let popup_layout = layout::Layout::new(&popup_node);
    let cursor = mouse::Cursor::Available(Point::new(100.0, 100.0));
    assert_eq!(
        popup
            .as_overlay()
            .mouse_interaction(popup_layout, cursor, &renderer),
        mouse::Interaction::Pointer
    );
    let mut bounds = Bounds::default();
    popup
        .as_overlay_mut()
        .operate(popup_layout, &renderer, &mut bounds);
    assert_eq!(bounds.0, vec![popup_node.bounds()]);
    let mut messages = Vec::new();
    popup.as_overlay_mut().update(
        &Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: Point::new(100.0, 100.0),
        }),
        popup_layout,
        cursor,
        &renderer,
        &mut clipboard::Null,
        &mut Shell::new(&mut messages),
    );
    assert_eq!(messages[0].cursor, Some(Point::new(170.0, 150.0)));
    assert_eq!(messages[0].touch, Some(Point::new(170.0, 150.0)));
    let mut nested = popup
        .as_overlay_mut()
        .overlay(popup_layout, &renderer)
        .expect("nested overlay");
    let nested_node = nested.as_overlay_mut().layout(&renderer, viewport.size());
    assert_eq!(
        nested_node.bounds(),
        Rectangle::new(Point::new(100.0, 95.0), Size::new(50.0, 30.0))
    );
}

#[test]
fn native_scrollable_wheel_moves_content_in_both_directions() {
    let renderer = renderer();
    let mut element: Element<'_, Seen> = overflow(Element::new(Probe))
        .mode(OverflowMode::Scroll)
        .width(100)
        .height(80)
        .into();
    let (mut tree, node) = measure(&mut element, &renderer, Size::new(100.0, 80.0));
    assert_eq!(node.size(), Size::new(100.0, 80.0));
    let layout = layout::Layout::new(&node);
    let viewport = Rectangle::with_size(node.size());
    let mut messages = Vec::new();
    let mut shell = Shell::new(&mut messages);
    element.as_widget_mut().update(
        &mut tree,
        &Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: -30.0, y: -20.0 },
        }),
        layout,
        mouse::Cursor::Available(Point::new(10.0, 10.0)),
        &renderer,
        &mut clipboard::Null,
        &mut shell,
        &viewport,
    );
    assert!(shell.is_event_captured());
    struct ScrollBounds(Option<Vector>);
    impl Operation for ScrollBounds {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn scrollable(
            &mut self,
            _: Option<&iced::advanced::widget::Id>,
            _: Rectangle,
            _: Rectangle,
            translation: Vector,
            _: &mut dyn iced::advanced::widget::operation::Scrollable,
        ) {
            self.0 = Some(translation);
        }
    }
    let mut bounds = ScrollBounds(None);
    element
        .as_widget_mut()
        .operate(&mut tree, layout, &renderer, &mut bounds);
    let translation = bounds.0.expect("native scrollable operation");
    assert!(
        translation.x.abs() > 0.0 && translation.y.abs() > 0.0,
        "{translation:?}"
    );
}

#[test]
fn rendered_content_and_overlay_match_independent_scaled_raster() {
    use iced::advanced::Renderer as _;
    let mut renderer = renderer();
    let mut element: Element<'_, Seen> = overflow(Element::new(Probe))
        .mode(OverflowMode::ScaleDown)
        .width(200)
        .height(100)
        .into();
    let (mut tree, node) = measure(&mut element, &renderer, Size::new(200.0, 100.0));
    let node = node.move_to(Point::new(30.0, 50.0));
    let viewport = Rectangle::with_size(Size::new(300.0, 200.0));
    let theme = Theme::light();
    let style = renderer::Style {
        text_color: iced::Color::BLACK,
    };
    for with_popup in [false, true] {
        renderer.reset(viewport);
        element.as_widget().draw(
            &tree,
            &mut renderer,
            &theme,
            &style,
            layout::Layout::new(&node),
            mouse::Cursor::Unavailable,
            &viewport,
        );
        if with_popup {
            let mut popup = element
                .as_widget_mut()
                .overlay(
                    &mut tree,
                    layout::Layout::new(&node),
                    &renderer,
                    &viewport,
                    Vector::new(10.0, 20.0),
                )
                .unwrap();
            let popup_node = popup.as_overlay_mut().layout(&renderer, viewport.size());
            popup.as_overlay().draw(
                &mut renderer,
                &theme,
                &style,
                layout::Layout::new(&popup_node),
                mouse::Cursor::Unavailable,
            );
        }
        let actual = renderer.screenshot(Size::new(300, 200), 1.0, iced::Color::WHITE);
        renderer.reset(viewport);
        for (bounds, color) in [
            (
                Rectangle::new(Point::new(30.0, 50.0), Size::new(200.0, 100.0)),
                iced::Color::from_rgb(1.0, 0.0, 0.0),
            ),
            (
                Rectangle::new(Point::new(80.0, 70.0), Size::new(50.0, 30.0)),
                iced::Color::from_rgb(0.0, 1.0, 0.0),
            ),
        ] {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    ..Default::default()
                },
                color,
            );
        }
        if with_popup {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(Point::new(90.0, 90.0), Size::new(50.0, 30.0)),
                    ..Default::default()
                },
                iced::Color::from_rgb(0.0, 0.0, 1.0),
            );
        }
        let expected = renderer.screenshot(Size::new(300, 200), 1.0, iced::Color::WHITE);
        assert_eq!(
            actual, expected,
            "content and overlay raster, popup={with_popup}"
        );
    }
}

#[test]
fn native_text_input_focus_operation_retains_state_and_transforms_focus_bounds() {
    struct FocusBounds {
        bounds: Vec<Rectangle>,
        focused: Vec<bool>,
    }
    impl Operation for FocusBounds {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn focusable(
            &mut self,
            _: Option<&iced::advanced::widget::Id>,
            bounds: Rectangle,
            state: &mut dyn iced::advanced::widget::operation::Focusable,
        ) {
            self.focused.push(state.is_focused());
            state.focus();
            self.bounds.push(bounds);
        }
    }
    let renderer = renderer();
    let mut element: Element<'_, String> = overflow(
        iced::widget::text_input("", "focus me")
            .width(400)
            .on_input(|value| value),
    )
    .mode(OverflowMode::ScaleDown)
    .min_size(Size::new(400.0, 200.0))
    .width(200)
    .height(100)
    .into();
    let (mut tree, node) = measure(&mut element, &renderer, Size::new(200.0, 100.0));
    let node = node.move_to(Point::new(30.0, 50.0));
    let mut first = FocusBounds {
        bounds: Vec::new(),
        focused: Vec::new(),
    };
    element
        .as_widget_mut()
        .operate(&mut tree, layout::Layout::new(&node), &renderer, &mut first);
    assert_eq!(first.focused, vec![false]);
    assert_eq!(first.bounds[0].position(), Point::new(30.0, 50.0));
    assert_eq!(first.bounds[0].width, 200.0);
    let mut second = FocusBounds {
        bounds: Vec::new(),
        focused: Vec::new(),
    };
    element.as_widget_mut().operate(
        &mut tree,
        layout::Layout::new(&node),
        &renderer,
        &mut second,
    );
    assert_eq!(second.focused, vec![true]);
    assert_eq!(first.bounds, second.bounds);
}

#[test]
fn zero_viewport_has_finite_bounds_and_no_pointer_overlay_or_touch_dispatch() {
    let renderer = renderer();
    let mut element: Element<'_, Seen> = overflow(Element::new(Probe))
        .mode(OverflowMode::ScaleDown)
        .width(0)
        .height(0)
        .into();
    let (mut tree, node) = measure(&mut element, &renderer, Size::ZERO);
    let layout = layout::Layout::new(&node);
    let viewport = Rectangle::with_size(Size::new(100.0, 100.0));
    let mut messages = Vec::new();
    element.as_widget_mut().update(
        &mut tree,
        &Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(0),
            position: Point::ORIGIN,
        }),
        layout,
        mouse::Cursor::Available(Point::ORIGIN),
        &renderer,
        &mut clipboard::Null,
        &mut Shell::new(&mut messages),
        &viewport,
    );
    assert!(messages.is_empty());
    assert!(
        element
            .as_widget_mut()
            .overlay(&mut tree, layout, &renderer, &viewport, Vector::ZERO)
            .is_none()
    );
    let mut bounds = Bounds::default();
    element
        .as_widget_mut()
        .operate(&mut tree, layout, &renderer, &mut bounds);
    assert_eq!(bounds.0, vec![Rectangle::with_size(Size::ZERO)]);
}
