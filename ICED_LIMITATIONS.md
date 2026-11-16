# iced limitations and upgrade checks

Read this before designing a feature or adding an iced workaround. This is the tracked register of iced constraints, integration contracts and local gaps that are easy to rediscover. Component-specific design rationale stays in [docs/hig-inferences/](docs/hig-inferences/README.md); link it here rather than treating an old observation as a permanent upstream fact.

## Evidence and maintenance

Baseline inspected on **2026-11-16**: `Cargo.toml` requests iced `0.14`; `Cargo.lock` resolves `iced`, `iced_core` and `iced_renderer` to **0.14.0**, and `iced_widget` to **0.14.2**. Features include wgpu and tiny-skia, advanced widgets and Wayland; X11 is intentionally disabled, not an iced limitation.

The initial entries below come from repository source, design notes and historical validation. This documentation pass does **not** freshly reproduce every upstream limitation. A passing regression verifies only its tested contract, not every renderer, platform or interaction.

- Before feature work, review relevant IDs and reuse the existing abstraction instead of duplicating its workaround.
- Add an entry when discovering a constraint or logical trap. Include a minimal reproducer/regression, affected versions/backends, source or upstream issue if known, workaround, and a concrete acceptance check. Do not invent upstream issue links.
- Distinguish **iced constraint**, **integration contract**, and **local gap**. An application lifecycle decision is not evidence that iced cannot implement a feature.
- Revisit **every entry on every iced-family dependency update**, including patch or lockfile-only changes to widget, renderer, runtime and backend crates. If behavior cannot be exercised, mark it pending with the reason; do not carry forward a pass automatically.
- Keep stable IDs. When fixed upstream, retain a short resolved entry with the first version actually tested, evidence and workaround removal/migration. Do not remove a workaround on release notes alone.

## Register

### ICED-01 — Text overflow and ellipsis

**Kind:** recorded iced constraint; needs live recheck on upgrade.

The existing typography implementation does not provide native ellipsis. Do not promise ellipsized labels or substitute character-count truncation (which ignores font shaping and available width). Choose wrapping, clipping or scrolling explicitly; a layout bound is not itself a clipping guarantee.

**Evidence/workaround:** [typography](src/components/typography.rs), [overflow policies](src/layouts/overflow.rs).

**Retest:** render long proportional-font, Unicode and multiline labels in narrow and resized containers, with enlarged font tokens. Inspect visible overflow. If a native ellipsis API becomes available, test shaping and layout before replacing the current policy.

### ICED-02 — Quad/text paint ordering within a layer

**Kind:** recorded renderer constraint.

Do not assume painter order across quad and text primitives in a single iced layer: quads paint before text. An incoming page's backing can leave the outgoing page's text visible unless the surfaces use separate layers.

**Evidence/workaround:** `PushStack::draw` in [motion](src/layouts/motion.rs) separates page layers.

**Retest:** overlap opaque and translucent pages containing both text and quads; inspect progress 0, .25, .5, .75 and 1 on both renderers. Text must not bleed through an opaque foreground. Keep target-derived backing geometry and test input during the transition, not just a settled screenshot.

### ICED-03 — Rounded clipping is not a native stencil

**Kind:** iced renderer API constraint plus local workaround/performance limit.

The current renderer integration exposes rectangular clips, not rounded stencils. A rounded background or border does not clip children. `Boundary` replays drawing in conservative, disjoint rectangular bands at one logical pixel precision; this is **not antialiased rounded compositing**. Large radii and deeply nested surfaces can multiply child draws. Draw code must be replay-safe.

**Evidence/workaround:** [Boundary module and rendered regressions](src/layouts/boundary.rs). The prior PROPS-4 validation recorded headless rendered checks and settled gallery inspection; it did not establish native AA or a performance budget.

**Retest:** run boundary regressions; inspect square, pill, asymmetric and nested corners with text/images, transparency, resize and non-unit display scaling. Profile large radii/deep nesting. If iced adds native clips, compare draw output, pointer boundaries, portal escape and performance before replacing the bands. Do not paint opaque corner covers as a substitute for clipping transparent content.

### ICED-04 — Focus depends on native focusable operations

**Kind:** recorded stock-widget constraint and integration contract.

Current docs record stock iced buttons as not focusable; entries and editors participate in native focus operations. `FocusScope` can only discover widgets that expose that operation. A focus trap does not automatically make arbitrary buttons keyboard-accessible; menu arrow-key behavior is a separate controlled interaction.

