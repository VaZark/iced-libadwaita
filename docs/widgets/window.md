# Window and ApplicationWindow

libadwaita: [`Window`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Window.html)

## What it is

Freeform window with no titlebar area; content goes in `content`. Hosts dialogs and breakpoints. Default minimum size 360×200. Adaptive preview: Ctrl+Shift+M.

## Spec

Min 360×200; corner radius 15.

## iced mapping

`window::settings()` (decorations off, transparent, min size 360×200), `window::settings_with_ssd(bool)` to opt back in, and `window::frame(content, state, on_resize)` (rounded 15px window background and border, 8 resize grips; square, borderless and grip-less when maximized or fullscreen per `window::State`); see [windows-and-csd](../design-language/windows-and-csd.md).

## Deviations / gaps

Child widgets are clipped to a rectangle, so square-cornered children (e.g. a sidebar background) can overpaint the rounded corners.
