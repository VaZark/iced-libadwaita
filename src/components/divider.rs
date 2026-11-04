//! `Divider`: a line between things, after Mantine's [Divider](https://mantine.dev/core/divider/):
//! horizontal or vertical, solid, dashed or dotted, of any size, with an optional label (text,
//! an icon with text, a link: any element) at the start, center or end of the line.
//!
//! ```ignore
//! divider()                                           // a hairline
//! divider().variant(DividerVariant::Dashed).size(2.0)
//! divider().label(text("Search results")).label_position(LabelPosition::Center)
//! divider().vertical().spacing(8.0)                   // between items of a row
//! ```
//!
//! The line takes its color from a [`DividerTone`] (the theme's hairline colors, so it follows
//! the scheme). A vertical divider has no label, as in Mantine.

use crate::Element;
use crate::styles::container as sc;
use crate::{Metrics, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Tree, Widget};
use iced::alignment::Vertical;
use iced::widget::{container, row};
use iced::{Background, Border, Color, Length, Point, Rectangle, Shadow, Size, mouse};

/// Which theme color the line uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DividerTone {
    /// `--border-color`.
    #[default]
    Border,
    /// Between boxed-list rows (`--card-shade-color`).
    Card,
    /// Under or over bars (`--headerbar-shade-color`).
    Bar,
    /// Between popover sections (`--popover-shade-color`).
    Popover,
    /// Beside a sidebar (`--sidebar-border-color`).
    Sidebar,
}

impl DividerTone {
    /// The line's color in `theme`.
    fn color(self, theme: &Theme) -> Color {
        let style = match self {
            Self::Border => sc::separator,
            Self::Card => sc::card_separator,
            Self::Bar => sc::bar_separator,
            Self::Popover => sc::popover_separator,
            Self::Sidebar => sc::sidebar_separator,
        };
        match style(theme).background {
            Some(Background::Color(c)) => c,
            _ => Color::TRANSPARENT,
        }
    }
}

/// How the line is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DividerVariant {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

/// Where the label sits on a horizontal line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelPosition {
    /// At the start; the line runs on after it.
    Left,
    /// In the middle, with a line on each side.
    #[default]
    Center,
    /// At the end, after the line.
    Right,
}

/// The pieces of a line of `length`: `(start, length)` of each dash or dot. A solid line is one.
pub(crate) fn segments(length: f32, variant: DividerVariant, thickness: f32) -> Vec<(f32, f32)> {
    let t = thickness.max(1.0);
    let (on, off) = match variant {
        DividerVariant::Solid => return vec![(0.0, length.max(0.0))],
        // Dashes a few lines long, dots a line wide with a gap of two.
        DividerVariant::Dashed => (3.0 * t + 1.0, 3.0 * t),
        DividerVariant::Dotted => (t, 2.0 * t),
    };
    let mut out = Vec::new();
    let mut at = 0.0;
    while at < length {
        out.push((at, on.min(length - at)));
        at += on + off;
    }
    out
}

/// What color a [`Rule`] is drawn in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuleColor {
    Tone(DividerTone),
    /// The accent color (a completed stretch of a timeline).
    Accent,
}

/// The line itself: fills its axis, `thickness` across. Shared with [`Timeline`](crate::widgets::Timeline).
pub(crate) struct Rule {
    pub vertical: bool,
    pub thickness: f32,
    pub variant: DividerVariant,
    pub color: RuleColor,
}

impl<M, R: iced::advanced::Renderer> Widget<M, Theme, R> for Rule {
    fn size(&self) -> Size<Length> {
        if self.vertical {
            Size::new(Length::Fixed(self.thickness), Length::Fill)
        } else {
            Size::new(Length::Fill, Length::Fixed(self.thickness))
        }
    }

    fn layout(&mut self, _: &mut Tree, _: &R, limits: &layout::Limits) -> layout::Node {
        let size = if self.vertical {
            limits.resolve(
                Length::Fixed(self.thickness),
                Length::Fill,
                Size::new(self.thickness, 0.0),
            )
        } else {
            limits.resolve(
                Length::Fill,
                Length::Fixed(self.thickness),
                Size::new(0.0, self.thickness),
            )
        };
        layout::Node::new(size)
    }

    fn draw(
        &self,
        _: &Tree,
        renderer: &mut R,
        theme: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        _: &Rectangle,
    ) {
        let b = layout.bounds();
        let color = match self.color {
            RuleColor::Tone(t) => t.color(theme),
            RuleColor::Accent => theme.semantic.accent.bg,
        };
        let length = if self.vertical { b.height } else { b.width };
        let t = self.thickness;
        for (start, len) in segments(length, self.variant, t) {
            let bounds = if self.vertical {
                Rectangle::new(Point::new(b.x, b.y + start), Size::new(b.width, len))
            } else {
                Rectangle::new(Point::new(b.x + start, b.y), Size::new(len, b.height))
            };
            renderer.fill_quad(
                Quad {
                    bounds,
                    border: Border::default().rounded(if self.variant == DividerVariant::Dotted {
                        t / 2.0
                    } else {
                        0.0
                    }),
                    shadow: Shadow::default(),
                    snap: self.variant == DividerVariant::Solid,
                },
                color,
            );
        }
    }
}

