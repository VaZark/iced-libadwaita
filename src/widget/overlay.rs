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
//! The layer is a [`FocusScope`]: Tab stays inside it, its first field takes focus when it
//! appears and focus goes back to what had it when it closes ([`focus`](super::focus)).
//!
//! Nest them with [`stacked`]; each layer's scrim deepens the dim of everything below it.

use super::{Element, FocusScope, focus_return};
use crate::primitives::anchor::{Align, Dim, Side};
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
    edge: Option<(Side, Dim)>,
    along: (Dim, Align),
    margin: f32,
    initial_focus: Option<iced::widget::Id>,
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
            edge: None,
            along: (Dim::Fill, Align::Start),
            margin: 0.0,
            initial_focus: None,
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
    /// Docks the layer against an edge instead of centering it, sized `size` along the axis
    /// across that edge (`Dim::Fixed`, a `Fraction`, `Fill`, or `Auto` for its own size) and
    /// full along it: a bottom sheet, a drawer. It slides in from that edge.
    pub fn docked(mut self, side: Side, size: Dim) -> Self {
        self.edge = Some((side, size));
        self
    }
    /// How long a [docked](Self::docked) layer is along its edge, and where along it (default the
    /// whole edge). Like `docked`'s size it is a `Dim`: `Fixed`, a `Fraction`, `Fill` or `Auto`.
    pub fn along(mut self, length: Dim, align: Align) -> Self {
        self.along = (length, align);
        self
    }
    /// Distance in px between a [docked](Self::docked) layer and the edge it is docked to, and no
    /// other: 0 (the default) is flush with that edge. A press in the gap is a press outside.
    pub fn margin(mut self, px: f32) -> Self {
        self.margin = px.max(0.0);
        self
    }
    /// The child of the layer that takes focus when it appears, by its widget id (the default is
    /// the first focusable one): `entry(..).id(id)` in the layer and `.initial_focus(id)` here.
    pub fn initial_focus(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.initial_focus = Some(id.into());
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
        // The layer is a focus scope: Tab stays inside it, its first field takes focus as it
        // appears, and closing it gives focus back to what had it (`focus_return` below).
        let mut scope = FocusScope::new(o.layer);
        if let Some(id) = o.initial_focus {
            scope = scope.initial_focus(id);
        }
        let content: Element<'a, M> = scope.into();
        let (layer, offset) = match o.edge {
            None => (
                container(opaque(content))
                    .center(Length::Fill)
                    .padding(12)
                    .into(),
                iced::Vector::new(0.0, 24.0),
            ),
            Some((side, size)) => (
                docked_layer(
                    content,
                    Dock {
                        side,
                        size,
                        along: o.along.0,
                        align: o.along.1,
                        margin: o.margin,
                    },
                ),
                match side {
                    Side::Bottom => iced::Vector::new(0.0, 160.0),
                    Side::Top => iced::Vector::new(0.0, -160.0),
                    Side::Left => iced::Vector::new(-160.0, 0.0),
                    Side::Right => iced::Vector::new(160.0, 0.0),
                },
            ),
        };
        parts.push(layer);
        let mut entrance = super::entrance(stack(parts), offset);
        if o.edge.is_some() {
            entrance = entrance.trigger(crate::animation::Trigger::Sheet);
        }
        if o.rules.scrim {
            entrance = entrance.scrim(o.window_radius);
        }
        focus_return(super::layered(o.base, Some(Element::from(entrance))), true)
    }
}

/// `base` with no layer over it, in the tree position an open [`Overlay`] has, so opening and
/// closing one does not disturb the base and focus returns to it.
pub fn closed<'a, M: 'a>(base: impl Into<Element<'a, M>>) -> Element<'a, M> {
    focus_return(super::layered(base, None), false)
}

/// The panel's length across its edge, and the weight of the room beside it when it is a
/// fraction (`Fraction(f)` shares the axis in `f : 1 - f`).
fn edge_lengths(size: Dim) -> (Length, Option<Length>) {
    match size {
        Dim::Auto => (Length::Shrink, None),
        Dim::Fixed(px) => (Length::Fixed(px.max(0.0)), None),
        Dim::MatchAnchor | Dim::Fill => (Length::Fill, None),
        Dim::Fraction(f) => {
            let p = (f.clamp(0.0, 1.0) * 100.0).round().clamp(1.0, 99.0) as u16;
            (Length::FillPortion(p), Some(Length::FillPortion(100 - p)))
        }
    }
}

/// `content` on one axis of the layer: `len` long (`rest` is the weight of the room when that is a
/// fraction), at the start, center or end of the axis. `vertical` says which axis; `other` is
/// the length on the other axis.
fn place<'a, M: 'a>(
    content: Element<'a, M>,
    vertical: bool,
    len: Length,
    rest: Option<Length>,
    at: Align,
    other: Length,
) -> Element<'a, M> {
    let sized: Element<'a, M> = if vertical {
        container(content).width(other).height(len).into()
    } else {
        container(content).width(len).height(other).into()
    };
    if matches!(len, Length::Fill) {
        return sized;
    }
    let room = |weight: Length| -> Element<'a, M> {
        if vertical {
            space().height(weight).into()
        } else {
            space().width(weight).into()
        }
    };
    let rest = rest.unwrap_or(Length::Fill);
    let half = match rest {
        Length::FillPortion(w) => Length::FillPortion((w / 2).max(1)),
        other => other,
    };
    let items: Vec<Element<'a, M>> = match at {
        Align::Start => vec![sized, room(rest)],
        Align::End => vec![room(rest), sized],
        Align::Center => vec![room(half), sized, room(half)],
    };
    if vertical {
        iced::widget::column(items).into()
    } else {
        iced::widget::row(items).into()
    }
}

/// How a docked layer sits: against `side`, `size` across it, `along` its length and `align`ed
/// there, `margin` clear of the edge it is docked to (and of nothing else).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Dock {
    side: Side,
    size: Dim,
    along: Dim,
    align: Align,
    margin: f32,
}

/// `panel` docked as `d` says.
fn docked_layer<'a, M: 'a>(panel: Element<'a, M>, d: Dock) -> Element<'a, M> {
    let horizontal = matches!(d.side, Side::Left | Side::Right);
    // Along the edge first, then across it.
    let (along_len, along_rest) = edge_lengths(d.along);
    let (len, rest) = edge_lengths(d.size);
    // The panel keeps its own size across the edge; a share of the axis is the outer place's.
    let across = match len {
        Length::FillPortion(_) => Length::Fill,
        other => other,
    };
    let along = place(
        opaque(panel),
        horizontal,
        along_len,
        along_rest,
        d.align,
        across,
    );
    let at = match d.side {
        Side::Left | Side::Top => Align::Start,
        Side::Right | Side::Bottom => Align::End,
    };
    let layer = place(along, !horizontal, len, rest, at, Length::Fill);
    // The margin keeps it clear of the docked edge only.
    let m = d.margin;
    let padding = match d.side {
        Side::Top => iced::Padding {
            top: m,
            ..iced::Padding::ZERO
        },
        Side::Bottom => iced::Padding {
            bottom: m,
            ..iced::Padding::ZERO
        },
        Side::Left => iced::Padding {
            left: m,
            ..iced::Padding::ZERO
        },
        Side::Right => iced::Padding {
            right: m,
            ..iced::Padding::ZERO
        },
    };
    container(layer)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(padding)
        .into()
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
