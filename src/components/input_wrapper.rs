//! `InputWrapper`: a label, a control and a message, with one validation presentation for every
//! form control (Radix `Form.Field`, HTML label + control + message).
//!
//! A thin applier of [`validation`](crate::primitives::validation): the application owns the
//! value, the `touched` flag and the [`Validity`]; the field only decides what to show
//! (pristine until touched, then valid / warning / invalid) and draws an outline around the
//! control (when its geometry is supplied) plus the message below it. The control is any element (entry, text view,
//! dropdown, check button, scale, rows …), kept at a fixed position in the tree so its state
//! and overlays (dropdown popovers) survive validity changes.
//!
//! Arbitrary elements do not expose their effective shape. Supply
//! [`InputWrapper::target_geometry`] or [`InputWrapper::target_style`] to paint their
//! validation outline; unknown targets still show messages and retain layout spacing.
//! Inline mode owns a separate padded frame, whose default corners use the current theme.

use crate::Element;
use crate::primitives::validation::{self, Interaction, Presentation, Validity};
use crate::{Icon, Metrics, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell, overlay};
use iced::widget::{column, container, row, space, text};
use iced::{Alignment, Border, Length, Padding, Rectangle};
use iced::{Color, Event, Size, Vector, mouse};

type TargetGeometry<'a> = Box<dyn Fn(&Theme, Rectangle) -> Option<iced::border::Radius> + 'a>;

/// Where the label sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputWrapperLayout {
    /// Label above the control (the standard form layout).
    #[default]
    Stacked,
    /// Label and description at the start, the control at the end, like a libadwaita row;
    /// the message sits below.
    Row,
}

pub struct InputWrapper<'a, M> {
    label_props: super::props::TextProps,
    description_props: super::props::TextProps,
    message_props: super::props::TextProps,
    label: String,
    control: Element<'a, M>,
    description: Option<String>,
    validity: Validity,
    required: bool,
    touched: bool,
    submitted: bool,
    disabled: bool,
    inline: bool,
    plain_label: bool,
    hide_label: bool,
    show_valid: bool,
    layout: InputWrapperLayout,
    spacing: Option<f32>,
    label_spacing: Option<f32>,
    inline_padding: Option<Padding>,
    metrics: Metrics,
    target_geometry: Option<TargetGeometry<'a>>,
}

