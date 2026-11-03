//! Form and display components: entries, text view, check/radio buttons, scale, link and
//! card buttons, tags and color swatches.
//!
//! Each is a thin, themed wrapper with libadwaita-style properties, so a layout (code, XML or
//! JSX-like) can use one vocabulary instead of raw iced widgets.

use super::{ColorInput, Element};
use crate::primitives::color_picker::{Format as ColorFormat, Hsva};
use crate::primitives::sections::{slot_width, text_padding};
use crate::style;
use crate::{AccentColor, Icon, Metrics, Role, Surface, Swatch, Theme};
use iced::widget::{Button, Checkbox, Radio, Slider};
use iced::widget::{
    button, checkbox, container, radio, row, slider, text, text_editor, text_input,
};
use iced::{Length, Padding};
use std::ops::RangeInclusive;

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
    width: Length,
    metrics: Metrics,
    left: Option<Element<'a, M>>,
    right: Option<Element<'a, M>>,
    left_width: Option<f32>,
    right_width: Option<f32>,
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
            width: Length::Fill,
            metrics: Metrics::desktop(),
            left: None,
            right: None,
            left_width: None,
            right_width: None,
            id: None,
        }
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
    /// The shared validation presentation (see [`Field`](super::Field)); same as
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
    /// icon, spinner, unit label. See [`section_icon`](super::section_icon). It is centered
    /// vertically in a slot of [`left_section_width`](Entry::left_section_width); the text
    /// starts after the slot.
    pub fn left_section(mut self, s: impl Into<Element<'a, M>>) -> Self {
        self.left = Some(s.into());
        self
    }
    /// An element inside the entry's border, after the text (`rightSection`): a clear or
    /// filter button ([`section_button`](super::section_button)), a badge. The element stays
    /// interactive when the entry is read-only.
    pub fn right_section(mut self, s: impl Into<Element<'a, M>>) -> Self {
        self.right = Some(s.into());
        self
    }
    /// Slot width of the left section in px; default [`section_size`](super::section_size)
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
        let default_w = super::section_size(&m);
        let lw = e.left.as_ref().map(|_| slot_width(e.left_width, default_w));
        let rw = e
            .right
            .as_ref()
            .map(|_| slot_width(e.right_width, default_w));
        let padding = text_padding(m.entry_padding, m.space_xs, m.space_xs, lw, rw);
        let mut input = text_input(&e.placeholder, &e.value)
            .padding(padding)
            .secure(e.secure)
            .width(e.width);
        input = match e.state {
            EntryState::Normal => input.style(style::text_input::default),
            EntryState::Error => input.style(style::text_input::error),
            EntryState::Warning => input.style(style::text_input::warning),
            EntryState::Success => input.style(style::text_input::success),
        };
        if let Some(c) = e.color {
            input = input.style(move |t: &Theme, status| {
                let mut st = style::text_input::default(t, status);
                st.border.color = c;
                st.border.width = t.metrics.focus_ring_width;
                st.value = c;
                st
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
        if lw.is_none() && rw.is_none() {
            return input.into();
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
        .padding([0.0, m.space_xs])
        .height(Length::Fill)
        .width(Length::Fill);
        iced::widget::stack![input, layer].width(e.width).into()
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
        metrics: Metrics::desktop(),
    }
}

impl<'a, M> TextView<'a, M> {
    pub fn on_action(mut self, f: impl Fn(text_editor::Action) -> M + 'a) -> Self {
        self.on_action = Some(Box::new(f));
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = Some(p.into());
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
            .style(style::text_editor::default);
        e = match growth {
            // Fits its text, between the two bounds.
            Some((min, max)) => e.height(Length::Shrink).min_height(min).max_height(max),
            None => e.height(t.height),
        };
        if let Some(p) = t.placeholder {
            e = e.placeholder(p);
        }
        if let Some(f) = t.on_action {
            e = e.on_action(f);
        }
        e.into()
    }
}

// ------------------------------------------------- CheckButton / RadioButton

/// `GtkCheckButton`: chain `.on_toggle(..)`; without it the button is disabled.
pub fn check_button<'a, M>(label: impl Into<String>, active: bool) -> Checkbox<'a, M, Theme> {
    checkbox(active).label(label.into())
}

/// `GtkCheckButton` in a group (radio): `value` is this button's value, `selected` the
/// group's current one.
pub fn radio_button<'a, V, M>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_select: impl FnOnce(V) -> M,
) -> Radio<'a, M, Theme>
where
    V: Copy + Eq,
    M: Clone,
{
    radio(label.into(), value, selected, on_select)
}

// ------------------------------------------------------------------ Scale

/// `GtkScale`: a horizontal slider filling the available width.
pub fn scale<'a, T, M>(
    range: RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> M + 'a,
) -> Slider<'a, T, M, Theme>
where
    T: Copy + From<u8> + PartialOrd + 'a,
    f64: From<T>,
    M: Clone + 'a,
{
    slider(range, value, on_change).width(Length::Fill)
}

