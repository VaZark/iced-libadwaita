//! `Spoiler`: show part of something and reveal the rest, after Mantine's
//! [Spoiler](https://mantine.dev/core/spoiler/). The rules (when a control is needed, how many
//! items are shown) are the [`spoiler`](crate::primitives::spoiler) primitive.
//!
//! Two forms, for the two kinds of "part", each also as a **preview**
//! ([`preview`](Spoiler::preview)): the part is all there is, with no control and no reveal (a
//! note card showing the start of its text).
//!
//! * **Part of the height.** [`Spoiler::new`]`(content)` clips its content to
//!   [`max_height`](Spoiler::max_height), fades the cut, and shows a Show more / Hide control
//!   under it, only when the content is taller than that. It slides open with the theme's reveal
//!   motion. Controlled: the app owns `expanded` and takes it from `on_toggle`.
//! * **Part of the data.** [`Spoiler::items`]`(&state, build)` builds only the first
//!   `state.shown()` items and a control for the rest. The hidden items are never built or laid
//!   out, so a long list costs what its visible part costs. A press reveals the next chunk (or
//!   all), and Hide goes back; apply the [`SpoilerEvent`] with [`SpoilerState::apply`].
//!
//! ```ignore
//! Spoiler::new(long_text).max_height(80.0).expanded(self.open).on_toggle(Msg::Open)
//!
//! Spoiler::items(&self.shown, |i| row_for(&self.data[i])).on_event(Msg::Spoiler)
//! // update: Msg::Spoiler(e) => { self.shown.apply(e); }
//! ```

use crate::Element;
use crate::components::flat_button;
use crate::components::typography::label as text_label;
use crate::layouts::animation::{Progress, Spec, Trigger};
use crate::primitives::spoiler::{self, Event, SpoilerState};
use crate::{Metrics, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell, overlay};
use iced::gradient::Linear;
use iced::widget::{column, row};
use iced::{
    Background, Border, Color, Event as IcedEvent, Length, Point, Radians, Rectangle, Shadow, Size,
    Vector, mouse,
};
use std::rc::Rc;

/// The height of the fade over the cut.
const FADE: f32 = 36.0;

/// The height-clamped spoiler. See the [module](self).
pub struct Spoiler<'a, M> {
    props: SpoilerWidgetProps,
    content: Element<'a, M>,
    max_height: f32,
    expanded: bool,
    preview: bool,
    on_toggle: Option<Rc<dyn Fn(bool) -> M + 'a>>,
    show: String,
    hide: String,
    width: Length,
    gap: Option<f32>,
    metrics: Metrics,
    duration: Option<std::time::Duration>,
}

impl<'a, M: Clone + 'a> Spoiler<'a, M> {
    /// Clips `content` to a maximum height (default 100) with a control for the rest.
    /// Presentation overrides for native children; actions remain parent-owned.
    pub fn props(mut self, props: SpoilerWidgetProps) -> Self {
        self.props = props;
        self
    }
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            props: SpoilerWidgetProps::default(),
            content: content.into(),
            max_height: 100.0,
            expanded: false,
            preview: false,
            on_toggle: None,
            show: "Show more".into(),
            hide: "Hide".into(),
            width: Length::Fill,
            gap: None,
            metrics: Metrics::desktop(),
            duration: None,
        }
    }
    /// The partial-data form: only the visible items of `state` are built.
    pub fn items(
        state: &'a SpoilerState,
        build: impl Fn(usize) -> Element<'a, M> + 'a,
    ) -> SpoilerList<'a, M> {
        SpoilerList {
            props: SpoilerWidgetProps::default(),
            state,
            build: Box::new(build),
            on_event: None,
            preview: false,
            show: None,
            hide: "Hide".into(),
            spacing: Metrics::desktop().space_s,
        }
    }
    /// How long it takes to open or close (default the theme's reveal motion).
    pub fn duration(mut self, d: std::time::Duration) -> Self {
        self.duration = Some(d);
        self
    }
    pub fn max_height(mut self, px: f32) -> Self {
        self.max_height = px.max(1.0);
        self
    }
    /// Preview mode: only the first [`max_height`](Self::max_height) of the content shows, always.
    /// No Show more / Hide control, no fade, nothing to toggle; the clipped part is neither drawn
    /// nor reachable by the pointer.
    pub fn preview(mut self, preview: bool) -> Self {
        self.preview = preview;
        self
    }
    /// Whether the whole content shows (controlled).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
    /// The control was pressed: the state the spoiler would take.
    pub fn on_toggle(mut self, f: impl Fn(bool) -> M + 'a) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }
    /// The gap between the content and the show/hide control (default `space_xs`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.gap = Some(px);
        self
    }
    pub fn labels(mut self, show: impl Into<String>, hide: impl Into<String>) -> Self {
        self.show = show.into();
        self.hide = hide.into();
        self
    }
    /// The width: `Fill` (the default) or a fixed width. The clamp cannot shrink to its content,
    /// so `Shrink` fills too.
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