impl<'a, M: 'a> InputWrapper<'a, M> {
    pub fn label_props(mut self, props: super::props::TextProps) -> Self {
        self.label_props = props;
        self
    }
    pub fn description_props(mut self, props: super::props::TextProps) -> Self {
        self.description_props = props;
        self
    }
    pub fn message_props(mut self, props: super::props::TextProps) -> Self {
        self.message_props = props;
        self
    }
    pub fn new(label: impl Into<String>, control: impl Into<Element<'a, M>>) -> Self {
        Self {
            label_props: Default::default(),
            description_props: Default::default(),
            message_props: Default::default(),
            label: label.into(),
            control: control.into(),
            description: None,
            validity: Validity::Ok,
            required: false,
            touched: false,
            submitted: false,
            disabled: false,
            inline: false,
            plain_label: false,
            hide_label: false,
            show_valid: false,
            layout: InputWrapperLayout::Stacked,
            spacing: None,
            label_spacing: None,
            inline_padding: None,
            metrics: Metrics::desktop(),
            target_geometry: None,
        }
    }
    /// Resolve the actual control's corners at draw time, using its current theme and
    /// laid-out bounds. Return `None` for unknown geometry: no outline is painted, but
    /// validation messages and reserved layout space are unchanged. This does not discover
    /// or modify the control's style, state, events or validation.
    ///
    /// For inline controls these bounds describe the wrapper-owned padded frame instead.
    pub fn target_geometry(
        mut self,
        resolve: impl Fn(&Theme, Rectangle) -> Option<iced::border::Radius> + 'a,
    ) -> Self {
        self.target_geometry = Some(Box::new(resolve));
        self
    }

    /// Share the final container style resolver with the target. Only its border corners
    /// are read; callers must supply the same effective style (including controlled state)
    /// as the target. Backgrounds, borders, events and validation are not overridden.
    pub fn target_style(self, style: impl Fn(&Theme) -> container::Style + 'a) -> Self {
        self.target_geometry(move |theme, _| Some(style(theme).border.radius))
    }

    /// Outline a known target without a label or message. The resolver receives the actual
    /// target bounds, not the expanded outline bounds. `None` suppresses painting, not padding.
    pub fn outline_target(
        control: impl Into<Element<'a, M>>,
        presentation: Presentation,
        metrics: Metrics,
        resolve: impl Fn(&Theme, Rectangle) -> Option<iced::border::Radius> + 'a,
    ) -> Element<'a, M> {
        outline_resolved(
            control.into(),
            presentation,
            metrics,
            Box::new(resolve),
            false,
        )
    }

    /// The gap between the label, the control and the note under it (default `space_xxs`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// In the row layout, the gap between the label and the control (default `space_m`).
    pub fn label_spacing(mut self, px: f32) -> Self {
        self.label_spacing = Some(px);
        self
    }
    /// The padding around an inline control (default `space_xxs` above and below, `space_xs`
    /// at the sides).
    pub fn inline_padding(mut self, p: impl Into<Padding>) -> Self {
        self.inline_padding = Some(p.into());
        self
    }
    /// Also draw the valid state (a success ring). Off by default: like CSS, only
    /// `:user-invalid` (and a warning) is highlighted, and `:user-valid` adds nothing special.
    pub fn show_valid(mut self, on: bool) -> Self {
        self.show_valid = on;
        self
    }
    /// Help text under the label (stacked) or the control; replaced by the message while one
    /// is shown.
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    /// The current [`Validity`] of the value (shown once touched or submitted).
    pub fn validity(mut self, v: Validity) -> Self {
        self.validity = v;
        self
    }
    /// Shorthand for `.validity(Validity::Error(msg))`, or `Ok` for `None`.
    pub fn error(mut self, msg: Option<impl Into<String>>) -> Self {
        self.validity = msg.map_or(Validity::Ok, |m| Validity::Error(m.into()));
        self
    }
    /// Marks the field required (an asterisk after the label).
    pub fn required(mut self, r: bool) -> Self {
        self.required = r;
        self
    }
    /// The user has edited or left the control.
    pub fn touched(mut self, t: bool) -> Self {
        self.touched = t;
        self
    }
    /// A submit was attempted: reveals validity like `touched`.
    pub fn submitted(mut self, s: bool) -> Self {
        self.submitted = s;
        self
    }
    /// Sets `touched` and `submitted` together from an [`Interaction`], e.g. the one a
    /// [`FormState`](crate::primitives::FormState) computes for this field's key.
    pub fn interaction(mut self, i: Interaction) -> Self {
        self.touched = i.touched;
        self.submitted = i.submitted;
        self
    }
    /// Dims the label and description and hides validation. Disable the control itself by
    /// leaving off its `on_*` callbacks.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// The control has no box of its own (check or radio button, scale): pad it inside the
    /// outline so the ring does not touch its content. This wrapper-owned frame uses
    /// `theme.metrics.radius_m` unless `target_geometry` / `target_style` overrides it;
    /// its frame is not a claim about the enclosed control's shape.
    pub fn inline(mut self, i: bool) -> Self {
        self.inline = i;
        self
    }
    /// Draws the label in the body size and regular weight instead of
    /// the bold form label: for a plain list of controls with their names (a settings card,
    /// a gallery of states), where bold labels would shout.
    pub fn plain_label(mut self, plain: bool) -> Self {
        self.plain_label = plain;
        self
    }
    /// The label is already inside a composite control (for example EntryRow).
    pub(crate) fn hide_label(mut self) -> Self {
        self.hide_label = true;
        self
    }
    pub fn layout(mut self, l: InputWrapperLayout) -> Self {
        self.layout = l;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    fn revealed(&self) -> bool {
        !self.disabled && (self.touched || self.submitted)
    }
    /// What this field currently shows.
    pub fn presentation(&self) -> Presentation {
        validation::present(&self.validity, self.revealed())
    }
}

