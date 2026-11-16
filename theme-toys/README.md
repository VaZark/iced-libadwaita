# Reference-driven theme toys

Four interactive reference scenes: `win95` (Windows Classic), `winxp`, `win7`, and `mate` (modern MATE 1.26). `gnome2` remains a **CLI compatibility alias for `mate`**, not a fifth selectable design. The [editor's split launcher](../theme-editor/README.md) offers these four targets alongside Gallery, with independent System / Light / Dark launch appearance.

```sh
TOYS_SCHEME=light cargo run -p iced-adaptive-design-theme-toys -- winxp
IAD_THEME=themes/mate.conf TOYS_SCHEME=light cargo run -p iced-adaptive-design-theme-toys -- mate
cargo run -p iced-adaptive-design-theme-toys -- --list
```

## Theme resolution

Each toy embeds its complete independent base from `theme-toys/themes/<target>.conf`. It builds with `Inheritance::OptOut`, `.files(false)`, the base as `.toolkit_text(...)` and an optional overlay as `.app_text(...)`. It installs `Config::default()` with that resolved theme config, rather than loading the shared desktop config or an installed `default.conf`. Both layers use the combined toolkit/app schema.

When present, `IAD_THEME` is explicitly read as an **overlay path**, including the editor's temporary file of current unsaved values. Valid overlay entries override the base; invalid entries warn and fall back. An unreadable path warns and leaves the base in use. In this toy-specific path, `IAD_THEME=none` is treated as a filename, not as the standard loader's bare-theme sentinel: omit `IAD_THEME` to use the base. No per-app theme file is read or regenerated.

Scene values have individual schema bounds. If resolved scene geometry fails the cross-property fit check, the toy discards overlay `toy.*` sections and rebuilds with base geometry while retaining native palette, typography and widget edits. These checks are not proof that every rendered child fits at every font size.

Pin `TOYS_SCHEME=light` for reference comparisons. Dark variants are adaptations, not historical reference targets. Glyph outlines, icon artwork, font _families_ and raster artifacts are excluded from fidelity evaluation; **font size, geometry, spacing and gradient structure are included**. No proprietary wallpaper or screenshot is bundled.

## Versioned reference manifest

[`src/references.rs`](src/references.rs) is the executable **v1 reference manifest**: source and image URLs, raster dimensions, logical scene sizes, estimated font sizes, measured/estimated proportions, and gradient angles/stops. Tests compare this metadata against the actual shipped configs through IAD's typed parser. Sizes are logical pixels, not CSS points or claims of exact font metrics.

| Scene   | Inspected reference and scope                                                                                                                                                                                                                                                                                                                                                                       | Current layout target                                                                                                                                                                                                                                                                                   |
| ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `win95` | [GUIdebook Server 2003 Classic](https://guidebookgallery.org/screenshots/winserv2003web), specifically [Display Properties / Desktop](https://guidebookgallery.org/pics/gui/settings/appearance/winserv2003web-1-1.png). Secondary: [PlayTool](http://www.playtool.com/pages/uninstallgraphics/uninstall.html), [Win9x adapter dialog](http://www.playtool.com/pages/uninstallgraphics/win9x1.gif). | 404×448, 11px text, compact five-tab dialog, 178px monitor, 112px sunken background list, right-side Browse/Position/Color, bottom 75px commands, horizontal navy→pale-blue title gradient. **Server 2003 Classic is not Windows 95 RTM**; the stable `win95` identifier is retained for compatibility. |
| `winxp` | [Exact supplied 1024×768 image](https://i.pinimg.com/1200x/71/65/b4/7165b40068a1b127276e482f04901a07.jpg)                                                                                                                                                                                                                                                                                           | Maximized **My Computer**, not a generic desktop or tabbed drive dashboard. 1024×738 client scene excludes the 30px desktop taskbar. 11px text, 210px blue task pane, menu/toolbar/address rows, four vertically stacked sections. No drive usage bars or notebook tabs.                                |
| `win7`  | [Windows 7 lesson](https://windows7lesson.blogspot.com/), with the supplied [1280×720 Start menu image](https://cdn.neowin.com/news/images/uploaded/2023/04/1680968851_windows-7-start-menu.jpg) as the precise scene                                                                                                                                                                               | **All Programs Start menu**, not Explorer. 498×598 menu at bottom left above a 48px taskbar; 304px program pane, 23px program rows, 42px places rows, approximately 14px text, blue-green glass-like gradient.                                                                                          |
| `mate`  | Official [MATE 1.26 English gallery](https://mate-desktop.org/gallery/1.26/english/), specifically [mate-1.26_7.png](https://mate-desktop.org/gallery/1.26/english/mate-1.26_7.png)                                                                                                                                                                                                                 | 1280×675 downloaded source mapped 1:1 to the logical scene. 18px desktop panels, left Pluma 408×410 at (114,104), right User Guide 444×410 at (798,50), roughly 10px body/editor text and 22px article heading; silver vertical chrome gradients.                                                       |

The explicit `/english/` subgallery contains the selected reference. See [reference status](REFERENCE_STATUS.md) for inspected captures, source provenance corrections, and the next concrete visual batch. Downloaded source images live only in ignored `target/reference-shots/`; development captures live in ignored `target/theme-shots/`. These are not tracked assets or golden tests.

## Tokens and library compatibility

- `themes/{win95,winxp,win7,mate}.conf` use only the library's native schema: palette, typography, metrics, gradients, frames, window controls and tree geometry.
- `theme-toys/themes/*.conf` hold **app-owned scene geometry and desktop gradients**. These complete bases include native styling as well as app tokens. Their typed schema and fallback defaults are declared in `src/design.rs` and installed before constructing the app. Toy code reads those tokens; it does not set arbitrary row heights, widths or spacing. Native child props are used for tab labels and the editor.
- This split is intentional: the unchanged library test validates every top-level theme against `Schema::toolkit()` and rejects unknown app-only tokens. App scene values are also independently validated and tested for invalid-value fallback.
- `themes/gnome2.conf` is the legacy modern-MATE theme alias, preserving its old key; there is no separate GNOME 2 scene. MATE and its alias retain independent Unix places data/state rather than sharing Windows drive labels.
- Scene geometry and native styling use validated tokens. The partial fit guard still contains hardcoded baseline estimates and needs replacement after the overflow policy is chosen; it is not full-layout acceptance. Declared app tokens do not need an unknown-token warning; genuine invalid values still produce diagnostics.
- Native text-trigger and context menus use the [public menu APIs](../docs/lib/windows-and-menus.md#menus). Local nested presentation uses [typed child props](../docs/lib/child-props.md#contract-and-usage), without taking over the parent's state or actions.

## Functional scope

- Classic: background selection and preview, Position, color toggle, bundled background chooser, notebook pages, checkboxes/sliders, Apply and confirmed Discard. No system display settings change. Non-Desktop tabs are control studies, not complete replicas.
- XP: task groups expand/collapse; items select; address edits; links and commands give status feedback. No programs are installed or removed, and no network shares are mounted.
- Win7: Start toggles the menu, literal case-insensitive search filters programs, selecting a program updates preview status/highlight, and shutdown opens a safe confirmation. No external program starts and no host shutdown occurs.
- MATE: native editable document, Undo/Redo, Save to preview memory, help search and section navigation; minimize/close and task-list restore; maximize/restore embedded preview windows. Places opens a functional Unix tree (Home/Desktop/File System/Network). Preview windows are embedded panels, not independent compositor windows, and are not draggable. Places does not browse the real host filesystem. Some menus report status rather than expose full application menus.

## Validation and honest limits

```sh
cargo check -p iced-adaptive-design-theme-toys --offline
cargo test -p iced-adaptive-design-theme-toys --offline
cargo test -p iced-adaptive-design --lib --offline
```

Tests cover CLI aliases/removal, native shipped-theme compatibility, app-token validation and invalid-value fallback, reference dimensions/proportions/font sizes/gradient stops, view construction, document editing/history, preview controls, filtering, Apply/Discard, and isolation of Unix versus Windows tree state. Boundary assertions test declared scene rectangles against their containing scene; they are **not renderer-layout or golden-pixel tests**.

For visual inspection against the manifest's sources, capture each target and its relevant states using the bounded, self-exiting development path:

```sh
IAD_THEME=themes/win7.conf TOYS_SCHEME=light \
ICED_ADAPTIVE_DESIGN_SCREENSHOT=target/theme-shots/win7.png \
ICED_ADAPTIVE_DESIGN_SHOT_DELAY_MS=1200 target/debug/theme-toys win7
```

Remaining differences: proprietary wallpaper is replaced by a theme gradient; no real Aero desktop blur; the classic monitor/background patterns are illustrative; detailed application contents, separators, toolbar edge treatments and some pane chrome remain approximations. There is no pixel-diff acceptance threshold. Font-family substitution and glyph rasterization are deliberately not graded. Full pointer/keyboard interaction, small-window responsiveness and every hover/focus/modal state have **not** received a manual acceptance pass. Do not infer those guarantees from successful captures or tests.
