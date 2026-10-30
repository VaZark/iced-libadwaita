# EntryRow

libadwaita: [`EntryRow`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.EntryRow.html)

## What it is

A row with an embedded text entry; the title doubles as placeholder and floats above the text once there is content.

## Anatomy and behavior

- Editable indicator icon, prefix/suffix widgets, optional **apply button** while editing (`show-apply-button`).
- Signals: `apply`, `entry-activated`. Implements `GtkEditable`.
- `.monospace` changes only the editable part.

## CSS nodes and classes

`row.entry`.

## Spec

Same height as an action row (≈50 ‡); text field has no own frame.

## iced mapping

`EntryRow::new(title, value).on_input(f).on_submit(msg).suffix(..)`; title shown as caption above the field. No apply-button yet.
