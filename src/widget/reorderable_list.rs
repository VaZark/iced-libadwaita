//! `ReorderableList`: a list that you declare once with its two views, **preview** and
//! **edit**, and that owns the switching between them; editing means dragging the rows into a
//! new order (libadwaita's draggable rows: a grip, `list-drag-handle-symbolic`, at the end of
//! each). Declared the way Chakra's `Editable` is (a preview, an edit view and their controls
//! in one container), but controlled, like every component here: the order and the mode live in
//! a [`ReorderState`], and the list speaks **one** message, [`ReorderEvent`].
//!
//! ```ignore
//! // view
//! ReorderableList::of(&self.order, |item| action_row(*item).into())
//!     .title("Sidebar")
//!     .on_event(Msg::List)
//!
//! // update
//! Msg::List(event) => { self.order.apply(event); }
//! ```
//!
//! That is the whole wiring: `of` takes the items and the mode from the state, the Edit /
//! Cancel / Done buttons and the drag all come out as `on_event`, and `apply` handles every
//! one (the draft, submit, cancel). The state can sit anywhere in the model.
//!
//! * **Views.** One row builder serves both; [`edit_row`](ReorderableList::edit_row) gives the
//!   edit view its own (a compact row, say). The edit view adds the grips. Only the view being
//!   shown is built.
//! * **Controls.** [`controls(false)`](ReorderableList::controls) drops the bar when the app
//!   has its own Edit button (send it `ReorderEvent::Edit`); [`disabled`](ReorderableList::disabled)
//!   dims Edit; [`labels`](ReorderableList::labels) renames the buttons.
//! * **Switching.** The views are picked like the variants of a
//!   [`MetaComponent`](super::MetaComponent) keyed by [`Mode`] (`Trigger::selected(Mode::Edit)`,
//!   the same `primitives::meta::pick`, called directly because `responsive` cannot shrink to
//!   its content's height; [`Form`](super::Form) does the same). A
//!   [`MetaParentComponent`](super::MetaParentComponent) can switch a list as well: register
//!   it with [`actions`](ReorderableList::actions) and call [`editing`](ReorderableList::editing)
//!   with the mode it is given.
//!
//! **Dragging.** Press a row's grip and move: the row lifts and follows the pointer, the others
//! move out of its way, and releasing drops it in the slot it is over (`Reorder { from, to }`).
//! Escape puts it back. Rows keep their own pointer behavior (buttons, entries) in edit mode;
//! only the grip starts a drag. Moving rows with the keyboard is not included: iced has no focus
//! model to hang it on yet.

use super::{Element, Mode, card_shadow, flat_button, raised_button, suggested_button};
use crate::color::alpha;
use crate::primitives::animation::{Progress, Spec, Trigger as Motion};
use crate::primitives::meta::{self, Trigger};
use crate::primitives::reorder::{self, Event as ReorderEvent, ReorderState, Span};
use crate::style::container as sc;
use crate::widget::typography::label;
use crate::{Icon, Metrics, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::alignment::Vertical;
use iced::widget::{column, container, row, space};
use iced::{
    Background, Border, Color, Event, Length, Point, Rectangle, Shadow, Size, Transformation,
    Vector, keyboard, mouse,
};
use std::rc::Rc;

/// Width of the grip's hit area, at the end of each row in edit mode.
const HANDLE_WIDTH: f32 = 48.0;
/// The hairline between rows.
const SEPARATOR: f32 = 1.0;
/// The held row's default hover: 2% bigger.
const DEFAULT_LIFT: f32 = 1.02;

/// A drop callback that may have nothing to say.
type Change<'a, M> = Box<dyn Fn(usize, usize) -> Option<M> + 'a>;

type Build<'a, T, M> = Box<dyn Fn(&T) -> Element<'a, M> + 'a>;

/// See the [module](self).
pub struct ReorderableList<'a, T, M> {
    items: &'a [T],
    row: Build<'a, T, M>,
    edit_row: Option<Build<'a, T, M>>,
    row_action: Option<Build<'a, T, M>>,
    title: Option<String>,
    editing: bool,
    disabled: bool,
    controls: bool,
    on_event: Option<Rc<dyn Fn(ReorderEvent) -> M + 'a>>,
    on_change: Option<Change<'a, M>>,
    lift: f32,
    edit_label: String,
    submit_label: String,
    cancel_label: String,
    metrics: Metrics,
}

