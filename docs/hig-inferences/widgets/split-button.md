# SplitButton

libadwaita: [`SplitButton`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.SplitButton.html)

## What it is

Button plus dropdown arrow in one control; popover from `menu-model` or custom.

## Anatomy and behavior

`can-shrink`, `dropdown-tooltip`, `direction`. Supports `.suggested-action`, `.destructive-action`, `.flat`, `.raised`; **not** `.pill`.

## CSS nodes and classes

`splitbutton` > `button`, `separator`, `menubutton` > `button.toggle` > `arrow`. `.image-button`/`.text-button`.

## Spec

Linked halves with a 1px separator.

## iced mapping

Two components, both a button linked to a menu arrow (`.linked` corners, 1px separator), built from
the same pieces as a two-segment toggle group:

- **`SplitButton`**: `split_button(content, on_press, menu)`. The button runs its action; the
  arrow opens a popover of **other** actions (a `Menu`, so items can be disabled, have accelerators
  or checks) and choosing an item runs that item. `AdwSplitButton`.
- **`SplitButtonAction`**: `SplitButtonAction::new(actions, selected, on_select, on_press)`. The
  menu **chooses what the button does**: the button shows the selected `SplitAction` (label,
  optional icon) and runs it; picking another item sends `on_select(value)` and changes the
  button's label and action without running anything (the "Create merge commit / Squash and
  merge / Rebase and merge" pattern). The selected item is the accent-colored one; actions can be
  `disabled()`. The app owns the selection.

Both take `.disabled(bool)` (button dimmed, menu cannot open) and `.metrics(..)`. The popover opens
below the arrow, aligned to its end. Not covered: `direction`, `dropdown-tooltip`, `can-shrink`.
