//! Internal text overflow layout support for typed `TextProps`.

use crate::{Element, Theme};
use iced::widget::{Text, container};
use iced::{Length, Size};

/// Opt-in text layout policy. An absent policy preserves the existing wrapping.
/// No policy truncates the source string, including long Unicode labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOverflow {
    /// Wrap at words, falling back to glyphs for long unbroken labels.
    Wrap,
    /// Disable wrapping and clip drawing to the allocated bounds.
    Clip,
    /// Disable wrapping and allow intrinsic horizontal layout beyond parent limits.
    /// Ancestor clipping (for example a scrollable viewport) still applies.
    Grow,
}

/// Add only the layout/drawing adapter. Line wrapping is set by `TextProps`.
/// Accept text, not arbitrary elements: the grow adapter intentionally handles a leaf.
pub(super) fn apply_overflow<'a, M: 'a>(
    text: Text<'a, Theme>,
    policy: Option<TextOverflow>,
) -> Element<'a, M> {
    match policy {
        Some(TextOverflow::Clip) => container(text).clip(true).into(),
        Some(TextOverflow::Grow) => Element::new(HorizontalGrow(text.into())),
        None | Some(TextOverflow::Wrap) => text.into(),
    }
}

// Text is a leaf: no event or overlay forwarding is necessary.
struct HorizontalGrow<'a, M>(Element<'a, M>);

fn grow_limits(limits: &iced::advanced::layout::Limits) -> iced::advanced::layout::Limits {
    iced::advanced::layout::Limits::new(
        Size::new(limits.min().width, limits.min().height),
        Size::new(f32::INFINITY, limits.max().height),
    )
}

impl<M> iced::advanced::Widget<M, Theme, iced::Renderer> for HorizontalGrow<'_, M> {
    fn size(&self) -> Size<Length> {
        self.0.as_widget().size()
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.0)]
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.0));
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let child =
            self.0
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &grow_limits(limits));
        iced::advanced::layout::Node::with_children(child.size(), vec![child])
    }
    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.0.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::{layout::Limits, widget::Tree};
    use iced::widget::text;

    #[test]
    fn absent_policy_preserves_leaf_geometry_without_wrapper() {
        let result: Element<'_, ()> = apply_overflow(text("日本語 🦀").width(123).height(45), None);
        assert!(Tree::new(&result).children.is_empty());
        assert_eq!(
            result.as_widget().size(),
            Size::new(Length::Fixed(123.0), Length::Fixed(45.0))
        );
    }

    #[test]
    fn grow_releases_only_horizontal_maximum() {
        let limits = Limits::new(Size::new(10.0, 20.0), Size::new(80.0, 40.0));
        let grown = grow_limits(&limits);
        assert_eq!(grown.min(), limits.min());
        assert_eq!(grown.max(), Size::new(f32::INFINITY, 40.0));
    }
}
