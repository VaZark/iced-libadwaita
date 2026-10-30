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
