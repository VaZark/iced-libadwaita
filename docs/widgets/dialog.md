# Dialog

libadwaita: [`Dialog`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Dialog.html)

## What it is

Adaptive dialog container: centered floating window or bottom sheet by available size (1.5).

## Anatomy and behavior

- Not resizable; `content-width`/`content-height`, `follows-content-size`; never larger than the parent window.
- `can-close`, `close-attempt`; closes via button/shortcut; header bar shows the dialog title and only a close button.
- Floating radius = `--window-radius` (15px ✓); shadow and dim layer per shadows.md ✓.

## CSS nodes and classes

`dialog`.

## Spec

Backdrop dims the window; floating dialogs have a shadow.

## iced mapping

`dialog(base, content, on_backdrop)`; radius 15 (`Metrics::radius_window`). Dialog width is `Fill` up to a max, with 12px margin, so it fits a 360px window.

## Deviations / gaps

No bottom-sheet mode.
