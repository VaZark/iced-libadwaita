//! `Timeline`: events in order along a line, after Mantine's
//! [Timeline](https://mantine.dev/core/timeline/). Each item has a bullet on the line, a title
//! and a body; the items up to the `active` one are drawn as done, and the line between two done
//! items is in the accent color.
//!
//! It shares the [`stepper`](crate::primitives::stepper) primitive with [`Stepper`](crate::widgets::Stepper):
//! an item's bullet is that primitive's [`Status`] (completed, current, upcoming, plus the
//! `invalid` and `loading` overrides) and the line after an item is its [`Fill`], so a sequence
//! reads the same whether it is a wizard or a history. The dashed and dotted lines are the
//! [`Divider`](crate::components::Divider)'s.
//!
//! ```ignore
//! Timeline::new(self.step)                         // the last item that is done
//!     .item(TimelineItem::new("Created").body(caption("2 hours ago")))
//!     .item(TimelineItem::new("Reviewed").body(caption("An hour ago")).line(DividerVariant::Dashed))
//!     .item(TimelineItem::new("Merged").loading(true))
//! ```
//!
//! [`reverse_active`](Timeline::reverse_active) counts the done items from the end instead (a
//! newest-first history), and [`align`](Timeline::align) puts the line on the right.

use crate::Element;
use crate::components::divider::{DividerVariant, Rule, RuleColor};
use crate::components::typography as ty;
use crate::primitives::stepper::{self, Fill, Status};
use crate::{Icon, Metrics, Theme};
use iced::alignment::Horizontal;
use iced::widget::{column, container, row, space};
use iced::{Background, Border, Color, Length};

/// Which side the line is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimelineAlign {
    /// The line on the left, the text after it.
    #[default]
    Left,
    /// The text first, the line on the right.
    Right,
}

/// One event of a [`Timeline`].
pub struct TimelineItem<'a, M> {
    title: String,
    body: Option<Element<'a, M>>,
    bullet: Option<Element<'a, M>>,
    line: DividerVariant,
    invalid: bool,
    loading: bool,
}

impl<'a, M> TimelineItem<'a, M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: None,
            bullet: None,
            line: DividerVariant::Solid,
            invalid: false,
            loading: false,
        }
    }
    /// Text, or any element, under the title.
    pub fn body(mut self, body: impl Into<Element<'a, M>>) -> Self {
        self.body = Some(body.into());
        self
    }
    /// What sits inside the bullet instead of the status mark: an icon, an avatar.
    pub fn bullet(mut self, bullet: impl Into<Element<'a, M>>) -> Self {
        self.bullet = Some(bullet.into());
        self
    }
    /// How the line after this item is drawn.
    pub fn line(mut self, variant: DividerVariant) -> Self {
        self.line = variant;
        self
    }
    /// The item failed: drawn with the error color (wins over loading).
    pub fn invalid(mut self, on: bool) -> Self {
        self.invalid = on;
        self
    }
    /// The item is in progress.
    pub fn loading(mut self, on: bool) -> Self {
        self.loading = on;
        self
    }
}

/// See the [module](self).
pub struct Timeline<'a, M> {
    items: Vec<TimelineItem<'a, M>>,
    active: usize,
    bullet_size: f32,
    line_width: f32,
    align: TimelineAlign,
    reverse: bool,
    metrics: Metrics,
}

/// The status of item `i` of `n`, by the stepper primitive; reversed, the done items are the
/// ones from the end up to `active`.
pub(crate) fn item_status(
    i: usize,
    n: usize,
    active: usize,
    reverse: bool,
    invalid: bool,
    loading: bool,
) -> Status {
    if reverse {
        stepper::status(n - 1 - i, n - 1 - active.min(n - 1), invalid, loading)
    } else {
        stepper::status(i, active, invalid, loading)
    }
}

/// How full the line after item `i` is (joining it to the next).
pub(crate) fn line_fill(i: usize, n: usize, active: usize, reverse: bool) -> Fill {
    if reverse {
        // In mirrored order the pair (i, i + 1) is the line after item `n - 2 - i`.
        match (n - 1).checked_sub(i + 1) {
            Some(_) => stepper::connector_fill(n - 2 - i, n - 1 - active.min(n - 1), n, 0.0),
            None => Fill::Empty,
        }
    } else {
        stepper::connector_fill(i, active, n, 0.0)
    }
}

