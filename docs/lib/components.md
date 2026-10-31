# Components

iced-libadwaita is a **component library**: every UI element is a reusable component with a
uniform shape, so layouts can be written in code, or generated from a declarative description
(XML/Blueprint, JSX-like, a macro) that maps tags to components. The machine-readable list is
`iced_libadwaita::catalog::COMPONENTS` (rendered in [components-catalog.md](components-catalog.md));
`iced_libadwaita::prelude::*` imports everything needed.

## Conventions

1. **Construction.** `Component::new(required…)` and/or a free function of the same name
   (`action_row(title)`). Required arguments are the minimum to render meaningfully.
2. **Properties** are chained builder methods named after libadwaita's properties
   (`.subtitle(..)`, `.revealed(..)`, `.sidebar_width(..)`, `.homogeneous(..)`). Booleans take a
   value (`.flat(true)`). `.metrics(Metrics)` is accepted wherever sizes matter.
3. **Slots** (children) are builder methods: `.push(child)` (lists), `.child(..)`,
   `.prefix(..)` / `.suffix(..)`, `.pack_start(..)` / `.pack_end(..)`, `.top_bar(..)`, `.content(..)`.
4. **Events** are `on_*` callbacks returning the app's message: `on_press`, `on_toggle(bool)`,
   `on_select(T)`, `on_input(String)`, `on_navigate(usize)`, `on_close`, `on_drag`.
5. **Controlled state.** Components don't own application state: the app passes `active`,
   `expanded`, `value`, `index`, `revealed`, … and updates it from the events. (Purely visual
   state, such as animation progress, hover or an open popover, lives inside the component.)
6. **Conversion.** Every component implements `Into<Element<'a, Message>>`; nest freely.
7. **Theme-driven.** Colors, radii, shadows and motion come from the `Theme` and the desktop
   [config](desktop-config.md), never from the component call site.
8. **No raw widgets in app code.** Entries, check/radio buttons, scales, text views, tags,
   swatches, cards, pages, rows with sliders and so on all have components; only layout
   primitives (`row`, `column`, `container`, `text`, `space`, `responsive`) remain, re-exported by
   the prelude.

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

Tag names are the snake_case constructors; attribute names are the properties; child elements go
to the `slot` named by the catalog (`push` by default); `on_*` attributes name messages.
