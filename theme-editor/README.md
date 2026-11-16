# Theme editor

Run `cargo run -p iced-adaptive-design-theme-editor -- [FILE]`.

The primary sidebar selects token sections. **All** groups every token on one
searchable page. **Modified only** filters unsaved changes; **Overrides only**
shows explicit entries in the active Light/Dark scheme, excluding inherited Base
values. Overrides only is unavailable for Base.

The selected theme in the header opens the theme picker, containing registered
files, added folders and recent files. Selected names and picker options use
editor-local typed text/container props to wrap long Unicode names and paths
within bounded widths; shared widget overflow defaults are unchanged.

**Save** updates the current file, following symlinks. **Save as** creates and
auto-registers a copy and makes it the current file for subsequent saves. Both
header actions use bundled `Icon::Save` / `Icon::SaveAs` artwork. Validation,
unsaved-change confirmation and undo/redo remain available.

There is no main-window live preview. **Try it out** is a primary launch action
with alternative targets: Gallery, Windows 95, Windows XP, Windows 7 and modern
MATE. Launch appearance (**System / Light / Dark**) is independent of the scheme
being edited. System clears inherited toy/gallery scheme overrides.

Toy discovery accepts only the four canonical IDs: `win95`, `winxp`, `win7`,
`mate`. A legacy installed-list `gnome2` entry normalizes to `mate`, with no
separate GNOME target. Stale labels and duplicate entries are normalized; removed
or unknown targets are not offered. Failed or unusable discovery uses the four
canonical targets. Upgrade an older installed `theme-toys` binary if it does not
accept the current IDs; discovery does not resurrect removed targets.

## Validation

`cargo test -p iced-adaptive-design-theme-editor --offline` covers file/save and
registration workflows, filtering, launch schemes, canonical target discovery,
and actual headless layout of long multilingual selected names and picker
options at narrow widths. Layout assertions are not screenshot or pixel-perfect
visual acceptance.
