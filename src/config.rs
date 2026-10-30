//! The shared desktop configuration.
//!
//! Desktop-level behavior (color scheme, accent, density, shadows, animations, window
//! decorations and buttons, the mobile breakpoint, menu presentation, the mobile sidebar,
//! row wrapping, the icon theme) is defined **once per desktop**, not per app, so every
//! iced-libadwaita app behaves the same way. It lives in a GLib key-file:
//!
//! 1. built-in defaults (libadwaita's own behavior),
//! 2. system files: `$XDG_CONFIG_DIRS/iced-libadwaita/desktop.conf` (default `/etc/xdg`),
//! 3. the user file: `$XDG_CONFIG_HOME/iced-libadwaita/desktop.conf`,
//! 4. the app's **opt-outs** (see below), if the desktop allows them.
//!
//! `ICED_LIBADWAITA_CONFIG=/path/file.conf` replaces 2 and 3 with a single file (testing).
//!
//! # Opting out
//!
//! An app may override a desktop setting to provide custom or fixed behavior. This is
//! **highly discouraged**: it makes the app behave differently from every other app on the
//! desktop, ignoring what the user configured. It is allowed, deliberate and reported:
//!
//! ```
//! # use iced_libadwaita::config::{Config, Setting};
//! let cfg = Config::default().opt_out(|o| {
//!     o.animations(false); // fixed behavior for this app; the user's setting is ignored
//! });
//! assert!(cfg.overridden().contains(&Setting::Animations));
//! ```
//!
//! A desktop can forbid opt-outs with `[policy] allow-app-overrides=false`; they are then
//! ignored and listed in [`Config::denied`].
//!
//! # Using it
//!
//! Call [`install`] at startup. The library then reads the installed config wherever it makes
//! a desktop-level decision, and [`Theme::from_config`](crate::Theme::from_config) builds the
//! theme from it.

use crate::palette::AccentColor;
use crate::widget::CollapsedSidebar;
use crate::{CollapseBehavior, Metrics, Scheme};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// The settings an app can opt out of (and that [`Config::overridden`] reports).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Setting {
    ColorScheme,
    Accent,
    FlatHeaderbars,
    Density,
    TextScale,
    Shadows,
    Animations,
    ReducedMotion,
    Motion,
    Decorations,
    ButtonLayout,
    MobileMaxWidth,
    CollapseWidth,
    MobileSidebar,
    MenuPresentation,
    RowCollapse,
    RowWrapBasis,
    IconTheme,
}

/// `system` defers to the desktop (GSettings); otherwise a fixed value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pick<T> {
    #[default]
    System,
    Fixed(T),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    #[default]
    Default,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DecorationMode {
    /// Client-side decorations: the app draws its header bar and controls (Adwaita).
    #[default]
    Csd,
    /// Server-side decorations (discouraged by Adwaita).
    Ssd,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MenuPresentationSetting {
    /// Full page on mobile widths, else dropdown.
    #[default]
    Auto,
    Dropdown,
    FullPage,
}

/// The resolved configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub color_scheme: Pick<Scheme>,
    pub accent: Pick<AccentColor>,
    pub flat_headerbars: bool,
    pub density: Density,
    pub text_scale: f32,
    pub shadows: bool,
    pub shadow_scale: f32,
    pub shadow_intensity: f32,
    /// `system` follows `enable-animations`; `Fixed(bool)` forces it.
    pub animations: Pick<bool>,
    /// `true`: every trigger resolves to [`Motion::None`] (prefers-reduced-motion).
    pub reduced_motion: bool,
    /// Per-trigger overrides from `[motion] <trigger>=default|none`.
    pub motion: std::collections::HashMap<crate::animation::Trigger, crate::animation::Motion>,
    pub decorations: DecorationMode,
    /// `system` follows GNOME's `button-layout`; `Fixed(string)` is a GTK layout string.
    pub button_layout: Pick<String>,
    pub mobile_max_width: f32,
    pub collapse_width: f32,
    pub mobile_sidebar: CollapsedSidebar,
    pub menu_presentation: MenuPresentationSetting,
    pub row_collapse: CollapseBehavior,
    pub row_wrap_basis: f32,
    /// `system` follows `icon-theme`.
    pub icon_theme: Pick<String>,
    /// May apps opt out? (`[policy] allow-app-overrides`, default yes.)
    pub allow_app_overrides: bool,

    overridden: Vec<Setting>,
    denied: Vec<Setting>,
    warnings: Vec<String>,
    metrics: Metrics,
}

