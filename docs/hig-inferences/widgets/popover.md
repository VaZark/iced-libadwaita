# Popover

libadwaita: [`GtkPopover`](https://docs.gtk.org/gtk4/class.Popover.html), and [`GtkPopoverMenu`](https://docs.gtk.org/gtk4/class.PopoverMenu.html) built on it.

## What it is

A trigger that opens a floating surface anchored to it. [`dropdown`](dropdown.md), [`Menu`](menu-button.md), `MultiSelect`, `DateInput`, `ComboRow` and the split button are all popovers with different triggers and content, the way a `GtkPopoverMenu` is a `GtkPopover`.

## Anatomy and behavior

- **Trigger and content.** `Popover::new(trigger, content)`; `content` is usually the `popover(items, max_height)` surface (rounded, 6 px padding, scrolls past the height). The trigger draws as a button (`appearance`: button, flat, list row, linked).
- **Placement** is the [anchor primitive](../../lib/primitives.md): `side(..)` and `align(..)` against the trigger, an `offset`, flipping to the opposite side when it does not fit and shifting to stay inside its bounds, with a 6 px margin.
- **Dismissal.** Click outside or Escape closes it; activating an item closes it unless `keep_open(true)`. `presentation(Page)` shows the content as a full-window page instead (the menu's mobile form).
- **Location aware.** A popover stays inside the **content area** it was opened from, not the whole window, so it never slides under a sidebar or a bar that is not part of that content. `content_area(content)` marks a region; `ToolbarView`'s content and the content of `NavigationSplitView` and `OverlaySplitView` are marked already, and the innermost one wins. A trigger outside every content area (a header bar button, a sidebar row) uses the window.
- **Dialog.** `presentation(Presentation::Dialog)` centers the content in the window over a faded, disabled window. `scrim(false)` keeps it clear. `close_on_outside(false)` keeps a press outside from closing it. It uses the [overlay rules](dialog.md#overlay).
- **Focus.** `initial_focus(id)` makes the content a focus scope (no Tab trap) that starts on that child.
- **Bounds.** `bounds(PopoverBounds::Window)` opts a popover out of the content area. The default is the desktop config's `[popovers] bounds` (`content` | `window`, default `content`); a full-window page always uses the window.

## iced mapping

`widget::Popover` is the trigger and the overlay; placement is the pure `place_within(anchor, size, region, placement)` (unit tested: a popover flips and shifts inside the region, not the window). `content_area` records its bounds while the overlays of what it wraps are collected, and a popover being created there reads them.
