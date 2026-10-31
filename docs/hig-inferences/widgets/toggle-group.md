# ToggleGroup and Toggle

libadwaita: [`ToggleGroup`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ToggleGroup.html)

## What it is

Mutually exclusive toggles; each has an icon, label, both, or custom child (1.7).

## Anatomy and behavior

`active`/`active-name`, `homogeneous`, `can-shrink` (ellipsize), orientation. `.flat` and `.round`. Radio-group a11y.

## CSS nodes and classes

`toggle-group` > `toggle` (`.text-button`/`.image-button`/`.image-text-button`), `separator` (`.hidden`).

## Spec

Active toggle uses the *active toggle* surface (white / 20% white) on a translucent trough; separators between toggles.

## iced mapping

`toggle_group(items, active, on_select)` is a custom segmented control that owns its layout: with `.width(Fill)`, or when a stacked row demands 100%, every segment gets an equal share (`flex: 1`) with its content centered, never narrower than its own content; too wide, it shrinks fairly (short segments keep their size) and never overshoots; the switch to icons follows the **breakpoint**: `.window_width(width)` (`compact(Some(is_mobile(width)))`) shows icon-only in mobile mode and icon + text above it; a segment with no icon falls back to its text. Without it, if any segment would be squeezed, **all** segments that have a compact form switch to icon-only together. Either way it is all icons or all full content. `.homogeneous(true)` forces equal widths. Segments take a label, an icon, icon+label, or any element (`ToggleItem::{new, icon_only, icon, custom}`). The active toggle uses the *active toggle* surface with a faint shadow, on a translucent trough.

## Deviations / gaps

No vertical orientation; no ellipsis (clipped inside its segment); no animated active indicator.
