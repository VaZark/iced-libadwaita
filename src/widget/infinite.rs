//! `InfiniteList`: content that grows as the user scrolls to its end, or presses "Load more".
//! The state (items loaded, one request in flight, stale answers discarded, the scroll rule) is
//! [`primitives::infinite`](crate::primitives::infinite); this widget draws the **footer** for
//! each status and tells the app when to load.
//!
//! ```ignore
//! InfiniteList::new(rows_column, &self.items)        // &Infinite<Query, Row>
//!     .on_load_more(Msg::More)                       // -> self.items.load_next(source, Msg::Loaded)
//!     .on_retry(Msg::Retry)                          // -> self.items.load_retry(source, Msg::Loaded)
//!     .trigger(LoadTrigger::Scroll)                  // or LoadTrigger::LoadMore (a button)
//! ```
//!
//! | Status      | Footer                                                  |
//! | ----------- | ------------------------------------------------------- |
//! | `Idle`      | nothing (scroll) or a "Load more" button                 |
//! | `Loading`   | a spinner and "Loading..."                               |
//! | `Failed`    | the message and a Retry button                           |
//! | `Exhausted` | "No more results" ("No results" if nothing was loaded)  |
//!
//! **When it asks.** `on_load_more` is published only while the state is `Idle` (so never while
//! loading, failed or exhausted) and, for [`LoadTrigger::Scroll`], while the footer is within
//! `threshold` pixels of the bottom of the visible area, decided by
//! [`should_load_more`](crate::primitives::infinite::should_load_more). That includes content
//! that does not fill the viewport yet, where a scrollable never reports a scroll position (iced
//! only notifies `on_scroll` when the content overflows): the footer itself is the sentinel, and
//! it fires again after each page lands, until the viewport is full. The message is published
//! once per idle period.
//!
//! The list scrolls by itself ([`scrolled_window`](super::scrolled_window), filling its
//! container); [`scrolled(false)`](InfiniteList::scrolled) returns content plus footer for a
//! parent that scrolls.

use super::Element;
use crate::primitives::infinite::{Infinite, ScrollMetrics, Status, should_load_more};
use crate::{Metrics, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::alignment::Vertical;
use iced::widget::{column, container, row, space};
use iced::{Event, Length, Rectangle, Size, Vector, mouse};

/// What asks for the next page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadTrigger {
    /// Reaching the end of the content (infinite scroll).
    #[default]
    Scroll,
    /// A "Load more" button (Mantine's usual alternative, and the accessible one).
    LoadMore,
}

