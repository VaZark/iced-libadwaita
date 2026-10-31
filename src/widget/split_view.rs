//! The adaptive sidebar: beside the content when there is room, a drawer when there is not, and
//! one continuous motion between the two.
//!
//! The sidebar is a single child that lives in one of two places:
//!
//! * **in flow**: it takes space at the start edge and displaces the content;
//! * **as a drawer**: it floats over the content on a scrim.
//!
//! Both use the sidebar spring (damping ratio 1, mass 0.5, stiffness 500), so a window crossing
//! the collapse breakpoint slides the sidebar out of (or into) the layout the same way the
//! drawer slides over the content. The only difference is the layer: in flow the content moves
//! out of its way instead of being covered.

use super::motion::{Progress, redraw_time, scrim};
use crate::Theme;
use crate::animation::{Spec, Trigger, presets};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell, overlay};
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Rectangle, Size, Vector, mouse,
};

pub struct AdaptiveSplit<'a, M> {
    /// `[sidebar, content]`.
    children: Vec<Element<'a, M, Theme>>,
    wide: bool,
    sidebar_width: f32,
    drawer_width: f32,
    drawer_open: bool,
    on_close: Option<M>,
    window_radius: f32,
}

/// A sidebar and a content area. `wide`: the sidebar sits beside the content (`sidebar_width`);
/// otherwise it is a drawer (`drawer_width`) shown while `drawer_open`.
pub fn adaptive_split<'a, M>(
    sidebar: impl Into<Element<'a, M, Theme>>,
    content: impl Into<Element<'a, M, Theme>>,
    wide: bool,
    sidebar_width: f32,
    drawer_width: f32,
) -> AdaptiveSplit<'a, M> {
    AdaptiveSplit {
        children: vec![sidebar.into(), content.into()],
        wide,
        sidebar_width,
        drawer_width,
        drawer_open: false,
        on_close: None,
        window_radius: 0.0,
    }
}

impl<'a, M> AdaptiveSplit<'a, M> {
    pub fn drawer_open(mut self, open: bool) -> Self {
        self.drawer_open = open;
        self
    }
    /// Sent when the scrim behind an open drawer is pressed.
    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
        self
    }
    /// Rounds the drawer scrim to the window's corners.
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = r;
        self
    }
}

struct SplitState {
    /// 1 when the sidebar sits beside the content, 0 when it does not.
    slot: Progress,
    /// 1 when the drawer is fully open.
    drawer: Progress,
}

fn spring() -> Spec {
    Spec::Spring(presets::sidebar())
}

