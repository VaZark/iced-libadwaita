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

## Overlay

`widget::Overlay::new(base, layer)` is the layer under `dialog_in_window` and a nested window (a preferences window over a dialog): the base is faded and disabled while it is up (`scrim(false)` keeps it unfaded), and a press outside sends `on_outside` (a dialog's app closes it; leave it unset, or send something else, for a layer that stays). Its layer is a focus scope ([focus.md](focus.md)): Tab stays inside it, its first field (or the one named by `initial_focus(id)`) takes focus as it appears, and closing it returns focus to what had it. `overlay_stack` nests layers; the rules (scrim, modality, `close_on_outside`, Escape closing only the top layer) are `primitives::overlay`. A `Popover` presented as `Presentation::Dialog` and the pickers' `dialog(true)` use the same rules, closing on a press outside unless `close_on_outside(false)`.
