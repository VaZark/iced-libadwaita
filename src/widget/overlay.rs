//! `Overlay`: a layer over a base view, for dialogs and nested windows (a preferences window over
//! a dialog over the main window).
//!
//! The rules (the fade, modality, dismissal) are the [`primitives::overlay`](crate::primitives::overlay)
//! ones. The base is faded and disabled while the layer is up: a modal layer eats presses
//! outside its content, and [`scrim(false)`](Overlay::scrim) keeps the base unfaded. A press
//! outside sends [`on_outside`](Overlay::on_outside): a dialog's app closes it, a window's
//! brings the window forward instead. Leave it unset for a layer a press outside does not
//! touch.
//! The layer slides in (and the scrim fades) on the dialog entrance. Escape is the app's:
//! [`Stack::escape`](crate::primitives::overlay::Stack::escape) says which layer it closes.
//!
//! ```ignore
//! Overlay::new(main_view, settings_dialog)
//!     .scrim(true)
//!     .on_outside(Message::CloseSettings)
//! ```
//!
//! Nest them with [`stacked`]; each layer's scrim deepens the dim of everything below it.

use super::Element;
use crate::primitives::overlay::Overlay as Rules;
use iced::Length;
use iced::widget::{container, mouse_area, opaque, space, stack};

/// A layer over a base. See the [module](self).
pub struct Overlay<'a, M> {
    base: Element<'a, M>,
    layer: Element<'a, M>,
    rules: Rules,
    on_outside: Option<M>,
    window_radius: f32,
}

impl<'a, M: Clone + 'a> Overlay<'a, M> {
    /// `layer` (centered) over `base`: modal, with the fade, closed by a press outside.
    pub fn new(base: impl Into<Element<'a, M>>, layer: impl Into<Element<'a, M>>) -> Self {
        Self {
            base: base.into(),
            layer: layer.into(),
            rules: Rules::default(),
            on_outside: None,
            window_radius: 0.0,
        }
    }
    /// Fades the base below the layer (default on).
    pub fn scrim(mut self, on: bool) -> Self {
        self.rules.scrim = on;
        self
    }
    /// Blocks the base (default on). A non-modal layer lets presses outside it through.
    pub fn modal(mut self, on: bool) -> Self {
        self.rules.modal = on;
        self
    }
    /// All the rules at once.
    pub fn rules(mut self, rules: Rules) -> Self {
        self.rules = rules;
        self
    }
    /// Sent for a press outside the layer (a modal layer only). Whether that closes the layer or
    /// does something else is up to the message.
    pub fn on_outside(mut self, m: M) -> Self {
        self.on_outside = Some(m);
        self
    }
    /// Rounds the scrim to a rounded window's corners (`window::State::radius`).
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = r;
        self
    }
}

impl<'a, M: Clone + 'a> From<Overlay<'a, M>> for Element<'a, M> {
    fn from(o: Overlay<'a, M>) -> Self {
        let mut parts: Vec<Element<'a, M>> = Vec::new();
        if o.rules.modal {
            let mut catcher =
                mouse_area(container(space()).width(Length::Fill).height(Length::Fill));
            if let Some(m) = o.on_outside {
                catcher = catcher.on_press(m);
            }
            parts.push(opaque(catcher));
        }
        parts.push(
            container(opaque(o.layer))
                .center(Length::Fill)
                .padding(12)
                .into(),
        );
        let mut entrance = super::entrance(stack(parts), iced::Vector::new(0.0, 24.0));
        if o.rules.scrim {
            entrance = entrance.scrim(o.window_radius);
        }
        super::layered(o.base, Some(Element::from(entrance)))
    }
}

/// One layer of [`stacked`].
pub struct Layer<'a, M> {
    pub content: Element<'a, M>,
    pub rules: Rules,
    pub on_outside: Option<M>,
}

/// `layers` over `base`, bottom first: each is an [`Overlay`] over everything below it.
pub fn stacked<'a, M: Clone + 'a>(
    base: impl Into<Element<'a, M>>,
    layers: impl IntoIterator<Item = Layer<'a, M>>,
    window_radius: f32,
) -> Element<'a, M> {
    layers.into_iter().fold(base.into(), |below, l| {
        let mut o = Overlay::new(below, l.content)
            .rules(l.rules)
            .window_radius(window_radius);
        if let Some(m) = l.on_outside {
            o = o.on_outside(m);
        }
        o.into()
    })
}