struct Clamp<'a, M> {
    /// `[content, control]`.
    children: Vec<Element<'a, M>>,
    max_height: f32,
    expanded: bool,
    /// Never reveals and has no control.
    preview: bool,
    gap: f32,
    width: Length,
    duration: Option<std::time::Duration>,
}

struct State {
    /// 0 collapsed, 1 expanded.
    progress: Progress,
    overflow: bool,
    content_height: f32,
}

impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for Clamp<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            // Already in its state the first time it is drawn: no slide on arrival.
            progress: Progress::new(if self.expanded { 1.0 } else { 0.0 }),
            overflow: false,
            content_height: 0.0,
        })
    }
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }
    fn size(&self) -> Size<Length> {
        Size::new(
            match self.width {
                Length::Fixed(px) => Length::Fixed(px),
                _ => Length::Fill,
            },
            Length::Shrink,
        )
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = match self.width {
            Length::Fixed(px) => px.min(limits.max().width),
            _ => limits.max().width,
        };
        let content = self.children[0].as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY)),
        );
        let state = tree.state.downcast_mut::<State>();
        let content_h = content.size().height;
        state.content_height = content_h;
        state.overflow = spoiler::overflows(content_h, self.max_height);
        let visible = spoiler::height(content_h, self.max_height, state.progress.shown);
        if self.preview {
            // Only the first part is the widget's size.
            let h = visible;
            return layout::Node::with_children(Size::new(width, h), vec![content]);
        }
        let mut nodes = vec![content];
        let mut total = visible;
        if state.overflow && !self.preview {
            let control = self.children[1]
                .as_widget_mut()
                .layout(
                    &mut tree.children[1],
                    renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY)),
                )
                .move_to(Point::new(0.0, visible + self.gap));
            total = visible + self.gap + control.size().height;
            nodes.push(control);
        }
        layout::Node::with_children(Size::new(width, total), nodes)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &IcedEvent,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        crate::layouts::motion::advance(
            &mut state.progress,
            event,
            if self.expanded { 1.0 } else { 0.0 },
            Trigger::Reveal.spec(match self.duration {
                Some(d) => Spec::STYLE.with_duration(d),
                None => Spec::STYLE,
            }),
            shell,
            true,
        );
        let (overflow, content_h) = (state.overflow, state.content_height);
        let b = layout.bounds();
        let mut kids = layout.children();
        let content_l = kids.next().expect("a content layout");
        // The clipped part is not there: the pointer does not reach it.
        let visible = Rectangle::new(
            b.position(),
            Size::new(
                b.width,
                spoiler::height(content_h, self.max_height, state.progress.shown),
            ),
        );
        let over_content = if cursor.position().is_some_and(|p| visible.contains(p)) {
            cursor
        } else {
            mouse::Cursor::Unavailable
        };
        self.children[0].as_widget_mut().update(
            &mut tree.children[0],
            event,
            content_l,
            over_content,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if overflow && let Some(control_l) = kids.next() {
            self.children[1].as_widget_mut().update(
                &mut tree.children[1],
                event,
                control_l,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let mut kids = layout.children();
        let content = self.children[0].as_widget().mouse_interaction(
            &tree.children[0],
            kids.next().expect("a content layout"),
            cursor,
            viewport,
            renderer,
        );
        match (state.overflow, kids.next()) {
            (true, Some(l)) => content.max(self.children[1].as_widget().mouse_interaction(
                &tree.children[1],
                l,
                cursor,
                viewport,
                renderer,
            )),
            _ => content,
        }
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
        let state = tree.state.downcast_ref::<State>();
        let b = layout.bounds();
        let mut kids = layout.children();
        let content_l = kids.next().expect("a content layout");
        let t = state.progress.shown;
        let visible = Rectangle::new(
            b.position(),
            Size::new(
                b.width,
                spoiler::height(state.content_height, self.max_height, t),
            ),
        );
        // Only the visible part is drawn: the content gets the cut as its viewport, so what is
        // outside it is skipped, not drawn and clipped.
        let in_view = visible.intersection(viewport).unwrap_or(Rectangle::new(
            visible.position(),
            Size::ZERO,
        ));
        renderer.with_layer(visible, |renderer| {
            self.children[0].as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                content_l,
                cursor,
                &in_view,
            );
        });
        // The cut fades into the page until it is fully open.
        if state.overflow && !self.preview && t < 0.999 {
            let h = FADE.min(visible.height);
            let page = theme.semantic.window.bg;
            let fade = Linear::new(Radians(std::f32::consts::PI))
                .add_stop(0.0, Color { a: 0.0, ..page })
                .add_stop(
                    1.0,
                    Color {
                        a: page.a * (1.0 - t),
                        ..page
                    },
                );
            // In a layer of its own, above the content's.
            renderer.with_layer(visible, |renderer| {
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point::new(visible.x, visible.y + visible.height - h),
                            Size::new(visible.width, h),
                        ),
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    Background::Gradient(fade.into()),
                );
            });
        }
        if state.overflow
            && let Some(control_l) = kids.next()
        {
            self.children[1].as_widget().draw(
                &tree.children[1],
                renderer,
                theme,
                style,
                control_l,
                cursor,
                viewport,
            );
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        let mut kids = layout.children();
        self.children[0].as_widget_mut().operate(
            &mut tree.children[0],
            kids.next().expect("a content layout"),
            renderer,
            operation,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.children[0].as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next()?,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, M: Clone + 'a> From<Spoiler<'a, M>> for Element<'a, M> {
    fn from(s: Spoiler<'a, M>) -> Self {
        let control: Element<'a, M> = s
            .props
            .toggle_button
            .apply(
                flat_button(s.props.toggle_label.apply(text_label(if s.expanded {
                    s.hide.clone()
                } else {
                    s.show.clone()
                })))
                .on_press_maybe(s.on_toggle.as_ref().map(|f| f(!s.expanded))),
            )
            .into();
        iced::Element::new(Clamp {
            children: vec![s.content, control],
            max_height: s.max_height,
            expanded: s.expanded && !s.preview,
            preview: s.preview,
            gap: s.gap.unwrap_or(s.metrics.space_xs),
            width: s.width,
            duration: s.duration,
        })
    }
}

/// The partial-data spoiler: the visible items and the control for the rest.
pub struct SpoilerList<'a, M> {
    props: SpoilerWidgetProps,
    state: &'a SpoilerState,
    build: Box<dyn Fn(usize) -> Element<'a, M> + 'a>,
    on_event: Option<Rc<dyn Fn(Event) -> M + 'a>>,
    preview: bool,
    show: Option<String>,
    hide: String,
    spacing: f32,
}