**Evidence/workaround:** [focus binding](src/binds/focus.rs), [focus guide](docs/lib/windows-and-menus.md#focus).

**Retest:** Tab/Shift+Tab through entries, editors, buttons and disabled controls; open a modal, check initial focus and wrapping, then dismiss and check focus return. Exercise menu arrows, activation and Escape separately. If native button focus changes, check traversal order and avoid duplicate keyboard activation.

### ICED-05 — Window screenshots may omit editor text

**Kind:** recorded screenshot limitation; backend scope not yet established.

`DESIGN.md` records that `iced::window::screenshot` omits `text_editor` text. Missing text in that capture alone is not proof of a broken editor or empty application state.

**Evidence/workaround:** [screenshot helper](src/dev.rs). Compare the live window with an independent compositor capture when investigating missing editor text; do not claim visual acceptance from an incomplete capture.

**Retest:** display distinctive nonempty editor text beside a normal text label; compare the live window, iced screenshot and compositor capture under wgpu and tiny-skia. Record renderer, compositor and display scale. Retire this warning only after both text paths are visibly captured.

### ICED-06 — OS child-window relationships

**Kind:** recorded window API/platform constraint.

Current design does not provide a true OS child-window relationship. Do not equate opening another iced window with transient parenting, modal ownership or independent process lifetime.

**Evidence/workaround:** [window design](DESIGN.md#dialogs-and-overlays), [Detached](src/windows/detached.rs). Use an in-window modal overlay; a window that must outlive the app belongs in a separate process.

**Retest:** inspect new iced window APIs and test any candidate parenting implementation on the supported Wayland environment: stacking, parent close, focus, modality and lifetime. Ordinary window creation does not establish these guarantees.

### ICED-07 — Wrappers must preserve tree, overlays and coordinates

**Kind:** integration contract, not an upstream bug.

A wrapper must forward child overlays and operations using the child's layout/tree position. Movement or scale must transform draw, cursor, touch, hit-testing, operations and overlay coordinates consistently. Preserve widget tree identity when rebuilding equivalent content. Portals deliberately escape ancestor surface clips; do not clip popup menus just because their owner is clipped.

**Evidence/workaround:** [Boundary portal handling](src/layouts/boundary.rs), [Overflow](src/layouts/overflow.rs), [downstream overflow regressions](tests/overflow_public.rs), [wrapper rule](CONTRIBUTING.md#working-on-a-component).

**Retest:** run boundary/overflow tests, then click and keyboard-operate nested controls during movement/scale. Open a tooltip, menu and submenu near a viewport edge; check anchor placement, dismissal, focus and operations. Test nested wrappers and sibling/unwind scope isolation. Compilation alone cannot validate forwarding.

### ICED-08 — Intrinsic measurement does not recover hidden content

**Kind:** layout integration contract.

Unbounded measurement cannot recover content a descendant deliberately fixes or clips. Fill-only content may need a caller-owned natural-size minimum. `OverflowMode::Grow` requests size but cannot force the OS window or parent to grow. Switching between Scroll and other modes changes the native widget tree and may reset descendant local state.

**Evidence/workaround:** [Overflow module/API](src/layouts/overflow.rs), [public policy tests](tests/overflow_public.rs). Choose Grow, Scroll or ScaleDown explicitly; do not hide a sizing failure behind preset-specific fit arithmetic.

**Retest:** enlarge font/spacing tokens and constrain the viewport; exercise all three modes, pointer/touch input, overlays and mode switches with focused/edited content. ScaleDown must preserve aspect ratio and never upscale; parent/window behavior must be checked separately from reported child size.

### ICED-09 — Hover-expanded stock scrollbar geometry

**Kind:** recorded stock-widget constraint; current backend/API scope needs recheck.

Current scrollbar comments record that the stock iced slider cannot grow on hover. Do not promise libadwaita's expanding overlay scrollbar merely by changing colors.

**Evidence/workaround:** [scrolled layout](src/layouts/scrolled.rs), [scrollbar metrics](src/metrics.rs). Geometry remains config-owned; do not copy historical numeric values from comments into rendering logic.

**Retest:** inspect stock scrollbar style/layout APIs; hover, drag and scroll in a constrained viewport. Check rail/slider bounds, hit areas, content shifts and theme-token overrides before adopting a native hover expansion.

### ICED-10 — Exit animation and gesture gaps are local scope

**Kind:** local implementation gaps, **not established iced impossibilities**.

The current design records no dialog/toast exit animations because the application unmounts them, mouse-only carousel drag, and no Dock drag gesture. Adding an animation requires retaining closing content until completion while preserving parent-owned state; adding gestures requires explicit input/cancellation semantics.

**Evidence:** [current design limitations](DESIGN.md#known-limitations).

**Retest:** if implementing these features or an iced update changes their input/lifecycle dependencies, exercise close/reopen during animation, reduced motion, modal capture/focus return, touch cancellation and nested scrolling. Do not silently change controlled APIs or declare a local gap fixed by a dependency bump.

## Required iced-update procedure

1. Record old/new locked versions, changed iced-family packages, enabled features, OS/compositor, renderer and display scale. Review upstream changes against **every ID**, including local contracts.
2. Run targeted regressions first, then workspace validation using the updated lockfile:

   ```sh
   cargo test --locked --lib layouts::boundary
   cargo test --locked --test overflow_public
   cargo test --locked --lib binds::focus
   cargo test --locked --workspace
   cargo clippy --locked --all-targets --workspace
   ```

3. Build/run the gallery and relevant theme toys following [CONTRIBUTING.md](CONTRIBUTING.md#the-gallery) and [toy validation guidance](theme-toys/README.md). Use `themes/libadwaita.conf` for the Adwaita gallery baseline. Exercise the per-entry scenarios live, in light/dark and narrow/wide layouts; include intermediate animation states. Test wgpu and tiny-skia separately and record the renderer actually used, not merely requested.
4. Save logs, screenshots and minimal reproducers under ignored `target/`. Record evidence paths and any environment blockers below. Screenshots cannot establish keyboard/gesture acceptance, and a successful build cannot establish visual acceptance.
5. Update each entry and add a dated validation row **in this tracked file**. Use `pass`, `reproduced`, `resolved`, `fail` or `pending` per ID. Pending checks remain explicitly outstanding; an upgrade must not be described as fully validated while they remain. Change/remove workarounds only with regression evidence and update gallery/toy consumers when their APIs or behavior change.

## Validation history

| Date | Locked baseline | Scope and result | Evidence / remaining checks |
| ---- | --------------- | ---------------- | --------------------------- |
| 2026-11-16 | iced/core/renderer 0.14.0; widget 0.14.2 | Register seeded from repository evidence; no dependency update. All live upgrade scenarios pending. | Source links above; no fresh full visual, interaction or backend acceptance claimed. |

For the next update, append a row with exact versions, commands/results, per-ID outcomes and evidence paths. Never replace an unperformed check with an inferred pass.
