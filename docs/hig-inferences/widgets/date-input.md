# DateInput

libadwaita and GTK 4 have no date picker (`GtkCalendar` is a bare month grid, with no entry). `DateInput` combines an entry with a calendar popover, in the manner of GNOME apps that add a `GtkCalendar` to a `GtkPopover`.

- Text is `YYYY-MM-DD` (also `/` or `.` as separators); an unparsable date, or one outside `min`/`max`, draws the entry in its error state.
- The calendar is a Monday-first month grid with previous/next month buttons. Days outside `min`/`max` are disabled. The chosen day is accent-colored like a selected menu item.
- State is controlled: the app owns the text, the month on show and the chosen date.

## iced mapping

`widget::DateInput`: an `Entry` plus a `Dropdown` (with `keep_open(true)`, so paging months and picking a day do not close the calendar; click outside or Escape closes it). Date arithmetic is the dependency-free `Date` type (no `chrono`): validation, weekday (Sakamoto), month navigation and the grid are unit tested. Locale is not modeled: English month names, Monday first, ISO format.