impl Default for Config {
    /// libadwaita's own behavior: follow the system, client-side decorations, drawer,
    /// stacked rows, 450px mobile breakpoint.
    fn default() -> Self {
        let mut c = Self {
            color_scheme: Pick::System,
            accent: Pick::System,
            flat_headerbars: true,
            density: Density::Default,
            text_scale: 1.0,
            shadows: true,
            shadow_scale: 1.0,
            shadow_intensity: 1.0,
            animations: Pick::System,
            reduced_motion: false,
            motion: Default::default(),
            decorations: DecorationMode::Csd,
            button_layout: Pick::System,
            mobile_max_width: 450.0,
            collapse_width: 400.0,
            mobile_sidebar: CollapsedSidebar::Drawer,
            menu_presentation: MenuPresentationSetting::Auto,
            row_collapse: CollapseBehavior::Stacked,
            row_wrap_basis: 128.0,
            icon_theme: Pick::System,
            allow_app_overrides: true,
            overridden: Vec::new(),
            denied: Vec::new(),
            warnings: Vec::new(),
            metrics: Metrics::default(),
        };
        c.metrics = c.compute_metrics();
        c
    }
}

// ---------------------------------------------------------------- parsing

/// Parses GLib key-file text into `(section, key) -> value`. Lines starting with `#` or `;`
/// are comments; unknown syntax is skipped.
pub fn parse_keyfile(text: &str) -> BTreeMap<(String, String), String> {
    let mut map = BTreeMap::new();
    let mut section = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = name.trim().to_string();
        } else if let Some((k, v)) = line.split_once('=') {
            let v = v.trim().trim_matches('"').trim_matches('\'');
            map.insert((section.clone(), k.trim().to_string()), v.to_string());
        }
    }
    map
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Some(true),
        "false" | "no" | "off" | "0" => Some(false),
        _ => None,
    }
}

fn parse_accent(v: &str) -> Option<AccentColor> {
    AccentColor::ALL.into_iter().find(|a| format!("{a:?}").eq_ignore_ascii_case(v))
}

impl Config {
    /// Builds a config from key-file text on top of the defaults.
    pub fn from_keyfile(text: &str) -> Self {
        let mut c = Self::default();
        c.apply(&parse_keyfile(text));
        c
    }

