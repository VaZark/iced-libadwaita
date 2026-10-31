//! Keyboard shortcuts.
//!
//! The toolkit handles shortcuts by default, with no subscription or `update` plumbing:
//!
//! * a [`MenuButton`](crate::widget::MenuButton) registers its menu's accelerators itself, so
//!   the shortcuts a menu shows work while the button is in the view;
//! * [`Standard`] lists GNOME's usual ones (quit, fullscreen, help, preferences, shortcuts);
//! * [`Shortcuts::scope`] (or `widget::shortcuts`) listens for any others around any content.
//!
//! The accelerator labels shown in menus (`MenuItem::accel("Ctrl+,")`) and in shortcut
//! dialogs are only text. This module makes them work: [`Accel`] parses the same strings,
//! [`key_presses`] is a subscription of key presses, and [`Shortcuts`] maps a press to the
//! app's message.
//!
//! ```ignore
//! // In `view`: wrap the window once. The messages arrive in `update` like any other.
//! Standard::new().quit(Msg::Quit).fullscreen(Msg::Fullscreen).into_shortcuts()
//!     .bind("Esc", Msg::Escape)
//!     .scope(window_content)
//! ```
//!
//! A focused widget has precedence: when a text input or an open popover handled the key, the
//! shortcut does not fire. [`key_presses`] and [`Shortcuts::handle`] remain for apps that prefer
//! to route key presses themselves.
//!
//! Accelerators are written like GNOME shows them: modifiers joined with `+`, then the key:
//! `Ctrl+,`, `Ctrl+Shift+Z`, `Alt+Left`, `F1`, `Esc`.

use crate::widget::Menu;
use iced::keyboard::{self, key::Named, Key, Modifiers};
use iced::Subscription;

/// The key of an [`Accel`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccelKey {
    Named(Named),
    /// A character, lowercased (`","`, `"z"`, `"?"`).
    Char(String),
}

/// A parsed accelerator such as `Ctrl+Shift+Z`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accel {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub logo: bool,
    pub key: AccelKey,
}

fn named(name: &str) -> Option<Named> {
    Some(match name.to_ascii_lowercase().as_str() {
        "esc" | "escape" => Named::Escape,
        "enter" | "return" => Named::Enter,
        "tab" => Named::Tab,
        "space" => Named::Space,
        "delete" | "del" => Named::Delete,
        "backspace" => Named::Backspace,
        "up" => Named::ArrowUp,
        "down" => Named::ArrowDown,
        "left" => Named::ArrowLeft,
        "right" => Named::ArrowRight,
        "home" => Named::Home,
        "end" => Named::End,
        "pageup" => Named::PageUp,
        "pagedown" => Named::PageDown,
        "f1" => Named::F1,
        "f2" => Named::F2,
        "f3" => Named::F3,
        "f4" => Named::F4,
        "f5" => Named::F5,
        "f6" => Named::F6,
        "f7" => Named::F7,
        "f8" => Named::F8,
        "f9" => Named::F9,
        "f10" => Named::F10,
        "f11" => Named::F11,
        "f12" => Named::F12,
        _ => return None,
    })
}

