# Contributing

## Setup

Stable Rust (edition 2024) and the system libraries iced needs on Linux (Wayland or X11, Vulkan or OpenGL).

```text
cargo build                       # the library only
cargo build --workspace           # library, gallery, xtask
cargo test
cargo clippy --all-targets --workspace
cargo xtask man --check           # man pages are current
```

`cargo test` and `cargo clippy` must be clean before you commit.

## Formatting

Local pre-commit hooks: dprint formats JSON, Markdown and TOML; `cargo fmt --all` formats Rust. Install with `pre-commit install`. Run everything with `pre-commit run --all-files`.

## Repo layout

| Path                   | Holds                                                                                          |
| ---------------------- | ---------------------------------------------------------------------------------------------- |
| `src/`                 | the library, in layers (see DESIGN.md): `layouts/`, `primitives/` (headless rules), `binds/`, `components/`, `metas/`, `widgets/`, `styles/`, `windows/`, theme, config |
| `gallery/`             | the gallery, a separate crate                                                                  |
| `xtask/`               | repository tasks                                                                               |
| `man/`                 | man pages. `iad(7)` and `iad-desktop.conf(5)` are written by hand. `iad-<alias>(3)` and the index `iad-components(3)` are generated |
| `docs/lib/`            | user guide. Published with the crate and `man/`                                                |
| `docs/hig-inferences/` | design rationale. Not published (`include` in `Cargo.toml`)                                    |

## Man pages

`cargo xtask man` reads the `///` and `//!` comments and the public API of `src/layouts`, `src/components`, `src/binds`, `src/metas` and `src/widgets`, and writes one page per component. `--check` fails when a page is stale.

A component is an item exported from the `mod.rs` of one of those layers:

- a public struct that becomes an `Element`, or
- a function that returns an element.

Builder methods on `self` are its properties. A method that takes an element is a slot. `on_*` is an event.

Write the doc comment. The page follows.

## The gallery

`cargo run -p iced-adaptive-design-gallery` is the visual test. Render every page in light and dark with `-- --screenshot DIR` and look at the images.

```text
GALLERY_SIZE=WxH                  window size
GALLERY_DETAIL=textinput          open a nested page
GALLERY_PAGE=inputs               open a group
GALLERY_SEARCH=text               open the search with this text
GALLERY_SHOT_ONLY=Docked          with --screenshot: that page only (the `Detail` name)
GALLERY_SHOT_DELAY_MS=2500        with --screenshot: wait per page (slow debug builds)
GALLERY_DARK=1                    dark scheme for GALLERY_ANIM captures
GALLERY_SIDEBAR_MODE=parent-page
GALLERY_MENU=page|dropdown
GALLERY_ANIM=switch|drawer|reveal|dialog|toast|carousel|page|push|breakpoint
GALLERY_ANIM=keys                 wait 6 s, then capture: send keys meanwhile
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
- A control in the component's playground (`gallery/src/plays.rs`): the `specs` entry, its use in `build`, its line in the generated code.
- A line in its `docs/hig-inferences/widgets/` page.
- If a desktop config key backs the default: `docs/lib/desktop-config.md`, `docs/lib/desktop.conf.example`, `man/iad-desktop.conf.5`.

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
