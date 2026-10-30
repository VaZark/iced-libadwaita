//! Widget gallery: exercises the theme, style classes and Adw widgets.
//!
//! ```text
//! cargo run --example gallery
//! cargo run --example gallery -- --screenshot target/shots   # renders every page, light + dark
//! ```

use iced::widget::{
    checkbox, column, container, radio, row, slider, text,
    text_editor, text_input, toggler,
};
use iced::{window, Element as IcedElement, Length, Subscription, Task};
use iced_libadwaita as adw;
use adw::style;
use adw::widget::{self as w, typography as ty};
use adw::{AccentColor, Icon, Metrics, Role, Scheme, Theme};
use std::path::PathBuf;
use std::time::Duration;

type Element<'a> = adw::Element<'a, Message>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Page {
    Buttons,
    Lists,
    Inputs,
    Feedback,
    Dialogs,
    Navigation,
    Theming,
}

impl Page {
    const ALL: [Page; 7] = [
        Page::Buttons,
        Page::Lists,
        Page::Inputs,
        Page::Feedback,
        Page::Dialogs,
        Page::Navigation,
        Page::Theming,
    ];
    fn title(self) -> &'static str {
        match self {
            Page::Buttons => "Buttons & Text",
            Page::Lists => "Rows & Preferences",
            Page::Inputs => "Inputs",
            Page::Feedback => "Feedback",
            Page::Dialogs => "Dialogs",
            Page::Navigation => "Navigation",
            Page::Theming => "Theming",
        }
    }
    fn icon(self) -> Icon {
        match self {
            Page::Buttons => Icon::Check,
            Page::Lists => Icon::OpenMenu,
            Page::Inputs => Icon::Edit,
            Page::Feedback => Icon::DialogInformation,
            Page::Dialogs => Icon::Window,
            Page::Navigation => Icon::GoNext,
            Page::Theming => Icon::Settings,
        }
    }
}

/// How the primary menu is presented: follow the window width, or force a mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuMode {
    Auto,
    Dropdown,
    FullPage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dialog {
    Alert,
    About,
    Shortcuts,
    Preferences,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    One,
    Two,
    Three,
}

struct Gallery {
    page: Page,
    scheme: Scheme,
    accent: AccentColor,
    compact: bool,
    radius_m: f32,
    radius_l: f32,
    card_override: bool,
    shadows_on: bool,
    shadow_scale: f32,
    shadow_intensity: f32,
    raised_bar: bool,

    wifi: bool,
    notifications: bool,
    expanded: bool,
    name: String,
    password: String,
    combo: Option<&'static str>,
    spin: f64,
    tab: Tab,
    check: bool,
    radio: Option<u8>,
    slider: f32,
    editor: text_editor::Content,
    picked: Option<&'static str>,

    dialog: Option<Dialog>,
    show_content: bool,
    menu_mode: MenuMode,
    win: adw::window::State,
    sheet: bool,
    carousel: usize,
    toast: Option<String>,
    banner: bool,
    angle: f32,

    shots: Option<Shots>,
    shot_tag: Option<&'static str>,
}

struct Shots {
    dir: PathBuf,
    index: usize,
}

