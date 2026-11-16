# Contributing

## Setup

Stable Rust (edition 2024) and the system libraries iced needs on Linux (Wayland, Vulkan or OpenGL; X11 is not supported).

```text
cargo build                       # the library only
cargo build --workspace           # library, gallery, theme manager, toys, workspace tools
cargo test
cargo clippy --all-targets --workspace
cargo xtask man                   # build man pages in target/man/
```

`cargo test` and `cargo clippy` must be clean before you commit.

## Updating iced

Every iced-family dependency update, including patch and lockfile-only changes, requires the [limitation register's upgrade checks](ICED_LIMITATIONS.md#required-iced-update-procedure). Rerun targeted regressions, workspace checks and the listed live gallery/toy scenarios. Record exact locked versions, actual renderer/environment, per-entry results and evidence in `ICED_LIMITATIONS.md`; explicitly carry blocked or unperformed checks as pending. Compilation is not visual or interaction acceptance, and release notes alone do not justify removing a workaround.

## Formatting

Local pre-commit hooks: dprint formats JSON, Markdown and TOML; `cargo fmt --all` formats Rust. Install with `pre-commit install`. Run everything with `pre-commit run --all-files`.

## Repo layout

| Path                   | Holds                                                                                                                                                                   |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/`                 | the library, in layers (see DESIGN.md): `layouts/`, `primitives/` (headless rules), `binds/`, `components/`, `metas/`, `widgets/`, `styles/`, `windows/`, theme, config |
| `gallery/`             | the gallery, a separate crate                                                                                                                                           |
| `theme-manager/`       | installable app: bundled themes, the global theme, section-first token editor; `theme-manager gallery` runs the gallery                                                                               |
| `theme-toys/`          | four reference scenes: `win95`, `winxp`, `win7`, `mate`; independent validated bases and optional overlays                                                              |
| `themes/`              | toolkit theme files: libadwaita, COSMIC, `win95`, `winxp`, `win7`, `mate` (`gnome2` compatibility alias) and additional palettes                                        |
| `xtask/`               | repository tasks                                                                                                                                                        |
| `docs/man/`            | handwritten overview and config man sources; all built pages go in `target/man/`                                                                                        |
| `docs/lib/`            | user guide. Published with the crate and `docs/man/`                                                                                                                    |
| `docs/hig-inferences/` | design rationale. Not published (`include` in `Cargo.toml`)                                                                                                             |

## Man pages

`cargo xtask man` reads the `///` and `//!` comments and the public API of `src/layouts`, `src/components`, `src/binds`, `src/metas` and `src/widgets`, and writes one page per component to `target/man/`, alongside copies of the handwritten pages in `docs/man/`. These build artifacts are ignored by Git. `--check` checks an existing build for stale or missing pages; it requires `cargo xtask man` first on a clean checkout.

A component is an item exported from the `mod.rs` of one of those layers:

- a public struct that becomes an `Element`, or
- a function that returns an element.

Builder methods on `self` are its properties. A method that takes an element is a slot. `on_*` is an event.

Write the doc comment. The page follows.

## Theme files

The library has no built-in look: without a bundled base or selected theme file an app is drawn like HTML without CSS (black on white, square, no shadows). Apps can supply a base using `Config::load_with_base_theme` or `ThemeConfig::builder(..).base_theme(..)`; `IAD_THEME=none` skips external discovery but retains that base. The gallery bundles `libadwaita.conf`. The look is a theme file in the key-file format of [docs/lib/theme-config.md](docs/lib/theme-config.md). `themes/` has the shipped ones. To try one, point `IAD_THEME` at it:

```text
IAD_THEME=themes/libadwaita.conf cargo run -p iced-adaptive-design-gallery
IAD_THEME=themes/cosmic.conf      cargo run -p iced-adaptive-design-gallery
IAD_THEME=none                   cargo run -p iced-adaptive-design-gallery   # bundled libadwaita base, no external overlay
```

Without `IAD_THEME`, the first `default.conf` found in `$XDG_DATA_HOME/iad/themes/` (`~/.local/share/iad/themes/`), then in `iad/themes/` of each `$XDG_DATA_DIRS` entry, applies. To make a look your default on this machine:

```text
mkdir -p ~/.local/share/iad/themes
ln -s "$PWD/themes/cosmic.conf" ~/.local/share/iad/themes/default.conf
```

For the standard theme loader, an unreadable `IAD_THEME` path prints a warning and falls through to those directories. `Inheritance::OptOut` skips automatic theme discovery; apps can still supply explicit validated theme layers. The toys use this for their own bundled bases plus an optional `IAD_THEME` overlay; see [theme toys](theme-toys/README.md#theme-resolution).

### Theme manager

Run `cargo run -p iced-adaptive-design-theme-manager -- [FILE]`.

- The **primary sidebar selects token sections**. **All** groups every token on one searchable page. The selected theme in the header opens a **picker dialog** with registered files, added folders and recent files; long names and paths wrap within bounded widths.
- **Modified only** filters unsaved changes. **Overrides only** shows explicit entries in the active Light/Dark scheme, not inherited Base values; it is unavailable for Base. Typed controls edit values, **Reset** removes an explicit entry, and Light/Dark edits write `[light:*]` / `[dark:*]` sections. **Copy from Base** fills the active scheme from Base. **Ctrl+Z** and **Ctrl+Shift+Z** undo and redo.
- **Save** updates the current file, following symlinks; an untitled file opens Save as. **Save as** writes a copy, makes it the current file for subsequent saves, and auto-registers it in `$XDG_DATA_HOME/iad/themes/` (normally with a `<name>.conf` symlink, unnecessary if already registered or saved there). A registration failure is reported separately: the saved copy remains current. **Register for all apps** and **Unregister** are in the main menu.
- There is **no embedded preview**: the header's play button starts the gallery with the current values (a temp `IAD_THEME` file, nothing saved); `theme-manager gallery` runs it with the global theme. **Use globally** links `default.conf` in `$XDG_DATA_HOME/iad/themes/` to the open theme (bundled themes are installed there first); **Remove global theme** in the same dialog removes it. Six themes are bundled (`libadwaita`, `cosmic`, `bubblegum`, `solarized`, `synthwave`, `terminal`); toy themes are not.

The editor remembers the open file, folders, recents, section and variant in `~/.config/iad-theme-editor/state.conf`. See the [theme manager guide](theme-manager/README.md) and [toy resolution and validation limits](theme-toys/README.md).

A new toolkit token goes in `Schema::toolkit()` (`src/theme_config.rs`) with a type and bounds. Prefer typed numbers (`Length`, `Percentage`) and enum strings. Add its intended libadwaita value to `themes/libadwaita.conf`, run schema tests, then inspect relevant gallery states; compilation alone does not establish visual acceptance.

## The gallery

`cargo run -p iced-adaptive-design-gallery` is the visual test. Render every page in light and dark with `-- --screenshot DIR` and look at the images.

```text
GALLERY_SIZE=WxH                  window size
GALLERY_DETAIL=textinput          open a nested page
GALLERY_PAGE=inputs               open a group
GALLERY_SEARCH=text               open the search with this text
GALLERY_SHOT_ONLY=Docked          with --screenshot: that page only (the `Detail` name)
GALLERY_SHOT_DELAY_MS=2500        with --screenshot: wait per page (slow debug builds)
GALLERY_SIDEBAR_MODE=parent-page
GALLERY_MENU=page|dropdown
GALLERY_ANIM=switch|drawer|reveal|dialog|toast|carousel|page|push|breakpoint
GALLERY_ANIM=keys                 wait 6 s, then capture: send keys meanwhile
GALLERY_ANIM=theme                capture system appearance, app override, then system again
IAD_THEME=path.conf               draw with this theme file (IAD_THEME=none: bundled base only)
GALLERY_ANIM=appshell             open the AppShell window (GALLERY_SHELL_SIZE=WxH)
GALLERY_ANIM=shell-back|embedded-back
APPSHELL_PIN=mobile|tablet|desktop
APPSHELL_SIDEBAR_MODE=parent-page
APPSHELL_CONTENT=1  APPSHELL_DRAWER=1  APPSHELL_EDGE=right|bottom
ICED_ADAPTIVE_DESIGN_CONFIG=path.conf  try a desktop config
ICED_ADAPTIVE_DESIGN_SCREENSHOT=f.png  an app using iad::dev saves a screenshot to f.png
ICED_ADAPTIVE_DESIGN_SHOT_DELAY_MS=2500 ... after this delay (default 1500)
ICED_ADAPTIVE_DESIGN_HOLD_ANIMATIONS=0.5 hold every animation half way, for a known frame
```

`gallery --appshell` runs only the AppShell, in its own process. The gallery's "Open" on that page starts it that way.

Screenshots can be stale on slow debug builds. Wait before calling a page broken. Gallery organization: [gallery/README.md](gallery/README.md).

## Working on a component

1. Read [ICED_LIMITATIONS.md](ICED_LIMITATIONS.md), then its page in `docs/hig-inferences/widgets/` and its **iced mapping**. Reuse documented workarounds and record newly discovered iced constraints or integration traps in the register.
2. Give it the shape in [docs/lib/components.md](docs/lib/components.md).
3. Write its doc comments, then `cargo xtask man`.
4. Show its states, including disabled, in the gallery.
5. Record deviations from the reference in its **iced mapping**.
6. A wrapper must forward `overlay()`. Test popovers inside it.

### Adding a property

Not done until it works end to end. Add it to the library, never only to an example. In the same change:

- A `///` comment on the method, then `cargo xtask man`.
- A control in the component's playground (`gallery/src/plays.rs`): the `specs` entry and its use in `build`.
- A line in its `docs/hig-inferences/widgets/` page.
- If a desktop config key backs the default: `docs/lib/desktop-config.md`, `docs/lib/desktop.conf.example`, `docs/man/iad-desktop.conf.5`.

A property whose default comes from the toolkit or the config is a plain value that starts at that default. The builder overrides it. No `Option` with a `None` case. The playground control shows the default, and the generated code mentions the call only once it changes.

A setting that is not on by default is opted into explicitly (surface tint, for example).

Use one parameter when two booleans exclude each other. `NumberInput::drag(NumberDrag)`, not two flags.

## Docs

- Diagrams are [Mermaid](https://mermaid.js.org/) blocks. No ASCII art, no images.
- The GTK CSS-node listings in `docs/hig-inferences/gtk4/` are verbatim from GTK and stay as they are.
- `docs/lib/` is the user guide. It says when and how to use things and points to the man pages for the API. Do not duplicate API lists there.
- State the current design. Do not write notes about what replaced what.

## Commits

Small logical groups. Each commit builds. The project is managed mostly by AI: review changes before relying on them.

## Status

No license chosen yet.

The gallery follows shared desktop appearance by default. Its header Light/Dark toggle overrides the scheme for this app; “Use System Appearance” in the primary menu resumes following it.
