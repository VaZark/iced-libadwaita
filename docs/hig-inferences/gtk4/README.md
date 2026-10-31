# GTK4 reference

Distilled from the [GTK4 docs](https://docs.gtk.org/gtk4/) (visual index, API 4.0, library 4.24) for
the plain GTK widgets that libadwaita builds on. Adwaita widgets are documented
in [../widgets](../widgets/README.md).

## Precedence: Adwaita wins

libadwaita is built on GTK4 and restyles it. **When GTK4 and Adwaita disagree we
follow Adwaita**:

- Colors, opacities and radii come from libadwaita's CSS variables
  ([../design-language/colors.md](../design-language/colors.md)), not GTK's old
  `@theme_*` names.
- Style classes follow libadwaita (`.dimmed` over `.dim-label`, `.boxed-list`
  over `.content`, `.navigation-sidebar` over `.sidebar`, `.suggested-action`…).
- GTK widgets that libadwaita deprecates or replaces are **mapped to the Adw
  widget** (see the table below), and the GTK file says so.
- GTK4 pages give structure (CSS nodes, states, behavior) but almost never pixel
  sizes. Sizes marked **✓** are verified against libadwaita's stylesheet source;
  **‡** are from memory and unverified.

## Replacement map (Adwaita over GTK4)

| GTK4                                                                    | Prefer (Adwaita)                                                        |
| ----------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| `GtkInfoBar` (deprecated 4.10)                                          | `AdwBanner`                                                             |
| `GtkDialog`, `GtkMessageDialog`, `GtkAboutDialog`, `GtkShortcutsWindow` | `AdwAlertDialog`, `AdwAboutDialog`, `AdwShortcutsDialog`                |
| `GtkNotebook`, `GtkStackSwitcher`, `GtkStackSidebar`                    | `AdwTabBar`, `AdwViewSwitcher`, `AdwSidebar` / `AdwViewSwitcherSidebar` |
| `GtkHeaderBar` + `GtkWindow` titlebar                                   | `AdwHeaderBar` inside `AdwToolbarView` in `AdwWindow` (no titlebar)     |
| `GtkActionBar`                                                          | `AdwToolbarView` bottom bar                                             |
| `GtkListBox` `.content` / `.sidebar`                                    | `.boxed-list`, `.navigation-sidebar`                                    |
| `GtkPaned`                                                              | `AdwNavigationSplitView` / `AdwOverlaySplitView`                        |
| `GtkExpander`                                                           | `AdwExpanderRow`                                                        |
| `.dim-label`, `.title`, `.subtitle`                                     | `.dimmed`, `.title-N`, `.heading`, `.caption`                           |

## Index

Scrolling: [scrollbar](scrollbar.md) · [scrolled-window](scrolled-window.md)
Buttons: [button](button.md) · [toggle-button](toggle-button.md) · [link-button](link-button.md) · [check-button](check-button.md) · [menu-button](menu-button.md) · [switch](switch.md) · [drop-down](drop-down.md)
Entries: [entry](entry.md) · [search-entry](search-entry.md) · [password-entry](password-entry.md) · [spin-button](spin-button.md) · [scale](scale.md) · [text-view](text-view.md)
Display: [label](label.md) · [spinner](spinner.md) · [progress-bar](progress-bar.md) · [level-bar](level-bar.md) · [info-bar](info-bar.md) · [separator](separator.md) · [window-controls](window-controls.md)
Popups: [popover](popover.md) · [popover-menu](popover-menu.md)
Containers: [box](box.md) · [center-box](center-box.md) · [frame](frame.md) · [expander](expander.md) · [overlay](overlay.md) · [stack](stack.md) · [stack-switcher](stack-switcher.md) · [notebook](notebook.md) · [paned](paned.md) · [list-box](list-box.md) · [action-bar](action-bar.md) · [header-bar](header-bar.md) · [search-bar](search-bar.md)
