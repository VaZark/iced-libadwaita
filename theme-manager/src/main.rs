//! The theme manager: edits and saves theme files with a control for each token by its type, ships
//! the default set of themes and chooses the global theme every app picks up.
//!
//! `theme-manager [FILE]` opens the manager; `theme-manager gallery [ARGS]` runs the gallery.
//! The manager's own look is `themes/libadwaita.ron`, so editing never changes the tool itself.

mod bundled;
mod files;
mod page;
#[cfg(test)]
mod preview;
mod tokens;
mod view;

#[cfg(test)]
mod tests;

use files::*;
use iad::theme_config::{Inheritance, Schema, Spec, Value, VariableType};
use iad::widgets as w;
use iced::Task;
use iced_adaptive_design as iad;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use tokens::GradEdit;

/// The editor's own base look: libadwaita's, bundled in the binary. It never depends on the file
/// being edited, the desktop, or an installed theme.
const DEFAULT_BASE_THEME: &str = include_str!("../../themes/libadwaita.ron");

/// Laid over the base: a neutral primary color, since the editor has no blue buttons.
const NEUTRAL_PRIMARY: &str = r##"{
    "menu": {
        "popover-max-height": 560,
    },
    "colors": {
        "primary": "#3d3d44",
        "primary-fg": "#ffffff",
    },
    "dark:colors": {
        "primary": "#d0d0d8",
        "primary-fg": "#1d1d20",
    },
}
"##;
/// DECISION (user, 2026-11-16): the app id stays `org.iad.ThemeEditor`.
const APP_ID: &str = "org.iad.ThemeEditor";
/// How many edits undo goes back.
const UNDO_MAX: usize = 100;

/// The editor's own look: the bundled base with the neutral primary over it, and nothing from
/// the desktop or the files.
fn own_theme() -> iad::theme_config::ThemeConfig {
    iad::theme_config::ThemeConfig::builder(APP_ID)
        .files(false)
        .inheritance(Inheritance::OptOut)
        .base_theme(DEFAULT_BASE_THEME)
        .app_text(NEUTRAL_PRIMARY)
        .build()
}

