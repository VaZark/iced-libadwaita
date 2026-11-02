//! `Autocomplete`: an entry whose suggestions come from an asynchronous lookup (a database, an
//! HTTP call), shown in a popover under it.
//!
//! A thin applier of [`primitives::lookup`](crate::primitives::lookup): the app owns the text
//! and the [`Lookup`] state, and drives the async side with [`crate::lookup`]. The widget draws
//! what the lookup's status says and reports what the user does:
//!
//! | Status | Popover |
//! | --- | --- |
//! | `Idle` | closed |
//! | `Waiting` / `Loading`, no results yet | "Searching…" with a spinner |
//! | `Waiting` / `Loading`, previous results | those results (the entry shows a spinner) |
//! | `Loaded` | the suggestions, or "No results" |
//! | `Failed` | the error and a Retry button ([`on_retry`](Autocomplete::on_retry)) |
//!
//! ```ignore
//! Autocomplete::new(&self.text, &self.cities, Msg::Input)
//!     .label(|c: &City| c.name.clone())
//!     .on_select(Msg::Picked)
//!     .on_retry(Msg::Retry)
//! ```
//!
//! # Interaction
//!
//! The popover opens when the user types or clicks the entry and there is something to show,
//! and closes on a pick, Escape, or a click elsewhere. It is as wide as the entry and aligned
//! to its start edge. Up / Down move the highlight (wrapping), Enter picks the highlighted
//! suggestion, Escape closes. Hovering a row highlights it and a click picks it. Keys are read
//! by the popover while it is open, so they work whatever the focus; Down reopens a closed
//! popover only while the entry was last clicked in (the widget cannot ask the text input
//! whether it has focus). Without a highlight Enter does nothing here (the text is yours).
//!
//! After a pick, `on_select(item)` fires; set the text to the item's label yourself. The widget
//! does not reopen the popover for that change.
//!
//! # Free text
//!
//! [`free_text(true)`](Autocomplete::free_text) (the default) accepts any text. With `false`
//! the text must be the label of a suggestion the user picked: keep the picked label in your
//! state and pass it to [`selected`](Autocomplete::selected); the entry then draws its error
//! ring once [`revealed`](Autocomplete::revealed), and
//! [`validity`](Autocomplete::validity) / [`lookup::validate_pick`](crate::primitives::lookup::validate_pick)
//! give the same [`Validity`] for a [`Field`](super::Field) message or a form.

use super::Element;
use super::{EntryState, entry, section_button, section_icon};
use crate::primitives::anchor::{self, Placement, Side};
use crate::primitives::lookup::{Highlight, Lookup, Status, validate_pick};
use crate::primitives::validation::{self, Validity};
use crate::style::{button as sb, container as sc};
use crate::{Icon, Metrics, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::keyboard::{self, key::Named};
use iced::widget::canvas;
use iced::widget::{button, container, row, space, text};
use iced::{Event, Length, Point, Rectangle, Size, Vector, mouse};
use std::rc::Rc;
use std::time::Instant;

/// Rows shown at once by default; the highlight scrolls the window past it.
pub const MAX_ROWS: usize = 8;

type ItemFn<'a, T, M> = Box<dyn Fn(&T) -> Element<'a, M> + 'a>;
type LabelFn<'a, T> = Box<dyn Fn(&T) -> String + 'a>;

pub struct Autocomplete<'a, T, M> {
    value: &'a str,
    lookup: &'a Lookup<String, T>,
    on_input: Box<dyn Fn(String) -> M + 'a>,
    on_select: Option<Box<dyn Fn(T) -> M + 'a>>,
    on_retry: Option<M>,
    item: Option<ItemFn<'a, T, M>>,
    label: Option<LabelFn<'a, T>>,
    selected: Option<&'a str>,
    placeholder: String,
    disabled: bool,
    width: Length,
    metrics: Metrics,
    revealed: bool,
    free_text: bool,
    pick_message: String,
    search_icon: bool,
    max_rows: usize,
    open_initially: bool,
}

