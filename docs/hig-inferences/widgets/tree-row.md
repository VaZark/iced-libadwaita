# TreeRow

No libadwaita widget. Inspired by [ExpanderRow](expander-row.md); the model for the arrow is GTK's [`GtkTreeExpander`](https://docs.gtk.org/gtk4/class.TreeExpander.html).

## Scope: nesting within a page, not logic across pages

A `TreeRow` nests elements **inside one page**: a group of settings, an outline, a file list, rows that belong together on a single screen. It is **not for logic that spans pages**. Moving between pages, a sidebar of destinations or a navigation stack belong to [NavigationView](navigation-view.md) and the [sidebar](sidebar.md) components, which the [HIG](https://developer.gnome.org/hig/patterns/nav/sidebars.html) keeps flat ("a vertical panel which contains a list of different locations"). If opening a row changes which page you are on, it is navigation, not a tree row.

## What it is

A row with an **arrow** at the start that opens and closes it, a **title** that is a separate target (select it, open it, run it), an optional prefix and suffix, and any number of children pushed under it. A leaf has no arrow but keeps its width, so labels line up with those of its siblings.

## Anatomy and behavior

- **Children are anything:** other `TreeRow`s, `ActionRow`s, switch rows, or a plain widget, via `.push(..)`. They lie on the shaded list that slides open (the `Collapsible` disclosure primitive, 250 ms) and are separated by hairlines.
- **Two targets.** `on_toggle(|open| ..)` is the arrow; `on_activate(msg)` is the title. `ExpanderRow` makes the whole header one toggle; a tree needs both, so selecting a row and opening it are separate.
- **Controlled.** The app owns `expanded` and `selected`; `selected(true)` puts the row on the selection highlight.
- **Indent.** `indent(px)` offsets the children. It is a plain value that starts at the toolkit's default, `Metrics::expander_indent` (`[rows] expander-indent`, 24 px, the arrow's width) and is overridden per row; 0 is libadwaita's flush nesting.
- **Lazy children.** `expandable(true)` shows the arrow before the children exist.

## iced mapping

`widget::TreeRow`, built on `ActionRow` (title, subtitle, prefix, suffix, centring, row height and padding) and `Collapsible`. The data-driven [Tree](tree.md) in its grouped mode is built from `TreeRow`s; use `TreeRow` directly for a hand-written hierarchy, and `Tree` for data. Not included: keyboard navigation (iced has no focus model), and drag and drop (use `Tree`).
