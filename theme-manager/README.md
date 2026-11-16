# Theme manager

Chooses the **global theme** every iced-adaptive-design app picks up, ships the default set of
themes, and edits theme files with a control for each token by its type.

```sh
cargo run -p iced-adaptive-design-theme-manager -- [FILE]
cargo run -p iced-adaptive-design-theme-manager -- gallery [ARGS]   # the gallery, from the command line
```

## Install

```sh
cargo build --release -p iced-adaptive-design-theme-manager
theme-manager/scripts/install.sh [--prefix PREFIX]     # default ~/.local
theme-manager/scripts/uninstall.sh [--prefix PREFIX]
```

Installs `bin/theme-manager`, the icon, `org.iad.ThemeEditor.desktop` and the AppStream metadata.
The bundled themes are inside the binary.

### Flatpak

```sh
theme-manager/scripts/flatpak-build.sh [--install] [--run]
```

Builds `org.iad.ThemeEditor` with `flatpak-builder` into `build-dir/theme-manager` (ignored by git);
`--install` installs it for your user, `--run` also starts it. You need `flatpak-builder`, the
Freedesktop 26.08 platform and SDK and the Rust extension (the script's header has the install
line), and network access, since the build downloads the Rust dependencies. The sandbox gets
Wayland, the GPU, `xdg-data/iad` (where the global theme lives, so host apps see it) and a read-only
`xdg-config/iced-adaptive-design`; its launcher points `XDG_DATA_HOME` at the host's.

## The global theme

Every theme is a file of its own. The global theme is `default.ron` in
`$XDG_DATA_HOME/iad/themes/` (default `~/.local/share/iad/themes/`), the file every iad app already
looks for (`theme_config::theme_path`), and it is always a **symlink** to the current theme's file.
There is one action:

- **Set as global theme** (the globe button in the header, and in the theme dialog) points
  that symlink at the open theme's file, creating or replacing it atomically. Unsaved changes are
  saved first; a theme with no file yet (new, or edited from a bundled one) opens Save as, which
  starts in that directory, and becomes global once written. A bundled theme is copied there as
  `<key>.ron` first (an existing file of that name with other values is never overwritten).
- A regular file called `default.ron` is never overwritten: it is renamed to a theme file of its
  own (from its `[theme] name`) before the link is made.
- **Remove global theme** (same dialog) deletes the symlink, never the theme it led to; apps fall
  back to their own base theme.
- `IAD_THEME=path` or `IAD_THEME=none` still override per process, as before.

## Bundled themes

`libadwaita`, `cosmic`, `bubblegum`, `solarized`, `synthwave`, `terminal`, embedded from
`themes/*.ron`. The theme picker lists them under **Bundled** (a bundled theme already installed
unchanged shows once, as the installed file). Opening one gives an unsaved copy: **Save as**
writes your own. The toy reference themes (`win95`, `winxp`, `win7`, `mate`, `gnome2`) belong to
`theme-toys` and are not bundled.

## Editing

The sidebar selects token sections. **All** groups every token on one searchable page. **Modified
only** filters unsaved changes; **Overrides only** shows explicit entries in the active Light/Dark
scheme, excluding inherited Base values (unavailable for Base). **Copy from Base** fills a scheme
from Base. Ctrl+Z / Ctrl+Shift+Z undo and redo.

The theme in the header opens the picker (installed themes, the global one marked, bundled themes, added folders, recent
files; long names wrap). The header has icon buttons for **Undo**, **Reset** (drops every unsaved change; undoable), **Save**, **Set as global theme** (a check once it is) and the menu. **Save** updates the file, following symlinks. **Save as** writes a copy and continues editing it. Validation and unsaved-change confirmation stay.

The **play** button in the header (and the menu) starts the gallery with the values being edited,
unsaved edits included, through `IAD_THEME` pointing at a temp file; the open file is never touched
and pressing it again restarts the gallery. `theme-manager gallery` (or the `gallery` binary) runs
it with the global theme. The gallery has no desktop entry. Theme toys are started from
`theme-toys`, not from here.

State (open file, folders, recents, section, variant) is kept in
`$XDG_CONFIG_HOME/iad-theme-editor/state.ron`.

## The token page

The token page is virtualized (`src/page.rs`): only the groups near the scroll position are built. Every row and heading is a box of a known height, and the window is at least 560px wide, so the heights are computed and tested rather than measured.

## Validation

`cargo test -p iced-adaptive-design-theme-manager --offline` covers files, saving and
filtering, undo, the bundled set (valid, matches `themes/`, no toys), setting and removing the
global theme (a symlink, replaced atomically; a regular default.ron is kept), starting the gallery, and headless layout of long
multilingual names at narrow widths. Layout assertions are not pixel-perfect visual acceptance.