impl<'a, T, M> Autocomplete<'a, T, M> {
    /// `value` is the entry's text, `lookup` the state of the suggestions for it, `on_input`
    /// receives every edit (also the clear button's empty string).
    pub fn new(
        value: &'a str,
        lookup: &'a Lookup<String, T>,
        on_input: impl Fn(String) -> M + 'a,
    ) -> Self {
        Self {
            value,
            lookup,
            on_input: Box::new(on_input),
            on_select: None,
            on_retry: None,
            item: None,
            label: None,
            selected: None,
            placeholder: String::new(),
            disabled: false,
            width: Length::Fill,
            metrics: Metrics::desktop(),
            revealed: true,
            free_text: true,
            pick_message: String::from("Choose one of the suggestions."),
            search_icon: false,
            max_rows: MAX_ROWS,
            open_initially: false,
        }
    }
    /// A suggestion was picked (click, or Enter on the highlighted one).
    pub fn on_select(mut self, f: impl Fn(T) -> M + 'a) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }
    /// Message for the Retry button of the error row.
    pub fn on_retry(mut self, m: M) -> Self {
        self.on_retry = Some(m);
        self
    }
    /// How a suggestion reads as text (its row, unless [`item`](Self::item) renders it
    /// otherwise). Default: `ToString`.
    pub fn label(mut self, f: impl Fn(&T) -> String + 'a) -> Self {
        self.label = Some(Box::new(f));
        self
    }
    /// A custom row for a suggestion (an avatar, a second line). It sits in a 6 px by 12 px
    /// padded row; the popover draws the highlight behind it.
    pub fn item(mut self, f: impl Fn(&T) -> Element<'a, M> + 'a) -> Self {
        self.item = Some(Box::new(f));
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
    /// A magnifier before the text.
    pub fn search_icon(mut self, on: bool) -> Self {
        self.search_icon = on;
        self
    }
    /// A disabled autocomplete is dimmed, cannot be edited and never opens.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
    /// Whether validation may show yet (`:user-invalid`); see
    /// [`DateInput::revealed`](super::DateInput::revealed). Default `true`; inside a
    /// [`Field`](super::Field) pass the field's interaction so a pristine field stays quiet.
    pub fn revealed(mut self, revealed: bool) -> Self {
        self.revealed = revealed;
        self
    }
    /// Whether text not chosen from the list is acceptable (default `true`). With `false`, the
    /// entry is invalid until the text is the label passed to [`selected`](Self::selected).
    pub fn free_text(mut self, free: bool) -> Self {
        self.free_text = free;
        self
    }
    /// The label of the suggestion the user picked, if any (see [`free_text`](Self::free_text)).
    pub fn selected(mut self, label: Option<&'a str>) -> Self {
        self.selected = label;
        self
    }
    /// The error message for text that was not picked (default "Choose one of the
    /// suggestions.").
    pub fn pick_message(mut self, m: impl Into<String>) -> Self {
        self.pick_message = m.into();
        self
    }
    /// Rows shown at once (default 8); more scroll with the highlight.
    pub fn max_rows(mut self, n: usize) -> Self {
        self.max_rows = n.max(1);
        self
    }
    /// Start with the popover open (screenshots, demos).
    pub fn open_initially(mut self, open: bool) -> Self {
        self.open_initially = open;
        self
    }

    /// The validity of the current text under [`free_text`](Self::free_text).
    pub fn validity(&self) -> Validity {
        validate_pick(
            self.value,
            self.selected,
            self.free_text,
            &self.pick_message,
        )
    }
    /// The entry state: an error only once [`revealed`](Self::revealed).
    pub fn text_state(&self) -> EntryState {
        match validation::present(&self.validity(), self.revealed) {
            validation::Presentation::Invalid => EntryState::Error,
            _ => EntryState::Normal,
        }
    }
}

// ------------------------------------------------------------------ the busy spinner

/// A spinner that animates itself from the redraw clock, so the app needs no frame
/// subscription while a lookup is loading.
struct Busy {
    size: f32,
}

#[derive(Default)]
struct BusyState {
    start: Option<Instant>,
    now: Option<Instant>,
}

impl<M> canvas::Program<M, Theme> for Busy {
    type State = BusyState;

    fn update(
        &self,
        state: &mut BusyState,
        event: &Event,
        _: Rectangle,
        _: mouse::Cursor,
    ) -> Option<canvas::Action<M>> {
        if let Event::Window(iced::window::Event::RedrawRequested(now)) = event {
            state.start.get_or_insert(*now);
            state.now = Some(*now);
            return Some(canvas::Action::request_redraw());
        }
        None
    }

    fn draw(
        &self,
        state: &BusyState,
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let turns = match (state.start, state.now) {
            (Some(s), Some(n)) => n.saturating_duration_since(s).as_secs_f32(),
            _ => 0.0,
        };
        let spinner = super::Spinner {
            size: self.size,
            rotation: turns * std::f32::consts::TAU * 0.9,
        };
        <super::Spinner as canvas::Program<M, Theme>>::draw(
            &spinner,
            &(),
            renderer,
            theme,
            bounds,
            cursor,
        )
    }
}

fn busy<'a, M: 'a>(size: f32) -> Element<'a, M> {
    canvas(Busy { size })
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}

// -------------------------------------------------------------------------- the widget

/// What the popover shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Body {
    None,
    /// `n` selectable rows.
    Rows(usize),
    /// One informative element (searching, no results, error).
    Notice,
}