impl<'a, T, M: Clone + 'a> ReorderableList<'a, T, M> {
    /// `items` in their current order and the closure that builds one row (an
    /// [`ActionRow`](super::ActionRow), for instance, without suffix: the grip is added).
    pub fn new(items: &'a [T], row: impl Fn(&T) -> Element<'a, M> + 'a) -> Self {
        Self {
            items,
            row: Box::new(row),
            title: None,
            editing: false,
            disabled: false,
            controls: true,
            edit_row: None,
            row_action: None,
            on_event: None,
            on_change: None,
            lift: DEFAULT_LIFT,
            edit_label: "Edit".into(),
            submit_label: "Done".into(),
            cancel_label: "Cancel".into(),
            metrics: Metrics::desktop(),
        }
    }
    /// A list over a [`ReorderState`]: its items, and edit mode while it has a draft. The usual
    /// way to build one.
    pub fn of(state: &'a ReorderState<T>, row: impl Fn(&T) -> Element<'a, M> + 'a) -> Self
    where
        T: Clone,
    {
        // A live list is always editing and has no Edit / Cancel / Done to show.
        Self::new(state.items(), row)
            .editing(state.is_editing())
            .controls(!state.is_live())
    }
    /// The row of the edit view, when it should differ from the preview's.
    pub fn edit_row(mut self, row: impl Fn(&T) -> Element<'a, M> + 'a) -> Self {
        self.edit_row = Some(Box::new(row));
        self
    }
    /// A control at the end of every row of the edit view, **before the grip**: a delete button,
    /// a menu. It is built from the item like the row, and keeps its own pointer behavior (only
    /// the grip starts a drag). The list's [`ReorderState::remove`] is what a delete button wants.
    pub fn row_action(mut self, action: impl Fn(&T) -> Element<'a, M> + 'a) -> Self {
        self.row_action = Some(Box::new(action));
        self
    }
    /// A heading at the start of the controls bar.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    /// Whether the edit view shows (controlled; Chakra's `isEditing`).
    pub fn editing(mut self, editing: bool) -> Self {
        self.editing = editing;
        self
    }
    /// A disabled list cannot be put into edit mode (the Edit button is dimmed).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Whether the bar with the Edit / Cancel / Done buttons shows (default true). Without it
    /// the app drives `editing` from its own buttons.
    pub fn controls(mut self, show: bool) -> Self {
        self.controls = show;
        self
    }
    /// The list's one message: Edit, Submit and Cancel from the buttons, and `Reorder` from a
    /// drop. Feed it to [`ReorderState::apply`].
    pub fn on_event(mut self, f: impl Fn(ReorderEvent) -> M + 'a) -> Self {
        self.on_event = Some(Rc::new(f));
        self
    }
    /// A second, independent callback for the same drop, for whoever watches the list from
    /// outside: a [`MetaParentComponent`](super::MetaParentComponent) hands one out
    /// ([`Reporter::on_change`](super::Reporter::on_change)) so it can react to its child's order changing while the
    /// app's own `on_event` still edits the order. Both messages are published, `on_event`
    /// first.
    pub fn on_change(mut self, f: impl Fn(usize, usize) -> Option<M> + 'a) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }
    /// How much bigger the held row grows while it is dragged, as a scale (default 1.02, a
    /// tiny hover; 1.0 turns it off). The row is always opaque and drawn above the others.
    pub fn lift(mut self, scale: f32) -> Self {
        self.lift = scale.max(1.0);
        self
    }
    /// The labels of the three buttons (defaults: Edit, Done, Cancel).
    pub fn labels(
        mut self,
        edit: impl Into<String>,
        submit: impl Into<String>,
        cancel: impl Into<String>,
    ) -> Self {
        self.edit_label = edit.into();
        self.submit_label = submit.into();
        self.cancel_label = cancel.into();
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    /// The actions this list announces when it registers with a
    /// [`MetaParentComponent`](super::MetaParentComponent): the two modes it can be switched to.
    pub fn actions() -> [Mode; 2] {
        [Mode::Edit, Mode::Preview]
    }

    /// The view a list shows: the first variant whose trigger fires, `Preview` being the
    /// fallback.
    pub fn pick(editing: bool) -> Mode {
        let variants = [
            (Mode::Edit, Trigger::selected(Mode::Edit)),
            (Mode::Preview, Trigger::always()),
        ];
        let selected = [Mode::from_editing(editing)];
        meta::pick(
            variants.iter().map(|(_, t)| t),
            &selected,
            0.0,
            &crate::breakpoints::Breakpoints::default(),
        )
        .map_or(Mode::Preview, |i| variants[i].0)
    }

    fn send(&self, e: ReorderEvent) -> Option<M> {
        self.on_event.as_ref().map(|f| f(e))
    }

    fn bar(&self, mode: Mode) -> Element<'a, M> {
        let m = &self.metrics;
        let mut bar = row![].spacing(m.space_s).align_y(Vertical::Center);
        if let Some(t) = &self.title {
            bar = bar.push(crate::widget::typography::heading(t.clone()));
        }
        bar = bar.push(space().width(Length::Fill));
        match mode {
            Mode::Preview => {
                let edit = flat_button(label(self.edit_label.clone())).on_press_maybe(
                    (!self.disabled)
                        .then(|| self.send(ReorderEvent::Edit))
                        .flatten(),
                );
                bar = bar.push(edit);
            }
            Mode::Edit => {
                bar = bar
                    .push(
                        raised_button(label(self.cancel_label.clone()))
                            .on_press_maybe(self.send(ReorderEvent::Cancel)),
                    )
                    .push(
                        suggested_button(label(self.submit_label.clone()))
                            .on_press_maybe(self.send(ReorderEvent::Submit)),
                    );
            }
        }
        bar.into()
    }

    fn preview(&self) -> Element<'a, M> {
        super::boxed_list(self.items.iter().map(|i| (self.row)(i)))
    }

    fn edit(&mut self) -> Element<'a, M> {
        let rows = self.items.iter().map(|item| {
            let grip = container(Icon::DragHandle.view(self.metrics.icon_size))
                .width(Length::Fixed(HANDLE_WIDTH))
                .align_x(iced::alignment::Horizontal::Center);
            let mut line = row![
                container((self.edit_row.as_ref().unwrap_or(&self.row))(item)).width(Length::Fill)
            ]
            .align_y(Vertical::Center);
            if let Some(action) = &self.row_action {
                line = line.push(action(item));
            }
            Element::from(line.push(grip))
        });
        let mut list = DragList::new(rows.collect());
        // Cloned, not taken: the Cancel and Done buttons are built from the same callback.
        list.on_reorder = self
            .on_event
            .clone()
            .map(|f| -> Box<dyn Fn(usize, usize) -> M + 'a> {
                Box::new(move |from, to| f(ReorderEvent::Reorder { from, to }))
            });
        list.on_change = self.on_change.take();
        list.lift = self.lift;
        let list: Element<'a, M> = Element::new(list);
        card_shadow(
            container(list)
                .width(Length::Fill)
                .clip(true)
                .style(sc::card_surface),
        )
        .into()
    }
}