/// See the [module](self).
pub struct InfiniteList<'a, M> {
    content: Element<'a, M>,
    status: Status,
    empty: bool,
    on_load_more: Option<M>,
    on_retry: Option<M>,
    trigger: LoadTrigger,
    threshold: f32,
    scrolled: bool,
    height: Length,
    rotation: f32,
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> InfiniteList<'a, M> {
    /// `content` (the rows loaded so far) with the footer for `state`.
    pub fn new<Q: Clone, T>(content: impl Into<Element<'a, M>>, state: &Infinite<Q, T>) -> Self {
        Self {
            content: content.into(),
            status: state.status().clone(),
            empty: state.is_empty(),
            on_load_more: None,
            on_retry: None,
            trigger: LoadTrigger::Scroll,
            threshold: 200.0,
            scrolled: true,
            height: Length::Fill,
            rotation: 0.0,
            metrics: Metrics::desktop(),
        }
    }
    /// Sent when the next page should load.
    pub fn on_load_more(mut self, msg: M) -> Self {
        self.on_load_more = Some(msg);
        self
    }
    /// Sent by the Retry button of a failed footer.
    pub fn on_retry(mut self, msg: M) -> Self {
        self.on_retry = Some(msg);
        self
    }
    pub fn trigger(mut self, t: LoadTrigger) -> Self {
        self.trigger = t;
        self
    }
    /// How close to the end the footer must come before `on_load_more` (default 200 px).
    pub fn threshold(mut self, px: f32) -> Self {
        self.threshold = px;
        self
    }
    /// Scroll by itself (default), or return content and footer for an outer scrollable.
    pub fn scrolled(mut self, scrolled: bool) -> Self {
        self.scrolled = scrolled;
        self
    }
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }
    /// The spinner's angle while loading, in radians (drive it from a subscription).
    pub fn rotation(mut self, radians: f32) -> Self {
        self.rotation = radians;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    fn footer(&self) -> Element<'a, M> {
        let m = &self.metrics;
        let dim = |s: String| -> Element<'a, M> {
            super::typography::caption(s)
                .style(crate::style::text::dim)
                .into()
        };
        let inner: Element<'a, M> = match &self.status {
            Status::Loading { .. } => row![
                super::spinner(m.icon_size + 4.0, self.rotation),
                dim(String::from("Loading\u{2026}")),
            ]
            .spacing(m.space_s)
            .align_y(Vertical::Center)
            .into(),
            Status::Failed(e) => {
                let mut c = column![
                    super::typography::label("Could not load more results"),
                    dim(e.clone()),
                ]
                .spacing(m.space_xs)
                .align_x(iced::Alignment::Center);
                if let Some(retry) = &self.on_retry {
                    c = c.push(
                        super::raised_button(super::typography::label("Retry"))
                            .on_press(retry.clone()),
                    );
                }
                c.spacing(m.space_s).into()
            }
            Status::Exhausted => dim(String::from(if self.empty {
                "No results"
            } else {
                "No more results"
            })),
            Status::Idle => match (self.trigger, &self.on_load_more) {
                (LoadTrigger::LoadMore, Some(more)) => {
                    super::raised_button(super::typography::label("Load more"))
                        .on_press(more.clone())
                        .into()
                }
                _ => space().height(Length::Fixed(1.0)).into(),
            },
        };
        let footer = container(inner)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(m.space_m);
        match (&self.status, self.trigger, &self.on_load_more) {
            (Status::Idle, LoadTrigger::Scroll, Some(more)) => Element::new(EndTrigger {
                child: footer.into(),
                on_end: more.clone(),
                threshold: self.threshold,
            }),
            _ => footer.into(),
        }
    }
}

impl<'a, M: Clone + 'a> From<InfiniteList<'a, M>> for Element<'a, M> {
    fn from(list: InfiniteList<'a, M>) -> Self {
        let footer = list.footer();
        let body = column![list.content, footer].width(Length::Fill);
        if list.scrolled {
            super::scrolled_window(body).height(list.height).into()
        } else {
            body.into()
        }
    }
}

/// Wraps the footer of an idle scroll-triggered list and publishes `on_end` when it comes
/// within `threshold` of the bottom of the visible area, once per idle period (the widget is
/// swapped out of the tree while loading, so its state resets).
struct EndTrigger<'a, M> {
    child: Element<'a, M>,
    on_end: M,
    threshold: f32,
}

#[derive(Default)]
struct Fired(bool);

impl<'a, M: Clone> Widget<M, Theme, iced::Renderer> for EndTrigger<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Fired>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(Fired::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.child)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.child));
    }
    fn size(&self) -> Size<Length> {
        self.child.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.child
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
        self.child.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        let fired = &mut tree.state.downcast_mut::<Fired>().0;
        if *fired
            || !matches!(
                event,
                Event::Window(iced::window::Event::RedrawRequested(_))
            )
        {
            return;
        }
        // The footer is the last thing in the content, so its bottom edge is the content's end
        // and `viewport` (in content coordinates) is the visible part.
        let b = layout.bounds();
        let metrics = ScrollMetrics {
            offset: viewport.y,
            viewport_h: viewport.height,
            content_h: b.y + b.height,
        };
        if should_load_more(metrics, self.threshold) {
            *fired = true;
            shell.publish(self.on_end.clone());
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
        self.child.as_widget().mouse_interaction(
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
        self.child
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.child.as_widget_mut().overlay(
            &mut tree.children[0],
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
        self.child.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }
}
