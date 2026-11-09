//! Text input components for building custom widgets. Application forms should use
//! [`EntryRow`](crate::widgets::EntryRow),
//! which share EntryRow framing and label/error presentation.

use crate::Element;
use crate::primitives::sections::{slot_width, text_padding};
use crate::styles;
use crate::{Metrics, Theme};
use iced::Length;
use iced::widget::{container, row, text_editor, text_input};

// ------------------------------------------------------------------ Entry

/// Validation state of an [`Entry`] (`.error` / `.warning` / `.success`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntryState {
    #[default]
    Normal,
    Error,
    Warning,
    Success,
}

impl From<crate::primitives::validation::Presentation> for EntryState {
    /// The shared validation presentation, as an entry's own ring.
    fn from(p: crate::primitives::validation::Presentation) -> Self {
        use crate::primitives::validation::Presentation as P;
        match p {
            P::Pristine => Self::Normal,
            // Nothing special for `:user-valid`; use `.state(EntryState::Success)` to opt in.
            P::Valid => Self::Normal,
            P::Warning => Self::Warning,
            P::Invalid => Self::Error,
        }
    }
}

/// `GtkEntry` / `GtkPasswordEntry`.
///
/// Without [`on_input`](Entry::on_input) it is read-only and drawn disabled.
pub struct Entry<'a, M> {
    placeholder: String,
    value: String,
    on_input: Option<Box<dyn Fn(String) -> M + 'a>>,
    on_submit: Option<M>,
    state: EntryState,
    color: Option<iced::Color>,
    secure: bool,
    embedded: bool,
    flat: bool,
    width: Length,
    metrics: Metrics,
    left: Option<Element<'a, M>>,
    right: Option<Element<'a, M>>,
    left_width: Option<f32>,
    right_width: Option<f32>,
    section_height: Option<f32>,
    id: Option<iced::widget::Id>,
}

/// `GtkEntry`.
pub fn entry<'a, M>(placeholder: impl Into<String>, value: impl Into<String>) -> Entry<'a, M> {
    Entry::new(placeholder, value)
}

/// `GtkPasswordEntry`: hides its contents.
pub fn password_entry<'a, M>(
    placeholder: impl Into<String>,
    value: impl Into<String>,
) -> Entry<'a, M> {
    Entry::new(placeholder, value).password(true)
}

impl<'a, M> Entry<'a, M> {
    pub fn new(placeholder: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            value: value.into(),
            on_input: None,
            on_submit: None,
            state: EntryState::Normal,
            color: None,
            secure: false,
            embedded: false,
            flat: false,
            width: Length::Fill,
            metrics: Metrics::desktop(),
            left: None,
            right: None,
            left_width: None,
            right_width: None,
            section_height: None,
            id: None,
        }
    }
    /// Transparent page-surface style, like a flat navigation row.
    pub fn flat(mut self, flat: bool) -> Self {
        self.flat = flat;
        self
    }
    /// Embed in an EntryRow, which supplies the surface and outer padding.
    pub(crate) fn embedded(mut self) -> Self {
        self.embedded = true;
        self
    }
    /// Align the EntryRow label with the editable text after its optional sections.
    pub(crate) fn label_insets(&self, metrics: &Metrics) -> iced::Padding {
        let size = crate::components::section_size(metrics);
        text_padding(
            iced::Padding::ZERO,
            0.0,
            0.0,
            self.left
                .as_ref()
                .map(|_| slot_width(self.left_width, size)),
            self.right
                .as_ref()
                .map(|_| slot_width(self.right_width, size)),
        )
    }
    pub(crate) fn section_height(mut self, height: f32) -> Self {
        self.section_height = Some(height);
        self
    }
    pub fn on_input(mut self, f: impl Fn(String) -> M + 'a) -> Self {
        self.on_input = Some(Box::new(f));
        self
    }
    /// `activate` (Enter).
    pub fn on_submit(mut self, m: M) -> Self {
        self.on_submit = Some(m);
        self
    }
    /// `.error` / `.warning` / `.success`.
    pub fn state(mut self, s: EntryState) -> Self {
        self.state = s;
        self
    }
    /// The shared validation presentation (see [`Field`](crate::widgets::Field)); same as
    /// [`state`](Entry::state) with the matching ring.
    pub fn presentation(self, p: crate::primitives::validation::Presentation) -> Self {
        self.state(p.into())
    }
    /// Overrides the entry's color: its text and its focus/validation ring use `c`. For an
    /// app-specific tint; prefer [`state`](Entry::state) for validation.
    pub fn color(mut self, c: iced::Color) -> Self {
        self.color = Some(c);
        self
    }
    /// `visibility = false`.
    pub fn password(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    /// The widget id: what a focus scope's `initial_focus` names.
    pub fn id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
    /// An element inside the entry's border, before the text (Mantine's `leftSection`): an
    /// icon, spinner, unit label. See [`section_icon`](crate::components::section_icon). It is centered
    /// vertically in a slot of [`left_section_width`](Entry::left_section_width); the text
    /// starts after the slot.
    pub fn left_section(mut self, s: impl Into<Element<'a, M>>) -> Self {
        self.left = Some(s.into());
        self
    }
    /// An element inside the entry's border, after the text (`rightSection`): a clear or
    /// filter button ([`section_button`](crate::components::section_button)), a badge. The element stays
    /// interactive when the entry is read-only.
    pub fn right_section(mut self, s: impl Into<Element<'a, M>>) -> Self {
        self.right = Some(s.into());
        self
    }
    /// Slot width of the left section in px; default [`section_size`](crate::components::section_size)
    /// (an icon or round button). iced cannot measure a section before laying out the text, so
    /// a wider section (a "kg" label) states its width.
    pub fn left_section_width(mut self, px: f32) -> Self {
        self.left_width = Some(px);
        self
    }
    /// Slot width of the right section in px; see [`left_section_width`](Entry::left_section_width).
    pub fn right_section_width(mut self, px: f32) -> Self {
        self.right_width = Some(px);
        self
    }
}

