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
use iced::Subscription;
use iced::keyboard::{self, Key, Modifiers, key::Named};

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
        let mut a = Accel {
            ctrl: false,
            shift: false,
            alt: false,
            logo: false,
            key: AccelKey::Char(String::new()),
        };
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

    /// How well this accelerator matches a key press: `Some(rank)` with a lower rank meaning a
    /// better match, `None` for no match. See [`candidates`] for the interpretations tried.
    ///
    /// Ctrl, Alt and Super must match exactly; Shift is part of each interpretation.
    pub fn match_rank(&self, press: &KeyPress) -> Option<u8> {
        if self.ctrl != press.modifiers.control()
            || self.alt != press.modifiers.alt()
            || self.logo != press.modifiers.logo()
        {
            return None;
        }
        candidates(press)
            .into_iter()
            .filter(|c| c.key == self.key && c.shift == self.shift)
            .map(|c| c.rank)
            .min()
    }

    /// Whether this accelerator is the given key press, under any interpretation.
    pub fn matches(&self, press: &KeyPress) -> bool {
        self.match_rank(press).is_some()
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
    /// The physical key, independent of the layout.
    pub physical_key: keyboard::key::Physical,
    pub modifiers: Modifiers,
    /// A focused widget (a text input) already handled the key.
    pub captured: bool,
}

/// One interpretation of a key press, to compare with an accelerator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub key: AccelKey,
    /// Whether Shift counts as part of the accelerator for this interpretation.
    pub shift: bool,
    /// Lower is better: 0 is what the user typed, 1 is a layout-independent fallback.
    pub rank: u8,
}

fn lower(key: &Key) -> Option<AccelKey> {
    match key {
        Key::Named(n) => Some(AccelKey::Named(*n)),
        Key::Character(c) => Some(AccelKey::Char(c.to_lowercase())),
        Key::Unidentified => None,
    }
}

/// The Latin letter or digit at a physical key position, so shortcuts keep working on layouts
/// where the key types something else (a Cyrillic or Greek layout, or AZERTY's number row).
fn latin(code: keyboard::key::Code) -> Option<char> {
    use keyboard::key::Code::*;
    Some(match code {
        KeyA => 'a',
        KeyB => 'b',
        KeyC => 'c',
        KeyD => 'd',
        KeyE => 'e',
        KeyF => 'f',
        KeyG => 'g',
        KeyH => 'h',
        KeyI => 'i',
        KeyJ => 'j',
        KeyK => 'k',
        KeyL => 'l',
        KeyM => 'm',
        KeyN => 'n',
        KeyO => 'o',
        KeyP => 'p',
        KeyQ => 'q',
        KeyR => 'r',
        KeyS => 's',
        KeyT => 't',
        KeyU => 'u',
        KeyV => 'v',
        KeyW => 'w',
        KeyX => 'x',
        KeyY => 'y',
        KeyZ => 'z',
        Digit0 => '0',
        Digit1 => '1',
        Digit2 => '2',
        Digit3 => '3',
        Digit4 => '4',
        Digit5 => '5',
        Digit6 => '6',
        Digit7 => '7',
        Digit8 => '8',
        Digit9 => '9',
        _ => return None,
    })
}

