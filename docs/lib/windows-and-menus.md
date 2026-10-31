# Windows, header bars, menus

#### Decorations

GNOME apps draw their own title bar (client-side decorations). Server-side decorations are off by
default.

```rust
iced::application(boot, update, view).window(adw::window::settings()).run()

// view: the header bar carries title, controls and drag; `frame` adds the rounded corners.
let corner = state.radius(&adw::Metrics::default()); // 0 when maximized or fullscreen
adw::window::frame(
    w::toolbar_view(content).top_bar(
        w::header_bar().title("My App").window_radius(corner)
            .on_drag(Msg::Drag).on_double_click(Msg::ToggleMaximize)
            .window_controls(w::WindowControls { on_minimize: Some(Msg::Min), on_maximize: Some(Msg::Max), on_close: Some(Msg::Close) }),
    ),
    state,
    Msg::Resize,
)
```

- Track the window state with `window::changes()` + `refresh()` so corners are rounded only
  while floating.
- Which buttons appear, and where, follows GNOME's `button-layout`; their icons follow the icon
  theme. In mobile mode they are hidden.
- Anything that touches a window corner takes the window radius (`NavigationSplitView::window_radius`,
  `dialog_in_window`, `bottom_sheet_in_window`, `HeaderBar::window_corners`).
- In a split view give each pane its own header bar; only the pane at a window edge shows the
  window buttons.

#### Dialogs in their own window

A dialog that is large or long-lived (preferences) can be a window of its own that the user moves
independently of its parent, while the parent is disabled.

- Open it with `window::open(adw::window::dialog_settings(size))` (client-side decorations,
  resizable, centered, never narrower than a phone) from an `iced::daemon` app, and give it its
  own view: a header bar with only a close button and `adw::window::frame` for corners and grips.
  Header drags and grips need the window's id (`window::drag(id)`, `window::drag_resize(id, d)`).
- Disable the parent while it is open by covering the parent's view with
  `dialog_in_window(root, space(), Some(Msg::FocusDialog), corner)`: an empty dialog is a scrim
  that swallows input, and a click on it can `window::gain_focus` the dialog.
- Handle `window::close_events()` to forget the window when the user closes it, and quit when the
  main window closes.

Limits: iced has no transient-parent or system-modal window, so the compositor does not keep the
dialog above its parent or center it on the parent, and Wayland ignores requested positions. Use
this for preferences-style windows; short questions stay in-window (`AlertDialog`).

#### Header bars

Header bars are continuous: no color of their own, they take the pane behind. A header holds the
title (or a `title_widget` such as `path_bar`) and menu items, not page content. Put the primary
menu at the end of the sidebar header.

#### Menus

- `primary_menu(Menu::new(..).section([..]))` is the app menu. Order: setting toggles, then
  Preferences, Keyboard Shortcuts, Help, About. 3-12 items.
- Present it as a popover on desktop and a full page when narrow (`MenuPresentation::resolve`).
- Disable unavailable items with `MenuItem::disabled()`.

#### Shortcuts

The toolkit handles keyboard shortcuts itself; the app needs no subscription and no `update` arm
for them.

- A menu (`MenuButton`, `primary_menu`, `split_button`) **registers the accelerators it shows**:
  `MenuItem::action("Preferences", Msg::Prefs).accel("Ctrl+,")` works while the menu button is in
  the view, even with its popover closed or its sidebar hidden.
- A widget only **exposes** a shortcut it can honor: the accelerator text is shown only when the
  item is enabled, the accelerator is valid, shortcuts are on, and a keyboard is likely (hidden in
  the touch full-page menu under `[shortcuts] hints=auto`).
- `shortcut::Standard` lists GNOME's usual ones; set the messages the app supports:
  `Standard::new().quit(Msg::Quit).fullscreen(Msg::Fullscreen).into_shortcuts().bind("Esc", Msg::Escape).scope(window)`.
  Wrap the window once; the messages arrive in `update` like any other.
- A focused widget has precedence: a text input or an open popover that handles the key stops the
  shortcut. Plain-character shortcuts never fire while typing.
- Accelerators are written as the character the user types and are **layout independent**. A key
  press is read several ways, best first, and the best-ranked binding wins (then the earliest):
  1. the key as typed, with Shift counted (letters, named keys, an explicit `Ctrl+Shift+,`);
  2. the character it produces, with Shift consumed: `Ctrl+?` fires for Ctrl+Shift+`/` on US and
     Ctrl+Shift+`,` on AZERTY, and `Ctrl+,` then does not also fire; `Ctrl+1` fires for AZERTY's
     Shift+`&`;
  3. the Latin letter or digit at the physical key position: `Ctrl+Q` works on a Cyrillic layout,
     and `Ctrl+1` on AZERTY's unshifted number row.
     So a binding for exactly what was typed beats a position fallback, whatever the order.

| Action             | Accelerator |
| ------------------ | ----------- |
| Quit               | `Ctrl+Q`    |
| Close window       | `Ctrl+W`    |
| Fullscreen         | `F11`       |
| Preferences        | `Ctrl+,`    |
| Help               | `F1`        |
| Keyboard shortcuts | `Ctrl+?`    |

#### Toasts

Place them with `Toast::position` (default from `[toasts] position`) in the whole window or the
content area (`[toasts] target`): wrap the window or the content pane with `toast_stack`, and keep
both layers mounted.
