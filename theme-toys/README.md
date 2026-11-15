# Reference-driven theme toys

Four interactive reference scenes: `win95` (Windows Classic), `winxp`, `win7`, and
`mate` (modern MATE 1.26). `gnome2` remains a **CLI compatibility alias for `mate`**,
not a fifth selectable design. Mac has been removed from the CLI and implementation.
The editor launcher is maintained separately.

```sh
IAD_THEME=themes/winxp.conf TOYS_SCHEME=light cargo run -p iced-adaptive-design-theme-toys -- winxp
IAD_THEME=themes/mate.conf TOYS_SCHEME=light cargo run -p iced-adaptive-design-theme-toys -- mate
cargo run -p iced-adaptive-design-theme-toys -- --list
```

A toy uses the theme in effect: `IAD_THEME`, installed default, or the bare toolkit.
Pin `TOYS_SCHEME=light` for reference comparisons. Dark variants are adaptations, not
historical reference targets. Glyph outlines, icon artwork, font *families* and raster
artifacts are excluded from fidelity evaluation; **font size, geometry, spacing and
gradient structure are included**. No proprietary wallpaper or screenshot is bundled.

## Versioned reference manifest

[`src/references.rs`](src/references.rs) is the executable **v1 reference manifest**:
source and image URLs, raster dimensions, logical scene sizes, estimated font sizes,
measured/estimated proportions, and gradient angles/stops. Tests compare this metadata
against the actual shipped configs through IAD's typed parser. Sizes are logical pixels,
not CSS points or claims of exact font metrics.