impl<'a, M: Clone + 'a> From<Entry<'a, M>> for Element<'a, M> {
    fn from(e: Entry<'a, M>) -> Self {
        let m = e.metrics;
        let default_w = crate::components::section_size(&m);
        let lw = e.left.as_ref().map(|_| slot_width(e.left_width, default_w));
        let rw = e
            .right
            .as_ref()
            .map(|_| slot_width(e.right_width, default_w));
        let mut padding = text_padding(
            if e.embedded {
                iced::Padding::ZERO
            } else {
                m.entry_padding
            },
            if e.embedded { 0.0 } else { m.space_xs },
            if e.embedded { 0.0 } else { m.space_xs },
            lw,
            rw,
        );
        let text_size = crate::Typography::default().body;
        if lw.is_some() || rw.is_some() {
            let inset = ((e.section_height.unwrap_or(default_w).max(default_w) - text_size * 1.3)
                / 2.0)
                .max(0.0);
            padding.top = padding.top.max(inset);
            padding.bottom = padding.bottom.max(inset);
        }
        let mut input = text_input(&e.placeholder, &e.value)
            .size(text_size)
            .line_height(iced::widget::text::LineHeight::Relative(1.3))
            .padding(padding)
            .secure(e.secure)
            .width(e.width);
        input = match e.state {
            EntryState::Normal => input.style(styles::text_input::default),
            EntryState::Error => input.style(styles::text_input::error),
            EntryState::Warning => input.style(styles::text_input::warning),
            EntryState::Success => input.style(styles::text_input::success),
        };
        if e.embedded {
            input = input.style(styles::text_input::embedded);
        }
        if e.flat && !e.embedded {
            let state = e.state;
            input = input.style(move |theme, status| {
                let mut style = styles::text_input::flat(theme, status);
                let color = match state {
                    EntryState::Error => Some(theme.semantic.error.color),
                    EntryState::Warning => Some(theme.semantic.warning.color),
                    EntryState::Success => Some(theme.semantic.success.color),
                    EntryState::Normal => None,
                };
                if let Some(color) = color {
                    style.border.color = color;
                    style.border.width = theme.metrics.focus_ring_width;
                }
                style
            });
        }
        if let Some(c) = e.color {
            let (embedded, flat) = (e.embedded, e.flat);
            input = input.style(move |t: &Theme, status| {
                let mut style = if embedded {
                    styles::text_input::embedded(t, status)
                } else if flat {
                    styles::text_input::flat(t, status)
                } else {
                    styles::text_input::default(t, status)
                };
                style.border.color = c;
                style.border.width = if embedded {
                    0.0
                } else {
                    t.metrics.focus_ring_width
                };
                style.value = c;
                style
            });
        }
        if let Some(id) = e.id {
            input = input.id(id);
        }
        if let Some(f) = e.on_input {
            input = input.on_input(f);
        }
        if let Some(m) = e.on_submit {
            input = input.on_submit(m);
        }
        let content: Element<'a, M> = input.into();
        if lw.is_none() && rw.is_none() {
            return content;
        }
        // The sections sit in a transparent layer over the input, inside its border and ring;
        // the input's padding keeps the text clear of them.
        let slot = |el: Option<Element<'a, M>>, w: Option<f32>| -> Element<'a, M> {
            match (el, w) {
                (Some(el), Some(w)) => container(el)
                    .center_y(Length::Fill)
                    .width(Length::Fixed(w))
                    .align_x(iced::alignment::Horizontal::Center)
                    .into(),
                _ => iced::widget::space().into(),
            }
        };
        let layer = row![
            slot(e.left, lw),
            iced::widget::space().width(Length::Fill),
            slot(e.right, rw)
        ]
        .padding([0.0, if e.embedded { 0.0 } else { m.space_xs }])
        .height(Length::Fill)
        .width(Length::Fill);
        iced::widget::stack![content, layer].width(e.width).into()
    }
}

