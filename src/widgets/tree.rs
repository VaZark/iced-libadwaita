//! `Tree`: hierarchical data as indented rows with an arrow to open each branch, after Mantine's
//! [Tree](https://mantine.dev/core/tree/), adapted to GNOME. The behavior is the
//! [`tree`](crate::primitives::tree) primitive; this draws it and speaks **one** message,
//! [`Event`], which the app applies with [`TreeState::apply`]:
//!
//! ```ignore
//! // view
//! Tree::new(&self.data, &self.tree)
//!     .lines(true)
//!     .checkboxes(true)
//!     .highlight(&self.search)
//!     .on_event(Msg::Tree)
//!
//! // update
//! Msg::Tree(event) => if let Some(Effect::Load(v)) = self.tree.apply(&mut self.data, event) { /* fetch v */ }
//! ```
//!
//! What Mantine's page shows is here: connecting [`lines`](Tree::lines), a [`level_offset`](Tree::level_offset),
//! [`checkboxes`](Tree::checkboxes) (with the indeterminate state of a half-checked branch),
//! [`icons`](Tree::icons) and a [`label`](Tree::label) renderer (`renderNode`), [`highlight`](Tree::highlight)ed
//! matches, lazily loaded branches (a spinner while loading, the error and Retry when it
//! failed), drag and drop ([`draggable`](Tree::draggable): by the row or by a grip, with
//! [`allow_drop`](Tree::allow_drop) for the nodes that must not take a drop), and
//! [`window`](Tree::window) for very large trees. Search and filtering stay outside, as in
//! Mantine: filter the data with [`tree::filter`](crate::primitives::tree::filter) and open the
//! result with [`TreeState::expand_all`].
//!
//! **Adapted to GNOME.** GNOME has no tree-view pattern in the HIG; the model is GTK's
//! `GtkTreeExpander`: rows indent by depth, a leaf keeps the width of the arrow so its label
//! lines up with its siblings' (`indent-for-icon`), the arrow points right when closed and down
//! when open, and the keys are `+` `*` / `-` `/` / Ctrl+Space and the arrows (the primitive's
//! `navigate`; not wired here, as iced has no focus model for it). A selected row is the
//! neutral flat-button highlight, as in libadwaita's sidebars; a drop shows as the accent line
//! and tint of the reorderable list. For small static hierarchies the HIG's own answer is an
//! [`ExpanderRow`](crate::widgets::ExpanderRow) in a boxed list.

use crate::Element;
use crate::components::flat_button;
use crate::components::typography::label as plain_label;
use crate::primitives::tree::{self, Event, Position, Row, TreeNode, TreeState};
use crate::styles::button as sb;
use crate::styles::text as st;
use crate::{Icon, Metrics, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree as WidgetTree, Widget, tree as wtree};
use iced::advanced::{Clipboard, Renderer as _, Shell, overlay};
use iced::alignment::Vertical;
use iced::widget::{button, container, rich_text, row, space, span};
use iced::{
    Background, Border, Color, Event as IcedEvent, Length, Point, Rectangle, Shadow, Size,
    Transformation, Vector, keyboard, mouse,
};
use std::ops::Range;
use std::rc::Rc;

/// Legacy row-height reference; widget defaults are resolved from `[tree] row-height`.
pub const ROW_HEIGHT: f32 = 32.0;

/// Resolved once so row layout, virtualization, painting and hit testing agree.
#[derive(Debug, Clone, Copy)]
struct TreeLayout {
    level_offset: f32,
    row_height: f32,
    handle_padding: f32,
    expander_icon_inset: f32,
    grip_width: f32,
    drag_threshold: f32,
    checkbox_size_extra: f32,
    checkbox_glyph_inset: f32,
    checkbox_radius: f32,
    checkbox_border_width: f32,
    connector_width: f32,
    drop_border_width: f32,
    drop_line_width: f32,
    drop_line_radius: f32,
    drag_border_width: f32,
    drag_scale: f32,
    drag_clip_margin: f32,
    drag_shadow_offset: f32,
    icon_size: f32,
    icon_padding: iced::Padding,
    label_padding: iced::Padding,
    checkbox_padding: iced::Padding,
    row_radius: f32,
}

impl TreeLayout {
    /// Design defaults are supplied by the schema, not duplicated in the widget.
    fn resolve(m: &Metrics, tokens: &crate::theme_config::Tokens, variant: &str) -> Self {
        Self::resolve_numbers(m, |key| tokens.variant_number("tree", variant, key))
    }