| Scene | Inspected reference and scope | Current layout target |
| --- | --- | --- |
| `win95` | [GUIdebook Server 2003 Classic](https://guidebookgallery.org/screenshots/winserv2003web), specifically [Display Properties / Desktop](https://guidebookgallery.org/pics/gui/settings/appearance/winserv2003web-1-1.png). Secondary: [PlayTool](http://www.playtool.com/pages/uninstallgraphics/uninstall.html), [Win9x adapter dialog](http://www.playtool.com/pages/uninstallgraphics/win9x1.gif). | 404×448, 11px text, compact five-tab dialog, 178px monitor, 112px sunken background list, right-side Browse/Position/Color, bottom 75px commands, horizontal navy→pale-blue title gradient. **Server 2003 Classic is not Windows 95 RTM**; the stable `win95` identifier is retained for compatibility. |
| `winxp` | [Exact supplied 1024×768 image](https://i.pinimg.com/1200x/71/65/b4/7165b40068a1b127276e482f04901a07.jpg) | Maximized **My Computer**, not a generic desktop or tabbed drive dashboard. 1024×738 client scene excludes the 30px desktop taskbar. 11px text, 210px blue task pane, menu/toolbar/address rows, four vertically stacked sections. No drive usage bars or notebook tabs. |
| `win7` | [Windows 7 lesson](https://windows7lesson.blogspot.com/), with the supplied [1280×720 Start menu image](https://cdn.neowin.com/news/images/uploaded/2023/04/1680968851_windows-7-start-menu.jpg) as the precise scene | **All Programs Start menu**, not Explorer. 498×598 menu at bottom left above a 48px taskbar; 304px program pane, 23px program rows, 42px places rows, approximately 14px text, blue-green glass-like gradient. |
| `mate` | Official [MATE 1.26 English gallery](https://mate-desktop.org/gallery/1.26/english/), specifically [mate-1.26_7.png](https://mate-desktop.org/gallery/1.26/english/mate-1.26_7.png) | 4096×2160 source mapped to 1280×675 logical scene (÷3.2). 18px desktop panels, left Pluma 408×410 at (114,104), right User Guide 444×410 at (798,50), roughly 10px body/editor text and 22px article heading; silver vertical chrome gradients. |

The gallery's `/gallery/1.26/` landing page returned an empty image list during inspection;
the explicit `/english/` subgallery contains the selected full-resolution reference.
Downloaded source images live only in ignored `target/reference-shots/`; development
captures live in ignored `target/theme-shots/`. These are not tracked assets or golden tests.

## Tokens and library compatibility

- `themes/{win95,winxp,win7,mate}.conf` use only the library's native schema: palette,
  typography, metrics, gradients, frames, window controls and tree geometry.
- `theme-toys/themes/*.conf` hold **app-owned scene geometry and desktop gradients**.
  Their typed schema and fallback defaults are declared in `src/design.rs` and installed
  before constructing the app. Toy code reads those tokens; it does not set arbitrary
  row heights, widths or spacing. Native child props are used for tab labels and the editor.
- This split is intentional: the unchanged library test validates every top-level theme
  against `Schema::toolkit()` and rejects unknown app-only tokens. App scene values are
  also independently validated and tested for invalid-value fallback.
- `themes/gnome2.conf` is the legacy modern-MATE theme alias, preserving its old key;
  there is no separate GNOME 2 scene. MATE and its alias retain independent Unix places
  data/state rather than sharing Windows drive labels.
- `themes/mac.conf` is a **comment-only compatibility tombstone**, with no `[theme]`
  metadata or styling. The existing library tests still `include_str!` that path. The
  parent can remove the tombstone once that include is removed; it is not a Mac preset.
- IAD reports app scene-file values as app overrides at startup. These are declared,
  validated app presentation tokens, not hardcoded native-widget overrides.

## Functional scope

- Classic: background selection and preview, Position, color toggle, bundled background
  chooser, notebook pages, checkboxes/sliders, Apply and confirmed Discard. No system
  display settings change. Non-Desktop tabs are control studies, not complete replicas.
- XP: task groups expand/collapse; items select; address edits; links and commands give
  status feedback. No programs are installed or removed, and no network shares are mounted.
- Win7: Start toggles the menu, literal case-insensitive search filters programs, selecting
  a program updates preview status/highlight, and shutdown opens a safe confirmation.
  No external program starts and no host shutdown occurs.
- MATE: native editable document, Undo/Redo, Save to preview memory, help search and
  section navigation; minimize/close and task-list restore; maximize/restore embedded
  preview windows. Places opens a functional Unix tree (Home/Desktop/File System/Network).
  Preview windows are embedded panels, not independent compositor windows, and are not
  draggable. Places does not browse the real host filesystem. Some menus report status
  rather than expose full application menus.

## Validation and honest limits

```sh
cargo check -p iced-adaptive-design-theme-toys --offline
cargo test -p iced-adaptive-design-theme-toys --offline
cargo test -p iced-adaptive-design --lib --offline
```

Tests cover CLI aliases/removal, native shipped-theme compatibility, app-token validation
and invalid-value fallback, reference dimensions/proportions/font sizes/gradient stops,
view construction, document editing/history, preview controls, filtering, Apply/Discard,
and isolation of Unix versus Windows tree state. Boundary assertions test declared scene
rectangles against their containing scene; they are **not renderer-layout or golden-pixel tests**.

All four source images were downloaded and inspected with the image reader, as were
Wayland captures of all four implemented scenes. Screenshot inspection found and corrected
classic command-row clipping, XP task-pane density and MATE editor background/text sizing.
Captures use the app's existing bounded, self-exiting development path:

```sh
IAD_THEME=themes/win7.conf TOYS_SCHEME=light \
ICED_ADAPTIVE_DESIGN_SCREENSHOT=target/theme-shots/win7.png \
ICED_ADAPTIVE_DESIGN_SHOT_DELAY_MS=1200 target/debug/theme-toys win7
```

Remaining differences: proprietary wallpaper is replaced by a theme gradient; no real
Aero desktop blur; the classic monitor/background patterns are illustrative; detailed
application contents, separators, toolbar edge treatments and some pane chrome remain
approximations. There is no pixel-diff acceptance threshold. Font-family substitution
and glyph rasterization are deliberately not graded. Full pointer/keyboard interaction,
small-window responsiveness and every hover/focus/modal state have **not** received a
manual acceptance pass. Do not infer those guarantees from successful captures or tests.
