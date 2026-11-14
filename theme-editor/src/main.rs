//! The theme editor: opens, edits and saves theme files, with a control for each token by its
//! type, and starts the gallery with the values being edited.
//!
//! `theme-editor [FILE]`. The editor's own look is `themes/libadwaita.conf`, so editing never
//! changes the tool itself.

use iced::widget::{column, container, row, scrollable, space, text};
use iced::{Alignment, Element, Length, Subscription, Task};
use iced_adaptive_design as iad;
use iad::components as cmp;
use iad::primitives::numeric::Decimal;
use iad::theme_config::{Inheritance, Schema, Spec, Value, VariableType};
use iad::widgets as w;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Child, Command};

const LIBADWAITA: &str = include_str!("../../themes/libadwaita.conf");
/// The editor's own look is libadwaita's, with a neutral primary color: it has no blue buttons.
const NEUTRAL_PRIMARY: &str = "\n[colors]\nprimary=#3d3d44\nprimary-fg=#ffffff\n[dark:colors]\nprimary=#d0d0d8\nprimary-fg=#1d1d20\n";
fn main() -> iced::Result {
    // The editor's look never depends on the file being edited or on the desktop.
    let own = iad::theme_config::ThemeConfig::builder("org.iad.ThemeEditor")
        .files(false)
        .inheritance(Inheritance::OptOut)
        .app_text(format!("{LIBADWAITA}{NEUTRAL_PRIMARY}"))
        .build();
    iad::config::install(iad::config::Config::default().with_theme_config(&own));
    iad::dev::Switches::from_env().apply();
    iced::application(Editor::new, Editor::update, Editor::view)
        .title(Editor::title)
        .theme(Editor::theme)
        .subscription(Editor::subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(1180.0, 780.0),
            ..iad::windows::settings_with_ssd(true)
        })
        .settings(iced::Settings {
            antialiasing: true,
            ..Default::default()
        })
        .run()
}

/// Which variant of the theme is being edited: the base, or the one for a color scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Variant {
    Base,
    Light,
    Dark,
}

impl Variant {
    const ALL: [Variant; 3] = [Variant::Base, Variant::Light, Variant::Dark];
    fn label(self) -> &'static str {
        match self {
            Variant::Base => "Base",
            Variant::Light => "Light",
            Variant::Dark => "Dark",
        }
    }
    fn key(self) -> &'static str {
        match self {
            Variant::Base => "base",
            Variant::Light => "light",
            Variant::Dark => "dark",
        }
    }
    fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.key() == key.trim())
    }
    fn prefix(self) -> &'static str {
        match self {
            Variant::Base => "",
            Variant::Light => "light:",
            Variant::Dark => "dark:",
        }
    }
}

/// One entry of the Themes list: a real file.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Preset {
    group: &'static str,
    name: String,
    path: PathBuf,
    /// Where the entry points, when the file is a symlink.
    target: Option<PathBuf>,
    /// `default.conf` points at this file: the one apps use when `IAD_THEME` is unset.
    is_default: bool,
}

/// What a click on the list or on New asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {
    File(PathBuf),
    New,
}

/// What the editor remembers between runs, in `$XDG_CONFIG_HOME/iad-theme-editor/state.conf`:
/// `[state] last-file`, `folders`, `recent` (paths joined with `:`), `section`, `variant`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct State {
    last_file: Option<PathBuf>,
    folders: Vec<PathBuf>,
    recent: Vec<PathBuf>,
    section: Option<String>,
    variant: Option<Variant>,
}

const RECENT_MAX: usize = 8;

impl State {
    fn parse(text: &str) -> Self {
        let kv = iad::config::parse_keyfile(text);
        let get = |k: &str| kv.get(&("state".to_string(), k.to_string()));
        let paths = |k: &str| {
            get(k)
                .map(|v| v.split(':').filter(|p| !p.is_empty()).map(PathBuf::from).collect())
                .unwrap_or_default()
        };
        Self {
            last_file: get("last-file").filter(|v| !v.is_empty()).map(PathBuf::from),
            folders: paths("folders"),
            recent: paths("recent"),
            section: get("section").filter(|v| !v.is_empty()).cloned(),
            variant: get("variant").and_then(|v| Variant::from_key(v)),
        }
    }

    fn to_text(&self) -> String {
        let join = |ps: &[PathBuf]| {
            ps.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(":")
        };
        format!(
            "[state]\nlast-file={}\nfolders={}\nrecent={}\nsection={}\nvariant={}\n",
            self.last_file.as_ref().map_or(String::new(), |p| p.display().to_string()),
            join(&self.folders),
            join(&self.recent),
            self.section.clone().unwrap_or_default(),
            self.variant.map_or("base", Variant::key),
        )
    }

    /// Puts `path` first in the recent list, once, keeping at most [`RECENT_MAX`].
    fn remember(&mut self, path: &std::path::Path) {
        self.recent.retain(|p| p != path);
        self.recent.insert(0, path.to_path_buf());
        self.recent.truncate(RECENT_MAX);
    }
}

/// Where the state is kept: `THEME_EDITOR_STATE`, else the user's config directory.
fn state_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("THEME_EDITOR_STATE").filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(p));
    }
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .map(|d| d.join("iad-theme-editor/state.conf"))
}

/// The `*.conf` files directly in `dir`, by name.
fn scan_folder(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "conf"))
        .collect();
    files.sort();
    files
}

/// A theme file's display name: its `[theme] name`, else its file name without `.conf`.
fn theme_name(path: &std::path::Path) -> String {
    let stem = path
        .file_stem()
        .map_or(String::new(), |s| s.to_string_lossy().into_owned());
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| {
            iad::config::parse_keyfile(&t)
                .get(&("theme".to_string(), "name".to_string()))
                .filter(|n| !n.is_empty())
                .cloned()
        })
        .unwrap_or(stem)
}

fn canon(p: &std::path::Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

fn is_link(p: &std::path::Path) -> bool {
    std::fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink())
}

fn is_default_name(p: &std::path::Path) -> bool {
    p.file_name().is_some_and(|n| n == "default.conf")
}

/// The file `default.conf` in `dir` points at.
fn default_target(dir: &std::path::Path) -> Option<PathBuf> {
    let link = dir.join("default.conf");
    (std::fs::symlink_metadata(&link).is_ok())
        .then(|| std::fs::canonicalize(link).ok())
        .flatten()
}

/// The symlinks in `dir` (other than `default.conf`) that point at `file`.
fn links_to(dir: &std::path::Path, file: &std::path::Path) -> Vec<PathBuf> {
    let target = canon(file);
    scan_folder(dir)
        .into_iter()
        .filter(|p| is_link(p) && !is_default_name(p) && canon(p) == target)
        .collect()
}

/// Whether apps see `file`: it sits in `dir` itself or a symlink there points at it.
fn is_registered(dir: &std::path::Path, file: &std::path::Path) -> bool {
    let inside = canon(file).parent() == Some(canon(dir).as_path()) && !is_default_name(file);
    inside || !links_to(dir, file).is_empty()
}