    fn resolve_numbers(m: &Metrics, number: impl Fn(&str) -> Option<f32>) -> Self {
        let required =
            |key: &str| number(key).unwrap_or_else(|| panic!("missing [tree] {key} token"));
        Self {
            level_offset: required("level-offset"),
            row_height: required("row-height"),
            handle_padding: required("handle-padding"),
            expander_icon_inset: required("expander-icon-inset"),
            grip_width: required("grip-width"),
            drag_threshold: required("drag-threshold"),
            checkbox_size_extra: required("checkbox-size-extra"),
            checkbox_glyph_inset: required("checkbox-glyph-inset"),
            checkbox_radius: required("checkbox-radius"),
            checkbox_border_width: required("checkbox-border-width"),
            connector_width: required("connector-width"),
            drop_border_width: required("drop-border-width"),
            drop_line_width: required("drop-line-width"),
            drop_line_radius: required("drop-line-radius"),
            drag_border_width: required("drag-border-width"),
            drag_scale: required("drag-scale"),
            drag_clip_margin: required("drag-clip-margin"),
            drag_shadow_offset: required("drag-shadow-offset"),
            icon_size: number("icon-size").unwrap_or(m.icon_size),
            icon_padding: iced::Padding::from([0.0, number("icon-padding").unwrap_or(m.space_xs)]),
            label_padding: iced::Padding::from([0.0, number("label-padding").unwrap_or(m.space_s)]),
            checkbox_padding: iced::Padding::from([
                0.0,
                number("checkbox-padding").unwrap_or(m.space_xs),
            ]),
            row_radius: number("row-radius").unwrap_or(m.radius_s),
        }
    }
}

/// How a row is picked up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DragMode {
    /// Nothing can be dragged.
    #[default]
    Off,
    /// Hold a row and move: a press that does not travel is still a click.
    Row,
    /// Only the grip at the end of the row picks it up (Mantine's `withDragHandle`).
    Handle,
}

/// What a custom [`label`](Tree::label) renderer is given (Mantine's `renderNode` payload).
pub struct Payload<'a, M> {
    pub node: &'a TreeNode,
    pub row: Row,
    pub selected: bool,
    pub checked: bool,
    pub indeterminate: bool,
    /// The default label: the text with its matches highlighted.
    pub text: Element<'a, M>,
}

type IconFn<'a> = Box<dyn Fn(&TreeNode, &Row) -> Option<Icon> + 'a>;
type LabelFn<'a, M> = Box<dyn Fn(Payload<'a, M>) -> Element<'a, M> + 'a>;
type AllowFn<'a> = Rc<dyn Fn(&TreeNode, &TreeNode, Position) -> bool + 'a>;
type Emit<'a, M> = Rc<dyn Fn(Event) -> M + 'a>;

/// See the [module](self).
pub struct Tree<'a, M> {
    data: &'a [TreeNode],
    state: &'a TreeState,
    lines: bool,
    level_offset: Option<f32>,
    row_height: Option<f32>,
    checkboxes: bool,
    highlight: String,
    fuzzy: bool,
    drag: DragMode,
    allow_drop: Option<AllowFn<'a>>,
    icons: Option<IconFn<'a>>,
    label: Option<LabelFn<'a, M>>,
    window: Option<Range<usize>>,
    on_event: Option<Emit<'a, M>>,
    metrics: Metrics,
    variant: String,
    explicit_variant: bool,
    tokens: Option<crate::theme_config::Tokens>,
    icon_padding: Option<iced::Padding>,
    label_padding: Option<iced::Padding>,
    checkbox_padding: Option<iced::Padding>,
}