/// The outline color for a presentation, if any.
fn ring_color(t: &Theme, p: Presentation) -> Option<iced::Color> {
    match p {
        Presentation::Pristine => None,
        Presentation::Valid => Some(t.semantic.success.color),
        Presentation::Warning => Some(t.semantic.warning.color),
        Presentation::Invalid => Some(t.semantic.error.color),
    }
}

/// Wraps a known control in a validation outline, reserving space while pristine.
/// The resolver receives the actual target bounds and current theme. Return `None`
/// for unknown geometry rather than guessing the control's corners.
pub fn outlined<'a, M: 'a>(
    control: impl Into<Element<'a, M>>,
    p: Presentation,
    m: Metrics,
    resolve: impl Fn(&Theme, Rectangle) -> Option<iced::border::Radius> + 'a,
) -> Element<'a, M> {
    InputWrapper::outline_target(control, p, m, resolve)
}

fn outline_resolved<'a, M: 'a>(
    control: Element<'a, M>,
    p: Presentation,
    m: Metrics,
    geometry: TargetGeometry<'a>,
    inset: bool,
) -> Element<'a, M> {
    let width = if inset {
        Length::Fill
    } else {
        match control.as_widget().size().width {
            Length::Shrink | Length::Fixed(_) => Length::Shrink,
            _ => Length::Fill,
        }
    };
    Element::new(ValidationOutline {
        content: container(control)
            .padding(if inset { 0.0 } else { m.focus_ring_width })
            .width(width)
            .into(),
        presentation: p,
        width: m.focus_ring_width,
        geometry,
        inset,
    })
}

// Match the renderer's clamping before expanding: e.g. a 999px pill on a 30px
// control has an effective 15px radius, not 999px on the enlarged outline.
fn outside_radius(
    radius: iced::border::Radius,
    bounds: Rectangle,
    spread: f32,
) -> iced::border::Radius {
    let limit = bounds.width.min(bounds.height).max(0.0) / 2.0;
    let expand = |r: f32| {
        let r = if r.is_finite() {
            r.max(0.0).min(limit)
        } else {
            0.0
        };
        if r == 0.0 { 0.0 } else { r + spread }
    };
    iced::border::Radius {
        top_left: expand(radius.top_left),
        top_right: expand(radius.top_right),
        bottom_right: expand(radius.bottom_right),
        bottom_left: expand(radius.bottom_left),
    }
}

struct ValidationOutline<'a, M> {
    content: Element<'a, M>,
    presentation: Presentation,
    width: f32,
    geometry: TargetGeometry<'a>,
    inset: bool,
}

impl<M> Widget<M, Theme, iced::Renderer> for ValidationOutline<'_, M> {
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }
    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }
    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }
    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let target = layout
            .children()
            .next()
            .expect("outline container child")
            .bounds();
        // An inset ring overlays the target's own opaque surface. Outside rings
        // retain the historical paint order and never obscure the control.
        if self.inset {
            self.content
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, viewport);
        }
        if let Some(color) = ring_color(theme, self.presentation) {
            if let Some(radius) = (self.geometry)(theme, target) {
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle {
                            x: target.x - if self.inset { 0.0 } else { self.width },
                            y: target.y - if self.inset { 0.0 } else { self.width },
                            width: target.width + if self.inset { 0.0 } else { 2.0 * self.width },
                            height: target.height + if self.inset { 0.0 } else { 2.0 * self.width },
                        },
                        border: Border {
                            color,
                            width: self.width,
                            radius: outside_radius(
                                radius,
                                target,
                                if self.inset { 0.0 } else { self.width },
                            ),
                        },
                        ..Quad::default()
                    },
                    Color::TRANSPARENT,
                );
            }
        }
        if !self.inset {
            self.content
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, viewport);
        }
    }
}