impl Accel {
    /// Parses `"Ctrl+Shift+Z"`. A trailing `+` is the plus key (`"Ctrl++"`). `None` if empty or
    /// a modifier name is unknown.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let (mods, key) = match text.strip_suffix('+') {
            // "Ctrl++": the key is "+".
            Some(rest) if text.len() > 1 => (rest.strip_suffix('+').unwrap_or(rest), "+"),
            _ => match text.rsplit_once('+') {
                Some((m, k)) => (m, k),
                None => ("", text),
            },
        };
        if key.is_empty() {
            return None;
        }
        let mut a = Accel { ctrl: false, shift: false, alt: false, logo: false, key: AccelKey::Char(String::new()) };
        for m in mods.split('+').filter(|m| !m.is_empty()) {
            match m.to_ascii_lowercase().as_str() {
                "ctrl" | "control" | "primary" => a.ctrl = true,
                "shift" => a.shift = true,
                "alt" => a.alt = true,
                "super" | "meta" | "logo" | "cmd" => a.logo = true,
                _ => return None,
            }
        }
        a.key = match named(key) {
            Some(n) => AccelKey::Named(n),
            None if key.chars().count() == 1 => AccelKey::Char(key.to_lowercase()),
            None => return None,
        };
        Some(a)
    }

    /// Whether this accelerator is the given key press.
    ///
    /// `key` is the key **without** modifiers and `modified_key` the character the press
    /// produces (iced's `key` and `modified_key`). Ctrl, Alt and Super must match exactly.
    ///
    /// * Named keys and letters: the key matches and Shift must match.
    /// * Other characters (`,`, `?`, `1`): they are written as the character they type. With
    ///   Shift held, that is the *produced* character, so `Ctrl+?` matches `Ctrl+Shift+/` on a
    ///   US layout and `Ctrl+Shift+,` on AZERTY, wherever `?` is typed with Shift; and `Ctrl+,`
    ///   then does not also fire. Without Shift it is the key itself. A `Shift+` written
    ///   explicitly means the unmodified key with Shift held.
    pub fn matches(&self, key: &Key, modified_key: &Key, modifiers: Modifiers) -> bool {
        if self.ctrl != modifiers.control() || self.alt != modifiers.alt() || self.logo != modifiers.logo() {
            return false;
        }
        let same = |key: &Key, a: &str| matches!(key, Key::Character(b) if a.eq_ignore_ascii_case(b.as_str()));
        match &self.key {
            AccelKey::Named(a) => matches!(key, Key::Named(b) if a == b) && self.shift == modifiers.shift(),
            AccelKey::Char(a) if a.chars().all(char::is_alphabetic) => same(key, a) && self.shift == modifiers.shift(),
            AccelKey::Char(a) if self.shift => modifiers.shift() && same(key, a),
            AccelKey::Char(a) if modifiers.shift() => same(modified_key, a),
            AccelKey::Char(a) => same(key, a),
        }
    }

    /// A shortcut that types a character (no Ctrl, Alt or Super): it must not fire while the user
    /// is typing into a focused widget.
    fn is_plain_character(&self) -> bool {
        matches!(self.key, AccelKey::Char(_)) && !self.ctrl && !self.alt && !self.logo
    }
}

/// How menus show accelerator hints (`[shortcuts] hints`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Hints {
    /// Show them where a keyboard is likely (desktop menus), hide them in touch presentations
    /// (the full-page menu of mobile mode).
    #[default]
    Auto,
    Always,
    Never,
}

/// Whether keyboard shortcuts are active at all (`[shortcuts] enabled`).
pub fn enabled() -> bool {
    crate::config::current().shortcuts_enabled
}

/// The accelerator text a menu item may show, or `None` when the shortcut is **not available**:
/// shortcuts are off, hints are hidden for this presentation (`touch`), the item is disabled, or
/// the accelerator does not parse (so it could never fire). A widget only exposes a shortcut it
/// can honor.
pub fn hint(accel: Option<&str>, item_enabled: bool, touch: bool) -> Option<String> {
    let accel = accel?;
    let shown = match crate::config::current().shortcut_hints {
        Hints::Always => true,
        Hints::Never => false,
        Hints::Auto => !touch,
    };
    (enabled() && shown && item_enabled && Accel::parse(accel).is_some()).then(|| accel.to_string())
}

/// A key press, from [`key_presses`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPress {
    /// The key without modifiers.
    pub key: Key,
    /// The character the press produces with Shift applied (`?` for Shift+`,` on AZERTY).
    pub modified_key: Key,
    pub modifiers: Modifiers,
    /// A focused widget (a text input) already handled the key.
    pub captured: bool,
}

