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

* Track the window state with `window::changes()` + `refresh()` so corners are rounded only
  while floating.
* Which buttons appear, and where, follows GNOME's `button-layout`; their icons follow the icon
  theme. In mobile mode they are hidden.
* Anything that touches a window corner takes the window radius (`NavigationSplitView::window_radius`,
  `dialog_in_window`, `bottom_sheet_in_window`, `HeaderBar::window_corners`).
* In a split view give each pane its own header bar; only the pane at a window edge shows the
  window buttons.

#### Header bars

Header bars are continuous: no color of their own, they take the pane behind. A header holds the
title (or a `title_widget` such as `path_bar`) and menu items, not page content. Put the primary
menu at the end of the sidebar header.

#### Menus

* `primary_menu(Menu::new(..).section([..]))` is the app menu. Order: setting toggles, then
  Preferences, Keyboard Shortcuts, Help, About. 3-12 items.
* Present it as a popover on desktop and a full page when narrow (`MenuPresentation::resolve`).
* Disable unavailable items with `MenuItem::disabled()`.

#### Toasts

Place them with `Toast::position` (default from `[toasts] position`) in the whole window or the
content area (`[toasts] target`): wrap the window or the content pane with `toast_stack`, and keep
both layers mounted.