    fn apply(&mut self, m: &BTreeMap<(String, String), String>) {
        let get = |s: &str, k: &str| m.get(&(s.to_string(), k.to_string())).map(String::as_str);
        let mut warn = Vec::new();
        macro_rules! bad {
            ($s:expr, $k:expr, $v:expr) => {
                warn.push(format!("[{}] {}: invalid value {:?}", $s, $k, $v))
            };
        }

        if let Some(v) = get("appearance", "color-scheme") {
            match v {
                "system" => self.color_scheme = Pick::System,
                "light" => self.color_scheme = Pick::Fixed(Scheme::Light),
                "dark" => self.color_scheme = Pick::Fixed(Scheme::Dark),
                _ => bad!("appearance", "color-scheme", v),
            }
        }
        if let Some(v) = get("appearance", "accent") {
            match (v, parse_accent(v)) {
                ("system", _) => self.accent = Pick::System,
                (_, Some(a)) => self.accent = Pick::Fixed(a),
                _ => bad!("appearance", "accent", v),
            }
        }
        if let Some(v) = get("appearance", "flat-headerbars") {
            parse_bool(v).map(|b| self.flat_headerbars = b).unwrap_or_else(|| bad!("appearance", "flat-headerbars", v));
        }
        if let Some(v) = get("appearance", "density") {
            match v {
                "default" => self.density = Density::Default,
                "compact" => self.density = Density::Compact,
                _ => bad!("appearance", "density", v),
            }
        }
        if let Some(v) = get("appearance", "text-scale") {
            match v.parse::<f32>() {
                Ok(f) if (0.5..=3.0).contains(&f) => self.text_scale = f,
                _ => bad!("appearance", "text-scale", v),
            }
        }
        if let Some(v) = get("appearance", "shadows") {
            parse_bool(v).map(|b| self.shadows = b).unwrap_or_else(|| bad!("appearance", "shadows", v));
        }
        if let Some(v) = get("appearance", "shadow-scale") {
            v.parse::<f32>().ok().filter(|f| *f >= 0.0).map(|f| self.shadow_scale = f).unwrap_or_else(|| bad!("appearance", "shadow-scale", v));
        }
        if let Some(v) = get("appearance", "shadow-intensity") {
            v.parse::<f32>().ok().filter(|f| *f >= 0.0).map(|f| self.shadow_intensity = f).unwrap_or_else(|| bad!("appearance", "shadow-intensity", v));
        }
        for t in crate::animation::Trigger::ALL {
            if let Some(v) = get("motion", t.key()) {
                match v {
                    "default" | "animated" => { self.motion.insert(t, crate::animation::Motion::Animated); }
                    "none" => { self.motion.insert(t, crate::animation::Motion::None); }
                    _ => bad!("motion", t.key(), v),
                }
            }
        }
        if let Some(v) = get("motion", "animations") {
            match v {
                "system" => self.animations = Pick::System,
                _ => parse_bool(v).map(|b| self.animations = Pick::Fixed(b)).unwrap_or_else(|| bad!("motion", "animations", v)),
            }
        }
        if let Some(v) = get("motion", "reduced-motion") {
            parse_bool(v).map(|b| self.reduced_motion = b).unwrap_or_else(|| bad!("motion", "reduced-motion", v));
        }
        if let Some(v) = get("windows", "decorations") {
            match v {
                "csd" => self.decorations = DecorationMode::Csd,
                "ssd" => self.decorations = DecorationMode::Ssd,
                _ => bad!("windows", "decorations", v),
            }
        }
        if let Some(v) = get("windows", "button-layout") {
            self.button_layout = if v == "system" { Pick::System } else { Pick::Fixed(v.to_string()) };
        }
        if let Some(v) = get("windows", "mobile-max-width") {
            v.parse::<f32>().ok().filter(|f| *f >= 200.0).map(|f| self.mobile_max_width = f).unwrap_or_else(|| bad!("windows", "mobile-max-width", v));
        }
        if let Some(v) = get("navigation", "collapse-width") {
            v.parse::<f32>().ok().filter(|f| *f >= 200.0).map(|f| self.collapse_width = f).unwrap_or_else(|| bad!("navigation", "collapse-width", v));
        }
        if let Some(v) = get("navigation", "mobile-sidebar") {
            match v {
                "drawer" => self.mobile_sidebar = CollapsedSidebar::Drawer,
                "first-page" => self.mobile_sidebar = CollapsedSidebar::FirstPage,
                _ => bad!("navigation", "mobile-sidebar", v),
            }
        }
        if let Some(v) = get("menus", "presentation") {
            match v {
                "auto" => self.menu_presentation = MenuPresentationSetting::Auto,
                "dropdown" => self.menu_presentation = MenuPresentationSetting::Dropdown,
                "full-page" => self.menu_presentation = MenuPresentationSetting::FullPage,
                _ => bad!("menus", "presentation", v),
            }
        }
        if let Some(v) = get("rows", "collapse") {
            match v {
                "stacked" => self.row_collapse = CollapseBehavior::Stacked,
                "flex" => self.row_collapse = CollapseBehavior::Flex,
                _ => bad!("rows", "collapse", v),
            }
        }
        if let Some(v) = get("rows", "wrap-basis") {
            v.parse::<f32>().ok().filter(|f| *f >= 0.0).map(|f| self.row_wrap_basis = f).unwrap_or_else(|| bad!("rows", "wrap-basis", v));
        }
        if let Some(v) = get("icons", "theme") {
            self.icon_theme = if v == "system" { Pick::System } else { Pick::Fixed(v.to_string()) };
        }
        if let Some(v) = get("policy", "allow-app-overrides") {
            parse_bool(v).map(|b| self.allow_app_overrides = b).unwrap_or_else(|| bad!("policy", "allow-app-overrides", v));
        }
        // Unknown keys are reported, not fatal.
        const KNOWN: &[(&str, &str)] = &[
            ("appearance", "color-scheme"), ("appearance", "accent"), ("appearance", "flat-headerbars"),
            ("appearance", "density"), ("appearance", "text-scale"), ("appearance", "shadows"),
            ("appearance", "shadow-scale"), ("appearance", "shadow-intensity"), ("motion", "animations"),
            ("motion", "reduced-motion"), ("windows", "decorations"), ("windows", "button-layout"),
            ("windows", "mobile-max-width"), ("navigation", "collapse-width"), ("navigation", "mobile-sidebar"),
            ("menus", "presentation"), ("rows", "collapse"), ("rows", "wrap-basis"), ("icons", "theme"),
            ("policy", "allow-app-overrides"),
        ];
        for (s, k) in m.keys() {
            let trigger = s == "motion" && crate::animation::Trigger::from_key(k).is_some();
            if !trigger && !KNOWN.contains(&(s.as_str(), k.as_str())) {
                warn.push(format!("[{s}] {k}: unknown setting"));
            }
        }
        self.warnings.extend(warn);
        self.metrics = self.compute_metrics();
    }

