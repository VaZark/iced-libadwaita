# DateRangeInput

libadwaita and GTK 4 have no date-range picker (`GtkCalendar` selects one day). `DateRangeInput` is the multi-value sibling of [`DateInput`](date-input.md): a start and an end entry (`YYYY-MM-DD`) and one calendar popover that selects the interval, in the manner of the range calendars of Mantine and Material.

- Selection rule (`primitives::range::DateRange::pick`): the first click sets the start; the second sets the end (the two swap if it is earlier; the same day is a range of one day); a third click starts over. Days outside `min`/`max` are disabled and ignored.
- The range reads as one accent band: the start and end days are filled and rounded on the outer side, the days between are tinted. While only the start is chosen, the band previews up to the day under the pointer (`on_hover`).
- Validity: an unparsable or out-of-range text marks its own entry; an end before the start marks the end entry ("End must not be before start."; with `strict(true)` the same day is an error too: "End must be after start."). Errors show once `revealed`, so a `Field` can keep a pristine field quiet and draw the one ring.
- The app owns the two texts, the `DateRange`, the month on show and the hovered day.

## iced mapping

`widget::DateRangeInput`: two `Entry`s and a `Dropdown` with `keep_open(true)`. The month grid is shared with `DateInput` (`date_input::calendar`); only the day cells differ. The pointer preview is app state (`on_hover`/`hover`, sent only while a start is chosen without an end) because iced buttons do not expose hover; without `on_hover` the range just has no preview. `.revealed(bool)` and `validity()` follow `DateInput`, so it works inside `Field`: pass `Field::validity(input.validity())` and `revealed(touched || submitted)`.

Deviations: one month on show (no two-month range view); no presets ("last 7 days"); English month names and ISO dates as in `DateInput`.
