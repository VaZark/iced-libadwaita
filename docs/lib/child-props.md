# Typed child presentation: API audit

Audit checkpoint: 2026-11-15, PROPS-3. **Partial coverage, not blanket completion.** The downstream integration suite is `tests/child_props_public.rs`. It imports only public crate paths and constructs real widgets; it does not inspect source strings or use private test helpers. This document inventories the current surface, records component and PROPS-MERGE widget fixes, and distinguishes remaining audit gaps from parent-owned integration work.

## Contract and usage

The child owns its presentation type. A parent accepts that exact type and applies it after its own child defaults. Do not introduce a string-keyed part/style map or a props type that merely gives an unrelated configuration type a new name. `components::props` contains concrete native adapters (`TextProps`, `ButtonProps`, `ContainerProps`, `EntryProps`); component-specific types live with their widgets. No generic string-map child API was found in those adapters or the public component/widget export inventory. Theme token/variant names are a separate API.

```rust
use iced_adaptive_design::{Element, components::{EntryProps, TextInputProps},
    widgets::{NumberInput, NumberInputProps}};

let control: Element<'_, String> = NumberInputProps::default()
    .entry_props(EntryProps::default()
        .input_props(TextInputProps::default().size(18).padding(4)))
    .apply(NumberInput::new("42", |value| value))
    .into();
```

Values, content, callbacks, IDs, password mode, validation decisions, disabled/open state and dismissal policy belong to the parent, not presentation overrides. Style callbacks are owned/cloneable and replace a widget's style rather than merging it. An absent field should invoke no builder and preserve existing local presentation. Component aggregates and the in-scope widget `apply` adapters merge explicitly supplied fields; excluded parent-owned adapters remain listed below.

`TextProps::apply_to_text` configures a native `Text` without a wrapper. `TextProps::apply` additionally implements explicit `Clip`/`Grow` adapters. `Wrap` uses word-or-glyph wrapping; `Clip` clips drawing, not the source string; `Grow` releases horizontal layout constraints, not ancestor clipping. There is no builder-time overflow detection or global forced wrapping. On `apply_to_text`, `Clip`/`Grow` only disable wrapping: use `apply` for their full behavior.

## Inventory and evidence

**E** = public `Default + Clone` export smoke coverage; **B** = actual public construction/conversion; **R** = observable layout/tree/event assertion. E alone is neither a forwarding test nor proof that all fields are useful.