impl<'a, T, M: Clone + 'a> From<ReorderableList<'a, T, M>> for Element<'a, M> {
    fn from(mut l: ReorderableList<'a, T, M>) -> Self {
        let mode = ReorderableList::<T, M>::pick(l.editing);
        let body = if l.items.is_empty() {
            space().into()
        } else if mode == Mode::Edit {
            l.edit()
        } else {
            l.preview()
        };
        let mut col = column![].spacing(l.metrics.space_m).width(Length::Fill);
        if l.controls || l.title.is_some() {
            col = col.push(l.bar(mode));
        }
        col.push(body).into()
    }
}

// ---------------------------------------------------------------------------
// DragList: the rows of the edit view
// ---------------------------------------------------------------------------

/// Rows stacked with hairlines between them, each draggable by the grip at its end.
struct DragList<'a, M, R = iced::Renderer> {
    rows: Vec<iced::Element<'a, M, Theme, R>>,
    on_reorder: Option<Box<dyn Fn(usize, usize) -> M + 'a>>,
    on_change: Option<Change<'a, M>>,
    lift: f32,
}

impl<'a, M, R: iced::advanced::Renderer> DragList<'a, M, R> {
    fn new(rows: Vec<iced::Element<'a, M, Theme, R>>) -> Self {
        Self {
            rows,
            on_reorder: None,
            on_change: None,
            lift: DEFAULT_LIFT,
        }
    }
    fn draggable(&self) -> bool {
        self.on_reorder.is_some() || self.on_change.is_some()
    }
}