impl<'a, M: Clone> Widget<M, Theme, iced::Renderer> for AdaptiveSplit<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SplitState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(SplitState {
            slot: Progress::new(if self.wide { 1.0 } else { 0.0 }),
            drawer: Progress::new(if self.drawer_open && !self.wide {
                1.0
            } else {
                0.0
            }),
        })
    }
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let max = limits.max();
        let (slot, drawer) = {
            let st = tree.state.downcast_ref::<SplitState>();
            (
                st.slot.shown.clamp(0.0, 1.0),
                st.drawer.shown.clamp(0.0, 1.0),
            )
        };
        let in_flow = self.wide || slot > 0.001;
        let side_w = if in_flow {
            self.sidebar_width
        } else {
            self.drawer_width
        };
        let fixed =
            |w: f32| layout::Limits::new(Size::new(w, max.height), Size::new(w, max.height));

        let (a, b) = tree.children.split_at_mut(1);
        let sidebar = self.children[0]
            .as_widget_mut()
            .layout(&mut a[0], renderer, &fixed(side_w));
        // The content gives up exactly the space the sidebar has slid in.
        let taken = self.sidebar_width * slot;
        let content_w = (max.width - taken).max(0.0);
        let content = self.children[1]
            .as_widget_mut()
            .layout(&mut b[0], renderer, &fixed(content_w))
            .move_to(Point::new(taken, 0.0));
        let side_x = if in_flow {
            -self.sidebar_width * (1.0 - slot)
        } else {
            -side_w * (1.0 - drawer)
        };
        layout::Node::with_children(max, vec![sidebar.move_to(Point::new(side_x, 0.0)), content])
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
        let (slot, drawer) = {
            let st = tree.state.downcast_mut::<SplitState>();
            if let Some(now) = redraw_time(event) {
                let moving = st.slot.drive(
                    if self.wide { 1.0 } else { 0.0 },
                    now,
                    Trigger::Layout.spec(spring()),
                ) | st.drawer.drive(
                    if self.drawer_open && !self.wide {
                        1.0
                    } else {
                        0.0
                    },
                    now,
                    Trigger::Drawer.spec(spring()),
                );
                if moving {
                    // The content's width changes every frame while the sidebar slides.
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
            }
            (st.slot.shown, st.drawer.shown)
        };
        let mut kids = layout.children();
        let (side_l, content_l) = (kids.next().unwrap(), kids.next().unwrap());
        let (a, b) = tree.children.split_at_mut(1);
        let drawer_visible = drawer > 0.001 && slot <= 0.001;
        // Keys reach the sidebar even when it is hidden: a menu button in its header registers
        // its shortcuts, which must work with the drawer closed.
        let keyboard = matches!(event, Event::Keyboard(_));

        if drawer_visible {
            // Settled: the drawer's own widgets work. Anything else is the scrim.
            if (drawer - 1.0).abs() < 0.001 || keyboard {
                self.children[0].as_widget_mut().update(
                    &mut a[0], event, side_l, cursor, renderer, clipboard, shell, viewport,
                );
            }
            if matches!(event, Event::Mouse(_) | Event::Touch(_)) && !shell.is_event_captured() {
                if matches!(
                    event,
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                        | Event::Touch(iced::touch::Event::FingerPressed { .. })
                ) && !cursor.is_over(side_l.bounds())
                    && let Some(m) = &self.on_close
                {
                    shell.publish(m.clone());
                }
                if cursor.is_over(layout.bounds()) {
                    shell.capture_event();
                }
            }
            return;
        }
        if slot > 0.001 || keyboard {
            self.children[0].as_widget_mut().update(
                &mut a[0], event, side_l, cursor, renderer, clipboard, shell, viewport,
            );
        }
        self.children[1].as_widget_mut().update(
            &mut b[0], event, content_l, cursor, renderer, clipboard, shell, viewport,
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
        let st = tree.state.downcast_ref::<SplitState>();
        let mut kids = layout.children();
        let (side_l, content_l) = (kids.next().unwrap(), kids.next().unwrap());
        let drawer_visible = st.drawer.shown > 0.001 && st.slot.shown <= 0.001;
        if drawer_visible || (st.slot.shown > 0.001 && cursor.is_over(side_l.bounds())) {
            return self.children[0].as_widget().mouse_interaction(
                &tree.children[0],
                side_l,
                cursor,
                viewport,
                renderer,
            );
        }
        self.children[1].as_widget().mouse_interaction(
            &tree.children[1],
            content_l,
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
        let mut kids = layout.children();
        let (side_l, content_l) = (kids.next().unwrap(), kids.next().unwrap());
        let (a, b) = tree.children.split_at_mut(1);
        self.children[0]
            .as_widget_mut()
            .operate(&mut a[0], side_l, renderer, operation);
        self.children[1]
            .as_widget_mut()
            .operate(&mut b[0], content_l, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        // Popovers from the sidebar's header (the primary menu) and from the content.
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
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
        let st = tree.state.downcast_ref::<SplitState>();
        let (slot, drawer) = (
            st.slot.shown.clamp(0.0, 1.0),
            st.drawer.shown.clamp(0.0, 1.0),
        );
        let b = layout.bounds();
        let mut kids = layout.children();
        let (side_l, content_l) = (kids.next().unwrap(), kids.next().unwrap());

        self.children[1].as_widget().draw(
            &tree.children[1],
            renderer,
            theme,
            style,
            content_l,
            cursor,
            viewport,
        );

        if slot > 0.001 {
            // In flow: the part of the sidebar that has slid in, clipped to its slot.
            let visible = Rectangle {
                x: b.x,
                y: b.y,
                width: self.sidebar_width * slot,
                height: b.height,
            };
            renderer.with_layer(visible, |r| {
                self.children[0].as_widget().draw(
                    &tree.children[0],
                    r,
                    theme,
                    style,
                    side_l,
                    cursor,
                    viewport,
                );
                r.fill_quad(
                    Quad {
                        bounds: Rectangle {
                            x: visible.x + visible.width - 1.0,
                            width: 1.0,
                            ..visible
                        },
                        border: Border::default(),
                        shadow: Default::default(),
                        snap: false,
                    },
                    Background::Color(theme.semantic.sidebar_border),
                );
            });
        } else if drawer > 0.001 {
            // A drawer: over a fading scrim, with a shadow.
            renderer.with_layer(b, |r| {
                r.fill_quad(
                    Quad {
                        bounds: b,
                        border: Border::default().rounded(self.window_radius),
                        shadow: Default::default(),
                        snap: false,
                    },
                    Background::Color(scrim(theme, drawer)),
                );
            });
            renderer.with_layer(b, |r| {
                let pane = side_l.bounds();
                r.fill_quad(
                    Quad {
                        bounds: pane,
                        border: Border::default().rounded(iced::border::Radius {
                            top_left: self.window_radius,
                            top_right: 0.0,
                            bottom_right: 0.0,
                            bottom_left: self.window_radius,
                        }),
                        shadow: theme.shadows.dialog().shadow,
                        snap: false,
                    },
                    Background::Color(Color::TRANSPARENT),
                );
                self.children[0].as_widget().draw(
                    &tree.children[0],
                    r,
                    theme,
                    style,
                    side_l,
                    cursor,
                    viewport,
                );
            });
        }
    }
}

impl<'a, M: Clone + 'a> From<AdaptiveSplit<'a, M>> for Element<'a, M, Theme> {
    fn from(s: AdaptiveSplit<'a, M>) -> Self {
        Element::new(s)
    }
}
