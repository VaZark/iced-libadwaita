//! Widget gallery: exercises the theme, style classes and Adw widgets.
//!
//! ```text
//! cargo run --example gallery
//! cargo run --example gallery -- --screenshot target/shots   # renders every page, light + dark
//! ```

use adw::style;
use adw::widget::{self as w, typography as ty};
use adw::{AccentColor, Icon, Metrics, Role, Scheme, Theme};
use iced::widget::{column, container, text};
use iced::{Element as IcedElement, Length, Subscription, Task, window};
use iced_libadwaita as adw;
use std::path::PathBuf;
use std::time::{Duration, Instant};

type Element<'a> = adw::Element<'a, Message>;

mod appshell;
mod autocomplete_demo;
mod data_demo;
mod date_demo;
mod details;
mod field_demo;
mod form_demo;
mod nav;
mod playground;
mod range_demo;
mod reorder_demo;
mod resizable_demo;
mod sections_demo;
mod stepper_demo;

use nav::{Action, Detail, Entry, Inline, Page};

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
        use adw::animation::{Easing, Spec, presets};
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
    /// The nested component page open inside `page`, if any.
    detail: Option<Detail>,
    scheme: Scheme,
    accent: AccentColor,
    compact: bool,
    radius_m: f32,
    radius_l: f32,
    radius_toast: f32,
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
    editor_empty: w::TextContent,
    editor_ro: w::TextContent,
    sw: bool,
    row_sw: bool,
    combo2: Option<&'static str>,
    /// The selected action of the split-button-action demo.
    split_choice: &'static str,
    picked: Option<&'static str>,
    multi: Vec<&'static str>,
    date: date_demo::State,
    field: field_demo::State,
    form: form_demo::State,
    autocomplete: autocomplete_demo::State,
    sections: sections_demo::State,
    range: range_demo::State,
    stepper: stepper_demo::State,
    data: data_demo::State,
    resizable: resizable_demo::State,
    reorder: reorder_demo::State,
    chips: Vec<&'static str>,
    acc_single: Vec<u8>,
    acc_multi: Vec<u8>,

    dialog: Option<Dialog>,
    show_content: bool,
    menu_mode: MenuMode,
    sidebar_mode: w::CollapsedSidebar,
    sidebar_padding: w::SidebarPadding,
    drawer_open: bool,
    win: adw::window::State,
    /// The main window, and the preferences window while it is open (a separate OS window).
    main: window::Id,
    prefs: Option<adw::window::Child<()>>,
    /// The AppShell demo: its own window, with its own state.
    shell: Option<adw::window::Child<appshell::AppShell>>,
    /// Screenshot harness: the window to capture instead of the main one.
    shot_window: Option<window::Id>,
    sheet: bool,
    carousel: usize,
    carousel_indicator: w::CarouselIndicator,
    /// The toasts on screen, oldest first, each with its id (stacked notifications).
    toasts: adw::primitives::notifications::Queue<ToastSpec>,
    toast_seq: u32,
    /// Whether toasts dismiss themselves after 4 s (the queue's timeout).
    toast_auto: bool,
    /// Screenshot harness: behave as if the window were narrow.
    narrow: bool,
    toast_position: w::ToastPosition,
    toast_target: w::ToastTarget,
    banner: bool,
    angle: f32,

    shots: Option<Shots>,
    shot_tag: Option<&'static str>,
}

/// A toast to show: its title, optional call-to-action label, and status dot.
#[derive(Debug, Clone, Copy)]
struct ToastSpec {
    title: &'static str,
    action: Option<&'static str>,
    status: Option<Role>,
}

impl ToastSpec {
    const fn plain(title: &'static str) -> Self {
        Self {
            title,
            action: None,
            status: None,
        }
    }
}

struct Shots {
    dir: PathBuf,
    index: usize,
}