impl<'a, M: Clone + 'a> SpoilerList<'a, M> {
    pub fn props(mut self, props: SpoilerWidgetProps) -> Self {
        self.props = props;
        self
    }

    /// Preview mode: the first `state.shown()` items and no control.
    pub fn preview(mut self, preview: bool) -> Self {
        self.preview = preview;
        self
    }
    /// The control was pressed: apply it with [`SpoilerState::apply`].
    pub fn on_event(mut self, f: impl Fn(Event) -> M + 'a) -> Self {
        self.on_event = Some(Rc::new(f));
        self
    }
    /// The labels of the control (default "Show N more" and "Hide").
    pub fn labels(mut self, show: impl Into<String>, hide: impl Into<String>) -> Self {
        self.show = Some(show.into());
        self.hide = hide.into();
        self
    }
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = px;
        self
    }
}

impl<'a, M: Clone + 'a> From<SpoilerList<'a, M>> for Element<'a, M> {
    fn from(l: SpoilerList<'a, M>) -> Self {
        // Only the visible items are built.
        let items: Vec<Element<'a, M>> = l.state.visible().map(|i| (l.build)(i)).collect();
        let mut col = column(items).spacing(l.spacing).width(Length::Fill);
        if l.state.needs_control() && !l.preview {
            let press = |e: Event| l.on_event.as_ref().map(|f| f(e));
            let mut controls = row![].spacing(l.spacing);
            if l.state.can_expand() {
                let label = l
                    .show
                    .clone()
                    .unwrap_or_else(|| format!("Show {} more", l.state.next_chunk()));
                controls = controls.push(
                    l.props.more_button.apply(
                        flat_button(l.props.more_label.apply(text_label(label)))
                            .on_press_maybe(press(Event::More)),
                    ),
                );
            }
            if l.state.can_collapse() {
                controls = controls.push(
                    l.props.less_button.apply(
                        flat_button(l.props.less_label.apply(text_label(l.hide.clone())))
                            .on_press_maybe(press(Event::Less)),
                    ),
                );
            }
            col = col.push(controls);
        }
        col.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::{space, text};

    #[test]
    fn only_the_visible_items_are_built() {
        use std::cell::Cell;
        let built = Cell::new(0);
        let state = SpoilerState::new(1000, 5).step(10);
        let _: Element<'_, ()> = Spoiler::items(&state, |_| {
            built.set(built.get() + 1);
            text("row").into()
        })
        .into();
        assert_eq!(built.get(), 5, "the other 995 are never built");
        let mut more = SpoilerState::new(1000, 5).step(10);
        more.more();
        built.set(0);
        let _: Element<'_, ()> = Spoiler::items(&more, |_| {
            built.set(built.get() + 1);
            text("row").into()
        })
        .into();
        assert_eq!(built.get(), 15);
    }

    #[test]
    fn a_short_list_has_no_control_and_a_clamp_builds() {
        let state = SpoilerState::new(3, 5);
        let _: Element<'_, ()> = Spoiler::items(&state, |_| text("row").into()).into();
        let _: Element<'_, u8> = Spoiler::new(space().height(300.0))
            .max_height(80.0)
            .expanded(true)
            .on_toggle(|open| open as u8)
            .into();
    }
}

