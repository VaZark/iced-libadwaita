# Components

Every UI element is a component with the same shape. Layouts can be written in code, or generated from a description (XML, JSX-like, a macro) that maps tags to components.

This page is the guide. The API of each component (constructors, properties, slots, events) is its man page: `iad-<alias>(3)`, for example `man -l man/iad-dock.3`. `iad-components(3)` is the index. `iced_adaptive_design::prelude::*` imports everything.

## Conventions

1. **Construction.** `Component::new(required…)` and/or a free function of the same name: `action_row(title)`. Required arguments are the minimum to render.
2. **Properties** are chained builder methods named after libadwaita's: `.subtitle(..)`, `.revealed(..)`, `.sidebar_width(..)`. Booleans take a value: `.flat(true)`. `.metrics(Metrics)` is accepted wherever sizes matter.
3. **Slots** are builder methods that take children: `.push(child)`, `.child(..)`, `.prefix(..)`, `.suffix(..)`, `.pack_start(..)`, `.top_bar(..)`, `.content(..)`.
4. **Events** are `on_*` callbacks that return the app's message: `on_press`, `on_toggle(bool)`, `on_select(T)`, `on_input(String)`, `on_close`, `on_outside`.
5. **Controlled state.** The app owns `active`, `expanded`, `value`, `text`, `open`, … and updates it from the events. Visual state (animation progress, hover, an open popover) lives in the component.
6. **Conversion.** Every component implements `Into<Element<'a, Message>>`.
7. **Theme-driven.** Colors, radii, shadows and motion come from the `Theme` and the desktop [config](desktop-config.md), never from the call site.
8. **No raw widgets in app code.** Entries, check and radio buttons, scales, text views, tags, swatches, cards, pages, rows all have components. Only layout primitives remain (`row`, `column`, `container`, `text`, `space`, `responsive`), re-exported by the prelude.

### Parameters

- A parameter whose default comes from the toolkit or the config is a plain value that starts at the default. The builder overrides it. It is not an `Option`.
- A setting that is not on by default is opted into: surface tint, for example.
- Two booleans that exclude each other are one parameter with an enum: `NumberInput::drag(NumberDrag)`, `Dock::mode(DockMode)`.
- Overlays have the fade on and `close_on_outside` true by default.

### Number input

```rust
NumberInput::new(&self.text, Msg::Text)   // the app keeps only the text
    .decimals(2)                           // fixed point, never float
    .min(0).max(100)
    .step(Decimal::new(25, 2))             // 0.25
```

- One message: `on_input(text)`, for typing, the handles and dragging.
- Read the number with `Numeric::parse(&text)`.
- Dragging on the input changes the value: up raises it (`NumberDrag::Vertical`, default). `Horizontal` uses left and right. `Select` selects text. Shift is ten steps, Alt is the last digit.

## Forms

The app owns the values and one `FormState<Key>`. A `FormSpec` lists the keys (one step is a single-page form). The state remembers what the user touched or submitted, and says per key whether the field may show its validity.

- `Field` draws label, control and message.
- `Form` (optional) adds Submit and Reset, and Back and Next with several steps.

```rust
use iced_adaptive_design::prelude::*;
use iced_adaptive_design::primitives::form::{FormSpec, FormState, Outcome};

struct App { name: String, form: FormState<&'static str> }

fn spec() -> FormSpec<&'static str> { FormSpec::single(["name"]) }

impl App {
    // Validity by key: the one thing the form needs from the values.
    fn validity(&self, key: &&'static str) -> Validity {
        match *key {
            "name" => required(&self.name, "Enter your name."),
            _ => Validity::Ok,
        }
    }

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Name(v) => { self.name = v; self.form.touch("name"); }
            Msg::Submit => {
                let mut form = std::mem::take(&mut self.form);
                match form.submit(&spec(), &|k| self.validity(k)) {
                    Outcome::Submitted => { /* send the values */ }
                    Outcome::Blocked { key, .. } => { /* focus `key` */ }
                    _ => {}
                }
                self.form = form;
            }
            Msg::Reset => { self.form.reset(); self.name.clear(); }
        }
    }

    fn view(&self) -> Element<'_, Msg> {
        let spec = spec();
        Form::new()
            .step(Step::new("Profile", move || {
                Field::new("Name", entry("Ada", &self.name).on_input(Msg::Name))
                    .required(true)
                    .validity(self.validity(&"name"))
                    .interaction(self.form.interaction(&spec, &"name"))
                    .into()
            }))
            .on_submit(Msg::Submit)
            .on_reset(Msg::Reset)
            .into()
    }
}
```

