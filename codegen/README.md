# Build-time XML UI code generation

XML tags and attribute names follow `iced_libadwaita::catalog::COMPONENTS`
(the same vocabulary documented in `docs/components.md`).

```xml
<toolbar_view>
  <header_bar slot="top_bar" title="Settings"/>
  <page>
    <preferences_group title="Appearance">
      <action_row title="Theme" subtitle="System"/>
    </preferences_group>
  </page>
</toolbar_view>
```

The build-time compiler validates component names, attribute names, and child slots
against the **existing** component catalog. It then emits ordinary Rust code.
The catalog describes the public vocabulary, while typed code-generation adapters
are needed for each constructor signature. Components without an adapter are
rejected explicitly. The initial supported set is `toolbar_view`, `header_bar`,
`page`, `preferences_group`, `preferences_page`, and `action_row`.
Only a subset of each component's catalogued attributes has a typed adapter so far.

This is analogous to schema validation, but **not yet a full XSD**: the catalog
currently records property and slot names, not all value types, required
constructor arguments, child cardinality, or event signatures. Those constraints
must be added as typed metadata to make validation complete.

In the application's `Cargo.toml`:

```toml
[build-dependencies]
iced-libadwaita-codegen = { path = "../iced-libadwaita/codegen" }
```

In `build.rs`:

```rust
fn main() {
    iced_libadwaita_codegen::compile_file(
        "ui/main.xml",
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("ui.rs"),
    ).unwrap();
}
```

In the application:

```rust
mod ui {
    include!(concat!(env!("OUT_DIR"), "/ui.rs"));
}
```

The generated `view` function returns an iced-libadwaita element with the
application's `crate::Message` type. No runtime XML parser or dynamic linking
is needed.
