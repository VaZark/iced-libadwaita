//! `TreeRow`: a row that can hold other rows, or any elements, nested under it. Inspired by
//! [`ExpanderRow`](super::ExpanderRow), made for hierarchies: the **arrow** at the start opens
//! and closes it (GTK's `GtkTreeExpander`), the **title** is a separate target that activates
//! the row (select it, open it), and a leaf keeps the arrow's width so labels line up.
//!
//! **Scope.** A `TreeRow` nests elements *within a page*: a group of settings, an outline, a
//! file list, rows that belong together on one screen. It is **not for logic that spans pages**:
//! moving between pages, a sidebar of destinations or a navigation stack belong to
//! [`NavigationView`](super::NavigationView) and the sidebar components, which the HIG keeps flat.
//! If opening a row changes what page you are on, it is navigation, not a tree row.
//!
//! Children are anything: more `TreeRow`s, [`ActionRow`](super::ActionRow)s, switch rows, or a
//! plain widget. They are pushed in and shown on the shaded list that slides open, each indented
//! by [`indent`](TreeRow::indent) (which starts at the toolkit's `[rows] expander-indent` and is
//! overridden per row).
//!
//! ```ignore
//! TreeRow::new("Network")
//!     .subtitle("Proxy and name servers")
//!     .expanded(self.open)
//!     .on_toggle(Msg::Network)
//!     .push(TreeRow::new("Proxy").subtitle("Automatic").on_activate(Msg::Proxy))
//!     .push(SwitchRow::new("Use VPN", self.vpn, Msg::Vpn))
//!     .push(my_custom_widget())
//! ```
//!
//! Controlled: the app owns `expanded` and `selected`. A row with children shows the arrow;
//! [`expandable`](TreeRow::expandable) shows it before the children exist (lazy loading).

use super::{ActionRow, Element};
use crate::style::button as sb;
use crate::style::container as sc;
use crate::{Icon, Metrics, Theme};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, space};
use iced::{Background, Length};

/// See the [module](self).
pub struct TreeRow<'a, M> {
    row: ActionRow<'a, M>,
    selected: bool,
    expanded: bool,
    expandable: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> M + 'a>>,
    children: Vec<Element<'a, M>>,
    indent: f32,
    metrics: Metrics,
}

impl<'a, M: 'a> TreeRow<'a, M> {
    pub fn new(title: impl Into<String>) -> Self {
        let metrics = Metrics::desktop();
        Self {
            row: ActionRow::new(title).metrics(metrics),
            selected: false,
            expanded: false,
            expandable: false,
            on_toggle: None,
            children: Vec::new(),
            // The toolkit's default; `indent(..)` overrides it for this row.
            indent: metrics.expander_indent,
            metrics,
        }
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.row = self.row.subtitle(s);
        self
    }
    /// At the start of the row, after the arrow.
    pub fn prefix(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.row = self.row.prefix(e);
        self
    }
    pub fn suffix(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.row = self.row.suffix(e);
        self
    }
    /// Pressing the title (not the arrow): select it, open it, run it.
    pub fn on_activate(mut self, m: M) -> Self {
        self.row = self.row.on_activate(m);
        self
    }
    /// The row is the current selection: it lies on the selection highlight.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// Whether the children show (controlled).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
    /// Pressing the arrow, with the state the row would take. Without it the arrow is inert.
    pub fn on_toggle(mut self, f: impl Fn(bool) -> M + 'a) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }
    /// Shows the arrow even without children yet (they load when it opens).
    pub fn expandable(mut self, on: bool) -> Self {
        self.expandable = on;
        self
    }
    /// How far the children are indented, in pixels. It starts at the toolkit's default,
    /// [`Metrics::expander_indent`] (`[rows] expander-indent`); this overrides it for this row.
    pub fn indent(mut self, px: f32) -> Self {
        self.indent = px.max(0.0);
        self
    }
    /// A nested row, or any element.
    pub fn push(mut self, child: impl Into<Element<'a, M>>) -> Self {
        self.children.push(child.into());
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self.row = self.row.metrics(m);
        self
    }

    /// Whether the arrow shows.
    fn has_arrow(&self) -> bool {
        self.expandable || !self.children.is_empty()
    }
    /// The message the arrow sends: the state the row would take.
    fn toggle(&self) -> Option<M> {
        self.on_toggle.as_ref().map(|f| f(!self.expanded))
    }
}