fn main() -> iced::Result {
    // DECISION (user, 2026-11-16): the gallery is part of this app: `theme-manager gallery` runs it
    // and the window can start it too. It has no desktop entry of its own.
    if std::env::args_os().nth(1).is_some_and(|a| a == "gallery") {
        return iced_adaptive_design_gallery::run();
    }
    let own = own_theme();
    iad::config::install(iad::config::Config::default().with_theme_config(&own));
    iad::dev::Switches::from_env().apply();
    iced::application(Editor::new, Editor::update, Editor::view)
        .title(Editor::title)
        .theme(Editor::theme)
        .window(iced::window::Settings {
            size: window_size(),
            // Token rows are one line from this width (see `page`); below it some would wrap.
            min_size: Some(iced::Size::new(page::MIN_WIDTH, 360.0)),
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

/// What follows a Save as that was asked for by another action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AfterSave {
    /// Make the saved file the global theme.
    Global,
}

/// A message shown as a toast.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Notice {
    text: String,
    error: bool,
}

/// The values of a theme file's text, as the RON each key holds (unknown keys and invalid
/// values included). Text that is not RON has none.
fn values_of(text: &str) -> BTreeMap<Addr, ron::Value> {
    iad::theme_config::Document::parse(text)
        .map(|doc| {
            doc.entries()
                .map(|(section, key, v)| ((section.to_string(), key.to_string()), v.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// What a token's control shows for the RON `v` the file holds: the value as typed text
/// (`24px`, `#3584e4`), or the RON itself when the value is not valid for the token.
fn text_of(spec: &Spec, v: &ron::Value) -> String {
    spec.ty
        .from_ron(v)
        .map(|typed| typed.to_text())
        .unwrap_or_else(|_| ron::ser::to_string(v).unwrap_or_default())
}

struct Editor {
    theme: iad::Theme,
    schema: Schema,
    specs: Vec<Spec>,
    sections: Vec<String>,
    section: String,
    variant: Variant,
    path: Option<PathBuf>,
    /// Every key of the file as written, including ones the schema does not know.
    values: BTreeMap<Addr, ron::Value>,
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
    overrides_only: bool,
    collapsed: BTreeSet<(String, String)>,
    // -- undo
    undo: Vec<BTreeMap<Addr, ron::Value>>,
    redo: Vec<BTreeMap<Addr, ron::Value>>,
    /// The token the last edit changed: typing in it again does not add an undo step.
    last_edit: Option<Addr>,
    /// The bundled theme the values were opened from while they have no file of their own.
    source: Option<&'static str>,
    /// The running gallery started from the window, and the program that starts it.
    gallery: Option<std::process::Child>,
    /// What to do once the Save as the user was sent to has written the file.
    after_save: Option<AfterSave>,
    /// `dirty()` for the values it was last computed for: regenerating the file text is too slow
    /// to repeat for every widget on every frame (a window resize draws many).
    /// How far the token page is scrolled: the blocks near it are the ones built.
    tokens_view: iad::layouts::virtualize::Viewport,
    dirty_cache: std::cell::Cell<Option<(u64, bool)>>,
    /// The color scheme the gallery starts in: the desktop's (System), Light or Dark.
    launch_mode: w::ThemeMode,
    exe: Option<PathBuf>,
    // -- the window
    drawer: bool,
    theme_picker: bool,
    /// Screenshots only (`THEME_EDITOR_MENU`): the main menu starts open.
    menu_open: bool,
}

#[derive(Debug, Clone)]
enum Msg {
    #[cfg(test)]
    Noop,
    Pick(PathBuf),
    New,
    SetGlobal,
    /// The token page was scrolled or resized.
    TokensScrolled(iad::layouts::virtualize::Viewport),
    /// Run the gallery with the values being edited.
    Gallery,
    /// The scheme the gallery starts in.
    LaunchMode(w::ThemeMode),
    ClearGlobal,
    PickBundled(&'static str),
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
    OverridesOnly(bool),
    Undo,
    Redo,
    /// Drop every unsaved change (undoable).
    Discard,
    Open,
    Opened(Option<PathBuf>),
    Save,
    SaveAs,
    SavedAs(Option<PathBuf>),
    DismissToast,
    Drawer(bool),
    ThemePicker(bool),
    Rule(String, usize, Option<(Option<String>, Option<String>)>),
    /// A window button or edge of the chrome.
    Window(iad::windows::Action),
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
        let mut e = Self {
            theme,
            schema,
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
            overrides_only: false,
            collapsed: BTreeSet::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            last_edit: None,
            source: None,
            gallery: None,
            after_save: None,
            tokens_view: iad::layouts::virtualize::Viewport::new(page::FIRST_VIEW),
            dirty_cache: std::cell::Cell::new(None),
            launch_mode: w::ThemeMode::System,
            exe: std::env::current_exe().ok(),
            drawer: false,
            theme_picker: false,
            menu_open: false,
        };
        if let Some(s) = e
            .state
            .section
            .clone()
            .filter(|s| e.section_choices().any(|id| id == s))
        {
            e.section = s;
        }
        if let Some(v) = e.state.variant {
            e.variant = v;
        }
        let open = cli_file.or_else(|| e.state.last_file.clone().filter(|p| p.is_file()));
        if let Some(path) = open {
            e.load(path);
        }
        if let Some(s) = std::env::var("THEME_EDITOR_SECTION")
            .ok()
            .filter(|s| e.section_choices().any(|id| id == s))
        {
            e.section = s;
        }
        if e.path.is_none() {
            e.clean = e.generate();
        }
        e.rescan();
        e.refresh();
        if iad::dev::Switches::from_env().screenshot.is_some() {
            e.theme_picker = std::env::var_os("THEME_EDITOR_PICKER").is_some();
            e.menu_open = std::env::var_os("THEME_EDITOR_MENU").is_some();
        }
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
        format!("{}{name} - Theme manager", self.dot())
    }

    fn theme(&self) -> iad::Theme {
        self.theme.clone()
    }

    fn dirty(&self) -> bool {
        use std::hash::{Hash, Hasher};
        // The text is a function of the values, so the values and the saved text tell whether
        // the answer is still the one computed.
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.values.hash(&mut hasher);
        self.clean.hash(&mut hasher);
        let key = hasher.finish();
        match self.dirty_cache.get() {
            Some((k, dirty)) if k == key => dirty,
            _ => {
                let dirty = self.generate() != self.clean;
                self.dirty_cache.set(Some((key, dirty)));
                dirty
            }
        }
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
                self.values = values_of(&text);
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
                self.source = None;
                self.pending = None;
                self.clear_history();
                self.rescan();
            }
            Err(e) => self.notify(format!("Could not open {}: {e}", path.display()), true),
        }
    }

    /// A copy of a bundled theme with no file of its own: Save asks where to write it, so the
    /// bundled text itself is never changed.
    fn load_bundled(&mut self, key: &'static str) {
        let Some(text) = bundled::text(key) else {
            return;
        };
        self.values = values_of(text);
        self.path = None;
        self.state.last_file = None;
        self.source = Some(key);
        self.clean = self.generate();
        self.pending = None;
        self.clear_history();
        self.notify(
            format!(
                "Opened the bundled {} theme. Save as writes your own copy.",
                bundled::name_of(text, key)
            ),
            false,
        );
        self.persist();
    }

    /// An empty theme with no file: Save asks where to write it.
    fn new_theme(&mut self) {
        self.values.clear();
        self.path = None;
        self.source = None;
        self.state.last_file = None;
        self.clean = self.generate();
        self.pending = None;
        self.clear_history();
        self.notify(
            "New theme. Nothing is set: every token is its default.",
            false,
        );
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
        if let (Target::File(p), Some(open)) = (&target, &self.path)
            && canon(p) == canon(open)
        {
            return;
        }
        if matches!(target, Target::Bundled(k) if self.path.is_none() && self.source == Some(k))
            && !self.dirty()
        {
            return;
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
            Target::Bundled(key) => self.load_bundled(key),
        }
        self.refresh();
    }

    /// The file for the values set: schema sections in order, a section's variants after its
    /// base, unknown keys kept at the end of their section. Values the schema understands are
    /// written the way a theme is written by hand; anything else is kept as it was read.
    fn generate(&self) -> String {
        let mut doc = iad::theme_config::Document::default();
        for ((section, key), v) in &self.values {
            doc.set(section, key, v.clone());
        }
        doc.to_text(
            "An iced-adaptive-design theme. Only the tokens set here take effect; everything else is\nthe toolkit default. \"light:*\" and \"dark:*\" sections win over the base on that color scheme.",
            &self.schema,
        )
    }

    /// Writes the values to `path`: creates the file, or replaces the one there, through a
    /// symlink to what it points at. This is the only place the editor writes a theme.
    fn save_to(&mut self, path: PathBuf) -> bool {
        let text = self.generate();
        if self.values.iter().any(|((section, key), v)| {
            key == "breakpoint-rules"
                && !matches!(v, ron::Value::String(raw)
                    if tokens::valid_rules(section.rsplit(':').next().unwrap_or(section), raw))
        }) {
            self.notify("Cannot save invalid breakpoint rules", true);
            return false;
        }
        match std::fs::write(&path, &text) {
            Ok(()) => {
                self.clean = text;
                // Edits after saving must undo back to this saved version, even in the same field.
                self.last_edit = None;
                self.notify(format!("Saved {}", file_name(&path)), false);
                self.remember(&path);
                self.path = Some(path);
                self.source = None;
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

    /// Save as: a new file, written; the editor goes on editing it.
    fn save_as_to(&mut self, path: PathBuf) {
        self.save_to(path);
    }

    // ------------------------------------------------------------- tokens

    fn section_of(&self, base: &str) -> String {
        format!("{}{base}", self.variant.prefix())
    }

    /// The raw text a token shows now: the variant's own value, else the base's, else the default.
    fn shown(&self, spec: &Spec) -> (String, bool) {
        let here = (self.section_of(&spec.section), spec.key.clone());
        if let Some(v) = self.values.get(&here) {
            return (text_of(spec, v), true);
        }
        if let Some(v) = self.values.get(&(spec.section.clone(), spec.key.clone())) {
            return (text_of(spec, v), false);
        }
        (
            spec.default
                .as_ref()
                .map(Value::to_text)
                .unwrap_or_default(),
            false,
        )
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
        let Some(typed) = spec.ty.parse(&raw) else {
            return false;
        };
        if key == "breakpoint-rules" && !tokens::valid_rules(section, &raw) {
            return false;
        }
        let value = typed.to_ron();
        let at = (self.section_of(section), key.to_string());
        // The same value written another way (`9` and `9.0`) is not a change.
        let unchanged = self
            .values
            .get(&at)
            .and_then(|v| spec.ty.from_ron(v).ok())
            .is_some_and(|current| current == typed);
        if !unchanged {
            self.checkpoint(Some(at.clone()));
            self.values.insert(at, value);
        }
        true
    }

    /// Copies what the base sets into the current variant, where the variant sets nothing yet.
    fn copy_from_base(&mut self) -> usize {
        if self.variant == Variant::Base {
            return 0;
        }
        let prefix = self.variant.prefix();
        let missing: Vec<(Addr, ron::Value)> = self
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

    // ------------------------------------------------------------ the global theme

    /// Whether the open theme is the one `default.ron` leads to (a bundled theme counts when that
    /// file holds its text).
    fn is_global(&self) -> bool {
        let Some(target) = self.registry().and_then(default_target) else {
            return false;
        };
        match (&self.path, self.source) {
            (Some(file), _) => canon(file) == target,
            (None, Some(key)) => {
                std::fs::read_to_string(target).ok().as_deref() == bundled::text(key)
            }
            (None, None) => false,
        }
    }

    /// Sets the open theme as the global one: `default.ron` in the standard theme directory, which
    /// every iad app looks at, becomes a symlink to the theme's file. Every theme is a file of its
    /// own, so a bundled one is copied into the directory first. The global theme is the saved file,
    /// so unsaved changes are saved first; a theme with no file of its own (new, or edited from a
    /// bundled one) opens Save as, and becomes global when that has written it.
    fn set_global(&mut self) -> Task<Msg> {
        let Some(dir) = self.registry().map(PathBuf::from) else {
            self.notify("There is no theme directory to install into", true);
            return Task::none();
        };
        if self.path.is_none() && (self.source.is_none() || self.dirty()) {
            self.after_save = Some(AfterSave::Global);
            self.notify(
                "Choose where to save the theme: it becomes the global theme",
                false,
            );
            return self.apply(Msg::SaveAs);
        }
        if self.dirty()
            && let Some(file) = self.path.clone()
            && !self.save_to(file)
        {
            return Task::none();
        }
        let done = match (self.path.clone(), self.source) {
            (Some(file), _) => set_default(&dir, &file).map(|linked| (file, linked)),
            (None, Some(key)) => set_global_bundled(&dir, key)
                .map(|linked| (PathBuf::from(format!("{key}.ron")), linked)),
            (None, None) => Err("Save the theme to a file first".to_string()),
        };
        match done {
            Ok((file, linked)) => {
                let name = file_name(&file);
                match linked.kept {
                    // The open file was the regular default.ron: it lives under its new name.
                    Some(kept) if file == linked.link => {
                        self.notify(
                            format!("{name} is now the global theme (kept as {})", kept.display()),
                            false,
                        );
                        self.path = Some(kept);
                    }
                    Some(kept) => self.notify(
                        format!(
                            "{name} is now the global theme; the default.ron that was there is kept as {}",
                            kept.display()
                        ),
                        false,
                    ),
                    None => self.notify(format!("{name} is now the global theme"), false),
                }
            }
            Err(e) => self.notify(e, true),
        }
        self.rescan();
        Task::none()
    }

    /// Removes the global theme; apps go back to the bare look (or their own base theme).
    fn clear_global(&mut self) {
        let Some(dir) = self.registry().map(PathBuf::from) else {
            return;
        };
        match clear_global(&dir) {
            Ok(()) => self.notify("No global theme: apps use their own base look", false),
            Err(e) => self.notify(e, true),
        }
        self.rescan();
    }

    // ------------------------------------------------------------ the gallery

    /// Runs `theme-manager gallery` with the values as they are now (unsaved edits included),
    /// through `IAD_THEME` pointing at a temp file of its own: the open file is never touched. A
    /// gallery started before is closed first.
    fn open_gallery(&mut self) {
        let Some(exe) = self.exe.clone() else {
            self.notify("Cannot find the program to start the gallery", true);
            return;
        };
        let file =
            std::env::temp_dir().join(format!("iad-theme-manager-{}.ron", std::process::id()));
        if let Err(e) = std::fs::write(&file, self.generate()) {
            self.notify(format!("Could not write {}: {e}", file.display()), true);
            return;
        }
        if let Some(mut old) = self.gallery.take() {
            let _ = old.kill();
            let _ = old.wait();
        }
        let mut command = std::process::Command::new(exe);
        command.arg("gallery").env("IAD_THEME", &file);
        match self.launch_mode.scheme_override() {
            Some(scheme) if scheme.is_dark() => command.env("GALLERY_SCHEME", "dark"),
            Some(_) => command.env("GALLERY_SCHEME", "light"),
            None => command.env_remove("GALLERY_SCHEME"),
        };
        match command.spawn() {
            Ok(child) => {
                self.gallery = Some(child);
                self.notify(
                    "The gallery shows these values (nothing saved). Press again to restart it with new ones.",
                    false,
                );
            }
            Err(e) => self.notify(format!("Could not start the gallery: {e}"), true),
        }
    }

    // ------------------------------------------------------------ update

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        let reset_scroll = matches!(
            &msg,
            Msg::Section(_)
                | Msg::Variant(_)
                | Msg::Search(_)
                | Msg::ModifiedOnly(_)
                | Msg::OverridesOnly(_)
        );
        let task = self.apply(msg);
        if reset_scroll {
            self.scroll_to_top();
            Task::batch([
                task,
                iced::widget::operation::snap_to(
                    iced::widget::Id::new(tokens::TOKENS_SCROLL),
                    iced::widget::scrollable::RelativeOffset { x: 0.0, y: 0.0 },
                ),
            ])
        } else {
            task
        }
    }

    fn apply(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            #[cfg(test)]
            Msg::Noop => {}
            Msg::ThemePicker(open) => self.theme_picker = open,
            Msg::Rule(section, index, edit) => self.edit_rule(section, index, edit),
            Msg::Section(s) => {
                if !self.section_choices().any(|id| id == s) {
                    return Task::none();
                }
                self.section = s;
                self.drawer = false;
                self.persist();
            }
            Msg::Variant(v) => {
                self.variant = v;
                if v == Variant::Base {
                    self.overrides_only = false;
                }
                self.last_edit = None;
                self.refresh();
                self.persist();
            }
            Msg::Pick(path) => {
                self.pick(Target::File(path));
                self.theme_picker = self.pending.is_some();
            }
            Msg::New => {
                self.pick(Target::New);
                self.theme_picker = self.pending.is_some();
            }
            Msg::SetGlobal => return self.set_global(),
            Msg::Gallery => self.open_gallery(),
            Msg::TokensScrolled(view) => self.scrolled(view.offset, view.height),
            Msg::LaunchMode(mode) => self.launch_mode = mode,
            Msg::ClearGlobal => self.clear_global(),
            Msg::PickBundled(key) => {
                self.pick(Target::Bundled(key));
                self.theme_picker = self.pending.is_some();
            }
            Msg::Refresh => self.rescan(),
            Msg::AddFolder => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Add a folder of theme files")
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
                self.texts
                    .insert((section.clone(), key.clone()), shown.clone());
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
                    format!(
                        "Copied {n} value(s) from the base into the {} variant",
                        self.variant.label().to_lowercase()
                    )
                };
                self.notify(message, false);
            }
            Msg::Search(s) => self.search = s,
            Msg::FilesFilter(s) => self.files_filter = s,
            Msg::ModifiedOnly(on) => self.modified_only = on,
            Msg::OverridesOnly(on) => self.overrides_only = on && self.variant != Variant::Base,
            Msg::Undo => self.undo(),
            Msg::Redo => self.redo(),
            Msg::Discard => {
                let saved = values_of(&self.clean);
                if saved != self.values {
                    self.checkpoint(None);
                    self.values = saved;
                    self.refresh();
                    self.notify("Unsaved changes dropped", false);
                }
            }
            Msg::Open => {
                self.theme_picker = false;
                let directory = self
                    .path
                    .as_ref()
                    .and_then(|p| p.parent())
                    .map(PathBuf::from);
                return Task::perform(
                    async move {
                        let mut dialog = rfd::AsyncFileDialog::new()
                            .set_title("Open a theme")
                            .add_filter("Theme", &["ron"]);
                        if let Some(dir) = directory {
                            dialog = dialog.set_directory(dir);
                        }
                        dialog.pick_file().await.map(|f| f.path().to_path_buf())
                    },
                    Msg::Opened,
                );
            }
            Msg::Opened(Some(path)) => self.pick(Target::File(path)),
            Msg::Opened(None) => {}
            Msg::SavedAs(None) => self.after_save = None,
            Msg::Save => match self.path.clone() {
                Some(p) => {
                    self.save_to(p);
                }
                None => return self.apply(Msg::SaveAs),
            },
            Msg::SaveAs => {
                let dir = self
                    .path
                    .as_ref()
                    .and_then(|p| p.parent())
                    .map(PathBuf::from)
                    .or_else(|| self.registry().map(PathBuf::from));
                // A new theme is offered the standard directory, which has to exist to be offered.
                if self.path.is_none()
                    && let Some(dir) = &dir
                {
                    let _ = std::fs::create_dir_all(dir);
                }
                let name = self
                    .path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "theme.ron".into());
                return Task::perform(
                    async move {
                        let mut dialog = rfd::AsyncFileDialog::new()
                            .set_title("Save theme as")
                            .add_filter("Theme", &["conf"])
                            .set_file_name(name);
                        if let Some(d) = dir.filter(|d| d.is_dir()) {
                            dialog = dialog.set_directory(d);
                        }
                        dialog.save_file().await.map(|f| f.path().to_path_buf())
                    },
                    Msg::SavedAs,
                );
            }
            Msg::SavedAs(Some(mut path)) => {
                if path.extension().is_none() {
                    path.set_extension("conf");
                }
                self.save_as_to(path);
                if self.after_save.take() == Some(AfterSave::Global) && !self.dirty() {
                    return self.set_global();
                }
            }
            Msg::DismissToast => self.toast = None,
            Msg::Drawer(open) => self.drawer = open,
            Msg::Window(a) => return iced::window::oldest().and_then(move |id| a.perform(id)),
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
}

/// A file's name, for messages.
fn file_name(path: &std::path::Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// A number as short as it reads: `90`, `12.5`.
fn trim_number(n: f32) -> String {
    let s = format!("{n:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
