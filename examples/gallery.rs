//! Widget gallery: exercises the theme, style classes and Adw widgets.
//!
//! ```text
//! cargo run --example gallery
//! cargo run --example gallery -- --screenshot target/shots   # renders every page, light + dark
//! ```

use iced::widget::{
    column, container, row, text,
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

/// The motions the Theming page can play: libadwaita easings and the sheet spring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoMotion {
    Linear,
    StyleEaseOutQuad,
    Ease,
    EaseOutCubic,
    EaseOutBack,
    EaseOutBounce,
    EaseOutElastic,
    Spring,
}

impl std::fmt::Display for DemoMotion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

impl DemoMotion {
    const ALL: [DemoMotion; 8] = [
        DemoMotion::Linear,
        DemoMotion::StyleEaseOutQuad,
        DemoMotion::Ease,
        DemoMotion::EaseOutCubic,
        DemoMotion::EaseOutBack,
        DemoMotion::EaseOutBounce,
        DemoMotion::EaseOutElastic,
        DemoMotion::Spring,
    ];

    fn label(self) -> &'static str {
        match self {
            DemoMotion::Linear => "Linear",
            DemoMotion::StyleEaseOutQuad => "Style 200ms",
            DemoMotion::Ease => "Ease",
            DemoMotion::EaseOutCubic => "OutCubic",
            DemoMotion::EaseOutBack => "OutBack",
            DemoMotion::EaseOutBounce => "OutBounce",
            DemoMotion::EaseOutElastic => "OutElastic",
            DemoMotion::Spring => "Sheet spring",
        }
    }

    fn spec(self) -> adw::animation::Spec {
        use adw::animation::{presets, Easing, Spec};
        let ms = std::time::Duration::from_millis(600);
        match self {
            DemoMotion::Linear => Spec::Timed(ms, Easing::Linear),
            DemoMotion::StyleEaseOutQuad => Spec::STYLE,
            DemoMotion::Ease => Spec::Timed(ms, Easing::Ease),
            DemoMotion::EaseOutCubic => Spec::Timed(ms, Easing::EaseOutCubic),
            DemoMotion::EaseOutBack => Spec::Timed(ms, Easing::EaseOutBack),
            DemoMotion::EaseOutBounce => Spec::Timed(ms, Easing::EaseOutBounce),
            DemoMotion::EaseOutElastic => Spec::Timed(ms, Easing::EaseOutElastic),
            DemoMotion::Spring => Spec::Spring(presets::sheet(false)),
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
    path_bar_title: bool,
    anim_enabled: bool,
    demo: DemoMotion,
    demo_tween: adw::animation::Tween,
    demo_value: f32,

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
    editor: w::TextContent,
    picked: Option<&'static str>,

    dialog: Option<Dialog>,
    show_content: bool,
    menu_mode: MenuMode,
    sidebar_mode: w::CollapsedSidebar,
    drawer_open: bool,
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
    PathBarTitle(bool),
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
    Editor(w::TextAction),
    Picked(&'static str),
    OpenDialog(Dialog),
    CloseDialog,
    Sheet(bool),
    Carousel(usize),
    ShowToast(&'static str),
    DismissToast,
    DismissBanner,
    Frame(std::time::Instant),
    Animations(bool),
    DemoMotion(DemoMotion),
    DemoPlay,
    Noop,
    Drag,
    Close,
    Minimize,
    ToggleMaximize,
    Resize(window::Direction),
    Back,
    MenuMode(MenuMode),
    SidebarMode(w::CollapsedSidebar),
    Drawer(bool),
    ToggleFullscreen,
    WindowChanged,
    WindowState(adw::window::State),
    // screenshot mode
    Shot(usize),
    Capture(usize),
    Saved(usize, window::Screenshot),
}

static SIDEBAR_MODES: [w::CollapsedSidebar; 3] = w::CollapsedSidebar::ALL;
static MOTIONS: [DemoMotion; 8] = DemoMotion::ALL;
const FRUITS: [&str; 3] = ["Apple", "Banana", "Cherry"];
const COMBO: [&str; 3] = ["Adwaita Sans", "Cantarell", "Inter"];

impl Gallery {
    fn new() -> (Self, Task<Message>) {
        // Desktop-level behavior comes from the shared config; the gallery's own switches
        // below are demos layered on top of it.
        let cfg = adw::config::current();
        let from_cfg = Theme::from_config(&cfg);
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
                scheme: from_cfg.scheme,
                accent: match cfg.accent {
                    adw::config::Pick::Fixed(a) => a,
                    adw::config::Pick::System => adw::system::accent_color().unwrap_or(AccentColor::Blue),
                },
                compact: cfg.density == adw::config::Density::Compact,
                radius_m: std::env::var("GALLERY_RM").ok().and_then(|v| v.parse().ok()).unwrap_or(Metrics::default().radius_m),
                radius_l: std::env::var("GALLERY_RL").ok().and_then(|v| v.parse().ok()).unwrap_or(12.0),
                card_override: false,
                shadows_on: cfg.shadows,
                shadow_scale: 1.0,
                shadow_intensity: 1.0,
                path_bar_title: std::env::var_os("GALLERY_PATHBAR").is_some(),
                anim_enabled: adw::animation::enabled(),
                demo: DemoMotion::EaseOutCubic,
                demo_tween: adw::animation::Tween::new(0.0),
                demo_value: 0.0,
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
                editor: w::TextContent::with_text("Multi-line text editor\nstyled by the theme."),
                picked: None,
                dialog: None,
                show_content: std::env::var_os("GALLERY_SIDEBAR").is_none(),
                sidebar_mode: match std::env::var("GALLERY_SIDEBAR_MODE").as_deref() {
                    Ok("page") => w::CollapsedSidebar::FullPage,
                    Ok("first-page") => w::CollapsedSidebar::FirstPage,
                    _ => cfg.mobile_sidebar,
                },
                drawer_open: std::env::var_os("GALLERY_DRAWER").is_some(),
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
                self.drawer_open = false;
            }
            Message::SidebarMode(m) => {
                self.sidebar_mode = m;
                self.drawer_open = false;
            }
            Message::Drawer(open) => self.drawer_open = open,
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
            Message::PathBarTitle(v) => self.path_bar_title = v,
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
            Message::Frame(now) => {
                self.angle += 0.12;
                self.demo_value = self.demo_tween.tick(now);
            }
            Message::Animations(on) => {
                self.anim_enabled = on;
                adw::animation::set_enabled(Some(on));
            }
            Message::DemoMotion(m) => self.demo = m,
            Message::DemoPlay => {
                let now = std::time::Instant::now();
                let to = if self.demo_tween.target() > 0.5 { 0.0 } else { 1.0 };
                self.demo_tween.set_target(to, now, self.demo.spec());
            }
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
        // GALLERY_ANIM=switch|drawer|reveal|dialog|toast: capture an animation instead of the
        // normal pages: settled start, then +70ms, +160ms and settled end after a trigger.
        if let Ok(kind) = std::env::var("GALLERY_ANIM") {
            let wait = |ms: u64, tag: &'static str, this: &mut Self| {
                this.shot_tag = Some(tag);
                Task::perform(tokio::time::sleep(Duration::from_millis(ms)), move |_| Message::Capture(i))
            };
            return match i {
                0 => {
                    self.scheme = Scheme::Light;
                    wait(700, "a0-start", self)
                }
                1 => {
                    let trigger = match kind.as_str() {
                        "switch" => Message::Wifi(!self.wifi),
                        "drawer" => Message::Drawer(true),
                        "reveal" => Message::Expanded(!self.expanded),
                        "carousel" => Message::Carousel(1),
                        "dialog" => Message::OpenDialog(Dialog::Alert),
                        _ => Message::ShowToast("Settings saved"),
                    };
                    Task::done(trigger).chain(wait(70, "a1-70ms", self))
                }
                2 => wait(90, "a2-160ms", self),
                3 => wait(700, "a3-end", self),
                _ => iced::exit(),
            };
        }
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
        let frames = if self.page == Page::Feedback || self.shots.is_some() || self.demo_tween.is_animating() {
            window::frames().map(Message::Frame)
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
            && adw::config::current().decorations == adw::config::DecorationMode::Csd
    }

    fn root(&self, size: iced::Size) -> IcedElement<'_, Message, Theme> {
        let m = self.metrics();
        let csd = Self::csd();
        let collapsed = size.width <= w::collapse_width();

        // Floating windows have rounded corners; maximized/fullscreen go edge to edge.
        // Mobile mode (narrow): no window decorations at all. Corners still follow the
        // real window state: rounded while floating, edge to edge only when the window
        // is actually maximized or fullscreen.
        let mobile = adw::window::is_mobile(size.width);
        let win_state = self.win;
        let corner = if csd { win_state.radius(&adw::Metrics::default()) } else { 0.0 };

        // Primary menu (HIG): open-menu icon, above the sidebar list, standard items last.
        let presentation = match self.menu_mode {
            MenuMode::Auto => w::MenuPresentation::resolve(size.width),
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
        // Collapsed, the sidebar is a drawer over the content (default) or its own page.
        let drawer_mode = collapsed && self.sidebar_mode == w::CollapsedSidebar::Drawer;
        let show_content_only = collapsed && !drawer_mode && self.show_content;
        let show_sidebar_only = collapsed && !drawer_mode && !self.show_content;

        // One header bar per pane, as in libadwaita split views. Window controls
        // appear only at the window edge (the end pane); the collapsed single pane has both.
        let mut sidebar_bar = w::header_bar::<Message>()
            .title("iced-libadwaita")
            .pack_end(primary)
            .window_corners(corner, if show_sidebar_only { corner } else { 0.0 })
            .metrics(m);
        let mut content_bar = w::header_bar::<Message>()
            .title(self.page.title())
            .window_corners(if show_content_only || drawer_mode { corner } else { 0.0 }, corner)
            .metrics(m);
        if self.path_bar_title {
            // Replaces the title; the window buttons are independent of it.
            content_bar = content_bar.title_widget(w::path_bar(
                vec!["Home".into(), "Projects".into(), "iced-libadwaita".into()],
                2,
                |_| Message::Noop,
            ));
        }
        if drawer_mode {
            // The drawer toggle: the sidebar lives behind it.
            content_bar = content_bar.pack_start(w::circular_button(Icon::Sidebar).on_press(Message::Drawer(true)));
        }
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

        let sidebar_list = w::navigation_sidebar_rounded(
            Page::ALL
                .iter()
                .map(|p| w::sidebar_row(Some(p.icon()), p.title(), *p == self.page, Message::Page(*p))),
            corner,
        );
        let sidebar_pane = w::toolbar_view(sidebar_list).top_bar(sidebar_bar);

        let page: Element = match self.page {
            Page::Buttons => self.buttons_page(),
            Page::Lists => self.lists_page(),
            Page::Inputs => self.inputs_page(),
            Page::Feedback => self.feedback_page(),
            Page::Dialogs => self.dialogs_page(),
            Page::Navigation => self.navigation_page(),
            Page::Theming => self.theming_page(),
        };

        let content_pane = w::toolbar_view(page).top_bar(content_bar);
        let body = w::NavigationSplitView::new(sidebar_pane, content_pane)
            .adapt(size.width)
            .window_radius(corner)
            .collapsed_sidebar(self.sidebar_mode)
            .show_sidebar(self.drawer_open)
            .on_close_drawer(Message::Drawer(false))
            .show_content(self.show_content);

        let base: Element = w::toast_overlay(
            // The window background comes from `adw::window::frame` (rounded when floating).
            container(body).width(Length::Fill).height(Length::Fill),
            self.toast.clone().map(|t| w::Toast::new(t).on_dismiss(Message::DismissToast)),
        );

        let base: Element = w::bottom_sheet_in_window(
            base,
            column![ty::title_3("Sheet"), ty::body("A bottom sheet over the content."), w::suggested_button(text("Done")).on_press(Message::Sheet(false))].spacing(m.space_m),
            self.sheet,
            Message::Sheet(false),
            corner,
        );
        let result: Element<'_> = match self.dialog {
            None => base,
            Some(Dialog::Alert) => w::dialog_in_window(
                base,
                w::AlertDialog::new("Discard Changes?")
                    .body("Unsaved changes will be permanently lost.")
                    .response("Cancel", w::ResponseAppearance::Default, Message::CloseDialog)
                    .response("Save", w::ResponseAppearance::Suggested, Message::CloseDialog)
                    .response("Discard", w::ResponseAppearance::Destructive, Message::CloseDialog),
                Some(Message::CloseDialog),
                corner,
            ),
            Some(Dialog::Shortcuts) => w::dialog_in_window(
                base,
                w::shortcuts_dialog(
                    vec![
                        ("General".into(), vec![("Open Menu".into(), "F10".into()), ("Quit".into(), "Ctrl+Q".into())]),
                        ("Editing".into(), vec![("Undo".into(), "Ctrl+Z".into()), ("Redo".into(), "Ctrl+Shift+Z".into())]),
                    ],
                    Message::CloseDialog,
                ),
                Some(Message::CloseDialog),
                corner,
            ),
            Some(Dialog::Preferences) => w::dialog_in_window(
                base,
                w::preferences_dialog("Preferences", self.lists_page(), Message::CloseDialog),
                Some(Message::CloseDialog),
                corner,
            ),
            Some(Dialog::About) => w::dialog_in_window(
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
                corner,
            ),
        };
        if csd { adw::window::frame(result, win_state, Message::Resize) } else { result }
    }

    // --- pages ----------------------------------------------------------

    fn page_frame<'a>(&self, content: impl Into<Element<'a>>) -> Element<'a> {
        let m = self.metrics();
        w::page(content).maximum_size(m.clamp_maximum + 120.0).metrics(m).into()
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
            w::link_button("Link").on_press(Message::Noop),
            w::card_button(text("Card button").width(Length::Fixed(110.0))).on_press(Message::Noop),
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
            w::entry("Search…", &self.name).on_input(Message::Name).metrics(m),
            w::entry("Invalid value", "oops").on_input(Message::Name).state(w::EntryState::Error).metrics(m),
            w::entry("Warning", "careful").on_input(Message::Name).state(w::EntryState::Warning).metrics(m),
            w::entry("Disabled", "read only").metrics(m),
            w::password_entry("Password", &self.password).on_input(Message::Password).metrics(m),
            w::text_view(&self.editor).on_action(Message::Editor),
        ]
        .spacing(m.space_m);

        let toggles = column![
            w::check_button("Checkbox", self.check).on_toggle(Message::Check),
            w::check_button("Unchecked", false),
            w::check_button("Disabled checked", true),
            row![
                w::radio_button("Radio A", 0u8, self.radio, Message::Radio),
                w::radio_button("Radio B", 1u8, self.radio, Message::Radio),
                w::radio_button("Radio C", 2u8, self.radio, Message::Radio),
            ]
            .spacing(m.space_l)
            .wrap(),
            row![
                w::switch(self.wifi).on_toggle(Message::Wifi),
                ty::body("Switch"),
            ]
            .spacing(m.space_m)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(m.space_m);

        let range = column![
            w::scale(0.0..=100.0, self.slider, Message::Slider),
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
            w::tag("Accent", Role::Accent),
            w::tag("Success", Role::Success),
            w::tag("Warning", Role::Warning),
            w::tag("Error", Role::Error),
            w::tag("Destructive", Role::Destructive),
        ]
        .spacing(m.space_s);

        let status = w::Card::new(
            w::status_page::<Message>()
                .icon(Icon::DialogInformation.colored(64.0, self.theme().semantic.dim_solid()))
                .title("No Results")
                .description("Try a different search term.")
                .child(w::suggested_pill_button(text("Clear Search")).on_press(Message::Noop))
                .compact(true)
                .metrics(m),
        )
        .padding(0)
        .height(Length::Fixed(300.0));

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
        // Each page has its own tint so the slide is easy to follow.
        let carousel_pages = || -> Vec<Element<'_>> {
            [("First page", Role::Accent), ("Second page", Role::Success), ("Third page", Role::Warning)]
                .into_iter()
                .map(|(t, role)| -> Element {
                    container(ty::title_2(t))
                        .center(Length::Fill)
                        .style(style::container::tinted(role))
                        .into()
                })
                .collect()
        };
        let car = w::Card::new(w::carousel(carousel_pages(), self.carousel, Message::Carousel))
            .padding(0)
            .height(Length::Fixed(160.0));
        let car_lines = w::Card::new(
            w::carousel(carousel_pages(), self.carousel, Message::Carousel).indicator(w::CarouselIndicator::Lines),
        )
        .padding(0)
        .height(Length::Fixed(160.0));
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
            .map(|t| w::tag(t, Role::Accent))
            .collect();
        self.page_frame(
            column![
                g("View Switcher").push(w::card(Element::from(w::view_switcher(items(), Some(self.tab), Message::Tab)))),
                g("Crowded Switcher (fills, shrinks, falls back to icons)").push(w::card(Element::from(
                    w::toggle_group(
                        vec![
                            w::ToggleItem::new(Tab::One, "Files").icon(Icon::Home),
                            w::ToggleItem::new(Tab::Two, "Search").icon(Icon::Search),
                            w::ToggleItem::new(Tab::Three, "Settings").icon(Icon::Settings),
                            w::ToggleItem::new(Tab::One, "Library").icon(Icon::Edit),
                            w::ToggleItem::new(Tab::Two, "Archive").icon(Icon::Delete),
                        ],
                        Some(self.tab),
                        Message::Tab,
                    )
                    .width(Length::Fill),
                ))),
                g("Tab Bar").push(w::Card::new(tabs).padding(0)),
                g("Carousel (drag, or click the dots)").push(car).push(car_lines),
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

        let surfaces = column![
            row![w::color_swatch("window", sem.window), w::color_swatch("view", sem.view), w::color_swatch("headerbar", sem.headerbar)].spacing(m.space_s),
            row![w::color_swatch("card", sem.card), w::color_swatch("dialog", sem.dialog), w::color_swatch("popover", sem.popover)].spacing(m.space_s),
            row![w::color_swatch("sidebar", sem.sidebar), w::color_swatch("thumbnail", sem.thumbnail), w::color_swatch("osd", sem.osd)].spacing(m.space_s),
            row![w::status_swatch("accent", sem.accent), w::status_swatch("destructive", sem.destructive), w::status_swatch("success", sem.success)].spacing(m.space_s),
            row![w::status_swatch("warning", sem.warning), w::status_swatch("error", sem.error)].spacing(m.space_s),
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
                    .suffix(w::accent_picker(self.accent, Message::Accent))
                    .metrics(m),
            )
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
            )
            .push(
                w::ComboRow::new(
                    "Mobile sidebar",
                    &SIDEBAR_MODES,
                    Some(self.sidebar_mode),
                    Message::SidebarMode,
                )
                .subtitle("Drawer over the content, its own page, or the first page of a stack"),
            )
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
            )
            .push(w::SwitchRow::new("Path bar title", self.path_bar_title, Message::PathBarTitle).subtitle("A title_widget replaces the title; window buttons stay").metrics(m))
            .push(
                w::SliderRow::new("Control radius", self.radius_m, 0.0..=Metrics::MAX_RADIUS_M, Message::RadiusM)
                    .subtitle(format!("radius_m = {:.0}px", self.radius_m))
                    .metrics(m),
            )
            .push(
                w::SliderRow::new("Card radius", self.radius_l, 0.0..=Metrics::MAX_RADIUS_L, Message::RadiusL)
                    .subtitle(format!("radius_l = {:.0}px", self.radius_l))
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

        self.page_frame(
            column![
                rows,
                w::PreferencesGroup::<Message>::new()
                    .title("Animations")
                    .description("libadwaita's easings and springs; honors the system enable-animations setting.")
                    .metrics(m)
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
                w::PreferencesGroup::<Message>::new()
                    .title("Collapse Behavior")
                    .description("Narrow the window: the control wraps. Stacked = 100% width; Flex = keeps its width.")
                    .metrics(m)
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
                w::PreferencesGroup::<Message>::new().title("Live Preview").metrics(m).push(w::card(preview)),
                w::PreferencesGroup::<Message>::new().title("Semantic Surfaces").metrics(m).push(w::card(surfaces)),
            ]
            .spacing(m.group_spacing),
        )
    }
}


fn main() -> iced::Result {
    // Shared desktop behavior: ~/.config/iced-libadwaita/desktop.conf (+ system files).
    let mut cfg = adw::config::Config::load();
    if std::env::var_os("GALLERY_OPT_OUT").is_some() {
        // Opting out of desktop settings is highly discouraged; shown here only to demonstrate
        // that it is allowed, explicit and reported (a notice is printed on install).
        cfg = cfg.opt_out(|o| {
            o.mobile_sidebar(w::CollapsedSidebar::FullPage);
        });
    }
    adw::config::install(cfg);
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
