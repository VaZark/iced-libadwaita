# HeaderBar

libadwaita: [`HeaderBar`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.HeaderBar.html)

## What it is

Title bar with start/end children and window controls; the window drag handle.

## Anatomy and behavior

- Title from the enclosing container (window/page/dialog) unless `title-widget`; `show-title`; `centering-policy` strict.
- Window controls per `decoration-layout`; in dialogs only close; in split views only at window edges.
- Back button automatically with NavigationView/SplitView (`show-back-button`).
- `pack_start` / `pack_end`; multiple children per side.

## CSS nodes and classes

`headerbar` > `windowhandle` > `box` > [`box.start` (windowcontrols.start, back button, children), title, `box.end`]. Classes `.start`, `.end`, `.back`.

## Spec

Height ≈47 ‡ (includes 1px border), 6px child spacing/margins (docs `.toolbar`), flat icon buttons.

## iced mapping

`header_bar().title().subtitle().pack_start().pack_end().window_controls().on_drag().on_double_click()`. Layout is a custom three-slot widget implementing the **loose** centering policy: title centered when both sides leave room, else shifted toward free space, else clipped; start/end slots are never squeezed, so window controls stay reachable down to 360px.

## Deviations / gaps

Window controls follow the user's GNOME `button-layout` and icon theme (see windows-and-csd.md); `show_start_title_buttons` / `show_end_title_buttons` let split-view panes show only their edge's buttons; hidden entirely in mobile mode. In a split view give **each pane its own header bar** (sidebar header: app title + primary menu; content header: page title) with `window_corners(left, right)` rounding only the outer corners; `window_radius(r)` rounds both. Strict centering policy and the automatic back button are not implemented (add the back button yourself).