impl<'a, M: 'a> From<Rule> for Element<'a, M> {
    fn from(r: Rule) -> Self {
        iced::Element::new(r)
    }
}

/// A line, horizontal by default. See the [module](self).
pub struct Divider<'a, M> {
    vertical: bool,
    tone: DividerTone,
    spacing: f32,
    size: f32,
    variant: DividerVariant,
    label: Option<Element<'a, M>>,
    label_position: LabelPosition,
}

/// A 1 px solid line.
pub fn divider<'a, M>() -> Divider<'a, M> {
    Divider {
        vertical: false,
        tone: DividerTone::Border,
        spacing: 0.0,
        size: 1.0,
        variant: DividerVariant::Solid,
        label: None,
        label_position: LabelPosition::Center,
    }
}

impl<'a, M> Divider<'a, M> {
    /// A vertical line, for between the items of a row. It has no label.
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }
    pub fn tone(mut self, tone: DividerTone) -> Self {
        self.tone = tone;
        self
    }
    /// Empty space on both sides of the line, across its direction.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
    /// The line's thickness in pixels (default 1).
    pub fn size(mut self, px: f32) -> Self {
        self.size = px.max(1.0);
        self
    }
    pub fn variant(mut self, v: DividerVariant) -> Self {
        self.variant = v;
        self
    }
    /// Text, an icon with text, a link: any element, drawn in the line.
    pub fn label(mut self, label: impl Into<Element<'a, M>>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn label_position(mut self, p: LabelPosition) -> Self {
        self.label_position = p;
        self
    }
}

impl<'a, M: 'a> From<Divider<'a, M>> for Element<'a, M> {
    fn from(d: Divider<'a, M>) -> Self {
        let rule = |vertical| -> Element<'a, M> {
            Rule {
                vertical,
                thickness: d.size,
                variant: d.variant,
                color: RuleColor::Tone(d.tone),
            }
            .into()
        };
        if d.vertical {
            return container(rule(true)).padding([0.0, d.spacing]).into();
        }
        let line: Element<'a, M> = match d.label {
            None => rule(false),
            Some(label) => {
                let gap = Metrics::desktop().space_s;
                let mut r = row![].spacing(gap).align_y(Vertical::Center);
                if d.label_position != LabelPosition::Left {
                    r = r.push(rule(false));
                }
                r = r.push(label);
                if d.label_position != LabelPosition::Right {
                    r = r.push(rule(false));
                }
                r.width(Length::Fill).into()
            }
        };
        container(line).padding([d.spacing, 0.0]).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_solid_line_is_one_piece() {
        assert_eq!(
            segments(100.0, DividerVariant::Solid, 1.0),
            vec![(0.0, 100.0)]
        );
        assert_eq!(segments(0.0, DividerVariant::Solid, 1.0), vec![(0.0, 0.0)]);
    }

    #[test]
    fn dashes_and_dots_repeat_along_the_line_and_stop_at_its_end() {
        let dashes = segments(50.0, DividerVariant::Dashed, 1.0);
        assert!(dashes.len() > 3);
        assert!(
            dashes
                .iter()
                .all(|(s, l)| *s >= 0.0 && s + l <= 50.0 + 1e-3)
        );
        let dots = segments(50.0, DividerVariant::Dotted, 2.0);
        assert!(dots.iter().all(|(_, l)| *l <= 2.0 + 1e-3));
        // The pieces are spaced by one period.
        assert!((dots[1].0 - dots[0].0 - 6.0).abs() < 1e-3);
    }

    #[test]
    fn a_thicker_line_has_longer_dashes() {
        let thin = segments(200.0, DividerVariant::Dashed, 1.0);
        let thick = segments(200.0, DividerVariant::Dashed, 4.0);
        assert!(thick[0].1 > thin[0].1);
    }

    #[test]
    fn size_is_at_least_one_pixel() {
        assert_eq!(divider::<()>().size(0.0).size, 1.0);
        assert_eq!(divider::<()>().size(3.0).size, 3.0);
    }

    #[test]
    fn a_labeled_divider_builds_in_each_position() {
        for p in [
            LabelPosition::Left,
            LabelPosition::Center,
            LabelPosition::Right,
        ] {
            let _: Element<'_, ()> = divider()
                .label(iced::widget::text("Label"))
                .label_position(p)
                .into();
        }
    }
}
