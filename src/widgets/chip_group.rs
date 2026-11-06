//! `ChipGroup`: a wrapping set of toggle chips, optionally filtered by a find-as-you-type entry.
//! [`MultiSelect`](crate::widgets::MultiSelect) is a dropdown; this is the in-page version, for tags,
//! genres, moods and filters.
//!
//! The application owns the selection ([controlled](crate::widgets)) and the query. Each option
//! has a key, a label and optionally an icon and a color of its own; the chips are
//! [`Chip`](crate::components::Chip)s, so [`size`](ChipGroup::size) and [`on_color`](ChipGroup::on_color)
//! apply to all.
//!
//! Opt into inline expansion with `.max_lines(3).expandable(self.open, Msg::Expand)`.
//! The actual wrapped height decides whether a trailing chevron is needed, so resizing or
//! searching a short list never leaves an unnecessary control. The entry is not clamped.
//!
//! ```ignore
//! ChipGroup::new(
//!     tags.iter().map(|t| ChipOption::new(t.id, &t.name).color(t.color)),
//!     &self.selected,
//!     Msg::Toggle,                                   // gets the key of the pressed chip
//! )
//! .search(&self.query, Msg::Query)                  // an entry above filters by label
//! ```

use crate::Theme;
use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced::{Event, Length, Point, Rectangle, Size, Vector};

use crate::Element;
use crate::components::{Chip, ChipSize, entry, typography as ty};
use crate::styles::text as st;
use crate::{IconSource, Metrics};
use iced::Color;
use iced::widget::{column, row};

/// One chip of a [`ChipGroup`].
#[derive(Clone)]
pub struct ChipOption<K> {
    key: K,
    label: String,
    icon: Option<IconSource>,
    color: Option<Color>,
}

impl<K> ChipOption<K> {
    pub fn new(key: K, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            icon: None,
            color: None,
        }
    }
    /// An icon of the chip's own (a built-in or your own SVG).
    pub fn icon(mut self, icon: impl Into<IconSource>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    /// A color of the chip's own, in place of the group's role.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// Whether `label` matches the `query`: every word of the query is somewhere in it, ignoring
/// case. An empty query matches everything.
pub fn matches(label: &str, query: &str) -> bool {
    let label = label.to_lowercase();
    query
        .to_lowercase()
        .split_whitespace()
        .all(|word| label.contains(word))
}

struct Expansion<'a, M> {
    expanded: bool,
    on_toggle: Box<dyn Fn(bool) -> M + 'a>,
}

/// See the [module](self).
pub struct ChipGroup<'a, K, M> {
    options: Vec<ChipOption<K>>,
    selected: &'a [K],
    on_toggle: Box<dyn Fn(K) -> M + 'a>,
    query: Option<&'a str>,
    on_query: Option<Box<dyn Fn(String) -> M + 'a>>,
    placeholder: String,
    empty: String,
    size: ChipSize,
    on_color: Option<Color>,
    min_contrast: f32,
    max_lines: Option<usize>,
    expand: Option<Expansion<'a, M>>,
    metrics: Metrics,
}

impl<'a, K: PartialEq + Clone + 'a, M: Clone + 'a> ChipGroup<'a, K, M> {
    /// `selected` are the keys that are on; `on_toggle` gets the key of a pressed chip.
    pub fn new(
        options: impl IntoIterator<Item = ChipOption<K>>,
        selected: &'a [K],
        on_toggle: impl Fn(K) -> M + 'a,
    ) -> Self {
        Self {
            options: options.into_iter().collect(),
            selected,
            on_toggle: Box::new(on_toggle),
            query: None,
            on_query: None,
            placeholder: "Search".into(),
            empty: "No matches".into(),
            size: ChipSize::Medium,
            on_color: None,
            min_contrast: 4.5,
            max_lines: None,
            expand: None,
            metrics: Metrics::desktop(),
        }
    }
    /// An entry above the chips that filters them by label as you type. `query` is what it holds
    /// (controlled); `on_query` gets each edit.
    pub fn search(mut self, query: &'a str, on_query: impl Fn(String) -> M + 'a) -> Self {
        self.query = Some(query);
        self.on_query = Some(Box::new(on_query));
        self
    }
    /// Filters the chips by `query` **without drawing an entry**: the application owns the field
    /// (shared by several groups, in a dialog's own layout). [`search`](Self::search) is this plus
    /// the entry.
    pub fn query(mut self, query: &'a str) -> Self {
        self.query = Some(query);
        self
    }
    /// The entry's placeholder (default "Search").
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }
    /// What shows when no chip matches (default "No matches").
    pub fn empty_text(mut self, text: impl Into<String>) -> Self {
        self.empty = text.into();
        self
    }
    /// Shows at most `lines` lines of chips: the rest is clipped, not drawn, and cannot be
    /// pressed. Put the chips that must stay visible (the selected ones) first, and let the query
    /// reach the others: an autocomplete with a limit.
    pub fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines.max(1));
        self
    }
    /// A trailing expand/collapse chevron when wrapping chips exceed `max_lines` (default three).
    /// Expansion is controlled by the caller. The search entry remains outside the clamp,
    /// and hidden chips cannot receive pointer input. Short/search-filtered lists omit the chevron.
    pub fn expandable(mut self, expanded: bool, on_toggle: impl Fn(bool) -> M + 'a) -> Self {
        self.expand = Some(Expansion {
            expanded,
            on_toggle: Box::new(on_toggle),
        });
        self
    }

    pub fn size(mut self, size: ChipSize) -> Self {
        self.size = size;
        self
    }
    /// The text and icon color of selected chips that have a color of their own (see
    /// [`Chip::on_color`]).
    pub fn on_color(mut self, color: Color) -> Self {
        self.on_color = Some(color);
        self
    }
    /// The least contrast of unselected chips' own-colored text against the page (default 4.5;
    /// `0.0` keeps the colors as given). See [`Chip::min_contrast`].
    pub fn min_contrast(mut self, ratio: f32) -> Self {
        self.min_contrast = ratio;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    /// The options the query leaves.
    fn visible(&self) -> Vec<&ChipOption<K>> {
        let q = self.query.unwrap_or("");
        self.options
            .iter()
            .filter(|o| matches(&o.label, q))
            .collect()
    }
}