/// Registers `file` for every app: a symlink `<stem>.conf` in `dir` pointing at it. An old symlink
/// of that name is replaced; a regular file of that name is never overwritten.
fn register(dir: &std::path::Path, file: &std::path::Path) -> Result<PathBuf, String> {
    if is_registered(dir, file) {
        return Err("already registered".into());
    }
    let stem = file
        .file_stem()
        .ok_or_else(|| "the file has no name".to_string())?
        .to_string_lossy()
        .into_owned();
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let link = dir.join(format!("{stem}.conf"));
    if std::fs::symlink_metadata(&link).is_ok() {
        if !is_link(&link) {
            return Err(format!("{} is a regular file: not overwritten", link.display()));
        }
        std::fs::remove_file(&link).map_err(|e| format!("{}: {e}", link.display()))?;
    }
    std::os::unix::fs::symlink(canon(file), &link).map_err(|e| format!("{}: {e}", link.display()))?;
    Ok(link)
}

/// Removes the symlinks in `dir` that register `file`; never a regular file.
fn unregister(dir: &std::path::Path, file: &std::path::Path) -> Result<usize, String> {
    let links = links_to(dir, file);
    if links.is_empty() {
        return Err("not registered by a symlink".into());
    }
    for l in &links {
        std::fs::remove_file(l).map_err(|e| format!("{}: {e}", l.display()))?;
    }
    Ok(links.len())
}

/// Makes `file` the one apps use without `IAD_THEME`: `default.conf` in `dir` points at it.
fn set_default(dir: &std::path::Path, file: &std::path::Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let link = dir.join("default.conf");
    if std::fs::symlink_metadata(&link).is_ok() {
        if !is_link(&link) {
            return Err(format!("{} is a regular file: not overwritten", link.display()));
        }
        std::fs::remove_file(&link).map_err(|e| format!("{}: {e}", link.display()))?;
    }
    std::os::unix::fs::symlink(canon(file), &link).map_err(|e| format!("{}: {e}", link.display()))?;
    Ok(link)
}

/// The theme files available, each once: the registered ones (the files and symlinks in the data
/// dirs `dirs`), then those in the folders the user added, then the recently opened ones.
fn list_presets(dirs: &[PathBuf], folders: &[PathBuf], recent: &[PathBuf]) -> Vec<Preset> {
    let default = dirs.iter().find_map(|d| default_target(d));
    let mut out: Vec<Preset> = Vec::new();
    let mut seen: Vec<PathBuf> = Vec::new();
    let mut add = |group: &'static str, files: Vec<PathBuf>| {
        for f in files {
            let key = canon(&f);
            if seen.contains(&key) {
                continue;
            }
            out.push(Preset {
                group,
                name: theme_name(&f),
                target: is_link(&f).then(|| key.clone()),
                is_default: default.as_ref() == Some(&key),
                path: f,
            });
            seen.push(key);
        }
    };
    add(
        "Registered",
        dirs.iter()
            .flat_map(|d| scan_folder(d))
            .filter(|p| !is_default_name(p))
            .collect(),
    );
    if let Some(d) = &default {
        add("Registered", vec![d.clone()]);
    }
    for folder in folders {
        add("Folders", scan_folder(folder));
    }
    add("Recent", recent.iter().filter(|p| p.is_file()).cloned().collect());
    out
}

/// What the registering buttons do.
#[derive(Debug, Clone, Copy)]
enum Registry {
    Register,
    Unregister,
    Default,
}

/// A token's address in a file: `(section, key)`, the section carrying a `light:`/`dark:` prefix
/// for a variant.
type Addr = (String, String);

struct Editor {
    theme: iad::Theme,
    specs: Vec<Spec>,
    sections: Vec<String>,
    section: String,
    variant: Variant,
    path: Option<PathBuf>,
    /// Every key of the file as written, including ones the schema does not know.
    values: BTreeMap<Addr, String>,
    /// The text last saved or opened, to tell whether there is anything to save.
    clean: String,
    /// What each control shows, per token of the variant being edited.
    texts: BTreeMap<Addr, String>,
    status: String,
    gallery: Option<Child>,
    /// Where the state is kept; `None` keeps nothing (tests).
    state_file: Option<PathBuf>,
    state: State,
    presets: Vec<Preset>,
    /// The data dirs themes are registered in (`theme_dirs()`); the first is written to.
    theme_dirs: Vec<PathBuf>,
    /// A switch waiting for a second click, because there are unsaved changes.
    pending: Option<Target>,
}

#[derive(Debug, Clone)]
enum Msg {
    Pick(PathBuf),
    New,
    Register,
    Unregister,
    SetDefault,
    AddFolder,
    FolderAdded(Option<PathBuf>),
    Refresh,
    Section(String),
    Variant(Variant),
    /// A control changed: `(section, key)` of the base token, the new text.
    Input(String, String, String),
    Toggle(String, String, bool),
    Reset(String, String),
    Open,
    Opened(Option<PathBuf>),
    Save,
    SaveAs,
    SavedAs(Option<PathBuf>),
    Gallery,
    /// Development screenshots (`iad::dev`).
    Window(Option<iced::window::Id>),
    Captured(Result<PathBuf, String>),
}

impl Editor {
    fn new() -> (Self, Task<Msg>) {
        Self::with(state_path(), std::env::args_os().nth(1).map(PathBuf::from))
    }

