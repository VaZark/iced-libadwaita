//! The gallery's content: a list of components per group, and one page per component that
//! shows all of its properties and states (disabled, validation, color override, …).

use super::*;
use iced::widget::{column, container, row, text};

/// Children stacked vertically with `spacing`.
fn stack<'a>(spacing: f32, children: Vec<Element<'a>>) -> iced::widget::Column<'a, Message, Theme> {
    column(children).spacing(spacing)
}

/// Moves a pending list of rows into the page.
fn flush<'a>(
    col: iced::widget::Column<'a, Message, Theme>,
    list: &mut Option<w::PreferencesGroup<'a, Message>>,
) -> iced::widget::Column<'a, Message, Theme> {
    match list.take() {
        Some(g) => col.push(g),
        None => col,
    }
}

impl Gallery {
    /// Identifies the page being shown, so switching it fades the new one in.
    pub(super) fn view_key(&self) -> u64 {
        (self.page as u64) * 1000 + self.detail.map_or(0, |d| d as u64 + 1)
    }

    pub(super) fn content_page(&self) -> Element<'_> {
        match (self.page, self.detail) {
            (Page::Typography, _) => self.typography_page(),
            (page, None) => self.group_page(page),
            (_, Some(d)) => self.detail_page(d),
        }
    }

    fn group<'a>(&self, title: &'static str) -> w::PreferencesGroup<'a, Message> {
        w::PreferencesGroup::<Message>::new()
            .title(title)
            .metrics(self.metrics())
    }

    /// Level 1: the group's components. Components with two or more significantly different
    /// properties open a nested page; single-trigger ones get a button on their row (see
    /// `README.md`).
    fn group_page(&self, page: Page) -> Element<'_> {
        let m = self.metrics();
        let mut col = column![].spacing(m.group_spacing);
        let mut first = true;
        for (title, entries) in page.sections() {
            // Consecutive rows (nested / action) share one list.
            let mut list: Option<w::PreferencesGroup<'_, Message>> = None;
            for entry in entries {
                match entry {
                    Entry::Nested(d) => {
                        let row = w::ActionRow::new(d.title())
                            .subtitle(d.description())
                            .chevron(true)
                            .on_activate(Message::OpenDetail(d))
                            .metrics(m);
                        let mut g = list.take().unwrap_or_else(|| {
                            let g = self.group(title);
                            if std::mem::take(&mut first) {
                                g.description(page.description())
                            } else {
                                g
                            }
                        });
                        g = g.push(row);
                        list = Some(g);
                    }
                }
            }
            col = flush(col, &mut list);
        }
        self.page_frame(col)
    }

    /// What the Open button of a dialog, sheet or shell page sends.
    fn open_message(d: Detail) -> Option<Message> {
        Some(match d {
            Detail::AlertDialog => Message::OpenDialog(Dialog::Alert),
            Detail::AboutDialog => Message::OpenDialog(Dialog::About),
            Detail::ShortcutsDialog => Message::OpenDialog(Dialog::Shortcuts),
            Detail::PreferencesDialog => Message::OpenDialog(Dialog::Preferences),
            Detail::BottomSheet => Message::Sheet(true),
            Detail::AppShell => Message::OpenShell,
            _ => return None,
        })
    }

    /// Level 2: one component, grouped by property.
    fn detail_page(&self, d: Detail) -> Element<'_> {
        let m = self.metrics();
        let noop = |_: String| Message::Noop;
        let lbl = |s: &'static str| -> Element<'static> { w::typography::label(s).into() };
        let sp = m.space_m;
        let g = |t| self.group(t);

        // A page with a playground shows it: one live component, its controls and its code.
        let played = self.plays.get(&d).and_then(|p| {
            plays::build(self, d, p, m)
                .map(|(el, code)| p.view(m, el, code, move |x| Message::Play(d, x)))
        });
        let body: Element<'_> = if let Some(b) = played {
            b
        } else {
            match d {
            Detail::Button => {
                let enabled = row![
                    w::raised_button(lbl("Default")).on_press(Message::Noop),
                    w::flat_button(lbl("Flat")).on_press(Message::Noop),
                    w::suggested_button(lbl("Suggested")).on_press(Message::Noop),
                    w::destructive_button(lbl("Destructive")).on_press(Message::Noop),
                ]
                .spacing(sp)
                .wrap();
                let disabled = row![
                    w::raised_button(lbl("Default")),
                    w::flat_button(lbl("Flat")),
                    w::suggested_button(lbl("Suggested")),
                    w::destructive_button(lbl("Destructive")),
                ]
                .spacing(sp)
                .wrap();
                let shapes = row![
                    w::pill_button(lbl("Pill")).on_press(Message::Noop),
                    w::suggested_pill_button(lbl("Suggested Pill")).on_press(Message::Noop),
                    w::circular_button(Icon::ListAdd).on_press(Message::Noop),
                    w::icon_button(Icon::Delete).on_press(Message::Noop),
                ]
                .spacing(sp)
                .align_y(iced::Alignment::Center)
                .wrap();
                let shapes_disabled = row![
                    w::pill_button(lbl("Pill")),
                    w::circular_button(Icon::ListAdd),
                    w::icon_button(Icon::Delete),
                ]
                .spacing(sp)
                .align_y(iced::Alignment::Center)
                .wrap();
                column![
                    g("Styles").push(w::card(enabled)),
                    g("Disabled").push(w::card(disabled)),
                    g("Shapes").push(w::card(shapes)),
                    g("Disabled shapes").push(w::card(shapes_disabled)),
                ]
                .spacing(m.group_spacing)
                .into()
            }
            Detail::ButtonContent => {
                let content = row![
                    w::raised_button(Element::from(w::ButtonContent::new("Open").icon(Icon::Home).metrics(m)))
                        .on_press(Message::Noop),
                    w::suggested_button(Element::from(w::ButtonContent::new("Add").icon(Icon::ListAdd)))
                        .on_press(Message::Noop),
                    w::raised_button(Element::from(w::ButtonContent::new("Disabled").icon(Icon::Home).metrics(m))),
                ]
                .spacing(sp)
                .align_y(iced::Alignment::Center)
                .wrap();
                // SplitButton: a main action plus a menu of other actions.
                let save_menu = || {
                    w::Menu::new("Save")
                        .section([
                            w::MenuItem::action("Save as…", Message::ShowToast("Save as…")),
                            w::MenuItem::action("Save a copy", Message::ShowToast("Saved a copy")),
                        ])
                        .section([w::MenuItem::action("Export…", Message::ShowToast("Export")).disabled()])
                };
                let split = row![
                    w::split_button(text("Save"), Message::ShowToast("Saved"), save_menu()),
                    w::SplitButton::new(text("Save"), Message::Noop, save_menu()).disabled(true),
                ]
                .spacing(sp)
                .wrap();
                // SplitButtonAction: the menu chooses what the button does.
                let merge = || {
                    [
                        w::SplitAction::new("merge", "Create merge commit").icon(Icon::ListAdd),
                        w::SplitAction::new("squash", "Squash and merge"),
                        w::SplitAction::new("rebase", "Rebase and merge"),
                        w::SplitAction::new("admin", "Merge without checks").disabled(),
                    ]
                };
                let action = row![
                    w::SplitButtonAction::new(merge(), self.split_choice, Message::SplitChoice, Message::SplitRun)
                        .open_initially(self.shots.is_some() && self.detail == Some(Detail::ButtonContent)),
                    w::SplitButtonAction::new(merge(), self.split_choice, Message::SplitChoice, Message::SplitRun)
                        .disabled(true),
                ]
                .spacing(sp)
                .wrap();
                column![
                    g("Icon and label").push(w::card(content)),
                    g("Split button")
                        .description("A button joined to a menu of other actions: the arrow opens the menu and choosing an item runs it.")
                        .push(w::card(split)),
                    g("Split button action")
                        .description("The menu chooses what the button does: pick an item to change its label and action, then press the button.")
                        .push(w::card(action)),
                ]
                .spacing(m.group_spacing)
                .into()
            }
            Detail::ToggleGroup => {
                let items = || {
                    vec![
                        w::ToggleItem::new(Tab::One, "One"),
                        w::ToggleItem::new(Tab::Two, "Two"),
                        w::ToggleItem::new(Tab::Three, "Three"),
                    ]
                };
                let icons = vec![
                    w::ToggleItem::new(Tab::One, "Files").icon(Icon::Home),
                    w::ToggleItem::new(Tab::Two, "Search").icon(Icon::Search),
                    w::ToggleItem::new(Tab::Three, "Settings").icon(Icon::Settings),
                ];
                let crowded = vec![
                    w::ToggleItem::new(Tab::One, "Files").icon(Icon::Home),
                    w::ToggleItem::new(Tab::Two, "Search").icon(Icon::Search),
                    w::ToggleItem::new(Tab::Three, "Settings").icon(Icon::Settings),
                    w::ToggleItem::new(Tab::One, "Library").icon(Icon::Edit),
                    w::ToggleItem::new(Tab::Two, "Archive").icon(Icon::Delete),
                ];
                column![
                    g("Text").push(w::card(Element::from(w::toggle_group(items(), Some(self.tab), Message::Tab)))),
                    g("Icons and text").push(w::card(Element::from(w::toggle_group(icons, Some(self.tab), Message::Tab)))),
                    g("Crowded: fills, shrinks, falls back to icons").push(w::card(Element::from(
                        w::toggle_group(crowded, Some(self.tab), Message::Tab).width(Length::Fill),
                    ))),
                    g("In a row").push(
                        w::ToggleGroupRow::new("View", items(), Some(self.tab), Message::Tab)
                            .subtitle("A toggle group as a row suffix")
                            .metrics(m),
                    ),
                ]
                .spacing(m.group_spacing)
                .into()
            }

            Detail::TextInput => {
                let states = stack(sp, vec![
                    w::entry("Normal", &self.name).on_input(Message::Name).metrics(m).into(),
                    w::entry("Placeholder", "").on_input(noop).metrics(m).into(),
                    w::entry("Error", "not a valid value").on_input(noop).state(w::EntryState::Error).metrics(m).into(),
                    w::entry("Warning", "check this").on_input(noop).state(w::EntryState::Warning).metrics(m).into(),
                    w::entry("Success", "looks good").on_input(noop).state(w::EntryState::Success).metrics(m).into(),
                    w::entry("Disabled", "read only").metrics(m).into(),
                ]);
                let password = stack(sp, vec![
                    w::password_entry("Password", &self.password).on_input(Message::Password).metrics(m).into(),
                    w::password_entry("Disabled", "hunter2").metrics(m).into(),
                ]);
                let color = stack(sp, vec![
                    w::entry("Purple", "text and ring override").on_input(noop).color(adw::color::hex(0x9141ac)).metrics(m).into(),
                    w::entry("Teal", "any color").on_input(noop).color(adw::color::hex(0x2190a4)).metrics(m).into(),
                    w::entry("Orange", "disabled keeps it dim").color(adw::color::hex(0xe66100)).metrics(m).into(),
                ]);
                let width = stack(sp, vec![
                    w::entry("Fixed width", "200px").on_input(noop).width(Length::Fixed(200.0)).metrics(m).into(),
                    w::entry("Fills", "fills the card").on_input(noop).metrics(m).into(),
                ]);
                // GALLERY_ONLY=sections: only the sections demo (a screenshot is one window tall).
                if std::env::var("GALLERY_ONLY").as_deref() == Ok("sections") {
                    return self.sections.view(m).map(Message::Sections);
                }
                column![
                    g("States").push(w::card(states)),
                    g("Password").push(w::card(password)),
                    g("Color override").push(w::card(color)),
                    g("Width").push(w::card(width)),
                    self.sections.view(m).map(Message::Sections),
                ]
                .spacing(m.group_spacing)
                .into()
            }
            Detail::TextView => {
                let editable = stack(sp, vec![w::text_view(&self.editor).on_action(Message::Editor).into()]);
                let placeholder = stack(sp, vec![w::text_view(&self.editor_empty)
                    .placeholder("Type here…")
                    .on_action(Message::EditorEmpty)
                    .into()]);
                let readonly = stack(sp, vec![w::text_view(&self.editor_ro).into()]);
                column![
                    g("Editable").push(w::card(editable)),
                    g("Placeholder").push(w::card(placeholder)),
                    g("Read-only (disabled)").push(w::card(readonly)),
                ]
                .spacing(m.group_spacing)
                .into()
            }
            Detail::Field => self.field.view(m).map(Message::Field),
            Detail::Form => self.form.view(m).map(Message::Form),
            Detail::PathBar => self.inline(Inline::PathBar),
            Detail::MenuButton => self.inline(Inline::MenuButton),
            Detail::Banner => self.inline(Inline::Banner),
            Detail::Progress => self.inline(Inline::Progress),
            Detail::BadgeChip => self.inline(Inline::BadgeChip),
            Detail::Indicator => self.inline(Inline::Indicator),
            Detail::Docked => self.inline(Inline::Docked),
            Detail::ResponsiveTable => self.inline(Inline::ResponsiveTable),
            Detail::Accordion => self.inline(Inline::Accordion),
            Detail::CardButton => self.inline(Inline::CardButton),
            Detail::Carousel => self.inline(Inline::Carousel),
            Detail::DateInput => self.inline(Inline::DateInput),
            Detail::RangeSlider => self.inline(Inline::RangeSlider),
            Detail::DateRange => self.inline(Inline::DateRange),
            Detail::MetaComponent => self.sections.meta_view(m).map(Message::Sections),
            Detail::MetaParent => self.reorder.parent_view(m).map(Message::Reorder),
            Detail::Reorderable => self.reorder.view(m).map(Message::Reorder),
            Detail::Resizable => self.resizable.view(m).map(Message::Resizable),
            Detail::LinkButton => self.inline(Inline::LinkButton),
            Detail::Switch => self.inline(Inline::Switch),
            Detail::CheckButton => self.inline(Inline::CheckButton),
            Detail::RadioButton => self.inline(Inline::RadioButton),
            Detail::Scale => self.inline(Inline::Scale),
            Detail::ButtonRow => self.inline(Inline::ButtonRow),
            Detail::Clamp => self.inline(Inline::Clamp),
            Detail::StatusPage => self.inline(Inline::StatusPage),
            Detail::WrapBox => self.inline(Inline::WrapBox),
            Detail::Collapse => self.inline(Inline::Collapse),
            Detail::ViewSwitcher => self.inline(Inline::ViewSwitcher),
            Detail::Avatar => self.inline(Inline::Avatar),
            Detail::Tag => self.inline(Inline::Tag),
            Detail::ShortcutLabel => self.inline(Inline::ShortcutLabel),
            Detail::MultiSelect => self.inline(Inline::MultiSelect),
            Detail::AnchoredPopover => self.inline(Inline::AnchoredPopover),
            Detail::Surfaces => self.inline(Inline::Surfaces),
            Detail::AlertDialog
            | Detail::AboutDialog
            | Detail::ShortcutsDialog
            | Detail::PreferencesDialog
            | Detail::BottomSheet
            | Detail::AppShell => column![
                g(d.title())
                    .description(d.description())
                    .push(w::card(row![
                        w::raised_button(text("Open")).on_press_maybe(Self::open_message(d))
                    ])),
            ]
            .into(),
            // Shown by its playground (see `plays.rs`).
            Detail::Tabs | Detail::Tree | Detail::TreeRow => column![].into(),
            Detail::TableVirtual => self.virtual_demo.virtual_view(m).map(Message::Virtual),
            Detail::TableAsync => self.virtual_demo.async_view(m).map(Message::Virtual),
            Detail::TreeAsync => self.tree_demo.async_view(m).map(Message::Tree),
            Detail::TreeVirtualized => self.tree_demo.virtual_view(m).map(Message::Tree),
            Detail::TreeFiles => self.tree_demo.files_view(m).map(Message::Tree),
            Detail::TreeDocs => self.tree_demo.docs_view(m).map(Message::Tree),
            Detail::Stepper => self.stepper.view(m).map(Message::Stepper),
            Detail::Data => self.data.view(m).map(Message::Data),
            Detail::Autocomplete => self.autocomplete.view(m).map(Message::Autocomplete),
            Detail::Dropdown => {
                let plain = stack(sp, vec![
                    w::Field::new("Choose a fruit", Element::from(w::dropdown(&FRUITS, self.picked, Message::Picked))).layout(w::FieldLayout::Row).plain_label(true).metrics(m).into(),
                    w::Field::new("Selected", Element::from(w::dropdown(&FRUITS, Some("Banana"), Message::Picked))).layout(w::FieldLayout::Row).plain_label(true).metrics(m).into(),
                    w::Field::new("Disabled", Element::from(w::dropdown_disabled::<&str, Message>(Some(&"Apple")))).layout(w::FieldLayout::Row).plain_label(true).metrics(m).into(),
                ]);
                column![
                    g("Dropdown").push(w::card(plain)),
                    g("Combo row")
                        .description("The selected option is the accent color; a disabled option is dimmed and cannot be chosen.")
                        .push(
                            w::ComboRow::new("Font", &COMBO, self.combo, Message::Combo)
                                .subtitle("“Inter” is disabled")
                                .disabled_when(|o| *o == "Inter")
                                .open_initially(self.shots.is_some()),
                        )
                        .push(
                            w::ComboRow::new("Disabled row", &COMBO, Some("Cantarell"), Message::Combo2)
                                .subtitle("Insensitive")
                                .disabled(true),
                        ),
                ]
                .spacing(m.group_spacing)
                .into()
            }

            Detail::ActionRow => column![
                g("Action row")
                    .push(w::ActionRow::new("Title only").metrics(m))
                    .push(w::ActionRow::new("With subtitle").subtitle("A secondary line").metrics(m))
                    .push(
                        w::ActionRow::new("Ada Lovelace")
                            .subtitle("Prefix avatar and a chevron")
                            .prefix(w::avatar(40.0, "Ada Lovelace"))
                            .chevron(true)
                            .on_activate(Message::ShowToast("Opened account"))
                            .metrics(m),
                    )
                    .push(
                        w::ActionRow::new("Suffix")
                            .subtitle("A button at the end")
                            .suffix(w::raised_button(lbl("Action")).on_press(Message::Noop))
                            .metrics(m),
                    ),
            ]
            .into(),
            Detail::SwitchAndComboRows => column![
                g("Switch rows")
                    .push(w::SwitchRow::new("Enabled", self.row_sw, Message::RowSw).metrics(m))
                    .push(w::SwitchRow::new("Disabled", true, |_| Message::Noop).disabled(true).metrics(m)),
                g("Combo rows")
                    .push(
                        w::ComboRow::new("Font", &COMBO, self.combo, Message::Combo)
                            .subtitle("Interface font")
                            .disabled_when(|o| *o == "Inter"),
                    )
                    .push(w::ComboRow::new("Disabled", &COMBO, Some("Cantarell"), Message::Combo2).disabled(true)),
                g("Spin and slider rows")
                    .push(w::SpinRow::new("Text scale", self.spin, 1.0..=10.0, 1.0, Message::Spin))
                    .push(w::SliderRow::new("Volume", self.slider, 0.0..=100.0, Message::Slider).basis(260.0).metrics(m)),
            ]
            .spacing(m.group_spacing)
            .into(),
            Detail::EntryRow => column![
                g("Entry rows")
                    .push(w::EntryRow::new("Display name", &self.name).on_input(Message::Name).metrics(m))
                    .push(w::EntryRow::new("Password", &self.password).on_input(Message::Password).password(true).metrics(m))
                    .push(w::EntryRow::new("Read-only", "disabled row").metrics(m)),
            ]
            .into(),
            Detail::PreferencesGroup => column![
                self.preferences_demo(),
            ]
            .into(),

            Detail::Toast => {
                let spec = |title, action, status| Message::ShowToastSpec(ToastSpec { title, action, status });
                let show = |label: &'static str, msg: Message| w::raised_button(text(label)).on_press(msg);
                let action = row![
                    show("Plain", Message::ShowToast("Settings saved")),
                    show("With action", spec("File deleted", Some("Undo"), None)),
                ]
                .spacing(sp)
                .wrap();
                let statuses = || {
                    row![
                        show("Success", spec("Changes saved", None, Some(Role::Success))),
                        show("Warning", spec("Low disk space", None, Some(Role::Warning))),
                        show("Error", spec("Upload failed", Some("Retry"), Some(Role::Error))),
                        show("Info", spec("New version available", Some("Update"), Some(Role::Accent))),
                    ]
                    .spacing(sp)
                    .wrap()
                };
                column![
                    g("Call to action").description("An optional button on the toast.").push(w::card(action)),
                    g("Status dot").description("A colored dot marks the type of message.").push(w::card(statuses())),
                    g("Stacked notifications")
                        .description("Several toasts at once, one column at the position: newest nearest the edge. At most four are kept.")
                        .push(w::card(
                            row![
                                show("Add toast", spec(
                                    ["Message sent", "Download finished", "Sync complete", "Backup created"][self.toast_seq as usize % 4],
                                    None,
                                    Some([Role::Success, Role::Accent, Role::Warning, Role::Error][self.toast_seq as usize % 4]),
                                )),
                                show("Clear all", Message::ClearToasts),
                            ]
                            .spacing(sp)
                            .wrap(),
                        )),
                    g("Auto-dismiss")
                        .description("The queue's timeout: each toast leaves on its own clock, restarted when its content is replaced (try Undo). Off, they stay until dismissed.")
                        .push(w::SwitchRow::new("Dismiss after 4 s", self.toast_auto, Message::ToastAuto).metrics(m)),
                    g("Radius")
                        .description("A toast's corner radius is its own theme token (default: a full pill).")
                        .push(
                            w::SliderRow::new("Toast radius", self.radius_toast, 0.0..=Metrics::MAX_RADIUS_TOAST, Message::RadiusToast)
                                .subtitle(format!("radius_toast = {:.0}px", self.radius_toast))
                                .metrics(m),
                        ),
                    g("Placement")
                        .description("Where toasts appear, and the container they are placed in. Both are desktop config settings (`[toasts]`).")
                        .push(w::ComboRow::new("Position", &TOAST_POSITIONS, Some(self.toast_position), Message::ToastPosition))
                        .push(
                            w::ToggleGroupRow::new(
                                "Container",
                                vec![
                                    w::ToggleItem::new(w::ToastTarget::Window, "Window").icon(Icon::Window),
                                    w::ToggleItem::new(w::ToastTarget::Content, "Content").icon(Icon::Sidebar),
                                ],
                                Some(self.toast_target),
                                Message::ToastTarget,
                            )
                            .subtitle("The whole window, or only the content area beside the sidebar")
                            .metrics(m),
                        ),
                ]
                .spacing(m.group_spacing)
                .into()
            }
            // ------------------------------------------------------------ theming
            Detail::ThemeColors => column![
                g("Scheme and accent").push(w::SwitchRow::new("Dark", self.scheme.is_dark(), Message::Scheme).metrics(m)).push(
                    w::ActionRow::new("Accent color")
                        .suffix(w::accent_picker(self.accent, Message::Accent))
                        .metrics(m),
                ),
                g("Overrides")
                    .description("Edits to the theme's tokens.")
                    .push(w::SwitchRow::new("Compact metrics", self.compact, Message::Compact).metrics(m))
                    .push(w::SwitchRow::new("Override card color", self.card_override, Message::CardOverride).metrics(m)),
            ]
            .spacing(m.group_spacing)
            .into(),
            Detail::ThemeShape => {
                // Everything here reads the theme's radii at draw time, so the sliders visibly
                // change it: control radius rounds buttons, entries and toggles; card radius
                // rounds the card and boxed list; toast radius rounds toasts.
                let preview = column![
                    row![
                        w::raised_button(lbl("Default")).on_press(Message::Noop),
                        w::suggested_button(lbl("Suggested")).on_press(Message::Noop),
                        w::destructive_button(lbl("Destructive")).on_press(Message::Noop),
                    ]
                    .spacing(m.space_s)
                    .wrap(),
                    w::entry("Entry", &self.name).on_input(Message::Name).metrics(m),
                    Element::from(w::toggle_group(
                        vec![
                            w::ToggleItem::new(Tab::One, "One"),
                            w::ToggleItem::new(Tab::Two, "Two"),
                            w::ToggleItem::new(Tab::Three, "Three"),
                        ],
                        Some(self.tab),
                        Message::Tab,
                    )),
                    w::boxed_list([
                        Element::from(w::ActionRow::new("Row in a boxed list").subtitle("Rounded by the card radius").metrics(m)),
                        Element::from(w::ActionRow::new("Second row").metrics(m)),
                    ]),
                ]
                .spacing(m.space_m);
                column![
                    g("Live preview").push(w::card(preview)),
                    g("Radii")
                        .push(
                            w::SliderRow::new("Control radius", self.radius_m, 0.0..=Metrics::MAX_RADIUS_M, Message::RadiusM)
                                .subtitle(format!("radius_m = {:.0}px", self.radius_m))
                                .metrics(m),
                        )
                        .push(
                            w::SliderRow::new("Card radius", self.radius_l, 0.0..=Metrics::MAX_RADIUS_L, Message::RadiusL)
                                .subtitle(format!("radius_l = {:.0}px", self.radius_l))
                                .metrics(m),
                        )
                        .push(
                            w::SliderRow::new("Toast radius", self.radius_toast, 0.0..=Metrics::MAX_RADIUS_TOAST, Message::RadiusToast)
                                .subtitle(format!("radius_toast = {:.0}px", self.radius_toast))
                                .metrics(m),
                        ),
                    g("Shadows")
                        .push(w::SwitchRow::new("Shadows", self.shadows_on, Message::Shadows).subtitle("Adwaita elevation: cards, popovers, dialogs").metrics(m))
                        .push(
                            w::SliderRow::new("Shadow intensity", self.shadow_intensity, 0.0..=2.0, Message::ShadowIntensity)
                                .subtitle(format!("{:.1}x", self.shadow_intensity))
                                .step(0.1)
                                .metrics(m),
                        )
                        .push(
                            w::SliderRow::new("Shadow size", self.shadow_scale, 0.0..=2.0, Message::ShadowScale)
                                .subtitle(format!("{:.1}x", self.shadow_scale))
                                .step(0.1)
                                .metrics(m),
                        ),
                ]
                .spacing(m.group_spacing)
                .into()
            }
            Detail::ThemeMotion => column![
                g("Animations")
                    .description("libadwaita's easings and springs; honors the system enable-animations setting.")
                    .push(w::SwitchRow::new("Animations", self.anim_enabled, Message::Animations).subtitle("Off: every transition jumps to its end value").metrics(m))
                    .push(
                        w::ComboRow::new("Motion", &MOTIONS, Some(self.demo), Message::DemoMotion)
                            .subtitle("Pick an easing or the sheet spring, then Play"),
                    )
                    .push(
                        w::ActionRow::new("Play")
                            .subtitle(format!("value {:.2}", self.demo_value))
                            .suffix(
                                row![
                                    w::progress_bar(0.0..=1.0, self.demo_value.clamp(0.0, 1.0)).length(Length::Fixed(120.0)),
                                    w::suggested_button(w::typography::label("Play")).on_press(Message::DemoPlay),
                                ]
                                .spacing(m.space_m)
                                .align_y(iced::Alignment::Center),
                            )
                            .metrics(m),
                    ),
            ]
            .into(),
            Detail::ThemeWindow => column![
                g("Menu")
                    .push(
                        w::ToggleGroupRow::new(
                            "Menu presentation",
                            vec![
                                w::ToggleItem::new(MenuMode::Auto, "Auto").icon(Icon::Settings),
                                w::ToggleItem::new(MenuMode::Dropdown, "Dropdown").icon(Icon::GoDown),
                                w::ToggleItem::new(MenuMode::FullPage, "Full page").icon(Icon::Window),
                            ],
                            Some(self.menu_mode),
                            Message::MenuMode,
                        )
                        .subtitle("Auto follows the desktop config (full page at the mobile breakpoint)")
                        .metrics(m),
                    ),
                g("Sidebar")
                    .push(
                        w::ComboRow::new("Mobile sidebar", &SIDEBAR_MODES, Some(self.sidebar_mode), Message::SidebarMode)
                            .subtitle("Drawer over the content, or the parent page of a stack"),
                    )
                    .push(
                        w::ToggleGroupRow::new(
                            "Row padding",
                            vec![
                                w::ToggleItem::new(w::SidebarPadding::Sm, "Small"),
                                w::ToggleItem::new(w::SidebarPadding::Md, "Medium"),
                                w::ToggleItem::new(w::SidebarPadding::Lg, "Large"),
                            ],
                            Some(self.sidebar_padding),
                            Message::SidebarPadding,
                        )
                        .subtitle("Small like Mantine, medium like Nautilus, large like GNOME Settings")
                        .metrics(m),
                    ),
                g("Header").push(
                    w::SwitchRow::new("Path bar title", self.path_bar_title, Message::PathBarTitle)
                        .subtitle("A title_widget replaces the title; window buttons stay")
                        .metrics(m),
                ),
            ]
            .spacing(m.group_spacing)
            .into(),
            Detail::Card => {
                let plain = w::Card::new(ty::body("A card with the default padding."));
                let chained = w::Card::new(w::ActionRow::new("First child").subtitle("padding(0): rows pad themselves").metrics(m))
                    .padding(0)
                    .push(w::ActionRow::new("Second child").metrics(m))
                    .push(w::ActionRow::new("Third child").metrics(m));
                let bare = w::Card::new(ty::body("separators(false)")).push(ty::body("Children share one frame and shadow."))
                    .separators(false);
                let sized = w::Card::new(ty::body("Fixed width and height"))
                    .width(Length::Fixed(240.0))
                    .height(Length::Fixed(80.0));
                column![
                    g("Card").push(plain),
                    g("Chained: one frame, one shadow").push(chained),
                    g("Without separators").push(bare),
                    g("Fixed size").push(sized),
                ]
                .spacing(m.group_spacing)
                .into()
            }
        }
        };
        // A page without controls still has the Preview / Code switch: its demo is the preview,
        // the snippet of the calls it makes is the code.
        let body: Element<'_> = match (plays::snippet(d), self.plays.get(&d)) {
            (Some(code), Some(p)) => {
                p.view(m, body, code.to_string(), move |x| Message::Play(d, x))
            }
            _ => body,
        };
        self.page_frame(body)
    }

    /// A component with one demo, shown directly on its group page.
    fn inline(&self, i: Inline) -> Element<'_> {
        let m = self.metrics();
        let noop = |_: String| Message::Noop;
        let sp = m.space_m;
        let g = |t| self.group(t);
        match i {
            Inline::Switch => {
                let bare = stack(sp, vec![
                    w::Field::new("Enabled", w::switch(self.sw).on_toggle(Message::Sw)).layout(w::FieldLayout::Row).plain_label(true).metrics(m).into(),
                    w::Field::new("Off", w::switch(false).on_toggle(|_| Message::Noop)).layout(w::FieldLayout::Row).plain_label(true).metrics(m).into(),
                    w::Field::new("Disabled, on", w::switch(true)).layout(w::FieldLayout::Row).plain_label(true).metrics(m).into(),
                    w::Field::new("Disabled, off", w::switch(false)).layout(w::FieldLayout::Row).plain_label(true).metrics(m).into(),
                ]);
                column![g("Switch").push(w::card(bare))].into()
            }
            Inline::CheckButton => {
                let checks = stack(sp, vec![
                    w::check_button("Enabled", self.check).on_toggle(Message::Check).into(),
                    w::check_button("Unchecked", false).on_toggle(|_| Message::Noop).into(),
                    w::check_button("Disabled, checked", true).into(),
                    w::check_button("Disabled, unchecked", false).into(),
                ]);
                column![g("Check button").push(w::card(checks))].into()
            }
            Inline::RadioButton => {
                let vertical = stack(sp, vec![
                    w::radio_button("Radio A", 0u8, self.radio, Message::Radio).into(),
                    w::radio_button("Radio B", 1u8, self.radio, Message::Radio).into(),
                    w::radio_button("Radio C", 2u8, self.radio, Message::Radio).into(),
                ]);
                column![g("Radio button").push(w::card(vertical))].into()
            }
            Inline::Scale => {
                let scale = stack(sp, vec![
                    w::scale(0.0..=100.0, self.slider, Message::Slider).into(),
                    ty::dim_label(format!("value {:.0}", self.slider)).into(),
                ]);
                column![g("Scale").push(w::card(scale))].into()
            }
            Inline::ButtonRow => column![
                g("Button rows")
                    .push(w::ButtonRow::new("Neutral").on_activate(Message::Noop))
                    .push(w::ButtonRow::new("Suggested").role(Role::Accent).on_activate(Message::Noop))
                    .push(w::ButtonRow::new("Destructive").role(Role::Destructive).on_activate(Message::OpenDialog(Dialog::Alert))),
            ]
            .into(),
            Inline::Clamp => column![
                g("Clamp at 280px").push(w::card(w::clamp(
                    w::entry("Never wider than 280px", "centered").on_input(noop).metrics(m),
                    280.0,
                ))),
            ]
            .into(),
            Inline::StatusPage => {
                let page = |icon: Icon, title: &'static str, desc: &'static str| {
                    w::Card::new(
                        w::status_page::<Message>()
                            .icon(icon.colored(64.0, self.build_theme().semantic.dim_solid()))
                            .title(title)
                            .description(desc)
                            .child(w::suggested_pill_button(text("Try Again")).on_press(Message::Noop))
                            .compact(true)
                            .metrics(m),
                    )
                    .padding(0)
                    .height(Length::Fixed(280.0))
                };
                column![g("Status page").push(page(Icon::Search, "No Results", "Try a different search term."))].into()
            }
            Inline::WrapBox => {
                let tags: Vec<Element> = ["rust", "iced", "gnome", "adwaita", "libadwaita", "theming", "widgets"]
                    .into_iter()
                    .map(|t| w::tag(t, Role::Accent))
                    .collect();
                column![g("Wrap box").description("Narrow the window: children wrap.").push(w::card(w::wrap_box(tags, m.space_s)))]
                    .into()
            }
            Inline::Collapse => column![
                g("Collapse behavior")
                    .description("Narrow the window: the control wraps. Stacked = 100% width; Flex = keeps its width.")
                    .push(
                        w::SliderRow::new("Stacked (default)", self.slider, 0.0..=100.0, Message::Slider)
                            .subtitle("collapse = Stacked")
                            .basis(260.0)
                            .collapse(adw::CollapseBehavior::Stacked)
                            .metrics(m),
                    )
                    .push(
                        w::SliderRow::new("Flex", self.slider, 0.0..=100.0, Message::Slider)
                            .subtitle("collapse = Flex")
                            .basis(260.0)
                            .collapse(adw::CollapseBehavior::Flex)
                            .metrics(m),
                    ),
            ]
            .into(),

            Inline::ViewSwitcher => {
                let items = || {
                    vec![
                        w::ToggleItem::new(Tab::One, "Files").icon(Icon::Home),
                        w::ToggleItem::new(Tab::Two, "Search").icon(Icon::Search),
                        w::ToggleItem::new(Tab::Three, "Settings").icon(Icon::Settings),
                    ]
                };
                column![
                    g("View switcher").push(w::card(Element::from(w::view_switcher(items(), Some(self.tab), Message::Tab)))),
                ]
                .into()
            }
            Inline::PathBar => column![
                g("Path bar").push(w::card(w::path_bar(
                    vec!["Home".into(), "Projects".into(), "iced-libadwaita".into()],
                    2,
                    |_| Message::Noop,
                ))),
                g("In the header").push(
                    w::SwitchRow::new("Path bar title", self.path_bar_title, Message::PathBarTitle)
                        .subtitle("A title_widget replaces the title; window buttons stay")
                        .metrics(m),
                ),
            ]
            .spacing(m.group_spacing)
            .into(),
            Inline::Carousel => {
                // Each page has its own tint so the slide is easy to follow.
                let pages = || -> Vec<Element<'_>> {
                    [("First page", Role::Accent), ("Second page", Role::Success), ("Third page", Role::Warning)]
                        .into_iter()
                        .map(|(t, role)| -> Element {
                            container(ty::title_2(t))
                                .center(Length::Fill)
                                .style(move |t: &adw::Theme| iced::widget::container::Style {
                                    border: iced::Border::default(),
                                    ..style::container::tinted(role)(t)
                                })
                                .into()
                        })
                        .collect()
                };
                // One card: the carousel and its control share a single frame and shadow.
                let car = w::Card::new(
                    container(w::carousel(pages(), self.carousel, Message::Carousel).indicator(self.carousel_indicator))
                        .height(Length::Fixed(160.0)),
                )
                .padding(0)
                .push(
                    w::ComboRow::new("Indicator", &INDICATORS, Some(self.carousel_indicator), Message::CarouselIndicator)
                        .subtitle("Drag the pages, or click the indicator"),
                );
                column![g("Carousel").push(car)].into()
            }
            Inline::MenuButton => {
                let presentation = match self.menu_mode {
                    MenuMode::Auto => w::MenuPresentation::resolve(900.0),
                    MenuMode::Dropdown => w::MenuPresentation::Dropdown,
                    MenuMode::FullPage => w::MenuPresentation::FullPage,
                };
                let menu = w::primary_menu(
                    w::Menu::new("Menu")
                        .section([w::MenuItem::check("Dark Style", self.scheme.is_dark(), Message::Scheme(!self.scheme.is_dark()))])
                        .section([
                            w::MenuItem::action("Preferences", Message::OpenDialog(Dialog::Preferences)).accel("Ctrl+,"),
                            w::MenuItem::action("Unavailable", Message::Noop).disabled(),
                            w::MenuItem::action("About", Message::OpenDialog(Dialog::About)),
                        ]),
                )
                .presentation(presentation);
                column![
                    g("Menu button").description("The menu includes a disabled item.").push(w::card(row![menu])),
                    g("Presentation").push(
                        w::ToggleGroupRow::new(
                            "Menu presentation",
                            vec![
                                w::ToggleItem::new(MenuMode::Auto, "Auto").icon(Icon::Settings),
                                w::ToggleItem::new(MenuMode::Dropdown, "Dropdown").icon(Icon::GoDown),
                                w::ToggleItem::new(MenuMode::FullPage, "Full page").icon(Icon::Window),
                            ],
                            Some(self.menu_mode),
                            Message::MenuMode,
                        )
                        .subtitle("Auto follows the desktop config")
                        .metrics(m),
                    ),
                ]
                .spacing(m.group_spacing)
                .into()
            }
            Inline::ShortcutLabel => {
                let shortcuts = column![
                    row![ty::body("Copy"), w::shortcut_label("Ctrl+C")].spacing(m.space_m),
                    row![ty::body("Palette"), w::shortcut_label("Ctrl+Shift+P")].spacing(m.space_m),
                ]
                .spacing(m.space_s);
                column![g("Shortcut labels").push(w::card(shortcuts))].into()
            }

            Inline::Banner => {
                let banner: Element = w::banner("Network connection lost")
                    .button("Retry", Message::Banner(false))
                    .revealed(self.banner)
                    .into();
                column![
                    g("Banner").push(banner),
                    g("State").push(w::SwitchRow::new("Revealed", self.banner, Message::Banner).metrics(m)),
                ]
                .spacing(m.group_spacing)
                .into()
            }
            Inline::Progress => {
                let bars = stack(sp, vec![
                    w::scale(0.0..=100.0, self.slider, Message::Slider).into(),
                    w::progress_bar(0.0..=100.0, self.slider).into(),
                    w::progress_bar(0.0..=100.0, self.slider).style(style::progress_bar::success).into(),
                    w::progress_bar(0.0..=100.0, self.slider).style(style::progress_bar::warning).into(),
                    w::progress_bar(0.0..=100.0, self.slider).style(style::progress_bar::error).into(),
                ]);
                column![
                    g("Progress bar").push(w::card(bars)),
                    g("Spinner").push(w::card(w::spinner(32.0, self.angle))),
                ]
                .spacing(m.group_spacing)
                .into()
            }
            Inline::Avatar => {
                let avatars = row![
                    w::avatar(32.0, "Ada Lovelace"),
                    w::avatar(48.0, "Grace Hopper"),
                    w::avatar(64.0, "Linus"),
                    w::avatar(48.0, "Margaret Hamilton").show_initials(false),
                ]
                .spacing(sp)
                .align_y(iced::Alignment::Center);
                column![g("Avatar").push(w::card(avatars))].into()
            }
            Inline::Surfaces => {
                let sem = self.build_theme().semantic;
                let surfaces = column![
                    row![w::color_swatch("window", sem.window), w::color_swatch("view", sem.view), w::color_swatch("headerbar", sem.headerbar)].spacing(m.space_s),
                    row![w::color_swatch("card", sem.card), w::color_swatch("dialog", sem.dialog), w::color_swatch("popover", sem.popover)].spacing(m.space_s),
                    row![w::color_swatch("sidebar", sem.sidebar), w::color_swatch("thumbnail", sem.thumbnail), w::color_swatch("osd", sem.osd)].spacing(m.space_s),
                    row![w::status_swatch("accent", sem.accent), w::status_swatch("destructive", sem.destructive), w::status_swatch("success", sem.success)].spacing(m.space_s),
                    row![w::status_swatch("warning", sem.warning), w::status_swatch("error", sem.error)].spacing(m.space_s),
                ]
                .spacing(m.space_s);
                column![g("Semantic surfaces").push(w::card(surfaces))].into()
            }
            Inline::Tag => {
                let tags = row![
                    w::tag("Accent", Role::Accent),
                    w::tag("Success", Role::Success),
                    w::tag("Warning", Role::Warning),
                    w::tag("Error", Role::Error),
                    w::tag("Destructive", Role::Destructive),
                ]
                .spacing(m.space_s)
                .wrap();
                column![g("Tag").push(w::card(tags))].into()
            }

            Inline::BadgeChip => {
                let badges = row![
                    w::Badge::new("New").role(Role::Accent),
                    w::Badge::new("Stable").role(Role::Success),
                    w::Badge::new("Beta").role(Role::Warning),
                    w::Badge::new("Broken").role(Role::Error),
                    w::Badge::count(7, 99).role(Role::Error).solid(true),
                    w::Badge::count(120, 99).role(Role::Accent).solid(true),
                    w::Badge::new("Neutral"),
                ]
                .spacing(m.space_s)
                .wrap();
                let chips = row(self.chips.iter().map(|c| {
                    Element::from(
                        w::Chip::new(*c).role(Role::Accent).on_press(Message::Noop).on_remove(Message::ChipRemove(c)),
                    )
                }))
                .spacing(m.space_s)
                .wrap();
                let states = row![
                    w::Chip::new("Plain").on_press(Message::Noop),
                    w::Chip::new("Icon").icon(Icon::Check).role(Role::Success).on_press(Message::Noop),
                    w::Chip::new("Selected").role(Role::Accent).selected(true).on_press(Message::Noop),
                    w::Chip::new("Disabled").on_press(Message::Noop).disabled(true),
                    w::Chip::new("Disabled ✕").role(Role::Accent).on_remove(Message::Noop).disabled(true),
                ]
                .spacing(m.space_s)
                .wrap();
                column![
                    g("Badge").push(w::card(badges)),
                    g("Chip").description("Press the ✕ to remove one.").push(w::card(chips)).push(w::card(states)),
                ]
                .spacing(m.group_spacing)
                .into()
            }
            Inline::Indicator => {
                let dot = |c: fn(&adw::Semantic) -> iced::Color| -> Element<'static> {
                    container(iced::widget::space().width(12).height(12))
                        .style(move |t: &adw::Theme| iced::widget::container::Style {
                            background: Some(iced::Background::Color(c(&t.semantic))),
                            border: iced::Border::default().rounded(6).width(2).color(t.semantic.window.bg),
                            ..Default::default()
                        })
                        .into()
                };
                let ok = |s: &adw::Semantic| s.success.color;
                let bad = |s: &adw::Semantic| s.error.color;
                let av = |n: &str| Element::from(w::avatar(48.0, n.to_string()));
                use adw::primitives::anchor::{Align, Side};
                let ind = |child: Element<'static>, badge: Element<'static>| w::Indicator::new(child, badge);
                let corners = row![
                    ind(av("Ada Lovelace"), w::Badge::count(3, 99).role(Role::Error).solid(true).into()),
                    ind(av("Grace Hopper"), dot(ok)).corner(Side::Bottom, Align::End).offset(4.0),
                    ind(av("Linus T"), dot(bad)).corner(Side::Top, Align::Start).offset(4.0),
                    ind(av("Margaret H"), w::Badge::new("Pro").role(Role::Accent).solid(true).into()).corner(Side::Bottom, Align::Center),
                    ind(av("Alan Turing"), w::Badge::count(120, 99).role(Role::Accent).solid(true).into()),
                    ind(av("Hidden"), dot(bad)).show(false),
                ]
                .spacing(m.space_l)
                .wrap();
                let on_buttons = row![
                    ind(w::raised_button(ty::label("Inbox")).on_press(Message::Noop).into(), w::Badge::count(12, 99).role(Role::Error).solid(true).into()),
                    ind(w::icon_button(Icon::Search).on_press(Message::Noop).into(), dot(bad)),
                    ind(w::flat_button(ty::label("Updates")).on_press(Message::Noop).into(), w::Badge::new("new").role(Role::Success).solid(true).into()).offset(2.0),
                ]
                .spacing(m.space_l)
                .align_y(iced::Alignment::Center)
                .wrap();
                column![
                    g("Indicator").description("A badge centered on a corner, whatever its shape (round dot, count, pill). `offset` moves it in; `show(false)` removes it.").push(w::card(container(corners).padding(sp))),
                    g("On buttons").push(w::card(container(on_buttons).padding(sp))),
                ]
                .spacing(m.group_spacing)
                .into()
            }
            Inline::Docked => {
                use adw::primitives::anchor::{Dim, Side};
                let stage = |lines: &'static str| -> Element<'static> {
                    container(column![ty::heading("Content"), ty::body(lines).style(adw::style::text::dim)].spacing(4))
                        .padding(12)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(|t: &adw::Theme| iced::widget::container::Style {
                            background: Some(iced::Background::Color(t.semantic.card.bg)),
                            border: iced::Border::default().rounded(8),
                            ..Default::default()
                        })
                        .into()
                };
                let bar = |label: &'static str| -> Element<'static> {
                    container(row![ty::label(label), iced::widget::space().width(Length::Fill), w::suggested_button(ty::label("Save")).on_press(Message::Noop)].align_y(iced::Alignment::Center))
                        .padding(8)
                        .width(Length::Fill)
                        .style(|t: &adw::Theme| iced::widget::container::Style {
                            background: Some(iced::Background::Color(t.semantic.window.bg)),
                            border: iced::Border::default().width(1).color(t.semantic.card.fg.scale_alpha(0.12)),
                            ..Default::default()
                        })
                        .into()
                };
                let pane = || -> Element<'static> {
                    container(column![ty::heading("Drawer"), ty::body("Docked left, full height").style(adw::style::text::dim)].spacing(4))
                        .padding(12)
                        .style(|t: &adw::Theme| iced::widget::container::Style {
                            background: Some(iced::Background::Color(t.semantic.window.bg)),
                            border: iced::Border::default().width(1).color(t.semantic.card.fg.scale_alpha(0.12)),
                            ..Default::default()
                        })
                        .into()
                };
                let frame = |e: Element<'static>| -> Element<'static> {
                    container(e).width(Length::Fixed(300.0)).height(Length::Fixed(190.0)).into()
                };
                let demos = row![
                    frame(w::dock_footer(stage("Footer: inside the bottom edge, as wide as the content."), bar("Unsaved changes"))),
                    frame(w::dock_banner(stage("Banner: inside the top edge, over the content."), bar("Offline: changes are kept"))),
                    frame(w::dock_drawer(stage("Drawer: left edge, 120 wide, full height."), pane(), Side::Left, Dim::Fixed(120.0))),
                    frame(w::dock_drawer(stage("Drawer: right edge, 60% of the width."), pane(), Side::Right, Dim::Fraction(0.6))),
                    frame(w::dock_center(stage("Centered over the content."), bar("Centered"))),
                ]
                .spacing(m.space_m)
                .wrap();
                column![g("Docked").description("One placement primitive: an edge, an alignment along it, an inset and a size rule (`anchor::place_sized`).").push(demos)]
                    .into()
            }
            Inline::MultiSelect => {
                let open = std::env::var("GALLERY_OPEN").as_deref() == Ok("multiselect");
                let controls = stack(sp, vec![
                    w::Field::new("Fruits", Element::from(w::MultiSelect::new(&FRUITS5, &self.multi, Message::MultiToggle).width(Length::Fixed(260.0)).open_initially(open))).layout(w::FieldLayout::Row).plain_label(true).metrics(m).into(),
                    w::Field::new("Empty", Element::from(w::MultiSelect::new(&FRUITS5, &[], Message::MultiToggle).placeholder("Pick fruits").width(Length::Fixed(260.0)))).layout(w::FieldLayout::Row).plain_label(true).metrics(m).into(),
                    w::Field::new("Crowded (+n)", Element::from(w::MultiSelect::new(&FRUITS5, &FRUITS5, Message::MultiToggle).max_chips(2).width(Length::Fixed(260.0)))).layout(w::FieldLayout::Row).plain_label(true).metrics(m).into(),
                    w::Field::new("Disabled", Element::from(w::MultiSelect::new(&FRUITS5, &self.multi, Message::MultiToggle).disabled(true).width(Length::Fixed(260.0)))).layout(w::FieldLayout::Row).plain_label(true).metrics(m).into(),
                ]);
                column![g("Multi select").description("The popover stays open while options are toggled.").push(w::card(controls))].into()
            }
            Inline::DateInput => self.date.view(m).map(Message::Date),
            Inline::RangeSlider => self.range.slider_view(m).map(Message::Range),
            Inline::DateRange => self.range.date_view(m).map(Message::Range),
            Inline::AnchoredPopover => {
                let open = std::env::var("GALLERY_OPEN").as_deref() == Ok("anchor");
                let menu = |t: &'static str| {
                    w::popover([w::menu_item(t, false, Message::Noop), w::menu_item("Second", true, Message::Noop)], 160.0)
                };
                let pop = |label: &'static str, side: w::Side, o: bool| {
                    Element::from(
                        w::Popover::new(text(label), menu(label))
                            .side(side)
                            .offset(8.0)
                            .popover_width(140.0)
                            .open_initially(o),
                    )
                };
                let sides = row![
                    pop("Bottom", w::Side::Bottom, false),
                    pop("Top", w::Side::Top, false),
                    pop("Right", w::Side::Right, open),
                    pop("Left", w::Side::Left, false),
                ]
                .spacing(m.space_m);
                // Opening to the left from the edge of the content: kept to the content area it flips
                // right; opted out to the window it opens over the sidebar.
                let bounded = |label: &'static str, b: w::PopoverBounds| {
                    Element::from(
                        w::Popover::new(text(label), menu(label))
                            .side(w::Side::Left)
                            .offset(8.0)
                            .popover_width(140.0)
                            .bounds(b),
                    )
                };
                let bounds = row![
                    bounded("Left, content area", w::PopoverBounds::Content),
                    bounded("Left, window", w::PopoverBounds::Window),
                ]
                .spacing(m.space_m);
                column![
                    g("Anchored popover").description("Side and offset against the trigger; it flips to the other side and shifts to stay inside the content area it opened from.").push(w::card(sides)),
                    g("Bounds").description("A popover is kept inside the content area (beside the sidebar), not the whole window. `bounds(PopoverBounds::Window)` opts out; the default is the desktop config's `[popovers] bounds`.").push(w::card(bounds)),
                ]
                .spacing(m.group_spacing)
                .into()
            }
            Inline::ResponsiveTable => {
                let table = |mode: w::TableMode| {
                    w::ResponsiveTable::<Person, Message>::new(&PEOPLE)
                        .column(w::Column::new("Name", |p: &Person| ty::heading(p.name).into()).min_width(110.0))
                        .column(w::Column::new("Role", |p: &Person| ty::body(p.role).into()).min_width(90.0))
                        .column(w::Column::new("Team", |p: &Person| ty::body(p.team).into()).min_width(100.0))
                        .column(w::Column::new("Email", |p: &Person| ty::body(p.email).into()).width(Length::FillPortion(2)).min_width(150.0))
                        .card(|p: &Person| {
                            column![
                                row![ty::heading(p.name), w::Badge::new(p.role).role(Role::Accent)].spacing(8).align_y(iced::Alignment::Center),
                                ty::caption(p.email),
                                ty::body(p.team),
                            ]
                            .spacing(4)
                            .into()
                        })
                        .mode(mode)
                        .height(Length::Fixed(300.0))
                };
                let wide = container(Element::from(table(w::TableMode::Auto))).width(Length::Fixed(600.0)).height(Length::Fixed(300.0));
                let narrow = container(Element::from(table(w::TableMode::Auto))).width(Length::Fixed(340.0)).height(Length::Fixed(300.0));
                column![
                    g("Wide container").description("The columns fit: a table.").push(wide),
                    g("Narrow container").description("They do not: every row is a card with a custom layout.").push(narrow),
                ]
                .spacing(m.group_spacing)
                .into()
            }

            Inline::Accordion => {
                let body = |t: &'static str| Element::from(container(ty::body(t)).padding(m.space_m));
                let items = |acc: w::Accordion<'static, u8, Message>| {
                    acc.item(w::AccordionItem::new(0u8, "Shipping", body("Two to four working days.")).subtitle("2-4 days"))
                        .item(w::AccordionItem::new(1u8, "Returns", body("Thirty days, unused.")).subtitle("30 days"))
                        .item(w::AccordionItem::new(2u8, "Warranty", body("Two years.")))
                };
                column![
                    g("Accordion, single").description("One item open at a time; opening another closes it, and the open one can be closed.")
                        .push(Element::from(items(w::Accordion::new(&self.acc_single, Message::AccSingle).selection(w::Selection::SINGLE)))),
                    g("Accordion, multiple").description("Any number open.")
                        .push(Element::from(items(w::Accordion::new(&self.acc_multi, Message::AccMulti).selection(w::Selection::Multiple)))),
                ]
                .spacing(m.group_spacing)
                .into()
            }

            Inline::LinkButton => column![g("Link button").push(w::card(
                row![w::link_button("Link").on_press(Message::Noop), w::link_button("Disabled link")].spacing(sp).wrap(),
            ))]
            .into(),
            Inline::CardButton => column![g("Card button").push(w::card(
                row![
                    w::card_button(text("Card button").width(Length::Fixed(110.0))).on_press(Message::Noop),
                    w::card_button(text("Disabled").width(Length::Fixed(110.0))),
                ]
                .spacing(sp)
                .wrap(),
            ))]
            .into(),
        }
    }

    fn typography_page(&self) -> Element<'_> {
        let m = self.metrics();
        let typo = column![
            ty::title_1("Title 1"),
            ty::title_2("Title 2"),
            ty::title_3("Title 3"),
            ty::title_4("Title 4"),
            ty::heading("Heading"),
            ty::body("Body text, the default."),
            ty::caption_heading("Caption heading"),
            ty::caption("Caption"),
            ty::monospace("monospace 0123456789"),
            ty::dim_label("Dim label"),
        ]
        .spacing(m.space_xs);
        let colors = row![
            ty::body("Accent").style(style::text::accent),
            ty::body("Success").style(style::text::success),
            ty::body("Warning").style(style::text::warning),
            ty::body("Error").style(style::text::error),
            ty::body("Destructive").style(style::text::destructive),
        ]
        .spacing(m.space_l)
        .wrap();
        self.page_frame(
            column![
                self.group("Text styles").push(w::card(typo)),
                self.group("Text colors").push(w::card(colors)),
                playground::code_block(
                    m,
                    "title_1(\"Title 1\")   // title_2, title_3, title_4, heading\nbody(\"Body text\")\ncaption(\"Caption\")   // caption_heading, monospace, dim_label\n\nbody(\"Accent\").style(style::text::accent) // success, warning, error, destructive".to_string(),
                ),
            ]
            .spacing(m.group_spacing),
        )
    }
}