impl<'a, M: 'a> From<InputWrapper<'a, M>> for Element<'a, M> {
    fn from(f: InputWrapper<'a, M>) -> Self {
        let m = f.metrics;
        // Valid is a state the app can read, but by default nothing is drawn for it.
        let p = match f.presentation() {
            Presentation::Valid if !f.show_valid => Presentation::Pristine,
            p => p,
        };
        let gap = f.spacing.unwrap_or(m.space_xxs);
        let msg = validation::message(&f.validity, f.revealed()).map(str::to_owned);
        let disabled = f.disabled;
        // An inline frame belongs to this wrapper; arbitrary controls do not. Never
        // infer a control's corners from an unrelated theme metric.
        let geometry: TargetGeometry<'a> = f.target_geometry.unwrap_or_else(|| {
            if f.inline {
                Box::new(|theme, _| Some(theme.metrics.radius_m.into()))
            } else {
                Box::new(|_, _| None)
            }
        });
        let control: Element<'a, M> = if f.inline {
            container(f.control)
                .padding(
                    f.inline_padding
                        .unwrap_or(Padding::from([m.space_xxs, m.space_xs])),
                )
                .into()
        } else {
            f.control
        };

        let dim = move |t: &Theme| {
            if disabled {
                crate::color::fade(t.semantic.window.fg, t.semantic.opacity.disabled)
            } else {
                t.semantic.window.fg
            }
        };

        // Label (+ required marker).
        let label = if f.plain_label {
            crate::Typography::desktop().body(f.label)
        } else {
            text(f.label)
                .size(crate::Typography::desktop().small)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..iced::Font::DEFAULT
                })
        }
        .style(move |t: &Theme| text::Style {
            color: Some(dim(t)),
        });
        let mut title = row![f.label_props.apply(label)].spacing(m.space_xxs);
        title = title.push(if f.required {
            Element::from(text("*").size(crate::Typography::desktop().small).style(
                move |t: &Theme| text::Style {
                    color: Some(if disabled {
                        dim(t)
                    } else {
                        t.semantic.error.color
                    }),
                },
            ))
        } else {
            space().into()
        });

        // Description or message, always one element so positions are stable.
        let has_note = msg.is_some() || f.description.is_some();
        let note: Element<'a, M> =
            if let Some(msg) = msg {
                let invalid = p == Presentation::Invalid;
                let icon = if invalid {
                    Icon::DialogError
                } else {
                    Icon::DialogWarning
                };
                let size = crate::Typography::desktop().small;
                row![
                    icon.view(size)
                        .style(move |t: &Theme, _| iced::widget::svg::Style {
                            color: ring_color(t, p),
                        }),
                    f.message_props
                        .apply(text(msg).size(size).style(move |t: &Theme| text::Style {
                            color: ring_color(t, p),
                        }))
                ]
                .spacing(m.space_xs)
                .align_y(Alignment::Center)
                .into()
            } else if let Some(d) = f.description {
                f.description_props
                    .apply(text(d).size(crate::Typography::desktop().small).style(
                        move |t: &Theme| text::Style {
                            color: Some(crate::color::fade(
                                t.semantic.window.fg,
                                if disabled {
                                    t.semantic.opacity.disabled * 0.6
                                } else {
                                    0.7
                                },
                            )),
                        },
                    ))
            } else {
                space().into()
            };
        let note = container(note).padding(Padding {
            // A row with nothing to say below it takes no space for it.
            top: if has_note || (!f.hide_label && f.layout == InputWrapperLayout::Stacked) {
                gap
            } else {
                0.0
            },
            ..Padding::ZERO
        });

        if f.hide_label {
            // EntryRow already supplies the row padding. Draw validation over that row
            // without reserving an extra inset or changing its alignment with siblings.
            let control = outline_resolved(control, p, m, geometry, true);
            let note = container(note).padding(iced::Padding {
                left: m.row_padding.left,
                right: m.row_padding.right,
                ..iced::Padding::ZERO
            });
            return column![control, note].width(Length::Fill).into();
        }
        match f.layout {
            InputWrapperLayout::Stacked => column![
                title,
                container(outline_resolved(control, p, m, geometry, false)).padding(Padding {
                    top: gap,
                    ..Padding::ZERO
                }),
                note
            ]
            .width(Length::Fill)
            .into(),
            InputWrapperLayout::Row => {
                let (start, end) = row_widths(control.as_widget().size().width);
                column![
                    row![
                        container(title).width(start),
                        container(outline_resolved(control, p, m, geometry, false))
                            .width(end)
                            .align_x(Alignment::End),
                    ]
                    .spacing(f.label_spacing.unwrap_or(m.space_m))
                    .align_y(Alignment::Center),
                    note
                ]
                .width(Length::Fill)
                .into()
            }
        }
    }
}

