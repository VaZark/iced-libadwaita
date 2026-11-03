# Components

iced-libadwaita is a **component library**: every UI element is a reusable component with a uniform shape, so layouts can be written in code, or generated from a declarative description (XML/Blueprint, JSX-like, a macro) that maps tags to components. This page is the guide to how components fit together. The API of each one (constructors, properties, slots, events) is its man page, `iced-libadwaita-<alias>(3)` in `man/` (for example `man -l man/iced-libadwaita-dock.3`), generated from the doc comments; `iced-libadwaita-components(3)` indexes them; `iced_libadwaita::prelude::*` imports everything needed.

## Conventions

1. **Construction.** `Component::new(required…)` and/or a free function of the same name (`action_row(title)`). Required arguments are the minimum to render meaningfully.
2. **Properties** are chained builder methods named after libadwaita's properties (`.subtitle(..)`, `.revealed(..)`, `.sidebar_width(..)`, `.homogeneous(..)`). Booleans take a value (`.flat(true)`). `.metrics(Metrics)` is accepted wherever sizes matter.
3. **Slots** (children) are builder methods: `.push(child)` (lists), `.child(..)`, `.prefix(..)` / `.suffix(..)`, `.pack_start(..)` / `.pack_end(..)`, `.top_bar(..)`, `.content(..)`.
4. **Events** are `on_*` callbacks returning the app's message: `on_press`, `on_toggle(bool)`, `on_select(T)`, `on_input(String)`, `on_navigate(usize)`, `on_close`, `on_drag`.
5. **Controlled state.** Components don't own application state: the app passes `active`, `expanded`, `value`, `index`, `revealed`, … and updates it from the events. (Purely visual state, such as animation progress, hover or an open popover, lives inside the component.)
6. **Conversion.** Every component implements `Into<Element<'a, Message>>`; nest freely.
7. **Theme-driven.** Colors, radii, shadows and motion come from the `Theme` and the desktop [config](desktop-config.md), never from the component call site.
8. **No raw widgets in app code.** Entries, check/radio buttons, scales, text views, tags, swatches, cards, pages, rows with sliders and so on all have components; only layout primitives (`row`, `column`, `container`, `text`, `space`, `responsive`) remain, re-exported by the prelude.

## How to use forms

The app owns the values and one `FormState<Key>`; a `FormSpec` lists the keys (one step is a single-page form). The state only remembers what the user touched or submitted, and answers, per key, whether the field may show its validity. `Field` draws the label, the control and the message; `Form` (optional) adds the Submit and Reset bar and, with several steps, Back and Next. See `Form` (`iced-libadwaita-form(3)`) and `Field` (`iced-libadwaita-field(3)`).

```rust
use iced_libadwaita::prelude::*;
use iced_libadwaita::primitives::form::{FormSpec, FormState, Outcome};

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

Do not keep a `touched` list or a `submitted` flag of your own; `touch`, `interaction`, `submit` and `reset` are those. The gallery's Field and Range slider pages are the live examples.

**Labels.** `Field` is also the label idiom outside forms. `FieldLayout::Stacked` puts the label above the control (Mantine's `Input.Wrapper`); `FieldLayout::Row` puts the label (and description) at the start and the control at the end, the message below, like a libadwaita row; add `.plain_label(true)` for the body-size label of a settings-like list. For a row that is a title with a subtitle and a suffix, use `ActionRow`. Mantine's `leftSection` / `rightSection` are sections inside an input (an icon or a unit in the entry), which are `Entry` sections here, not a label layout.

## A layout, in code

```rust
use iced_libadwaita::prelude::*;

toolbar_view(page(
    preferences_group()
        .title("Appearance")
        .push(SwitchRow::new("Dark Style", dark, Msg::Dark))
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
      <switch_row title="Dark Style" active="{dark}" on_toggle="Dark"/>
      <toggle_group_row title="Menus" active="{mode}" on_select="Mode"/>
      <slider_row title="Shadow" value="{shadow}" min="0" max="2" on_change="Shadow"/>
    </preferences_group>
  </page>
</toolbar_view>
```

Tag names are the snake_case constructors; attribute names are the properties; child elements go to the slot named in the component's man page (its SLOTS section) (`push` by default); `on_*` attributes name messages.
