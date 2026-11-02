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

#### Anchored placement and MultiSelect

Deviations (neither exists in libadwaita; GTK popovers pick a side with `position` and flip on their own):

- **Anchor positioning.** The popover's position is the pure function `anchor::place` (CSS anchor-positioning vocabulary: `Side`, `Align`, `offset`, flip, shift), used by `Dropdown::side(..)` and `.offset(..)`. It flips to the opposite side when the preferred one does not fit (or has less room) and shifts along the side to stay inside the window, keeping a 6px margin. `Align` gained `Center`.
- **`keep_open`.** A popover normally closes when an item is activated. `Dropdown::keep_open(true)` keeps it open (multi-selection, calendar paging); outside click and Escape still close it. The popover's open state stays internal to the widget, so an app cannot close it from `update`.
- **MultiSelect** shares the option rows (`option_items`) with `dropdown()`. Its trigger shows the selection as display-only badges, `+n` after `max_chips`: the trigger toggles the popover and does not forward events, so values are removed by toggling them in the list.
