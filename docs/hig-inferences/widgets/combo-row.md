# ComboRow

libadwaita: [`ComboRow`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ComboRow.html)

## What it is

A row for choosing one item from a list. Shows the current selection; activating opens a popover list. Mirrors `GtkDropDown`.

## Anatomy and behavior

- Activatable only when a model is set. `selected` / `selected-item`; `use-subtitle` shows the value as subtitle instead of in the suffix.
- `enable-search` adds a search entry in the popup (1.4).
- Row shows the current value (dim) and a down-arrow icon at the end.
- The popover is a **menu popover**: a scrolled list, selected item marked with a check.

## CSS nodes and classes

Row: `row.combo`. Popover: `popover.menu` > `scrolledwindow` > `listview`. Role: combo box.

## Spec

- Popover: radius 15 ✓, padding 8 ✓, layered shadow (see shadows.md) ✓; items radius 9 (`$menu_radius`) ✓, min height ≈32 ‡, hover tint, check icon at end for the selected item.
- Opens below the row, at least as wide as the trigger; click outside / Escape closes.

## iced mapping

`ComboRow::new(title, options, selected, on_select)` using the shared `adw::widget::Dropdown` (custom overlay popover).

## Deviations / gaps

No search entry yet.
