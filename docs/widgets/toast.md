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

The toast is a **solid, always-dark surface**, not a translucent OSD and never a light slab on a
dark window: the theme's own elevated surface (`popover`) in the dark scheme, a dark ink tone in the
light scheme. The text, action button and close icon are chosen with CSS `color-contrast()`:
whichever of light and dark text contrasts more with the surface (`color::readable_on`).

Overrides: `Toast::role(Role::…)` uses a status fill instead (text still by contrast),
`Toast::background(color)` hard-codes the surface, `Toast::text_color(color)` hard-codes the text.
`Toast::action(label, msg)` adds the optional call-to-action button.

## Type of message and the action button

`Toast::status(Role::Success | Warning | Error | Accent | Destructive)` marks the type of message,
like a Mantine notification, with a colored 10px dot before the title. The surface stays neutral;
`role` is the separate option that fills it.

## Radius

`Toast::radius(px)` sets this toast's corner radius; otherwise the theme's `Metrics::radius_toast`
applies (default 24, a full pill, since iced caps a radius at half the height; the Theming gallery
page has a slider). It is clamped to `Metrics::MAX_RADIUS_TOAST`.

The call-to-action button (`Toast::action`) is filled with the **accent color** (text chosen by
contrast) so it reads as a button on the dark surface, with the usual hover and pressed fills. On a
toast filled with the accent (`role(Role::Accent)`) it falls back to a text-colored tint.

## Position and target

`Toast::position(ToastPosition::…)` places the toast at `TopLeft`, `TopCenter`, `TopRight`,
`BottomLeft`, `BottomCenter` (default) or `BottomRight`, and it slides in from the edge it is
anchored to. Without the call it follows `[toasts] position` in the desktop config.

The **target** is the container the toast is placed in: the whole **window** (over the sidebar too)
or only the **content area** beside the sidebar. `toast_overlay` fills whatever it wraps, so the
app chooses by wrapping the window or the content pane, and reads the shared setting with
`widget::toast_target()` (`[toasts] target`). Keep both layers mounted
(`toast_overlay(x, None)` for the inactive one) so switching the target does not rebuild the tree.

## Stacked notifications

`toast_stack(content, toasts)` shows any number of toasts at once, in one column at the position of
the first (`[toasts] position` by default), ordered oldest to newest with a small gap. At a bottom
position the newest sits nearest the edge. Each toast has its own entrance, call to action and
dismiss button; the app owns the list (add, dismiss, cap its length). `toast_overlay` is the
single-toast case.