struct State {
    open: bool,
    /// The entry was clicked and no click since went elsewhere.
    focused: bool,
    highlight: Highlight,
    last_value: String,
    /// The label just picked: the text changing to it does not reopen the popover.
    picked: Option<String>,
    version: u64,
}

pub(crate) struct Widgeted<'a, T, M> {
    entry: Element<'a, M>,
    /// The popover's rows, or its single notice.
    items: Vec<Element<'a, M>>,
    labels: Vec<String>,
    choices: &'a [T],
    on_select: Option<Box<dyn Fn(T) -> M + 'a>>,
    body: Body,
    value: &'a str,
    version: u64,
    disabled: bool,
    max_rows: usize,
    open_initially: bool,
    metrics: Metrics,
    width: Length,
}

impl<'a, T: ToString + Clone + 'a, M: Clone + 'a> From<Autocomplete<'a, T, M>> for Element<'a, M> {
    fn from(a: Autocomplete<'a, T, M>) -> Self {
        let m = a.metrics;
        let l = a.lookup;
        let enabled = !a.disabled;
        let loading = enabled && l.is_loading();

        // The entry: optional magnifier, and a spinner / clear button / nothing at the end
        // (always a right section, so the text input keeps its place in the tree and its focus).
        let mut e = entry::<M>(a.placeholder.as_str(), a.value)
            .state(a.text_state())
            .width(Length::Fill)
            .metrics(m);
        let on_input: Rc<dyn Fn(String) -> M + 'a> = Rc::from(a.on_input);
        if enabled {
            let f = on_input.clone();
            e = e.on_input(move |s| f(s));
        }
        if a.search_icon {
            e = e.left_section(section_icon(Icon::Search).disabled(a.disabled).metrics(m));
        }
        let right: Element<'a, M> = if loading {
            busy(m.icon_size)
        } else if enabled && !a.value.is_empty() {
            section_button(Icon::WindowClose, on_input(String::new()))
                .metrics(m)
                .into()
        } else {
            space().into()
        };
        e = e.right_section(right);

        // The popover content, from the lookup's status.
        let results = l.results();
        let labels: Vec<String> = results
            .iter()
            .map(|t| a.label.as_ref().map_or_else(|| t.to_string(), |f| f(t)))
            .collect();
        let notice = |el: Element<'a, M>| -> Element<'a, M> {
            container(el)
                .padding([m.space_s + 2.0, m.space_m])
                .width(Length::Fill)
                .into()
        };
        let dim = crate::style::text::dim;
        let (body, items): (Body, Vec<Element<'a, M>>) = match l.status() {
            _ if !enabled => (Body::None, Vec::new()),
            Status::Idle => (Body::None, Vec::new()),
            Status::Failed(msg) => {
                let mut r = row![
                    text(msg.clone())
                        .style(crate::style::text::error)
                        .width(Length::Fill)
                ]
                .spacing(m.space_s)
                .align_y(iced::alignment::Vertical::Center);
                if let Some(retry) = a.on_retry.clone() {
                    r = r.push(button(text("Retry")).style(sb::flat).on_press(retry));
                }
                (Body::Notice, vec![notice(r.into())])
            }
            Status::Waiting(_) | Status::Loading { .. } | Status::Loaded if !results.is_empty() => {
                let rows = results
                    .iter()
                    .zip(&labels)
                    .map(|(t, label)| {
                        let content: Element<'a, M> = match &a.item {
                            Some(f) => f(t),
                            None => text(label.clone()).wrapping(text::Wrapping::None).into(),
                        };
                        container(content)
                            .padding([m.space_s, m.space_m])
                            .width(Length::Fill)
                            .into()
                    })
                    .collect::<Vec<_>>();
                (Body::Rows(rows.len()), rows)
            }
            Status::Waiting(_) | Status::Loading { .. } => {
                let r = row![busy(m.icon_size), text("Searching…").style(dim)]
                    .spacing(m.space_s)
                    .align_y(iced::alignment::Vertical::Center);
                (Body::Notice, vec![notice(r.into())])
            }
            Status::Loaded => (
                Body::Notice,
                vec![notice(text("No results").style(dim).into())],
            ),
        };

        let w = Widgeted {
            entry: Element::from(e),
            items,
            labels,
            choices: results,
            on_select: a.on_select,
            body,
            value: a.value,
            version: l.version(),
            disabled: a.disabled,
            max_rows: a.max_rows,
            open_initially: a.open_initially,
            metrics: m,
            width: a.width,
        };
        iced::Element::new(w)
    }
}