// --------------------------------------------------------------- TextView

/// The text size of a [`TextView`] and the line height that goes with it.
const TEXT_SIZE: f32 = 16.0;
const LINE_HEIGHT: f32 = TEXT_SIZE * 1.3;

/// `GtkTextView`: a multi-line editor over a [`text_editor::Content`]. Without
/// [`on_action`](TextView::on_action) it is read-only and drawn disabled.
///
/// **Fixed or growing.** By default it is a fixed [`height`](TextView::height) and scrolls. Give it
/// a maximum, [`max_lines`](TextView::max_lines) or [`max_height`](TextView::max_height), and it
/// becomes a textarea that **grows with its text** from [`min_lines`](TextView::min_lines)
/// (default one line) up to that maximum, then scrolls. Without a maximum it never grows: an unbounded textarea
/// would push the rest of the page away without limit.
pub struct TextView<'a, M> {
    content: &'a text_editor::Content,
    on_action: Option<Box<dyn Fn(text_editor::Action) -> M + 'a>>,
    placeholder: Option<String>,
    height: Length,
    min_lines: usize,
    max_lines: Option<usize>,
    max_height: Option<f32>,
    single_line: bool,
    embedded: bool,
    flat: bool,
    metrics: Metrics,
}

pub fn text_view<'a, M>(content: &'a text_editor::Content) -> TextView<'a, M> {
    TextView {
        content,
        on_action: None,
        placeholder: None,
        height: Length::Fixed(90.0),
        min_lines: 1,
        max_lines: None,
        max_height: None,
        single_line: false,
        embedded: false,
        flat: false,
        metrics: Metrics::desktop(),
    }
}

impl<'a, M> TextView<'a, M> {
    /// Transparent editor surface with normal focus and validation behavior.
    pub fn flat(mut self, flat: bool) -> Self {
        self.flat = flat;
        self
    }
    pub(crate) fn embedded(mut self, metrics: Metrics) -> Self {
        self.embedded = true;
        self.metrics = metrics;
        self.metrics.entry_padding = iced::Padding::ZERO;
        self
    }
    pub fn on_action(mut self, f: impl Fn(text_editor::Action) -> M + 'a) -> Self {
        self.on_action = Some(Box::new(f));
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = Some(p.into());
        self
    }
    /// Keep input to a single line: ignore Enter and normalize pasted line breaks to spaces.
    pub fn single_line(mut self, enabled: bool) -> Self {
        self.single_line = enabled;
        if enabled {
            self.max_lines = Some(1);
            self.min_lines = 1;
        }
        self
    }
    /// The fixed height of a view that does not grow (default 90). Ignored once a maximum is set.
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }
    /// The fewest lines a growing view shows, however short its text (default 1). Only applies
    /// with a maximum.
    pub fn min_lines(mut self, lines: usize) -> Self {
        self.min_lines = lines.max(1);
        self
    }
    /// Makes the view grow with its text up to this many lines, then scroll.
    pub fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines.max(1));
        self
    }
    /// Makes the view grow with its text up to this height in px, then scroll. With
    /// [`max_lines`](Self::max_lines) too, the smaller of the two applies.
    pub fn max_height(mut self, px: f32) -> Self {
        self.max_height = Some(px.max(1.0));
        self
    }

    /// The `(min, max)` height in px of a growing view, or `None` for a fixed one.
    fn growth(&self) -> Option<(f32, f32)> {
        if self.max_lines.is_none() && self.max_height.is_none() {
            return None;
        }
        let pad = self.metrics.entry_padding.y();
        let lines_px = |n: usize| n as f32 * LINE_HEIGHT + pad;
        let min = lines_px(self.min_lines);
        let max = [self.max_lines.map(lines_px), self.max_height]
            .into_iter()
            .flatten()
            .fold(f32::INFINITY, f32::min);
        Some((min, max.max(min)))
    }
}