impl<'a, M: Clone + 'a> Tree<'a, M> {
    pub fn new(data: &'a [TreeNode], state: &'a TreeState) -> Self {
        Self {
            data,
            state,
            lines: false,
            level_offset: None,
            row_height: None,
            checkboxes: false,
            highlight: String::new(),
            fuzzy: false,
            drag: DragMode::Off,
            allow_drop: None,
            icons: None,
            label: None,
            window: None,
            on_event: None,
            metrics: Metrics::desktop(),
            variant: "default".into(),
            explicit_variant: false,
            tokens: None,
            icon_padding: None,
            label_padding: None,
            checkbox_padding: None,
        }
    }
    /// Selects a `[tree.<name>]` variant (default: `"default"`).
    /// Unset keys inherit `[tree]`; undeclared names leave base geometry unchanged.
    /// Explicit geometry builders take priority over variant tokens.
    pub fn variant(mut self, name: impl Into<String>) -> Self {
        self.variant = name.into();
        self.explicit_variant = true;
        self
    }

    /// Uses a local token snapshot instead of the installed global theme config.
    /// Inherited metrics come from `.metrics(...)`, not from this snapshot.
    pub fn tokens(mut self, tokens: crate::theme_config::Tokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    /// Uses this config's tokens locally; pass its resolved metrics with `.metrics(...)`.
    pub fn theme_config(self, cfg: &crate::theme_config::ThemeConfig) -> Self {
        self.tokens(cfg.tokens())
    }

    /// Connecting lines down each level (Mantine's `withLines`).
    pub fn lines(mut self, on: bool) -> Self {
        self.lines = on;
        self
    }
    /// The indent per level, in pixels (default 20).
    pub fn level_offset(mut self, px: f32) -> Self {
        self.level_offset = Some(px.max(0.0));
        self
    }
    /// The height of every row (default 32). Fixed, so a tree can be windowed.
    pub fn row_height(mut self, px: f32) -> Self {
        self.row_height = Some(px.max(1.0));
        self
    }
    /// A checkbox on every row, with the indeterminate state for a half-checked branch.
    pub fn checkboxes(mut self, on: bool) -> Self {
        self.checkboxes = on;
        self
    }
    /// Marks where `query` occurs in the labels (case-insensitive).
    pub fn highlight(mut self, query: &str) -> Self {
        self.highlight = query.to_string();
        self
    }
    /// Highlight the characters of a fuzzy match rather than the substring.
    pub fn fuzzy(mut self, on: bool) -> Self {
        self.fuzzy = on;
        self
    }
    pub fn draggable(mut self, mode: DragMode) -> Self {
        self.drag = mode;
        self
    }
    /// Which drops are allowed, on top of the rule that a node cannot go into its own branch:
    /// `(dragged, target, position)` (Mantine's `allowDrop`).
    pub fn allow_drop(mut self, f: impl Fn(&TreeNode, &TreeNode, Position) -> bool + 'a) -> Self {
        self.allow_drop = Some(Rc::new(f));
        self
    }
    /// An icon before each label (a folder or a file, say).
    pub fn icons(mut self, f: impl Fn(&TreeNode, &Row) -> Option<Icon> + 'a) -> Self {
        self.icons = Some(Box::new(f));
        self
    }
    /// Draws each label yourself (Mantine's `renderNode`).
    pub fn label(mut self, f: impl Fn(Payload<'a, M>) -> Element<'a, M> + 'a) -> Self {
        self.label = Some(Box::new(f));
        self
    }
    /// Builds only these rows of the visible ones and spaces the rest, for a tree of thousands
    /// of rows in a scrollable. Pair it with [`virtualize::window`](crate::layouts::virtualize::window).
    pub fn window(mut self, rows: Range<usize>) -> Self {
        self.window = Some(rows);
        self
    }
    /// The one message: apply it with [`TreeState::apply`].
    pub fn on_event(mut self, f: impl Fn(Event) -> M + 'a) -> Self {
        self.on_event = Some(Rc::new(f));
        self
    }
    /// The padding around a row's icon (default: the theme's `space_xs` on each side).
    pub fn icon_padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.icon_padding = Some(p.into());
        self
    }
    /// The padding around a row's label (default: the theme's `space_s` on each side).
    pub fn label_padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.label_padding = Some(p.into());
        self
    }
    /// The padding around a row's checkbox (default: the theme's `space_xs` on each side).
    pub fn checkbox_padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.checkbox_padding = Some(p.into());
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    fn resolved_tokens(&self) -> crate::theme_config::Tokens {
        self.tokens
            .clone()
            .unwrap_or_else(|| crate::theme_config::current().tokens())
    }

    fn selected_variant(&self, tokens: &crate::theme_config::Tokens, width: f32) -> String {
        if self.explicit_variant {
            self.variant.clone()
        } else {
            crate::breakpoints::variant_for_tokens(
                tokens,
                "tree",
                &crate::breakpoints::Breakpoints::current(),
                width,
                &self.variant,
            )
        }
    }

    #[cfg(test)]
    fn geometry_with_tokens(&self, tokens: &crate::theme_config::Tokens) -> TreeLayout {
        self.geometry_for_variant(tokens, &self.variant)
    }

    fn geometry_for_variant(
        &self,
        tokens: &crate::theme_config::Tokens,
        variant: &str,
    ) -> TreeLayout {
        let mut g = TreeLayout::resolve(&self.metrics, tokens, variant);
        g.level_offset = self.level_offset.unwrap_or(g.level_offset);
        g.row_height = self.row_height.unwrap_or(g.row_height);
        g.icon_padding = self.icon_padding.unwrap_or(g.icon_padding);
        g.label_padding = self.label_padding.unwrap_or(g.label_padding);
        g.checkbox_padding = self.checkbox_padding.unwrap_or(g.checkbox_padding);
        g
    }

    fn emit(&self, e: Event) -> Option<M> {
        self.on_event.as_ref().map(|f| f(e))
    }

    /// The label's text, the matches marked.
    fn text(&self, r: &Row) -> Element<'a, M> {
        let ranges = if self.highlight.trim().is_empty() {
            Vec::new()
        } else if self.fuzzy {
            tree::fuzzy_ranges(&r.label, &self.highlight)
        } else {
            tree::match_ranges(&r.label, &self.highlight)
        };
        if ranges.is_empty() {
            return plain_label(r.label.clone()).into();
        }
        let mark = |t: &Theme| crate::color::alpha(t.semantic.accent.bg, 0.3);
        let mut spans: Vec<iced::widget::text::Span<'a, (), iced::Font>> = Vec::new();
        let mut at = 0;
        for rg in ranges {
            if rg.start > at {
                spans.push(span(r.label[at..rg.start].to_string()));
            }
            spans.push(
                span(r.label[rg.clone()].to_string())
                    .background(Background::Color(mark(&Theme::default())))
                    .padding(0),
            );
            at = rg.end;
        }
        if at < r.label.len() {
            spans.push(span(r.label[at..].to_string()));
        }
        rich_text(spans)
            .wrapping(iced::widget::text::Wrapping::None)
            .into()
    }

    /// One row's element.
    fn row(&self, r: &Row, g: TreeLayout) -> Element<'a, M> {
        let node = tree::find(self.data, &r.value);
        let selected = self.state.is_selected(&r.value);
        let mut line = row![].align_y(Vertical::Center);

        // Indent: one cell per level, each with its connecting line when asked.
        for _ in 0..r.level {
            let cell: Element<'a, M> = if self.lines {
                let hairline = container(space())
                    .width(g.connector_width)
                    .height(Length::Fill)
                    .style(|t: &Theme| iced::widget::container::Style {
                        background: Some(Background::Color(crate::color::alpha(
                            t.semantic.window.fg,
                            0.15,
                        ))),
                        ..Default::default()
                    });
                container(hairline)
                    .width(g.level_offset)
                    .height(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center)
                    .into()
            } else {
                space().width(g.level_offset).into()
            };
            line = line.push(cell);
        }

        // The arrow; a leaf keeps its width so labels line up (GTK's indent-for-icon).
        let arrow = g.icon_size + g.handle_padding;
        let expander: Element<'a, M> = if r.error.is_some() {
            flat_button(plain_label("Retry").style(st::error))
                .on_press_maybe(self.emit(Event::Retry(r.value.clone())))
                .into()
        } else if r.loading {
            container(plain_label("…").style(st::dim))
                .width(arrow)
                .center_x(arrow)
                .into()
        } else if r.has_children {
            let icon = if r.expanded {
                Icon::GoDown
            } else {
                Icon::GoNext
            };
            button(container(icon.view(g.icon_size - g.expander_icon_inset)).center(arrow))
                .padding(0)
                .style(sb::flat)
                .on_press_maybe(self.emit(Event::Toggle(r.value.clone())))
                .into()
        } else {
            space().width(arrow).into()
        };
        line = line.push(expander);

        if self.checkboxes {
            line = line.push(self.checkbox(r, g));
        }
        if let (Some(f), Some(n)) = (&self.icons, node)
            && let Some(icon) = f(n, r)
        {
            line = line.push(container(icon.view(g.icon_size)).padding(g.icon_padding));
        }

        let text: Element<'a, M> = match (&self.label, node) {
            (Some(f), Some(n)) => f(Payload {
                node: n,
                row: r.clone(),
                selected,
                checked: self.state.is_checked(&r.value),
                indeterminate: self.state.is_indeterminate(self.data, &r.value),
                text: self.text(r),
            }),
            _ => self.text(r),
        };
        line = line.push(
            button(text)
                .padding(g.label_padding)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(sb::flat_toggle(selected))
                .on_press_maybe(self.emit(Event::Click(r.value.clone()))),
        );
        if self.drag == DragMode::Handle {
            line = line.push(
                container(Icon::DragHandle.view(g.icon_size))
                    .width(g.grip_width)
                    .center_x(g.grip_width),
            );
        }
        container(line)
            .height(g.row_height)
            .width(Length::Fill)
            .into()
    }

    fn checkbox(&self, r: &Row, g: TreeLayout) -> Element<'a, M> {
        let on = self.state.is_checked(&r.value);
        let mixed = !on && self.state.is_indeterminate(self.data, &r.value);
        let size = g.icon_size + g.checkbox_size_extra;
        let mark: Element<'a, M> = if on {
            Icon::Check
                .colored(size - g.checkbox_glyph_inset, Color::WHITE)
                .into()
        } else if mixed {
            Icon::ListRemove
                .colored(size - g.checkbox_glyph_inset, Color::WHITE)
                .into()
        } else {
            space().into()
        };
        let boxed =
            container(mark)
                .center(size)
                .style(move |t: &Theme| iced::widget::container::Style {
                    background: (on || mixed).then_some(Background::Color(t.semantic.accent.bg)),
                    border: Border {
                        color: if on || mixed {
                            t.semantic.accent.bg
                        } else {
                            t.semantic.border
                        },
                        width: g.checkbox_border_width,
                        radius: g.checkbox_radius.into(),
                    },
                    ..Default::default()
                });
        button(container(boxed).padding(g.checkbox_padding))
            .padding(0)
            .style(sb::flat)
            .on_press_maybe(self.emit(Event::Check(r.value.clone(), !on)))
            .into()
    }
}