impl<'a, T: Clone + 'a, M: 'a> Widgeted<'a, T, M> {
    fn shows(&self) -> bool {
        !self.disabled && self.body != Body::None
    }
}

impl<'a, T: Clone + 'a, M: 'a> Widget<M, Theme, iced::Renderer> for Widgeted<'a, T, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            open: self.open_initially,
            focused: false,
            highlight: Highlight::default(),
            last_value: self.value.to_string(),
            picked: None,
            version: self.version,
        })
    }

    fn children(&self) -> Vec<Tree> {
        std::iter::once(&self.entry)
            .chain(&self.items)
            .map(Tree::new)
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let children: Vec<&dyn Widget<M, Theme, iced::Renderer>> = std::iter::once(&self.entry)
            .chain(&self.items)
            .map(|e| e.as_widget())
            .collect();
        tree.diff_children(&children);
        let st = tree.state.downcast_mut::<State>();
        if self.value != st.last_value {
            st.last_value = self.value.to_string();
            // Typing opens the popover; the text becoming the label just picked does not.
            if st.picked.take().as_deref() != Some(self.value) {
                st.open = true;
            }
        }
        if self.version != st.version {
            st.version = self.version;
            st.highlight.reset();
        }
        if self.disabled {
            st.open = false;
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let node = self
            .entry
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        layout::Node::with_children(node.size(), vec![node])
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
        let entry_layout = layout.children().next().unwrap();
        let st = tree.state.downcast_mut::<State>();
        if !self.disabled {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    st.focused = cursor.is_over(entry_layout.bounds());
                    if st.focused && !st.open {
                        st.open = true;
                        shell.invalidate_layout();
                        shell.request_redraw();
                    }
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::ArrowDown),
                    ..
                }) if st.focused && !st.open => {
                    st.open = true;
                    shell.capture_event();
                    shell.invalidate_layout();
                    shell.request_redraw();
                    return;
                }
                _ => {}
            }
        }
        self.entry.as_widget_mut().update(
            &mut tree.children[0],
            event,
            entry_layout,
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
        self.entry.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
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
        self.entry.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
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
        let shown = self.shows();
        let Tree {
            state, children, ..
        } = tree;
        let st = state.downcast_mut::<State>();
        let entry_layout = layout.children().next()?;
        let (entry_tree, item_trees) = children.split_first_mut()?;
        if !(st.open && shown) {
            // Not ours to show: the entry's own overlay, if any.
            return self.entry.as_widget_mut().overlay(
                entry_tree,
                entry_layout,
                renderer,
                viewport,
                translation,
            );
        }
        let b = entry_layout.bounds();
        let anchor = Rectangle {
            x: b.x + translation.x,
            y: b.y + translation.y,
            ..b
        };
        Some(overlay::Element::new(Box::new(Popover {
            items: &mut self.items,
            trees: item_trees,
            body: self.body,
            labels: &self.labels,
            choices: self.choices,
            on_select: self.on_select.as_deref(),
            open: &mut st.open,
            highlight: &mut st.highlight,
            picked: &mut st.picked,
            anchor,
            max_rows: self.max_rows,
            metrics: self.metrics,
            window: 0..0,
        })))
    }
}