// --------------------------------------------------- LinkButton / card button

/// `GtkLinkButton`: accent-colored text button (`.link`).
pub fn link_button<'a, M: 'a>(label: impl Into<String>) -> Button<'a, M, Theme> {
    button(super::typography::label(label.into()))
        .padding(Metrics::desktop().space_xs)
        .style(style::button::link)
}

/// A clickable `.card`: any content on a card surface that reacts to hover and press.
pub fn card_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    button(content)
        .padding(Metrics::desktop().card_padding)
        .style(style::button::card)
}

// -------------------------------------------------------------------- Tag

/// A small rounded label tinted with a [`Role`] (status chip). Not a libadwaita widget; the
/// pattern behind `.tag`-style badges in GNOME apps.
pub fn tag<'a, M: 'a>(label: impl Into<String>, role: Role) -> Element<'a, M> {
    container(
        text(label.into())
            .size(Metrics::desktop().space_m + 1.0)
            .wrapping(text::Wrapping::None),
    )
    .padding(Padding::from([2.0, 10.0]))
    .style(style::container::tinted(role))
    .into()
}

// ----------------------------------------------------------------- Swatches

/// A labeled color sample (`bg` with `fg` text), e.g. to document a theme's surfaces.
pub fn color_swatch<'a, M: 'a>(label: impl Into<String>, surface: Surface) -> Element<'a, M> {
    let m = Metrics::desktop();
    container(text(label.into()).size(12).color(surface.fg))
        .padding(m.space_m)
        .width(Length::Fill)
        .style(move |t: &Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(surface.bg)),
            border: iced::Border {
                color: crate::color::alpha(surface.fg, 0.2),
                width: 1.0,
                radius: t.metrics.radius_m.into(),
            },
            ..Default::default()
        })
        .into()
}

/// [`color_swatch`] for a status [`Swatch`] (`bg` / `fg`).
pub fn status_swatch<'a, M: 'a>(label: impl Into<String>, swatch: Swatch) -> Element<'a, M> {
    color_swatch(label, swatch.surface())
}

/// The accent color chooser: a row of round swatches, one per desktop accent, with a check on
/// the selected one. With [`custom`](AccentPicker::custom) a last swatch opens a
/// [`ColorPicker`](super::ColorPicker) (with a text input) in a popover for any other color, and
/// the choice is an [`AccentColor::Custom`].
/// The text the custom accent's entry shows and what typing sends.
type Custom<'a, M> = (&'a str, std::rc::Rc<dyn Fn(String) -> M + 'a>);

pub struct AccentPicker<'a, M> {
    selected: AccentColor,
    on_select: std::rc::Rc<dyn Fn(AccentColor) -> M + 'a>,
    custom: Option<Custom<'a, M>>,
    dialog: bool,
    scrim: bool,
    close_on_outside: bool,
    metrics: Metrics,
}