impl<'a, M: 'a> Timeline<'a, M> {
    /// `active`: the index of the last item that is done (items after it are not yet).
    pub fn new(active: usize) -> Self {
        Self {
            items: Vec::new(),
            active,
            bullet_size: 24.0,
            line_width: 2.0,
            align: TimelineAlign::Left,
            reverse: false,
            metrics: Metrics::desktop(),
        }
    }
    pub fn item(mut self, item: TimelineItem<'a, M>) -> Self {
        self.items.push(item);
        self
    }
    /// The bullet's diameter (default 24).
    pub fn bullet_size(mut self, px: f32) -> Self {
        self.bullet_size = px.max(8.0);
        self
    }
    /// The line's thickness (default 2).
    pub fn line_width(mut self, px: f32) -> Self {
        self.line_width = px.max(1.0);
        self
    }
    pub fn align(mut self, a: TimelineAlign) -> Self {
        self.align = a;
        self
    }
    /// Count the done items from the end, for a newest-first history.
    pub fn reverse_active(mut self, on: bool) -> Self {
        self.reverse = on;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

/// The bullet: a circle drawn for the status, with the mark (or the custom content) inside.
fn bullet<'a, M: 'a>(
    status: Status,
    size: f32,
    custom: Option<Element<'a, M>>,
    m: &Metrics,
) -> Element<'a, M> {
    let inner = size * 0.55;
    let mark: Element<'a, M> = match (custom, status) {
        (Some(c), _) => c,
        (None, Status::Completed) => Icon::Check.colored(inner, Color::WHITE).into(),
        (None, Status::Invalid) => Icon::WindowClose.colored(inner, Color::WHITE).into(),
        (None, Status::Current) => container(space())
            .width(inner * 0.5)
            .height(inner * 0.5)
            .style(move |_: &Theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::WHITE)),
                border: Border::default().rounded(inner),
                ..Default::default()
            })
            .into(),
        (None, Status::Loading) => ty::caption("…").into(),
        (None, Status::Upcoming) => space().into(),
    };
    let _ = m;
    container(mark)
        .width(size)
        .height(size)
        .center(size)
        .style(move |t: &Theme| {
            let s = &t.semantic;
            let (fill, line) = match status {
                Status::Completed | Status::Current => (Some(s.accent.bg), s.accent.bg),
                Status::Invalid => (Some(s.error.bg), s.error.bg),
                Status::Loading => (None, s.accent.bg),
                Status::Upcoming => (None, s.border),
            };
            iced::widget::container::Style {
                background: fill.map(Background::Color),
                border: Border {
                    color: line,
                    width: 2.0,
                    radius: (size / 2.0).into(),
                },
                ..Default::default()
            }
        })
        .into()
}

impl<'a, M: 'a> From<Timeline<'a, M>> for Element<'a, M> {
    fn from(t: Timeline<'a, M>) -> Self {
        let m = t.metrics;
        let n = t.items.len();
        let mut col = column![].width(Length::Fill);
        for (i, item) in t.items.into_iter().enumerate() {
            let status = item_status(i, n, t.active, t.reverse, item.invalid, item.loading);
            let fill = line_fill(i, n, t.active, t.reverse);
            let last = i + 1 == n;
            let rail = column![
                bullet(status, t.bullet_size, item.bullet, &m),
                // The line fills what is left of the item's height.
                container(Element::from(Rule {
                    vertical: true,
                    thickness: t.line_width,
                    variant: item.line,
                    color: if fill == Fill::Full {
                        RuleColor::Accent
                    } else {
                        RuleColor::Tone(crate::components::DividerTone::Border)
                    },
                }))
                .height(if last { Length::Shrink } else { Length::Fill }),
            ]
            .align_x(Horizontal::Center)
            .width(t.bullet_size);

            let mut text = column![ty::heading(item.title)].spacing(m.space_xs);
            if let Some(body) = item.body {
                text = text.push(body);
            }
            // Space under every item but the last, so the line has something to cross.
            let text = container(text).padding(if last {
                iced::Padding::ZERO
            } else {
                iced::Padding::ZERO.bottom(m.space_l)
            });
            let line = match t.align {
                TimelineAlign::Left => row![rail, text.width(Length::Fill)],
                TimelineAlign::Right => {
                    row![text.width(Length::Fill).align_right(Length::Fill), rail]
                }
            }
            .spacing(m.space_m);
            col = col.push(line);
        }
        col.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_up_to_active_are_done_and_the_active_one_is_current() {
        let st = |i| item_status(i, 4, 1, false, false, false);
        assert_eq!(st(0), Status::Completed);
        assert_eq!(st(1), Status::Current);
        assert_eq!(st(2), Status::Upcoming);
    }

    #[test]
    fn the_overrides_are_the_steppers() {
        assert_eq!(
            item_status(3, 4, 1, false, true, true),
            Status::Invalid,
            "invalid wins"
        );
        assert_eq!(item_status(3, 4, 1, false, false, true), Status::Loading);
    }

    #[test]
    fn a_line_is_full_between_two_done_items_and_empty_after() {
        let f = |i| line_fill(i, 4, 2, false);
        assert_eq!(f(0), Fill::Full);
        assert_eq!(f(1), Fill::Full);
        assert_eq!(f(2), Fill::Empty, "after the active one");
        assert_eq!(f(3), Fill::Empty, "the last item has no line");
    }

    #[test]
    fn reversed_it_counts_from_the_end() {
        // Four items, active 1 reversed: items 3 and 2 are done, 1 is current, 0 not yet.
        let st = |i| item_status(i, 4, 1, true, false, false);
        assert_eq!(st(3), Status::Completed);
        assert_eq!(st(2), Status::Completed);
        assert_eq!(st(1), Status::Current);
        assert_eq!(st(0), Status::Upcoming);
        // Lines: full between two done items (2-3) and into the current one (1-2); the one
        // leading on to an upcoming item (0-1) is empty, and the last item has none.
        assert_eq!(line_fill(2, 4, 1, true), Fill::Full);
        assert_eq!(line_fill(1, 4, 1, true), Fill::Full);
        assert_eq!(line_fill(0, 4, 1, true), Fill::Empty);
        assert_eq!(line_fill(3, 4, 1, true), Fill::Empty);
    }

    #[test]
    fn a_timeline_of_any_items_builds_on_either_side() {
        for align in [TimelineAlign::Left, TimelineAlign::Right] {
            let t: Timeline<'_, ()> = Timeline::new(1)
                .align(align)
                .item(TimelineItem::new("One").body(iced::widget::text("body")))
                .item(TimelineItem::new("Two").line(DividerVariant::Dashed))
                .item(TimelineItem::new("Three").loading(true));
            let _: Element<'_, ()> = t.into();
        }
    }
}