/// A subscription of every key press, to feed [`Shortcuts::handle`].
pub fn key_presses() -> Subscription<KeyPress> {
    iced::event::listen_with(|event, status, _| match event {
        iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modified_key, modifiers, .. }) => Some(KeyPress {
            key,
            modified_key,
            modifiers,
            captured: status == iced::event::Status::Captured,
        }),
        _ => None,
    })
}

/// Accelerators bound to messages.
pub struct Shortcuts<M> {
    bindings: Vec<(Accel, M)>,
}

impl<M> Default for Shortcuts<M> {
    fn default() -> Self {
        Self { bindings: Vec::new() }
    }
}

impl<M: Clone> Shortcuts<M> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds `accel` (e.g. `"Ctrl+Q"`) to `message`. An unparsable accelerator is ignored.
    pub fn bind(mut self, accel: &str, message: M) -> Self {
        if let Some(a) = Accel::parse(accel) {
            self.bindings.push((a, message));
        }
        self
    }

    /// Binds every enabled item of `menu` that has an accelerator to that item's message, so
    /// the shortcuts shown in the menu work. Earlier bindings win.
    pub fn from_menu(mut self, menu: &Menu<M>) -> Self {
        for (accel, message) in menu.accelerators() {
            if let Some(a) = Accel::parse(&accel) {
                self.bindings.push((a, message));
            }
        }
        self
    }

    /// Listens for these shortcuts around `content` (the toolkit's default behavior): the bound
    /// message is published when its keys are pressed, with no subscription needed.
    pub fn scope<'a>(self, content: impl Into<iced::Element<'a, M, crate::Theme>>) -> iced::Element<'a, M, crate::Theme>
    where
        M: 'a,
    {
        crate::widget::shortcuts(content, self)
    }

    /// The message bound to this key press, if any. Always `None` when shortcuts are turned off
    /// (`[shortcuts] enabled=false`).
    pub fn handle(&self, press: &KeyPress) -> Option<M> {
        if !enabled() {
            return None;
        }
        self.bindings
            .iter()
            .find(|(a, _)| a.matches(&press.key, &press.modified_key, press.modifiers) && !(press.captured && a.is_plain_character()))
            .map(|(_, m)| m.clone())
    }
}

/// GNOME's standard shortcuts. Set the message for each the app supports; the rest stay unbound.
///
/// | Action | Accelerator |
/// |---|---|
/// | Quit | `Ctrl+Q` |
/// | Close window | `Ctrl+W` |
/// | Fullscreen | `F11` |
/// | Preferences | `Ctrl+,` |
/// | Help | `F1` |
/// | Keyboard shortcuts | `Ctrl+?` |
///
/// Do not bind the same accelerator again in a menu: earlier bindings win, and a menu's own
/// accelerators are registered by the menu button.
pub struct Standard<M> {
    quit: Option<M>,
    close_window: Option<M>,
    fullscreen: Option<M>,
    preferences: Option<M>,
    help: Option<M>,
    shortcuts: Option<M>,
}

impl<M> Default for Standard<M> {
    fn default() -> Self {
        Self { quit: None, close_window: None, fullscreen: None, preferences: None, help: None, shortcuts: None }
    }
}

