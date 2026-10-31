# Design reference

Notes distilled from the GNOME HIG and, as the reference implementation, the official
libadwaita documentation (v1.11 alpha). We follow the HIG; libadwaita is a source of values and
behavior to learn from, not something to stay compatible with. Read these before changing a
widget; each widget file ends with an **iced mapping** section recording how we implement it and
where we deliberately deviate.

* Primary sources: the [libadwaita docs](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/index.html), especially
  [style classes](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/style-classes.html), [CSS variables](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/css-variables.html),
  [adaptive layouts](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/adaptive-layouts.html), [boxed lists](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/boxed-lists.html)
  and each `class.*.html` page.
* The published docs describe behavior, structure (CSS nodes) and colors, but
  rarely give pixel sizes. Where this folder lists a size that is **not** in the
  docs it is either **✓** (verified against libadwaita's stylesheet source,
  `src/stylesheet/**/*.scss` on `main`) or **‡** (from memory, unverified).
* Anything the docs don't say is recorded as "not specified" rather than guessed.

## Contents

* [Components](components.md) — the conventions every component follows, and the
  generated [catalog](components-catalog.md) (tag → class, constructor, properties, slots, events).
* [Desktop config](design-language/desktop-config.md) — shared desktop behavior and opt-outs.
* [Design language](design-language/) — colors, typography, style classes,
  metrics, adaptive layout, windows and client-side decorations.
* [Widgets](widgets/) — one file per libadwaita element.
* [GTK4](gtk4/) — the plain GTK4 widgets Adwaita builds on (scrollbars, entries,
  popovers, …). **Adwaita wins** wherever the two disagree.
