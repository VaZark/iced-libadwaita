//! Drag and drop between containers: a [`drag_source`] that starts a drag after the pointer
//! moves a few pixels with the button held, and a [`drop_zone`] that takes the drop. The app holds
//! what is being dragged; these only say when it starts and where it lands.
//!
//! The picked-up copy floats at the pointer above everything. It changes with what it is over:
//! a [`DropZone::hint`] gives it a label and a scale (shrunk when it is going inside the target),
//! so the same copy says "Move to Design" over one place and "Make a subtask of …" over another.
//! Zones nest: the innermost one under the pointer takes the drop and the hint
//! (see [`primitives::drag`](crate::primitives::drag)).
//!
//! A press that does not move is still a click on whatever the source holds (a button, say). Once
//! the drag has started the source's content is told the pointer left, so it never also sees the
//! release as a click.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{
    Background, Border, Color, Element, Event, Length, Padding, Point, Rectangle, Shadow, Size, Transformation,
    Vector, mouse,
};

use crate::Theme;
use crate::primitives::drag::{self, DropHint, Place, Targets};

/// How far the pointer moves, in pixels, before a press is a drag.
const THRESHOLD: f32 = 6.0;

#[derive(Default)]
struct SourceState {
    /// Where the pointer went down, in the content's own coordinates.
    press: Option<Point>,
    dragging: bool,
}

/// `content`, which can be picked up. `on_start` is sent once when the drag begins.
pub struct DragSource<'a, M> {
    content: Element<'a, M, Theme, iced::Renderer>,
    on_start: M,
    ghost_padding: Padding,
}

impl<M> DragSource<'_, M> {
    /// Tells the drop zones where the copy is: the source's bounds, with its ghost padding,
    /// moved as far as the pointer has moved since it was picked up.
    fn publish_ghost(&self, layout: Layout<'_>, press: Option<Point>, at: Point) {
        let Some(press) = press else { return };
        let b = layout.bounds().expand(self.ghost_padding);
        drag::set_ghost(Some((b.y + at.y - press.y, b.height)));
    }

    /// Room the picked-up copy keeps around the content, inside its frame: a number for all sides
    /// or `[vertical, horizontal]`. Zero by default: a row that already has its own padding needs
    /// none, a bare label does.
    pub fn ghost_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.ghost_padding = padding.into();
        self
    }
}

pub fn drag_source<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M, Theme, iced::Renderer>>,
    on_start: M,
) -> DragSource<'a, M> {
    DragSource {
        content: content.into(),
        on_start,
        ghost_padding: Padding::ZERO,
    }
}

