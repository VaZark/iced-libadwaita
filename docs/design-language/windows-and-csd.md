# Windows and client-side decorations

Source: [`AdwWindow`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Window.html), [`AdwHeaderBar`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.HeaderBar.html), [CSS variables](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/css-variables.html).

* **Adwaita dislikes server-side decorations (SSD).** `AdwWindow` has *no*
  titlebar area; the app supplies an `AdwHeaderBar` in an `AdwToolbarView`, and
  that bar carries the title, the window controls (min/max/close per the system
  `decoration-layout`) and acts as the drag handle.
* **Project decision: SSD is disabled by default.** Use
  `iced_libadwaita::window::settings()`; it sets `decorations: false`,
  `transparent: true` and the 360×200 minimum size. `settings_with_ssd(true)` is
  the explicit opt-out.
* Window corners use `--window-radius` = 15px for **floating** windows; maximized, tiled and fullscreen windows go **edge to edge** (square corners, no border, no resize edges).
* `AdwHeaderBar` in a dialog shows the dialog title and only a close button.
* In split views, only the bars at the window edge show window controls.
* Drag: header bar background is a window handle (press → drag; double click →
  toggle maximize). Without SSD the app must also provide resize edges.
* `AdwWindow:content` replaces `titlebar`/`child`.

## Window buttons follow the desktop

* **Which buttons, which side:** `org.gnome.desktop.wm.preferences button-layout`
  (`appmenu:close` is GNOME's default: a lone close button at the end;
  `close,minimize,maximize:` puts three on the start side). `DecorationLayout::detect()`
  reads it with `gsettings` (cached) and falls back to GNOME's default. A button
  appears only if the layout lists it *and* the app supplies its action.
* **Which icons:** the user's icon theme (`org.gnome.desktop.interface icon-theme`):
  `window-close-symbolic`, `window-minimize-symbolic`, `window-maximize-symbolic`,
  `window-restore-symbolic` are looked up through the theme and its `Inherits`
  chain (then Adwaita, hicolor); built-in drawings are the fallback.
* **Split views:** each pane's header shows only the buttons at its own window edge
  (`show-start-title-buttons` / `show-end-title-buttons`).
* **Dialogs:** the header shows the dialog title and only a close button, on the
  side the system puts it (end by default). The header is **flat** (no color of its
  own), so it flows into the dialog body like Nautilus's dialogs; the dialog's
  rounded background provides the corners.
* **Mobile mode** (`window::is_mobile`, width ≤ 450): decorations are hidden
  completely and menus become full pages. Corners are **not** forced: they follow
  the real window state, so a narrow *floating* window keeps its rounded corners
  and resize grips, and only a maximized/fullscreen one goes edge to edge. 450
  matches the `AdwDialog` bottom-sheet threshold.
* Not covered: non-GNOME desktops (KDE, etc.) are not read; they get GNOME's
  defaults and the Adwaita/hicolor icon fallback.

## iced mapping
`window::settings()` (SSD off by default), `window::settings_with_ssd(bool)`,
`HeaderBar::{on_drag, on_double_click, window_controls}`, and
`window::frame(content, state, on_resize)` which adds the rounded window background and
invisible resize grips (`iced::window::drag_resize`). `window::State` (maximized/fullscreen) comes from
`window::changes()` + `window::refresh()`; when edge to edge the frame is square, borderless and has no grips.
The theme must be transparent behind the root (`Theme::with_transparent_window(true)`, the default).
iced cannot clip to a rounded rectangle, so `HeaderBar::window_radius(r)` and
`navigation_sidebar_rounded(rows, r)` round the surfaces that touch the corners. The gallery draws its own
decorations by default; `GALLERY_SSD=1` opts back in.