impl<'a, M: Clone + 'a> From<TextView<'a, M>> for Element<'a, M> {
    fn from(t: TextView<'a, M>) -> Self {
        let growth = t.growth();
        let mut e = text_editor(t.content)
            .size(TEXT_SIZE)
            .padding(t.metrics.entry_padding)
            .style(styles::text_editor::default);
        if t.embedded || t.flat {
            let embedded = t.embedded;
            e = e.style(move |theme, status| {
                let mut style = styles::text_editor::default(theme, status);
                style.background = iced::Color::TRANSPARENT.into();
                if embedded {
                    style.border = iced::Border::default();
                }
                style
            });
        }
        e = match growth {
            // Fits its text, between the two bounds.
            Some((min, max)) => e.height(Length::Shrink).min_height(min).max_height(max),
            None => e.height(t.height),
        };
        if let Some(p) = t.placeholder {
            e = e.placeholder(p);
        }
        if let Some(f) = t.on_action {
            let single_line = t.single_line;
            e = e.on_action(move |action| {
                use iced::widget::text_editor::{Action, Edit};
                let action = if single_line {
                    match action {
                        Action::Edit(Edit::Enter | Edit::Insert('\n' | '\r')) => {
                            Action::Scroll { lines: 0 }
                        }
                        Action::Edit(Edit::Paste(text)) => {
                            Action::Edit(Edit::Paste(std::sync::Arc::new(
                                text.replace("\r\n", " ").replace(['\n', '\r'], " "),
                            )))
                        }
                        other => other,
                    }
                } else {
                    action
                };
                f(action)
            });
        }
        e.into()
    }
}

#[cfg(test)]
mod valid_tests {
    use super::*;
    use crate::primitives::validation::Presentation as P;

    #[test]
    fn only_invalid_and_warning_get_a_ring_by_default() {
        assert_eq!(EntryState::from(P::Pristine), EntryState::Normal);
        assert_eq!(EntryState::from(P::Valid), EntryState::Normal);
        assert_eq!(EntryState::from(P::Warning), EntryState::Warning);
        assert_eq!(EntryState::from(P::Invalid), EntryState::Error);
    }
}

#[cfg(test)]
mod text_view_tests {
    use super::*;

    fn view(content: &text_editor::Content) -> TextView<'_, ()> {
        text_view(content)
    }

    #[test]
    fn without_a_maximum_it_does_not_grow() {
        let c = text_editor::Content::new();
        assert_eq!(view(&c).min_lines(3).growth(), None);
    }

    #[test]
    fn a_maximum_in_lines_or_pixels_makes_it_grow_between_bounds() {
        let c = text_editor::Content::new();
        let pad = Metrics::desktop().entry_padding.y();
        let (min, max) = view(&c).min_lines(2).max_lines(6).growth().unwrap();
        assert_eq!(min, 2.0 * LINE_HEIGHT + pad);
        assert_eq!(max, 6.0 * LINE_HEIGHT + pad);
        // In pixels.
        assert_eq!(view(&c).max_height(120.0).growth().unwrap().1, 120.0);
        // Both: the smaller applies.
        assert_eq!(
            view(&c).max_lines(10).max_height(100.0).growth().unwrap().1,
            100.0
        );
        // The maximum is never below the minimum.
        let (min, max) = view(&c).min_lines(5).max_height(10.0).growth().unwrap();
        assert_eq!(max, min);
    }
}

#[cfg(test)]
mod section_alignment_tests {
    use super::*;
    use iced::Size;
    use iced::advanced::{layout, renderer::Headless, widget::Tree};
    #[test]
    fn sections_and_text_share_a_center_and_fit_without_overflow() {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        for metrics in [Metrics::desktop(), Metrics::compact()] {
            for embedded in [false, true] {
                let mut input = entry("", "Centered")
                    .on_input(|_| ())
                    .left_section(crate::EntryIcon::Search.view(16.0))
                    .right_section(
                        iced::widget::column![
                            crate::EntryIcon::Increment.view(12.0),
                            crate::EntryIcon::Decrement.view(12.0)
                        ]
                        .spacing(4),
                    )
                    .section_height(28.0)
                    .metrics(metrics);
                if embedded {
                    input = input.embedded();
                }
                let mut element: Element<'_, ()> = input.into();
                let mut tree = Tree::new(&element);
                let node = element.as_widget_mut().layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(300.0, 200.0)),
                );
                let root = layout::Layout::new(&node);
                let mut layers = root.children();
                let control = layers.next().unwrap();
                let text = control.children().next().unwrap().bounds();
                let layer = layers.next().unwrap();
                let slots: Vec<_> = layer.children().collect();
                for slot in [slots[0], slots[2]] {
                    let content = slot.children().next().unwrap().bounds();
                    assert!(
                        (content.center_y() - text.center_y()).abs() < 0.01,
                        "{content:?} vs {text:?}"
                    );
                    assert!(
                        content.y >= root.bounds().y
                            && content.y + content.height <= root.bounds().height + 0.01
                    );
                }
            }
        }
    }
}
