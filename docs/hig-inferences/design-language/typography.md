# Typography

Source: [style classes](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/style-classes.html) (Typography), [CSS variables](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/css-variables.html) (Fonts).

| Class                          | Meaning                                                                                          |
| ------------------------------ | ------------------------------------------------------------------------------------------------ |
| `.title-1` … `.title-4`        | Four title levels; size depends on context, larger for roomy views                               |
| `.heading`                     | Standard UI heading at default size (window titles, boxed-list labels)                           |
| `.body`                        | Increased line height for UI text such as descriptions/dialog body (avoid for non-wrapping text) |
| `.document`                    | Document font, larger size and line height, for app content (since 1.8)                          |
| `.caption-heading`, `.caption` | Smaller sub-text                                                                                 |
| `.monospace`                   | Monospace font; on `AdwEntryRow` only the editable part changes                                  |
| `.numeric`                     | Tabular figures (`tnum=1`) for aligned numbers                                                   |
| `.dimmed`                      | Partially transparent via `--dim-opacity` (replaces `.dim-label`)                                |

Font variables (example values): document Adwaita Sans 12pt, monospace Adwaita
Mono 11pt. Default UI size is the system's (11pt ≈ 14.67px).

## iced mapping

`Typography` scale: title_1 1.81×, title_2 1.36×, title_3 1.2×, title_4 1.1×,
heading 1× bold, caption .82×. iced cannot do tabular figures; `.numeric` is a
no-op. Text never relies on a fixed width.
