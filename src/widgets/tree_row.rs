//! `TreeRow`: a row that can hold other rows, or any elements, nested under it. Inspired by
//! [`ExpanderRow`](crate::widgets::ExpanderRow), made for hierarchies: the **arrow** at the start opens
//! and closes it (GTK's `GtkTreeExpander`). The **whole header surface** activates the row
//! (select it, open it), or toggles it when no separate activation is set. A leaf keeps the
//! arrow's width so labels line up.
//!
//! **Scope.** A `TreeRow` nests elements *within a page*: a group of settings, an outline, a
//! file list, rows that belong together on one screen. It is **not for logic that spans pages**:
//! moving between pages, a sidebar of destinations or a navigation stack belong to
//! [`NavigationView`](super::NavigationView) and the sidebar components, which the HIG keeps flat.
//! If opening a row changes what page you are on, it is navigation, not a tree row.
//!
//! Children are anything: more `TreeRow`s, [`ActionRow`](crate::widgets::ActionRow)s, switch rows, or a
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

use crate::Element;
use crate::styles::button as sb;
use crate::styles::container as sc;
use crate::widgets::ActionRow;
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
    on_activate: Option<M>,
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
            on_activate: None,
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
    /// Pressing the header surface: select it, open it, run it.
    /// With a separate toggle action, the arrow toggles without activating.
    pub fn on_activate(mut self, m: M) -> Self {
        self.on_activate = Some(m);
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
    /// The state the row would take when toggled. Without `on_activate`, the whole
    /// header toggles; otherwise only the arrow does.
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
        let toggle = x.has_arrow().then(|| x.toggle()).flatten();
        let action = x.on_activate.clone().or_else(|| toggle.clone());
        let separate_toggle = x.on_activate.is_some();
        // The arrow cell; a leaf keeps its width (GTK's indent-for-icon).
        let cell = m.icon_size + 8.0;
        let handle: Element<'a, M> = if x.has_arrow() {
            let icon = if x.expanded {
                Icon::GoDown
            } else {
                Icon::GoNext
            };
            let glyph = container(icon.view(m.icon_size - 2.0)).center(cell);
            if separate_toggle {
                button(glyph)
                    .padding(0)
                    .style(sb::flat)
                    .on_press_maybe(toggle)
                    .into()
            } else {
                glyph.into()
            }
        } else {
            space().width(cell).into()
        };
        let selected = x.selected;
        let header = row![
            container(handle).padding(iced::Padding::ZERO.left(m.row_padding.left)),
            Element::from(x.row),
        ]
        .align_y(Vertical::Center);
        let header: Element<'a, M> = match action {
            Some(message) => button(header)
                .padding(0)
                .width(Length::Fill)
                .style(sb::list_row)
                .on_press(message)
                .into(),
            None => header.into(),
        };
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
        crate::widgets::Collapsible::new(header, Element::from(list), x.expanded).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn renderer() -> iced::Renderer {
        use iced::advanced::renderer::Headless;
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer")
    }

    fn click<M: Clone>(
        element: &mut Element<'_, M>,
        renderer: &iced::Renderer,
        at: iced::Point,
    ) -> Vec<M> {
        use iced::advanced::{Shell, clipboard, layout, widget::Tree};
        use iced::{Event, Rectangle, Size, mouse};
        let mut tree = Tree::new(&*element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(320.0, 1000.0)),
        );
        let mut messages = vec![];
        for event in [
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
        ] {
            element.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(event),
                layout::Layout::new(&node),
                mouse::Cursor::Available(at),
                renderer,
                &mut clipboard::Null,
                &mut Shell::new(&mut messages),
                &Rectangle::new(iced::Point::ORIGIN, Size::new(320.0, 1000.0)),
            );
        }
        messages
    }

    #[test]
    fn the_whole_header_toggles_or_activates_and_the_arrow_stays_independent() {
        let renderer = renderer();
        for activate in [false, true] {
            for at in [
                iced::Point::new(1.0, 1.0),
                iced::Point::new(140.0, 20.0),
                iced::Point::new(319.0, 20.0),
            ] {
                let mut row = TreeRow::new("Branch").expandable(true).on_toggle(|_| 1u8);
                if activate {
                    row = row.on_activate(2);
                }
                let mut element = Element::from(row);
                assert_eq!(
                    click(&mut element, &renderer, at),
                    vec![if activate { 2 } else { 1 }]
                );
            }
            let m = Metrics::desktop();
            let at = iced::Point::new(
                m.row_padding.left + (m.icon_size + 8.0) / 2.0,
                m.row_min_height / 2.0,
            );
            let mut row = TreeRow::new("Branch").expandable(true).on_toggle(|_| 1u8);
            if activate {
                row = row.on_activate(2);
            }
            assert_eq!(
                click(&mut row.into(), &renderer, at),
                vec![1],
                "arrow never also activates"
            );
        }
        let mut leaf = TreeRow::new("Leaf").on_activate(2u8).into();
        assert_eq!(
            click(&mut leaf, &renderer, iced::Point::new(319.0, 20.0)),
            vec![2]
        );
        let mut inert = TreeRow::<u8>::new("Inert").expandable(true).into();
        assert!(click(&mut inert, &renderer, iced::Point::new(319.0, 20.0)).is_empty());
    }

    #[test]
    fn an_interactive_suffix_keeps_its_own_action() {
        let renderer = renderer();
        let mut row = TreeRow::new("Branch")
            .expandable(true)
            .on_toggle(|_| 1u8)
            .on_activate(2)
            .suffix(
                button(space().width(40.0).height(24.0))
                    .padding(0)
                    .on_press(3),
            )
            .into();
        assert_eq!(
            click(&mut row, &renderer, iced::Point::new(288.0, 24.0)),
            vec![3]
        );
    }

    #[test]
    fn accordion_uses_full_surface_disclosure_and_keeps_its_selection_rule() {
        use crate::primitives::Selection;
        use crate::widgets::{Accordion, AccordionItem};
        let renderer = renderer();
        for (rule, expected) in [
            (Selection::Multiple, vec![2, 1]),
            (Selection::SINGLE, vec![1]),
        ] {
            let mut element = Accordion::new(&[2u8], |keys| keys)
                .selection(rule)
                .item(AccordionItem::new(1, "First", space()))
                .into();
            assert_eq!(
                click(&mut element, &renderer, iced::Point::new(319.0, 20.0)),
                vec![expected]
            );
        }
    }

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
