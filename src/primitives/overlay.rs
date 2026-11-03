//! Overlays: the rules of a surface laid over another one (a dialog over its window, a nested
//! window such as a preferences window over a dialog), with no drawing.
//!
//! An [`Overlay`] says what the layer does to what is below it: whether it fades it (`scrim`,
//! on by default), whether it blocks it (`modal`) and how it is dismissed (a press outside, Escape).
//! [`Overlay::route`] turns an input into one [`Action`]. A [`Stack`] holds the open layers in
//! order: only the top one is dismissed by Escape, and a press goes down through layers that let
//! it through. [`centered`] places a layer's content in the area it covers.
//!
//! Everything is controlled: the application owns the [`Stack`] (or one `Overlay`) and the
//! widgets ([`widget::Overlay`](crate::widget::Overlay), a [`Popover`](crate::widget::Popover)
//! presented as a dialog) draw it.

use iced::{Rectangle, Size};

/// How a layer treats what is below it and how it is dismissed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overlay {
    /// Fades what is below (the dialog dim). On by default.
    pub scrim: bool,
    /// Blocks input to what is below: presses outside the content go nowhere else.
    pub modal: bool,
    /// A press outside the content closes it (default true; not for a window, which only
    /// disables its parent).
    pub close_on_outside: bool,
    /// Escape closes it.
    pub dismiss_on_escape: bool,
}

impl Default for Overlay {
    /// A modal layer with a scrim that closes on a press outside or Escape.
    fn default() -> Self {
        Self {
            scrim: true,
            modal: true,
            close_on_outside: true,
            dismiss_on_escape: true,
        }
    }
}

/// An input a layer is asked about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Escape,
    /// A press; `inside` is whether it landed on the layer's content.
    Press {
        inside: bool,
    },
}

/// What a layer does with an [`Input`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// The layer closes.
    Close,
    /// The layer's content handles it.
    Content,
    /// The layer eats it: nothing below sees it.
    Swallow,
    /// The layer lets it through to what is below.
    Through,
}

impl Overlay {
    pub fn scrim(mut self, on: bool) -> Self {
        self.scrim = on;
        self
    }
    pub fn modal(mut self, on: bool) -> Self {
        self.modal = on;
        self
    }
    /// Whether a press outside the content closes the layer (default true).
    pub fn close_on_outside(mut self, on: bool) -> Self {
        self.close_on_outside = on;
        self
    }
    pub fn dismiss_on_escape(mut self, on: bool) -> Self {
        self.dismiss_on_escape = on;
        self
    }