| Family / meaningful parent-built children | Public typed paths                                                                                                                                                                                                                                                                                                   | Evidence in this suite                                                                                                                                                    |
| ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Native text, surface, action, entry       | `TextProps`, `ContainerProps`, `ButtonProps`, `EntryProps` → `TextInputProps`                                                                                                                                                                                                                                        | E/B/R: native builders, default geometry/leaf tree, Unicode wrap layout, click publication, nested input width                                                            |
| Basic components                          | `AvatarProps`, `BadgeProps`, `ChipProps`, `RoundBadgeProps`, `ButtonContentProps`, `CircularButtonProps`, `ToggleGroupProps`, `ColorSwatchProps`, `LinkButtonProps`, `TagProps`, `SectionButtonProps`, `SectionIconProps`                                                                                            | E; ToggleGroup label chain B via ChoiceRow; remaining paths need field-level regressions                                                                                  |
| Input and help components                 | `EntryInputProps`, `TextInputProps`, `TextEditorProps`, `TextViewProps`, `InputWrapperProps`, `SearchBarProps`, `IconToggleButtonProps`, `TooltipProps`                                                                                                                                                              | E; SearchBar B/R for actual input, close button and surface; EntryInput default regression active; partial SearchBar/InputWrapper and IconToggle state/action regressions |
| Moved slider / appearance atoms           | `components::{Scale, ScaleProps, RangeSlider, RangeSliderProps, ThemeToggle, ThemeMode}`; identical `widgets` compatibility exports                                                                                                                                                                                  | E/B/R for Scale's nested horizontal width and vertical height; B/type identity for ThemeToggle; not a claim of nested ThemeToggle props                                   |
| Row/adaptive composition                  | `ActionRowProps`, `ComboRowProps`, `ToggleGroupRowProps`, `ChoiceRow::{row_props,toggle_props,dropdown_props}`, `EntryRowProps`; TreeRow's row/title/handle/children setters                                                                                                                                         | E/B/R for ActionRow/ComboRow merges and actions; native nested types passed through multiple parent levels; adaptive branches not exhaustively asserted                   |
| Selection/menu surfaces                   | `DropdownItemProps`, `DropdownOptions::{label_props,option_props,trigger_props,combobox_props,dropdown_props}`, `MenuItem` text/container/button setters, `Menu` title/subtitle/surface setters, `MenuButton` text-trigger/label/trigger/combobox setters, `ContextMenuProps`, `PopoverProps`, `PopoverSurfaceProps` | E for exported props; menu sheet/text trigger B; sheet item button geometry R; context target geometry, left-action preservation and right-click native overlay R         |
| Input/picker compositions                 | `AccentPickerProps`, `ChipGroupProps`, `ColorInputProps`, `ColorPickerProps`, `DateInputProps`, `DateRangeInputProps`, `NumberInputProps`, `IconPickerProps`; Autocomplete/MultiSelect direct typed setters                                                                                                          | E where an aggregate is exported; NumberInput B including Entry → native input; others not field/branch validated                                                         |
| Navigation/chrome                         | `HeaderBarProps`, `SidebarRowProps`, `ToolbarProps`; Tabs, Pagination, SortHeader and SplitButton direct typed child setters                                                                                                                                                                                         | E for exported aggregates, HeaderBar B; direct-setter-only widgets not exercised here                                                                                     |
| Content/feedback                          | `BannerProps`, `CardProps`, `ClampProps`, `PageProps`, `StatusPageProps`, `PreferencesGroupProps`, `PreferencesPageProps`, `ProfileHeaderProps`, `SectionDividerProps`, `SectionHeadingProps`, `ToastProps`                                                                                                          | E; PreferencesGroup → Card → child container, StatusPage and SectionHeading B; remaining forwarding not asserted                                                          |
| Dialogs/workflows                         | `AboutDialogProps`, `AlertDialogProps`, `DialogFrameProps`, `ShortcutsDialogProps`, `FormWidgetProps`, `StepperWidgetProps`, `AccordionProps`, `AccordionItemProps`, `CollapsibleProps`                                                                                                                              | E/B/R for Form/Stepper merges and submit/busy policy; remaining paths need branch/field coverage                                                                          |
| Collections/structured content            | `InfiniteListWidgetProps`, `ItemCardProps`, `ItemRowProps`, `ReorderableListWidgetProps`, `ResponsiveTableWidgetProps`, `RichTextWidgetProps`, `RichTextLineProps`, `SpoilerWidgetProps`, `SyncedTextWidgetProps`, `TagFilterProps`, `TagGroupListProps`, `TimelineWidgetProps`, `TreeProps`                         | E; aggregate B/R for InfiniteList, ReorderableList, Spoiler, SyncedText, Timeline, ResponsiveTable and RichText; other paths/branches not exhaustively validated          |
| Function-built helpers                    | `CarouselIndicatorWidgetProps`, `PathBarWidgetProps`, `ShortcutLabelWidgetProps`, `TabButtonWidgetProps`, `ViewSwitcherBarWidgetProps` and public `*_with_props` helpers                                                                                                                                             | E only; helper invocation/forwarding remains untested here                                                                                                                |

The export smoke test enumerates the names explicitly: removing a public export breaks a downstream compile rather than silently shrinking a discovered list. The inventory also includes direct setter APIs without aggregate props; inventing a same-named empty aggregate would not improve their coverage.

### Aliases that are not fake coverage

- `ChoiceToggleProps = ToggleGroupRowProps` and `ChoiceDropdownProps = ComboRowProps` are compatibility names for the **actual nested row types**. `ChoiceRow::toggles`/`dropdown` call those types' `apply`. Tests pass native row/group/dropdown props, not unrelated shape aliases.
- `Field`/`FieldProps` re-export the actual `InputWrapper`/`InputWrapperProps`.
- `widgets::{Scale, ScaleProps, RangeSlider, RangeSliderProps, ThemeToggle, ThemeMode}` re-export component-owned implementations. Assignment in both directions proves identity; this is compatibility, not additional child coverage.

### Justified exceptions, not whole-parent exemptions