    /// An editor keeping its state in `state_file` (none: nothing is read or written), opening
    /// `cli_file`, else the file it had open last.
    fn with(state_file: Option<PathBuf>, cli_file: Option<PathBuf>) -> (Self, Task<Msg>) {
        let state = state_file
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|t| State::parse(&t))
            .unwrap_or_default();
        let schema = Schema::toolkit();
        let specs = schema.specs().to_vec();
        let mut sections: Vec<String> = Vec::new();
        for s in &specs {
            if !sections.contains(&s.section) {
                sections.push(s.section.clone());
            }
        }
        let theme = iad::Theme::from_config(&iad::config::current());
        let mut e = Self {
            theme,
            specs,
            section: sections.first().cloned().unwrap_or_default(),
            sections,
            variant: Variant::Base,
            path: None,
            values: BTreeMap::new(),
            clean: String::new(),
            texts: BTreeMap::new(),
            status: "New theme. Nothing is set: every token is its default.".into(),
            gallery: None,
            presets: Vec::new(),
            theme_dirs: iad::theme_config::theme_dirs(),
            pending: None,
            state_file,
            state,
        };
        if let Some(s) = e.state.section.clone().filter(|s| e.sections.contains(s)) {
            e.section = s;
        }
        if let Some(v) = e.state.variant {
            e.variant = v;
        }
        let open = cli_file.or_else(|| e.state.last_file.clone().filter(|p| p.is_file()));
        if let Some(path) = open {
            e.load(path);
        }
        if let Some(s) = std::env::var("THEME_EDITOR_SECTION").ok().filter(|s| e.sections.contains(s)) {
            e.section = s;
        }
        if e.path.is_none() {
            e.clean = e.generate();
        }
        e.rescan();
        e.refresh();
        let boot = if iad::dev::Switches::from_env().screenshot.is_some() {
            iced::window::latest().map(Msg::Window)
        } else {
            Task::none()
        };
        (e, boot)
    }

    fn title(&self) -> String {
        let name = self
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map_or("Untitled".to_string(), |n| n.to_string_lossy().into_owned());
        format!("{}{name} - Theme editor", self.dot())
    }

    fn theme(&self) -> iad::Theme {
        self.theme.clone()
    }

    fn dirty(&self) -> bool {
        self.generate() != self.clean
    }

    /// "● " when there are unsaved changes, nothing otherwise.
    fn dot(&self) -> &'static str {
        if self.dirty() { "● " } else { "" }
    }

    /// The directory themes are registered in (the user's data dir).
    fn registry(&self) -> Option<&std::path::Path> {
        self.theme_dirs.first().map(PathBuf::as_path)
    }

    // ------------------------------------------------------------ the file

    fn load(&mut self, path: PathBuf) {
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let report = Schema::toolkit().validate(&text);
                self.values = iad::config::parse_keyfile(&text).into_iter().collect();
                self.clean = self.generate();
                self.status = if report.errors.is_empty() && report.unknown.is_empty() {
                    format!("Opened {}", path.display())
                } else {
                    format!(
                        "Opened {} with {} invalid and {} unknown values (unknown ones are kept)",
                        path.display(),
                        report.errors.len(),
                        report.unknown.len()
                    )
                };
                self.remember(&path);
                self.path = Some(path);
                self.pending = None;
                self.rescan();
            }
            Err(e) => self.status = format!("Could not open {}: {e}", path.display()),
        }
    }

    /// An empty theme with no file: Save asks where to write it.
    fn new_theme(&mut self) {
        self.values.clear();
        self.path = None;
        self.state.last_file = None;
        self.clean = self.generate();
        self.pending = None;
        self.status = "New theme. Nothing is set: every token is its default.".into();
        self.persist();
    }

    fn rescan(&mut self) {
        self.presets = list_presets(&self.theme_dirs, &self.state.folders, &self.state.recent);
    }

    /// Records `path` as the open file and the most recent one, and saves the state.
    fn remember(&mut self, path: &std::path::Path) {
        self.state.last_file = Some(path.to_path_buf());
        self.state.remember(path);
        self.persist();
    }

    /// Writes the state, best effort.
    fn persist(&mut self) {
        self.state.section = Some(self.section.clone());
        self.state.variant = Some(self.variant);
        if let Some(file) = &self.state_file {
            if let Some(dir) = file.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(file, self.state.to_text());
        }
    }

    /// Switches to a file, or to a new theme. With unsaved changes the first click only warns.
    fn pick(&mut self, target: Target) {
        if let (Target::File(p), Some(open)) = (&target, &self.path) {
            if canon(p) == canon(open) {
                return;
            }
        }
        if self.dirty() && self.pending.as_ref() != Some(&target) {
            self.status = "● unsaved changes: click again to discard".into();
            self.pending = Some(target);
            return;
        }
        self.pending = None;
        match target {
            Target::New => self.new_theme(),
            Target::File(path) => self.load(path),
        }
        self.refresh();
    }

    /// The file for the values set: schema sections in order, a section's variants after its
    /// base, unknown keys kept at the end of their section.
    fn generate(&self) -> String {
        let mut out = String::from(
            "# An iced-adaptive-design theme file. Only the tokens set here take effect;\n# everything else is the toolkit default. [light:*] and [dark:*] sections win over the base\n# on that color scheme.\n",
        );
        let mut order: Vec<&str> = self.sections.iter().map(String::as_str).collect();
        let mut extra: Vec<&str> = Vec::new();
        for (section, _) in self.values.keys() {
            let base = section.split_once(':').map_or(section.as_str(), |(_, b)| b);
            if !order.contains(&base) && !extra.contains(&base) {
                extra.push(base);
            }
        }
        order.extend(extra);
        for base in order {
            for prefix in ["", "light:", "dark:"] {
                let section = format!("{prefix}{base}");
                let mut keys: Vec<&String> = self
                    .values
                    .keys()
                    .filter(|(s, _)| *s == section)
                    .map(|(_, k)| k)
                    .collect();
                if keys.is_empty() {
                    continue;
                }
                let rank = |k: &String| {
                    self.specs
                        .iter()
                        .position(|s| s.section == base && s.key == *k)
                        .unwrap_or(usize::MAX)
                };
                keys.sort_by(|a, b| rank(a).cmp(&rank(b)).then(a.cmp(b)));
                out.push_str(&format!("\n[{section}]\n"));
                for k in keys {
                    out.push_str(&format!("{k}={}\n", self.values[&(section.clone(), k.clone())]));
                }
            }
        }
        out
    }

    /// Writes the values to `path`: creates the file, or replaces the one there, through a
    /// symlink to what it points at. This is the only place the editor writes a theme.
    fn save_to(&mut self, path: PathBuf) {
        let text = self.generate();
        match std::fs::write(&path, &text) {
            Ok(()) => {
                self.clean = text;
                self.status = format!("Saved {}", path.display());
                self.remember(&path);
                self.path = Some(path);
                self.pending = None;
                self.rescan();
            }
            Err(e) => self.status = format!("Could not save {}: {e}", path.display()),
        }
    }

    // ------------------------------------------------------------- tokens

    fn section_of(&self, base: &str) -> String {
        format!("{}{base}", self.variant.prefix())
    }

    /// The raw text a token shows now: the variant's own value, else the base's, else the default.
    fn shown(&self, spec: &Spec) -> (String, bool) {
        let here = (self.section_of(&spec.section), spec.key.clone());
        if let Some(v) = self.values.get(&here) {
            return (v.clone(), true);
        }
        if let Some(v) = self.values.get(&(spec.section.clone(), spec.key.clone())) {
            return (v.clone(), false);
        }
        (spec.default.as_ref().map(Value::to_text).unwrap_or_default(), false)
    }

    /// What a number or text control shows for `raw`: the number without its unit.
    fn display(spec: &Spec, raw: &str) -> String {
        match spec.ty {
            VariableType::Length { .. } => raw.trim_end_matches("px").trim().to_string(),
            VariableType::Percentage { .. } => raw.trim_end_matches('%').trim().to_string(),
            _ => raw.to_string(),
        }
    }

    /// The raw file text for what a number or text control holds.
    fn raw(spec: &Spec, shown: &str) -> String {
        match spec.ty {
            VariableType::Length { .. } => format!("{}px", shown.trim()),
            VariableType::Percentage { .. } => format!("{}%", shown.trim()),
            _ => shown.trim().to_string(),
        }
    }

    fn refresh(&mut self) {
        self.texts = self
            .specs
            .iter()
            .map(|s| {
                let (raw, _) = self.shown(s);
                ((s.section.clone(), s.key.clone()), Self::display(s, &raw))
            })
            .collect();
    }

    fn spec(&self, section: &str, key: &str) -> Option<&Spec> {
        self.specs
            .iter()
            .find(|s| s.section == section && s.key == key)
    }

    // ----------------------------------------------------------- registering

    fn registry_op(&mut self, op: Registry) {
        let (Some(dir), Some(file)) = (self.registry().map(PathBuf::from), self.path.clone()) else {
            self.status = "Save the theme to a file first".into();
            return;
        };
        let done = match op {
            Registry::Register => register(&dir, &file).map(|l| format!("Registered {} -> {}", l.display(), file.display())),
            Registry::Unregister => unregister(&dir, &file).map(|n| format!("Unregistered {} symlink(s) of {}", n, file.display())),
            Registry::Default => set_default(&dir, &file).map(|l| format!("{} is the default theme ({})", file.display(), l.display())),
        };
        self.status = match done {
            Ok(s) | Err(s) => s,
        };
        self.rescan();
    }

    // ------------------------------------------------------------ try it out

    /// Starts the gallery with the values as they are now, from a temp file of its own: the file
    /// being edited is never touched.
    fn launch_gallery(&mut self) {
        if let Some(mut old) = self.gallery.take() {
            let _ = old.kill();
            let _ = old.wait();
        }
        let file = std::env::temp_dir().join(format!("iad-theme-editor-{}.conf", std::process::id()));
        if let Err(e) = std::fs::write(&file, self.generate()) {
            self.status = format!("Could not write {}: {e}", file.display());
            return;
        }
        let sibling = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("gallery")))
            .filter(|p| p.is_file());
        let mut cmd = match (std::env::var_os("IAD_GALLERY"), sibling) {
            (Some(path), _) => Command::new(path),
            (None, Some(path)) => Command::new(path),
            (None, None) => {
                let mut c = Command::new("cargo");
                c.args(["run", "-q", "-p", "iced-adaptive-design-gallery"])
                    .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."));
                c
            }
        };
        cmd.env("IAD_THEME", &file);
        match cmd.spawn() {
            Ok(child) => {
                self.gallery = Some(child);
                self.status = "Tried it out: the gallery runs with these values from a temp file (nothing saved). Press again to restart it with new ones.".into();
            }
            Err(e) => self.status = format!("Could not start the gallery: {e}"),
        }
    }

    // ------------------------------------------------------------ update

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Section(s) => {
                self.section = s;
                self.persist();
            }
            Msg::Variant(v) => {
                self.variant = v;
                self.refresh();
                self.persist();
            }
            Msg::Pick(path) => self.pick(Target::File(path)),
            Msg::New => self.pick(Target::New),
            Msg::Register => self.registry_op(Registry::Register),
            Msg::Unregister => self.registry_op(Registry::Unregister),
            Msg::SetDefault => self.registry_op(Registry::Default),
            Msg::Refresh => self.rescan(),
            Msg::AddFolder => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .pick_folder()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    Msg::FolderAdded,
                );
            }
            Msg::FolderAdded(Some(dir)) => {
                if !self.state.folders.contains(&dir) {
                    self.state.folders.push(dir);
                }
                self.persist();
                self.rescan();
            }
            Msg::FolderAdded(None) => {}
            Msg::Input(section, key, shown) => {
                self.texts.insert((section.clone(), key.clone()), shown.clone());
                if let Some(spec) = self.spec(&section, &key) {
                    let raw = Self::raw(spec, &shown);
                    if spec.ty.parse(&raw).is_some() {
                        let at = (self.section_of(&section), key);
                        self.values.insert(at, raw);
                        self.status.clear();
                    } else {
                        self.status = format!("[{section}] {key}: {shown:?} is not valid yet");
                    }
                }
            }
            Msg::Toggle(section, key, on) => {
                let at = (self.section_of(&section), key.clone());
                self.values.insert(at, on.to_string());
                self.refresh();
            }
            Msg::Reset(section, key) => {
                self.values.remove(&(self.section_of(&section), key));
                self.refresh();
            }
            Msg::Open => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("Theme", &["conf"])
                            .pick_file()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    Msg::Opened,
                );
            }
            Msg::Opened(Some(path)) => {
                self.pick(Target::File(path));
            }
            Msg::Opened(None) | Msg::SavedAs(None) => {}
            Msg::Save => match self.path.clone() {
                Some(p) => self.save_to(p),
                None => return self.update(Msg::SaveAs),
            },
            Msg::SaveAs => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("Theme", &["conf"])
                            .set_file_name("theme.conf")
                            .save_file()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    Msg::SavedAs,
                );
            }
            Msg::SavedAs(Some(path)) => self.save_to(path),
            Msg::Gallery => self.launch_gallery(),
            Msg::Window(Some(id)) => {
                if let Some(shot) = iad::dev::Switches::from_env().capture(id) {
                    return shot.map(Msg::Captured);
                }
            }
            Msg::Window(None) => {}
            Msg::Captured(done) => {
                if let Err(e) = &done {
                    eprintln!("screenshot: {e}");
                }
                return iced::exit();
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Msg> {
        iced::keyboard::listen().filter_map(|event| match event {
            iced::keyboard::Event::KeyPressed {
                key: iced::keyboard::Key::Character(c),
                modifiers,
                ..
            } if modifiers.command() => match (c.as_str(), modifiers.shift()) {
                ("s", false) => Some(Msg::Save),
                ("s", true) => Some(Msg::SaveAs),
                ("o", _) => Some(Msg::Open),
                _ => None,
            },
            _ => None,
        })
    }

    // -------------------------------------------------------------- view

    fn view(&self) -> Element<'_, Msg, iad::Theme> {
        let m = iad::Metrics::desktop();
        let mut toolbar = row![
            cmp::raised_button(cmp::typography::label("New")).on_press(Msg::New),
            cmp::raised_button(cmp::typography::label("Open…")).on_press(Msg::Open),
            cmp::raised_button(cmp::typography::label("Save")).on_press(Msg::Save),
            cmp::raised_button(cmp::typography::label("Save as")).on_press(Msg::SaveAs),
        ]
        .spacing(m.space_s)
        .align_y(Alignment::Center);
        if let (Some(dir), Some(file)) = (self.registry(), &self.path) {
            let linked = !links_to(dir, file).is_empty();
            if !is_registered(dir, file) {
                toolbar = toolbar.push(
                    cmp::flat_button(cmp::typography::label("Register")).on_press(Msg::Register),
                );
            }
            if linked {
                toolbar = toolbar.push(
                    cmp::flat_button(cmp::typography::label("Unregister")).on_press(Msg::Unregister),
                );
            }
            if default_target(dir) != Some(canon(file)) {
                toolbar = toolbar.push(
                    cmp::flat_button(cmp::typography::label("Set as default")).on_press(Msg::SetDefault),
                );
            }
        }
        let toolbar = toolbar
            .push(space().width(Length::Fill))
            .push(cmp::raised_button(cmp::typography::label("Try it out")).on_press(Msg::Gallery));

        let variants = row(Variant::ALL.map(|v| {
            let b = cmp::typography::label(v.label());
            if v == self.variant {
                cmp::raised_button(b).on_press(Msg::Variant(v)).into()
            } else {
                Element::from(cmp::flat_button(b).on_press(Msg::Variant(v)))
            }
        }))
        .spacing(m.space_xs);

        let sections = column(self.sections.iter().map(|s| {
            let b = cmp::typography::label(s.as_str());
            let b = if *s == self.section {
                cmp::raised_button(b)
            } else {
                cmp::flat_button(b)
            };
            b.on_press(Msg::Section(s.clone())).width(Length::Fill).into()
        }))
        .spacing(m.space_xs)
        .width(Length::Fill);

        let dot = self.dot();
        let open = self.path.as_deref().map(canon);
        let mut themes = column![].spacing(m.space_xs).width(Length::Fill);
        let mut group = "";
        for preset in &self.presets {
            if preset.group != group {
                group = preset.group;
                themes = themes.push(text(group).size(m.space_m));
            }
            let active = open.as_deref() == Some(canon(&preset.path).as_path());
            let mut title = row![text(format!(
                "{}{}",
                if active { dot } else { "" },
                preset.name
            ))]
            .spacing(m.space_xs);
            if preset.is_default {
                title = title
                    .push(space().width(Length::Fill))
                    .push(text("default").size(m.space_m));
            }
            let mut label = column![title].width(Length::Fill);
            if let Some(target) = &preset.target {
                label = label.push(text(target.display().to_string()).size(m.space_m - 1.0));
            }
            let b = if active {
                cmp::raised_button(label)
            } else {
                cmp::flat_button(label)
            };
            themes = themes.push(b.on_press(Msg::Pick(preset.path.clone())).width(Length::Fill));
        }
        let folder_buttons = row![
            cmp::flat_button(cmp::typography::label("Add folder…")).on_press(Msg::AddFolder),
            cmp::flat_button(cmp::typography::label("Refresh")).on_press(Msg::Refresh),
        ]
        .spacing(m.space_xs);
        let side = column![
            text("Variant").size(m.space_m),
            variants,
            text("Section").size(m.space_m),
            scrollable(sections).height(Length::FillPortion(2)),
            text("Themes").size(m.space_m),
            scrollable(themes).height(Length::FillPortion(3)),
            folder_buttons,
        ]
        .spacing(m.space_xs)
        .width(Length::Fixed(240.0))
        .height(Length::Fill);

        let page = scrollable(
            container(self.page(m))
                .padding(m.page_padding)
                .max_width(m.clamp_maximum + 120.0),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        let path = self
            .path
            .as_ref()
            .map_or("no file yet".to_string(), |p| p.display().to_string());
        let status = row![
            text(format!("{dot}{path}")).size(m.space_m),
            space().width(Length::Fill),
            text(self.status.as_str()).size(m.space_m),
        ];

        container(
            column![
                toolbar,
                row![side, page].spacing(m.space_m).height(Length::Fill),
                status,
            ]
            .spacing(m.space_s),
        )
        .padding(m.space_m)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|t: &iad::Theme| iced::widget::container::Style {
            background: Some(t.semantic.window.bg.into()),
            text_color: Some(t.semantic.window.fg),
            ..Default::default()
        })
        .into()
    }

    /// The tokens of the current section, grouped by the first word of their key.
    fn page(&self, m: iad::Metrics) -> Element<'_, Msg, iad::Theme> {
        let in_section: Vec<&Spec> = self
            .specs
            .iter()
            .filter(|s| s.section == self.section)
            .collect();
        let prefix = |k: &str| k.split('-').next().unwrap_or(k).to_string();
        let count = |p: &str| in_section.iter().filter(|s| prefix(&s.key) == p).count();
        let mut groups: Vec<(String, Vec<&Spec>)> = Vec::new();
        for s in &in_section {
            let p = prefix(&s.key);
            let title = if count(&p) >= 2 { p } else { "general".to_string() };
            match groups.iter_mut().find(|(t, _)| *t == title) {
                Some((_, v)) => v.push(s),
                None => groups.push((title, vec![s])),
            }
        }
        let mut page = column![].spacing(m.group_spacing);
        let variant_note = match self.variant {
            Variant::Base => String::new(),
            v => format!(
                " Editing the {} variant: values here win over the base on that color scheme.",
                v.label().to_lowercase()
            ),
        };
        for (title, specs) in groups {
            let mut g = w::preferences_group().title(title);
            if title_is_first(&page_titles(&in_section), &specs) && !variant_note.is_empty() {
                g = g.description(variant_note.clone());
            }
            for spec in specs {
                g = g.push(self.token_row(spec, m));
            }
            page = page.push(g);
        }
        page.into()
    }

    /// One token: its name, description, and a control for its type.
    fn token_row<'a>(&'a self, spec: &'a Spec, m: iad::Metrics) -> Element<'a, Msg, iad::Theme> {
        let (raw, set_here) = self.shown(spec);
        let section = spec.section.clone();
        let key = spec.key.clone();
        let shown = self.texts.get(&(section.clone(), key.clone())).map_or("", String::as_str);
        let default = spec.default.as_ref().map(Value::to_text).unwrap_or_default();
        let control: Element<'a, Msg, iad::Theme> = match &spec.ty {
            VariableType::Bool => {
                let (s, k) = (section.clone(), key.clone());
                cmp::switch(raw == "true")
                    .on_toggle(move |on| Msg::Toggle(s.clone(), k.clone(), on))
                    .into()
            }
            VariableType::String { values, .. } if !values.is_empty() => {
                let (s, k) = (section.clone(), key.clone());
                w::dropdown(values, Some(raw.clone()), move |v| {
                    Msg::Input(s.clone(), k.clone(), v)
                })
                .into()
            }
            VariableType::Color => {
                let color = match VariableType::Color.parse(&raw) {
                    Some(Value::Color(c)) => c,
                    _ => iced::Color::TRANSPARENT,
                };
                let (s, k) = (section.clone(), key.clone());
                row![
                    container(space())
                        .width(Length::Fixed(24.0))
                        .height(Length::Fixed(24.0))
                        .style(move |t: &iad::Theme| iced::widget::container::Style {
                            background: Some(color.into()),
                            border: iced::Border {
                                color: t.semantic.window.fg.scale_alpha(0.3),
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        }),
                    iad::components::entry("#rrggbb", shown)
                        .on_input(move |v| Msg::Input(s.clone(), k.clone(), v))
                        .width(Length::Fixed(190.0)),
                ]
                .spacing(m.space_s)
                .align_y(Alignment::Center)
                .into()
            }
            VariableType::String { .. } => {
                let (s, k) = (section.clone(), key.clone());
                iad::components::entry("", shown)
                    .on_input(move |v| Msg::Input(s.clone(), k.clone(), v))
                    .width(Length::Fixed(320.0))
                    .into()
            }
            ty => {
                let (s, k) = (section.clone(), key.clone());
                let (lo, hi, digits, step) = bounds(ty, &spec.section);
                let mut n = w::NumberInput::new(shown, move |v| Msg::Input(s.clone(), k.clone(), v))
                    .decimals(digits)
                    .step(step)
                    .width(150.0);
                if let Some(lo) = lo {
                    n = n.min(lo);
                }
                if let Some(hi) = hi {
                    n = n.max(hi);
                }
                match ty {
                    VariableType::Length { .. } => row![n, text("px")]
                        .spacing(m.space_xs)
                        .align_y(Alignment::Center)
                        .into(),
                    VariableType::Percentage { .. } => row![n, text("%")]
                        .spacing(m.space_xs)
                        .align_y(Alignment::Center)
                        .into(),
                    _ => n.into(),
                }
            }
        };
        let reset: Element<'a, Msg, iad::Theme> = if set_here {
            cmp::flat_button(cmp::typography::label("Reset"))
                .on_press(Msg::Reset(section, key))
                .into()
        } else {
            space().width(Length::Fixed(0.0)).into()
        };
        w::ActionRow::new(spec.key.as_str())
            .subtitle(format!("{}  (default {default})", spec.doc))
            .suffix(
                row![control, reset]
                    .spacing(m.space_s)
                    .align_y(Alignment::Center),
            )
            .metrics(m)
            .into()
    }
}

