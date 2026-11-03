# Spoiler

No libadwaita widget. After Mantine's [Spoiler](https://mantine.dev/core/spoiler/).

## What it is

Show part of something and reveal the rest on request. Two kinds of "part".

## Anatomy and behavior

- **Part of the height.** `Spoiler::new(content)` clips its content to `max_height`, fades the cut, and shows a Show more / Hide control under it, only when the content is taller than that. It slides open with the theme's reveal motion. Controlled: `expanded(bool)` and `on_toggle(|open| ..)`.
- **Part of the data.** `Spoiler::items(&state, build)` builds **only the first `state.shown()` items** plus a control for the rest. The hidden items are never built or laid out, so a long list costs what its visible part costs. A press reveals the next chunk (`SpoilerState::step(n)`) or all of them, and Hide goes back to the first part; the control reads "Show N more" with the number the next press reveals. The app applies `SpoilerEvent` with `state.apply(event)`.
- Content that fits needs no control.

## iced mapping

`primitives::spoiler` is pure and unit tested (`overflows`, `height`, `SpoilerState`); the height form is a small widget that measures its content, clips it and positions the control; the data form is a column of the built items. For a list that is very long even when fully shown, use [virtualization](../../lib/primitives.md) in the list itself.