#[derive(Debug, Clone)]
enum Message {
    Page(Page),
    Scheme(bool),
    Accent(AccentColor),
    Compact(bool),
    RadiusM(f32),
    RadiusL(f32),
    CardOverride(bool),
    Shadows(bool),
    ShadowScale(f32),
    ShadowIntensity(f32),
    RaisedBar(bool),
    Wifi(bool),
    Notifications(bool),
    Expanded(bool),
    Name(String),
    Password(String),
    Combo(&'static str),
    Spin(f64),
    Tab(Tab),
    Check(bool),
    Radio(u8),
    Slider(f32),
    Editor(text_editor::Action),
    Picked(&'static str),
    OpenDialog(Dialog),
    CloseDialog,
    Sheet(bool),
    Carousel(usize),
    ShowToast(&'static str),
    DismissToast,
    DismissBanner,
    Frame,
    Noop,
    Drag,
    Close,
    Minimize,
    ToggleMaximize,
    Resize(window::Direction),
    Back,
    MenuMode(MenuMode),
    ToggleFullscreen,
    WindowChanged,
    WindowState(adw::window::State),
    // screenshot mode
    Shot(usize),
    Capture(usize),
    Saved(usize, window::Screenshot),
}

const FRUITS: [&str; 3] = ["Apple", "Banana", "Cherry"];
const COMBO: [&str; 3] = ["Adwaita Sans", "Cantarell", "Inter"];

impl Gallery {
    fn new() -> (Self, Task<Message>) {
        let args: Vec<String> = std::env::args().collect();
        let shots = args
            .iter()
            .position(|a| a == "--screenshot")
            .and_then(|i| args.get(i + 1))
            .map(|d| Shots { dir: d.into(), index: 0 });
        let boot = if shots.is_some() {
            Task::done(Message::Shot(0))
        } else {
            Task::none()
        };
        (
            Self {
                page: std::env::var("GALLERY_PAGE")
                    .ok()
                    .and_then(|v| Page::ALL.into_iter().find(|p| format!("{p:?}").eq_ignore_ascii_case(&v)))
                    .unwrap_or(Page::Buttons),
                scheme: Scheme::Light,
                accent: AccentColor::Blue,
                compact: false,
                radius_m: std::env::var("GALLERY_RM").ok().and_then(|v| v.parse().ok()).unwrap_or(Metrics::default().radius_m),
                radius_l: std::env::var("GALLERY_RL").ok().and_then(|v| v.parse().ok()).unwrap_or(12.0),
                card_override: false,
                shadows_on: true,
                shadow_scale: 1.0,
                shadow_intensity: 1.0,
                raised_bar: std::env::var_os("GALLERY_RAISED").is_some(),
                wifi: true,
                notifications: false,
                expanded: true,
                name: "Ada Lovelace".into(),
                password: "hunter2".into(),
                combo: Some("Adwaita Sans"),
                spin: 3.0,
                tab: Tab::One,
                check: true,
                radio: Some(1),
                slider: 40.0,
                editor: text_editor::Content::with_text("Multi-line text editor\nstyled by the theme."),
                picked: None,
                dialog: None,
                show_content: std::env::var_os("GALLERY_SIDEBAR").is_none(),
                menu_mode: match std::env::var("GALLERY_MENU").as_deref() {
                    Ok("page") => MenuMode::FullPage,
                    Ok("dropdown") => MenuMode::Dropdown,
                    _ => MenuMode::Auto,
                },
                win: adw::window::State {
                    maximized: std::env::var_os("GALLERY_FULL").is_some(),
                    fullscreen: false,
                },
                sheet: false,
                carousel: 0,
                toast: None,
                banner: true,
                angle: 0.0,
                shots,
                shot_tag: None,
            },
            boot,
        )
    }

    fn metrics(&self) -> Metrics {
        let base = if self.compact { Metrics::compact() } else { Metrics::default() };
        Metrics { radius_m: self.radius_m, radius_l: self.radius_l, ..base }
    }

    fn theme(&self) -> Theme {
        let shadows = if self.shadows_on {
            adw::Shadows::default().with_scale(self.shadow_scale).with_intensity(self.shadow_intensity)
        } else {
            adw::Shadows::none()
        };
        let mut t = Theme::new(self.scheme)
            .accent(self.accent)
            .with_metrics(self.metrics())
            .with_shadows(shadows)
            .with_transparent_window(Self::csd());
        if self.card_override {
            t = t.with_semantic(|s| {
                s.card.bg = adw::color::hex(0x3d2c5a);
                s.card.fg = iced::Color::WHITE;
            });
        }
        t
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Page(p) => {
                self.page = p;
                self.show_content = true;
            }
            Message::Back => self.show_content = false,
            Message::MenuMode(m) => self.menu_mode = m,
            Message::Scheme(dark) => self.scheme = if dark { Scheme::Dark } else { Scheme::Light },
            Message::Accent(a) => self.accent = a,
            Message::Compact(c) => self.compact = c,
            Message::RadiusM(r) => self.radius_m = r,
            Message::RadiusL(r) => self.radius_l = r,
            Message::CardOverride(c) => self.card_override = c,
            Message::Shadows(v) => self.shadows_on = v,
            Message::ShadowScale(v) => self.shadow_scale = v,
            Message::ShadowIntensity(v) => self.shadow_intensity = v,
            Message::RaisedBar(v) => self.raised_bar = v,
            Message::Wifi(v) => self.wifi = v,
            Message::Notifications(v) => self.notifications = v,
            Message::Expanded(v) => self.expanded = v,
            Message::Name(s) => self.name = s,
            Message::Password(s) => self.password = s,
            Message::Combo(s) => self.combo = Some(s),
            Message::Spin(v) => self.spin = v,
            Message::Tab(t) => self.tab = t,
            Message::Check(c) => self.check = c,
            Message::Radio(r) => self.radio = Some(r),
            Message::Slider(v) => self.slider = v,
            Message::Editor(a) => self.editor.perform(a),
            Message::Picked(s) => self.picked = Some(s),
            Message::OpenDialog(d) => self.dialog = Some(d),
            Message::CloseDialog => self.dialog = None,
            Message::Sheet(o) => self.sheet = o,
            Message::Carousel(i) => self.carousel = i,
            Message::ShowToast(s) => self.toast = Some(s.into()),
            Message::DismissToast => self.toast = None,
            Message::DismissBanner => self.banner = false,
            Message::Frame => self.angle += 0.12,
            Message::Noop => {}
            Message::Drag => return window::oldest().and_then(window::drag),
            Message::Close => return iced::exit(),
            Message::ToggleFullscreen => {
                return window::oldest().and_then(|id| {
                    window::mode(id).then(move |m| {
                        window::set_mode(
                            id,
                            if m == window::Mode::Fullscreen { window::Mode::Windowed } else { window::Mode::Fullscreen },
                        )
                    })
                });
            }
            Message::WindowChanged => return adw::window::refresh().map(Message::WindowState),
            // GALLERY_FULL forces the edge-to-edge look for screenshots.
            Message::WindowState(st) => {
                if std::env::var_os("GALLERY_FULL").is_none() {
                    self.win = st;
                }
            }
            Message::Minimize => return window::oldest().and_then(|id| window::minimize(id, true)),
            Message::ToggleMaximize => return window::oldest().and_then(window::toggle_maximize),
            Message::Resize(d) => return window::oldest().and_then(move |id| window::drag_resize(id, d)),
            Message::Shot(i) => return self.shot_next(i),
            Message::Capture(i) => {
                return window::oldest()
                    .and_then(window::screenshot)
                    .map(move |s| Message::Saved(i, s));
            }
            Message::Saved(i, shot) => {
                if let Some(s) = &self.shots {
                    let name = format!(
                        "{}-{}{}.png",
                        if self.scheme.is_dark() { "dark" } else { "light" },
                        format!("{:?}", self.page).to_lowercase(),
                        self.shot_tag.map(|t| format!("-{t}")).unwrap_or_default()
                    );
                    std::fs::create_dir_all(&s.dir).ok();
                    let path = s.dir.join(name);
                    image::save_buffer(
                        &path,
                        &shot.rgba,
                        shot.size.width,
                        shot.size.height,
                        image::ColorType::Rgba8,
                    )
                    .expect("write screenshot");
                    println!("saved {}", path.display());
                }
                return Task::done(Message::Shot(i + 1));
            }
        }
        Task::none()
    }

    /// Screenshot mode: step through (scheme x page), with dialogs/toasts on
    /// the Dialogs page so overlays are covered too.
    fn shot_next(&mut self, i: usize) -> Task<Message> {
        let total = 2 * Page::ALL.len();
        if i > total {
            return iced::exit();
        }
        if i == total {
            // Drive the real update path with the radius slider messages.
            self.scheme = Scheme::Light;
            self.page = Page::Theming;
            self.dialog = None;
            self.toast = None;
            self.shot_tag = Some("radius0");
            return Task::batch([Task::done(Message::RadiusM(0.0)), Task::done(Message::RadiusL(0.0))])
                .chain(Task::perform(tokio::time::sleep(Duration::from_millis(500)), move |_| Message::Capture(i)));
        }
        self.scheme = if i < Page::ALL.len() { Scheme::Light } else { Scheme::Dark };
        self.page = Page::ALL[i % Page::ALL.len()];
        self.dialog = (self.page == Page::Dialogs).then(|| match std::env::var("GALLERY_DIALOG").as_deref() {
            Ok("preferences") => Dialog::Preferences,
            Ok("shortcuts") => Dialog::Shortcuts,
            Ok("about") => Dialog::About,
            _ => Dialog::Alert,
        });
        self.toast = (self.page == Page::Dialogs).then(|| "Settings saved".to_string());
        if let Some(s) = &mut self.shots {
            s.index = i;
        }
        Task::perform(tokio::time::sleep(Duration::from_millis(500)), move |_| Message::Capture(i))
    }

    fn subscription(&self) -> Subscription<Message> {
        let frames = if self.page == Page::Feedback || self.shots.is_some() {
            window::frames().map(|_| Message::Frame)
        } else {
            Subscription::none()
        };
        Subscription::batch([
            frames,
            adw::window::changes().map(|_| Message::WindowChanged),
            iced::event::listen_with(|event, _, _| match event {
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::F11),
                    ..
                }) => Some(Message::ToggleFullscreen),
                _ => None,
            }),
        ])
    }

    fn view(&self) -> IcedElement<'_, Message, Theme> {
        iced::widget::responsive(|size| self.root(size)).into()
    }

    /// Whether the window draws its own decorations (the Adwaita default).
    fn csd() -> bool {
        std::env::var_os("GALLERY_SSD").is_none()
    }

    fn root(&self, size: iced::Size) -> IcedElement<'_, Message, Theme> {
        let m = self.metrics();
        let csd = Self::csd();
        let collapsed = size.width <= w::COLLAPSE_WIDTH;

        // Floating windows have rounded corners; maximized/fullscreen go edge to edge.
        // Mobile mode (narrow): no window decorations at all. Corners still follow the
        // real window state: rounded while floating, edge to edge only when the window
        // is actually maximized or fullscreen.
        let mobile = adw::window::is_mobile(size.width);
        let win_state = self.win;
        let corner = if csd { win_state.radius(&adw::Metrics::default()) } else { 0.0 };

        // Primary menu (HIG): open-menu icon, above the sidebar list, standard items last.
        let presentation = match self.menu_mode {
            MenuMode::Auto => w::MenuPresentation::for_width(size.width),
            MenuMode::Dropdown => w::MenuPresentation::Dropdown,
            MenuMode::FullPage => w::MenuPresentation::FullPage,
        };
        let dark = self.scheme.is_dark();
        let primary = w::primary_menu(
            w::Menu::new("Main Menu")
                .section([w::MenuItem::check("Dark Style", dark, Message::Scheme(!dark))])
                .section([
                    w::MenuItem::action("Preferences", Message::OpenDialog(Dialog::Preferences)).accel("Ctrl+,"),
                    w::MenuItem::action("Keyboard Shortcuts", Message::OpenDialog(Dialog::Shortcuts)).accel("Ctrl+?"),
                    w::MenuItem::action("Help", Message::ShowToast("Opening Help…")).accel("F1"),
                    w::MenuItem::action("About iced-libadwaita", Message::OpenDialog(Dialog::About)),
                ]),
        )
        .presentation(presentation)
        .window_radius(corner)
        .open_initially(self.shots.is_some() && std::env::var_os("GALLERY_MENU").is_some());

        // Which buttons exist, on which side, follows the user's GNOME `button-layout`
        // (this machine: `appmenu:close`, so a lone close button at the end).
        let controls = || w::WindowControls {
            on_minimize: Some(Message::Minimize),
            on_maximize: Some(Message::ToggleMaximize),
            on_close: Some(Message::Close),
            maximized: self.win.maximized,
            layout: None,
        };
        let show_content_only = collapsed && self.show_content;
        let show_sidebar_only = collapsed && !self.show_content;

        // One header bar per pane, as in libadwaita split views. Window controls
        // appear only at the window edge (the end pane); the collapsed single pane has both.
        let mut sidebar_bar = w::header_bar::<Message>()
            .title("iced-libadwaita")
            .pack_end(primary)
            .window_corners(corner, if show_sidebar_only { corner } else { 0.0 })
            .metrics(m);
        let mut content_bar = w::header_bar::<Message>()
            .title(self.page.title())
            .window_corners(if show_content_only { corner } else { 0.0 }, corner)
            .metrics(m);
        if show_content_only {
            content_bar = content_bar.pack_start(w::circular_button(Icon::GoPrevious).on_press(Message::Back));
        }
        if csd {
            sidebar_bar = sidebar_bar.on_drag(Message::Drag).on_double_click(Message::ToggleMaximize);
            content_bar = content_bar.on_drag(Message::Drag).on_double_click(Message::ToggleMaximize);
            // Controls only outside mobile mode. In a split view each pane shows only the
            // buttons at its own window edge; a collapsed single pane shows both sides.
            if !mobile {
                if show_sidebar_only {
                    sidebar_bar = sidebar_bar.window_controls(controls());
                } else if show_content_only {
                    content_bar = content_bar.window_controls(controls());
                } else {
                    sidebar_bar = sidebar_bar.window_controls(controls()).show_end_title_buttons(false);
                    content_bar = content_bar.window_controls(controls()).show_start_title_buttons(false);
                }
            }
        }
        let bar_style = if self.raised_bar { w::BarStyle::Raised } else { w::BarStyle::Flat };

        let sidebar_list = w::navigation_sidebar_rounded(
            Page::ALL
                .iter()
                .map(|p| w::sidebar_row(Some(p.icon()), p.title(), *p == self.page, Message::Page(*p))),
            corner,
        );
        let sidebar_pane = w::toolbar_view(sidebar_list).top_bar(sidebar_bar).top_bar_style(bar_style);

        let page: Element = match self.page {
            Page::Buttons => self.buttons_page(),
            Page::Lists => self.lists_page(),
            Page::Inputs => self.inputs_page(),
            Page::Feedback => self.feedback_page(),
            Page::Dialogs => self.dialogs_page(),
            Page::Navigation => self.navigation_page(),
            Page::Theming => self.theming_page(),
        };

        let content_pane = w::toolbar_view(page).top_bar(content_bar).top_bar_style(bar_style);
        let body = w::NavigationSplitView::new(sidebar_pane, content_pane)
            .adapt(size.width)
            .show_content(self.show_content);

        let base: Element = w::toast_overlay(
            // The window background comes from `adw::window::frame` (rounded when floating).
            container(body).width(Length::Fill).height(Length::Fill),
            self.toast.clone().map(|t| w::Toast::new(t).on_dismiss(Message::DismissToast)),
        );

        let base: Element = w::bottom_sheet(
            base,
            column![ty::title_3("Sheet"), ty::body("A bottom sheet over the content."), w::suggested_button(text("Done")).on_press(Message::Sheet(false))].spacing(m.space_m),
            self.sheet,
            Message::Sheet(false),
        );
        let result: Element<'_> = match self.dialog {
            None => base,
            Some(Dialog::Alert) => w::dialog(
                base,
                w::AlertDialog::new("Discard Changes?")
                    .body("Unsaved changes will be permanently lost.")
                    .response("Cancel", w::ResponseAppearance::Default, Message::CloseDialog)
                    .response("Save", w::ResponseAppearance::Suggested, Message::CloseDialog)
                    .response("Discard", w::ResponseAppearance::Destructive, Message::CloseDialog),
                Some(Message::CloseDialog),
            ),
            Some(Dialog::Shortcuts) => w::dialog(
                base,
                w::shortcuts_dialog(
                    vec![
                        ("General".into(), vec![("Open Menu".into(), "F10".into()), ("Quit".into(), "Ctrl+Q".into())]),
                        ("Editing".into(), vec![("Undo".into(), "Ctrl+Z".into()), ("Redo".into(), "Ctrl+Shift+Z".into())]),
                    ],
                    Message::CloseDialog,
                ),
                Some(Message::CloseDialog),
            ),
            Some(Dialog::Preferences) => w::dialog(
                base,
                w::preferences_dialog("Preferences", self.lists_page(), Message::CloseDialog),
                Some(Message::CloseDialog),
            ),
            Some(Dialog::About) => w::dialog(
                base,
                w::AboutDialog::new("Gallery")
                    .developer_name("The iced-libadwaita authors")
                    .version("0.1.0")
                    .comments("Widget gallery for iced-libadwaita.")
                    .website("https://iced.rs")
                    .license("MIT OR Apache-2.0")
                    .application_icon(Icon::Home.colored(48.0, self.theme().semantic.accent.color))
                    .on_close(Message::CloseDialog),
                Some(Message::CloseDialog),
            ),
        };
        if csd { adw::window::frame(result, win_state, Message::Resize) } else { result }
    }

    // --- pages ----------------------------------------------------------

    fn page_frame<'a>(&self, content: impl Into<Element<'a>>) -> Element<'a> {
        let m = self.metrics();
        w::scrolled_window(w::clamp(
            container(content).padding(m.page_padding).width(Length::Fill),
            m.clamp_maximum + 120.0,
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn buttons_page(&self) -> Element<'_> {
        let m = self.metrics();
        let g = |title: &'static str| w::PreferencesGroup::<Message>::new().title(title).metrics(m);
        let lbl = |s: &'static str| -> Element<'static> { w::typography::label(s).into() };

        let plain = row![
            w::raised_button(lbl("Default")).on_press(Message::Noop),
            w::flat_button(lbl("Flat")).on_press(Message::Noop),
            w::suggested_button(lbl("Suggested")).on_press(Message::Noop),
            w::destructive_button(lbl("Destructive")).on_press(Message::Noop),
            w::raised_button(lbl("Disabled")),
        ]
        .spacing(m.space_m)
        .wrap();

        let shapes = row![
            w::pill_button(lbl("Pill")).on_press(Message::Noop),
            w::suggested_pill_button(lbl("Suggested Pill")).on_press(Message::Noop),
            w::circular_button(Icon::ListAdd).on_press(Message::Noop),
            w::icon_button(Icon::Delete).on_press(Message::Noop),
            iced::widget::button(text("Link")).style(style::button::link).on_press(Message::Noop),
            iced::widget::button(text("Card button").width(Length::Fixed(110.0)))
                .padding(m.card_padding)
                .style(style::button::card)
                .on_press(Message::Noop),
        ]
        .spacing(m.space_m)
        .align_y(iced::Alignment::Center)
        .wrap();

        let with_icon = row![
            w::raised_button(Element::from(w::ButtonContent::new("Open").icon(Icon::Home).metrics(m)))
                .on_press(Message::Noop),
            w::suggested_button(Element::from(w::ButtonContent::new("Add").icon(Icon::ListAdd)))
                .on_press(Message::Noop),
            w::split_button(text("Split"), Message::Noop, Message::ShowToast("Split menu")),
            Element::from(w::toggle_group(
                vec![
                    w::ToggleItem::new(Tab::One, "One"),
                    w::ToggleItem::new(Tab::Two, "Two"),
                    w::ToggleItem::new(Tab::Three, "Three"),
                ],
                Some(self.tab),
                Message::Tab,
            )),
        ]
        .spacing(m.space_m)
        .align_y(iced::Alignment::Center)
        .wrap();

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
            row![
                ty::body("Accent").style(style::text::accent),
                ty::body("Success").style(style::text::success),
                ty::body("Warning").style(style::text::warning),
                ty::body("Error").style(style::text::error),
                ty::body("Destructive").style(style::text::destructive),
            ]
            .spacing(m.space_l),
        ]
        .spacing(m.space_xs);

        self.page_frame(
            column![
                g("Styles").push(w::card(plain)),
                g("Shapes").push(w::card(shapes)),
                g("Content").push(w::card(with_icon)),
                g("Typography").push(w::card(typo)),
            ]
            .spacing(m.group_spacing),
        )
    }

    fn lists_page(&self) -> Element<'_> {
        let m = self.metrics();
        let mut page = w::PreferencesPage::<Message>::new().metrics(m);

        page = page.push(
            w::PreferencesGroup::new()
                .title("Appearance")
                .description("Rows, switches and choosers.")
                .metrics(m)
                .push(w::SwitchRow::new("Dark Style", self.scheme.is_dark(), Message::Scheme)
                    .subtitle("Switches the whole gallery")
                    .metrics(m))
                .push(w::SwitchRow::new("Wi-Fi", self.wifi, Message::Wifi).metrics(m))
                .push(w::ComboRow::new("Font", &COMBO, self.combo, Message::Combo)
                    .subtitle("Interface font")
                    .open_initially(self.shots.is_some()))
                .push(w::SpinRow::new("Text Scale", self.spin, 1.0..=10.0, 1.0, Message::Spin)),
        );

        page = page.push(
            w::PreferencesGroup::new()
                .title("Account")
                .metrics(m)
                .push(
                    w::ActionRow::new("Ada Lovelace")
                        .subtitle("ada@example.org")
                        .prefix(w::avatar(40.0, "Ada Lovelace"))
                        .chevron(true)
                        .on_activate(Message::ShowToast("Opened account"))
                        .metrics(m),
                )
                .push(w::EntryRow::new("Display name", &self.name).on_input(Message::Name).metrics(m))
                .push(
                    w::EntryRow::new("Password", &self.password)
                        .on_input(Message::Password)
                        .password(true)
                        .metrics(m),
                ),
        );

        page = page.push(
            w::PreferencesGroup::new()
                .title("Advanced")
                .header_suffix(w::flat_button(text("Reset")).on_press(Message::Noop))
                .metrics(m)
                .push(
                    w::ExpanderRow::new("Notifications", self.expanded, Message::Expanded)
                        .subtitle("Expand for details")
                        .push(w::SwitchRow::new("Sounds", self.notifications, Message::Notifications).metrics(m))
                        .push(w::ActionRow::new("Do Not Disturb").subtitle("Until tomorrow").metrics(m)),
                )
                .push(
                    w::ButtonRow::new("Sign Out")
                        .role(Role::Destructive)
                        .on_activate(Message::OpenDialog(Dialog::Alert)),
                ),
        );
        page.into()
    }

    fn inputs_page(&self) -> Element<'_> {
        let m = self.metrics();
        let g = |title: &'static str| w::PreferencesGroup::<Message>::new().title(title).metrics(m);
        let entries = column![
            text_input("Search…", &self.name).on_input(Message::Name).padding(m.entry_padding),
            text_input("Invalid value", "oops")
                .style(style::text_input::error)
                .padding(m.entry_padding),
            text_input("Disabled", "read only").padding(m.entry_padding),
            text_editor(&self.editor).on_action(Message::Editor).height(90),
        ]
        .spacing(m.space_m);

        let toggles = column![
            checkbox(self.check).label("Checkbox").on_toggle(Message::Check),
            checkbox(false).label("Unchecked"),
            checkbox(true).label("Disabled checked"),
            row![
                radio("Radio A", 0u8, self.radio, Message::Radio),
                radio("Radio B", 1u8, self.radio, Message::Radio),
                radio("Radio C", 2u8, self.radio, Message::Radio),
            ]
            .spacing(m.space_l)
            .wrap(),
            toggler(self.wifi).label("Switch").on_toggle(Message::Wifi).size(m.switch_size),
        ]
        .spacing(m.space_m);

        let range = column![
            slider(0.0..=100.0, self.slider, Message::Slider),
            w::progress_bar(0.0..=100.0, self.slider),
            w::progress_bar(0.0..=100.0, self.slider).style(style::progress_bar::success),
            w::progress_bar(0.0..=100.0, self.slider).style(style::progress_bar::warning),
            w::progress_bar(0.0..=100.0, self.slider).style(style::progress_bar::error),
            Element::from(w::dropdown(&FRUITS, self.picked, Message::Picked)),
        ]
        .spacing(m.space_m);

        self.page_frame(
            column![
                g("Entries").push(w::card(entries)),
                g("Toggles").push(w::card(toggles)),
                g("Ranges & Choosers").push(w::card(range)),
            ]
            .spacing(m.group_spacing),
        )
    }

    fn feedback_page(&self) -> Element<'_> {
        let m = self.metrics();
        let g = |title: &'static str| w::PreferencesGroup::<Message>::new().title(title).metrics(m);

        let avatars = row![
            w::avatar(32.0, "Ada Lovelace"),
            w::avatar(48.0, "Grace Hopper"),
            w::avatar(64.0, "Linus"),
            w::avatar(48.0, "Margaret Hamilton").show_initials(false),
            w::spinner(32.0, self.angle),
        ]
        .spacing(m.space_m)
        .align_y(iced::Alignment::Center);

        let tags = row![
            tag("Accent", Role::Accent),
            tag("Success", Role::Success),
            tag("Warning", Role::Warning),
            tag("Error", Role::Error),
            tag("Destructive", Role::Destructive),
        ]
        .spacing(m.space_s);

        let status = container(
            w::status_page::<Message>()
                .icon(Icon::DialogInformation.colored(64.0, self.theme().semantic.dim_solid()))
                .title("No Results")
                .description("Try a different search term.")
                .child(w::suggested_pill_button(text("Clear Search")).on_press(Message::Noop))
                .compact(true)
                .metrics(m),
        )
        .height(Length::Fixed(300.0))
        .style(style::container::card);

        let banner: Element = w::banner("Network connection lost")
            .button("Retry", Message::DismissBanner)
            .revealed(self.banner)
            .into();

        column![
            banner,
            self.page_frame(
                column![
                    g("Avatars & Spinner").push(w::card(avatars)),
                    g("Tags").push(w::card(tags)),
                    g("Status Page").push(status),
                ]
                .spacing(m.group_spacing)
            )
        ]
        .into()
    }

    fn dialogs_page(&self) -> Element<'_> {
        let m = self.metrics();
        let buttons = row![
            w::raised_button(text("Alert Dialog")).on_press(Message::OpenDialog(Dialog::Alert)),
            w::raised_button(text("About Dialog")).on_press(Message::OpenDialog(Dialog::About)),
            w::raised_button(text("Shortcuts Dialog")).on_press(Message::OpenDialog(Dialog::Shortcuts)),
            w::raised_button(text("Preferences Dialog")).on_press(Message::OpenDialog(Dialog::Preferences)),
            w::raised_button(text("Show Toast")).on_press(Message::ShowToast("Settings saved")),
        ]
        .spacing(m.space_m)
        .wrap();
        self.page_frame(
            w::PreferencesGroup::<Message>::new()
                .title("Overlays")
                .description("Modal dialogs and toasts are layered with iced's stack.")
                .metrics(m)
                .push(w::card(buttons)),
        )
    }

    fn navigation_page(&self) -> Element<'_> {
        let m = self.metrics();
        let g = |title: &'static str| w::PreferencesGroup::<Message>::new().title(title).metrics(m);
        let items = || {
            vec![
                w::ToggleItem::new(Tab::One, "Files").icon(Icon::Home),
                w::ToggleItem::new(Tab::Two, "Search").icon(Icon::Search),
                w::ToggleItem::new(Tab::Three, "Settings").icon(Icon::Settings),
            ]
        };
        let pages: Vec<Element> = ["First page", "Second page", "Third page"]
            .into_iter()
            .map(|t| -> Element { container(ty::title_2(t)).center(Length::Fill).into() })
            .collect();
        let car = container(w::carousel(pages, self.carousel, Message::Carousel))
            .height(Length::Fixed(140.0))
            .style(style::container::card);
        let tabs = w::tab_bar(
            vec!["Home".into(), "Docs".into(), "Settings".into()],
            self.tab as usize,
            |i| Message::Tab([Tab::One, Tab::Two, Tab::Three][i]),
            Some(|_| Message::Noop),
        );
        let shortcuts = column![
            row![ty::body("Copy"), w::shortcut_label("Ctrl+C")].spacing(m.space_m),
            row![ty::body("Palette"), w::shortcut_label("Ctrl+Shift+P")].spacing(m.space_m),
        ]
        .spacing(m.space_s);
        let tags: Vec<Element> = ["rust", "iced", "gnome", "adwaita", "libadwaita", "theming", "widgets"]
            .into_iter()
            .map(|t| tag(t, Role::Accent))
            .collect();
        self.page_frame(
            column![
                g("View Switcher").push(w::card(Element::from(w::view_switcher(items(), Some(self.tab), Message::Tab)))),
                g("Tab Bar").push(container(tabs).style(style::container::card)),
                g("Carousel").push(car).push(w::carousel_indicator_lines(3, self.carousel, Message::Carousel)),
                g("Shortcut Labels").push(w::card(shortcuts)),
                g("Wrap Box").push(w::card(w::wrap_box(tags, m.space_s))),
                g("Bottom Sheet").push(w::card(
                    w::raised_button(text("Open Sheet")).on_press(Message::Sheet(true)),
                )),
            ]
            .spacing(m.group_spacing),
        )
    }

    fn theming_page(&self) -> Element<'_> {
        let m = self.metrics();
        let sem = self.theme().semantic;

        let accents = row(AccentColor::ALL.iter().map(|a| {
            let c = a.to_color();
            let selected = *a == self.accent;
            iced::widget::button(
                container(if selected {
                    Icon::Check.colored(14.0, iced::Color::WHITE).into()
                } else {
                    Element::from(text(""))
                })
                .center(Length::Fixed(26.0)),
            )
            .padding(0)
            .style(move |_: &Theme, _| iced::widget::button::Style {
                background: Some(iced::Background::Color(c)),
                border: iced::Border::default().rounded(999.0),
                ..Default::default()
            })
            .on_press(Message::Accent(*a))
            .into()
        }))
        .spacing(m.space_s)
        .wrap();

        let swatch = |name: &'static str, s: adw::Surface| -> Element<'static> {
            container(text(name).size(12).color(s.fg))
                .padding([m.space_m, m.space_m])
                .width(Length::Fill)
                
                .style(move |t: &Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(s.bg)),
                    border: iced::Border {
                        color: adw::color::alpha(s.fg, 0.2),
                        width: 1.0,
                        radius: t.metrics.radius_m.into(),
                    },
                    ..Default::default()
                })
                .into()
        };
        let sw = |name: &'static str, s: adw::Swatch| swatch(name, s.surface());

        let surfaces = column![
            row![swatch("window", sem.window), swatch("view", sem.view), swatch("headerbar", sem.headerbar)].spacing(m.space_s),
            row![swatch("card", sem.card), swatch("dialog", sem.dialog), swatch("popover", sem.popover)].spacing(m.space_s),
            row![swatch("sidebar", sem.sidebar), swatch("thumbnail", sem.thumbnail), swatch("osd", sem.osd)].spacing(m.space_s),
            row![sw("accent", sem.accent), sw("destructive", sem.destructive), sw("success", sem.success)].spacing(m.space_s),
            row![sw("warning", sem.warning), sw("error", sem.error)].spacing(m.space_s),
        ]
        .spacing(m.space_s);

        let rows = w::PreferencesGroup::new()
            .title("Theme Variables")
            .description("Everything here is a token on Theme: palette, semantic, metrics.")
            .metrics(m)
            .push(w::SwitchRow::new("Dark", self.scheme.is_dark(), Message::Scheme).metrics(m))
            .push(w::SwitchRow::new("Compact metrics", self.compact, Message::Compact).metrics(m))
            .push(w::SwitchRow::new("Override card color", self.card_override, Message::CardOverride).metrics(m))
            .push(
                w::ActionRow::new("Accent color")
                    .suffix(Element::from(accents))
                    .metrics(m),
            )
            .push(
                w::ActionRow::new("Menu presentation")
                    .subtitle("Auto: full page at ≤ 450px, else dropdown")
                    .suffix(Element::from(w::toggle_group(
                        vec![
                            w::ToggleItem::new(MenuMode::Auto, "Auto"),
                            w::ToggleItem::new(MenuMode::Dropdown, "Dropdown"),
                            w::ToggleItem::new(MenuMode::FullPage, "Full page"),
                        ],
                        Some(self.menu_mode),
                        Message::MenuMode,
                    )))
                    .metrics(m),
            )
            .push(w::SwitchRow::new("Shadows", self.shadows_on, Message::Shadows).subtitle("Adwaita elevation: cards, popovers, dialogs").metrics(m))
            .push(
                w::ActionRow::new("Shadow intensity")
                    .subtitle(format!("{:.1}x", self.shadow_intensity))
                    .suffix(slider(0.0..=2.0, self.shadow_intensity, Message::ShadowIntensity).step(0.1_f32).width(Length::Fill))
                    .suffix_basis(160.0)
                    .metrics(m),
            )
            .push(
                w::ActionRow::new("Shadow size")
                    .subtitle(format!("{:.1}x", self.shadow_scale))
                    .suffix(slider(0.0..=2.0, self.shadow_scale, Message::ShadowScale).step(0.1_f32).width(Length::Fill))
                    .suffix_basis(160.0)
                    .metrics(m),
            )
            .push(w::SwitchRow::new("Raised header bar", self.raised_bar, Message::RaisedBar).subtitle("top-bar-style: raised").metrics(m))
            .push(
                w::ActionRow::new("Control radius")
                    .subtitle(format!("radius_m = {:.0}px", self.radius_m))
                    .suffix(slider(0.0..=Metrics::MAX_RADIUS_M, self.radius_m, Message::RadiusM).width(Length::Fill))
                    .suffix_basis(160.0)
                    .metrics(m),
            )
            .push(
                w::ActionRow::new("Card radius")
                    .subtitle(format!("radius_l = {:.0}px", self.radius_l))
                    .suffix(slider(0.0..=Metrics::MAX_RADIUS_L, self.radius_l, Message::RadiusL).width(Length::Fill))
                    .suffix_basis(160.0)
                    .metrics(m),
            );

        // Everything here reads the theme's radii at draw time, so the sliders above
        // visibly change it: control radius (radius_m) rounds buttons, entries and
        // toggles; card radius (radius_l) rounds the card and boxed list.
        let preview = column![
            row![
                w::raised_button(w::typography::label("Default")).on_press(Message::Noop),
                w::suggested_button(w::typography::label("Suggested")).on_press(Message::Noop),
                w::destructive_button(w::typography::label("Destructive")).on_press(Message::Noop),
            ]
            .spacing(m.space_s)
            .wrap(),
            text_input("Entry", &self.name).on_input(Message::Name).padding(m.entry_padding),
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

        self.page_frame(
            column![
                rows,
                w::PreferencesGroup::<Message>::new()
                    .title("Collapse Behavior")
                    .description("Narrow the window: the control wraps. Stacked = 100% width; Flex = keeps its width.")
                    .metrics(m)
                    .push(
                        w::ActionRow::new("Stacked (default)")
                            .subtitle("collapse = Stacked")
                            .suffix(slider(0.0..=100.0, self.slider, Message::Slider).width(Length::Fill))
                            .suffix_basis(260.0)
                            .collapse(adw::CollapseBehavior::Stacked)
                            .metrics(m),
                    )
                    .push(
                        w::ActionRow::new("Flex")
                            .subtitle("collapse = Flex")
                            .suffix(slider(0.0..=100.0, self.slider, Message::Slider).width(Length::Fill))
                            .suffix_basis(260.0)
                            .collapse(adw::CollapseBehavior::Flex)
                            .metrics(m),
                    ),
                w::PreferencesGroup::<Message>::new().title("Live Preview").metrics(m).push(w::card(preview)),
                w::PreferencesGroup::<Message>::new().title("Semantic Surfaces").metrics(m).push(w::card(surfaces)),
            ]
            .spacing(m.group_spacing),
        )
    }
}

fn tag(label: &'static str, role: Role) -> Element<'static> {
    container(text(label).size(13))
        .padding([2.0, 10.0])
        .style(style::container::tinted(role))
        .into()
}

fn main() -> iced::Result {
    iced::application(Gallery::new, Gallery::update, Gallery::view)
        .title("iced-libadwaita gallery")
        .theme(Gallery::theme)
        .subscription(Gallery::subscription)
        .settings(iced::Settings {
            default_text_size: 14.67.into(),
            antialiasing: true,
            ..Default::default()
        })
        .window(window::Settings {
            size: std::env::var("GALLERY_SIZE")
                .ok()
                .and_then(|v| {
                    let (w, h) = v.split_once('x')?;
                    Some(iced::Size::new(w.parse().ok()?, h.parse().ok()?))
                })
                .unwrap_or(iced::Size::new(1040.0, 760.0)),
            // Adwaita has no server-side decorations; opt back in with GALLERY_SSD=1.
            ..adw::window::settings_with_ssd(!Gallery::csd())
        })
        .run()
}
