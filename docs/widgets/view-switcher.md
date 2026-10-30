# ViewSwitcher, ViewSwitcherBar, InlineViewSwitcher

libadwaita: [`ViewSwitcher`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ViewSwitcher.html)

## What it is

Switch between ViewStack pages. `ViewSwitcher` in the header bar (policy wide: icon beside label; narrow: icon above). `ViewSwitcherBar` is the bottom bar revealed at `max-width: 550sp`. `InlineViewSwitcher` (1.7) is a ToggleGroup bound to a stack with `display-mode`.

## Anatomy and behavior

Bar can't be revealed with fewer than two pages.

## CSS nodes and classes

`viewswitcher` (`.wide`/`.narrow`); `viewswitcherbar`; `inline-view-switcher`.

## iced mapping

`view_switcher`, `view_switcher_bar`, `inline_view_switcher` (all `ToggleGroup`).

## Deviations / gaps

No narrow policy / automatic hand-off to the bar.