impl<'a, K: PartialEq + Clone + 'a, M: Clone + 'a> From<ChipGroup<'a, K, M>> for Element<'a, M> {
    fn from(g: ChipGroup<'a, K, M>) -> Self {
        let m = g.metrics;
        let visible = g.visible();
        let body: Element<'a, M> = if visible.is_empty() {
            ty::caption(g.empty.clone()).style(st::dim).into()
        } else {
            let chips: Vec<Element<'a, M>> = visible
                .into_iter()
                .map(|o| {
                    let mut c = Chip::new(o.label.clone())
                        .size(g.size)
                        .selected(g.selected.contains(&o.key))
                        .on_press((g.on_toggle)(o.key.clone()));
                    if let Some(i) = o.icon.clone() {
                        c = c.icon(i);
                    }
                    if let Some(col) = o.color {
                        c = c.color(col).min_contrast(g.min_contrast);
                        if let Some(on) = g.on_color {
                            c = c.on_color(on);
                        }
                    }
                    Element::from(c)
                })
                .collect();
            if let Some(Expansion {
                expanded,
                on_toggle,
            }) = g.expand
            {
                let height = g.size.height(&m);
                let control = crate::components::flat_button(
                    crate::Icon::GoUp.view(14.0),
                );
                let control = if expanded {
                    control
                } else {
                    crate::components::flat_button(
                        crate::Icon::GoDown.view(14.0),
                    )
                }
                .width(iced::Length::Fixed(height))
                .height(iced::Length::Fixed(height))
                .padding(0)
                .on_press(on_toggle(!expanded));
                let mut items = chips;
                items.push(crate::components::tooltip(
                    control,
                    if expanded { "Collapse" } else { "Expand" },
                    crate::components::TooltipPosition::Bottom,
                ));
                iced::Element::new(ChipFlow {
                    items,
                    lines: g.max_lines.unwrap_or(3),
                    expanded,
                    gap: m.space_s,
                })
            } else {
                let wrapped = row(chips)
                    .spacing(m.space_s)
                    .width(iced::Length::Fill)
                    .wrap()
                    .vertical_spacing(m.space_s);
                if let Some(lines) = g.max_lines {
                    let lines = lines as f32;
                    let height = lines * g.size.height(&m) + (lines - 1.0) * m.space_s + 1.0;
                    iced::widget::container(wrapped)
                        .width(iced::Length::Fill)
                        .max_height(height)
                        .clip(true)
                        .into()
                } else {
                    wrapped.into()
                }
            }
        };
        // The entry is drawn only when `search` gave it somewhere to send what is typed.
        match (g.query, g.on_query) {
            (Some(q), Some(on_query)) => column![
                entry(g.placeholder, q.to_string())
                    .on_input(on_query)
                    .metrics(m),
                body
            ]
            .spacing(m.space_m)
            .into(),
            _ => body,
        }
    }
}

/// Each child retains its tree slot; hidden children are excluded from every traversal.
/// The last child is the chevron. Reserving its actual width prevents a fourth control row.
struct ChipFlow<'a, M> {
    items: Vec<Element<'a, M>>,
    lines: usize,
    expanded: bool,
    gap: f32,
}
#[derive(Default)]
struct FlowState {
    shown: usize,
    overflow: bool,
}
impl FlowState {
    fn active(&self, index: usize, count: usize) -> bool {
        index < self.shown || (self.overflow && index + 1 == count)
    }
}