impl<'a, M: Clone + 'a> Widget<M, Theme, iced::Renderer> for DragSource<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SourceState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(SourceState::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.content.as_widget())]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<SourceState>();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(layout.bounds()) =>
            {
                state.press = cursor.position();
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if !state.dragging => {
                if let Some(from) = state.press
                    && from.distance(*position) > THRESHOLD
                {
                    state.dragging = true;
                    self.publish_ghost(layout, state.press, *position);
                    shell.publish(self.on_start.clone());
                    // The content's button must not take the release for a click.
                    self.content.as_widget_mut().update(
                        &mut tree.children[0],
                        &Event::Mouse(mouse::Event::CursorLeft),
                        layout,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                    shell.request_redraw();
                    return;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let was = state.dragging;
                state.press = None;
                state.dragging = false;
                if was {
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
            }
            _ => {}
        }
        if state.dragging {
            // The floating copy follows the pointer.
            if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
                let press = tree.state.downcast_ref::<SourceState>().press;
                self.publish_ghost(layout, press, *position);
                shell.request_redraw();
            }
            return;
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if tree.state.downcast_ref::<SourceState>().dragging {
            return mouse::Interaction::Grabbing;
        }
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        // Where it was picked up from stays, dimmed: the copy in hand is the real one.
        if tree.state.downcast_ref::<SourceState>().dragging {
            renderer.fill_quad(
                Quad {
                    bounds: layout.bounds(),
                    border: Border::default().rounded(theme.metrics.radius_s),
                    ..Quad::default()
                },
                Background::Color(Color {
                    a: 0.55,
                    ..theme.semantic.window.bg
                }),
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        let (dragging, press) = {
            let st = tree.state.downcast_ref::<SourceState>();
            (st.dragging, st.press)
        };
        let [child] = tree.children.as_mut_slice() else {
            return None;
        };
        if dragging {
            if let Some(grab) = press {
                return Some(overlay::Element::new(Box::new(Ghost {
                    content: &self.content,
                    tree: child,
                    source: layout,
                    grab,
                    pad: self.ghost_padding,
                })));
            }
        }
        self.content
            .as_widget_mut()
            .overlay(child, layout, renderer, viewport, translation)
    }
}

/// The picked-up copy, floating at the pointer above everything else.
struct Ghost<'a, 'b, M> {
    content: &'b Element<'a, M, Theme, iced::Renderer>,
    tree: &'b Tree,
    source: Layout<'b>,
    grab: Point,
    pad: Padding,
}

impl<M> overlay::Overlay<M, Theme, iced::Renderer> for Ghost<'_, '_, M> {
    fn layout(&mut self, _: &iced::Renderer, bounds: Size) -> layout::Node {
        layout::Node::new(bounds)
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let Some(at) = cursor.position() else { return };
        // What the zone under the pointer said, if there is one.
        let hint = drag::hovered().flatten();
        let scale = hint.as_ref().map_or(1.0, |h| h.scale);
        let shift = Vector::new(at.x - self.grab.x, at.y - self.grab.y);
        let b = self.source.bounds().expand(self.pad);
        let lift = theme.shadows.popover();
        let radius = theme.metrics.radius_s;
        // The copy is scaled about the point it is held by, so it stays under the pointer.
        let grow = Transformation::translate(self.grab.x, self.grab.y)
            * Transformation::scale(scale)
            * Transformation::translate(-self.grab.x, -self.grab.y);
        let everything = Rectangle::new(Point::new(-1.0e6, -1.0e6), Size::new(2.0e6, 2.0e6));
        renderer.with_layer(layout.bounds(), |renderer| {
            renderer.with_translation(shift, |renderer| {
                renderer.with_transformation(grow, |renderer| {
                    // The card colours are translucent, so the copy gets the opaque view colour
                    // under it, a ring in the accent colour and a shadow: lifted off the page.
                    renderer.fill_quad(
                        Quad {
                            bounds: b,
                            border: Border {
                                color: theme.semantic.accent.color,
                                width: 2.0,
                                radius: radius.into(),
                            },
                            shadow: Shadow {
                                color: lift.shadow.color,
                                offset: Vector::new(0.0, 4.0),
                                blur_radius: lift.shadow.blur_radius,
                            },
                            snap: false,
                        },
                        Background::Color(Color {
                            a: 1.0,
                            ..theme.semantic.view.bg
                        }),
                    );
                    self.content.as_widget().draw(
                        self.tree,
                        renderer,
                        theme,
                        style,
                        self.source,
                        mouse::Cursor::Unavailable,
                        &everything,
                    );
                });
            });
        });
        // The label, a pill under the pointer: not scaled, so it stays readable.
        if let Some(h) = hint.filter(|h| !h.label.is_empty()) {
            draw_label(renderer, theme, layout.bounds(), at, &h.label);
        }
    }
}

/// A pill with `label` just below and to the right of the pointer, kept inside `window`.
fn draw_label(
    renderer: &mut iced::Renderer,
    theme: &Theme,
    window: Rectangle,
    at: Point,
    label: &str,
) {
    use iced::advanced::text::{self, Paragraph as _, Renderer as _};
    const SIZE: f32 = 13.0;
    let content = text::Text {
        content: label,
        bounds: Size::new(f32::INFINITY, f32::INFINITY),
        size: iced::Pixels(SIZE),
        line_height: text::LineHeight::default(),
        font: renderer.default_font(),
        align_x: text::Alignment::Left,
        align_y: iced::alignment::Vertical::Top,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::None,
    };
    let measured = <iced::Renderer as text::Renderer>::Paragraph::with_text(content).min_bounds();
    let pad = Size::new(10.0, 5.0);
    let size = Size::new(measured.width + 2.0 * pad.width, measured.height + 2.0 * pad.height);
    let x = (at.x + 14.0).min(window.x + window.width - size.width - 4.0).max(4.0);
    let y = (at.y + 22.0).min(window.y + window.height - size.height - 4.0).max(4.0);
    let pill = Rectangle::new(Point::new(x, y), size);
    let accent = theme.semantic.accent;
    renderer.fill_quad(
        Quad {
            bounds: pill,
            border: Border::default().rounded(size.height / 2.0),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            snap: false,
        },
        Background::Color(accent.bg),
    );
    renderer.fill_text(
        text::Text {
            content: label.to_string(),
            bounds: measured,
            size: iced::Pixels(SIZE),
            line_height: text::LineHeight::default(),
            font: renderer.default_font(),
            align_x: text::Alignment::Left,
            align_y: iced::alignment::Vertical::Top,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::None,
        },
        Point::new(x + pad.width, y + pad.height),
        accent.fg,
        pill,
    );
}

impl<'a, M: Clone + 'a> From<DragSource<'a, M>> for Element<'a, M, Theme, iced::Renderer> {
    fn from(d: DragSource<'a, M>) -> Self {
        Element::new(d)
    }
}

/// `content`, which a drag can be dropped on. While `active` (a drag is going on) the zone under
/// the pointer is outlined, and releasing the button over it sends `on_drop`.
pub struct DropZone<'a, M> {
    content: Element<'a, M, Theme, iced::Renderer>,
    active: bool,
    on_drop: Box<dyn Fn(Place) -> M + 'a>,
    hint: Option<DropHint>,
    edge_hint: Option<DropHint>,
    targets: Targets,
    reach: f32,
}

impl<M> DropZone<'_, M> {
    /// Pixels above and below the content that still count as the zone: for rows with a gap
    /// between them, so the pointer is never over nothing between two (and the edge between them
    /// is as easy to hit as the rows). Neighbours that overlap there: the first one wins.
    pub fn reach(mut self, px: f32) -> Self {
        self.reach = px.max(0.0);
        self
    }

    /// How the zone is divided: the whole of it ([`Targets::Whole`], the default), before / after
    /// only ([`Targets::Between`], to reorder) or edges with a middle ([`Targets::Edges`]). Over
    /// an edge the zone draws an insertion line instead of an outline, and `on_drop` of
    /// [`drop_zone_at`] is told which.
    pub fn targets(mut self, targets: Targets) -> Self {
        self.targets = targets;
        self
    }

    /// What the floating copy says over an edge (before / after), where the copy keeps its size.
    /// Without it the copy says nothing there.
    pub fn edge_hint(mut self, hint: DropHint) -> Self {
        self.edge_hint = Some(hint);
        self
    }

    /// What the floating copy says and how it looks while it is over this zone.
    pub fn hint(mut self, hint: DropHint) -> Self {
        self.hint = Some(hint);
        self
    }
}

pub fn drop_zone<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M, Theme, iced::Renderer>>,
    active: bool,
    on_drop: M,
) -> DropZone<'a, M> {
    drop_zone_at(content, active, move |_| on_drop.clone())
}