impl<'a, M: Clone + 'a> From<Tree<'a, M>> for Element<'a, M> {
    fn from(t: Tree<'a, M>) -> Self {
        let tokens = t.resolved_tokens();
        if !t.explicit_variant && tokens.text("tree", "breakpoint-rules").is_some() {
            crate::layouts::responsive(move |size| {
                let variant = t.selected_variant(&tokens, size.width);
                t.render(&tokens, &variant)
            })
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
        } else {
            t.render(&tokens, &t.variant)
        }
    }
}

impl<'a, M: Clone + 'a> Tree<'a, M> {
    fn render(&self, tokens: &crate::theme_config::Tokens, variant: &str) -> Element<'a, M> {
        let t = self;
        let g = t.geometry_for_variant(tokens, variant);
        let rows = t.state.rows(t.data);
        let range = t.window.clone().map_or(0..rows.len(), |w| {
            w.start.min(rows.len())..w.end.min(rows.len())
        });
        let elements: Vec<Element<'a, M>> =
            rows[range.clone()].iter().map(|r| t.row(r, g)).collect();
        let metas: Vec<RowMeta> = rows[range.clone()]
            .iter()
            .map(|r| RowMeta {
                value: r.value.clone(),
                level: r.level,
            })
            .collect();
        let valid: Option<Valid<'a>> = (t.drag != DragMode::Off).then(|| {
            let data = t.data;
            let allow = t.allow_drop.clone();
            Rc::new(move |from: &str, to: &str, pos: Position| {
                tree::can_move(data, from, to)
                    && match (&allow, tree::find(data, from), tree::find(data, to)) {
                        (Some(f), Some(a), Some(b)) => f(a, b, pos),
                        _ => true,
                    }
            }) as Valid<'a>
        });
        let on_drop = t.on_event.clone();
        let mut list = RowsWidget::new(elements, metas, t.drag, valid, g);
        list.emit = on_drop.map(|f| {
            Rc::new(move |from: String, to: String, position: Position| {
                f(Event::Drop { from, to, position })
            }) as DropEmit<'a, M>
        });
        let body: Element<'a, M> = Element::new(list);
        match &t.window {
            Some(_) => {
                let above = range.start as f32 * g.row_height;
                let below = (rows.len() - range.end) as f32 * g.row_height;
                iced::widget::column![space().height(above), body, space().height(below)].into()
            }
            None => body,
        }
    }
}

