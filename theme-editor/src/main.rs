//! The theme editor: opens, edits and saves theme files with a control for each token by its type,
//! previews them live, and starts the gallery or a toy window of a design language with the
//! values being edited.
//!
//! `theme-editor [FILE]`. The editor's own look is `themes/libadwaita.conf`, so editing never
//! changes the tool itself.

mod files;
mod preview;
mod toys;
mod tokens;
mod view;

#[cfg(test)]
mod tests;

use files::*;
use iced::{Subscription, Task};
use iced_adaptive_design as iad;
use iad::theme_config::{Inheritance, Schema, Spec, Value, VariableType};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use tokens::GradEdit;

const LIBADWAITA: &str = include_str!("../../themes/libadwaita.conf");
/// The editor's own look is libadwaita's, with a neutral primary color: it has no blue buttons.
const NEUTRAL_PRIMARY: &str = "\n[colors]\nprimary=#3d3d44\nprimary-fg=#ffffff\n[dark:colors]\nprimary=#d0d0d8\nprimary-fg=#1d1d20\n";
const APP_ID: &str = "org.iad.ThemeEditor";
/// How many edits undo goes back.
const UNDO_MAX: usize = 100;

fn main() -> iced::Result {
    // The editor's look never depends on the file being edited or on the desktop.
    let own = iad::theme_config::ThemeConfig::builder(APP_ID)
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
            size: window_size(),
            ..iad::windows::settings_for(APP_ID)
        })
        .settings(iced::Settings {
            antialiasing: true,
            ..Default::default()
        })
        .run()
}

/// The window size: `THEME_EDITOR_SIZE=WxH` (for screenshots), else 1480x860.
fn window_size() -> iced::Size {
    std::env::var("THEME_EDITOR_SIZE")
        .ok()
        .and_then(|v| {
            let (w, h) = v.split_once('x')?;
            Some(iced::Size::new(w.parse().ok()?, h.parse().ok()?))
        })
        .unwrap_or(iced::Size::new(1480.0, 860.0))
}

/// A token's address in a file: `(section, key)`, the section carrying a `light:`/`dark:` prefix
/// for a variant.
type Addr = (String, String);

/// A message shown as a toast.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Notice {
    text: String,
    error: bool,
}

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
    /// What the gradient controls show: `<section>.<key>.angle` and `.pos<i>`.
    gtexts: BTreeMap<String, String>,
    /// The last thing that happened, as a line of text (also shown as a toast).
    status: String,
    toast: Option<Notice>,
    /// Where the state is kept; `None` keeps nothing (tests).
    state_file: Option<PathBuf>,
    state: State,
    presets: Vec<Preset>,
    /// The data dirs themes are registered in (`theme_dirs()`); the first is written to.
    theme_dirs: Vec<PathBuf>,
    /// A switch waiting for a second click, because there are unsaved changes.
    pending: Option<Target>,
    // -- finding things
    search: String,
    files_filter: String,
    modified_only: bool,
    collapsed: BTreeSet<(String, String)>,
    // -- undo
    undo: Vec<BTreeMap<Addr, String>>,
    redo: Vec<BTreeMap<Addr, String>>,
    /// The token the last edit changed: typing in it again does not add an undo step.
    last_edit: Option<Addr>,
    // -- trying it out
    runner: toys::Runner,
    bins: toys::Bins,
    toys: Vec<toys::Toy>,
    // -- the window
    win: iad::windows::State,
    drawer: bool,
    preview: preview::Preview,
    preview_dark: bool,
    /// On a narrow window the preview replaces the tokens.
    show_preview: bool,
}