/// A drag in progress.
#[derive(Debug, Clone, Copy)]
struct Drag {
    /// The row being dragged.
    from: usize,
    /// Where in the row the pointer took hold, from the row's top.
    grab: f32,
    /// The pointer's y.
    y: f32,
}

#[derive(Default)]
struct State {
    drag: Option<Drag>,
    /// How far each row has slid out of the held row's way (px), eased toward `shift`.
    slide: Vec<Progress>,
}

fn spans(layout: Layout<'_>) -> Vec<Span> {
    layout
        .children()
        .map(|c| Span::new(c.bounds().y, c.bounds().height))
        .collect()
}

/// Where the dragged row's top sits, kept inside the list.
fn dragged_top(d: &Drag, rows: &[Span], height: f32) -> f32 {
    let (first, last) = (rows[0].top, rows[rows.len() - 1]);
    (d.y - d.grab).clamp(first, (last.top + last.height - height).max(first))
}

fn drop_slot(d: &Drag, rows: &[Span]) -> usize {
    let h = rows[d.from].height;
    // The pointer decides, not the clamped drawing, so the ends are always reachable.
    reorder::target(rows, d.from, d.y - d.grab + h / 2.0)
}

fn handle_rect(row: Rectangle) -> Rectangle {
    Rectangle {
        x: row.x + (row.width - HANDLE_WIDTH).max(0.0),
        width: HANDLE_WIDTH.min(row.width),
        ..row
    }
}

