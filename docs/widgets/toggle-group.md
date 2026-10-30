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

`toggle_group(items, active, on_select)`; the active toggle uses the *active toggle* surface (white / 20% white) with a faint shadow, on a translucent trough.

## Deviations / gaps

No `homogeneous`/`can-shrink`/vertical orientation yet.
