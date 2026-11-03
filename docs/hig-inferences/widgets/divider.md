# Divider

No libadwaita widget (libadwaita draws its own hairlines between rows and bars). After Mantine's [Divider](https://mantine.dev/core/divider/).

## What it is

A line between things: horizontal or vertical, solid, dashed or dotted, of any thickness, with an optional label at the start, center or end of a horizontal line.

## Anatomy and behavior

- **Label.** `label(element)` is any element (text, an icon with text, a link); `label_position(Left | Center | Right)`. At the left or right the line runs on one side only; in the center it runs on both. A vertical divider has no label.
- **Variant and size.** `variant(Solid | Dashed | Dotted)`, `size(px)`. Dashes and dots scale with the thickness.
- **Color.** `tone(..)` picks one of the theme's hairline colors (border, card shade, bar, popover, sidebar), so the line follows the scheme.
- `spacing(px)` is empty space on both sides across the line.

## iced mapping

`widget::Divider` draws its line with a small widget of its own (a container cannot dash); the dash geometry is a unit tested pure function. The same line is what a [Timeline](timeline.md) uses for its connectors.