#[cfg(test)]
mod preview_tests {
    use super::*;
    use iced::widget::{space, text};

    #[test]
    fn a_preview_has_no_control_and_is_as_tall_as_its_window() {
        use iced::advanced::{layout, renderer::Headless, widget::Tree};
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        // The control of the open form makes the widget taller than its window.
        let height = |preview: bool| {
            let mut el: Element<'_, ()> = Spoiler::new(space().height(300.0))
                .max_height(80.0)
                .preview(preview)
                .into();
            let mut tree = Tree::new(&el);
            el.as_widget_mut()
                .layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(200.0, 1000.0)),
                )
                .size()
                .height
        };
        assert_eq!(height(true), 80.0);
        assert!(height(false) > 80.0);
    }

    #[test]
    fn a_preview_list_builds_only_the_shown_items() {
        let state = SpoilerState::new(1000, 4);
        let _: Element<'_, ()> = Spoiler::items(&state, |_| text("row").into())
            .preview(true)
            .into();
    }
}

#[cfg(test)]
mod spacing_params {
    use super::*;

    #[test]
    fn the_control_gap_is_a_parameter() {
        let s = Spoiler::<()>::new(iced::widget::text("x")).spacing(11.0);
        assert_eq!(s.gap, Some(11.0));
        let _: Element<'_, ()> = s.into();
    }
}

/// Typed presentation for parent-built native children. Defaults preserve the original view.
#[derive(Clone, Default)]
pub struct SpoilerWidgetProps {
    pub toggle_button: crate::components::ButtonProps,
    pub toggle_label: crate::components::TextProps,
    pub more_button: crate::components::ButtonProps,
    pub more_label: crate::components::TextProps,
    pub less_button: crate::components::ButtonProps,
    pub less_label: crate::components::TextProps,
}

impl SpoilerWidgetProps {
    pub fn toggle_button(mut self, props: crate::components::ButtonProps) -> Self {
        self.toggle_button = props;
        self
    }
    pub fn toggle_label(mut self, props: crate::components::TextProps) -> Self {
        self.toggle_label = props;
        self
    }
    pub fn more_button(mut self, props: crate::components::ButtonProps) -> Self {
        self.more_button = props;
        self
    }
    pub fn more_label(mut self, props: crate::components::TextProps) -> Self {
        self.more_label = props;
        self
    }
    pub fn less_button(mut self, props: crate::components::ButtonProps) -> Self {
        self.less_button = props;
        self
    }
    pub fn less_label(mut self, props: crate::components::TextProps) -> Self {
        self.less_label = props;
        self
    }
    pub fn apply<'a, M>(&self, widget: Spoiler<'a, M>) -> Spoiler<'a, M>
    where
        M: Clone + 'a,
    {
        let mut widget = widget;
        widget.props.merge(self);
        widget
    }
}

#[cfg(test)]
mod native_props_tests {
    use super::*;
    use crate::widgets::synced_text::native_props_tests::{assert_override, text_props};
    #[test]
    fn defaults_preserve_layout_and_overrides_reach_native_children() {
        let state = SpoilerState::new(10, 2);
        let make = || Spoiler::<()>::items(&state, |_| iced::widget::space().height(10).into());
        assert_override(
            make().into(),
            make().props(SpoilerWidgetProps::default()).into(),
            make()
                .props(SpoilerWidgetProps::default().more_label(text_props()))
                .into(),
        );
        let make = || Spoiler::<()>::new(iced::widget::space().height(300)).max_height(80.0);
        assert_override(
            make().into(),
            make().props(SpoilerWidgetProps::default()).into(),
            make()
                .props(SpoilerWidgetProps::default().toggle_label(text_props()))
                .into(),
        );
    }
}

crate::components::props::impl_merge!(SpoilerWidgetProps; fields: []; optional: []; nested: [toggle_button, toggle_label, more_button, more_label, less_button, less_label]);
