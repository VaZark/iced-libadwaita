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

Apps should bundle at least one theme `.ron` file to provide a consistent appearance even when no desktop theme is installed. We recommend [`themes/libadwaita.ron`](themes/libadwaita.ron) as the base theme. Load it with `Config::load_with_base_theme(DEFAULT_BASE_THEME)`, where the app's `DEFAULT_BASE_THEME` embeds the file with `include_str!`. iad retains this base when `IAD_THEME` is unset or `none`; without any base or selected theme it uses the bare look. See [app theme config](docs/lib/theme-config.md) for loading and applying theme files.

## What is where

| Path                   | Holds                                                                                                                                        |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/`                 | the library: `layouts/`, `primitives/` (headless rules), `binds/`, `components/`, `metas/`, `widgets/`, `styles/`, `windows/`, theme, config |
| `docs/man/`            | handwritten man sources; built pages go in `target/man/`                                                                                     |
| `docs/lib/`            | the user guide (published with the crate)                                                                                                    |
| `docs/hig-inferences/` | design rationale for contributors (not published)                                                                                            |
| `gallery/`             | the gallery: a separate crate that depends on the library                                                                                    |
| `xtask/`               | repository tasks (`cargo xtask man`)                                                                                                         |

`cargo build` builds only the library.

## Reference

- API: run `cargo xtask man` to build the man pages in `target/man/`. `man -l target/man/iad.7` (overview), `man -l target/man/iad-dock.3` (one component), `man -l target/man/iad-desktop.ron.5` (config).
- Usage: [docs/lib/](docs/lib/README.md).
- Gallery: `cargo run -p iced-adaptive-design-gallery`.
- [Theme manager](theme-manager/README.md): `cargo run -p iced-adaptive-design-theme-manager -- [FILE]`. Installable app: bundled themes, the global theme, section-first token editing; `theme-manager gallery` runs the gallery.
- [Theme toys](theme-toys/README.md): `cargo run -p iced-adaptive-design-theme-toys -- <win95|winxp|win7|mate>`. Four reference targets: Windows Classic/95, XP, Windows 7 and modern MATE; `gnome2` is a compatibility alias only. Each toy has a validated opt-out base; `IAD_THEME` supplies an optional overlay.
- Public [text-trigger and context menus](docs/lib/windows-and-menus.md#menus), and [typed child presentation](docs/lib/child-props.md#contract-and-usage).

## Parameters belong to the library

Every new property goes into the library first: the component, its doc comment, its docs and its gallery playground, in the same change. A parameter the library's own API cannot reach does not exist. The man page is generated from the doc comment: `cargo xtask man`. Checklist: [CONTRIBUTING.md](CONTRIBUTING.md#working-on-a-component).

## More

- [DESIGN.md](DESIGN.md): structure and targets.
- [CONTRIBUTING.md](CONTRIBUTING.md): build, test, work on it.

Icons support installed XDG themes and bundled `iad-icons` artwork, including the editor's Save / Save As actions. Configure `[icons] theme` to select an XDG theme (`system` follows shared desktop toolkit settings, then falls back to `hicolor`). See [system icons and bundled defaults](docs/lib/icons.md).