#[derive(Debug, Clone)]
enum Message {
    Page(Page),
    OpenDetail(Detail),
    CloseDetail,
    Banner(bool),
    Sw(bool),
    RowSw(bool),
    Combo2(&'static str),
    SplitChoice(&'static str),
    SplitRun(&'static str),
    Scheme(bool),
    Accent(AccentColor),
    Compact(bool),
    RadiusM(f32),
    RadiusL(f32),
    RadiusToast(f32),
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
    EditorEmpty(w::TextAction),
    Picked(&'static str),
    MultiToggle(&'static str),
    Date(date_demo::Msg),
    Field(field_demo::Msg),
    Form(form_demo::Msg),
    Autocomplete(autocomplete_demo::Msg),
    Sections(sections_demo::Msg),
    Range(range_demo::Msg),
    Stepper(stepper_demo::Msg),
    Data(data_demo::Msg),
    Resizable(resizable_demo::Msg),
    Reorder(reorder_demo::Msg),
    ChipRemove(&'static str),
    AccSingle(Vec<u8>),
    AccMulti(Vec<u8>),
    OpenDialog(Dialog),
    /// Opens the AppShell window (Layout > AppShell).
    OpenShell,
    Shell(appshell::Message),
    /// A window was resized; the AppShell window's width drives its layout.
    Resized(window::Id, f32),
    CloseDialog,
    Sheet(bool),
    Carousel(usize),
    CarouselIndicator(w::CarouselIndicator),
    ShowToast(&'static str),
    ShowToastSpec(ToastSpec),
    ToastPosition(w::ToastPosition),
    ToastTarget(w::ToastTarget),
    ToastAction(adw::primitives::notifications::Id),
    DismissToast(adw::primitives::notifications::Id),
    ToastAuto(bool),
    ToastTick(Instant),
    ClearToasts,
    Frame(std::time::Instant),
    Animations(bool),
    DemoMotion(DemoMotion),
    DemoPlay,
    Noop,
    Drag,
    /// Chrome of a secondary window (preferences, AppShell): performed on that window.
    ChildAction(window::Id, adw::window::Action),
    /// Closes the preferences window.
    ClosePrefs,
    /// The window was closed (by the user or the compositor).
    WindowClosed(window::Id),
    /// A click on the disabled parent: bring the preferences window forward.
    FocusPrefs,
    /// Escape: dismisses the topmost overlay.
    Escape,
    Narrow(bool),
    Close,
    Minimize,
    ToggleMaximize,
    Resize(window::Direction),
    Back,
    MenuMode(MenuMode),
    SidebarMode(w::CollapsedSidebar),
    SidebarPadding(w::SidebarPadding),
    Drawer(bool),
    ToggleFullscreen,
    WindowChanged,
    WindowState(adw::window::State),
    // screenshot mode
    Shot(usize),
    Capture(usize),
    Saved(usize, window::Screenshot),
}

static SIDEBAR_MODES: [w::CollapsedSidebar; 2] = w::CollapsedSidebar::ALL;
static INDICATORS: [w::CarouselIndicator; 3] = w::CarouselIndicator::ALL;
static TOAST_POSITIONS: [w::ToastPosition; 6] = w::ToastPosition::ALL;
static MOTIONS: [DemoMotion; 8] = DemoMotion::ALL;
const FRUITS: [&str; 3] = ["Apple", "Banana", "Cherry"];
const FRUITS5: [&str; 5] = ["Apple", "Banana", "Cherry", "Damson", "Elderberry"];

struct Person {
    name: &'static str,
    role: &'static str,
    team: &'static str,
    email: &'static str,
}
static PEOPLE: [Person; 4] = [
    Person {
        name: "Ada Lovelace",
        role: "Engineer",
        team: "Compilers",
        email: "ada@example.org",
    },
    Person {
        name: "Grace Hopper",
        role: "Admiral",
        team: "Languages",
        email: "grace@example.org",
    },
    Person {
        name: "Linus T.",
        role: "Maintainer",
        team: "Kernel",
        email: "linus@example.org",
    },
    Person {
        name: "Margaret H.",
        role: "Director",
        team: "Flight software",
        email: "margaret@example.org",
    },
];

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
            .map(|d| Shots {
                dir: d.into(),
                index: 0,
            });
        // GALLERY_DETAIL=textinput|switch|…: open a nested component page at startup.
        let start_detail = std::env::var("GALLERY_DETAIL").ok().and_then(|v| {
            Detail::all()
                .into_iter()
                .find(|d| format!("{d:?}").eq_ignore_ascii_case(&v))
        });
        let boot = if shots.is_some() {
            Task::done(Message::Shot(0))
        } else {
            Task::none()
        };
        // A daemon opens its windows itself: the main one now, the preferences window on demand.
        let (main, open_main) = window::open(Self::main_settings());
        let boot = Task::batch([
            open_main.map(|_| Message::Noop),
            boot,
            autocomplete_demo::script().map(Message::Autocomplete),
        ]);
        (
            Self {
                main,
                prefs: None,
                shell: None,
                shot_window: None,
                page: start_detail
                    .map(Detail::page)
                    .or_else(|| {
                        std::env::var("GALLERY_PAGE").ok().and_then(|v| {
                            Page::ALL
                                .into_iter()
                                .find(|p| format!("{p:?}").eq_ignore_ascii_case(&v))
                        })
                    })
                    .unwrap_or(Page::Buttons),
                detail: start_detail,
                scheme: from_cfg.scheme,
                accent: match cfg.accent {
                    adw::config::Pick::Fixed(a) => a,
                    adw::config::Pick::System => {
                        adw::system::accent_color().unwrap_or(AccentColor::Blue)
                    }
                },
                compact: cfg.density == adw::config::Density::Compact,
                radius_m: std::env::var("GALLERY_RM")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(Metrics::default().radius_m),
                radius_toast: Metrics::default().radius_toast,
                radius_l: std::env::var("GALLERY_RL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(12.0),
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
                editor_empty: w::TextContent::new(),
                editor_ro: w::TextContent::with_text("Read-only text.\nIt cannot be edited."),
                sw: true,
                row_sw: false,
                combo2: None,
                split_choice: "merge",
                picked: None,
                multi: vec!["Banana", "Cherry"],
                date: date_demo::State::default(),
                sections: sections_demo::State::default(),
                range: range_demo::State::default(),
                stepper: stepper_demo::State::default(),
                data: data_demo::State::default(),
                resizable: resizable_demo::State::default(),
                reorder: reorder_demo::State::default(),
                field: {
                    let mut f = field_demo::State::default();
                    if std::env::var_os("GALLERY_FIELD_SUBMIT").is_some() {
                        f.submit();
                    }
                    f
                },
                form: form_demo::State::default(),
                autocomplete: autocomplete_demo::State::default(),
                chips: vec!["Rust", "iced", "GTK"],
                acc_single: vec![0],
                acc_multi: vec![0, 1],
                dialog: None,
                show_content: std::env::var_os("GALLERY_SIDEBAR").is_none(),
                sidebar_mode: match std::env::var("GALLERY_SIDEBAR_MODE").as_deref() {
                    Ok("parent-page") => w::CollapsedSidebar::ParentPage,
                    _ => cfg.mobile_sidebar,
                },
                drawer_open: std::env::var_os("GALLERY_DRAWER").is_some(),
                sidebar_padding: w::SidebarPadding::default(),
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
                carousel_indicator: w::CarouselIndicator::default(),
                toasts: adw::primitives::notifications::Queue::new(),
                toast_seq: 0,
                toast_auto: false,
                narrow: false,
                toast_position: cfg.toast_position,
                toast_target: cfg.toast_target,
                banner: true,
                angle: 0.0,
                shots,
                shot_tag: None,
            },
            boot,
        )
    }

    fn main_settings() -> window::Settings {
        window::Settings {
            size: std::env::var("GALLERY_SIZE")
                .ok()
                .and_then(|v| {
                    let (w, h) = v.split_once('x')?;
                    Some(iced::Size::new(w.parse().ok()?, h.parse().ok()?))
                })
                .unwrap_or(iced::Size::new(1040.0, 760.0)),
            // Client-side or system decorations: the toolkit's `[windows] decorations` setting.
            ..adw::window::settings()
        }
    }

    fn title(&self, id: window::Id) -> String {
        if self.prefs.as_ref().is_some_and(|c| c.is(id)) {
            "Preferences".into()
        } else if self.shell.as_ref().is_some_and(|c| c.is(id)) {
            "AppShell".into()
        } else {
            "iced-libadwaita gallery".into()
        }
    }

    /// Opens the preferences dialog as its own window, or brings it forward if open.
    fn open_prefs(&mut self) -> Task<Message> {
        if let Some(c) = &self.prefs {
            return c.focus();
        }
        // A dialog: while it is open the parent is disabled (see `root`).
        let (child, open) = adw::window::Child::dialog(iced::Size::new(620.0, 640.0), ());
        self.prefs = Some(child);
        open.map(|_| Message::Noop)
    }

    /// Opens the AppShell demo as its own window (a width of its own drives its layout), or
    /// brings it forward if open. `GALLERY_SHELL_SIZE=WxH` sets its size.
    fn open_shell(&mut self) -> Task<Message> {
        if let Some(c) = &self.shell {
            return c.focus();
        }
        let size = std::env::var("GALLERY_SHELL_SIZE")
            .ok()
            .and_then(|v| {
                let (w, h) = v.split_once('x')?;
                Some(iced::Size::new(w.parse().ok()?, h.parse().ok()?))
            })
            .unwrap_or(iced::Size::new(1100.0, 720.0));
        // An ordinary window: it does not disable this one.
        let (shell, boot) = appshell::AppShell::new(size.width);
        let (child, open) = adw::window::Child::window(size, shell);
        self.shell = Some(child);
        Task::batch([open.map(|_| Message::Noop), boot.map(Message::Shell)])
    }

    fn metrics(&self) -> Metrics {
        let base = if self.compact {
            Metrics::compact()
        } else {
            Metrics::default()
        };
        Metrics {
            radius_m: self.radius_m,
            radius_l: self.radius_l,
            radius_toast: self.radius_toast,
            ..base
        }
    }

    fn theme(&self, _window: window::Id) -> Theme {
        self.build_theme()
    }

    fn build_theme(&self) -> Theme {
        let shadows = if self.shadows_on {
            adw::Shadows::default()
                .with_scale(self.shadow_scale)
                .with_intensity(self.shadow_intensity)
        } else {
            adw::Shadows::none()
        };
        let mut t = Theme::new(self.scheme)
            .accent(self.accent)
            .with_metrics(self.metrics())
            .with_shadows(shadows)
            .with_auto_contrast(adw::config::current().auto_contrast)
            .with_transparent_window(Self::csd());
        if self.card_override {
            t = t.with_semantic(|s| {
                s.card.bg = adw::color::hex(0x3d2c5a);
                s.card.fg = iced::Color::WHITE;
            });
        }
        t
    }

    /// Adds a toast to the queue (at most four kept, the oldest dropped first).
    fn push_toast(&mut self, spec: ToastSpec) {
        self.toast_seq += 1;
        self.toasts.push(spec, Instant::now());
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Page(p) => {
                self.page = p;
                self.detail = None;
                self.show_content = true;
                self.drawer_open = false;
            }
            Message::OpenDetail(d) => {
                self.page = d.page();
                self.detail = Some(d);
            }
            Message::CloseDetail => self.detail = None,
            Message::Banner(v) => self.banner = v,
            Message::Sw(v) => self.sw = v,
            Message::RowSw(v) => self.row_sw = v,
            Message::Combo2(s) => self.combo2 = Some(s),
            Message::SplitChoice(c) => self.split_choice = c,
            Message::SplitRun(c) => {
                let title = match c {
                    "squash" => "Squashed and merged",
                    "rebase" => "Rebased and merged",
                    _ => "Merged",
                };
                self.push_toast(ToastSpec {
                    title,
                    action: None,
                    status: Some(Role::Success),
                });
            }
            Message::SidebarMode(m) => {
                self.sidebar_mode = m;
                self.drawer_open = false;
            }
            Message::Drawer(open) => self.drawer_open = open,
            Message::Back => self.show_content = false,
            Message::MenuMode(m) => self.menu_mode = m,
            Message::SidebarPadding(p) => self.sidebar_padding = p,
            Message::Scheme(dark) => self.scheme = if dark { Scheme::Dark } else { Scheme::Light },
            Message::Accent(a) => self.accent = a,
            Message::Compact(c) => self.compact = c,
            Message::RadiusM(r) => self.radius_m = r,
            Message::RadiusL(r) => self.radius_l = r,
            Message::RadiusToast(r) => self.radius_toast = r,
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
            Message::EditorEmpty(a) => self.editor_empty.perform(a),
            Message::Picked(s) => self.picked = Some(s),
            Message::MultiToggle(s) => match self.multi.iter().position(|x| *x == s) {
                Some(i) => {
                    self.multi.remove(i);
                }
                None => self.multi.push(s),
            },
            Message::Date(m) => self.date.update(m),
            Message::Field(m) => self.field.update(m),
            Message::Form(m) => self.form.update(m),
            Message::Autocomplete(m) => {
                return self.autocomplete.update(m).map(Message::Autocomplete);
            }
            Message::Sections(m) => self.sections.update(m),
            Message::Range(m) => self.range.update(m),
            Message::Stepper(m) => self.stepper.update(m),
            Message::Data(m) => return self.data.update(m).map(Message::Data),
            Message::Resizable(m) => return self.resizable.update(m).map(Message::Resizable),
            Message::Reorder(m) => self.reorder.update(m),
            Message::AccSingle(v) => self.acc_single = v,
            Message::AccMulti(v) => self.acc_multi = v,
            Message::ChipRemove(c) => self.chips.retain(|x| *x != c),
            // Preferences is a window of its own; the other dialogs stay in the window.
            Message::OpenShell => return self.open_shell(),
            // Chrome of the shell's window is its own message; everything else goes to the shell.
            Message::Shell(appshell::Message::Window(action)) => {
                if let Some(c) = &self.shell {
                    return action.perform(c.id());
                }
            }
            Message::Shell(m) => {
                if let Some(c) = &mut self.shell {
                    return c.state.update(m).map(Message::Shell);
                }
            }
            Message::ChildAction(id, action) => return action.perform(id),
            Message::Resized(id, width) => {
                if let Some(c) = &mut self.shell
                    && c.is(id)
                {
                    // The frame follows the window: edge to edge when maximized.
                    return Task::batch([
                        c.state
                            .update(appshell::Message::Resized(width))
                            .map(Message::Shell),
                        window::is_maximized(id)
                            .map(|m| Message::Shell(appshell::Message::Maximized(m))),
                    ]);
                }
            }
            Message::OpenDialog(Dialog::Preferences) => return self.open_prefs(),
            Message::OpenDialog(d) => self.dialog = Some(d),
            Message::CloseDialog => self.dialog = None,
            Message::Sheet(o) => self.sheet = o,
            Message::Carousel(i) => self.carousel = i,
            Message::CarouselIndicator(i) => self.carousel_indicator = i,
            Message::ShowToast(s) => self.push_toast(ToastSpec::plain(s)),
            Message::ShowToastSpec(t) => self.push_toast(t),
            Message::ToastPosition(p) => self.toast_position = p,
            Message::ToastTarget(t) => self.toast_target = t,
            // The action runs and the toast is replaced by a plain confirmation.
            Message::ToastAction(id) => {
                self.toasts
                    .replace(id, ToastSpec::plain("File restored"), Instant::now());
            }
            Message::DismissToast(id) => {
                self.toasts.dismiss(id);
            }
            Message::ToastAuto(on) => {
                self.toast_auto = on;
                self.toasts
                    .set_timeout(on.then_some(Duration::from_secs(4)));
            }
            Message::ToastTick(now) => {
                self.toasts.expire(now);
            }
            Message::ClearToasts => self.toasts.clear(),
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
                let to = if self.demo_tween.target() > 0.5 {
                    0.0
                } else {
                    1.0
                };
                self.demo_tween.set_target(to, now, self.demo.spec());
            }
            Message::Noop => {}
            Message::Drag => return window::drag(self.main),
            Message::ClosePrefs => {
                if let Some(c) = &self.prefs {
                    return c.close();
                }
            }
            Message::FocusPrefs => {
                if let Some(c) = &self.prefs {
                    return c.focus();
                }
            }
            Message::WindowClosed(id) => {
                if id == self.main {
                    return iced::exit();
                }
                if self.prefs.as_ref().is_some_and(|c| c.is(id)) {
                    self.prefs = None;
                }
                if self.shell.as_ref().is_some_and(|c| c.is(id)) {
                    self.shell = None;
                }
            }
            Message::Escape => {
                if self.dialog.is_some() {
                    self.dialog = None;
                } else if self.sheet {
                    self.sheet = false;
                } else if self.drawer_open {
                    self.drawer_open = false;
                } else {
                    // Leaves the search bar (the on-select trigger demo) back to the path bar.
                    self.sections.leave_search();
                }
            }
            Message::Narrow(n) => self.narrow = n,
            Message::Close => return iced::exit(),
            Message::ToggleFullscreen => {
                return window::oldest().and_then(|id| {
                    window::mode(id).then(move |m| {
                        window::set_mode(
                            id,
                            if m == window::Mode::Fullscreen {
                                window::Mode::Windowed
                            } else {
                                window::Mode::Fullscreen
                            },
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
            Message::Resize(d) => {
                return window::oldest().and_then(move |id| window::drag_resize(id, d));
            }
            Message::Shot(i) => return self.shot_next(i),
            Message::Capture(i) => {
                return window::screenshot(self.shot_window.unwrap_or(self.main))
                    .map(move |s| Message::Saved(i, s));
            }
            Message::Saved(i, shot) => {
                if let Some(s) = &self.shots {
                    let name = format!(
                        "{}-{}{}{}.png",
                        if self.scheme.is_dark() {
                            "dark"
                        } else {
                            "light"
                        },
                        format!("{:?}", self.page).to_lowercase(),
                        self.detail
                            .map(|d| format!("-{d:?}").to_lowercase())
                            .unwrap_or_default(),
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
                Task::perform(tokio::time::sleep(Duration::from_millis(ms)), move |_| {
                    Message::Capture(i)
                })
            };
            // GALLERY_ANIM=prefs: open the preferences window, capture the disabled parent, then
            // the preferences window itself.
            // GALLERY_ANIM=appshell: open the AppShell window (GALLERY_SHELL_SIZE=WxH) and capture
            // it.
            if kind == "appshell" {
                return match i {
                    0 => {
                        self.scheme = if std::env::var_os("GALLERY_DARK").is_some() {
                            Scheme::Dark
                        } else {
                            Scheme::Light
                        };
                        let open = self.open_shell();
                        self.shot_tag = Some("parent");
                        open.chain(Task::perform(
                            tokio::time::sleep(Duration::from_millis(1500)),
                            move |_| Message::Capture(i),
                        ))
                    }
                    1 => {
                        self.shot_window = self.shell.as_ref().map(|c| c.id());
                        wait(300, "appshell", self)
                    }
                    _ => iced::exit(),
                };
            }
            // GALLERY_ANIM=shell-back: the AppShell window with a page open (use APPSHELL_PIN=mobile
            // APPSHELL_SIDEBAR_MODE=parent-page APPSHELL_CONTENT=1 GALLERY_SHELL_SIZE=420x640), then
            // Back to the parent page, captured while the page slides out.
            if kind == "shell-back" {
                return match i {
                    0 => {
                        self.scheme = if std::env::var_os("GALLERY_DARK").is_some() {
                            Scheme::Dark
                        } else {
                            Scheme::Light
                        };
                        let open = self.open_shell();
                        self.shot_tag = Some("a0-child");
                        open.chain(Task::perform(
                            tokio::time::sleep(Duration::from_millis(1500)),
                            move |_| Message::Capture(i),
                        ))
                    }
                    1 => {
                        self.shot_window = self.shell.as_ref().map(|c| c.id());
                        let back = self.update(Message::Shell(appshell::Message::Back));
                        Task::batch([back, wait(70, "a1-70ms", self)])
                    }
                    2 => wait(90, "a2-160ms", self),
                    3 => wait(120, "a3-280ms", self),
                    4 => wait(700, "a4-end", self),
                    _ => iced::exit(),
                };
            }
            // GALLERY_ANIM=embedded-back: the shell embedded in Layout > Resizable, with a page open
            // (APPSHELL_PIN=mobile APPSHELL_SIDEBAR_MODE=parent-page APPSHELL_CONTENT=1
            // GALLERY_PAGE=layout GALLERY_ONLY=resizable), then Back, captured mid-transition.
            if kind == "embedded-back" {
                return match i {
                    0 => {
                        self.scheme = if std::env::var_os("GALLERY_DARK").is_some() {
                            Scheme::Dark
                        } else {
                            Scheme::Light
                        };
                        wait(900, "a0-child", self)
                    }
                    1 => {
                        let back = self.update(Message::Resizable(resizable_demo::Msg::Shell(
                            appshell::Message::Back,
                        )));
                        Task::batch([back, wait(70, "a1-70ms", self)])
                    }
                    2 => wait(90, "a2-160ms", self),
                    3 => wait(120, "a3-280ms", self),
                    4 => wait(700, "a4-end", self),
                    _ => iced::exit(),
                };
            }
            if kind == "prefs" {
                return match i {
                    0 => {
                        self.scheme = Scheme::Light;
                        let open = self.open_prefs();
                        self.shot_tag = Some("parent");
                        open.chain(Task::perform(
                            tokio::time::sleep(Duration::from_millis(1500)),
                            move |_| Message::Capture(i),
                        ))
                    }
                    1 => {
                        self.shot_window = self.prefs.as_ref().map(|c| c.id());
                        wait(300, "prefs", self)
                    }
                    _ => iced::exit(),
                };
            }
            return match i {
                0 => {
                    self.scheme = if std::env::var_os("GALLERY_DARK").is_some() {
                        Scheme::Dark
                    } else {
                        Scheme::Light
                    };
                    wait(700, "a0-start", self)
                }
                // GALLERY_ANIM=keys: no trigger; send keys from outside during the wait.
                1 if kind == "keys" => wait(6000, "keys", self),
                1 => {
                    let trigger = match kind.as_str() {
                        "switch" => Message::Wifi(!self.wifi),
                        "drawer" => Message::Drawer(true),
                        "reveal" => Message::Expanded(!self.expanded),
                        "carousel" => Message::Carousel(1),
                        "page" => Message::Page(Page::Theming),
                        "breakpoint" => Message::Narrow(true),
                        "push" => Message::Page(Page::Dialogs),
                        "dialog" => Message::OpenDialog(Dialog::Alert),
                        "toast-action" => Message::ShowToastSpec(ToastSpec {
                            title: "File deleted",
                            action: Some("Undo"),
                            status: Some(Role::Error),
                        }),
                        _ => Message::ShowToast("Settings saved"),
                    };
                    Task::done(trigger).chain(wait(70, "a1-70ms", self))
                }
                2 => wait(90, "a2-160ms", self),
                3 => wait(700, "a3-end", self),
                _ => iced::exit(),
            };
        }
        // Every group page, then every component page (light), then the groups again (dark).
        let views: Vec<(Page, Option<Detail>)> = Page::ALL
            .iter()
            .map(|p| (*p, None))
            .chain(Detail::all().into_iter().map(|d| (d.page(), Some(d))))
            .collect();
        let total = views.len() + Page::ALL.len();
        if i > total {
            return iced::exit();
        }
        if i == total {
            // Drive the real update path with the radius slider messages.
            self.scheme = Scheme::Light;
            self.page = Page::Theming;
            self.detail = Some(Detail::ThemeShape);
            self.dialog = None;
            self.toasts.clear();
            self.shot_tag = Some("radius0");
            return Task::batch([
                Task::done(Message::RadiusM(0.0)),
                Task::done(Message::RadiusL(0.0)),
                Task::done(Message::RadiusToast(0.0)),
            ])
            .chain(Task::perform(
                tokio::time::sleep(Duration::from_millis(500)),
                move |_| Message::Capture(i),
            ));
        }
        if i < views.len() {
            self.scheme = Scheme::Light;
            (self.page, self.detail) = views[i];
        } else {
            self.scheme = Scheme::Dark;
            self.page = Page::ALL[i - views.len()];
            self.detail = None;
        }
        // The Dialogs group page also shows an open dialog and a toast, so overlays are covered.
        let overlays = self.page == Page::Dialogs && self.detail.is_none();
        self.dialog = overlays.then(|| match std::env::var("GALLERY_DIALOG").as_deref() {
            Ok("preferences") => Dialog::Preferences,
            Ok("shortcuts") => Dialog::Shortcuts,
            Ok("about") => Dialog::About,
            _ => Dialog::Alert,
        });
        self.toasts.clear();
        if overlays {
            self.push_toast(ToastSpec {
                title: "Changes saved",
                action: None,
                status: Some(Role::Success),
            });
            self.push_toast(ToastSpec {
                title: "File deleted",
                action: Some("Undo"),
                status: Some(Role::Error),
            });
        }
        if let Some(s) = &mut self.shots {
            s.index = i;
        }
        Task::perform(tokio::time::sleep(Duration::from_millis(900)), move |_| {
            Message::Capture(i)
        })
    }

    fn subscription(&self) -> Subscription<Message> {
        let frames = if self.page == Page::Feedback
            || self.shots.is_some()
            || self.demo_tween.is_animating()
        {
            window::frames().map(Message::Frame)
        } else {
            Subscription::none()
        };
        // Tick only while a toast can expire.
        let toast_clock = if self.toasts.next_expiry().is_some() {
            iced::time::every(Duration::from_millis(250)).map(Message::ToastTick)
        } else {
            Subscription::none()
        };
        Subscription::batch([
            frames,
            toast_clock,
            self.autocomplete.subscription().map(Message::Autocomplete),
            self.resizable.subscription().map(Message::Resizable),
            adw::window::changes().map(|_| Message::WindowChanged),
            window::close_events().map(Message::WindowClosed),
            window::resize_events().map(|(id, size)| Message::Resized(id, size.width)),
            self.shell.as_ref().map_or_else(Subscription::none, |c| {
                c.state.subscription().map(Message::Shell)
            }),
        ])
    }

    fn view(&self, id: window::Id) -> IcedElement<'_, Message, Theme> {
        if self.prefs.as_ref().is_some_and(|c| c.is(id)) {
            return self.prefs_window(id);
        }
        if let Some(c) = &self.shell
            && c.is(id)
        {
            return c.state.view().map(Message::Shell);
        }
        iced::widget::responsive(|size| self.root(size)).into()
    }

    /// Whether the window draws its own decorations: the toolkit's setting, nothing of ours.
    fn csd() -> bool {
        adw::window::csd()
    }

    fn root(&self, size: iced::Size) -> IcedElement<'_, Message, Theme> {
        let m = self.metrics();
        let csd = Self::csd();
        // GALLERY_ANIM=breakpoint pretends the window just got narrower (the compositor ignores
        // programmatic resizes), to capture the sidebar sliding out.
        let width = if self.narrow {
            size.width.min(600.0)
        } else {
            size.width
        };
        let collapsed = w::split_collapsed(width);

        // Floating windows have rounded corners; maximized/fullscreen go edge to edge.
        // Mobile mode (narrow): no window decorations at all. Corners still follow the
        // real window state: rounded while floating, edge to edge only when the window
        // is actually maximized or fullscreen.
        let mobile = adw::window::is_mobile(width);
        let win_state = self.win;
        let corner = if csd {
            win_state.radius(&adw::Metrics::default())
        } else {
            0.0
        };

        // Primary menu (HIG): open-menu icon, above the sidebar list, standard items last.
        let presentation = match self.menu_mode {
            MenuMode::Auto => w::MenuPresentation::resolve(width),
            MenuMode::Dropdown => w::MenuPresentation::Dropdown,
            MenuMode::FullPage => w::MenuPresentation::FullPage,
        };
        let dark = self.scheme.is_dark();
        let primary = w::primary_menu(
            w::Menu::new("Main Menu")
                .section([w::MenuItem::check(
                    "Dark Style",
                    dark,
                    Message::Scheme(!dark),
                )])
                .section([
                    w::MenuItem::action("Preferences", Message::OpenDialog(Dialog::Preferences))
                        .accel("Ctrl+,"),
                    w::MenuItem::action(
                        "Keyboard Shortcuts",
                        Message::OpenDialog(Dialog::Shortcuts),
                    )
                    .accel("Ctrl+?"),
                    w::MenuItem::action("Help", Message::ShowToast("Opening Help…")).accel("F1"),
                    w::MenuItem::action(
                        "About iced-libadwaita",
                        Message::OpenDialog(Dialog::About),
                    ),
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
            .title(self.detail.map_or(self.page.title(), Detail::title))
            .window_corners(
                if show_content_only || drawer_mode {
                    corner
                } else {
                    0.0
                },
                corner,
            )
            .metrics(m);
        if self.path_bar_title {
            // Replaces the title; the window buttons are independent of it.
            content_bar = content_bar.title_widget(w::path_bar(
                vec!["Home".into(), "Projects".into(), "iced-libadwaita".into()],
                2,
                |_| Message::Noop,
            ));
        }
        if self.detail.is_some() {
            // Inside a component page: back to the group's list.
            content_bar = content_bar
                .pack_start(w::circular_button(Icon::GoPrevious).on_press(Message::CloseDetail));
        } else if drawer_mode {
            // The drawer toggle: the sidebar lives behind it.
            content_bar = content_bar
                .pack_start(w::circular_button(Icon::Sidebar).on_press(Message::Drawer(true)));
        } else if show_content_only {
            content_bar = content_bar
                .pack_start(w::circular_button(Icon::GoPrevious).on_press(Message::Back));
        }
        if csd {
            sidebar_bar = sidebar_bar
                .on_drag(Message::Drag)
                .on_double_click(Message::ToggleMaximize);
            content_bar = content_bar
                .on_drag(Message::Drag)
                .on_double_click(Message::ToggleMaximize);
            // Controls only outside mobile mode. In a split view each pane shows only the
            // buttons at its own window edge; a collapsed single pane shows both sides.
            if !mobile {
                if show_sidebar_only {
                    sidebar_bar = sidebar_bar.window_controls(controls());
                } else if show_content_only {
                    content_bar = content_bar.window_controls(controls());
                } else {
                    sidebar_bar = sidebar_bar
                        .window_controls(controls())
                        .show_end_title_buttons(false);
                    content_bar = content_bar
                        .window_controls(controls())
                        .show_start_title_buttons(false);
                }
            }
        }

        // Collapsed as the parent page, the sidebar is a page of cards with descriptions;
        // otherwise (beside the content, or in the drawer) the compact list.
        let sidebar_style = if collapsed && self.sidebar_mode == w::CollapsedSidebar::ParentPage {
            w::SidebarStyle::Cards
        } else {
            w::SidebarStyle::List
        };
        let sidebar_list = w::SidebarList::new(Page::ALL.iter().map(|p| {
            w::SidebarItem::new(p.title(), Message::Page(*p))
                .icon(p.icon())
                .subtitle(p.description())
                .selected(*p == self.page)
        }))
        .style(sidebar_style)
        .padding(self.sidebar_padding)
        .window_radius(corner);
        let sidebar_pane = w::toolbar_view(Element::from(sidebar_list)).top_bar(sidebar_bar);

        let page: Element = self.content_page();

        // Switching pages fades the new one in (as a parent page, the push slide does that job).
        let page = w::page_transition(
            page,
            if collapsed && self.sidebar_mode == w::CollapsedSidebar::ParentPage {
                0
            } else {
                self.view_key()
            },
        )
        .scroll_key(self.view_key());
        // The toast is placed in the whole window or only the content area (the shared
        // `[toasts] target` setting). Both layers stay mounted so the tree does not change.
        let toast_for = |id, spec: &ToastSpec| {
            let mut t = w::Toast::new(spec.title)
                .on_dismiss(Message::DismissToast(id))
                .position(self.toast_position);
            if let Some(label) = spec.action {
                t = t.action(label, Message::ToastAction(id));
            }
            if let Some(status) = spec.status {
                t = t.status(status);
            }
            t
        };
        let in_content = self.toast_target == w::ToastTarget::Content;
        let content_pane: Element = if in_content {
            w::toast_stack_from(
                Element::from(w::toolbar_view(page).top_bar(content_bar)),
                &self.toasts,
                toast_for,
            )
        } else {
            w::toast_stack(
                Element::from(w::toolbar_view(page).top_bar(content_bar)),
                Vec::new(),
            )
        };
        let body = w::NavigationSplitView::new(sidebar_pane, content_pane)
            .adapt(width)
            .window_radius(corner)
            .collapsed_sidebar(self.sidebar_mode)
            .show_sidebar(self.drawer_open)
            .on_close_drawer(Message::Drawer(false))
            .show_content(self.show_content);

        // The window background comes from `adw::window::frame` (rounded when floating).
        let body = container(body).width(Length::Fill).height(Length::Fill);
        let base: Element = if in_content {
            w::toast_stack(body, Vec::new())
        } else {
            w::toast_stack_from(body, &self.toasts, toast_for)
        };

        let base: Element = w::bottom_sheet_in_window(
            base,
            column![
                ty::title_3("Sheet"),
                ty::body("A bottom sheet over the content."),
                w::suggested_button(text("Done")).on_press(Message::Sheet(false))
            ]
            .spacing(m.space_m),
            self.sheet,
            Message::Sheet(false),
            corner,
        );
        let dialog: Option<Element<'_>> = match self.dialog {
            None => None,
            Some(Dialog::Alert) => Some(Element::from(
                w::AlertDialog::new("Discard Changes?")
                    .body("Unsaved changes will be permanently lost.")
                    .response(
                        "Cancel",
                        w::ResponseAppearance::Default,
                        Message::CloseDialog,
                    )
                    .response(
                        "Save",
                        w::ResponseAppearance::Suggested,
                        Message::CloseDialog,
                    )
                    .response(
                        "Discard",
                        w::ResponseAppearance::Destructive,
                        Message::CloseDialog,
                    ),
            )),
            Some(Dialog::Shortcuts) => Some(w::shortcuts_dialog(
                vec![
                    (
                        "General".into(),
                        vec![
                            ("Preferences".into(), "Ctrl+,".into()),
                            ("Keyboard Shortcuts".into(), "Ctrl+?".into()),
                            ("Help".into(), "F1".into()),
                            ("Quit".into(), "Ctrl+Q".into()),
                        ],
                    ),
                    (
                        "Window".into(),
                        vec![
                            ("Fullscreen".into(), "F11".into()),
                            ("Close dialog".into(), "Esc".into()),
                        ],
                    ),
                ],
                Message::CloseDialog,
            )),
            // The preferences dialog is a window of its own (see `prefs_window`).
            Some(Dialog::Preferences) => None,
            Some(Dialog::About) => Some(Element::from(
                w::AboutDialog::new("Gallery")
                    .developer_name("The iced-libadwaita authors")
                    .version("0.1.0")
                    .comments("Widget gallery for iced-libadwaita.")
                    .website("https://iced.rs")
                    .license("MIT OR Apache-2.0")
                    .application_icon(
                        Icon::Home.colored(48.0, self.build_theme().semantic.accent.color),
                    )
                    .on_close(Message::CloseDialog),
            )),
        };
        let result = w::present_dialog(base, dialog, Some(Message::CloseDialog), corner);
        // The toolkit listens: no subscription or update arm. The primary menu registers its own
        // accelerators (Ctrl+, Ctrl+? F1); these are the standard ones the app supports.
        let result = adw::shortcut::Standard::new()
            .quit(Message::Close)
            .fullscreen(Message::ToggleFullscreen)
            .into_shortcuts()
            .bind("Esc", Message::Escape)
            .scope(result);
        // While the preferences window (a modal child) is open the parent is disabled: a scrim
        // that swallows input, and a click on it brings the preferences window forward.
        let result = adw::window::Child::disable_parent(
            self.prefs.as_ref(),
            result,
            Message::FocusPrefs,
            corner,
        );
        if csd {
            adw::window::frame(result, win_state, Message::Resize)
        } else {
            result
        }
    }

    /// The preferences dialog in its own window: a header with a close button, the preference
    /// page, rounded corners and resize grips like the main window.
    fn prefs_window(&self, id: window::Id) -> IcedElement<'_, Message, Theme> {
        let m = self.metrics();
        // A dialog: close button only, both side edges, like any window of the toolkit.
        let chrome = adw::window::Chrome::new(adw::window::State::default()).close_only();
        let header = w::header_bar::<Message>()
            .title("Preferences")
            .chrome(&chrome, move |a| Message::ChildAction(id, a))
            .metrics(m);
        // Esc closes it, like any dialog.
        let content = adw::shortcut::Shortcuts::new()
            .bind("Esc", Message::ClosePrefs)
            .scope(Element::from(
                w::toolbar_view(self.preferences_demo()).top_bar(header),
            ));
        chrome.frame(content, move |a| Message::ChildAction(id, a))
    }

    // --- pages ----------------------------------------------------------

    fn page_frame<'a>(&self, content: impl Into<Element<'a>>) -> Element<'a> {
        let m = self.metrics();
        w::page(content)
            .maximum_size(m.clamp_maximum + 120.0)
            .metrics(m)
            .into()
    }

    fn preferences_demo(&self) -> Element<'_> {
        let m = self.metrics();
        let mut page = w::PreferencesPage::<Message>::new().metrics(m);

        page = page.push(
            w::PreferencesGroup::new()
                .title("Appearance")
                .description("Rows, switches and choosers.")
                .metrics(m)
                .push(
                    w::SwitchRow::new("Dark Style", self.scheme.is_dark(), Message::Scheme)
                        .subtitle("Switches the whole gallery")
                        .metrics(m),
                )
                .push(w::SwitchRow::new("Wi-Fi", self.wifi, Message::Wifi).metrics(m))
                .push(
                    w::ComboRow::new("Font", &COMBO, self.combo, Message::Combo)
                        .subtitle("Interface font")
                        // Shows the disabled state: dimmed and unselectable (selected is accent).
                        .disabled_when(|o| *o == "Inter")
                        .open_initially(false),
                )
                .push(w::SpinRow::new(
                    "Text Scale",
                    self.spin,
                    1.0..=10.0,
                    1.0,
                    Message::Spin,
                )),
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
                .push(
                    w::EntryRow::new("Display name", &self.name)
                        .on_input(Message::Name)
                        .metrics(m),
                )
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
                        .push(
                            w::SwitchRow::new("Sounds", self.notifications, Message::Notifications)
                                .metrics(m),
                        )
                        .push(
                            w::ActionRow::new("Do Not Disturb")
                                .subtitle("Until tomorrow")
                                .metrics(m),
                        ),
                )
                .push(
                    w::ButtonRow::new("Sign Out")
                        .role(Role::Destructive)
                        .on_activate(Message::OpenDialog(Dialog::Alert)),
                ),
        );
        page.into()
    }
}

fn main() -> iced::Result {
    // Shared desktop behavior: ~/.config/iced-libadwaita/desktop.conf (+ system files).
    let mut cfg = adw::config::Config::load();
    if std::env::var_os("GALLERY_OPT_OUT").is_some() {
        // Opting out of desktop settings is highly discouraged; shown here only to demonstrate
        // that it is allowed, explicit and reported (a notice is printed on install).
        cfg = cfg.opt_out(|o| {
            o.mobile_sidebar(w::CollapsedSidebar::ParentPage);
        });
    }
    adw::config::install(cfg);
    // A daemon, so the preferences dialog can be a window of its own.
    iced::daemon(Gallery::new, Gallery::update, Gallery::view)
        .title(Gallery::title)
        .theme(Gallery::theme)
        .subscription(Gallery::subscription)
        .settings(iced::Settings {
            default_text_size: 14.67.into(),
            antialiasing: true,
            ..Default::default()
        })
        .run()
}
