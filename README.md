# iced-libadwaita

A theme and component library for [iced](https://iced.rs) 0.14 that follows the [GNOME Human Interface Guidelines](https://developer.gnome.org/hig/), so an iced app can look and behave like a GNOME app.

We target the HIG, with libadwaita as a reference. It is not a port of libadwaita and not a drop-in replacement.

> **Note.** Built for the author's personal project and managed primarily by AI (Claude). It is independent, contains no libadwaita code, and is not affiliated with GNOME. Expect rough edges and API churn.

```rust
use iced_libadwaita as adw;
use adw::widget as w;

fn view(&self) -> iced::Element<'_, Message, adw::Theme> {
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

Use `adw::Theme` as the application theme. Try `cargo run --example gallery`.

#### Parameters belong to the library

Every component property (a builder method, a desktop-config key behind its default, an override of that default) is added **to the library itself**, never only to an example or an app. The gallery is a client of the library: it shows what exists, and a parameter that cannot be reached through the library's own API does not exist. So whenever a new parameter is added, the same change adds it to the library's component, its catalog entry (`src/catalog.rs`), its docs, and its gallery playground, so it works end to end. The checklist is in [CONTRIBUTING.md](CONTRIBUTING.md#working-on-a-component).

#### Where to go next

- [DESIGN.md](DESIGN.md): how it is put together and what it targets.
- [docs/lib/](docs/lib/README.md): how and when to use each component.
- [CONTRIBUTING.md](CONTRIBUTING.md): building, testing and working on it.
