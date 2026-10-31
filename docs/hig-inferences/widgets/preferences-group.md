# PreferencesGroup

libadwaita: [`PreferencesGroup`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.PreferencesGroup.html)

## What it is

A titled group of closely related preferences rendered as a boxed list.

## Anatomy and behavior

- `title`, `description`, `header-suffix`, `separate-rows` (≙ `.boxed-list-separate`, 1.6).
- Non-row children go in a box below the list.
- Title is a heading; description is dimmed.

## CSS nodes and classes

`preferencesgroup`. Role: group.

## Spec

Header → list gap 12 ‡; groups spaced 24 apart on a page.

## iced mapping

`PreferencesGroup::new().title().description().header_suffix().push(row)`.

## Deviations / gaps

`separate_rows` not implemented.
