# DateRangeInput

libadwaita and GTK 4 have no date-range picker (`GtkCalendar` selects one day). `DateRangeInput` is the multi-value sibling of [`DateInput`](date-input.md): a start and an end entry (`YYYY-MM-DD`) and one calendar popover that selects the interval, in the manner of the range calendars of Mantine and Material.

- Selection rule (`primitives::range::DateRange::pick`): the first click sets the start; the second sets the end (the two swap if it is earlier; the same day is a range of one day); a third click starts over. Days outside `min`/`max` are disabled and ignored.
- The range reads as one accent band: the start and end days are filled and rounded on the outer side, the days between are tinted. While only the start is chosen, the band previews up to the day under the pointer (`on_hover`).
- Validity: an unparsable or out-of-range text marks its own entry; an end before the start marks the end entry ("End must not be before start."; with `strict(true)` the same day is an error too: "End must be after start."). Errors show once `revealed`, so a `Field` can keep a pristine field quiet and draw the one ring.
- The calendar has the same levels as [`DateInput`](date-input.md) (title zooms out to year and decade, arrows step by the level, bounds disable months and years). Days are picked, and the range rule applies, **only at the month level**: the month and year cells always drill down (the range calendar has no `min_level`, as in Mantine's `DatePicker type="range"`), so a range is never made from months. The range stays highlighted across levels: a month or year that holds the start or end is filled, one strictly between them is tinted.
- The app owns the two texts, the `DateRange`, the calendar position (`.calendar(CalendarView)` with `on_nav`) and the hovered day.

## iced mapping

`widget::DateRangeInput`: two `Entry`s and a `Popover` with `keep_open(true)`. The month grid is shared with `DateInput` (`date_input::calendar`); only the day cells differ. The pointer preview is app state (`on_hover`/`hover`, sent only while a start is chosen without an end) because iced buttons do not expose hover; without `on_hover` the range just has no preview. `.revealed(bool)` and `validity()` follow `DateInput`, so it works inside `Field`: pass `Field::validity(input.validity())` and `revealed(touched || submitted)`.

Navigation is the shared `primitives::calendar` (see `DateInput`); `on_view` remains the month-level compatibility path.

Deviations: one month on show (no two-month range view); no presets ("last 7 days"); English month names and ISO dates as in `DateInput`.
