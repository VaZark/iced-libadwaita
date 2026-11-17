//! `NoteCard`: a note as a card, as in Google Keep. A rounded surface holding a title, a snippet
//! and chips; it takes an optional colour tint (a soft wash, not a solid fill), shows a ring when
//! selected, and lifts a little when hovered or pressed. The whole card is one button; controls
//! inside it stay independent.
//!
//! Put a long body in a [`Spoiler`](crate::widgets::Spoiler) with
//! [`preview`](crate::widgets::Spoiler::preview) so a card shows only the start of it, clipped
//! at the card's content edge.
//!
//! ```ignore
//! NoteCard::new(column![title, Spoiler::new(body).max_height(170.0).preview(true)])
//!     .tint(epic_color)
//!     .selected(open_id == Some(id))
//!     .on_press(Msg::Open(id))
//! ```

use crate::color::{color_contrast, mix};
use crate::semantic::Role;
use crate::styles::container::card_surface;
use crate::{Element, Theme};
use iced::widget::button;
use iced::{Background, Border, Color, Length, Padding};

/// How strong a tint is: a soft pastel wash.
const TINT_ALPHA: f32 = 0.22;

/// The card's fill. An untinted card keeps the card surface; a tinted one is its tint at a soft
/// alpha. Hovering and pressing lift it: more opaque over a translucent surface, lighter or
/// darker over an opaque one, according to its contrast.
fn fill(theme: &Theme, tint: Option<Color>, status: button::Status) -> Color {
    let surface = match card_surface(theme).background {
        Some(Background::Color(c)) => c,
        _ => Color::TRANSPARENT,
    };
    let base = tint.map_or(surface, |t| Color { a: TINT_ALPHA, ..t });
    let lift = match status {
        button::Status::Hovered => 0.06,
        button::Status::Pressed => 0.10,
        _ => 0.0,
    };
    if lift == 0.0 {
        base
    } else if base.a < 0.9 {
        Color {
            a: (base.a + lift).min(1.0),
            ..base
        }
    } else {
        mix(
            base,
            color_contrast(base, &[Color::BLACK, Color::WHITE]),
            lift,
        )
    }
}

/// A note card. See the [module](self).
pub struct NoteCard<'a, M> {
    content: Element<'a, M>,
    tint: Option<Color>,
    selected: bool,
    on_press: Option<M>,
    padding: Padding,
    width: Length,
}

impl<'a, M: Clone + 'a> NoteCard<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
            tint: None,
            selected: false,
            on_press: None,
            padding: Padding::from(12.0),
            width: Length::Fill,
        }
    }
    /// The note's colour.
    pub fn tint(mut self, tint: impl Into<Option<Color>>) -> Self {
        self.tint = tint.into();
        self
    }
    /// Draws the accent ring (controlled).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn on_press(mut self, message: M) -> Self {
        self.on_press = Some(message);
        self
    }
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, M: Clone + 'a> From<NoteCard<'a, M>> for Element<'a, M> {
    fn from(c: NoteCard<'a, M>) -> Self {
        let (tint, selected) = (c.tint, c.selected);
        button(c.content)
            .width(c.width)
            .padding(c.padding)
            .on_press_maybe(c.on_press)
            .style(move |theme: &Theme, status| {
                let surface = card_surface(theme);
                let mut border = surface.border;
                if selected {
                    border = Border {
                        color: theme.semantic.swatch(Role::Accent).color,
                        width: 2.0,
                        ..border
                    };
                }
                button::Style {
                    background: Some(Background::Color(fill(theme, tint, status))),
                    text_color: surface
                        .text_color
                        .unwrap_or(theme.semantic.swatch(Role::Neutral).color),
                    border,
                    shadow: surface.shadow,
                    ..button::Style::default()
                }
            })
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    #[test]
    fn a_tint_replaces_the_surface_with_a_soft_wash_of_its_colour() {
        for theme in [Theme::dark(), Theme::light()] {
            let tinted = fill(&theme, Some(RED), button::Status::Active);
            assert_eq!((tinted.r, tinted.g, tinted.b), (1.0, 0.0, 0.0));
            assert_eq!(tinted.a, TINT_ALPHA);
        }
    }

    #[test]
    fn hover_and_press_lift_the_card() {
        for theme in [Theme::dark(), Theme::light()] {
            for tint in [None, Some(RED)] {
                let rest = fill(&theme, tint, button::Status::Active);
                let hover = fill(&theme, tint, button::Status::Hovered);
                let press = fill(&theme, tint, button::Status::Pressed);
                assert_ne!(hover, rest);
                assert_ne!(press, hover);
                assert_eq!(fill(&theme, tint, button::Status::Disabled), rest);
            }
        }
    }

    #[test]
    fn opaque_custom_surfaces_use_their_own_colour() {
        for mut theme in [Theme::dark(), Theme::light()] {
            for base in [Color::BLACK, Color::WHITE] {
                theme.semantic.card.bg = base;
                let hover = fill(&theme, None, button::Status::Hovered);
                assert_eq!(hover.a, 1.0);
                assert_ne!(hover, base);
            }
        }
    }

    #[test]
    fn builds() {
        let _: Element<'_, u8> = NoteCard::new(iced::widget::text("n"))
            .tint(RED)
            .selected(true)
            .on_press(1)
            .into();
    }
}