struct Popover<'a, 'b, T, M> {
    items: &'b mut [Element<'a, M>],
    trees: &'b mut [Tree],
    body: Body,
    labels: &'b [String],
    choices: &'a [T],
    on_select: Option<&'b (dyn Fn(T) -> M + 'a)>,
    open: &'b mut bool,
    highlight: &'b mut Highlight,
    picked: &'b mut Option<String>,
    anchor: Rectangle,
    max_rows: usize,
    metrics: Metrics,
    /// The rows laid out (indices into `items`), set by `layout`.
    window: std::ops::Range<usize>,
}

const MARGIN: f32 = 6.0;
const GAP: f32 = 4.0;
const ROW_GAP: f32 = 2.0;

impl<'a, 'b, T: Clone + 'a, M: 'a> Popover<'a, 'b, T, M> {
    fn selectable(&self) -> bool {
        matches!(self.body, Body::Rows(_))
    }

    fn close(&mut self, shell: &mut Shell<'_, M>) {
        *self.open = false;
        shell.invalidate_layout();
        shell.request_redraw();
    }

    fn pick(&mut self, i: usize, shell: &mut Shell<'_, M>) {
        if let (Some(f), Some(t)) = (self.on_select, self.choices.get(i)) {
            shell.publish(f(t.clone()));
        }
        *self.picked = self.labels.get(i).cloned();
        *self.open = false;
        shell.capture_event();
        shell.invalidate_layout();
        shell.request_redraw();
    }

    /// The index (into `items`) of the row under the cursor.
    fn row_at(&self, layout: Layout<'_>, cursor: mouse::Cursor) -> Option<usize> {
        layout
            .children()
            .zip(self.window.clone())
            .find(|(l, _)| cursor.is_over(l.bounds()))
            .map(|(_, i)| i)
    }
}