    fn compute_metrics(&self) -> Metrics {
        let base = match self.density {
            Density::Default => Metrics::default(),
            Density::Compact => Metrics::compact(),
        };
        Metrics { row_collapse: self.row_collapse, row_wrap_basis: self.row_wrap_basis, ..base }
    }

    /// The config directories searched, lowest priority first (system, then user).
    pub fn search_paths() -> Vec<PathBuf> {
        if let Some(p) = std::env::var_os("ICED_LIBADWAITA_CONFIG") {
            return vec![PathBuf::from(p)];
        }
        let mut out = Vec::new();
        let dirs = std::env::var("XDG_CONFIG_DIRS").unwrap_or_else(|_| "/etc/xdg".into());
        // XDG_CONFIG_DIRS is ordered most-important first; we apply least-important first.
        for d in dirs.split(':').filter(|s| !s.is_empty()).collect::<Vec<_>>().into_iter().rev() {
            out.push(PathBuf::from(d).join("iced-libadwaita/desktop.conf"));
        }
        let user = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")));
        if let Some(u) = user {
            out.push(u.join("iced-libadwaita/desktop.conf"));
        }
        out
    }

    /// Loads the layered files (missing files are fine) over the defaults.
    pub fn load() -> Self {
        Self::load_from(&Self::search_paths())
    }

    /// Like [`load`](Self::load) with explicit files, lowest priority first.
    pub fn load_from(paths: &[PathBuf]) -> Self {
        let mut merged = BTreeMap::new();
        for p in paths {
            if let Ok(text) = std::fs::read_to_string(p) {
                merged.extend(parse_keyfile(&text));
            }
        }
        let mut c = Self::default();
        c.apply(&merged);
        c
    }

    // ------------------------------------------------------------ opt-out

    /// Overrides desktop settings for this app. **Highly discouraged**: the app then ignores
    /// what the user configured and behaves differently from other apps. Allowed unless the
    /// desktop sets `allow-app-overrides=false`, in which case the overrides are ignored and
    /// reported in [`denied`](Self::denied). Applied ones are in [`overridden`](Self::overridden).
    #[must_use]
    pub fn opt_out(mut self, f: impl FnOnce(&mut Overrides)) -> Self {
        let mut o = Overrides::default();
        f(&mut o);
        if !o.touched.is_empty() {
            if self.allow_app_overrides {
                let touched = o.touched.clone();
                o.apply_to(&mut self);
                self.overridden.extend(touched);
                self.metrics = self.compute_metrics();
            } else {
                self.denied.extend(o.touched);
            }
        }
        self.overridden.sort();
        self.overridden.dedup();
        self
    }

