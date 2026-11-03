# Timeline

No libadwaita widget. After Mantine's [Timeline](https://mantine.dev/core/timeline/).

## What it is

Events in order along a line. Each item has a bullet on the line, a title and a body; the items up to the `active` one are drawn as done, and the line between two done items is in the accent color.

## Anatomy and behavior

- **Shares the [stepper](stepper.md) primitive.** An item's bullet is the stepper's `Status` (completed, current, upcoming, plus the `invalid` and `loading` overrides) and the line after an item is the stepper's `Fill`, so a sequence reads the same whether it is a wizard or a history. A timeline's `active` is the last item that is done, as the stepper's `active` is the current step.
- **Items.** `TimelineItem::new(title)` with `body(element)`, `bullet(element)` (an icon or avatar inside the bullet instead of the status mark), `line(variant)` (solid, dashed or dotted: the [Divider](divider.md)'s), `invalid(true)` and `loading(true)`.
- `bullet_size`, `line_width`, `align(Left | Right)` (the line on the other side), and `reverse_active(true)` to count the done items from the end, for a newest-first history.

## iced mapping

`widget::Timeline`; the status and line fill come from `primitives::stepper` (`status`, `connector_fill`) and the connectors are the `Divider`'s rule, so there is no second implementation of either.