impl<'a, M: Clone + 'a> From<TreeRow<'a, M>> for Element<'a, M> {
    fn from(x: TreeRow<'a, M>) -> Self {
        let m = x.metrics;
        // The arrow cell; a leaf keeps its width (GTK's indent-for-icon).
        let cell = m.icon_size + 8.0;
        let handle: Element<'a, M> = if x.has_arrow() {
            let icon = if x.expanded {
                Icon::GoDown
            } else {
                Icon::GoNext
            };
            button(container(icon.view(m.icon_size - 2.0)).center(cell))
                .padding(0)
                .style(sb::flat)
                .on_press_maybe(x.toggle())
                .into()
        } else {
            space().width(cell).into()
        };
        let selected = x.selected;
        let header = row![
            container(handle).padding(iced::Padding::ZERO.left(m.row_padding.left)),
            Element::from(x.row),
        ]
        .align_y(Vertical::Center);
        let header: Element<'a, M> = container(header)
            .width(Length::Fill)
            .style(move |t: &Theme| iced::widget::container::Style {
                background: selected.then_some(Background::Color(crate::color::alpha(
                    t.semantic.window.fg,
                    t.semantic.opacity.flat_hover,
                ))),
                ..Default::default()
            })
            .into();
        if x.children.is_empty() {
            return header;
        }

        let indent = x.indent;
        let mut inner = column![];
        for c in x.children {
            // The hairline spans the list; the child itself is indented.
            let c: Element<'a, M> = if indent > 0.0 {
                container(c)
                    .width(Length::Fill)
                    .padding(iced::Padding::ZERO.left(indent))
                    .into()
            } else {
                c
            };
            inner = inner
                .push(
                    container(space().height(1))
                        .width(Length::Fill)
                        .style(sc::card_separator),
                )
                .push(c);
        }
        let list = container(inner).width(Length::Fill).style(|t: &Theme| {
            iced::widget::container::Style {
                background: Some(Background::Color(crate::color::alpha(
                    t.semantic.card.fg,
                    0.03,
                ))),
                ..Default::default()
            }
        });
        // The disclosure primitive: always mounted, the revealer slides it (250 ms).
        super::Collapsible::new(header, Element::from(list), x.expanded).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row() -> TreeRow<'static, u8> {
        TreeRow::new("A")
    }

    #[test]
    fn the_arrow_shows_with_children_or_when_expandable() {
        assert!(!row().has_arrow(), "a leaf has none");
        assert!(
            row().expandable(true).has_arrow(),
            "lazy: before its children exist"
        );
        assert!(row().push(iced::widget::space()).has_arrow());
    }

    #[test]
    fn the_arrow_sends_the_state_the_row_would_take() {
        let t = row().on_toggle(|open| open as u8);
        assert_eq!(t.toggle(), Some(1), "closed: it would open");
        assert_eq!(
            row().expanded(true).on_toggle(|open| open as u8).toggle(),
            Some(0)
        );
        assert_eq!(row().toggle(), None, "without on_toggle the arrow is inert");
    }

    #[test]
    fn the_indent_starts_at_the_toolkit_default_and_a_row_can_override_it() {
        assert_eq!(row().indent, Metrics::desktop().expander_indent);
        assert_eq!(row().indent(10.0).indent, 10.0);
        assert_eq!(row().indent(-3.0).indent, 0.0, "never negative");
    }

    #[test]
    fn children_can_be_any_element_including_other_tree_rows() {
        let inner: Element<'static, u8> = TreeRow::new("inner").push(iced::widget::space()).into();
        let t = row().expanded(true).push(inner).push(iced::widget::space());
        assert_eq!(t.children.len(), 2);
        let _: Element<'static, u8> = t.into();
    }
}