    /// Settings this app has overridden (opted out of).
    pub fn overridden(&self) -> &[Setting] {
        &self.overridden
    }
    /// Opt-outs the desktop's policy rejected.
    pub fn denied(&self) -> &[Setting] {
        &self.denied
    }
    /// Problems found while reading the files (invalid or unknown values).
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// The metrics this config implies (density and row behavior).
    pub fn metrics(&self) -> Metrics {
        self.metrics
    }
}

/// App-side overrides; see [`Config::opt_out`].
#[derive(Default)]
pub struct Overrides {
    touched: Vec<Setting>,
    color_scheme: Option<Scheme>,
    accent: Option<AccentColor>,
    flat_headerbars: Option<bool>,
    density: Option<Density>,
    text_scale: Option<f32>,
    shadows: Option<bool>,
    animations: Option<bool>,
    reduced_motion: Option<bool>,
    decorations: Option<DecorationMode>,
    button_layout: Option<String>,
    mobile_max_width: Option<f32>,
    collapse_width: Option<f32>,
    mobile_sidebar: Option<CollapsedSidebar>,
    menu_presentation: Option<MenuPresentationSetting>,
    row_collapse: Option<CollapseBehavior>,
    row_wrap_basis: Option<f32>,
    icon_theme: Option<String>,
    motion: Vec<(crate::animation::Trigger, crate::animation::Motion)>,
}

macro_rules! override_fn {
    ($name:ident, $field:ident, $ty:ty, $setting:expr) => {
        #[doc = concat!("Overrides `", stringify!($name), "` for this app (discouraged).")]
        pub fn $name(&mut self, v: $ty) -> &mut Self {
            self.$field = Some(v);
            self.touched.push($setting);
            self
        }
    };
}

impl Overrides {
    override_fn!(color_scheme, color_scheme, Scheme, Setting::ColorScheme);
    override_fn!(accent, accent, AccentColor, Setting::Accent);
    override_fn!(flat_headerbars, flat_headerbars, bool, Setting::FlatHeaderbars);
    override_fn!(density, density, Density, Setting::Density);
    override_fn!(text_scale, text_scale, f32, Setting::TextScale);
    override_fn!(shadows, shadows, bool, Setting::Shadows);
    override_fn!(animations, animations, bool, Setting::Animations);
    override_fn!(reduced_motion, reduced_motion, bool, Setting::ReducedMotion);
    override_fn!(decorations, decorations, DecorationMode, Setting::Decorations);
    override_fn!(button_layout, button_layout, String, Setting::ButtonLayout);
    override_fn!(mobile_max_width, mobile_max_width, f32, Setting::MobileMaxWidth);
    override_fn!(collapse_width, collapse_width, f32, Setting::CollapseWidth);
    override_fn!(mobile_sidebar, mobile_sidebar, CollapsedSidebar, Setting::MobileSidebar);
    override_fn!(menu_presentation, menu_presentation, MenuPresentationSetting, Setting::MenuPresentation);
    override_fn!(row_collapse, row_collapse, CollapseBehavior, Setting::RowCollapse);
    override_fn!(row_wrap_basis, row_wrap_basis, f32, Setting::RowWrapBasis);
    override_fn!(icon_theme, icon_theme, String, Setting::IconTheme);

    /// Overrides one trigger's animation for this app (discouraged).
    pub fn motion(&mut self, trigger: crate::animation::Trigger, motion: crate::animation::Motion) -> &mut Self {
        self.motion.push((trigger, motion));
        self.touched.push(Setting::Motion);
        self
    }

