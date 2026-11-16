# Reference compatibility checkpoint

The toys have independent validated bases and smaller-part interaction tests. They are not pixel-equivalent reproductions. Exact icon artwork and font-family rasterization are excluded from comparison; the surrounding controls, dimensions, font sizes, spacing, gradients and interactions are not excluded.

## Capture evidence

Captured on 2026-11-15 using the Wayland desktop, `TOYS_SCHEME=light`, the bundled per-design bases and no added editor overlay. Raster sizes match configured logical sizes; the capture command did not log the selected renderer/backend, so this is not a pinned-renderer comparison. Captures exit automatically through `ICED_ADAPTIVE_DESIGN_SCREENSHOT`; images and downloaded references remain ignored under `target/`.

| Target  | Capture                                     | Source / scene                                                                                                                                              | Source raster          |
| ------- | ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------- |
| Classic | `target/theme-shots/win95-classic-tabs.png` | [Server 2003 Classic Display Properties](https://guidebookgallery.org/pics/gui/settings/appearance/winserv2003web-1-1.png), Desktop tab; not Windows 95 RTM | 404×448                |
| XP      | `target/theme-shots/winxp-current.png`      | [My Computer](https://i.pinimg.com/1200x/71/65/b4/7165b40068a1b127276e482f04901a07.jpg), desktop taskbar excluded from toy                                  | 1024×768; toy 1024×738 |
| Win7    | `target/theme-shots/win7-current.png`       | [All Programs Start menu](https://cdn.neowin.com/news/images/uploaded/2023/04/1680968851_windows-7-start-menu.jpg)                                          | 1280×720               |
| MATE    | `target/theme-shots/mate-current.png`       | [Official MATE 1.26 Pluma/help scene](https://mate-desktop.org/gallery/1.26/english/mate-1.26_7.png)                                                        | 1280×675               |

Source raster sizes were checked from the downloaded binary images with Pillow. The MATE source is 1280×675: the previous manifest's 4096×2160 and `/3.2` provenance claim was incorrect. All four source images and output captures were viewed; this is qualitative visual inspection, not an automated full-image equivalence test.

## Completed surface correction

Classic uses the existing raised classic-tab renderer rather than ordinary outline buttons. `[frame.tab]` and `[tabs]` own its colors, bevel and geometry. The selected tab opens into the panel, the extra panel frame is removed, and `tabs.panel-padding` is a validated optional length rather than deriving panel padding from tab-label padding. The classic panel resolves its background from `frame.panel.bg`, then the window surface. A headless rendered regression checks gray tab/panel fill and independent configured panel padding. Other renderers are unchanged.

## Next visual batch

These are concrete remaining findings, not accepted exceptions:

- **Classic:** monitor casing/foot geometry differs from the reference beyond icon artwork; its preview sits higher. Background list/control widths and placement differ, and Position/Color triggers lack the reference's conventional field frame. Other tabs contain preview content rather than faithful target scenes. Check each independently rather than treating the tab-seam fix as whole-dialog completion.
- **XP:** toolbar controls are white standalone buttons rather than the reference toolbar treatment; sidebar task headings, separators and content placement differ. Later section headings drift upward compared with the source. Compare section spacing and item geometry independently of icon silhouettes.
- **Win7:** user picture container is not positioned partly above the menu, the program-list scrollbar and framed search field differ, group separators are missing, and the bottom Back/search areas differ in placement. Desktop wallpaper/glass remain approximations; ignoring proprietary artwork does not exempt their geometry or surrounding controls.
- **MATE:** the help window is missing breadcrumb/navigation rows, the editor toolbar/tab/status treatments and content density differ, and desktop item arrangement differs. Resolve these against the actual 1:1 source, not the invalid `/3.2` provenance claim.
- **All targets:** coordinated menubar keyboard traversal/cascading submenu semantics and the remaining preview-only commands need explicit acceptance checks. Current menu tests do not establish complete native behavior.

## Layout-fit policy decision

The current `design::layout_fits` is only a partial geometry guard. It uses hardcoded scene-baseline arithmetic and does not measure native font/widget content, so it is not whole-layout fit acceptance. Generic default-relative bounds also need individual per-property review. Before replacing this guard, choose how overlays that exceed fixed reference geometry should behave: enlarge the scene, scroll it, or reject offending layout edits. Preserve color-only edits and report rejected values explicitly. Do not silently choose this interaction policy or describe the current guard as complete.
