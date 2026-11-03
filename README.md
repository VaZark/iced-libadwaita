# iced-adaptive-design (`iad`)

An adaptive design system for [iced](https://iced.rs) 0.14: a theme, components and behavior primitives that follow the [GNOME HIG](https://developer.gnome.org/hig/). An iced app looks and behaves like a GNOME app.

Import it as `iad`: `use iced_adaptive_design as iad;`.

The HIG is the target and libadwaita the reference. It is not a port of libadwaita and not a drop-in replacement.

> Built for the author's own project and managed mostly by AI (Claude). Independent, no libadwaita code, not affiliated with GNOME. Expect rough edges and API changes.

```rust
use iced_adaptive_design as iad;
use iad::widget as w;

fn view(&self) -> iced::Element<'_, Message, iad::Theme> {
    w::toolbar_view(
        w::PreferencesPage::new().push(
            w::PreferencesGroup::new()
                .title("General")
                .push(w::SwitchRow::new("Dark Style", self.dark, Message::Dark))
                .push(w::ActionRow::new("About").chevron(true).on_activate(Message::About)),
        ),
    )
    .top_bar(w::header_bar().title("My App"))
    .into()
}
```

Use `iad::Theme` as the application theme: `iad::Theme::from_config(&iad::config::Config::load())`.

## What is where

| Path                | Holds                                                                           |
| ------------------- | ------------------------------------------------------------------------------- |
| `src/`              | the library: `widget/`, `primitives/` (headless logic), `window/`, `style/`, theme, config |
| `man/`              | man pages: overview, desktop config, one per component                          |
| `docs/lib/`         | the user guide (published with the crate)                                       |
| `docs/hig-inferences/` | design rationale for contributors (not published)                            |
| `gallery/`          | the gallery: a separate crate that depends on the library                       |
| `xtask/`            | repository tasks (`cargo xtask man`)                                            |

`cargo build` builds only the library.

## Reference

- API: man pages. `man -l man/iad.7` (overview), `man -l man/iad-dock.3` (one component), `man -l man/iad-desktop.conf.5` (config).
- Usage: [docs/lib/](docs/lib/README.md).
- Gallery: `cargo run -p iced-adaptive-design-gallery`.

## Parameters belong to the library

Every new property goes into the library first: the component, its doc comment, its docs and its gallery playground, in the same change. A parameter the library's own API cannot reach does not exist. The man page is generated from the doc comment: `cargo xtask man`. Checklist: [CONTRIBUTING.md](CONTRIBUTING.md#working-on-a-component).

## More

- [DESIGN.md](DESIGN.md): structure and targets.
- [CONTRIBUTING.md](CONTRIBUTING.md): build, test, work on it.