- Caller `Element` slots own their internal presentation: ActionRow prefix/suffix, Entry sections, InputWrapper control, Card children, ToolbarView bars/content, dialog content and custom table/list renderers. Configure these before passing them in. Parent-built wrappers/separators/buttons around them still need their own meaningful typed forwarding.
- `EntryInput::Custom` and the `Select.options` element are caller-owned. This does **not** exempt EntryRow's internally built label, selected-value text, description or validation message.
- RangeSlider rails/thumbs, Switch artwork, Spinner/SpectrumBars/ProgressBlocks, and Carousel indicators are custom drawing rather than nested native text or button widgets. Use their actual geometry/style builders or tokens, not fake `TextProps`/`ButtonProps`. Carousel pages remain caller elements.
- Divider's label is a caller element; its line is custom drawing. Segmented's item contents are supplied elements and its segment targets are custom drawn; ToggleGroup still constructs labels itself and exposes real `TextProps`.
- Icons are artwork views, not hidden text labels. Exempting an icon does not exempt the button, container or tooltip that a parent builds around it.

## Component aggregate merge fix

`apply` now overlays only explicitly supplied child presentation fields for `EntryInputProps`, `TextViewProps`, `EntryProps`' optional native-input override, `SearchBarProps`, `InputWrapperProps`, `ButtonContentProps`, `ToggleGroupProps`, `CircularButtonProps`, `AvatarProps`, `BadgeProps`, `ChipProps`, `RoundBadgeProps`, `IconToggleButtonProps`, `SectionIconProps` and `SectionButtonProps`.

The concrete props types and public aggregate shapes are unchanged. `None` means retain the child's existing field; `Some(value)` wins, including `Some(false)`, zero padding/gaps, and explicit callback replacements. Nested tooltip labels merge recursively. Callback `Arc`s are cloned, not borrowed or combined. An aggregate default is a no-op even on an already customized child. `EntryProps::input_props(Some(default))` is now also a no-op rather than a reset. Native widget builders and direct `.input_props(...)`/`.label_props(...)`/`.props(...)` setters retain their existing replacement semantics; use them when a deliberate reset is needed.

Public regressions exercise default and partial Entry input overlays, SearchBar input/button/surface dimensions and close publication, ButtonContent/InputWrapper text dimensions, and IconToggle active/disabled behavior. Focused component unit tests check every native text/container/button merge field, preservation/replacement of owned styles, Entry value/password/ID/validation/actions, TextView content/growth/actions, and recursively merged tooltip presentation. These are representative adapter tests, not full branch coverage of all fifteen adapters.

## PROPS-MERGE bounded sprint — closed

Reported issues: nine widget aggregates replaced the whole stored props object; optional adapters such as `ActionRowProps` replaced whole nested native props. Acceptance: customized defaults and explicit empty child overrides remain no-ops, partial overlays retain omitted fields and siblings, and presentation does not change parent events/state/security. No public props fields or controlled-state APIs were added.

Fixed aggregate `apply` paths: `FormWidgetProps`, `InfiniteListWidgetProps`, `ReorderableListWidgetProps`, `SpoilerWidgetProps`, `SyncedTextWidgetProps`, `TimelineWidgetProps`, `StepperWidgetProps`, `ResponsiveTableWidgetProps`, and `RichTextWidgetProps`. Recursive merging includes Stepper indicators, Card props, dropdown items, popover trigger/surface props, and rich-text-line props. Native style callbacks retain their owned `Arc` unless explicitly replaced.

Fixed optional nested `apply` paths in `accent_picker`, `accordion`, `banner`, `chip_group`, `collapsible`, `color_picker`, `date_input`, `date_range_input`, `dialog`, `header_bar`, `item_card`, `item_row`, `number_input`, `preferences`, `profile_header`, `rows`, `section_divider`, `section_heading`, `status_page`, `tag_filter`, `tag_group_list`, `toast`, `toolbar`, and `tree`. Scalar presentation options still apply only when supplied; nested props merge instead of invoking replacing child setters. TagFilter merges its stored EntryRowProps through its public fields without editing the parent-owned EntryRow implementation.

Optional child objects preserve absence: an empty `Some(ContainerProps::default())` does not create Page/heading/group wrappers. Explicit `false`, zero, and nonempty overrides remain meaningful. Direct replacing setters retain their previous semantics; `.apply` is the additive overlay operation. Function-built `*_with_props` helpers build fresh children, so they have no prior-builder reset to merge.

