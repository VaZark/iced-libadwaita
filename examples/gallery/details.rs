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

/// A label on the start edge and a control on the end edge.
fn labeled<'a>(label: &'static str, control: impl Into<Element<'a>>) -> Element<'a> {
    row![container(ty::body(label)).width(Length::Fill), control.into()]
        .align_y(iced::Alignment::Center)
        .into()
}

impl Gallery {
    /// Identifies the page being shown, so switching it fades the new one in.
    pub(super) fn view_key(&self) -> u64 {
        (self.page as u64) * 1000 + self.detail.map_or(0, |d| d as u64 + 1)
    }

    pub(super) fn content_page(&self) -> Element<'_> {
        match (self.page, self.detail) {
            (Page::Typography, _) => self.typography_page(),
            (Page::Theming, _) => self.theming_page(),
            (page, None) => self.group_page(page),
            (_, Some(d)) => self.detail_page(d),
        }
    }

    fn group<'a>(&self, title: &'static str) -> w::PreferencesGroup<'a, Message> {
        w::PreferencesGroup::<Message>::new().title(title).metrics(self.metrics())
    }

    /// Level 1: the group's components. Components with two or more significantly different
    /// properties open a nested page; single-trigger ones get a button on their row; the rest
    /// show their one demo right here (see `README.md`).
    fn group_page(&self, page: Page) -> Element<'_> {
        let m = self.metrics();
        let mut col = column![].spacing(m.group_spacing);
        let mut first = true;
        for (title, entries) in page.sections() {
            // Consecutive rows (nested / action) share one list; inline demos are their own groups.
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
                            if std::mem::take(&mut first) { g.description(page.description()) } else { g }
                        });
                        g = g.push(row);
                        list = Some(g);
                    }
                    Entry::Action(a) => {
                        let row = w::ActionRow::new(a.title())
                            .subtitle(a.description())
                            .suffix(w::raised_button(text(a.button())).on_press(self.action_message(a)))
                            .metrics(m);
                        let mut g = list.take().unwrap_or_else(|| {
                            let g = self.group(title);
                            if std::mem::take(&mut first) { g.description(page.description()) } else { g }
                        });
                        g = g.push(row);
                        list = Some(g);
                    }
                    Entry::Inline(inl) => {
                        col = flush(col, &mut list);
                        first = false;
                        col = col.push(self.inline(inl));
                    }
                }
            }
            col = flush(col, &mut list);
        }
        self.page_frame(col)
    }

    fn action_message(&self, a: Action) -> Message {
        match a {
            Action::AlertDialog => Message::OpenDialog(Dialog::Alert),
            Action::AboutDialog => Message::OpenDialog(Dialog::About),
            Action::ShortcutsDialog => Message::OpenDialog(Dialog::Shortcuts),
            Action::PreferencesDialog => Message::OpenDialog(Dialog::Preferences),
            Action::BottomSheet => Message::Sheet(true),
            Action::ShowToast => Message::ShowToast("Settings saved"),
            Action::ShowUndoToast => Message::ShowUndoToast,
        }
    }

    /// Level 2: one component, grouped by property.
    fn detail_page(&self, d: Detail) -> Element<'_> {
        let m = self.metrics();
        let noop = |_: String| Message::Noop;
        let lbl = |s: &'static str| -> Element<'static> { w::typography::label(s).into() };
        let sp = m.space_m;
        let g = |t| self.group(t);

        let body: Element<'_> = match d {
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
                let split = row![w::split_button(text("Split"), Message::Noop, Message::ShowToast("Split menu"))]
                    .spacing(sp);
                column![g("Icon and label").push(w::card(content)), g("Split button").push(w::card(split))]
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
                column![
                    g("States").push(w::card(states)),
                    g("Password").push(w::card(password)),
                    g("Color override").push(w::card(color)),
                    g("Width").push(w::card(width)),
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
            Detail::Dropdown => {
                let plain = stack(sp, vec![
                    labeled("Choose a fruit", Element::from(w::dropdown(&FRUITS, self.picked, Message::Picked))),
                    labeled("Selected", Element::from(w::dropdown(&FRUITS, Some("Banana"), Message::Picked))),
                    labeled("Disabled", Element::from(w::dropdown_disabled::<&str, Message>(Some(&"Apple")))),
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
                    labeled("Enabled", w::switch(self.sw).on_toggle(Message::Sw)),
                    labeled("Off", w::switch(false).on_toggle(|_| Message::Noop)),
                    labeled("Disabled, on", w::switch(true)),
                    labeled("Disabled, off", w::switch(false)),
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
            Inline::ExpanderRow => column![
                g("Expander row").push(
                    w::ExpanderRow::new("Notifications", self.expanded, Message::Expanded)
                        .subtitle("Expand for details")
                        .push(w::SwitchRow::new("Sounds", self.notifications, Message::Notifications).metrics(m))
                        .push(w::ActionRow::new("Do Not Disturb").subtitle("Until tomorrow").metrics(m)),
                ),
            ]
            .into(),
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
                            .icon(icon.colored(64.0, self.theme().semantic.dim_solid()))
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
            Inline::TabBar => {
                let tabs = w::tab_bar(
                    vec!["Home".into(), "Docs".into(), "Settings".into()],
                    self.tab as usize,
                    |i| Message::Tab([Tab::One, Tab::Two, Tab::Three][i]),
                    Some(|_| Message::Noop),
                );
                column![g("Tab bar").push(w::Card::new(tabs).padding(0))].into()
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
            column![self.group("Text styles").push(w::card(typo)), self.group("Text colors").push(w::card(colors))]
                .spacing(m.group_spacing),
        )
    }
}
