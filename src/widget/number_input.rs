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
//! **Dragging.** [`drag`](NumberInput::drag) says what dragging on the input does: change the value
//! continuously (the default): a vertical drag raises it going up and lowers it going down, a
//! horizontal one raises it going right; or select text, as in any entry. A scrub moves by the
//! [`step`](NumberInput::step), so it keeps the digits and the limits, and the text updates on
//! every step through `on_input`. Holding Shift makes each step ten steps, Alt one unit of the
//! last digit. A click without travel still places the caret.
//!
//! The handles are two small buttons stacked in the right section, dimmed at a limit;
//! `handles(false)` hides them. Arrow keys and the wheel are not included: iced has no focus
//! model to hang them on.

use super::{Element, EntryState, entry};
use crate::Theme;
use crate::primitives::numeric::{Axis, Precision, Scrub};
use crate::primitives::numeric::{Decimal, Numeric};
use crate::primitives::validation::{self, Presentation, Validity};
use crate::style::button as sb;
use crate::{Icon, Metrics};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::widget::{button, column, container};
use iced::{Event, Length, Rectangle, Size, Vector, keyboard, mouse};

/// What dragging on a [`NumberInput`] does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumberDrag {
    /// Changes the value: up raises it, down lowers it (the default).
    #[default]
    Vertical,
    /// Changes the value: right raises it, left lowers it.
    Horizontal,
    /// Selects text, as in any entry.
    Select,
}

pub struct NumberInput<'a, M> {
    text: &'a str,
    on_input: Option<std::rc::Rc<dyn Fn(String) -> M + 'a>>,
    numeric: Numeric,
    placeholder: String,
    handles: bool,
    drag: NumberDrag,
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
            drag: NumberDrag::Vertical,
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
    /// What dragging on the input does: change the value continuously, [`Vertical`](NumberDrag::Vertical)
    /// (default) or [`Horizontal`](NumberDrag::Horizontal), or [`Select`](NumberDrag::Select) text
    /// as in any entry. While dragging, Shift makes each step ten steps and Alt one unit of the last
    /// digit.
    pub fn drag(mut self, mode: NumberDrag) -> Self {
        self.drag = mode;
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
        let field: Element<'a, M> = field.into();
        let axis = match n.drag {
            NumberDrag::Select => return field,
            NumberDrag::Horizontal => Axis::Horizontal,
            NumberDrag::Vertical => Axis::Vertical,
        };
        let Some(on_input) = n.on_input else {
            return field;
        };
        iced::Element::new(Scrubber {
            content: field,
            scrub: Scrub::new(axis),
            numeric: n.numeric,
            text: n.text.to_string(),
            on_input,
            // A press on the handles is theirs, not the start of a drag.
            guard: if n.handles {
                HANDLE_W + m.space_xs
            } else {
                0.0
            },
        })
    }
}

/// Turns a drag on the input into continuous updates (see [`NumberInput::drag`]).
struct Scrubber<'a, M> {
    content: Element<'a, M>,
    scrub: Scrub,
    numeric: Numeric,
    text: String,
    on_input: std::rc::Rc<dyn Fn(String) -> M + 'a>,
    /// Width at the right edge where a press is not a drag.
    guard: f32,
}

/// A press that may become a drag.
#[derive(Debug, Clone, Copy)]
struct Press {
    /// Where the press landed: a drag has begun once the pointer is far enough from it.
    from: (f32, f32),
    /// The value reached so far; each step is taken from it, so a modifier changing mid-drag
    /// changes only the steps still to come.
    value: Option<crate::primitives::numeric::Decimal>,
    /// Where the steps are counted from: the press, moved on by the travel each step used.
    anchor: (f32, f32),
    /// Travelled far enough to be a drag.
    dragging: bool,
}

#[derive(Default)]
struct ScrubState {
    press: Option<Press>,
    /// Shift and Alt as last reported.
    shift: bool,
    alt: bool,
}

impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for Scrubber<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ScrubState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(ScrubState::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<ScrubState>();
        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => {
                (state.shift, state.alt) = (m.shift(), m.alt());
            }
            Event::Keyboard(keyboard::Event::KeyPressed { modifiers, .. }) => {
                (state.shift, state.alt) = (modifiers.shift(), modifiers.alt());
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(Rectangle {
                    width: (bounds.width - self.guard).max(0.0),
                    ..bounds
                }) =>
            {
                if let Some(from) = cursor.position() {
                    state.press = Some(Press {
                        value: self.numeric.parse(&self.text),
                        anchor: (from.x, from.y),
                        from: (from.x, from.y),
                        dragging: false,
                    });
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let precision = Precision::from_modifiers(state.shift, state.alt);
                if let (Some(press), Some(at)) = (&mut state.press, cursor.position()) {
                    let to = (at.x, at.y);
                    if !press.dragging && self.scrub.started(press.from, to) {
                        press.dragging = true;
                        // Steps count from where the drag began.
                        press.anchor = press.from;
                    }
                    if press.dragging {
                        let steps = self.scrub.steps(press.anchor, to);
                        if steps != 0 {
                            let value = self.numeric.offset_by(press.value, steps, precision);
                            press.value = Some(value);
                            press.anchor = self.scrub.advance(press.anchor, steps);
                            let text = self.numeric.format(value);
                            if text != self.text {
                                shell.publish((self.on_input)(text));
                            }
                        }
                        // The entry does not see the move: it would extend a selection.
                        shell.capture_event();
                        shell.request_redraw();
                        return;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.press.take().is_some_and(|p| p.dragging) =>
            {
                shell.capture_event();
                return;
            }
            _ => {}
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if tree
            .state
            .downcast_ref::<ScrubState>()
            .press
            .is_some_and(|p| p.dragging)
        {
            return match self.scrub.axis {
                Axis::Horizontal => mouse::Interaction::ResizingHorizontally,
                Axis::Vertical => mouse::Interaction::ResizingVertically,
            };
        }
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
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
        for mode in [
            NumberDrag::Select,
            NumberDrag::Horizontal,
            NumberDrag::Vertical,
        ] {
            let _: Element<'_, ()> = mk("5").drag(mode).into();
        }
        let _: Element<'_, ()> = mk("5").into();
        let _: Element<'_, ()> = mk("5").handles(false).into();
        let _: Element<'_, ()> = mk("5").disabled(true).into();
    }
}