fn page_titles(specs: &[&Spec]) -> Vec<String> {
    specs.iter().map(|s| s.key.clone()).collect()
}

/// Whether `group` holds the first token of the page (where the variant note goes).
fn title_is_first(page: &[String], group: &[&Spec]) -> bool {
    group.first().is_some_and(|s| page.first() == Some(&s.key))
}

/// The limits, digits and step a number control of `ty` has. Tokens are px unless the section
/// says otherwise: typography ratios and the scale are small decimals.
fn bounds(ty: &VariableType, section: &str) -> (Option<Decimal>, Option<Decimal>, u8, Decimal) {
    let d = |s: &str| Decimal::parse(s);
    match ty {
        VariableType::Integer { min, max, step, .. } => (
            min.and_then(|v| d(&v.to_string())),
            max.and_then(|v| d(&v.to_string())),
            0,
            d(&step.to_string()).unwrap_or_else(|| Decimal::from(1)),
        ),
        VariableType::Decimal { no_decimals, min, max, step, .. } => (
            min.and_then(|v| d(&v.to_string())),
            max.and_then(|v| d(&v.to_string())),
            *no_decimals,
            step.and_then(|v| d(&v.to_string()))
                .unwrap_or_else(|| Decimal::from(1)),
        ),
        VariableType::Length { min, max } => (
            min.and_then(|v| d(&v.to_string())),
            max.and_then(|v| d(&v.to_string())),
            0,
            Decimal::from(1),
        ),
        VariableType::Percentage { min, max, .. } => (
            min.and_then(|v| d(&v.to_string())),
            max.and_then(|v| d(&v.to_string())),
            0,
            Decimal::from(1),
        ),
        _ if section == "typography" => (d("0"), None, 4, d("0.05").unwrap()),
        _ => (d("0"), None, 2, d("0.5").unwrap()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    fn editor() -> Editor {
        let mut e = Editor::with(None, None).0;
        // Tests never look at, or write into, the real data dirs.
        e.theme_dirs = Vec::new();
        e.presets.clear();
        e
    }

    fn with_text(text: &str) -> Editor {
        let mut e = editor();
        e.values = iad::config::parse_keyfile(text).into_iter().collect();
        e.clean = e.generate();
        e.refresh();
        e
    }

    fn temp_dir(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let d = std::env::temp_dir().join(format!(
            "iad-editor-test-{tag}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn read(p: &std::path::Path) -> String {
        std::fs::read_to_string(p).unwrap()
    }

    #[test]
    fn the_shipped_themes_round_trip_through_the_editor() {
        let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../themes"));
        let files = scan_folder(&dir);
        assert!(files.len() >= 6);
        for f in files {
            let text = read(&f);
            let e = with_text(&text);
            let out = e.generate();
            let a: BTreeMap<_, _> = iad::config::parse_keyfile(&text).into_iter().collect();
            let b: BTreeMap<_, _> = iad::config::parse_keyfile(&out).into_iter().collect();
            assert_eq!(a, b, "{}", f.display());
            let report = Schema::toolkit().validate(&out);
            assert!(report.errors.is_empty() && report.unknown.is_empty(), "{}", f.display());
        }
    }

    #[test]
    fn only_what_is_set_is_written_and_a_variant_goes_in_its_own_section() {
        let mut e = editor();
        assert!(!e.generate().lines().any(|l| l.starts_with('[')));
        let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
        let _ = e.update(Msg::Variant(Variant::Dark));
        let _ = e.update(Msg::Input("colors".into(), "window-bg".into(), "#101010".into()));
        let out = e.generate();
        assert!(out.contains("[metrics]\nradius-m=7\n"));
        assert!(out.contains("[dark:colors]\nwindow-bg=#101010\n"));
        assert!(!out.contains("[colors]"));
        // An unset dark value shows the base's, and reset returns to it.
        let _ = e.update(Msg::Reset("colors".into(), "window-bg".into()));
        assert!(!e.generate().lines().any(|l| l.starts_with("[dark:")));
    }

    #[test]
    fn a_value_that_is_not_valid_yet_is_not_written() {
        let mut e = editor();
        let _ = e.update(Msg::Input("colors".into(), "window-bg".into(), "#12".into()));
        assert!(!e.generate().contains("window-bg"));
        let _ = e.update(Msg::Input("windows".into(), "button-size".into(), "30".into()));
        assert!(e.generate().contains("button-size=30px"));
    }

    #[test]
    fn unknown_keys_are_kept() {
        let e = with_text("[player]\nartwork=disc\n[colors]\naccent=#112233\n");
        let out = e.generate();
        assert!(out.contains("[player]\nartwork=disc\n"));
        assert!(out.contains("accent=#112233"));
    }

    // ---------------------------------------------------------- the dot

    #[test]
    fn the_unsaved_dot_shows_only_when_there_are_unsaved_changes() {
        let d = temp_dir("dot");
        let f = d.join("t.conf");
        std::fs::write(&f, "[metrics]\nradius-m=1\n").unwrap();
        let mut e = editor();
        e.load(f.clone());
        assert!(!e.dirty());
        assert_eq!(e.dot(), "");
        assert!(!e.title().contains('●') && e.title().starts_with("t.conf"));
        let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "9".into()));
        assert_eq!(e.dot(), "● ");
        assert!(e.title().starts_with("● t.conf"));
        let _ = e.update(Msg::SavedAs(Some(f.clone())));
        assert_eq!(e.dot(), "");
        assert!(!e.title().contains('●'));
        let _ = std::fs::remove_dir_all(d);
    }

    // ------------------------------------------------------------ saving

    #[test]
    fn save_upserts_the_original_file() {
        let d = temp_dir("save");
        let f = d.join("mine.conf");
        let mut e = editor();
        let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
        let _ = e.update(Msg::SavedAs(Some(f.clone())));
        // Created.
        assert!(read(&f).contains("[metrics]\nradius-m=7\n"));
        assert!(!e.dirty());
        // Replaced, by Save, at the same path.
        let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "8".into()));
        assert!(e.dirty());
        let _ = e.update(Msg::Save);
        assert!(read(&f).contains("radius-m=8") && !read(&f).contains("radius-m=7"));
        assert_eq!(e.path, Some(f));
        assert!(!e.dirty());
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn save_writes_through_a_symlink_to_its_target() {
        let d = temp_dir("through");
        let real = d.join("real.conf");
        let link = d.join("link.conf");
        std::fs::write(&real, "[metrics]\nradius-m=1\n").unwrap();
        symlink(&real, &link).unwrap();
        let mut e = editor();
        e.load(link.clone());
        let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "5".into()));
        let _ = e.update(Msg::Save);
        assert!(is_link(&link), "the link is still a link");
        assert!(read(&real).contains("radius-m=5"));
        assert!(!e.dirty());
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn save_without_a_path_asks_where_instead_of_writing() {
        let mut e = editor();
        let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
        assert!(e.path.is_none());
        let task = e.update(Msg::Save);
        assert!(task.units() > 0, "Save as opens the file dialog");
        assert!(e.path.is_none() && e.dirty());
    }

    // -------------------------------------------------------- try it out

    #[test]
    fn try_it_out_runs_the_gallery_on_a_temp_file_and_leaves_the_original_alone() {
        let d = temp_dir("try");
        let f = d.join("orig.conf");
        std::fs::write(&f, "[metrics]\nradius-m=1\n").unwrap();
        let mut e = editor();
        e.load(f.clone());
        let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
        // SAFETY: this test is the only one that touches the variable.
        unsafe { std::env::set_var("IAD_GALLERY", "true") };
        e.launch_gallery();
        assert!(e.gallery.is_some(), "{}", e.status);
        let temp = std::env::temp_dir().join(format!("iad-theme-editor-{}.conf", std::process::id()));
        assert!(read(&temp).contains("radius-m=7"), "the unsaved value");
        assert_eq!(read(&f), "[metrics]\nradius-m=1\n", "the original is untouched");
        assert!(e.dirty() && e.path == Some(f));
        let _ = std::fs::remove_file(temp);
        let _ = std::fs::remove_dir_all(d);
    }

    // ------------------------------------------------------- registering

    #[test]
    fn register_makes_a_symlink_and_replaces_an_old_one() {
        let d = temp_dir("reg");
        let (data, other) = (d.join("data"), d.join("else"));
        std::fs::create_dir_all(&other).unwrap();
        let a = other.join("mine.conf");
        let b = other.join("sub");
        std::fs::create_dir_all(&b).unwrap();
        let b = b.join("mine.conf");
        std::fs::write(&a, "").unwrap();
        std::fs::write(&b, "").unwrap();
        assert!(!is_registered(&data, &a));
        let link = register(&data, &a).unwrap();
        assert_eq!(link, data.join("mine.conf"));
        assert!(is_link(&link) && canon(&link) == canon(&a));
        assert!(is_registered(&data, &a));
        assert_eq!(register(&data, &a), Err("already registered".to_string()));
        // Another file with the same name replaces the old symlink.
        register(&data, &b).unwrap();
        assert_eq!(canon(&data.join("mine.conf")), canon(&b));
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn register_never_overwrites_a_regular_file() {
        let d = temp_dir("regfile");
        let data = d.join("data");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(data.join("mine.conf"), "keep me").unwrap();
        let file = d.join("mine.conf");
        std::fs::write(&file, "").unwrap();
        let err = register(&data, &file).unwrap_err();
        assert!(err.contains("regular file"), "{err}");
        assert_eq!(read(&data.join("mine.conf")), "keep me");
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn unregister_removes_only_symlinks() {
        let d = temp_dir("unreg");
        let data = d.join("data");
        std::fs::create_dir_all(&data).unwrap();
        let file = d.join("mine.conf");
        std::fs::write(&file, "x").unwrap();
        register(&data, &file).unwrap();
        assert_eq!(unregister(&data, &file), Ok(1));
        assert!(!data.join("mine.conf").exists() && file.exists());
        assert!(unregister(&data, &file).is_err());
        // A regular file inside the data dir is registered but has no symlink to remove.
        let inside = data.join("own.conf");
        std::fs::write(&inside, "x").unwrap();
        assert!(is_registered(&data, &inside));
        assert!(unregister(&data, &inside).is_err());
        assert!(inside.exists());
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn set_as_default_points_default_conf_at_the_file_and_the_list_tags_it() {
        let d = temp_dir("default");
        let data = d.join("data");
        std::fs::create_dir_all(&data).unwrap();
        let (a, b) = (d.join("a.conf"), d.join("b.conf"));
        std::fs::write(&a, "[theme]\nname=Alpha\nkey=a\n").unwrap();
        std::fs::write(&b, "[theme]\nname=Beta\nkey=b\n").unwrap();
        register(&data, &a).unwrap();
        register(&data, &b).unwrap();
        set_default(&data, &a).unwrap();
        assert_eq!(default_target(&data), Some(canon(&a)));
        let list = list_presets(&[data.clone()], &[], &[]);
        assert_eq!(list.len(), 2, "default.conf is not a third entry");
        let tagged: Vec<_> = list.iter().filter(|p| p.is_default).map(|p| p.name.as_str()).collect();
        assert_eq!(tagged, ["Alpha"]);
        // Replaced by another file.
        set_default(&data, &b).unwrap();
        let list = list_presets(&[data.clone()], &[], &[]);
        assert_eq!(list.iter().filter(|p| p.is_default).map(|p| p.name.as_str()).collect::<Vec<_>>(), ["Beta"]);
        // A regular default.conf is not overwritten.
        let other = temp_dir("default2");
        std::fs::write(other.join("default.conf"), "mine").unwrap();
        assert!(set_default(&other, &a).unwrap_err().contains("regular file"));
        assert_eq!(read(&other.join("default.conf")), "mine");
        let _ = std::fs::remove_dir_all(d);
        let _ = std::fs::remove_dir_all(other);
    }

    #[test]
    fn a_default_that_is_not_registered_still_shows_up_tagged() {
        let d = temp_dir("loosedefault");
        let data = d.join("data");
        std::fs::create_dir_all(&data).unwrap();
        let f = d.join("far.conf");
        std::fs::write(&f, "").unwrap();
        set_default(&data, &f).unwrap();
        let list = list_presets(&[data], &[], &[]);
        assert_eq!(list.len(), 1);
        assert!(list[0].is_default && list[0].group == "Registered");
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn the_registry_buttons_report_in_the_status_line_and_rescan() {
        let d = temp_dir("buttons");
        let data = d.join("data");
        let f = d.join("t.conf");
        std::fs::write(&f, "[theme]\nname=T\nkey=t\n").unwrap();
        let mut e = editor();
        e.theme_dirs = vec![data.clone()];
        let _ = e.update(Msg::Register);
        assert!(e.status.contains("Save the theme"), "no file yet");
        e.load(f.clone());
        let _ = e.update(Msg::Register);
        assert!(e.status.starts_with("Registered"), "{}", e.status);
        assert!(e.presets.iter().any(|p| p.group == "Registered" && p.name == "T"));
        let _ = e.update(Msg::SetDefault);
        assert!(e.presets.iter().any(|p| p.is_default));
        let _ = e.update(Msg::Unregister);
        assert!(!is_registered(&data, &f));
        let _ = std::fs::remove_dir_all(d);
    }

    // ------------------------------------------------------------- list

    #[test]
    fn a_clean_environment_lists_nothing() {
        let d = temp_dir("empty");
        assert!(list_presets(&[d.clone()], &[], &[]).is_empty());
        assert!(editor().presets.is_empty());
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn the_list_has_the_registered_files_the_folders_and_the_recent_ones() {
        let d = temp_dir("list");
        let data = d.join("data");
        let folder = d.join("folder");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("mine.conf"), "[theme]\nname=Mine\nkey=mine\n").unwrap();
        std::fs::write(folder.join("plain.conf"), "").unwrap();
        std::fs::write(data.join("own.conf"), "[theme]\nname=Own\nkey=own\n").unwrap();
        let far = temp_dir("recent").join("far.conf");
        std::fs::write(&far, "").unwrap();
        register(&data, &folder.join("plain.conf")).unwrap();
        let list = list_presets(
            &[data.clone()],
            &[folder.clone()],
            &[far.clone(), folder.join("mine.conf"), d.join("gone.conf")],
        );
        let group = |n: &str| list.iter().find(|p| p.name == n).map(|p| p.group);
        assert_eq!(group("Own"), Some("Registered"));
        assert_eq!(group("plain"), Some("Registered"), "registered, so not repeated under Folders");
        assert_eq!(group("Mine"), Some("Folders"));
        assert_eq!(group("far"), Some("Recent"));
        assert_eq!(list.iter().filter(|p| p.name == "Mine").count(), 1);
        assert!(list.iter().all(|p| p.path.is_file()), "only real files, none missing");
        // The symlink entry shows where it points.
        let plain = list.iter().find(|p| p.name == "plain").unwrap();
        assert_eq!(plain.target, Some(canon(&folder.join("plain.conf"))));
        assert_eq!(list.iter().find(|p| p.name == "Own").unwrap().target, None);
        // No embedded, shipped or repository group exists.
        assert!(list.iter().all(|p| ["Registered", "Folders", "Recent"].contains(&p.group)));
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn a_folder_scan_lists_only_conf_files_by_name() {
        let d = temp_dir("scan");
        for f in ["b.conf", "a.conf", "notes.txt", "c.conf.bak"] {
            std::fs::write(d.join(f), "").unwrap();
        }
        std::fs::create_dir(d.join("sub.conf")).unwrap();
        symlink(d.join("nowhere.conf"), d.join("broken.conf")).unwrap();
        let names: Vec<_> = scan_folder(&d)
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["a.conf", "b.conf"]);
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn switching_with_unsaved_changes_needs_a_second_click() {
        let d = temp_dir("switch");
        let (a, b) = (d.join("a.conf"), d.join("b.conf"));
        std::fs::write(&a, "[metrics]\nradius-m=1\n").unwrap();
        std::fs::write(&b, "[metrics]\nradius-m=2\n").unwrap();
        let mut e = editor();
        e.pick(Target::File(a.clone()));
        assert_eq!(e.path, Some(a.clone()));
        assert!(!e.dirty());
        // Clean: switching is at once.
        e.pick(Target::File(b.clone()));
        assert_eq!(e.path, Some(b.clone()));
        let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "9".into()));
        assert!(e.dirty());
        e.pick(Target::File(a.clone()));
        assert_eq!(e.path, Some(b.clone()), "the first click only warns");
        assert!(e.status.contains("click again"));
        assert_eq!(e.values[&("metrics".into(), "radius-m".into())], "9");
        e.pick(Target::File(a.clone()));
        assert_eq!(e.path, Some(a.clone()));
        assert_eq!(e.values[&("metrics".into(), "radius-m".into())], "1");
        // New is a switch too: it starts empty, with no path, and is not dirty.
        let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "4".into()));
        let _ = e.update(Msg::New);
        assert_eq!(e.path, Some(a.clone()), "the first click only warns");
        let _ = e.update(Msg::New);
        assert!(e.path.is_none() && e.values.is_empty() && !e.dirty());
        let _ = std::fs::remove_dir_all(d);
    }

    // ------------------------------------------------------------ state

    #[test]
    fn the_state_round_trips() {
        let state = State {
            last_file: Some("/a/one.conf".into()),
            folders: vec!["/x".into(), "/y".into()],
            recent: vec!["/a/one.conf".into(), "/b/two.conf".into()],
            section: Some("metrics".into()),
            variant: Some(Variant::Dark),
        };
        assert_eq!(State::parse(&state.to_text()), state);
        assert_eq!(State::parse(""), State::default());
        assert!(state.to_text().starts_with("[state]\nlast-file=/a/one.conf\n"));
    }

    #[test]
    fn the_recent_list_has_no_duplicates_and_a_limit() {
        let mut s = State::default();
        for i in 0..12 {
            s.remember(std::path::Path::new(&format!("/t/{i}.conf")));
        }
        s.remember(std::path::Path::new("/t/5.conf"));
        assert_eq!(s.recent.len(), RECENT_MAX);
        assert_eq!(s.recent[0], PathBuf::from("/t/5.conf"));
        assert_eq!(s.recent.iter().filter(|p| **p == PathBuf::from("/t/5.conf")).count(), 1);
    }

    #[test]
    fn the_open_file_section_and_variant_are_remembered_across_runs() {
        let d = temp_dir("state");
        let state = d.join("state.conf");
        let theme = d.join("t.conf");
        std::fs::write(&theme, "[theme]\nname=T\nkey=t\n").unwrap();
        let (mut e, _) = Editor::with(Some(state.clone()), None);
        e.theme_dirs = Vec::new();
        let _ = e.update(Msg::Pick(theme.clone()));
        let _ = e.update(Msg::Section("metrics".into()));
        let _ = e.update(Msg::Variant(Variant::Light));
        let _ = e.update(Msg::FolderAdded(Some(d.clone())));
        let (again, _) = Editor::with(Some(state.clone()), None);
        assert_eq!(again.path, Some(theme.clone()), "reopens the last file");
        assert_eq!(again.section, "metrics");
        assert_eq!(again.variant, Variant::Light);
        assert_eq!(again.state.folders, vec![d.clone()]);
        assert!(again.presets.iter().any(|p| p.path == theme));
        // A file on the command line wins over the last one.
        let other = d.join("o.conf");
        std::fs::write(&other, "").unwrap();
        let (cli, _) = Editor::with(Some(state), Some(other.clone()));
        assert_eq!(cli.path, Some(other));
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn saving_remembers_the_file_as_last_and_recent() {
        let d = temp_dir("remember");
        let state = d.join("state.conf");
        let (mut e, _) = Editor::with(Some(state.clone()), None);
        e.theme_dirs = Vec::new();
        let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
        let file = d.join("mine.conf");
        let _ = e.update(Msg::SavedAs(Some(file.clone())));
        assert_eq!(e.state.last_file, Some(file.clone()));
        assert_eq!(e.state.recent.first(), Some(&file));
        assert!(read(&state).contains("mine.conf"));
        assert!(!read(&state).contains("embedded"));
        let _ = std::fs::remove_dir_all(d);
    }
}