/// Every way a key press can be read, best first.
///
/// 1. **The key as typed** (rank 0): the unmodified key, with Shift counted as held. This is how
///    letters, named keys and an explicit `Shift+,` match.
/// 2. **The character produced** (rank 0): when the press types something other than its key
///    (Shift+`,` types `?` on AZERTY, Shift+`/` on US; Shift+`&` types `1` on AZERTY), that
///    character with Shift *consumed*, so `Ctrl+?` and `Ctrl+1` match on any layout.
/// 3. **The physical key** (rank 1): the Latin letter or digit at that position, with Shift as
///    held, so `Ctrl+Q` works on a Cyrillic layout and `Ctrl+1` on AZERTY's unshifted number row.
///
/// A binding is chosen by the best rank any of its interpretations reaches, then by registration
/// order. So an exact typed match beats a layout-independent fallback registered earlier.
pub fn candidates(press: &KeyPress) -> Vec<Candidate> {
    let shift = press.modifiers.shift();
    let mut out = Vec::new();
    if let Some(key) = lower(&press.key) {
        out.push(Candidate {
            key,
            shift,
            rank: 0,
        });
    }
    // Skipped when the press only changed the letter's case (Shift+q types 'Q'): that is still
    // the key with Shift held, and must not match a plain `Ctrl+Q`.
    if lower(&press.modified_key) != lower(&press.key)
        && let Some(produced @ AccelKey::Char(_)) = lower(&press.modified_key)
    {
        out.push(Candidate {
            key: produced,
            shift: false,
            rank: 0,
        });
    }
    if let keyboard::key::Physical::Code(code) = press.physical_key
        && let Some(c) = latin(code)
    {
        out.push(Candidate {
            key: AccelKey::Char(c.to_string()),
            shift,
            rank: 1,
        });
    }
    out
}

