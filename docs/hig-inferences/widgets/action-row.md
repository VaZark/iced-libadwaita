# ActionRow

libadwaita: [`ActionRow`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ActionRow.html)

## What it is

A list-box row presenting an action: title, subtitle, prefix and suffix widgets.

## Anatomy and behavior

- Layout: `[prefix] [title / subtitle] [suffix]`, optional **body** widget below the header (full-width content).
- Unactivatable by default; setting an `activatable-widget` makes it activatable and emits `activated`.
- `title-lines` / `subtitle-lines` limit lines before ellipsizing (default unlimited, so text **wraps**).
- `.property`: de-emphasizes the title, emphasizes the subtitle (read-only values); with `.monospace` only the subtitle is monospace.

## CSS nodes and classes

`row` > `box.content` > `box.header` > `box.title` > `label.title`, `label.subtitle`. Classes: `.property`, `.monospace`.

## Spec

- Row header min height 50 ✓; padding 6×12 ‡; spacing between prefix/title/suffix 12 ‡.
- Hover/active layer on activatable rows is a faint foreground tint (3% / 8% ✓).
- Title never narrower than its content: the title column takes the remaining space and wraps.

## iced mapping

`ActionRow::new(title).subtitle().prefix().suffix().on_activate(msg).chevron(true).wrap_suffix(true)`.
Title/subtitle live in a `Length::Fill` column with word wrapping; prefix/suffix are `Length::Shrink`
so they never collapse. Activatable rows are a flat `button::list_row`.
**Flex-wrap (default for all rows):** the suffix shares a line with prefix+title while the title keeps at
least `Metrics::row_wrap_basis` (128px); if it can't, the suffix wraps to its own line below (CSS
`flex-wrap`), so a wide control like a row of color swatches never crushes or overlaps the title. Applies
to ActionRow (and so SwitchRow, ComboRow, SpinRow, ExpanderRow), EntryRow suffixes and PreferencesGroup
header suffixes; opt out with `wrap_suffix(false)`.
**Pinned suffix:** rows whose suffix is compact and fixed-size (`SwitchRow`, `ComboRow`, `SpinRow`, `ExpanderRow`) never flex-wrap: the control is pinned to the end edge (`wrap_suffix(false)`). A wrapping row can place its wrapped control at the end with `suffix_align(Horizontal::Right)` (`flex-end`).
**Collapse behavior** (`CollapseBehavior`, default `Stacked`, set globally with `Metrics::row_collapse` or per row with
`.collapse(..)`): when the suffix wraps, `Stacked` gives it 100% of the row width on its own line (`Fill`-width
controls such as sliders and entries stretch); `Flex` keeps the control's own width, left-aligned. A `Fill`-width suffix
needs `.suffix_basis(px)`: the width it takes while sharing a line, since a `Fill` child has no natural width to measure.
The behavior lives in the public `adw::widget::FlexRow` (`FlexRow::new(main, aside).collapse(..).main_basis(..).aside_basis(..)`).
