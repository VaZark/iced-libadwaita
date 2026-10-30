# DropDown (GtkDropDown) and menu popover

libadwaita: [`ComboRow`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ComboRow.html)

## What it is

The generic choose-one control used by ComboRow and standalone as `GtkDropDown`. (Not an Adw class; documented here because it drives `ComboRow`.)

## Anatomy and behavior

- Closed state: a button with the current value left-aligned and a `pan-down` chevron at the end. It follows button styling (translucent fill, 9px radius); inside rows/toolbars it is flat (`.flat` supported).
- Open state: popover (`.menu`) anchored below the button, list of items, selected item marked with a trailing check, keyboard navigable.

## CSS nodes and classes

`dropdown` > `button` + `popover.menu`.

## Spec

- Button 34 high ✓. Popover radius 15, items radius 9, item height ≈32 ‡, popover content padding 8 ✓.

## iced mapping

`adw::widget::Dropdown` (advanced widget with overlay) + `dropdown()` helper; styles `style::button::dropdown`, `style::container::popover`, `popover_item`.