// ---------------------------------------------------------------------------
// The rows, and dragging them
// ---------------------------------------------------------------------------

struct RowMeta {
    value: String,
    level: usize,
}

type Valid<'a> = Rc<dyn Fn(&str, &str, Position) -> bool + 'a>;
type DropEmit<'a, M> = Rc<dyn Fn(String, String, Position) -> M + 'a>;

struct RowsWidget<'a, M> {
    rows: Vec<Element<'a, M>>,
    metas: Vec<RowMeta>,
    mode: DragMode,
    valid: Option<Valid<'a>>,
    emit: Option<DropEmit<'a, M>>,
    geometry: TreeLayout,
}

impl<'a, M> RowsWidget<'a, M> {
    fn new(
        rows: Vec<Element<'a, M>>,
        metas: Vec<RowMeta>,
        mode: DragMode,
        valid: Option<Valid<'a>>,
        geometry: TreeLayout,
    ) -> Self {
        Self {
            rows,
            metas,
            mode,
            valid,
            emit: None,
            geometry,
        }
    }
}

/// A drag in progress.
#[derive(Debug, Clone, Copy)]
struct Drag {
    from: usize,
    grab: f32,
    y: f32,
}

#[derive(Default)]
struct State {
    /// A row held but not yet moved far enough to be a drag: (row, where it was pressed).
    pending: Option<(usize, Point)>,
    drag: Option<Drag>,
}

/// Where a drop at pointer `y` would land: the row under it and the position, if allowed.
fn target(
    d: &Drag,
    rows: &[Rectangle],
    metas: &[RowMeta],
    valid: &Option<Valid<'_>>,
) -> Option<(usize, Position)> {
    let (first, last) = (rows.first()?, rows.last()?);
    let y = d.y.clamp(first.y, last.y + last.height - 0.01);
    let j = rows.iter().position(|r| y >= r.y && y < r.y + r.height)?;
    if j == d.from {
        return None;
    }
    let frac = (y - rows[j].y) / rows[j].height;
    let ok = |p: Position| {
        valid
            .as_ref()
            .is_some_and(|v| v(&metas[d.from].value, &metas[j].value, p))
    };
    let pos = tree::drop_position(frac, ok(Position::Inside));
    ok(pos).then_some((j, pos))
}

impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for RowsWidget<'a, M> {
    fn tag(&self) -> wtree::Tag {
        wtree::Tag::of::<State>()
    }
    fn state(&self) -> wtree::State {
        wtree::State::new(State::default())
    }
    fn children(&self) -> Vec<WidgetTree> {
        self.rows.iter().map(WidgetTree::new).collect()
    }
    fn diff(&self, t: &mut WidgetTree) {
        t.diff_children(&self.rows);
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        t: &mut WidgetTree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = limits.max().width;
        let child_limits = layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY));
        let mut y = 0.0;
        let mut nodes = Vec::with_capacity(self.rows.len());
        for (row, ct) in self.rows.iter_mut().zip(&mut t.children) {
            let node = row
                .as_widget_mut()
                .layout(ct, renderer, &child_limits)
                .move_to(Point::new(0.0, y));
            y += node.size().height;
            nodes.push(node);
        }
        layout::Node::with_children(Size::new(width, y), nodes)
    }

    fn update(
        &mut self,
        t: &mut WidgetTree,
        event: &IcedEvent,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        let bounds: Vec<Rectangle> = layout.children().map(|c| c.bounds()).collect();
        let state = t.state.downcast_mut::<State>();

        if let Some(mut d) = state.drag {
            match event {
                IcedEvent::Mouse(mouse::Event::CursorMoved { .. }) => {
                    if let Some(p) = cursor.position() {
                        d.y = p.y;
                        state.drag = Some(d);
                    }
                    shell.request_redraw();
                }
                IcedEvent::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    if let Some(p) = cursor.position() {
                        d.y = p.y;
                    }
                    let dropped = target(&d, &bounds, &self.metas, &self.valid);
                    state.drag = None;
                    if let (Some((j, pos)), Some(emit)) = (dropped, &self.emit) {
                        shell.publish(emit(
                            self.metas[d.from].value.clone(),
                            self.metas[j].value.clone(),
                            pos,
                        ));
                    }
                    // The row's button saw the press: let it go without a click.
                    for ((row, ct), l) in self
                        .rows
                        .iter_mut()
                        .zip(&mut t.children)
                        .zip(layout.children())
                    {
                        row.as_widget_mut().update(
                            ct,
                            event,
                            l,
                            mouse::Cursor::Unavailable,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );
                    }
                    shell.request_redraw();
                }
                IcedEvent::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                }) => {
                    state.drag = None;
                    shell.request_redraw();
                }
                _ => {}
            }
            shell.capture_event();
            return;
        }

        if self.mode != DragMode::Off {
            match event {
                IcedEvent::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    if let Some(p) = cursor.position()
                        && let Some(i) = bounds.iter().position(|b| b.contains(p))
                    {
                        match self.mode {
                            DragMode::Handle => {
                                let grip = Rectangle {
                                    x: bounds[i].x + bounds[i].width - self.geometry.grip_width,
                                    width: self.geometry.grip_width,
                                    ..bounds[i]
                                };
                                if grip.contains(p) {
                                    state.drag = Some(Drag {
                                        from: i,
                                        grab: p.y - bounds[i].y,
                                        y: p.y,
                                    });
                                    shell.capture_event();
                                    shell.request_redraw();
                                    return;
                                }
                            }
                            _ => state.pending = Some((i, p)),
                        }
                    }
                }
                IcedEvent::Mouse(mouse::Event::CursorMoved { .. }) => {
                    if let (Some((i, from)), Some(p)) = (state.pending, cursor.position())
                        && ((p.x - from.x).powi(2) + (p.y - from.y).powi(2)).sqrt()
                            > self.geometry.drag_threshold
                    {
                        state.pending = None;
                        state.drag = Some(Drag {
                            from: i,
                            grab: from.y - bounds[i].y,
                            y: p.y,
                        });
                        shell.capture_event();
                        shell.request_redraw();
                        return;
                    }
                }
                IcedEvent::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    state.pending = None;
                }
                _ => {}
            }
        }

        for ((row, ct), l) in self
            .rows
            .iter_mut()
            .zip(&mut t.children)
            .zip(layout.children())
        {
            row.as_widget_mut()
                .update(ct, event, l, cursor, renderer, clipboard, shell, viewport);
        }
    }

    fn mouse_interaction(
        &self,
        t: &WidgetTree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if let Some(d) = t.state.downcast_ref::<State>().drag {
            let bounds: Vec<Rectangle> = layout.children().map(|c| c.bounds()).collect();
            return if target(&d, &bounds, &self.metas, &self.valid).is_some() {
                mouse::Interaction::Grabbing
            } else {
                mouse::Interaction::NotAllowed
            };
        }
        if self.mode == DragMode::Handle
            && layout.children().any(|c| {
                let b = c.bounds();
                cursor.is_over(Rectangle {
                    x: b.x + b.width - self.geometry.grip_width,
                    width: self.geometry.grip_width,
                    ..b
                })
            })
        {
            return mouse::Interaction::Grab;
        }
        self.rows
            .iter()
            .zip(&t.children)
            .zip(layout.children())
            .map(|((row, ct), l)| {
                row.as_widget()
                    .mouse_interaction(ct, l, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        t: &WidgetTree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let drag = t.state.downcast_ref::<State>().drag;
        let bounds: Vec<Rectangle> = layout.children().map(|c| c.bounds()).collect();
        let accent = theme.semantic.accent.bg;
        let drop = drag.and_then(|d| target(&d, &bounds, &self.metas, &self.valid));

        for (i, ((row, ct), l)) in self
            .rows
            .iter()
            .zip(&t.children)
            .zip(layout.children())
            .enumerate()
        {
            let held = drag.is_some_and(|d| d.from == i);
            row.as_widget().draw(
                ct,
                renderer,
                theme,
                style,
                l,
                if drag.is_some() {
                    mouse::Cursor::Unavailable
                } else {
                    cursor
                },
                viewport,
            );
            if held {
                // The place it came from, dimmed.
                renderer.fill_quad(
                    Quad {
                        bounds: bounds[i],
                        border: Border::default().rounded(self.geometry.row_radius),
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    crate::color::alpha(theme.semantic.window.bg, 0.6),
                );
            }
        }

        // The target: a line before or after the row (from its own indent), or the row tinted
        // and ringed to say "inside".
        if let Some((j, pos)) = drop {
            let b = bounds[j];
            let indent = self.metas[j].level as f32 * self.geometry.level_offset;
            match pos {
                Position::Inside => renderer.fill_quad(
                    Quad {
                        bounds: b,
                        border: Border {
                            color: accent,
                            width: self.geometry.drop_border_width,
                            radius: self.geometry.row_radius.into(),
                        },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    crate::color::alpha(accent, 0.14),
                ),
                _ => {
                    let y = if pos == Position::Before {
                        b.y
                    } else {
                        b.y + b.height
                    } - self.geometry.drop_line_width / 2.0;
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(b.x + indent, y),
                                Size::new(
                                    (b.width - indent).max(0.0),
                                    self.geometry.drop_line_width,
                                ),
                            ),
                            border: Border::default().rounded(self.geometry.drop_line_radius),
                            shadow: Shadow::default(),
                            snap: false,
                        },
                        accent,
                    );
                }
            }
        }

        // The held row, lifted: opaque, ringed and shadowed, following the pointer.
        if let Some(d) = drag
            && let (Some(row), Some(ct), Some(l)) = (
                self.rows.get(d.from),
                t.children.get(d.from),
                layout.children().nth(d.from),
            )
        {
            let b = l.bounds();
            let (first, last) = (bounds[0], bounds[bounds.len() - 1]);
            let top = (d.y - d.grab).clamp(first.y, (last.y + last.height - b.height).max(first.y));
            let dy = top - b.y;
            let lift = theme.shadows.popover();
            let (cx, cy) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
            let grow = Transformation::translate(cx, cy)
                * Transformation::scale(self.geometry.drag_scale)
                * Transformation::translate(-cx, -cy);
            let solid = Color {
                a: 1.0,
                ..theme.semantic.view.bg
            };
            renderer.with_layer(
                layout.bounds().expand(self.geometry.drag_clip_margin),
                |renderer| {
                    renderer.with_translation(Vector::new(0.0, dy), |renderer| {
                        renderer.with_transformation(grow, |renderer| {
                            renderer.fill_quad(
                                Quad {
                                    bounds: b,
                                    border: Border {
                                        color: accent,
                                        width: self.geometry.drag_border_width,
                                        radius: self.geometry.row_radius.into(),
                                    },
                                    shadow: Shadow {
                                        color: lift.shadow.color,
                                        offset: Vector::new(0.0, self.geometry.drag_shadow_offset),
                                        blur_radius: lift.shadow.blur_radius,
                                    },
                                    snap: false,
                                },
                                Background::Color(solid),
                            );
                            let vp = Rectangle {
                                y: viewport.y - dy,
                                ..*viewport
                            };
                            row.as_widget().draw(
                                ct,
                                renderer,
                                theme,
                                style,
                                l,
                                mouse::Cursor::Unavailable,
                                &vp,
                            );
                        });
                    });
                },
            );
        }
    }

    fn operate(
        &mut self,
        t: &mut WidgetTree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        for ((row, ct), l) in self
            .rows
            .iter_mut()
            .zip(&mut t.children)
            .zip(layout.children())
        {
            row.as_widget_mut().operate(ct, l, renderer, operation);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        t: &'b mut WidgetTree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        let overlays: Vec<_> = self
            .rows
            .iter_mut()
            .zip(&mut t.children)
            .zip(layout.children())
            .filter_map(|((row, ct), l)| {
                row.as_widget_mut()
                    .overlay(ct, l, renderer, viewport, translation)
            })
            .collect();
        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::tree::TreeNode as N;

    #[test]
    fn layout_resolves_design_tokens_and_inherits_only_metrics() {
        let m = Metrics::desktop();
        let inherited = [
            "icon-size",
            "icon-padding",
            "label-padding",
            "checkbox-padding",
            "row-radius",
        ];
        let g = TreeLayout::resolve_numbers(&m, |key| (!inherited.contains(&key)).then_some(7.0));
        assert_eq!(g.level_offset, 7.0);
        assert_eq!(g.row_height, 7.0);
        assert_eq!(g.grip_width, 7.0);
        assert_eq!(g.drag_threshold, 7.0);
        assert_eq!(g.checkbox_border_width, 7.0);
        assert_eq!(g.connector_width, 7.0);
        assert_eq!(g.drop_line_width, 7.0);
        assert_eq!(g.icon_size, m.icon_size);
        assert_eq!(g.icon_padding, iced::Padding::from([0.0, m.space_xs]));
        assert_eq!(g.label_padding, iced::Padding::from([0.0, m.space_s]));
        assert_eq!(g.row_radius, m.radius_s);
        let explicit = TreeLayout::resolve_numbers(&m, |_| Some(9.0));
        assert_eq!(explicit.icon_size, 9.0);
        assert_eq!(explicit.checkbox_padding, iced::Padding::from([0.0, 9.0]));
        assert_eq!(explicit.row_radius, 9.0);
    }

    #[test]
    #[should_panic(expected = "missing [tree] level-offset token")]
    fn design_tokens_are_required() {
        TreeLayout::resolve_numbers(&Metrics::desktop(), |_| None);
    }

    #[test]
    fn variants_inherit_per_key_and_explicit_builders_win() {
        use crate::theme_config::{Schema, ThemeConfig};
        let tokens = ThemeConfig::builder("tree-variants")
            .files(false)
            .schema(Schema::new().variant("tree", "compact", &[]))
            .app_text("[tree]\nlevel-offset=21px\nrow-height=35px\nicon-padding=6px\n[tree.compact]\nrow-height=27px\nlabel-padding=3px")
            .build()
            .tokens();
        let d = data();
        let state = TreeState::new();
        let t = Tree::<()>::new(&d, &state);
        assert_eq!(t.variant, "default");
        let base = t.geometry_with_tokens(&tokens);
        let t = t.variant("compact");
        let compact = t.geometry_with_tokens(&tokens);
        assert_eq!(compact.row_height, 27.0);
        assert_eq!(compact.level_offset, 21.0);
        assert_eq!(compact.icon_padding, iced::Padding::from([0.0, 6.0]));
        assert_eq!(compact.label_padding, iced::Padding::from([0.0, 3.0]));
        assert_eq!(compact.grip_width, base.grip_width);
        let explicit = t
            .level_offset(45.0)
            .row_height(50.0)
            .icon_padding(1.0)
            .label_padding(2.0)
            .checkbox_padding(3.0)
            .geometry_with_tokens(&tokens);
        assert_eq!(explicit.level_offset, 45.0);
        assert_eq!(explicit.row_height, 50.0);
        assert_eq!(explicit.icon_padding, iced::Padding::from(1.0));
        assert_eq!(explicit.label_padding, iced::Padding::from(2.0));
        assert_eq!(explicit.checkbox_padding, iced::Padding::from(3.0));
        let unknown = Tree::<()>::new(&d, &state)
            .variant("undeclared")
            .geometry_with_tokens(&tokens);
        assert_eq!(unknown.row_height, base.row_height);
        assert_eq!(unknown.level_offset, base.level_offset);
        assert_eq!(unknown.label_padding, base.label_padding);
        assert_eq!(unknown.icon_size, base.icon_size);
    }

    #[test]
    fn responsive_boundaries_local_isolation_and_explicit_precedence() {
        use crate::theme_config::{Schema, ThemeConfig};
        let cfg = ThemeConfig::builder("tree-local")
            .files(false)
            .schema(Schema::new().variant("tree", "narrow", &[]).variant("tree", "wide", &[]))
            .app_text("[tree]\nbreakpoint-rules=xs:narrow,md:wide\nlevel-offset=23px\n[tree.narrow]\nrow-height=26px\n[tree.wide]\nrow-height=42px\ngrip-width=48px")
            .build();
        let other = ThemeConfig::builder("tree-other")
            .files(false)
            .app_text("[tree]\nrow-height=61px")
            .build();
        let d = data();
        let state = TreeState::new();
        let mut metrics = Metrics::desktop();
        metrics.icon_size = 29.0;
        let t = Tree::<()>::new(&d, &state)
            .theme_config(&cfg)
            .metrics(metrics);
        let tokens = t.resolved_tokens();
        let md = crate::breakpoints::Breakpoints::current().md;
        for (width, expected, height) in [
            (0.0, "narrow", 26.0),
            (md - 0.01, "narrow", 26.0),
            (md, "wide", 42.0),
            (md + 1.0, "wide", 42.0),
        ] {
            let variant = t.selected_variant(&tokens, width);
            assert_eq!(variant, expected);
            let g = t.geometry_for_variant(&tokens, &variant);
            assert_eq!(g.row_height, height);
            assert_eq!(g.level_offset, 23.0);
            assert_eq!(g.icon_size, 29.0);
            if expected == "wide" {
                assert_eq!(g.grip_width, 48.0);
            }
        }
        let independent = Tree::<()>::new(&d, &state).tokens(other.tokens());
        assert_eq!(
            independent
                .geometry_with_tokens(&independent.resolved_tokens())
                .row_height,
            61.0
        );
        assert_eq!(
            t.geometry_with_tokens(&t.resolved_tokens()).level_offset,
            23.0
        );
        let explicit = t.variant("undeclared").row_height(57.0);
        assert_eq!(explicit.selected_variant(&tokens, md), "undeclared");
        let g = explicit.geometry_for_variant(&tokens, "undeclared");
        assert_eq!(g.row_height, 57.0);
        assert_eq!(g.level_offset, 23.0);
        let _: Element<'_, ()> = explicit.into();
        // No rules means the original unwrapped rows widget is retained.
        let plain: Element<'_, ()> = independent.into();
        assert_eq!(plain.as_widget().children().len(), state.rows(&d).len());
    }

    fn data() -> Vec<N> {
        vec![
            N::branch("a", "A", vec![N::new("a1", "A1")]),
            N::new("b", "B"),
        ]
    }

    #[test]
    fn the_row_parts_take_their_own_padding() {
        let d = data();
        let state = TreeState::new();
        let t = Tree::<()>::new(&d, &state)
            .checkboxes(true)
            .icons(|_, _| Some(Icon::Folder))
            .icon_padding([0.0, 1.0])
            .label_padding([0.0, 2.0])
            .checkbox_padding([0.0, 3.0]);
        assert_eq!(t.label_padding, Some(iced::Padding::from([0.0, 2.0])));
        let _: Element<'_, ()> = t.into();
    }

    /// The rows the widget builds: open branches show their children; a window builds fewer.
    #[test]
    fn builds_a_row_per_visible_node_or_only_the_window() {
        let d = data();
        let mut s = TreeState::new();
        s.expand("a");
        let all: Element<'_, u8> = Tree::new(&d, &s).on_event(|_| 0).into();
        // The middle row only: spacers above and below, with the rows between them.
        let part: Element<'_, u8> = Tree::new(&d, &s).window(1..2).on_event(|_| 0).into();
        assert_eq!(
            all.as_widget().children().len(),
            3,
            "a row per visible node"
        );
        assert_eq!(part.as_widget().children().len(), 3, "spacer, rows, spacer");
    }

    #[test]
    fn a_drop_is_refused_into_the_nodes_own_branch() {
        let d = data();
        assert!(!tree::can_move(&d, "a", "a1"));
        assert!(tree::can_move(&d, "b", "a1"));
    }
}