impl<'a, M: 'a, R: iced::advanced::Renderer + 'a> Widget<M, Theme, R> for DragList<'a, M, R> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        self.rows.iter().map(Tree::new).collect()
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.rows);
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &R, limits: &layout::Limits) -> layout::Node {
        let width = limits.max().width;
        let child_limits = layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY));
        let mut y = 0.0;
        let mut nodes = Vec::with_capacity(self.rows.len());
        for (i, (row, t)) in self.rows.iter_mut().zip(&mut tree.children).enumerate() {
            if i > 0 {
                y += SEPARATOR;
            }
            let node = row
                .as_widget_mut()
                .layout(t, renderer, &child_limits)
                .move_to(Point::new(0.0, y));
            y += node.size().height;
            nodes.push(node);
        }
        layout::Node::with_children(Size::new(width, y), nodes)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &R,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        if let Some(mut d) = state.drag {
            match event {
                // The rows slide toward where the held row would land.
                Event::Window(iced::window::Event::RedrawRequested(now)) => {
                    let rows = spans(layout);
                    let to = drop_slot(&d, &rows);
                    let step = rows[d.from].height + SEPARATOR;
                    let spec = Motion::Selection.spec(Spec::STYLE);
                    state.slide.resize_with(rows.len(), || Progress::new(0.0));
                    let mut moving = false;
                    for (j, p) in state.slide.iter_mut().enumerate() {
                        moving |= p.drive(reorder::shift(j, d.from, to, step), *now, spec);
                    }
                    if moving {
                        shell.request_redraw();
                    }
                    return;
                }
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    // The pointer from `cursor`, not from the event: a scrollable parent shifts
                    // it by its scroll offset (see `Resizable`).
                    if let Some(p) = cursor.position() {
                        d.y = p.y;
                        state.drag = Some(d);
                    }
                    shell.request_redraw();
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    if let Some(p) = cursor.position() {
                        d.y = p.y;
                    }
                    let rows = spans(layout);
                    let (from, to) = (d.from, drop_slot(&d, &rows));
                    state.drag = None;
                    state.slide.clear();
                    if from != to {
                        if let Some(f) = &self.on_reorder {
                            shell.publish(f(from, to));
                        }
                        if let Some(m) = self.on_change.as_ref().and_then(|f| f(from, to)) {
                            shell.publish(m);
                        }
                    }
                    shell.request_redraw();
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                }) => {
                    state.drag = None;
                    state.slide.clear();
                    shell.request_redraw();
                }
                _ => {}
            }
            // While a row is held, nothing under it sees the pointer.
            shell.capture_event();
            return;
        }

        if self.draggable()
            && let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && let Some(p) = cursor.position()
            && let Some((i, row)) = layout
                .children()
                .enumerate()
                .find(|(_, c)| handle_rect(c.bounds()).contains(p))
        {
            state.drag = Some(Drag {
                from: i,
                grab: p.y - row.bounds().y,
                y: p.y,
            });
            shell.capture_event();
            shell.request_redraw();
            return;
        }

        for ((row, t), l) in self
            .rows
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            row.as_widget_mut()
                .update(t, event, l, cursor, renderer, clipboard, shell, viewport);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &R,
    ) -> mouse::Interaction {
        if tree.state.downcast_ref::<State>().drag.is_some() {
            return mouse::Interaction::Grabbing;
        }
        if self.draggable()
            && layout
                .children()
                .any(|c| cursor.is_over(handle_rect(c.bounds())))
        {
            return mouse::Interaction::Grab;
        }
        self.rows
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((row, t), l)| {
                row.as_widget()
                    .mouse_interaction(t, l, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut R,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let drag = tree.state.downcast_ref::<State>().drag;
        let rows = spans(layout);
        if rows.is_empty() {
            return;
        }

        // The hairlines stay where the slots are.
        for r in rows.iter().skip(1) {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(
                        Point::new(layout.bounds().x, r.top - SEPARATOR),
                        Size::new(layout.bounds().width, SEPARATOR),
                    ),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: true,
                },
                theme.semantic.card_shade,
            );
        }

        let slot = drag.as_ref().map(|d| (d, drop_slot(d, &rows)));
        let accent = theme.semantic.accent.bg;
        let slide = &tree.state.downcast_ref::<State>().slide;

        // The target: the slot the held row would land in, tinted in the accent color, with an
        // accent line on the edge it is moving toward (Spotify's insertion line, GNOME's
        // `drop-active` row). Where the row started it is the same tinted slot, without a line.
        if let Some((d, to)) = slot {
            let h = rows[d.from].height;
            let top = reorder::slot_top(&rows, d.from, to, SEPARATOR);
            let b = layout.bounds();
            let fill = |renderer: &mut R, y: f32, height: f32, color: Color| {
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(Point::new(b.x, y), Size::new(b.width, height)),
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    color,
                );
            };
            fill(renderer, top, h, alpha(accent, 0.14));
            const LINE: f32 = 2.0;
            if to > d.from {
                fill(renderer, top + h - LINE, LINE, accent);
            } else if to < d.from {
                fill(renderer, top, LINE, accent);
            }
        }
        for (j, ((row, t), l)) in self
            .rows
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .enumerate()
        {
            match slot {
                // The held row is drawn last, on top.
                Some((d, _)) if d.from == j => {}
                Some((_, _)) => {
                    let dy = slide.get(j).map_or(0.0, |p| p.shown);
                    // Clip to what the viewport shows *after* the shift: the row moves into a
                    // neighbor's slot, so clipping to its own slot would cut it off.
                    let vp = Rectangle {
                        y: viewport.y - dy,
                        ..*viewport
                    };
                    renderer.with_translation(Vector::new(0.0, dy), |r| {
                        row.as_widget().draw(
                            t,
                            r,
                            theme,
                            style,
                            l,
                            mouse::Cursor::Unavailable,
                            &vp,
                        );
                    });
                }
                None => row
                    .as_widget()
                    .draw(t, renderer, theme, style, l, cursor, viewport),
            }
        }

        if let Some((d, _)) = slot
            && let (Some(row), Some(t), Some(l)) = (
                self.rows.get(d.from),
                tree.children.get(d.from),
                layout.children().nth(d.from),
            )
        {
            let b = l.bounds();
            let dy = dragged_top(d, &rows, b.height) - b.y;
            let lift = theme.shadows.popover();
            // Scaled about its own center: the tiny hover.
            let (cx, cy) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
            let grow = Transformation::translate(cx, cy)
                * Transformation::scale(self.lift)
                * Transformation::translate(-cx, -cy);
            // The card color is translucent in libadwaita (the rows below would show through),
            // so the held row gets the opaque view color under it.
            let solid = |c: Color| Color { a: 1.0, ..c };
            renderer.with_layer(layout.bounds().expand(24.0), |renderer| {
                renderer.with_translation(Vector::new(0.0, dy), |renderer| {
                    renderer.with_transformation(grow, |renderer| {
                        renderer.fill_quad(
                            Quad {
                                bounds: b,
                                // The held row: lifted, and ringed in the accent color.
                                border: Border {
                                    color: accent,
                                    width: 2.0,
                                    radius: theme.metrics.radius_s.into(),
                                },
                                shadow: Shadow {
                                    color: lift.shadow.color,
                                    offset: Vector::new(0.0, 2.0),
                                    blur_radius: lift.shadow.blur_radius,
                                },
                                snap: false,
                            },
                            Background::Color(solid(theme.semantic.view.bg)),
                        );
                        renderer.fill_quad(
                            Quad {
                                bounds: b,
                                border: Border::default().rounded(theme.metrics.radius_s),
                                shadow: Shadow::default(),
                                snap: false,
                            },
                            Background::Color(theme.semantic.card.bg),
                        );
                        let vp = Rectangle {
                            y: viewport.y - dy,
                            ..*viewport
                        };
                        row.as_widget().draw(
                            t,
                            renderer,
                            theme,
                            style,
                            l,
                            mouse::Cursor::Unavailable,
                            &vp,
                        );
                    });
                });
            });
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &R,
        operation: &mut dyn Operation,
    ) {
        for ((row, t), l) in self
            .rows
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            row.as_widget_mut().operate(t, l, renderer, operation);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &R,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, R>> {
        let children: Vec<_> = self
            .rows
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((row, t), l)| {
                row.as_widget_mut()
                    .overlay(t, l, renderer, viewport, translation)
            })
            .collect();
        (!children.is_empty()).then(|| overlay::Group::with_children(children).overlay())
    }
}