/// A [`drop_zone`] that says where it was dropped: `on_drop` gets the [`Place`] under the pointer
/// (see [`DropZone::targets`]; without it, always [`Place::Inside`]).
pub fn drop_zone_at<'a, M: 'a>(
    content: impl Into<Element<'a, M, Theme, iced::Renderer>>,
    active: bool,
    on_drop: impl Fn(Place) -> M + 'a,
) -> DropZone<'a, M> {
    DropZone {
        content: content.into(),
        active,
        on_drop: Box::new(on_drop),
        hint: None,
        edge_hint: None,
        targets: Targets::Whole,
        reach: 0.0,
    }
}

impl<M> DropZone<'_, M> {
    /// The place under the cursor, when it is over the zone.
    fn place(&self, layout: Layout<'_>, cursor: mouse::Cursor) -> Option<Place> {
        let b = self.reached(layout);
        // A zone with edges follows the carried copy, not the pointer: a row is chosen by where
        // the copy sits over it, so what is carried can be lined up with the gap it goes in.
        if self.targets != Targets::Whole
            && let Some(ghost) = drag::ghost()
        {
            return self.targets.place_of_ghost((b.y, b.height), ghost);
        }
        let at = cursor.position_over(b)?;
        Some(self.targets.place_at(at.y - b.y, b.height))
    }

    /// The zone's bounds with its [`reach`](Self::reach).
    fn reached(&self, layout: Layout<'_>) -> Rectangle {
        let b = layout.bounds();
        Rectangle {
            y: b.y - self.reach,
            height: b.height + 2.0 * self.reach,
            ..b
        }
    }
}

impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for DropZone<'a, M> {
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }
    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }
    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }
    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        drag::enter();
        self.content.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
        if self.active {
            match event {
                Event::Mouse(mouse::Event::CursorMoved { .. }) => shell.request_redraw(),
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                    if self.place(layout, cursor).is_some() && drag::take_drop() =>
                {
                    let place = self.place(layout, cursor).unwrap_or(Place::Inside);
                    shell.publish((self.on_drop)(place));
                }
                _ => {}
            }
        }
        drag::exit();
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
        let Some(place) = self.place(layout, cursor).filter(|_| self.active) else {
            return;
        };
        let hint = match place {
            Place::Inside => self.hint.as_ref(),
            _ => self.edge_hint.as_ref(),
        };
        if !drag::take_hover(hint) {
            return;
        }
        let b = layout.bounds();
        let accent = theme.semantic.accent.color;
        match place {
            Place::Inside => renderer.fill_quad(
                Quad {
                    bounds: b,
                    border: Border {
                        color: accent,
                        width: 2.0,
                        radius: theme.metrics.radius_m.into(),
                    },
                    ..Quad::default()
                },
                Color::TRANSPARENT,
            ),
            // An insertion line on the edge, with a dot at its start, as GTK draws it.
            Place::Before | Place::After => {
                const LINE: f32 = 3.0;
                // Centred in the gap, when there is one, between this row and its neighbour.
                let y = if place == Place::Before {
                    b.y - self.reach / 2.0
                } else {
                    b.y + b.height + self.reach / 2.0
                };
                let inset = 6.0_f32.min(b.width / 4.0);
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point::new(b.x + inset, y - LINE / 2.0),
                            Size::new(b.width - 2.0 * inset, LINE),
                        ),
                        border: Border::default().rounded(LINE / 2.0),
                        ..Quad::default()
                    },
                    Background::Color(accent),
                );
            }
        }
    }
}

impl<'a, M: Clone + 'a> From<DropZone<'a, M>> for Element<'a, M, Theme, iced::Renderer> {
    fn from(d: DropZone<'a, M>) -> Self {
        Element::new(d)
    }
}
