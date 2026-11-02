# Table sorting

`ResponsiveTable` columns can be sortable, after Mantine's table sorting recipes and `mantine-datatable`: clicking a header cycles **ascending, descending, unsorted**.

- **Controlled.** Mark columns with `Column::sortable(key)`, pass the app's `primitives::sorting::SortState` to `ResponsiveTable::sort(&state)` and handle `on_sort(key)` with `state.activate(key, false)`, then re-sort the rows with `state.apply(&mut rows, comparator)` (a **stable** sort). Paging the sorted rows is the app's job (see [Pagination](pagination.md)); `ResponsiveTable::footer` is where it goes.
- **Header.** A button with an arrow: both chevrons for a sortable column that is not sorted (`GoUpDown`), up or down for the sorted one, a number next to it when several columns sort (multi-sort).
- **Cards.** The cards presentation has no header row, so the same events come from a **sort control above the cards**: a dropdown of the sortable columns and a direction button. Both send `on_sort(key)`, so the app has one code path and the two presentations stay consistent.
- **Comparators.** `cmp_f64`, `cmp_str` (case-insensitive), `cmp_natural` (digit-aware: "item 2" before "item 10"), `cmp_date`, and `cmp_option` with nulls first or last.

## iced mapping

`primitives::sorting` is pure and unit tested (the cycle, multi-sort priority, stability, comparators). Deviations: two-state cycling (`Cycle`) and multi-sort (`multi(true)`, additive activation) are options on `SortState`; there is no keyboard activation of headers beyond the button's own.