impl<'a, M: 'a, R: iced::advanced::Renderer + 'a> From<DragList<'a, M, R>>
    for iced::Element<'a, M, Theme, R>
{
    fn from(l: DragList<'a, M, R>) -> Self {
        iced::Element::new(l)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::clipboard;
    use iced::widget::space;

    type E = iced::Element<'static, (usize, usize), Theme, ()>;

    /// Three rows 40 tall (tops 0, 41, 82) in a list 300 wide.
    fn list() -> E {
        let rows: Vec<E> = (0..3)
            .map(|_| space().width(Length::Fill).height(40).into())
            .collect();
        let mut l = DragList::<_, ()>::new(rows);
        l.on_reorder = Some(Box::new(|a, b| (a, b)));
        l.into()
    }

    /// Presses at `press`, moves to `to` and releases there; returns what was published.
    fn drag(press: Point, to: Point) -> Vec<(usize, usize)> {
        let mut el = list();
        let mut tree = Tree::new(&el);
        let node = el.as_widget_mut().layout(
            &mut tree,
            &(),
            &layout::Limits::new(Size::ZERO, Size::new(300.0, 600.0)),
        );
        let layout = Layout::new(&node);
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        let view = Rectangle::new(Point::ORIGIN, Size::new(300.0, 600.0));
        let mut send = |shell: &mut Shell<'_, (usize, usize)>, e: Event, at: Point| {
            el.as_widget_mut().update(
                &mut tree,
                &e,
                layout,
                mouse::Cursor::Available(at),
                &(),
                &mut clipboard::Null,
                shell,
                &view,
            );
        };
        send(
            &mut shell,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            press,
        );
        send(
            &mut shell,
            Event::Mouse(mouse::Event::CursorMoved { position: to }),
            to,
        );
        send(
            &mut shell,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            to,
        );
        drop(shell);
        messages
    }

    const GRIP_X: f32 = 290.0;

    #[test]
    fn rows_stack_with_a_hairline_between_them() {
        let mut el = list();
        let mut tree = Tree::new(&el);
        let node = el.as_widget_mut().layout(
            &mut tree,
            &(),
            &layout::Limits::new(Size::ZERO, Size::new(300.0, 600.0)),
        );
        assert_eq!(node.size(), Size::new(300.0, 122.0));
        let tops: Vec<f32> = Layout::new(&node)
            .children()
            .map(|c| c.bounds().y)
            .collect();
        assert_eq!(tops, vec![0.0, 41.0, 82.0]);
    }

    #[test]
    fn dragging_a_grip_down_past_the_next_row_reorders() {
        // Center at 90: past the middle of row 1 (61), not of row 2 (102).
        assert_eq!(
            drag(Point::new(GRIP_X, 20.0), Point::new(GRIP_X, 90.0)),
            vec![(0, 1)]
        );
        assert_eq!(
            drag(Point::new(GRIP_X, 20.0), Point::new(GRIP_X, 120.0)),
            vec![(0, 2)]
        );
    }

    #[test]
    fn dragging_up_reorders_the_other_way() {
        assert_eq!(
            drag(Point::new(GRIP_X, 100.0), Point::new(GRIP_X, 10.0)),
            vec![(2, 0)]
        );
    }

    #[test]
    fn dropping_in_place_publishes_nothing() {
        assert!(drag(Point::new(GRIP_X, 60.0), Point::new(GRIP_X, 66.0)).is_empty());
    }

    #[test]
    fn only_the_grip_starts_a_drag() {
        assert!(drag(Point::new(100.0, 20.0), Point::new(100.0, 100.0)).is_empty());
    }

    #[test]
    fn the_held_row_stays_inside_the_list() {
        assert_eq!(
            drag(Point::new(GRIP_X, 20.0), Point::new(GRIP_X, 5000.0)),
            vec![(0, 2)]
        );
        assert_eq!(
            drag(Point::new(GRIP_X, 100.0), Point::new(GRIP_X, -5000.0)),
            vec![(2, 0)]
        );
    }

    #[test]
    fn the_rows_slide_out_of_the_way_and_settle_and_clear_on_drop() {
        use std::time::{Duration, Instant};
        let mut el = list();
        let mut tree = Tree::new(&el);
        let node = el.as_widget_mut().layout(
            &mut tree,
            &(),
            &layout::Limits::new(Size::ZERO, Size::new(300.0, 600.0)),
        );
        let layout = Layout::new(&node);
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        let view = Rectangle::new(Point::ORIGIN, Size::new(300.0, 600.0));
        let mut send = |tree: &mut Tree, e: Event, at: Point| {
            el.as_widget_mut().update(
                tree,
                &e,
                layout,
                mouse::Cursor::Available(at),
                &(),
                &mut clipboard::Null,
                &mut shell,
                &view,
            );
        };
        let at = Point::new(GRIP_X, 120.0);
        send(
            &mut tree,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Point::new(GRIP_X, 20.0),
        );
        send(
            &mut tree,
            Event::Mouse(mouse::Event::CursorMoved { position: at }),
            at,
        );
        let t0 = Instant::now();
        let redraw = |t| Event::Window(iced::window::Event::RedrawRequested(t));
        send(&mut tree, redraw(t0), at);
        send(&mut tree, redraw(t0 + Duration::from_secs(2)), at);
        let slid: Vec<f32> = tree
            .state
            .downcast_ref::<State>()
            .slide
            .iter()
            .map(|p| p.shown)
            .collect();
        // Row 0 is held; rows 1 and 2 each moved up by the held row's height and a hairline.
        assert_eq!(slid, vec![0.0, -41.0, -41.0]);
        send(
            &mut tree,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            at,
        );
        assert!(tree.state.downcast_ref::<State>().slide.is_empty());
    }

    #[test]
    fn building_the_edit_view_leaves_the_buttons_their_callback() {
        let items = [1, 2, 3];
        let mut l = ReorderableList::<i32, u8>::new(&items, |_| iced::widget::space().into())
            .on_event(|_| 7);
        let _ = l.edit();
        assert_eq!(l.send(ReorderEvent::Submit), Some(7));
        assert_eq!(l.send(ReorderEvent::Cancel), Some(7));
    }

    #[test]
    fn the_view_follows_the_editing_flag() {
        type L = ReorderableList<'static, u8, ()>;
        assert_eq!(L::pick(false), Mode::Preview);
        assert_eq!(L::pick(true), Mode::Edit);
    }
}