#[derive(Debug, Clone)]
enum Msg {
    Noop,
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
    Gradient(String, String, GradEdit),
    Reset(String, String),
    ResetGroup(String, String),
    ToggleGroup(String, String),
    CopyFromBase,
    Search(String),
    FilesFilter(String),
    ModifiedOnly(bool),
    Undo,
    Redo,
    Open,
    Opened(Option<PathBuf>),
    Save,
    SaveAs,
    SavedAs(Option<PathBuf>),
    /// Try the values out in the gallery (`gallery`) or a toy window (its name).
    TryOut(String),
    DismissToast,
    Drawer(bool),
    PreviewDark(bool),
    ShowPreview(bool),
    /// A window button or edge of the chrome.
    Window(iad::windows::Action),
    WindowChanged,
    WindowState(iad::windows::State),
    /// Development screenshots (`iad::dev`).
    Shot(Option<iced::window::Id>),
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
        let bins = toys::Bins::from_env();
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
            gtexts: BTreeMap::new(),
            status: String::new(),
            toast: None,
            presets: Vec::new(),
            theme_dirs: iad::theme_config::theme_dirs(),
            pending: None,
            state_file,
            state,
            search: String::new(),
            files_filter: String::new(),
            modified_only: false,
            collapsed: BTreeSet::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            last_edit: None,
            runner: toys::Runner::default(),
            toys: toys::list_toys(&bins),
            bins,
            win: iad::windows::State::default(),
            drawer: false,
            preview: preview::Preview::build("", false),
            preview_dark: false,
            show_preview: false,
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
        e.refresh_preview();
        // Reopening the last file is not news.
        e.toast = None;
        let boot = if iad::dev::Switches::from_env().screenshot.is_some() {
            iced::window::latest().map(Msg::Shot)
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

    /// Says something: in the status line and as a toast.
    fn notify(&mut self, text: impl Into<String>, error: bool) {
        let text = text.into();
        self.status = text.clone();
        self.toast = (!text.is_empty()).then_some(Notice { text, error });
    }

    // ------------------------------------------------------------ the file

    fn load(&mut self, path: PathBuf) {
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let report = Schema::toolkit().validate(&text);
                self.values = iad::config::parse_keyfile(&text).into_iter().collect();
                self.clean = self.generate();
                let message = if report.errors.is_empty() && report.unknown.is_empty() {
                    format!("Opened {}", file_name(&path))
                } else {
                    format!(
                        "Opened {} with {} invalid and {} unknown values (unknown ones are kept)",
                        file_name(&path),
                        report.errors.len(),
                        report.unknown.len()
                    )
                };
                self.notify(message, false);
                self.remember(&path);
                self.path = Some(path);
                self.pending = None;
                self.clear_history();
                self.rescan();
            }
            Err(e) => self.notify(format!("Could not open {}: {e}", path.display()), true),
        }
    }

    /// An empty theme with no file: Save asks where to write it.
    fn new_theme(&mut self) {
        self.values.clear();
        self.path = None;
        self.state.last_file = None;
        self.clean = self.generate();
        self.pending = None;
        self.clear_history();
        self.notify("New theme. Nothing is set: every token is its default.", false);
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
            self.notify("● unsaved changes: click again to discard", true);
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
    fn save_to(&mut self, path: PathBuf) -> bool {
        let text = self.generate();
        match std::fs::write(&path, &text) {
            Ok(()) => {
                self.clean = text;
                self.notify(format!("Saved {}", file_name(&path)), false);
                self.remember(&path);
                self.path = Some(path);
                self.pending = None;
                self.rescan();
                true
            }
            Err(e) => {
                self.notify(format!("Could not save {}: {e}", path.display()), true);
                false
            }
        }
    }

    /// Save as: a new file, written and then registered for every app (a symlink in the data
    /// dir, unless the file is there already); the editor goes on editing the new file.
    fn save_as_to(&mut self, path: PathBuf) {
        if !self.save_to(path.clone()) {
            return;
        }
        let Some(dir) = self.registry().map(PathBuf::from) else {
            return;
        };
        let name = theme_name(&path);
        if is_registered(&dir, &path) {
            self.notify(format!("Saved {name}: it is registered already"), false);
            return;
        }
        match register(&dir, &path) {
            Ok(_) => self.notify(format!("Saved and registered as {name}"), false),
            Err(e) => self.notify(format!("Saved {name}, but could not register it: {e}"), true),
        }
        self.rescan();
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
        self.refresh_gradients();
    }

    /// The numbers the gradient controls show, from the gradients' current values.
    fn refresh_gradients(&mut self) {
        let mut out = BTreeMap::new();
        for s in &self.specs {
            if s.ty.kind() != iad::theme_config::Kind::Gradient {
                continue;
            }
            let (raw, _) = self.shown(s);
            let Some(g) = iad::theme_config::Gradient::parse(&raw) else {
                continue;
            };
            let at = |f: &str| format!("{}.{}.{f}", s.section, s.key);
            out.insert(at("angle"), trim_number(g.angle));
            for (i, (pos, _)) in g.stops.iter().enumerate() {
                out.insert(at(&format!("pos{i}")), trim_number(pos * 100.0));
            }
        }
        self.gtexts = out;
    }

    fn spec(&self, section: &str, key: &str) -> Option<&Spec> {
        self.specs
            .iter()
            .find(|s| s.section == section && s.key == key)
    }

    /// The preview follows the values and the scheme chosen for it.
    fn refresh_preview(&mut self) {
        let text = self.generate();
        if self.preview.source != text || self.preview.dark != self.preview_dark {
            self.preview = preview::Preview::build(&text, self.preview_dark);
        }
    }

    // --------------------------------------------------------------- undo

    fn clear_history(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.last_edit = None;
    }

    /// Records the values as they are before an edit. Typing in the same token again does not
    /// add a step: undo takes the whole run back.
    fn checkpoint(&mut self, token: Option<Addr>) {
        if token.is_some() && self.last_edit == token {
            return;
        }
        self.undo.push(self.values.clone());
        if self.undo.len() > UNDO_MAX {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.last_edit = token;
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.values, prev));
            self.last_edit = None;
            self.refresh();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.values, next));
            self.last_edit = None;
            self.refresh();
        }
    }

    /// Sets a token's raw text in the current variant when it is valid for the token's type.
    fn set_value(&mut self, section: &str, key: &str, raw: String) -> bool {
        let Some(spec) = self.spec(section, key) else {
            return false;
        };
        if spec.ty.parse(&raw).is_none() {
            return false;
        }
        let at = (self.section_of(section), key.to_string());
        if self.values.get(&at) != Some(&raw) {
            self.checkpoint(Some(at.clone()));
            self.values.insert(at, raw);
        }
        true
    }

    /// Copies what the base sets into the current variant, where the variant sets nothing yet.
    fn copy_from_base(&mut self) -> usize {
        if self.variant == Variant::Base {
            return 0;
        }
        let prefix = self.variant.prefix();
        let missing: Vec<(Addr, String)> = self
            .values
            .iter()
            .filter(|((section, _), _)| !section.contains(':') && section != "theme")
            .map(|((section, key), v)| ((format!("{prefix}{section}"), key.clone()), v.clone()))
            .filter(|(at, _)| !self.values.contains_key(at))
            .collect();
        if missing.is_empty() {
            return 0;
        }
        self.checkpoint(None);
        let n = missing.len();
        self.values.extend(missing);
        self.refresh();
        n
    }

    // -------------------------------------------------------------- gradients

    fn edit_gradient(&mut self, section: &str, key: &str, e: GradEdit) {
        let Some(spec) = self.spec(section, key) else {
            return;
        };
        let (raw, _) = self.shown(spec);
        let start = || {
            iad::theme_config::Gradient::parse("linear-gradient(90deg, #ffffff 0%, #000000 100%)")
                .expect("a valid gradient")
        };
        let mut g = iad::theme_config::Gradient::parse(&raw).unwrap_or_else(start);
        let at = |f: &str| format!("{section}.{key}.{f}");
        match e {
            GradEdit::Start => {
                if iad::theme_config::Gradient::parse(&raw).is_some() {
                    return;
                }
            }
            GradEdit::Angle(text) => {
                self.gtexts.insert(at("angle"), text.clone());
                match text.trim().parse::<f32>() {
                    Ok(a) if a.is_finite() => g.angle = a.rem_euclid(360.0),
                    _ => return,
                }
            }
            GradEdit::Pos(i, text) => {
                self.gtexts.insert(at(&format!("pos{i}")), text.clone());
                let Ok(p) = text.trim().parse::<f32>() else {
                    return;
                };
                if i >= g.stops.len() {
                    return;
                }
                let lo = if i == 0 { 0.0 } else { g.stops[i - 1].0 };
                let hi = g.stops.get(i + 1).map_or(1.0, |s| s.0);
                g.stops[i].0 = (p / 100.0).clamp(lo, hi);
            }
            GradEdit::Color(i, hex) => {
                let Some(Value::Color(c)) = VariableType::Color.parse(&hex) else {
                    return;
                };
                if i < g.stops.len() {
                    g.stops[i].1 = c;
                }
            }
            GradEdit::Add => {
                if g.stops.len() >= 8 || g.stops.len() < 2 {
                    return;
                }
                let n = g.stops.len();
                let (a, b) = (g.stops[n - 2], g.stops[n - 1]);
                let mid = (
                    (a.0 + b.0) / 2.0,
                    iced::Color::from_rgba(
                        (a.1.r + b.1.r) / 2.0,
                        (a.1.g + b.1.g) / 2.0,
                        (a.1.b + b.1.b) / 2.0,
                        (a.1.a + b.1.a) / 2.0,
                    ),
                );
                g.stops.insert(n - 1, mid);
            }
            GradEdit::Remove(i) => {
                if g.stops.len() <= 2 || i >= g.stops.len() {
                    return;
                }
                g.stops.remove(i);
            }
        }
        if self.set_value(section, key, g.to_text()) {
            self.refresh_gradients();
        }
    }

    // ----------------------------------------------------------- registering

    fn registry_op(&mut self, op: Registry) {
        let (Some(dir), Some(file)) = (self.registry().map(PathBuf::from), self.path.clone()) else {
            self.notify("Save the theme to a file first", true);
            return;
        };
        let done = match op {
            Registry::Register => register(&dir, &file).map(|l| format!("Registered {} -> {}", l.display(), file.display())),
            Registry::Unregister => unregister(&dir, &file).map(|n| format!("Unregistered {} symlink(s) of {}", n, file.display())),
            Registry::Default => set_default(&dir, &file).map(|l| format!("{} is the default theme ({})", file.display(), l.display())),
        };
        match done {
            Ok(s) => self.notify(s, false),
            Err(s) => self.notify(s, true),
        }
        self.rescan();
    }

    // ------------------------------------------------------------ try it out

    /// Starts the gallery or a toy window with the values as they are now, from a temp file of
    /// its own: the file being edited is never touched.
    fn try_out(&mut self, kind: &str) {
        let text = self.generate();
        match self.runner.launch(kind, &text, &self.bins) {
            Ok(_) => {
                let what = if kind == "gallery" { "the gallery".to_string() } else { format!("the {kind} window") };
                self.notify(
                    format!("Tried it out: {what} runs with these values from a temp file (nothing saved). Press again to restart it with new ones."),
                    false,
                );
            }
            Err(e) => self.notify(e, true),
        }
    }

    // ------------------------------------------------------------ update

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        let task = self.apply(msg);
        self.refresh_preview();
        task
    }

    fn apply(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Noop => {}
            Msg::Section(s) => {
                self.section = s;
                self.drawer = false;
                self.persist();
            }
            Msg::Variant(v) => {
                self.variant = v;
                // The preview shows the scheme being edited.
                match v {
                    Variant::Light => self.preview_dark = false,
                    Variant::Dark => self.preview_dark = true,
                    Variant::Base => {}
                }
                self.last_edit = None;
                self.refresh();
                self.persist();
            }
            Msg::Pick(path) => {
                self.drawer = false;
                self.pick(Target::File(path));
            }
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
                    if self.set_value(&section, &key, raw) {
                        self.status.clear();
                    } else {
                        self.status = format!("[{section}] {key}: {shown:?} is not valid yet");
                    }
                }
            }
            Msg::Toggle(section, key, on) => {
                self.set_value(&section, &key, on.to_string());
                self.last_edit = None;
                self.refresh();
            }
            Msg::Gradient(section, key, e) => self.edit_gradient(&section, &key, e),
            Msg::Reset(section, key) => {
                let at = (self.section_of(&section), key);
                if self.values.contains_key(&at) {
                    self.checkpoint(None);
                    self.values.remove(&at);
                }
                self.refresh();
            }
            Msg::ResetGroup(section, group) => {
                let keys = self.group_keys(&section, &group);
                if !keys.is_empty() {
                    self.checkpoint(None);
                    for k in &keys {
                        self.values.remove(k);
                    }
                    self.notify(format!("Reset {} token(s) of {group}", keys.len()), false);
                }
                self.refresh();
            }
            Msg::ToggleGroup(section, group) => {
                let key = (section, group);
                if !self.collapsed.remove(&key) {
                    self.collapsed.insert(key);
                }
            }
            Msg::CopyFromBase => {
                let n = self.copy_from_base();
                let message = if n == 0 {
                    "Nothing to copy: the variant sets everything the base sets".to_string()
                } else {
                    format!("Copied {n} value(s) from the base into the {} variant", self.variant.label().to_lowercase())
                };
                self.notify(message, false);
            }
            Msg::Search(s) => self.search = s,
            Msg::FilesFilter(s) => self.files_filter = s,
            Msg::ModifiedOnly(on) => self.modified_only = on,
            Msg::Undo => self.undo(),
            Msg::Redo => self.redo(),
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
            Msg::Opened(Some(path)) => self.pick(Target::File(path)),
            Msg::Opened(None) | Msg::SavedAs(None) => {}
            Msg::Save => match self.path.clone() {
                Some(p) => {
                    self.save_to(p);
                }
                None => return self.apply(Msg::SaveAs),
            },
            Msg::SaveAs => {
                let dir = self.registry().map(PathBuf::from);
                return Task::perform(
                    async move {
                        let mut dialog = rfd::AsyncFileDialog::new()
                            .add_filter("Theme", &["conf"])
                            .set_file_name("theme.conf");
                        if let Some(d) = dir.filter(|d| d.is_dir()) {
                            dialog = dialog.set_directory(d);
                        }
                        dialog.save_file().await.map(|f| f.path().to_path_buf())
                    },
                    Msg::SavedAs,
                );
            }
            Msg::SavedAs(Some(path)) => self.save_as_to(path),
            Msg::TryOut(kind) => self.try_out(&kind),
            Msg::DismissToast => self.toast = None,
            Msg::Drawer(open) => self.drawer = open,
            Msg::PreviewDark(dark) => self.preview_dark = dark,
            Msg::ShowPreview(on) => self.show_preview = on,
            Msg::Window(a) => return iced::window::oldest().and_then(move |id| a.perform(id)),
            Msg::WindowChanged => return iad::windows::refresh().map(Msg::WindowState),
            Msg::WindowState(state) => self.win = state,
            Msg::Shot(Some(id)) => {
                if let Some(shot) = iad::dev::Switches::from_env().capture(id) {
                    return shot.map(Msg::Captured);
                }
            }
            Msg::Shot(None) => {}
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
        iad::windows::changes().map(|_| Msg::WindowChanged)
    }
}

/// A file's name, for messages.
fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned())
}

/// A number as short as it reads: `90`, `12.5`.
fn trim_number(n: f32) -> String {
    let s = format!("{n:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