Deterministic downstream evidence covers all nine aggregates with customized baselines, default overlays, explicit empty child overlays, and height-only overlays retaining width. It also covers ActionRow subtitle siblings/actions, Card child dimensions, Page wrapper absence, and Form submit/busy policy. Focused internal regressions cover nested Form → Stepper, table → Card/dropdown/popover, rich-text-line callbacks, and ComboRow selection/disabled callbacks. This is acceptance for the reported merge defects, not exhaustive field/branch or visual coverage of every widget.

### Remaining findings / next batch for parent

- **MERGE-EXCLUDED-1 resolved:** `EntryRowProps::apply` merges label/button/popover/input fields. Regressions verify partial overlays retain earlier dimensions, explicit `false`/zero override, controlled state remains intact, and empty optional input props stay absent. The additional EntryRow presentation gaps below remain open. Navigation uses fresh-child helpers/direct setters rather than this aggregate `apply` path.
- **MERGE-COVERAGE-1:** The other optional adapters now use native merges, but most have only existing default/native-forwarding unit coverage, not a new per-field regression for every branch. Next checks: all affected optional families under non-default baselines, style evaluation at draw time, responsive variants, and empty/loading/error/disabled branches. Do not infer exhaustive audit completion from the nine representative native text checks.
- **HELPER-COVERAGE-1:** Public `*_with_props` invocation and branch coverage remains incomplete. These are fresh-child forwarding tests, not aggregate-reset bugs. Gallery/toy adoption, workspace validation and screenshots remain parent follow-ups; no demo or effect-shape files were changed.

## Parent-owned integration findings

These findings are outside this component-adapter change; parent work may supersede them.

1. **Aggregate menu API resolved.** `widgets::MenuProps { popover, trigger, label }` is publicly exported. The downstream suite explicitly names the type and constructs a text-trigger menu with `.props(MenuProps { ... })`.

2. **ThemeToggle is genuinely composed, not a custom-drawing exemption.** `src/components/theme_toggle.rs` builds a container, native button and tooltip, but exposes only padding/metrics plus controlled mode/action/disabled policy. Add real `ButtonProps`/`TooltipProps` (and meaningful container presentation) forwarding without exposing the mode or callback through presentation props. Keep component/widget aliases identical. Current tests verify only identity and construction, not nonexistent child APIs.
3. **EntryRow does not forward all meaningful built-in presentation.** In `src/widgets/entries.rs`, the `Select` branch constructs selected-value `text(...)` directly. The outer internally constructed `InputWrapper` gets description/validity/interaction but no typed wrapper props. Expose value `TextProps` and `InputWrapperProps` (or exact description/message child props) without allowing presentation to replace validity/interaction. The multiline branch has no row-level `TextViewProps` forwarding; callers can configure a supplied TextView, but convenience-built multiline controls need a considered policy rather than a blanket caller-slot exemption.

No inert sheet-menu button API was found: `touch_lists` reaches `pressed`, which applies `MenuItem::button_props`; the active sheet layout regression verifies it. ContextMenu intentionally reapplies flat appearance/zero trigger padding after surface props so the caller target stays unchanged; those two fields are not permission to restyle the target. A narrower context-only surface type could make this limitation clearer.

## Validation and remaining work

```sh
cargo test --test child_props_public --offline
rustfmt --edition 2024 --check tests/child_props_public.rs
cargo test --lib --offline native_props_tests
cargo test --lib --offline nested_props_tests
cargo test --lib --offline
```

PROPS-MERGE validation in the actual working tree: downstream suite **21 passed / 0 ignored**; focused native-props suite **14 passed**; focused nested-props suite **13 passed**; full library suite **997 passed / 0 ignored**. Scoped `rustfmt` ran only on changed Rust files; scoped `dprint` validates this document. Library-test warnings remain (float fallback and navigation test dead-code warnings); compilation/interaction-layout tests are not visual acceptance. The formerly ignored EntryInputProps regression remains enabled.

Remaining audit work: non-default field forwarding and default-preservation for all E-only families; all responsive presentations and empty/loading/error/disabled branches; callback/state/validation/security preservation beyond the exercised native click/context paths; style callback evaluation at draw time; context menu keyboard/submenu/dismissal behavior; public helper invocation. Existing private unit tests may cover some of these, but are not substituted for downstream coverage here. No exhaustive absence-of-inert-props, full-workspace, visual, screenshot or pixel-perfect acceptance claim is made.
