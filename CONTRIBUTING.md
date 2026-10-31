# Contributing

#### Setup

Stable Rust (edition 2024) and the system libraries iced needs on Linux (Wayland or X11, Vulkan
or OpenGL). Then:

```text
cargo build --all-targets
cargo run --example gallery
cargo test
cargo clippy --all-targets
```

Both `cargo test` and `cargo clippy` should be clean before committing.

#### The gallery

`cargo run --example gallery` is the visual test. Render every page in light and dark with
`-- --screenshot DIR`, and inspect the images; useful switches:

```text
GALLERY_SIZE=WxH        GALLERY_DETAIL=textinput     GALLERY_PAGE=inputs
GALLERY_SIDEBAR_MODE=first-page   GALLERY_MENU=page|dropdown
GALLERY_ANIM=switch|drawer|reveal|dialog|toast|carousel|page|push   
ICED_LIBADWAITA_CONFIG=path.conf     # try a desktop config
```

How the gallery is organized, and when a component gets a nested page: [examples/gallery/README.md](examples/gallery/README.md).
Screenshots can be stale on slow debug builds; give them time before concluding a page is broken.

#### Layout of the repo

* `src/`: the library (`widget/`, `style/`, `theme`, `config`, `animation`, …).
* `examples/`: `gallery/` and `catalog.rs` (prints the component catalog).
* `docs/lib/`: guidance on using the library. `docs/hig-inferences/`: what we target.

#### Working on a component

1. Read its file in `docs/hig-inferences/widgets/` and its *iced mapping*.
2. Give it the shape in [docs/lib/components.md](docs/lib/components.md); add it to
   `src/catalog.rs` and regenerate the catalog: `cargo run --example catalog > docs/lib/components-catalog.md`.
3. Show its states (including disabled) in the gallery.
4. Record deviations from the reference in its *iced mapping* section.
5. Anything that wraps a child widget must forward `overlay()`; test popovers inside it.

#### Docs

Diagrams and charts are [Mermaid](https://mermaid.js.org/) fenced blocks (` ```mermaid `), not ASCII
art or images. The GTK CSS-node listings under `docs/hig-inferences/gtk4/` are verbatim text from
the GTK documentation and stay as they are.

#### Commits

Small, logical groups; each commit should build. This project is managed primarily by AI; review
changes before depending on them.

#### Status

No license has been chosen yet.
