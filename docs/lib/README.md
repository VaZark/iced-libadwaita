# Using the library

The user guide: when and how to use the library. It ships with the crate.

It is not the API reference. The API is in the man pages (`man/`), generated from the doc comments:

| Page                              | Holds                                                          |
| --------------------------------- | -------------------------------------------------------------- |
| `iad(7)`              | overview: conventions, theming, dialogs, focus, primitives     |
| `iad-<alias>(3)`      | one component: constructors, properties, slots, events         |
| `iad-components(3)`   | index of the components                                        |
| `iad-desktop.conf(5)` | every key of the desktop config                                |

Read one: `man -l man/iad-dock.3`. Or put them under `man3`, `man5`, `man7` of your `MANPATH`.

In this guide a component is named by its type. Its page is its snake_case alias with dashes: `NumberInput` is `iad-number-input(3)`.

The design rationale (HIG notes, libadwaita sources) is for contributors. It is in the repository under `docs/hig-inferences/` and is not published.

## The guide

- [choosing.md](choosing.md): which component for which job.
- [layout.md](layout.md): pages, rows, the sidebar, adaptive layouts, docks.
- [windows-and-menus.md](windows-and-menus.md): decorations, dialogs, overlays, docks, focus, separate windows, menus, shortcuts, toasts.
- [theme-config.md](theme-config.md): per-app styling: hardcoded, app theme file, toolkit theme file, each regenerable.
- [theming.md](theming.md): colors, accent, contrast, radii, shadows, density, transparency, tint.
- [motion.md](motion.md): animation, reduced motion, ordered transitions.
- [components.md](components.md): the shape every component has, forms, number input.
- [primitives.md](primitives.md): the headless logic under the components.
- [desktop-config.md](desktop-config.md): the shared desktop config. [Sample](desktop.conf.example).
- [config-registry.md](config-registry.md): the versioned registry behind it: types, defaults, `since`, deprecation.

`ChipGroup::max_lines(n).expandable(open, on_toggle)` limits wrapping chips to `n`
lines (three when omitted). An inline downward chevron follows the last visible chip
on the final allowed line, reserving space by hiding the last chip if necessary.
Expanding reveals all chips, followed by an upward chevron to collapse again.
Short lists omit the control. Selection and search remain controlled by the app;
the search entry stays outside the chip flow. Actual measured widget widths drive
wrapping, resizing and control placement; hidden chips do not participate in drawing,
input, focus operations or overlays.

`ProfileHeader::new(name, image, description)` is the generic profile primitive.
Optional `after_name`, `top_right` and `controls` slots belong to the caller. The
opinionated media composition is no longer a toolkit export; see the complete
`gallery/src/example_detail_header.rs` example or Weave's `MediaHeader`.

`SectionDivider` is controlled: `collapsible`, `default_collapse_state`, `open`,
`on_toggle`, `chevron` and `trailing` configure its header, while the application
owns the body and open state. An explicit `collapsible(false)` suppresses toggling
and the chevron even if a callback exists. With a callback and no explicit flag,
existing dividers remain collapsible; without a callback the divider stays a heading.

- [icons.md](icons.md): installed XDG icon themes, symbolic colors, missing icons, and API migration.
