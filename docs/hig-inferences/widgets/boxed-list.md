# BoxedList

libadwaita: [`ActionRow`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ActionRow.html)

## What it is

`.boxed-list` (one card, hairline separators from the card shade color) and `.boxed-list-separate` (each row its own card). `GtkListBox` with no selection.

## Anatomy and behavior

Rows: ActionRow, SwitchRow, ComboRow, ExpanderRow, EntryRow, PasswordEntryRow, SpinRow, ButtonRow.

## Spec

Card radius 12; separator = `--card-shade-color`.

## iced mapping

`boxed_list(rows)` clips to the card radius.