impl<M: Clone> Standard<M> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn quit(mut self, m: M) -> Self {
        self.quit = Some(m);
        self
    }
    pub fn close_window(mut self, m: M) -> Self {
        self.close_window = Some(m);
        self
    }
    pub fn fullscreen(mut self, m: M) -> Self {
        self.fullscreen = Some(m);
        self
    }
    pub fn preferences(mut self, m: M) -> Self {
        self.preferences = Some(m);
        self
    }
    pub fn help(mut self, m: M) -> Self {
        self.help = Some(m);
        self
    }
    /// Opening the keyboard shortcuts dialog.
    pub fn shortcuts(mut self, m: M) -> Self {
        self.shortcuts = Some(m);
        self
    }

    fn entries(&self) -> [(&'static str, &'static str, &Option<M>); 6] {
        [
            ("Ctrl+Q", "Quit", &self.quit),
            ("Ctrl+W", "Close Window", &self.close_window),
            ("F11", "Fullscreen", &self.fullscreen),
            ("Ctrl+,", "Preferences", &self.preferences),
            ("F1", "Help", &self.help),
            ("Ctrl+?", "Keyboard Shortcuts", &self.shortcuts),
        ]
    }

    /// The bindings, ready to extend with [`Shortcuts::bind`] and to [`scope`](Shortcuts::scope).
    pub fn into_shortcuts(self) -> Shortcuts<M> {
        let mut s = Shortcuts::new();
        for (accel, _, message) in self.entries() {
            if let Some(m) = message {
                s = s.bind(accel, m.clone());
            }
        }
        s
    }

    /// The bound standard shortcuts as `(label, accelerator)` pairs, for a shortcuts dialog.
    pub fn labels(&self) -> Vec<(String, String)> {
        self.entries()
            .into_iter()
            .filter(|(_, _, m)| m.is_some())
            .map(|(accel, label, _)| (label.to_string(), accel.to_string()))
            .collect()
    }
}