/// The widths of the title and the control in a [`InputWrapperLayout::Row`]. A control that hugs
/// its content (a switch, a dropdown, a fixed-width input) keeps its size and the title takes
/// the rest, so a long title wraps only where it must; a control that fills (a slider, an
/// entry) shares the row, a third for the title and two thirds for the control.
fn row_widths(control: Length) -> (Length, Length) {
    match control {
        Length::Shrink | Length::Fixed(_) => (Length::Fill, Length::Shrink),
        _ => (Length::Fill, Length::FillPortion(2)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field() -> InputWrapper<'static, ()> {
        InputWrapper::new("Name", iced::widget::space()).validity(Validity::error("x"))
    }

    #[test]
    fn pristine_until_touched_or_submitted() {
        assert_eq!(field().presentation(), Presentation::Pristine);
        assert_eq!(field().touched(true).presentation(), Presentation::Invalid);
        assert_eq!(
            field().submitted(true).presentation(),
            Presentation::Invalid
        );
    }

    #[test]
    fn row_gives_a_hugging_control_its_size() {
        assert_eq!(row_widths(Length::Shrink), (Length::Fill, Length::Shrink));
        assert_eq!(
            row_widths(Length::Fixed(260.0)),
            (Length::Fill, Length::Shrink)
        );
        assert_eq!(
            row_widths(Length::Fill),
            (Length::Fill, Length::FillPortion(2))
        );
    }

    #[test]
    fn disabled_never_shows_validation() {
        assert_eq!(
            field().touched(true).disabled(true).presentation(),
            Presentation::Pristine
        );
    }
}

#[cfg(test)]
mod spacing_params {
    use super::*;

    #[test]
    fn wrapper_gaps_and_padding_are_parameters() {
        let w = InputWrapper::<()>::new("L", iced::widget::space())
            .spacing(1.0)
            .label_spacing(2.0)
            .inline_padding(3.0);
        assert_eq!((w.spacing, w.label_spacing), (Some(1.0), Some(2.0)));
        let _: Element<'_, ()> = w.into();
    }
}

#[cfg(test)]
mod target_shape_tests {
    use super::*;
    use iced::advanced::{clipboard, renderer::Headless};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn renderer() -> iced::Renderer {
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer")
    }

    fn viewport() -> Rectangle {
        Rectangle::with_size(Size::new(160.0, 100.0))
    }

    fn prepare<M>(element: &mut Element<'_, M>, renderer: &iced::Renderer) -> (Tree, layout::Node) {
        let mut tree = Tree::new(element.as_widget());
        let node = element.as_widget_mut().layout(
            &mut tree,
            renderer,
            &layout::Limits::new(Size::ZERO, viewport().size()),
        );
        (tree, node)
    }

    fn draw<M>(
        element: &Element<'_, M>,
        tree: &Tree,
        node: &layout::Node,
        renderer: &mut iced::Renderer,
        theme: &Theme,
    ) -> Vec<u8> {
        renderer.reset(viewport());
        element.as_widget().draw(
            tree,
            renderer,
            theme,
            &renderer::Style {
                text_color: Color::BLACK,
            },
            Layout::new(node),
            mouse::Cursor::Unavailable,
            &viewport(),
        );
        renderer.screenshot(Size::new(160, 100), 1.0, Color::WHITE)
    }

    #[test]
    fn expanded_radius_preserves_square_and_clamps_pill_before_expansion() {
        let bounds = Rectangle::with_size(Size::new(80.0, 30.0));
        assert_eq!(outside_radius(0.0.into(), bounds, 3.0), 0.0.into());
        assert_eq!(outside_radius(999.0.into(), bounds, 3.0), 18.0.into());
        assert_eq!(
            outside_radius(
                iced::border::Radius {
                    top_left: 8.0,
                    top_right: 0.0,
                    bottom_left: 999.0,
                    bottom_right: 2.0,
                },
                bounds,
                3.0
            ),
            iced::border::Radius {
                top_left: 11.0,
                top_right: 0.0,
                bottom_left: 18.0,
                bottom_right: 5.0,
            }
        );
    }

    #[test]
    fn rendered_outline_matches_current_square_pill_asymmetric_and_outside_geometry() {
        let mut renderer = renderer();
        let metrics = Metrics {
            focus_ring_width: 3.0,
            ..Metrics::default()
        };
        for asymmetric in [false, true] {
            for inset in [false, true] {
                let mut element: Element<'_, ()> = outline_resolved(
                    container(space())
                        .width(80)
                        .height(30)
                        .style(move |theme: &Theme| container::Style {
                            background: Some(Color::BLACK.into()),
                            border: Border::default().rounded(if asymmetric {
                                iced::border::Radius {
                                    top_left: theme.metrics.radius_m,
                                    top_right: 0.0,
                                    bottom_right: 4.0,
                                    bottom_left: 0.0,
                                }
                            } else {
                                theme.metrics.radius_m.into()
                            }),
                            ..Default::default()
                        })
                        .into(),
                    Presentation::Invalid,
                    metrics,
                    Box::new(move |theme, bounds| {
                        assert_eq!(bounds.size(), Size::new(80.0, 30.0));
                        Some(if asymmetric {
                            iced::border::Radius {
                                top_left: theme.metrics.radius_m,
                                top_right: 0.0,
                                bottom_right: 4.0,
                                bottom_left: 0.0,
                            }
                        } else {
                            theme.metrics.radius_m.into()
                        })
                    }),
                    inset,
                );
                let (tree, node) = prepare(&mut element, &renderer);
                // Reuse the same view/tree/layout while the theme changes.
                for radius in [0.0, 8.0, 999.0] {
                    let mut theme = Theme::light();
                    theme.metrics.radius_m = radius;
                    theme.semantic.error.color = Color::from_rgb(0.8, 0.1, 0.2);
                    let actual = draw(&element, &tree, &node, &mut renderer, &theme);
                    let spread = if inset { 0.0 } else { 3.0 };
                    let grown = if radius == 0.0 {
                        0.0
                    } else {
                        radius.min(15.0) + spread
                    };
                    let expected_radius = if asymmetric {
                        iced::border::Radius {
                            top_left: grown,
                            top_right: 0.0,
                            bottom_right: 4.0 + spread,
                            bottom_left: 0.0,
                        }
                    } else {
                        grown.into()
                    };
                    renderer.reset(viewport());
                    let target_quad = Quad {
                        bounds: Rectangle {
                            x: spread,
                            y: spread,
                            width: 80.0,
                            height: 30.0,
                        },
                        border: Border::default().rounded(if asymmetric {
                            iced::border::Radius {
                                top_left: radius,
                                top_right: 0.0,
                                bottom_right: 4.0,
                                bottom_left: 0.0,
                            }
                        } else {
                            radius.into()
                        }),
                        ..Quad::default()
                    };
                    if inset {
                        renderer.fill_quad(target_quad, Color::BLACK);
                    }
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::with_size(if inset {
                                Size::new(80.0, 30.0)
                            } else {
                                Size::new(86.0, 36.0)
                            }),
                            border: Border {
                                color: theme.semantic.error.color,
                                width: 3.0,
                                radius: expected_radius,
                            },
                            ..Quad::default()
                        },
                        Color::TRANSPARENT,
                    );
                    if !inset {
                        renderer.fill_quad(target_quad, Color::BLACK);
                    }
                    let expected = renderer.screenshot(Size::new(160, 100), 1.0, Color::WHITE);
                    assert_eq!(
                        actual, expected,
                        "radius={radius}, asymmetric={asymmetric}, inset={inset}"
                    );
                }
            }
        }
    }

    #[test]
    fn unknown_target_keeps_layout_but_never_paints_a_guessed_ring() {
        let mut renderer = renderer();
        let metrics = Metrics {
            focus_ring_width: 3.0,
            ..Metrics::default()
        };
        let mut element = InputWrapper::<()>::outline_target(
            space().width(80).height(30),
            Presentation::Invalid,
            metrics,
            |_, _| None,
        );
        let (tree, node) = prepare(&mut element, &renderer);
        assert_eq!(node.size(), Size::new(86.0, 36.0));
        assert!(
            draw(&element, &tree, &node, &mut renderer, &Theme::light())
                .chunks_exact(4)
                .all(|pixel| pixel == [255, 255, 255, 255])
        );
        let mut known = outlined::<()>(
            space().width(80).height(30),
            Presentation::Invalid,
            metrics,
            |_, _| Some(0.0.into()),
        );
        let (tree, known_node) = prepare(&mut known, &renderer);
        assert_eq!(known_node.size(), node.size());
        assert!(
            draw(&known, &tree, &known_node, &mut renderer, &Theme::light())
                .chunks_exact(4)
                .any(|pixel| pixel != [255, 255, 255, 255])
        );
        let field = InputWrapper::<()>::new("", space());
        assert!(field.target_geometry.is_none());
    }

    #[test]
    fn all_wrapper_branches_resolve_actual_frame_and_preserve_parent_validation() {
        let mut renderer = renderer();
        for hidden in [false, true] {
            for inline in [false, true] {
                for layout in [InputWrapperLayout::Stacked, InputWrapperLayout::Row] {
                    for (touched, submitted, disabled, expected) in [
                        (false, false, false, false),
                        (true, false, false, true),
                        (false, true, false, true),
                        (true, true, true, false),
                    ] {
                        let seen = Rc::new(RefCell::new(vec![]));
                        let capture = seen.clone();
                        let metrics = Metrics {
                            focus_ring_width: 3.0,
                            ..Metrics::default()
                        };
                        let mut field =
                            InputWrapper::<()>::new("Label", space().width(80).height(30))
                                .validity(Validity::error("Error"))
                                .touched(touched)
                                .submitted(submitted)
                                .disabled(disabled)
                                .inline(inline)
                                .inline_padding([2.0, 4.0])
                                .layout(layout)
                                .metrics(metrics)
                                .target_geometry(move |theme, bounds| {
                                    capture
                                        .borrow_mut()
                                        .push((bounds.size(), theme.metrics.radius_m));
                                    Some(0.0.into())
                                });
                        if hidden {
                            field = field.hide_label();
                        }
                        let mut element: Element<'_, ()> = field.into();
                        let (mut tree, node) = prepare(&mut element, &renderer);
                        let mut theme = Theme::light();
                        theme.metrics.radius_m = 19.0;
                        let _ = draw(&element, &tree, &node, &mut renderer, &theme);
                        let calls = seen.borrow();
                        assert_eq!(calls.len(), usize::from(expected));
                        if expected {
                            assert_eq!(
                                calls[0],
                                (
                                    if inline {
                                        Size::new(88.0, 34.0)
                                    } else {
                                        Size::new(80.0, 30.0)
                                    },
                                    19.0
                                )
                            );
                        }
                        // Reconciliation retains the existing child tree.
                        let before = tree.children.len();
                        element.as_widget().diff(&mut tree);
                        assert_eq!(before, tree.children.len());
                    }
                }
            }
        }
    }

    #[test]
    fn target_style_and_default_props_do_not_replace_controlled_validation() {
        let style = |theme: &Theme| container::Style {
            border: Border::default().rounded(theme.metrics.radius_l),
            ..Default::default()
        };
        let field = InputWrapperProps::default().apply(
            InputWrapper::<()>::new("", space())
                .validity(Validity::error("parent"))
                .submitted(true)
                .target_style(style),
        );
        assert_eq!(field.presentation(), Presentation::Invalid);
        assert_eq!(field.validity, Validity::error("parent"));
        let mut theme = Theme::light();
        theme.metrics.radius_l = 23.0;
        assert_eq!(
            field.target_geometry.unwrap()(&theme, viewport()),
            Some(23.0.into())
        );
    }

    #[test]
    fn events_mouse_interaction_and_real_tooltip_overlay_survive_validity_diff() {
        let renderer = renderer();
        let make = |p| {
            InputWrapper::<i32>::outline_target(
                iced::widget::tooltip(
                    iced::widget::button(space())
                        .width(80)
                        .height(30)
                        .on_press(42),
                    text("Tooltip"),
                    iced::widget::tooltip::Position::Top,
                ),
                p,
                Metrics {
                    focus_ring_width: 3.0,
                    ..Metrics::default()
                },
                |_, _| Some(0.0.into()),
            )
        };
        let mut element = make(Presentation::Pristine);
        let (mut tree, node) = prepare(&mut element, &renderer);
        let cursor = mouse::Cursor::Available(iced::Point::new(30.0, 15.0));
        for p in [
            Presentation::Pristine,
            Presentation::Invalid,
            Presentation::Warning,
            Presentation::Valid,
        ] {
            element = make(p);
            tree.diff(element.as_widget());
            assert_eq!(
                element.as_widget().mouse_interaction(
                    &tree,
                    Layout::new(&node),
                    cursor,
                    &viewport(),
                    &renderer
                ),
                mouse::Interaction::Pointer
            );
            let mut messages = vec![];
            for event in [
                mouse::Event::CursorMoved {
                    position: iced::Point::new(30.0, 15.0),
                },
                mouse::Event::ButtonPressed(mouse::Button::Left),
                mouse::Event::ButtonReleased(mouse::Button::Left),
            ] {
                element.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(event),
                    Layout::new(&node),
                    cursor,
                    &renderer,
                    &mut clipboard::Null,
                    &mut Shell::new(&mut messages),
                    &viewport(),
                );
            }
            assert_eq!(messages, [42]);
            assert!(
                element
                    .as_widget_mut()
                    .overlay(
                        &mut tree,
                        Layout::new(&node),
                        &renderer,
                        &viewport(),
                        Vector::new(5.0, 7.0)
                    )
                    .is_some()
            );
        }
    }
}