    fn apply_to(self, c: &mut Config) {
        for (t, m) in self.motion {
            c.motion.insert(t, m);
        }
        if let Some(v) = self.color_scheme { c.color_scheme = Pick::Fixed(v) }
        if let Some(v) = self.accent { c.accent = Pick::Fixed(v) }
        if let Some(v) = self.flat_headerbars { c.flat_headerbars = v }
        if let Some(v) = self.density { c.density = v }
        if let Some(v) = self.text_scale { c.text_scale = v }
        if let Some(v) = self.shadows { c.shadows = v }
        if let Some(v) = self.animations { c.animations = Pick::Fixed(v) }
        if let Some(v) = self.reduced_motion { c.reduced_motion = v }
        if let Some(v) = self.decorations { c.decorations = v }
        if let Some(v) = self.button_layout { c.button_layout = Pick::Fixed(v) }
        if let Some(v) = self.mobile_max_width { c.mobile_max_width = v }
        if let Some(v) = self.collapse_width { c.collapse_width = v }
        if let Some(v) = self.mobile_sidebar { c.mobile_sidebar = v }
        if let Some(v) = self.menu_presentation { c.menu_presentation = v }
        if let Some(v) = self.row_collapse { c.row_collapse = v }
        if let Some(v) = self.row_wrap_basis { c.row_wrap_basis = v }
        if let Some(v) = self.icon_theme { c.icon_theme = Pick::Fixed(v) }
    }
}

// ----------------------------------------------------------------- global

static CURRENT: RwLock<Option<Arc<Config>>> = RwLock::new(None);

/// Installs `config` as the process-wide desktop configuration. Call once at startup (and
/// again after a reload). Prints a one-line notice to stderr when the app opted out of
/// desktop settings, so opt-outs are never silent.
pub fn install(config: Config) {
    if !config.overridden.is_empty() {
        eprintln!(
            "iced-libadwaita: this app overrides desktop settings {:?} (opt-out; discouraged)",
            config.overridden
        );
    }
    if !config.denied.is_empty() {
        eprintln!(
            "iced-libadwaita: the desktop policy forbids app overrides; ignored {:?}",
            config.denied
        );
    }
    for w in &config.warnings {
        eprintln!("iced-libadwaita: config: {w}");
    }
    *CURRENT.write().unwrap() = Some(Arc::new(config));
}

