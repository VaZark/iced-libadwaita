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
