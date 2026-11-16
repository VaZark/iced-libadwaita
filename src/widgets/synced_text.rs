//! `SyncedText`: a scrolling list of timed lines with the current one highlighted (lyrics,
//! subtitles, a transcript). Pressing a line asks to seek to its time.
//!
//! The application owns the playback position. The component is controlled: give it the lines
//! and the position, and call [`follow_synced_text`] whenever the current line changes so the
//! list scrolls with it.
//!
//! ```ignore
//! SyncedText::new(LYRICS, &self.lines, Msg::Seek).position(self.pos)
//!
//! // in update, when the position moves to another line:
//! follow_synced_text(LYRICS, synced::current(&self.lines, self.pos), self.lines.len())
//! ```

use crate::Element;
use crate::components::typography as ty;
use crate::primitives::synced::{self, Line};
use crate::styles::text as st;
use iced::widget::{Id, column, container};
use iced::{Alignment, Length, Task};

/// See the [module](self).
pub struct SyncedText<'a, M> {
    props: SyncedTextWidgetProps,
    id: Id,
    lines: &'a [Line],
    current: Option<usize>,
    on_seek: Box<dyn Fn(f32) -> M + 'a>,
    placeholder: String,
    spacing: Option<f32>,
    padding: Option<iced::Padding>,
    line_padding: Option<iced::Padding>,
    align: Alignment,
}

impl<'a, M> SyncedText<'a, M> {
    /// `id` names the scrollable (for [`follow_synced_text`]); `on_seek` gets the time of the
    /// pressed line.
    /// Presentation overrides for native children; actions remain parent-owned.
    pub fn props(mut self, props: SyncedTextWidgetProps) -> Self {
        self.props = props;
        self
    }
    pub fn new(id: impl Into<Id>, lines: &'a [Line], on_seek: impl Fn(f32) -> M + 'a) -> Self {
        Self {
            props: SyncedTextWidgetProps::default(),
            id: id.into(),
            lines,
            current: None,
            on_seek: Box::new(on_seek),
            placeholder: "Nothing to show.".into(),
            spacing: None,
            padding: None,
            line_padding: None,
            align: Alignment::Start,
        }
    }
    /// The highlighted line.
    pub fn current(mut self, current: Option<usize>) -> Self {
        self.current = current;
        self
    }
    /// The playback position in seconds: highlights the line active at it.
    pub fn position(self, pos: f32) -> Self {
        let current = synced::current(self.lines, pos);
        self.current(current)
    }
    /// The gap between lines (default: the theme's `space_xxs`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// The padding around the lines (default: the theme's `empty_padding_y` by `space_m`).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    /// The padding inside each line's button (default: the theme's `space_xs` by `gap`).
    pub fn line_padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.line_padding = Some(p.into());
        self
    }
    /// Where the lines sit across the column (default: [`Alignment::Start`]).
    pub fn align(mut self, a: Alignment) -> Self {
        self.align = a;
        self
    }
    /// What an empty list says.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }
}

/// The task that scrolls the [`SyncedText`] named `id` to line `current` of `total`. Run it when
/// the current line changes.
pub fn follow_synced_text<T: Send + 'static>(
    id: impl Into<Id>,
    current: Option<usize>,
    total: usize,
) -> Task<T> {
    iced::widget::operation::snap_to(
        id,
        iced::widget::scrollable::RelativeOffset {
            x: 0.0,
            y: synced::scroll_to(current, total),
        },
    )
}

impl<'a, M: Clone + 'a> From<SyncedText<'a, M>> for Element<'a, M> {
    fn from(s: SyncedText<'a, M>) -> Self {
        if s.lines.is_empty() {
            return s
                .props
                .empty_container
                .apply(
                    container(
                        s.props
                            .placeholder
                            .apply(ty::body(s.placeholder).style(st::dim)),
                    )
                    .center(Length::Fill),
                )
                .into();
        }
        let current = s.current;
        let on_seek = s.on_seek;
        let m = crate::Metrics::desktop();
        let line_padding = s
            .line_padding
            .unwrap_or(iced::Padding::from([m.space_xs, m.gap]));
        let rows = s.lines.iter().enumerate().map(|(i, l)| {
            let t = if Some(i) == current {
                s.props.active_line.apply(ty::title_3(l.text.as_str()))
            } else {
                s.props
                    .line
                    .apply(ty::body(l.text.as_str()).size(18).style(st::dim))
            };
            s.props
                .line_button
                .apply(
                    crate::components::flat_button(
                        s.props
                            .line_container
                            .apply(container(t).padding(line_padding)),
                    )
                    .on_press(on_seek(l.at)),
                )
                .into()
        });
        crate::layouts::scrolled_window(
            column(rows)
                .spacing(s.spacing.unwrap_or(m.space_xxs))
                .padding(
                    s.padding
                        .unwrap_or(iced::Padding::from([m.empty_padding_y, m.space_m])),
                )
                .align_x(s.align)
                .width(Length::Fill),
        )
        .id(s.id)
        .height(Length::Fill)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gaps_and_padding_are_parameters() {
        let lines = [Line::new(0.0, "a"), Line::new(4.0, "b")];
        let s = SyncedText::new("lyrics", &lines, |_| ())
            .spacing(1.0)
            .padding(2.0)
            .line_padding([3.0, 4.0])
            .align(Alignment::Center);
        assert_eq!(s.spacing, Some(1.0));
        let _: Element<'_, ()> = s.into();
    }

    #[test]
    fn the_position_picks_the_current_line() {
        let lines = [Line::new(0.0, "a"), Line::new(4.0, "b")];
        let s = SyncedText::new("lyrics", &lines, |_| ()).position(5.0);
        assert_eq!(s.current, Some(1));
    }
}

/// Typed presentation for parent-built native children. Defaults preserve the original view.
#[derive(Clone, Default)]
pub struct SyncedTextWidgetProps {
    pub empty_container: crate::components::ContainerProps,
    pub placeholder: crate::components::TextProps,
    pub active_line: crate::components::TextProps,
    pub line: crate::components::TextProps,
    pub line_button: crate::components::ButtonProps,
    pub line_container: crate::components::ContainerProps,
}

impl SyncedTextWidgetProps {
    pub fn empty_container(mut self, props: crate::components::ContainerProps) -> Self {
        self.empty_container = props;
        self
    }
    pub fn placeholder(mut self, props: crate::components::TextProps) -> Self {
        self.placeholder = props;
        self
    }
    pub fn active_line(mut self, props: crate::components::TextProps) -> Self {
        self.active_line = props;
        self
    }
    pub fn line(mut self, props: crate::components::TextProps) -> Self {
        self.line = props;
        self
    }
    pub fn line_button(mut self, props: crate::components::ButtonProps) -> Self {
        self.line_button = props;
        self
    }
    pub fn line_container(mut self, props: crate::components::ContainerProps) -> Self {
        self.line_container = props;
        self
    }
    pub fn apply<'a, M>(&self, widget: SyncedText<'a, M>) -> SyncedText<'a, M> {
        let mut widget = widget;
        widget.props.merge(self);
        widget
    }
}

