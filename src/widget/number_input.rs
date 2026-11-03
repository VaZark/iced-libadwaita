//! `NumberInput`: an entry for a number with handles that step it (Mantine's `NumberInput`).
//!
//! The number is a fixed-point [`Decimal`](crate::primitives::numeric::Decimal), never a float:
//! [`decimals`](NumberInput::decimals) is how many fraction digits it keeps (0 is an integer),
//! [`min`](NumberInput::min) / [`max`](NumberInput::max) bound it and [`step`](NumberInput::step)
//! is what a handle adds or takes. The rules are the [`numeric`](crate::primitives::numeric)
//! primitive.
//!
//! The app keeps only the text (so typing is not interrupted), and the widget has one message:
//! `on_input(text)`, for typing and for a handle (which sends the stepped number formatted with
//! the digits). Read the number with [`Numeric::parse`](crate::primitives::numeric::Numeric::parse).
//! Text that is not a number or is out of range draws the error state.
//!
//! ```ignore
//! NumberInput::new(&self.text, Msg::Text)
//!     .decimals(2)
//!     .min(0)
//!     .max(100)
//!     .step(Decimal::new(25, 2))   // 0.25
//! ```
//!
//! The handles are two small buttons stacked in the right section, dimmed at a limit;
//! `handles(false)` hides them. Arrow keys and the wheel are not included: iced has no focus
//! model to hang them on.

use super::{Element, EntryState, entry};
use crate::primitives::numeric::{Decimal, Numeric};
use crate::primitives::validation::{self, Presentation, Validity};
use crate::style::button as sb;
use crate::{Icon, Metrics};
use iced::Length;
use iced::widget::{button, column, container};

pub struct NumberInput<'a, M> {
    text: &'a str,
    on_input: Option<std::rc::Rc<dyn Fn(String) -> M + 'a>>,
    numeric: Numeric,
    placeholder: String,
    handles: bool,
    disabled: bool,
    revealed: bool,
    width: Length,
    metrics: Metrics,
}

impl<'a, M> NumberInput<'a, M> {
    /// An integer input with no limits, stepping by 1. `text` is the entry's content.
    pub fn new(text: &'a str, on_input: impl Fn(String) -> M + 'a) -> Self {
        Self {
            text,
            on_input: Some(std::rc::Rc::new(on_input)),
            numeric: Numeric::default(),
            placeholder: String::new(),
            handles: true,
            disabled: false,
            revealed: true,
            width: Length::Fixed(160.0),
            metrics: Metrics::desktop(),
        }
    }
    /// All the rules at once.
    pub fn numeric(mut self, n: Numeric) -> Self {
        self.numeric = n;
        self
    }
    /// Fraction digits kept and shown; 0 (the default) is an integer.
    pub fn decimals(mut self, digits: u8) -> Self {
        self.numeric = self.numeric.digits(digits);
        self
    }
    pub fn min(mut self, d: impl Into<Decimal>) -> Self {
        self.numeric = self.numeric.min(d);
        self
    }
    pub fn max(mut self, d: impl Into<Decimal>) -> Self {
        self.numeric = self.numeric.max(d);
        self
    }
    /// What a handle adds or takes (default 1).
    pub fn step(mut self, d: impl Into<Decimal>) -> Self {
        self.numeric = self.numeric.step(d);
        self
    }
    /// Whether the step handles show (default true).
    pub fn handles(mut self, on: bool) -> Self {
        self.handles = on;
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
    /// Whether validation may show yet; see [`DateInput::revealed`](super::DateInput::revealed).
    pub fn revealed(mut self, revealed: bool) -> Self {
        self.revealed = revealed;
        self
    }
    /// Drawn dimmed; cannot be edited or stepped.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
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

    /// The validity of the typed text.
    pub fn validity(&self) -> Validity {
        self.numeric.validate(self.text)
    }
    /// The entry state: an error only once [`revealed`](Self::revealed).
    pub fn text_state(&self) -> EntryState {
        match validation::present(&self.validity(), self.revealed) {
            Presentation::Invalid => EntryState::Error,
            _ => EntryState::Normal,
        }
    }
}

/// Width of the handles' slot.
const HANDLE_W: f32 = 22.0;

impl<'a, M: Clone + 'a> From<NumberInput<'a, M>> for Element<'a, M> {
    fn from(n: NumberInput<'a, M>) -> Self {
        let m = n.metrics;
        let mut field = entry::<M>(n.placeholder.clone(), n.text)
            .state(n.text_state())
            .width(n.width)
            .metrics(m);
        if n.disabled {
            return field.into();
        }
        if let Some(f) = &n.on_input {
            let f = f.clone();
            field = field.on_input(move |t| f(t));
        }
        if n.handles
            && let Some(f) = &n.on_input
        {
            let value = n.numeric.parse(n.text);
            let up = value.is_none_or(|v| n.numeric.can_up(v));
            let down = value.is_none_or(|v| n.numeric.can_down(v));
            let handle = |icon: Icon, enabled: bool, text: String| {
                let glyph = container(icon.view(10.0))
                    .center(Length::Fill)
                    .width(Length::Fixed(HANDLE_W))
                    .height(Length::Fixed(14.0));
                let mut b = button(glyph).padding(0).style(sb::flat);
                if enabled {
                    b = b.on_press(f(text));
                }
                b
            };
            let handles = column![
                handle(Icon::GoUp, up, n.numeric.up_text(n.text)),
                handle(Icon::GoDown, down, n.numeric.down_text(n.text)),
            ];
            field = field.right_section(handles).right_section_width(HANDLE_W);
        }
        field.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(text: &str) -> NumberInput<'_, ()> {
        NumberInput::new(text, |_| ()).decimals(2).min(0).max(100)
    }

    #[test]
    fn the_entry_flags_text_that_is_not_a_number_or_out_of_range() {
        assert_eq!(mk("").text_state(), EntryState::Normal);
        assert_eq!(mk("12.5").text_state(), EntryState::Normal);
        assert_eq!(mk("abc").text_state(), EntryState::Error);
        assert_eq!(mk("101").text_state(), EntryState::Error);
        assert_eq!(mk("101").revealed(false).text_state(), EntryState::Normal);
    }

    #[test]
    fn it_builds_with_and_without_handles() {
        let _: Element<'_, ()> = mk("5").into();
        let _: Element<'_, ()> = mk("5").handles(false).into();
        let _: Element<'_, ()> = mk("5").disabled(true).into();
    }
}