#[cfg(test)]
mod target_state_tests {
    use super::*;
    use iced::advanced::{clipboard, renderer::Headless};

    #[test]
    fn validity_changes_keep_pressed_state_and_layout_without_changing_control_events() {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer");
        let metrics = Metrics {
            focus_ring_width: 3.0,
            ..Metrics::default()
        };
        let make = |p| {
            InputWrapper::<i32>::outline_target(
                iced::widget::button(space())
                    .width(80)
                    .height(30)
                    .on_press(7),
                p,
                metrics,
                |_, _| Some(0.0.into()),
            )
        };
        let viewport = Rectangle::with_size(Size::new(100.0, 60.0));
        let limits = layout::Limits::new(Size::ZERO, viewport.size());
        let mut element = make(Presentation::Pristine);
        let mut tree = Tree::new(element.as_widget());
        let node = element
            .as_widget_mut()
            .layout(&mut tree, &renderer, &limits);
        let cursor = mouse::Cursor::Available(iced::Point::new(20.0, 15.0));
        let mut messages = vec![];
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Layout::new(&node),
            cursor,
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &viewport,
        );
        for p in [
            Presentation::Invalid,
            Presentation::Warning,
            Presentation::Valid,
        ] {
            element = make(p);
            tree.diff(element.as_widget());
            let next = element
                .as_widget_mut()
                .layout(&mut tree, &renderer, &limits);
            assert_eq!(next.size(), node.size());
            assert_eq!(
                Layout::new(&next).children().next().unwrap().bounds(),
                Layout::new(&node).children().next().unwrap().bounds()
            );
        }
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            Layout::new(&node),
            cursor,
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &viewport,
        );
        assert_eq!(messages, [7]);
    }
}

/// Overrides for the text children of an input wrapper.
#[derive(Clone, Default)]
pub struct InputWrapperProps {
    pub label_props: super::props::TextProps,
    pub description_props: super::props::TextProps,
    pub message_props: super::props::TextProps,
}
impl InputWrapperProps {
    pub fn label_props(mut self, props: super::props::TextProps) -> Self {
        self.label_props = props;
        self
    }
    pub fn description_props(mut self, props: super::props::TextProps) -> Self {
        self.description_props = props;
        self
    }
    pub fn message_props(mut self, props: super::props::TextProps) -> Self {
        self.message_props = props;
        self
    }
    pub fn apply<'a, M: 'a>(&self, mut widget: InputWrapper<'a, M>) -> InputWrapper<'a, M> {
        widget.label_props.merge(&self.label_props);
        widget.description_props.merge(&self.description_props);
        widget.message_props.merge(&self.message_props);
        widget
    }
}
