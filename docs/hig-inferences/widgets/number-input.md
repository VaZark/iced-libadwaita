# NumberInput

No libadwaita widget (`GtkSpinButton` is the closest, with float values). After Mantine's [NumberInput](https://mantine.dev/core/number-input/).

## What it is

An entry for a number with two small handles (up and down) in its right section that step it.

## Anatomy and behavior

- **Fixed point, not float.** The number is a `Decimal` (`units` of `10^-digits`): `decimals(n)` is how many fraction digits it keeps, 0 (the default) an integer. A step is exact (`0.1 + 0.2` is `0.3`) and the text always has exactly the digits (`12.50`). Typed text with more digits is rounded to them. The rules are `primitives::numeric` (`Numeric`: digits, `min`, `max`, `step`).
- **Limits.** `min` / `max` clamp what a handle gives; typed text outside them is the error state ("Must be 0.00 or more."). A handle is dimmed at its limit.
- **Step.** `step(..)` is what a handle adds or takes (default 1; `Decimal::new(25, 2)` is 0.25). From an empty or invalid entry a handle starts at the limit nearest zero.
- **Dragging.** `drag(NumberDrag)`: `Vertical` (default) and `Horizontal` change the value continuously while the pointer is dragged on the input (up or right raises it, down or left lowers it); `Select` selects text as in any entry. A scrub moves by the step, so it keeps the digits and the limits (`primitives::numeric::Scrub`: 4 px per step, and a drag only begins after 4 px of travel along the axis, so a click still places the caret). **Shift** makes each step ten steps (`Precision::Coarse`) and **Alt** one unit of the last digit (`Precision::Fine`); a modifier pressed mid-drag changes only the steps still to come. The handles keep their own presses.
- **Handles.** `handles(false)` hides them.
- **One message.** The app keeps the text; `on_input(text)` is sent for typing and for a handle (the stepped number as text). Read the number with `Numeric::parse`.
- Controlled: the app owns the text.

## iced mapping

`widget::NumberInput` is an `Entry` whose `right_section` is two stacked flat buttons. `NumberInput` wraps the entry in a widget that turns a press-and-move into `on_input` messages and keeps the entry from extending a selection while it scrubs. Not included: arrow keys and the wheel (iced has no focus model for them) and hold-to-repeat.