/// A subscription of every key press, to feed [`Shortcuts::handle`].
pub fn key_presses() -> Subscription<KeyPress> {
    iced::event::listen_with(|event, status, _| match event {
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modified_key,
            physical_key,
            modifiers,
            ..
        }) => Some(KeyPress {
            key,
            modified_key,
            physical_key,
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
        Self {
            bindings: Vec::new(),
        }
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
    pub fn scope<'a>(
        self,
        content: impl Into<iced::Element<'a, M, crate::Theme>>,
    ) -> iced::Element<'a, M, crate::Theme>
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
        // The best-ranked match wins; among equal ranks, the earliest binding.
        self.bindings
            .iter()
            .filter(|(a, _)| !(press.captured && a.is_plain_character()))
            .filter_map(|(a, m)| a.match_rank(press).map(|rank| (rank, m)))
            .min_by_key(|(rank, _)| *rank)
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
        Self {
            quit: None,
            close_window: None,
            fullscreen: None,
            preferences: None,
            help: None,
            shortcuts: None,
        }
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
    use keyboard::key::{Code, Physical};

    fn ch(c: &str) -> Key {
        Key::Character(c.into())
    }

    /// A press as iced reports it: the unmodified key, the produced character, the physical key
    /// and the modifiers.
    fn press(key: &str, produced: &str, code: Code, modifiers: Modifiers) -> KeyPress {
        KeyPress {
            key: ch(key),
            modified_key: ch(produced),
            physical_key: Physical::Code(code),
            modifiers,
            captured: false,
        }
    }

    fn named_press(key: Named, modifiers: Modifiers) -> KeyPress {
        KeyPress {
            key: Key::Named(key),
            modified_key: Key::Named(key),
            physical_key: Physical::Code(Code::F1),
            modifiers,
            captured: false,
        }
    }

    fn hit(accel: &str, p: &KeyPress) -> bool {
        Accel::parse(accel).unwrap().matches(p)
    }

    const CTRL_SHIFT: Modifiers = Modifiers::CTRL.union(Modifiers::SHIFT);

    #[test]
    fn parses_what_menus_show() {
        let a = Accel::parse("Ctrl+Shift+Z").unwrap();
        assert!(a.ctrl && a.shift && !a.alt);
        assert_eq!(a.key, AccelKey::Char("z".into()));
        assert_eq!(Accel::parse("F1").unwrap().key, AccelKey::Named(Named::F1));
        assert_eq!(
            Accel::parse("Esc").unwrap().key,
            AccelKey::Named(Named::Escape)
        );
        assert_eq!(
            Accel::parse("Ctrl+,").unwrap().key,
            AccelKey::Char(",".into())
        );
        assert_eq!(
            Accel::parse("Ctrl++").unwrap().key,
            AccelKey::Char("+".into())
        );
        assert!(Accel::parse("").is_none());
        assert!(Accel::parse("Hyper+X").is_none());
        assert!(Accel::parse("Ctrl+Nope").is_none());
    }

    #[test]
    fn modifiers_must_match_exactly() {
        assert!(hit("Ctrl+Q", &press("q", "q", Code::KeyQ, Modifiers::CTRL)));
        assert!(
            hit("Ctrl+Q", &press("Q", "Q", Code::KeyQ, Modifiers::CTRL)),
            "case-insensitive"
        );
        assert!(!hit(
            "Ctrl+Q",
            &press("q", "q", Code::KeyQ, Modifiers::empty())
        ));
        assert!(!hit(
            "Ctrl+Q",
            &press("q", "q", Code::KeyQ, Modifiers::CTRL | Modifiers::ALT)
        ));
        assert!(
            !hit("Ctrl+Q", &press("q", "Q", Code::KeyQ, CTRL_SHIFT)),
            "Shift is part of a letter accelerator"
        );
        assert!(hit(
            "Ctrl+Shift+Z",
            &press("z", "Z", Code::KeyZ, CTRL_SHIFT)
        ));
        assert!(hit("F1", &named_press(Named::F1, Modifiers::empty())));
        assert!(!hit("F1", &named_press(Named::F1, Modifiers::SHIFT)));
    }

    #[test]
    fn a_shifted_symbol_matches_the_character_it_types_on_any_layout() {
        // US: `?` is Shift+/ (unmodified key '/').
        assert!(hit("Ctrl+?", &press("/", "?", Code::Slash, CTRL_SHIFT)));
        // AZERTY: `?` is Shift+, (unmodified key ','), on the physical M position.
        let azerty = press(",", "?", Code::KeyM, CTRL_SHIFT);
        assert!(hit("Ctrl+?", &azerty));
        assert!(!hit("Ctrl+,", &azerty), "Ctrl+, must not also fire");
        assert!(hit("Ctrl+,", &press(",", ",", Code::KeyM, Modifiers::CTRL)));
        assert!(!hit(
            "Ctrl+?",
            &press(",", ",", Code::KeyM, Modifiers::CTRL)
        ));
        // An explicit Shift means the unmodified key with Shift held.
        assert!(hit("Ctrl+Shift+,", &azerty));
        assert!(!hit(
            "Ctrl+Shift+,",
            &press(",", ",", Code::KeyM, Modifiers::CTRL)
        ));
    }

    #[test]
    fn digits_work_on_azerty_shifted_or_not() {
        // AZERTY's number row types symbols unshifted: Digit1 types '&', Shift+Digit1 types '1'.
        assert!(
            hit("Ctrl+1", &press("&", "1", Code::Digit1, CTRL_SHIFT)),
            "the produced '1'"
        );
        assert!(
            hit("Ctrl+1", &press("&", "&", Code::Digit1, Modifiers::CTRL)),
            "the physical digit"
        );
        // A different row position must not fire it.
        assert!(!hit(
            "Ctrl+1",
            &press("é", "é", Code::Digit2, Modifiers::CTRL)
        ));
    }

    #[test]
    fn letters_work_on_a_non_latin_layout_through_the_physical_key() {
        // Cyrillic: the Q position types 'й'.
        let cyr = press("й", "й", Code::KeyQ, Modifiers::CTRL);
        assert!(hit("Ctrl+Q", &cyr));
        assert!(hit("Ctrl+Й", &cyr), "and the typed letter still matches");
        assert!(hit(
            "Ctrl+Shift+Q",
            &press("й", "Й", Code::KeyQ, CTRL_SHIFT)
        ));
        assert!(!hit("Ctrl+W", &cyr));
    }

    #[test]
    fn what_was_typed_beats_a_layout_fallback_whatever_the_order() {
        #[derive(Clone, PartialEq, Debug)]
        enum M {
            ByPosition,
            ByTyping,
        }
        // On AZERTY the Digit1 key types '&'. A binding for the character beats one for the digit.
        let s = Shortcuts::new()
            .bind("Ctrl+1", M::ByPosition)
            .bind("Ctrl+&", M::ByTyping);
        assert_eq!(
            s.handle(&press("&", "&", Code::Digit1, Modifiers::CTRL)),
            Some(M::ByTyping)
        );
        // Without a binding for what was typed, the physical fallback applies.
        let s = Shortcuts::new().bind("Ctrl+1", M::ByPosition);
        assert_eq!(
            s.handle(&press("&", "&", Code::Digit1, Modifiers::CTRL)),
            Some(M::ByPosition)
        );
        // Equal rank: the earliest binding wins.
        let s = Shortcuts::new()
            .bind("Ctrl+Q", M::ByPosition)
            .bind("Ctrl+Q", M::ByTyping);
        assert_eq!(
            s.handle(&press("q", "q", Code::KeyQ, Modifiers::CTRL)),
            Some(M::ByPosition)
        );
    }

    #[test]
    fn shortcuts_resolve_and_plain_characters_do_not_steal_typing() {
        #[derive(Clone, PartialEq, Debug)]
        enum M {
            Quit,
            Help,
            Slash,
        }
        let s = Shortcuts::new()
            .bind("Ctrl+Q", M::Quit)
            .bind("F1", M::Help)
            .bind("/", M::Slash)
            .bind("Bogus+Q", M::Quit);
        let captured = |mut p: KeyPress| {
            p.captured = true;
            p
        };
        assert_eq!(
            s.handle(&press("q", "q", Code::KeyQ, Modifiers::CTRL)),
            Some(M::Quit)
        );
        assert_eq!(
            s.handle(&captured(named_press(Named::F1, Modifiers::empty()))),
            Some(M::Help)
        );
        assert_eq!(
            s.handle(&press("/", "/", Code::Slash, Modifiers::empty())),
            Some(M::Slash)
        );
        assert_eq!(
            s.handle(&captured(press("/", "/", Code::Slash, Modifiers::empty()))),
            None,
            "typing into a field"
        );
        assert_eq!(
            s.handle(&press("x", "x", Code::KeyX, Modifiers::CTRL)),
            None
        );
    }

    #[test]
    fn standard_binds_only_what_is_set() {
        #[derive(Clone, PartialEq, Debug)]
        enum M {
            Quit,
            Full,
        }
        let std = Standard::new().quit(M::Quit).fullscreen(M::Full);
        assert_eq!(
            std.labels(),
            vec![
                ("Quit".into(), "Ctrl+Q".into()),
                ("Fullscreen".into(), "F11".into())
            ]
        );
        let s = std.into_shortcuts();
        assert_eq!(
            s.handle(&press("q", "q", Code::KeyQ, Modifiers::CTRL)),
            Some(M::Quit)
        );
        assert_eq!(
            s.handle(&named_press(Named::F11, Modifiers::empty())),
            Some(M::Full)
        );
        assert_eq!(
            s.handle(&named_press(Named::F1, Modifiers::empty())),
            None,
            "help is not set"
        );
    }

    #[test]
    fn a_hint_is_shown_only_when_the_shortcut_is_available() {
        // Defaults: shortcuts on, hints automatic.
        assert_eq!(hint(Some("Ctrl+,"), true, false).as_deref(), Some("Ctrl+,"));
        assert_eq!(
            hint(Some("Ctrl+,"), true, true),
            None,
            "no hints in touch presentations"
        );
        assert_eq!(
            hint(Some("Ctrl+,"), false, false),
            None,
            "a disabled item has no shortcut"
        );
        assert_eq!(
            hint(Some("Hyper+Q"), true, false),
            None,
            "an accelerator that can never fire"
        );
        assert_eq!(hint(None, true, false), None);
    }
}
