# NumberInput

No libadwaita widget. `GtkSpinButton` is the closest, with float values. After Mantine's [NumberInput](https://mantine.dev/core/number-input/).

## What it is

An entry for a number, with two handles (up, down) in its right section that step it. Dragging on it changes the value.

## Parameters

| Param         | Meaning                                                  | Default            |
| ------------- | -------------------------------------------------------- | ------------------ |
| `decimals(n)` | fraction digits kept and shown; 0 is an integer          | `0`                |
| `min`, `max`  | limits                                                   | none               |
| `step`        | what a handle or a drag step adds                        | `1`                |
| `drag`        | `Vertical`, `Horizontal`, `Select`                       | `Vertical`         |
| `handles`     | show the up and down handles                             | on                 |
| `placeholder`, `width`, `disabled`, `revealed` | as in an entry               |                    |

## Behavior

- **Fixed point, never float.** The value is a `Decimal` (`units` of `10^-digits`). A step is exact: `0.1 + 0.2` is `0.3`. The text always has exactly the digits: `12.50`. More typed digits are rounded. Rules: `primitives::numeric` (`Numeric`).
- **Limits.** `min` and `max` clamp handles and drags. Typed text outside them, or not a number, is the error state. A handle is dimmed at its limit. From an empty entry a handle starts at the limit nearest zero.
- **Handles.** Press to step. `handles(false)` hides them.
- **Dragging.** On the input:

  | Mode         | Drag                                   |
  | ------------ | -------------------------------------- |
  | `Vertical`   | up raises, down lowers                 |
  | `Horizontal` | right raises, left lowers              |
  | `Select`     | selects text, as in any entry          |

  - 4 px of travel is one step (`Scrub`). Nothing counts as a drag until 4 px along the axis, so a click still places the caret.
  - **Shift:** each step is ten steps. **Alt:** each step is one unit of the last digit (`Precision`).
  - A modifier pressed mid-drag changes only the steps still to come.
  - The handles keep their own presses.
- **One message.** `on_input(text)`, for typing, handles and dragging. The app keeps the text. Read the number with `Numeric::parse`.

```rust
NumberInput::new(&self.text, Msg::Text)
    .decimals(2).min(0).max(100)
    .step(Decimal::new(25, 2))   // 0.25
```

## iced mapping

An `Entry` whose `right_section` is two stacked flat buttons, wrapped in a widget that turns press-and-move into `on_input` messages and keeps the entry from extending a selection while it scrubs.

## Deviations

No arrow keys, no mouse wheel, no hold-to-repeat on the handles (iced has no focus model for them).
