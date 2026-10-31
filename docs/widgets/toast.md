# Toast and ToastOverlay

libadwaita: [`ToastOverlay`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ToastOverlay.html)

## What it is

Transient notification over content. `AdwToast`: title (or custom widget), one optional button, timeout, priority, always a close button.

## Anatomy and behavior

Re-adding a visible toast bumps its timeout; `dismiss_all`; signals `button-clicked`, `dismissed`.

## CSS nodes and classes

`toastoverlay` > child, `toast` > `widget`(`label.heading`) + `button` + `button.circular.flat`.

## Spec

OSD surface, pill radius, centered at the bottom.

## iced mapping

`toast_overlay(content, Option<Toast>)`; the app clears it (no timers in widgets). Slides up on show (300ms, `ADW_EASE`).

## Deviations / gaps

No timeout/priority queue; no hide animation (fade + 0.95 scale) because the app removes the toast.

## Colors

The toast is a **solid surface**, not a translucent OSD. Its colors follow CSS `color-contrast()`:
the surface is whichever of a dark and a light tone contrasts more with the window behind it (dark
on a light window, light on a dark one), and the text, action button and close icon are whichever
of light and dark text contrasts more with that surface (`color::readable_on`).

Overrides: `Toast::role(Role::…)` uses a status fill instead (text still by contrast),
`Toast::background(color)` hard-codes the surface, `Toast::text_color(color)` hard-codes the text.
`Toast::action(label, msg)` adds the optional call-to-action button.
