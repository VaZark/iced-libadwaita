# Contributing

## Setup

Stable Rust (edition 2024) and the system libraries iced needs on Linux (Wayland, Vulkan or OpenGL; X11 is not supported).

```text
cargo build                       # the library only
cargo build --workspace           # library, gallery, theme editor, xtask
cargo test
cargo clippy --all-targets --workspace
cargo xtask man                   # build man pages in target/man/
```

`cargo test` and `cargo clippy` must be clean before you commit.

## Formatting

Local pre-commit hooks: dprint formats JSON, Markdown and TOML; `cargo fmt --all` formats Rust. Install with `pre-commit install`. Run everything with `pre-commit run --all-files`.

## Repo layout

| Path                   | Holds                                                                                          |
| ---------------------- | ---------------------------------------------------------------------------------------------- |
| `src/`                 | the library, in layers (see DESIGN.md): `layouts/`, `primitives/` (headless rules), `binds/`, `components/`, `metas/`, `widgets/`, `styles/`, `windows/`, theme, config |
| `gallery/`             | the gallery, a separate crate                                                                  |
| `theme-editor/`        | the theme editor: edits theme files in place, registers them, tries them in the gallery        |
| `themes/`              | theme files: `libadwaita.conf`, `cosmic.conf`, and a few fun ones                           |
| `xtask/`               | repository tasks                                                                               |
| `docs/man/`            | handwritten overview and config man sources; all built pages go in `target/man/` |
| `docs/lib/`            | user guide. Published with the crate and `docs/man/`                                                |
| `docs/hig-inferences/` | design rationale. Not published (`include` in `Cargo.toml`)                                    |

## Man pages

`cargo xtask man` reads the `///` and `//!` comments and the public API of `src/layouts`, `src/components`, `src/binds`, `src/metas` and `src/widgets`, and writes one page per component to `target/man/`, alongside copies of the handwritten pages in `docs/man/`. These build artifacts are ignored by Git. `--check` checks an existing build for stale or missing pages; it requires `cargo xtask man` first on a clean checkout.

A component is an item exported from the `mod.rs` of one of those layers:

- a public struct that becomes an `Element`, or
- a function that returns an element.

Builder methods on `self` are its properties. A method that takes an element is a slot. `on_*` is an event.

Write the doc comment. The page follows.

## Theme files

The library has no built-in look: with no theme file an app is drawn like HTML without CSS (black on white, square, no shadows). The look is a theme file in the key-file format of [docs/lib/theme-config.md](docs/lib/theme-config.md). `themes/` has the shipped ones. To try one, point `IAD_THEME` at it:

```text
IAD_THEME=themes/libadwaita.conf cargo run -p iced-adaptive-design-gallery
IAD_THEME=themes/cosmic.conf      cargo run -p iced-adaptive-design-gallery
IAD_THEME=none                   cargo run -p iced-adaptive-design-gallery   # the bare look
```

Without `IAD_THEME`, the first `default.conf` found in `$XDG_DATA_HOME/iad/themes/` (`~/.local/share/iad/themes/`), then in `iad/themes/` of each `$XDG_DATA_DIRS` entry, applies. To make a look your default on this machine:

```text
mkdir -p ~/.local/share/iad/themes
ln -s "$PWD/themes/cosmic.conf" ~/.local/share/iad/themes/default.conf
```

An unreadable `IAD_THEME` path prints a warning and falls through to those directories. An app that opts out of inheriting (`Inheritance::OptOut`) ignores all of this and uses only its own look.

Edit theme files in the theme editor: `cargo run -p iced-adaptive-design-theme-editor [FILE]`. It has a control for each token by its type. Its Themes list shows real files only: the **registered** ones (the `*.conf` in `$XDG_DATA_HOME/iad/themes/` and the other data dirs, with the target of each symlink and a `default` tag on the one `default.conf` points at), the folders you add, and the recent files; click one to load it. **Save** and **Save as** write the original file (created if missing, written through a symlink); nothing else is ever written. A ● before a name, in the window title and in the status line means unsaved changes. **Try it out** writes the current values, saved or not, to a separate temp file and starts the gallery with `IAD_THEME` pointing at it; press it again to restart the gallery with new values. **Register** makes a symlink `<name>.conf` in the data dir so every app and this list see the file (**Unregister** removes only that symlink), and **Set as default** points `default.conf` at it, the file apps use without `IAD_THEME`. The editor remembers the open file, folders, recent files, section and variant in `~/.config/iad-theme-editor/state.conf`. A new token goes in `Schema::toolkit()` (`src/theme_config.rs`) with a type and range: prefer typed numbers (`Length`, `Percentage`) and enum strings over new Rust enums or switches. Add its value to `themes/libadwaita.conf` so that look stays complete, and check that the gallery with `IAD_THEME=themes/libadwaita.conf` is pixel-identical to before.

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
IAD_THEME=path.conf               draw with this theme file (IAD_THEME=none: the bare look)
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

1. Read its page in `docs/hig-inferences/widgets/` and its **iced mapping**.
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
