# PreferencesDialog

libadwaita: [`PreferencesDialog`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.PreferencesDialog.html)

## What it is

A dialog showing application preferences in pages and groups, with search (1.5).

## Anatomy and behavior

Subpages via push/pop; `navigation.pop` action; `search-enabled`. Inherits AdwDialog.

## CSS nodes and classes

`dialog.preferences`.

## Spec

Floating when room, bottom sheet when narrow.

## iced mapping

`preferences_dialog(title, page, on_close)`. No search or subpages.
