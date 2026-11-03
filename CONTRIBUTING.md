# Contributing

#### Setup

Stable Rust (edition 2024) and the system libraries iced needs on Linux (Wayland or X11, Vulkan or OpenGL). Then:

```text
cargo build --all-targets
cargo run --example gallery
cargo test
cargo clippy --all-targets
```

Both `cargo test` and `cargo clippy` should be clean before committing.

#### Formatting

Formatting is handled by the local pre-commit hooks: dprint formats configured JSON, Markdown, and TOML files, while `cargo fmt --all` formats Rust. Install the hooks with `pre-commit install`; run them across the repository with `pre-commit run --all-files`.

#### The gallery

`cargo run --example gallery` is the visual test. Render every page in light and dark with `-- --screenshot DIR`, and inspect the images; useful switches:

```text
GALLERY_SIZE=WxH        GALLERY_DETAIL=textinput     GALLERY_PAGE=inputs
GALLERY_SHOT_ONLY=treegrouped   # with --screenshot: that one page only
GALLERY_SHOT_DELAY_MS=2500      # with --screenshot: wait longer per page (slow debug builds)
GALLERY_SIDEBAR_MODE=parent-page   GALLERY_MENU=page|dropdown
GALLERY_ANIM=switch|drawer|reveal|dialog|toast|carousel|page|push|breakpoint
GALLERY_DARK=1          # dark scheme for GALLERY_ANIM captures
GALLERY_ANIM=keys       # waits 6s, then captures: send keys meanwhile (`ydotool key 29:1 50:1 50:0 29:0`)
GALLERY_ANIM=prefs      # opens the preferences window; captures the parent, then the window
GALLERY_ANIM=appshell   # opens the AppShell window (GALLERY_SHELL_SIZE=WxH); same capture
GALLERY_ANIM=shell-back|embedded-back  # a page open in the mobile shell (window or embedded in Layout > Resizable), then Back, captured while it slides out
APPSHELL_PIN=mobile|tablet|desktop  APPSHELL_SIDEBAR_MODE=parent-page  APPSHELL_CONTENT=1  APPSHELL_DRAWER=1  APPSHELL_EDGE=right|bottom  # AppShell start-up
ICED_LIBADWAITA_CONFIG=path.conf     # try a desktop config
```

How the gallery is organized, and when a component gets a nested page: [examples/gallery/README.md](examples/gallery/README.md). Screenshots can be stale on slow debug builds; give them time before concluding a page is broken.

#### Layout of the repo

- `src/`: the library (`widget/`, `primitives/` headless logic, `window/` window logic and the dialogs, `style/`, `theme`, `config`, `animation`, …).
- `examples/`: `gallery/` (including `appshell.rs`, the adaptive-layout demo) and `catalog.rs` (prints the component catalog).
- `docs/lib/`: guidance on using the library. `docs/hig-inferences/`: what we target.

#### Working on a component

1. Read its file in `docs/hig-inferences/widgets/` and its _iced mapping_.
2. Give it the shape in [docs/lib/components.md](docs/lib/components.md); add it to `src/catalog.rs` and regenerate the catalog: `cargo run --example catalog > docs/lib/components-catalog.md`.
3. Show its states (including disabled) in the gallery.
4. Record deviations from the reference in its _iced mapping_ section.
5. Anything that wraps a child widget must forward `overlay()`; test popovers inside it.

**Adding a property to an existing component** is not done until it works end to end. The property is added **to the library** (never only to an example or an app; the gallery is a client of it), and every new builder method gets, in the same change:

- its entry in `src/catalog.rs` (`props`, `slots` or `events`), and the catalog regenerated;
- a **control in the component's gallery playground** (`examples/gallery/plays.rs`: the `specs` control, its use in `build`, and its line in the generated code), so the property can be tried and its code read. A property whose default comes from the toolkit or the desktop config is a **plain value that starts at that default** and is overridden by its builder method, not an `Option` with a `None` case; its control shows the default, and the generated code only mentions it once it is changed;
- a line in its `docs/hig-inferences/widgets/` page, and in `docs/lib/desktop-config.md` plus `desktop.conf.example` if a desktop config key backs the default.

A property that is missing from the catalog or the playground is invisible to everyone using the gallery or a declarative layer, so treat that as unfinished work.

#### Docs

Diagrams and charts are [Mermaid](https://mermaid.js.org/) fenced blocks (` ```mermaid `), not ASCII art or images. The GTK CSS-node listings under `docs/hig-inferences/gtk4/` are verbatim text from the GTK documentation and stay as they are.

#### Commits

Small, logical groups; each commit should build. This project is managed primarily by AI; review changes before depending on them.

#### Status

No license has been chosen yet.