pub fn accent_picker<'a, M: Clone + 'a>(
    selected: AccentColor,
    on_select: impl Fn(AccentColor) -> M + 'a,
) -> AccentPicker<'a, M> {
    AccentPicker {
        selected,
        on_select: std::rc::Rc::new(on_select),
        custom: None,
        dialog: false,
        scrim: true,
        close_on_outside: true,
        metrics: Metrics::desktop(),
    }
}

impl<'a, M: Clone + 'a> AccentPicker<'a, M> {
    /// Adds the custom swatch. `text` is what the popover's entry shows (the app keeps it, so
    /// typing is not interrupted); `on_text` gets what was typed. Parse it with
    /// [`AccentColor::parse`] and select the result.
    pub fn custom(mut self, text: &'a str, on_text: impl Fn(String) -> M + 'a) -> Self {
        self.custom = Some((text, std::rc::Rc::new(on_text)));
        self
    }
    /// Opens the color picker as a dialog centered in the window (default: a popover).
    pub fn dialog(mut self, on: bool) -> Self {
        self.dialog = on;
        self
    }
    /// Fades the window behind the dialog (default on).
    /// Whether a press outside the dialog closes it (default true).
    pub fn close_on_outside(mut self, on: bool) -> Self {
        self.close_on_outside = on;
        self
    }
    pub fn scrim(mut self, on: bool) -> Self {
        self.scrim = on;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

/// A round swatch of `color`, with a check when `on`.
fn round_swatch<'a, M: 'a>(color: iced::Color, on: bool) -> Element<'a, M> {
    let content: Element<'a, M> = if on {
        Icon::Check
            .colored(14.0, crate::color::readable_on(color))
            .into()
    } else {
        iced::widget::space().into()
    };
    container(content)
        .center(Length::Fixed(26.0))
        .style(move |_: &Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border::default().rounded(999.0),
            ..Default::default()
        })
        .into()
}

impl<'a, M: Clone + 'a> From<AccentPicker<'a, M>> for Element<'a, M> {
    fn from(p: AccentPicker<'a, M>) -> Self {
        let m = p.metrics;
        let mut swatches: Vec<Element<'a, M>> = AccentColor::ALL
            .iter()
            .map(|a| {
                button(round_swatch(a.to_color(), *a == p.selected))
                    .padding(0)
                    .style(|_: &Theme, _| iced::widget::button::Style::default())
                    .on_press((p.on_select)(*a))
                    .into()
            })
            .collect();
        if let Some((text, on_text)) = p.custom {
            let current = p.selected.to_color();
            let c = Hsva::from_rgba(current.r, current.g, current.b, 1.0);
            let on_select = p.on_select.clone();
            let custom = matches!(p.selected, AccentColor::Custom(_));
            // A custom accent shows its color; otherwise a plus invites choosing one.
            let trigger: Element<'a, M> = if custom {
                round_swatch(current, true)
            } else {
                container(Icon::ListAdd.view(14.0))
                    .center(Length::Fixed(26.0))
                    .style(|t: &Theme| iced::widget::container::Style {
                        border: iced::Border {
                            color: t.semantic.border,
                            width: 1.5,
                            radius: 999.0.into(),
                        },
                        ..Default::default()
                    })
                    .into()
            };
            let input = ColorInput::new(
                c,
                // The text is the app's; a custom accent is its hex.
                text,
                move |t| on_text(t),
                move |h: Hsva| {
                    let [r, g, b] = h.to_rgb8();
                    on_select(AccentColor::custom(r, g, b))
                },
            );
            swatches.push(
                input
                    .swatches(AccentColor::ALL.map(|a| {
                        let c = a.to_color();
                        Hsva::from_rgba(c.r, c.g, c.b, 1.0).format(ColorFormat::Hex)
                    }))
                    .trigger(trigger)
                    .dialog(p.dialog)
                    .scrim(p.scrim)
                    .close_on_outside(p.close_on_outside)
                    .metrics(m)
                    .into(),
            );
        }
        row(swatches).spacing(m.space_s).wrap().into()
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