impl<M: Clone> From<Standard<M>> for Shortcuts<M> {
    fn from(s: Standard<M>) -> Self {
        s.into_shortcuts()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::keyboard::Key;

    fn ch(c: &str) -> Key {
        Key::Character(c.into())
    }

    #[test]
    fn parses_what_menus_show() {
        let a = Accel::parse("Ctrl+Shift+Z").unwrap();
        assert!(a.ctrl && a.shift && !a.alt);
        assert_eq!(a.key, AccelKey::Char("z".into()));
        assert_eq!(Accel::parse("F1").unwrap().key, AccelKey::Named(Named::F1));
        assert_eq!(Accel::parse("Esc").unwrap().key, AccelKey::Named(Named::Escape));
        assert_eq!(Accel::parse("Ctrl+,").unwrap().key, AccelKey::Char(",".into()));
        assert_eq!(Accel::parse("Ctrl++").unwrap().key, AccelKey::Char("+".into()));
        assert!(Accel::parse("").is_none());
        assert!(Accel::parse("Hyper+X").is_none());
        assert!(Accel::parse("Ctrl+Nope").is_none());
    }

    /// A press as iced reports it: the unmodified key, the produced character, the modifiers.
    fn m(key: &str, modified: &str, modifiers: Modifiers) -> (Key, Key, Modifiers) {
        (ch(key), ch(modified), modifiers)
    }

    fn hit(accel: &str, (key, modified, modifiers): (Key, Key, Modifiers)) -> bool {
        Accel::parse(accel).unwrap().matches(&key, &modified, modifiers)
    }

    #[test]
    fn matches_modifiers_exactly() {
        assert!(hit("Ctrl+Q", m("q", "q", Modifiers::CTRL)));
        assert!(hit("Ctrl+Q", m("Q", "Q", Modifiers::CTRL)), "case-insensitive for the character");
        assert!(!hit("Ctrl+Q", m("q", "q", Modifiers::empty())));
        assert!(!hit("Ctrl+Q", m("q", "q", Modifiers::CTRL | Modifiers::ALT)));
        assert!(!hit("Ctrl+Q", m("q", "Q", Modifiers::CTRL | Modifiers::SHIFT)), "Shift is part of the accelerator for letters");
        assert!(hit("Ctrl+Shift+Z", m("z", "Z", Modifiers::CTRL | Modifiers::SHIFT)));
    }

    #[test]
    fn a_shifted_symbol_matches_the_character_it_types_on_any_layout() {
        let ctrl_shift = Modifiers::CTRL | Modifiers::SHIFT;
        // US: `?` is Shift+/ . The unmodified key is '/'.
        assert!(hit("Ctrl+?", m("/", "?", ctrl_shift)));
        // AZERTY: `?` is Shift+, . The unmodified key is ','; this is the reported bug.
        assert!(hit("Ctrl+?", m(",", "?", ctrl_shift)));
        // ...and Ctrl+, must not also fire for that press, nor `?` for a plain comma.
        assert!(!hit("Ctrl+,", m(",", "?", ctrl_shift)));
        assert!(hit("Ctrl+,", m(",", ",", Modifiers::CTRL)));
        assert!(!hit("Ctrl+?", m(",", ",", Modifiers::CTRL)));
        // An explicit Shift means the unmodified key with Shift held.
        assert!(hit("Ctrl+Shift+,", m(",", "?", ctrl_shift)));
        assert!(!hit("Ctrl+Shift+,", m(",", ",", Modifiers::CTRL)));
        assert!(Accel::parse("F1").unwrap().matches(&Key::Named(Named::F1), &Key::Named(Named::F1), Modifiers::empty()));
        assert!(!Accel::parse("F1").unwrap().matches(&Key::Named(Named::F1), &Key::Named(Named::F1), Modifiers::SHIFT));
    }

    #[test]
    fn shortcuts_resolve_and_plain_characters_do_not_steal_typing() {
        #[derive(Clone, PartialEq, Debug)]
        enum M {
            Quit,
            Help,
            Slash,
        }
        let s = Shortcuts::new().bind("Ctrl+Q", M::Quit).bind("F1", M::Help).bind("/", M::Slash).bind("Bogus+Q", M::Quit);
        let press = |key: Key, modifiers, captured| KeyPress { modified_key: key.clone(), key, modifiers, captured };
        assert_eq!(s.handle(&press(ch("q"), Modifiers::CTRL, false)), Some(M::Quit));
        assert_eq!(s.handle(&press(Key::Named(Named::F1), Modifiers::empty(), true)), Some(M::Help));
        assert_eq!(s.handle(&press(ch("/"), Modifiers::empty(), false)), Some(M::Slash));
        assert_eq!(s.handle(&press(ch("/"), Modifiers::empty(), true)), None, "typing into a field");
        assert_eq!(s.handle(&press(ch("x"), Modifiers::CTRL, false)), None);
    }

    #[test]
    fn standard_binds_only_what_is_set() {
        #[derive(Clone, PartialEq, Debug)]
        enum M {
            Quit,
            Full,
        }
        let std = Standard::new().quit(M::Quit).fullscreen(M::Full);
        assert_eq!(std.labels(), vec![("Quit".into(), "Ctrl+Q".into()), ("Fullscreen".into(), "F11".into())]);
        let s = std.into_shortcuts();
        let press = |key: Key, modifiers| KeyPress { modified_key: key.clone(), key, modifiers, captured: false };
        assert_eq!(s.handle(&press(Key::Character("q".into()), Modifiers::CTRL)), Some(M::Quit));
        assert_eq!(s.handle(&press(Key::Named(Named::F11), Modifiers::empty())), Some(M::Full));
        assert_eq!(s.handle(&press(Key::Named(Named::F1), Modifiers::empty())), None, "help is not set");
    }

    #[test]
    fn a_hint_is_shown_only_when_the_shortcut_is_available() {
        // Defaults: shortcuts on, hints automatic.
        assert_eq!(hint(Some("Ctrl+,"), true, false).as_deref(), Some("Ctrl+,"));
        assert_eq!(hint(Some("Ctrl+,"), true, true), None, "no hints in touch presentations");
        assert_eq!(hint(Some("Ctrl+,"), false, false), None, "a disabled item has no shortcut");
        assert_eq!(hint(Some("Hyper+Q"), true, false), None, "an accelerator that can never fire");
        assert_eq!(hint(None, true, false), None);
    }
}