Do not keep your own `touched` list or `submitted` flag. `touch`, `interaction`, `submit` and `reset` are those. Live examples: the gallery's Field and Range slider pages.

**Labels.** `Field` is the label idiom outside forms too.

- `FieldLayout::Stacked`: label above the control.
- `FieldLayout::Row`: label and description at the start, control at the end, message below. Add `.plain_label(true)` for a body-size label in a settings-like list.
- A row that is a title, a subtitle and a suffix: `ActionRow`.
- An icon or unit inside an input is an `Entry` section (`left_section`, `right_section`), not a label layout.

**Pickers.** `DateInput` has its calendar button in the entry's left section. `DateRangeInput` is one entry (`2026-11-02 – 2026-11-09`). Both, and `ColorInput`, open a popover, or a dialog with `dialog(true)`. `ColorInput` is a color box that opens a `ColorPicker` with an entry for the code.

## A layout, in code

```rust
use iced_adaptive_design::prelude::*;

toolbar_view(page(
    preferences_group()
        .title("Appearance")
        .push(SwitchEntry::new("Notifications", notifications, Msg::Notifications))
        .push(ToggleGroupRow::new("Menus", items, Some(mode), Msg::Mode))
        .push(SliderRow::new("Shadow", shadow, 0.0..=2.0, Msg::Shadow)),
))
.top_bar(header_bar().title("Settings"))
```

## The same layout, declaratively (what a mapper binds to)

```xml
<toolbar_view>
  <header_bar slot="top_bar" title="Settings"/>
  <page>
    <preferences_group title="Appearance">
      <switch_entry title="Notifications" active="{notifications}" on_toggle="Notifications"/>
      <toggle_group_row title="Menus" active="{mode}" on_select="Mode"/>
      <slider_row title="Shadow" value="{shadow}" min="0" max="2" on_change="Shadow"/>
    </preferences_group>
  </page>
</toolbar_view>
```

Tag names are the snake_case constructors. Attribute names are the properties. Child elements go to the slot named in the component's man page (SLOTS), `push` by default. `on_*` attributes name messages.

### Application entries and custom inputs

Use `widgets::EntryRow` and its typed entries for application forms. Every entry
fills its parent width and shares `components::InputWrapper` for the label,
required marker, description and validation message. `InputWrapper` is the shared
`Field` presentation; use `.validity(...)` and `.interaction(...)` to reveal errors
from your form state.

- `TextEntry` edits a single-line string. `from_input` accepts an `Entry` configured
  with password mode, submit behavior and left/right sections for prefixes/suffixes.
- `NumberEntry::from_input` accepts a configured `NumberInput`, retaining numeric
  limits, precision and drag behavior, with step controls at the right edge.
- `DateEntry::from_input` accepts a `DateInput`; `.dialog(false)` opens a dropdown
  calendar and `.dialog(true)` opens a modal. Its calendar icon is on the right.
- `SwitchEntry` edits a boolean. `ActionEntry` shows a read-only value with one
  trailing action. Use `NavigationRow` for navigation destinations.
- `SelectEntry`, `MultiselectEntry` and `ChipGroupEntry` present selections from
  the left edge; dropdown triggers span the available width.
- `MultilineEntry` uses an application-owned `text_editor::Content`. Perform the
  received actions on that content. It grows from one to twelve lines, then
  scrolls. `from_input(text_view(...).max_lines(6))` changes that limit;
  `.single_line(true)` ignores Enter and normalizes pasted line breaks to spaces.

`EntryRow::new(label, EntryInput::Text(entry(...)))` selects the input type explicitly.
`EntryInput` also has Number, Date, Switch, Action, Select, Multiline and Custom variants. Low-level
`Entry`, `TextView`, `NumberInput` and `DateInput` components are building blocks
for custom widgets; prefer the typed entries in application forms. `EntryRow` now accepts an `EntryInput` rather than a text value; use
`TextEntry::new(label, value, on_input)` for text. `EditableText` and
`password_entry_row` have been removed; use `MultilineEntry` and
`TextEntry::from_input(label, password_entry(...))` instead.