/// The installed configuration, or libadwaita's defaults when none was installed.
pub fn current() -> Arc<Config> {
    if let Some(c) = CURRENT.read().unwrap().as_ref() {
        return c.clone();
    }
    let c = Arc::new(Config::default());
    *CURRENT.write().unwrap() = Some(c.clone());
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = r#"
# desktop defaults
[appearance]
color-scheme=dark
accent=teal
flat-headerbars=true
density=compact
shadows=false

[motion]
animations=off

[windows]
decorations=ssd
button-layout=close,minimize,maximize:
mobile-max-width=500

[navigation]
mobile-sidebar=first-page

[menus]
presentation=dropdown

[rows]
collapse=flex
wrap-basis=200

[bogus]
thing=1
"#;

    #[test]
    fn defaults_are_libadwaita() {
        let c = Config::default();
        assert_eq!(c.color_scheme, Pick::System);
        assert_eq!(c.decorations, DecorationMode::Csd);
        assert_eq!(c.mobile_sidebar, CollapsedSidebar::Drawer);
        assert_eq!(c.mobile_max_width, 450.0);
        assert!(c.allow_app_overrides && c.overridden().is_empty());
    }

    #[test]
    fn keyfile_parses_every_section() {
        let c = Config::from_keyfile(FILE);
        assert_eq!(c.color_scheme, Pick::Fixed(Scheme::Dark));
        assert_eq!(c.accent, Pick::Fixed(AccentColor::Teal));
        assert!(c.flat_headerbars && !c.shadows);
        assert_eq!(c.density, Density::Compact);
        assert_eq!(c.animations, Pick::Fixed(false));
        assert_eq!(c.decorations, DecorationMode::Ssd);
        assert_eq!(c.button_layout, Pick::Fixed("close,minimize,maximize:".into()));
        assert_eq!(c.mobile_max_width, 500.0);
        assert_eq!(c.mobile_sidebar, CollapsedSidebar::FirstPage);
        assert_eq!(c.menu_presentation, MenuPresentationSetting::Dropdown);
        assert_eq!(c.row_collapse, CollapseBehavior::Flex);
        assert_eq!(c.row_wrap_basis, 200.0);
        assert!(c.warnings().iter().any(|w| w.contains("[bogus] thing")));
    }

    #[test]
    fn invalid_values_warn_and_keep_defaults() {
        let c = Config::from_keyfile("[motion]\nanimations=maybe\n[appearance]\ntext-scale=99\n");
        assert_eq!(c.animations, Pick::System);
        assert_eq!(c.text_scale, 1.0);
        assert_eq!(c.warnings().len(), 2);
    }

    #[test]
    fn density_and_rows_feed_the_metrics() {
        let c = Config::from_keyfile("[appearance]\ndensity=compact\n[rows]\ncollapse=flex\nwrap-basis=90\n");
        let m = c.metrics();
        assert_eq!(m.button_min_height, Metrics::compact().button_min_height);
        assert_eq!(m.row_collapse, CollapseBehavior::Flex);
        assert_eq!(m.row_wrap_basis, 90.0);
    }

    #[test]
    fn later_files_win_per_key() {
        let dir = std::env::temp_dir().join(format!("iced-adw-cfg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sys = dir.join("system.conf");
        let user = dir.join("user.conf");
        std::fs::write(&sys, "[appearance]\ncolor-scheme=dark\nshadows=false\n").unwrap();
        std::fs::write(&user, "[appearance]\ncolor-scheme=light\n").unwrap();
        let c = Config::load_from(&[sys, user, dir.join("missing.conf")]);
        assert_eq!(c.color_scheme, Pick::Fixed(Scheme::Light), "user beats system");
        assert!(!c.shadows, "untouched keys keep the system value");
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn app_opt_out_overrides_and_is_reported() {
        let c = Config::from_keyfile("[motion]\nanimations=on\n[navigation]\nmobile-sidebar=drawer\n")
            .opt_out(|o| {
                o.animations(false).mobile_sidebar(CollapsedSidebar::FirstPage);
            });
        assert_eq!(c.animations, Pick::Fixed(false));
        assert_eq!(c.mobile_sidebar, CollapsedSidebar::FirstPage);
        assert_eq!(c.overridden(), &[Setting::Animations, Setting::MobileSidebar]);
        assert!(c.denied().is_empty());
    }

    #[test]
    fn desktop_policy_can_forbid_opt_outs() {
        let c = Config::from_keyfile("[policy]\nallow-app-overrides=false\n[motion]\nanimations=on\n")
            .opt_out(|o| {
                o.animations(false);
            });
        assert_eq!(c.animations, Pick::Fixed(true), "the desktop value stands");
        assert!(c.overridden().is_empty());
        assert_eq!(c.denied(), &[Setting::Animations]);
    }

    #[test]
    fn each_trigger_can_be_set_to_none() {
        use crate::animation::{Motion, Trigger};
        let c = Config::from_keyfile("[motion]\npage-change=none\npopover=default\ncarousel=wobble\n");
        assert_eq!(c.motion.get(&Trigger::PageChange), Some(&Motion::None));
        assert_eq!(c.motion.get(&Trigger::Popover), Some(&Motion::Animated));
        assert!(c.motion.get(&Trigger::Carousel).is_none());
        assert!(c.warnings().iter().any(|w| w.contains("carousel")));
        assert!(!c.warnings().iter().any(|w| w.contains("unknown")), "trigger keys are known settings");
    }

    #[test]
    fn trigger_keys_round_trip() {
        use crate::animation::Trigger;
        for t in Trigger::ALL {
            assert_eq!(Trigger::from_key(t.key()), Some(t));
            assert!(!t.animation().is_empty());
        }
    }
}
