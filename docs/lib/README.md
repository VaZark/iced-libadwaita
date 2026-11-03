# Using the library

The **user guide**: how, when and what to use to build a good GNOME app. It is written for the people and agents who build applications with iced-libadwaita, and it is bundled with the crate.

It is not the API reference. The API, component by component, is in the **man pages** (`man/`), which are generated from the doc comments in the source:

- `iced-libadwaita(7)`: the overview: conventions, theming, windows and dialogs, focus, primitives.
- `iced-libadwaita-<alias>(3)`: one page per component, such as `iced-libadwaita-dock(3)` or `iced-libadwaita-number-input(3)`: its constructors, properties, slots and events. `iced-libadwaita-components(3)` is the index.
- `iced-libadwaita-desktop.conf(5)`: every key of the desktop configuration.

Read them with `man -l man/iced-libadwaita-dock.3`, or install them under a `man3` / `man7` / `man5` directory of your `MANPATH`. In this guide a component is named by its type; the page name is its snake_case alias with dashes. The design rationale behind the guidelines (the HIG notes, libadwaita sources) is for contributors and lives in the repository under `docs/hig-inferences/`; it is not bundled.

## The guide

- [choosing.md](choosing.md): which component for which job.
- [layout.md](layout.md): pages, rows, adaptive layouts and the sidebar.
- [windows-and-menus.md](windows-and-menus.md): decorations, header bars, dialogs, menus, toasts.
- [theming.md](theming.md): colors, contrast, radii, shadows, density, transparency and tint.
- [motion.md](motion.md): animation and reduced motion.
- [components.md](components.md): the shape every component has, and how forms work.
- [desktop-config.md](desktop-config.md): the shared desktop config; [sample](desktop.conf.example).
- [config-registry.md](config-registry.md): the versioned registry behind it: types, defaults, `since`, deprecation, and what apps ignore.