    /// The scrim's strength while the layer is `progress` (0 closed .. 1 open) in: `0` without a
    /// scrim.
    pub fn scrim_amount(&self, progress: f32) -> f32 {
        if self.scrim {
            progress.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// What the layer does with `input`.
    pub fn route(&self, input: Input) -> Action {
        match input {
            Input::Escape if self.dismiss_on_escape => Action::Close,
            Input::Escape => self.otherwise(true),
            Input::Press { inside: true } => Action::Content,
            Input::Press { inside: false } if self.close_on_outside => Action::Close,
            Input::Press { inside: false } => self.otherwise(false),
        }
    }

    fn otherwise(&self, content: bool) -> Action {
        match (self.modal, content) {
            (true, true) => Action::Content,
            (true, false) => Action::Swallow,
            (false, _) => Action::Through,
        }
    }
}

/// Where content of `size` goes in `area`: centered, shrunk to leave `margin` on every side.
pub fn centered(area: Rectangle, size: Size, margin: f32) -> Rectangle {
    let room = max_size(area.size(), margin);
    let size = Size::new(size.width.min(room.width), size.height.min(room.height));
    Rectangle {
        x: area.x + (area.width - size.width) / 2.0,
        y: area.y + (area.height - size.height) / 2.0,
        width: size.width,
        height: size.height,
    }
}

/// The largest content that fits `area` with `margin` on every side.
pub fn max_size(area: Size, margin: f32) -> Size {
    Size::new(
        (area.width - 2.0 * margin).max(0.0),
        (area.height - 2.0 * margin).max(0.0),
    )
}

/// The open layers, bottom to top, each with a key.
#[derive(Debug, Clone, PartialEq)]
pub struct Stack<K> {
    layers: Vec<(K, Overlay)>,
}

impl<K> Default for Stack<K> {
    fn default() -> Self {
        Self { layers: Vec::new() }
    }
}

impl<K: PartialEq + Clone> Stack<K> {
    pub fn new() -> Self {
        Self::default()
    }
    /// Opens `key` on top; an open one moves to the top with the new options.
    pub fn open(&mut self, key: K, overlay: Overlay) {
        self.layers.retain(|(k, _)| *k != key);
        self.layers.push((key, overlay));
    }
    /// Closes `key` (and the layers opened over it, which belong to it). Whether it was open.
    pub fn close(&mut self, key: &K) -> bool {
        match self.layers.iter().position(|(k, _)| k == key) {
            Some(i) => {
                self.layers.truncate(i);
                true
            }
            None => false,
        }
    }
    /// Closes the top layer and returns its key.
    pub fn close_top(&mut self) -> Option<K> {
        self.layers.pop().map(|(k, _)| k)
    }
    pub fn top(&self) -> Option<&K> {
        self.layers.last().map(|(k, _)| k)
    }
    pub fn is_open(&self, key: &K) -> bool {
        self.layers.iter().any(|(k, _)| k == key)
    }
    pub fn len(&self) -> usize {
        self.layers.len()
    }
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }
    /// The layers bottom to top.
    pub fn layers(&self) -> impl Iterator<Item = &(K, Overlay)> {
        self.layers.iter()
    }
    /// Whether `key` is covered by a modal layer above it, so it takes no input.
    pub fn inert(&self, key: &K) -> bool {
        match self.layers.iter().position(|(k, _)| k == key) {
            Some(i) => self.layers[i + 1..].iter().any(|(_, o)| o.modal),
            None => false,
        }
    }
    /// How many scrims lie over the layer `key` (or over the base for `None`): the dim deepens
    /// with each one.
    pub fn scrims_over(&self, key: Option<&K>) -> usize {
        let from = match key {
            Some(k) => match self.layers.iter().position(|(l, _)| l == k) {
                Some(i) => i + 1,
                None => return 0,
            },
            None => 0,
        };
        self.layers[from..].iter().filter(|(_, o)| o.scrim).count()
    }
    /// Which layer gets `input` and what it does with it: from the top down, past the layers
    /// that let it through. `inside(key)` says whether a press landed on that layer's content.
    pub fn route(&self, input: impl Fn(&K) -> Input) -> Option<(K, Action)> {
        for (key, overlay) in self.layers.iter().rev() {
            match overlay.route(input(key)) {
                Action::Through => continue,
                action => return Some((key.clone(), action)),
            }
        }
        None
    }
    /// Escape: closes the top layer that is dismissed by it; the key of what closed.
    pub fn escape(&mut self) -> Option<K> {
        let (key, action) = self.route(|_| Input::Escape)?;
        (action == Action::Close).then(|| {
            self.close(&key);
            key
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_layer_fades_what_is_below_unless_it_opts_out() {
        assert!(Overlay::default().scrim);
        assert_eq!(Overlay::default().scrim(false).scrim_amount(1.0), 0.0);
        let o = Overlay::default();
        assert_eq!(o.scrim_amount(0.5), 0.5);
        assert_eq!(o.scrim_amount(3.0), 1.0);
    }

    #[test]
    fn a_modal_layer_closes_swallows_or_hands_to_its_content() {
        let o = Overlay::default();
        assert_eq!(o.route(Input::Escape), Action::Close);
        assert_eq!(o.route(Input::Press { inside: true }), Action::Content);
        assert_eq!(o.route(Input::Press { inside: false }), Action::Close);
        let stay = o.close_on_outside(false);
        assert_eq!(stay.route(Input::Press { inside: false }), Action::Swallow);
        let no_esc = o.dismiss_on_escape(false);
        assert_eq!(no_esc.route(Input::Escape), Action::Content);
    }

    #[test]
    fn a_non_modal_layer_lets_presses_through() {
        let o = Overlay::default().modal(false).close_on_outside(false);
        assert_eq!(o.route(Input::Press { inside: false }), Action::Through);
        assert_eq!(o.route(Input::Press { inside: true }), Action::Content);
    }

    #[test]
    fn content_is_centered_and_kept_inside_the_margin() {
        let area = Rectangle::new(iced::Point::new(10.0, 20.0), Size::new(400.0, 300.0));
        let r = centered(area, Size::new(200.0, 100.0), 12.0);
        assert_eq!((r.x, r.y, r.width, r.height), (110.0, 120.0, 200.0, 100.0));
        let big = centered(area, Size::new(900.0, 900.0), 12.0);
        assert_eq!((big.width, big.height), (376.0, 276.0));
        assert_eq!((big.x, big.y), (22.0, 32.0));
    }

    #[test]
    fn closing_a_layer_closes_what_was_opened_over_it() {
        let mut s = Stack::new();
        s.open("settings", Overlay::default());
        s.open("about", Overlay::default());
        assert_eq!(s.top(), Some(&"about"));
        assert!(s.close(&"settings"));
        assert!(s.is_empty());
        assert!(!s.close(&"settings"));
    }

    #[test]
    fn only_the_top_layer_is_dismissed_by_escape() {
        let mut s = Stack::new();
        s.open(1, Overlay::default());
        s.open(2, Overlay::default());
        assert_eq!(s.escape(), Some(2));
        assert_eq!(s.top(), Some(&1));
        assert_eq!(s.escape(), Some(1));
        assert_eq!(s.escape(), None);
    }

    #[test]
    fn a_modal_layer_makes_those_below_inert() {
        let mut s = Stack::new();
        s.open(1, Overlay::default());
        s.open(2, Overlay::default().modal(false));
        assert!(!s.inert(&1), "a non-modal layer over it does not block it");
        s.open(3, Overlay::default());
        assert!(s.inert(&1) && s.inert(&2) && !s.inert(&3));
    }

    #[test]
    fn scrims_deepen_with_each_layer_that_has_one() {
        let mut s = Stack::new();
        s.open("a", Overlay::default());
        s.open("b", Overlay::default().scrim(false));
        s.open("c", Overlay::default());
        assert_eq!(s.scrims_over(None), 2);
        assert_eq!(s.scrims_over(Some(&"a")), 1);
        assert_eq!(s.scrims_over(Some(&"c")), 0);
    }

    #[test]
    fn a_press_goes_down_through_layers_that_let_it_through() {
        let mut s = Stack::new();
        s.open("window", Overlay::default());
        s.open(
            "hint",
            Overlay::default().modal(false).close_on_outside(false),
        );
        // Outside both: the hint passes it, the window closes.
        let hit = s.route(|_| Input::Press { inside: false });
        assert_eq!(hit, Some(("window", Action::Close)));
        // Inside the hint: it keeps it.
        let hit = s.route(|k| Input::Press {
            inside: *k == "hint",
        });
        assert_eq!(hit, Some(("hint", Action::Content)));
    }
}