fn place(sizes: impl IntoIterator<Item = Size>, width: f32, gap: f32) -> (Vec<Point>, usize, f32) {
    let (mut x, mut y, mut row_height, mut rows) = (0.0f32, 0.0f32, 0.0f32, 1usize);
    let mut points = Vec::new();
    for size in sizes {
        if x > 0.0 && x + size.width > width {
            y += row_height + gap;
            x = 0.0;
            row_height = 0.0;
            rows += 1;
        }
        points.push(Point::new(x, y));
        x += size.width + gap;
        row_height = row_height.max(size.height);
    }
    (points, rows, y + row_height)
}
impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for ChipFlow<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<FlowState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(FlowState::default())
    }
    fn children(&self) -> Vec<Tree> {
        self.items.iter().map(Tree::new).collect()
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.items);
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = limits.max().width;
        let own = layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY));
        let mut nodes: Vec<_> = self
            .items
            .iter_mut()
            .zip(&mut tree.children)
            .map(|(item, state)| item.as_widget_mut().layout(state, renderer, &own))
            .collect();
        let count = nodes.len() - 1;
        let sizes: Vec<_> = nodes[..count].iter().map(|n| n.size()).collect();
        let control = nodes[count].size();
        let (natural, rows, _) = place(sizes.iter().copied(), width, self.gap);
        for (node, point) in nodes[..count].iter_mut().zip(natural) {
            node.move_to_mut(point);
        }
        let overflow = rows > self.lines;
        let shown = if !overflow || self.expanded {
            count
        } else {
            // Scan once, keeping the largest prefix for which the chevron also fits.
            let (mut x, mut rows, mut shown) = (0.0, 1usize, 0);
            for (i, size) in sizes.iter().enumerate() {
                if x > 0.0 && x + size.width > width {
                    rows += 1;
                    x = 0.0;
                }
                x += size.width + self.gap;
                let control_rows = rows + usize::from(x + control.width > width);
                if control_rows > self.lines {
                    break;
                }
                shown = i + 1;
            }
            shown
        };
        let visible_sizes = sizes[..shown]
            .iter()
            .copied()
            .chain(overflow.then_some(control));
        let (points, _, height) = place(visible_sizes, width, self.gap);
        for (node, point) in nodes[..shown].iter_mut().zip(&points) {
            node.move_to_mut(*point);
        }
        if overflow {
            nodes[count].move_to_mut(points[shown]);
        }
        *tree.state.downcast_mut::<FlowState>() = FlowState { shown, overflow };
        layout::Node::with_children(Size::new(width, height), nodes)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_ref::<FlowState>();
        let count = self.items.len();
        operation.container(None, layout.bounds());
        operation.traverse(&mut |op| {
            for (i, ((item, tree), layout)) in self
                .items
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .enumerate()
            {
                if state.active(i, count) {
                    item.as_widget_mut().operate(tree, layout, renderer, op);
                }
            }
        });
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
        let state = tree.state.downcast_ref::<FlowState>();
        let count = self.items.len();
        for (i, ((item, tree), layout)) in self
            .items
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .enumerate()
        {
            if state.active(i, count) {
                item.as_widget_mut().update(
                    tree, event, layout, cursor, renderer, clipboard, shell, viewport,
                );
            }
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
        let state = tree.state.downcast_ref::<FlowState>();
        self.items
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .enumerate()
            .filter(|(i, _)| state.active(*i, self.items.len()))
            .map(|(_, ((item, tree), layout))| {
                item.as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
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
        let Some(viewport) = layout.bounds().intersection(viewport) else {
            return;
        };
        let state = tree.state.downcast_ref::<FlowState>();
        for (i, ((item, tree), layout)) in self
            .items
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .enumerate()
        {
            if state.active(i, self.items.len()) {
                item.as_widget()
                    .draw(tree, renderer, theme, style, layout, cursor, &viewport);
            }
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
        if !tree.state.downcast_ref::<FlowState>().overflow {
            return None;
        }
        self.items.last_mut()?.as_widget_mut().overlay(
            tree.children.last_mut()?,
            layout.children().last()?,
            renderer,
            viewport,
            translation,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_word_of_the_query_must_match() {
        assert!(matches("Late Night Drive", "late drive"));
        assert!(matches("Late Night Drive", ""));
        assert!(matches("Late Night Drive", "NIGHT"));
        assert!(!matches("Late Night Drive", "late day"));
    }

    #[test]
    fn a_query_filters_without_an_entry() {
        let sel: [u8; 0] = [];
        let g: ChipGroup<'_, u8, ()> = ChipGroup::new(
            [ChipOption::new(1, "Jazz"), ChipOption::new(2, "Rock")],
            &sel,
            |_| (),
        )
        .query("rock");
        assert!(g.on_query.is_none());
        assert_eq!(g.visible().len(), 1);
    }

    #[test]
    fn the_query_filters_the_options() {
        let sel: [u8; 0] = [];
        let g: ChipGroup<'_, u8, ()> = ChipGroup::new(
            [
                ChipOption::new(1, "Jazz"),
                ChipOption::new(2, "Jazz fusion"),
                ChipOption::new(3, "Rock"),
            ],
            &sel,
            |_| (),
        )
        .search("jazz", |_| ());
        let labels: Vec<&str> = g.visible().iter().map(|o| o.label.as_str()).collect();
        assert_eq!(labels, vec!["Jazz", "Jazz fusion"]);
    }

    #[test]
    fn max_lines_limits_the_height_and_a_chip_has_a_height() {
        let sel: [u8; 0] = [];
        let g: ChipGroup<'_, u8, ()> =
            ChipGroup::new([ChipOption::new(1, "Jazz")], &sel, |_| ()).max_lines(2);
        assert_eq!(g.max_lines, Some(2));
        let _: Element<'_, ()> = g.into();
        let m = Metrics::desktop();
        // About 22 px for the default size, as documented.
        let h = ChipSize::Medium.height(&m);
        assert!((18.0..=26.0).contains(&h), "{h}");
        assert!(ChipSize::Small.height(&m) < h && h < ChipSize::Large.height(&m));
    }
    fn renderer() -> iced::Renderer {
        use iced::advanced::renderer::Headless;
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer")
    }

    fn group(expanded: bool, query: &'static str) -> Element<'static, (usize, bool)> {
        ChipGroup::new(
            (0..36).map(|i| ChipOption::new(i, format!("Long selectable tag {i}"))),
            &[],
            |key| (key, false),
        )
        .query(query)
        .max_lines(3)
        .expandable(expanded, |on| (usize::MAX, on))
        .into()
    }

    #[test]
    fn chevron_is_inline_on_the_third_row_and_toggles_without_pressing_hidden_chips() {
        use iced::advanced::{Shell, clipboard, layout, widget::Tree};
        use iced::{Event, Point, Rectangle, Size, mouse};
        let renderer = renderer();
        for expanded in [false, true] {
            let mut el = group(expanded, "");
            let mut tree = Tree::new(&el);
            let node = el.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(300.0, 2000.0)),
            );
            let state = tree.state.downcast_ref::<FlowState>();
            assert!(state.overflow);
            let shown = state.shown;
            assert_eq!(shown == 36, expanded);
            let last = node.children()[shown - 1].bounds();
            let control = node.children().last().unwrap().bounds();
            assert_eq!(control.y, last.y);
            assert_eq!(control.x, last.x + last.width + Metrics::desktop().space_s);
            assert!(control.x + control.width <= 300.0);
            assert_eq!(node.size().height, control.y + control.height);
            if !expanded {
                let rows: std::collections::HashSet<_> = node.children()[..shown]
                    .iter()
                    .map(|n| n.bounds().y.to_bits())
                    .collect();
                assert_eq!(rows.len(), 3);
            }
            let view = Rectangle::new(Point::ORIGIN, Size::new(300.0, 2000.0));
            let layout = layout::Layout::new(&node);
            let mut messages = Vec::new();
            for at in [node.children()[35].bounds().center(), control.center()] {
                for event in [
                    mouse::Event::ButtonPressed(mouse::Button::Left),
                    mouse::Event::ButtonReleased(mouse::Button::Left),
                ] {
                    el.as_widget_mut().update(
                        &mut tree,
                        &Event::Mouse(event),
                        layout,
                        mouse::Cursor::Available(at),
                        &renderer,
                        &mut clipboard::Null,
                        &mut Shell::new(&mut messages),
                        &view,
                    );
                }
            }
            let expected = if expanded {
                vec![(35, false), (usize::MAX, false)]
            } else {
                vec![(usize::MAX, true)]
            };
            assert_eq!(messages, expected);
        }
        // Re-layout at a new width, retaining widget state: no needless expansion affordance.
        let mut el = group(false, "");
        let mut tree = Tree::new(&el);
        for width in [300.0, 8000.0, 300.0] {
            el.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(width, 2000.0)),
            );
            assert_eq!(
                tree.state.downcast_ref::<FlowState>().overflow,
                width < 8000.0
            );
        }
        let mut el = group(false, "tag 35");
        let mut tree = Tree::new(&el);
        el.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(300.0, 2000.0)),
        );
        assert!(!tree.state.downcast_ref::<FlowState>().overflow);
    }
}