#[cfg(test)]
pub(crate) mod native_props_tests {
    pub(crate) use super::native_click_support::click_sized;
    use super::*;
    use iced::advanced::{layout, renderer::Headless, widget::Tree};

    pub(crate) fn renderer() -> iced::Renderer {
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap()
    }

    pub(crate) fn layout<M>(
        element: &mut Element<'_, M>,
        renderer: &iced::Renderer,
    ) -> (Tree, layout::Node) {
        let mut tree = Tree::new(&*element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            renderer,
            &layout::Limits::new(iced::Size::ZERO, iced::Size::new(800.0, 600.0)),
        );
        (tree, node)
    }

    pub(crate) fn sizes(node: &layout::Node) -> Vec<iced::Size> {
        let mut all = vec![node.size()];
        for child in node.children() {
            all.extend(sizes(child));
        }
        all
    }

    pub(crate) fn text_props() -> crate::components::TextProps {
        crate::components::TextProps::default()
            .width(137)
            .height(29)
    }

    pub(crate) fn assert_override(
        base: Element<'_, ()>,
        default: Element<'_, ()>,
        changed: Element<'_, ()>,
    ) {
        let renderer = renderer();
        let (mut base, mut default, mut changed) = (base, default, changed);
        let (_, a) = layout(&mut base, &renderer);
        let (_, b) = layout(&mut default, &renderer);
        let (_, c) = layout(&mut changed, &renderer);
        assert_eq!(
            sizes(&a),
            sizes(&b),
            "default props change no native layout nodes"
        );
        assert!(
            sizes(&c).contains(&iced::Size::new(137.0, 29.0)),
            "override must reach a rendered native child: {:?}",
            sizes(&c)
        );
        assert_ne!(sizes(&a), sizes(&c));
    }

    #[test]
    fn synced_labels_override_native_children_without_changing_current_line() {
        let lines = [Line::new(0.0, "日本語 🦀"), Line::new(4.0, "second")];
        let make = || SyncedText::new("native-lines", &lines, |_| ()).position(5.0);
        let changed = make().props(SyncedTextWidgetProps::default().active_line(text_props()));
        assert_eq!(changed.current, Some(1));
        assert_override(
            make().into(),
            make().props(SyncedTextWidgetProps::default()).into(),
            changed.into(),
        );
        let empty = || SyncedText::new("empty-lines", &[], |_| ());
        assert_override(
            empty().into(),
            empty().props(SyncedTextWidgetProps::default()).into(),
            empty()
                .props(SyncedTextWidgetProps::default().placeholder(text_props()))
                .into(),
        );
    }
}

#[cfg(test)]
mod native_click_support {
    use super::*;
    use iced::advanced::{Clipboard, Shell, clipboard, layout::Layout};
    use iced::{Event, Point, Rectangle, Size, mouse};

    pub(crate) fn click_sized<M>(mut element: Element<'_, M>) -> Vec<M> {
        let renderer = super::native_props_tests::renderer();
        let (mut tree, node) = super::native_props_tests::layout(&mut element, &renderer);
        fn target(layout: Layout<'_>) -> Option<Rectangle> {
            if layout.bounds().size() == Size::new(137.0, 29.0) {
                return Some(layout.bounds());
            }
            layout.children().find_map(target)
        }
        let layout = Layout::new(&node);
        let bounds = target(layout).expect("native size override must reach the button");
        let at = bounds.center();
        let mut messages = Vec::new();
        let mut clipboard: Box<dyn Clipboard> = Box::new(clipboard::Null);
        for event in [
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
        ] {
            element.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(event),
                layout,
                mouse::Cursor::Available(at),
                &renderer,
                clipboard.as_mut(),
                &mut Shell::new(&mut messages),
                &Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0)),
            );
        }
        messages
    }
}

crate::components::props::impl_merge!(SyncedTextWidgetProps; fields: []; optional: []; nested: [empty_container, placeholder, active_line, line, line_button, line_container]);
