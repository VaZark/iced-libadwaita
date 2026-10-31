//! Form and display components: entries, text view, check/radio buttons, scale, link and
//! card buttons, tags and color swatches.
//!
//! Each is a thin, themed wrapper with libadwaita-style properties, so a layout (code, XML or
//! JSX-like) can use one vocabulary instead of raw iced widgets.

use super::Element;
use crate::style;
use crate::{AccentColor, Icon, Metrics, Role, Surface, Swatch, Theme};
use iced::widget::{button, checkbox, container, radio, row, slider, text, text_editor, text_input};
use iced::widget::{Button, Checkbox, Radio, Slider};
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
}

/// `GtkEntry`.
pub fn entry<'a, M>(placeholder: impl Into<String>, value: impl Into<String>) -> Entry<'a, M> {
    Entry::new(placeholder, value)
}

/// `GtkPasswordEntry`: hides its contents.
pub fn password_entry<'a, M>(placeholder: impl Into<String>, value: impl Into<String>) -> Entry<'a, M> {
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
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<Entry<'a, M>> for Element<'a, M> {
    fn from(e: Entry<'a, M>) -> Self {
        let mut input = text_input(&e.placeholder, &e.value)
            .padding(e.metrics.entry_padding)
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
        if let Some(f) = e.on_input {
            input = input.on_input(f);
        }
        if let Some(m) = e.on_submit {
            input = input.on_submit(m);
        }
        input.into()
    }
}

// --------------------------------------------------------------- TextView

/// `GtkTextView`: a multi-line editor over a [`text_editor::Content`]. Without
/// [`on_action`](TextView::on_action) it is read-only and drawn disabled.
pub struct TextView<'a, M> {
    content: &'a text_editor::Content,
    on_action: Option<Box<dyn Fn(text_editor::Action) -> M + 'a>>,
    placeholder: Option<String>,
    height: Length,
    metrics: Metrics,
}

pub fn text_view<'a, M>(content: &'a text_editor::Content) -> TextView<'a, M> {
    TextView { content, on_action: None, placeholder: None, height: Length::Fixed(90.0), metrics: Metrics::desktop() }
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
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }
}

impl<'a, M: Clone + 'a> From<TextView<'a, M>> for Element<'a, M> {
    fn from(t: TextView<'a, M>) -> Self {
        let mut e = text_editor(t.content)
            .height(t.height)
            .padding(t.metrics.entry_padding)
            .style(style::text_editor::default);
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
    button(super::typography::label(label.into())).padding(Metrics::desktop().space_xs).style(style::button::link)
}

/// A clickable `.card`: any content on a card surface that reacts to hover and press.
pub fn card_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    button(content).padding(Metrics::desktop().card_padding).style(style::button::card)
}

// -------------------------------------------------------------------- Tag

/// A small rounded label tinted with a [`Role`] (status chip). Not a libadwaita widget; the
/// pattern behind `.tag`-style badges in GNOME apps.
pub fn tag<'a, M: 'a>(label: impl Into<String>, role: Role) -> Element<'a, M> {
    container(text(label.into()).size(Metrics::desktop().space_m + 1.0).wrapping(text::Wrapping::None))
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

/// The system accent color chooser: a row of round swatches with a check on the selected one.
pub fn accent_picker<'a, M: Clone + 'a>(
    selected: AccentColor,
    on_select: impl Fn(AccentColor) -> M + 'a,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    row(AccentColor::ALL.iter().map(|a| {
        let color = a.to_color();
        let content: Element<'a, M> = if *a == selected {
            Icon::Check.colored(14.0, iced::Color::WHITE).into()
        } else {
            iced::widget::space().into()
        };
        button(container(content).center(Length::Fixed(26.0)))
            .padding(0)
            .style(move |_: &Theme, _| iced::widget::button::Style {
                background: Some(iced::Background::Color(color)),
                border: iced::Border::default().rounded(999.0),
                ..Default::default()
            })
            .on_press(on_select(*a))
            .into()
    }))
    .spacing(m.space_s)
    .wrap()
    .into()
}
