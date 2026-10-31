# StatusPage

libadwaita: [`StatusPage`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.StatusPage.html)

## What it is

Empty/error state: icon, title, description, custom child below.

## Anatomy and behavior

`icon-name`/`paintable` (a spinner paintable adds `.spinner`), `title`, `description` (markup), `child`. `.compact` for sidebars/popovers.

## CSS nodes and classes

`statuspage` (+ `.spinner`). Class `.compact`.

## Spec

Icon is large (≈128 ‡), title `.title-1`, description body dimmed, child pill button.

## iced mapping

`status_page().icon().title().description().child().compact(true)`; content clamped and centered, wraps text.
