# Build-time XML UI code generation

This optional crate generates normal Rust view code. It is not a runtime dependency.
XML uses `<interface><element type="..."><property name="...">value</property></element></interface>`.

Supported initial mappings: Text, Row, Column, Button, PreferencesPage, PreferencesGroup, ActionRow.
Unsupported components, properties and malformed XML fail the build. More catalog mappings can be added incrementally.

In the application's `build.rs`:

```rust
fn main() {
    iced_libadwaita_codegen::compile_file(
        "ui/main.xml",
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("ui.rs"),
    ).unwrap();
}
```

In application code, include the generated module and invoke its `view` function.
The generated function uses `Message` paths from the XML; message types and callbacks
are checked by Rust during compilation. The generated module's message type is generic,
so messages referenced in the XML must be in scope at its inclusion site.

```rust
mod ui {
    use super::Message;
    include!(concat!(env!("OUT_DIR"), "/ui.rs"));
}
```

The generated UI uses ordinary iced and iced-libadwaita constructors. There is no
separate runtime or dynamic linking.