impl<'a, 'b, T: Clone + 'a, M: 'a> overlay::Overlay<M, Theme, iced::Renderer>
    for Popover<'a, 'b, T, M>
{
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let pad = self.metrics.popover_padding;
        let w = self
            .anchor
            .width
            .clamp(120.0, (bounds.width - 2.0 * MARGIN).max(120.0));
        let inner = Size::new(w - 2.0 * pad, f32::INFINITY);
        self.window = match self.body {
            Body::Rows(n) => self.highlight.window(n, self.max_rows),
            _ => 0..self.items.len(),
        };
        let mut y = pad;
        let mut nodes = Vec::new();
        for i in self.window.clone() {
            let node = self.items[i].as_widget_mut().layout(
                &mut self.trees[i],
                renderer,
                &layout::Limits::new(Size::ZERO, inner),
            );
            let h = node.size().height;
            nodes.push(node.move_to(Point::new(pad, y)));
            y += h + ROW_GAP;
        }
        let height = (y - ROW_GAP).max(pad) + pad;
        let size = Size::new(w, height.min((bounds.height - 2.0 * MARGIN).max(40.0)));
        let placed = anchor::place(
            self.anchor,
            size,
            bounds,
            Placement {
                side: Side::Bottom,
                align: anchor::Align::Start,
                offset: GAP,
                margin: MARGIN,
                ..Placement::default()
            },
        );
        layout::Node::with_children(size, nodes).move_to(placed.position)
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let surface = sc::popover(theme);
        if let Some(bg) = surface.background {
            renderer.fill_quad(
                Quad {
                    bounds: layout.bounds(),
                    border: surface.border,
                    shadow: surface.shadow,
                    snap: false,
                },
                bg,
            );
        }
        let hl = self.highlight.index().filter(|_| self.selectable());
        for (child, i) in layout.children().zip(self.window.clone()) {
            if hl == Some(i) {
                let st = sb::menu_item(false)(theme, button::Status::Hovered);
                if let Some(bg) = st.background {
                    renderer.fill_quad(
                        Quad {
                            bounds: child.bounds(),
                            border: st.border,
                            shadow: iced::Shadow::default(),
                            snap: false,
                        },
                        bg,
                    );
                }
            }
            self.items[i].as_widget().draw(
                &self.trees[i],
                renderer,
                theme,
                style,
                child,
                cursor,
                &Rectangle::with_size(Size::INFINITE),
            );
        }
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
    ) {
        let n = match self.body {
            Body::Rows(n) => n,
            _ => 0,
        };
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                ..
            }) => match key {
                Named::Escape => {
                    shell.capture_event();
                    return self.close(shell);
                }
                Named::ArrowDown if n > 0 => {
                    self.highlight.next(n);
                    shell.capture_event();
                    shell.invalidate_layout();
                    shell.request_redraw();
                    return;
                }
                Named::ArrowUp if n > 0 => {
                    self.highlight.prev(n);
                    shell.capture_event();
                    shell.invalidate_layout();
                    shell.request_redraw();
                    return;
                }
                Named::Enter if n > 0 => {
                    if let Some(i) = self.highlight.index().filter(|i| *i < n) {
                        return self.pick(i, shell);
                    }
                }
                _ => {}
            },
            Event::Mouse(mouse::Event::CursorMoved { .. }) if self.selectable() => {
                if let Some(i) = self.row_at(layout, cursor)
                    && self.highlight.index() != Some(i)
                {
                    self.highlight.set(Some(i));
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if !cursor.is_over(layout.bounds()) {
                    // Outside: close, and let the click through (the entry reopens on itself).
                    if !cursor.is_over(self.anchor) {
                        self.close(shell);
                    }
                    return;
                }
                if self.selectable() {
                    if let Some(i) = self.row_at(layout, cursor) {
                        return self.pick(i, shell);
                    }
                    shell.capture_event();
                    return;
                }
            }
            _ => {}
        }
        // The notice (its Retry button) takes the rest.
        if !self.selectable() {
            for (child, i) in layout.children().zip(self.window.clone()) {
                self.items[i].as_widget_mut().update(
                    &mut self.trees[i],
                    event,
                    child,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    &Rectangle::with_size(Size::INFINITE),
                );
            }
            if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_)))
                && cursor.is_over(layout.bounds())
            {
                shell.capture_event();
            }
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if self.selectable() {
            return if self.row_at(layout, cursor).is_some() {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::None
            };
        }
        layout
            .children()
            .zip(self.window.clone())
            .map(|(child, i)| {
                self.items[i].as_widget().mouse_interaction(
                    &self.trees[i],
                    child,
                    cursor,
                    &Rectangle::with_size(Size::INFINITE),
                    renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }
}
