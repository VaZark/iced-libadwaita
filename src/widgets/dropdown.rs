//! `GtkDropDown` and the items of a popover list: a [`Popover`] whose trigger is a button
//! (label + chevron) and whose content is a `.menu` list below it: rounded surface, 6px padding,
//! items with a hover layer and a trailing check on the selected one.

use crate::Element;
use crate::styles::button as sb;
use crate::widgets::popover::{Popover, popover};
use crate::{Icon, Metrics, Theme};
use iced::Length;
use iced::alignment::Vertical;
use iced::widget::{button, container, row, space, text};

/// A menu item for a dropdown popover: label with a trailing check when selected. The
/// selected item is the active one, in the accent color.
pub fn menu_item<'a, M: Clone + 'a>(
    label: impl Into<String>,
    selected: bool,
    on_press: M,
) -> Element<'a, M> {
    menu_item_inner(label.into(), None, selected, Some(on_press), false)
}

/// A [`menu_item`] with `lead` (a colour dot, an icon) before the label.
pub fn menu_item_lead<'a, M: Clone + 'a>(
    label: impl Into<String>,
    lead: Option<Element<'a, M>>,
    selected: bool,
    on_press: M,
) -> Element<'a, M> {
    menu_item_inner(label.into(), lead, selected, Some(on_press), false)
}

/// A menu item that cannot be chosen: dimmed text, no hover or press effect, no message.
/// (Distinct from selected, which is accent-colored.)
pub fn menu_item_disabled<'a, M: Clone + 'a>(
    label: impl Into<String>,
    selected: bool,
) -> Element<'a, M> {
    menu_item_inner(label.into(), None, selected, None, false)
}

fn menu_item_inner<'a, M: Clone + 'a>(
    label: String,
    lead: Option<Element<'a, M>>,
    selected: bool,
    on_press: Option<M>,
    wrap: bool,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    let mut r = row![].spacing(m.space_m).align_y(Vertical::Center);
    if let Some(l) = lead {
        r = r.push(l);
    }
    r = r.push(
        text(label)
            .wrapping(if wrap {
                text::Wrapping::WordOrGlyph
            } else {
                text::Wrapping::None
            })
            .width(Length::Fill),
    );
    r = r.push(if selected {
        // The check follows the text color (accent when active, dimmed when disabled).
        let disabled = on_press.is_none();
        Element::from(Icon::Check.view(m.icon_size).style(move |t: &Theme, _| {
            iced::widget::svg::Style {
                color: Some(if disabled {
                    crate::color::fade(t.semantic.accent.color, t.semantic.opacity.disabled)
                } else {
                    t.semantic.accent.color
                }),
            }
        }))
    } else {
        space().width(m.icon_size).into()
    });
    let row = container(r);
    button(if wrap {
        // A wrapped label grows the row.
        row.padding([6.0, m.space_m])
    } else {
        row.padding([0.0, m.space_m])
            .center_y(Length::Fixed(32.0))
    })
    .padding(0)
    .width(Length::Fill)
    .style(sb::menu_item(selected))
    .on_press_maybe(on_press)
    .into()
}

/// A `GtkDropDown` that is insensitive: its value and arrow are dimmed and it cannot open.
pub fn dropdown_disabled<'a, T, M>(selected: Option<&T>) -> Popover<'a, M>
where
    T: ToString,
    M: Clone + 'a,
{
    let m = Metrics::desktop();
    let current = selected.map(ToString::to_string).unwrap_or_default();
    let trigger = container(
        row![
            text(current)
                .wrapping(text::Wrapping::None)
                .style(crate::styles::text::dim),
            Icon::GoDown
                .view(m.icon_size)
                .style(|t: &Theme, _| iced::widget::svg::Style {
                    color: Some(crate::color::fade(
                        t.semantic.window.fg,
                        t.semantic.opacity.disabled
                    )),
                })
        ]
        .spacing(m.space_s)
        .align_y(Vertical::Center),
    )
    .center_y(Length::Fixed(m.button_min_height - m.button_padding.y()));
    Popover::new(trigger, popover(Vec::<Element<'a, M>>::new(), 0.0)).disabled(true)
}

/// The shared core of [`dropdown`] and [`MultiSelect`](crate::widgets::MultiSelect): one
/// [`menu_item`] per option, checked by `is_selected`, sending `on_press(option)`.
pub(crate) fn option_items<'a, T, M>(
    options: &'a [T],
    is_selected: impl Fn(&T) -> bool,
    on_press: impl Fn(T) -> M,
) -> Vec<Element<'a, M>>
where
    T: ToString + Clone + 'a,
    M: Clone + 'a,
{
    options
        .iter()
        .map(|o| menu_item(o.to_string(), is_selected(o), on_press(o.clone())))
        .collect()
}

/// How wide a dropdown may get before its labels are cut, in pixels.
pub const DROPDOWN_MAX_WIDTH: f32 = 320.0;

/// What a character of a label takes, on average: how many fit in a width.
const CHAR_PX: f32 = 7.5;

/// How wide a dropdown is at least, in pixels (about 10 characters of label), so one with a short
/// value ("Open") is still something to aim at.
pub const DROPDOWN_MIN_WIDTH: f32 = 10.0 * CHAR_PX;

/// `label` cut to fit `px` wide, with an ellipsis (the text widget cannot draw one itself).
fn clip(label: &str, px: f32) -> String {
    let fits = (px / CHAR_PX).max(1.0) as usize;
    if label.chars().count() <= fits {
        label.to_string()
    } else {
        label
            .chars()
            .take(fits.saturating_sub(1))
            .collect::<String>()
            + "…"
    }
}

/// How a [`dropdown_with`] looks.
pub struct DropdownOptions<'a, T, M> {
    max_width: f32,
    min_width: f32,
    wrap: bool,
    lead_width: f32,
    lead: Option<Box<dyn Fn(&T) -> Option<Element<'a, M>> + 'a>>,
}

impl<T, M> Default for DropdownOptions<'_, T, M> {
    fn default() -> Self {
        Self {
            max_width: DROPDOWN_MAX_WIDTH,
            min_width: DROPDOWN_MIN_WIDTH,
            wrap: false,
            lead_width: 0.0,
            lead: None,
        }
    }
}

impl<'a, T, M> DropdownOptions<'a, T, M> {
    /// The widest the dropdown gets, button and list alike ([`DROPDOWN_MAX_WIDTH`] by default).
    /// Longer labels are cut with an ellipsis, or wrapped with [`wrap`](Self::wrap).
    pub fn max_width(mut self, px: f32) -> Self {
        self.max_width = px;
        self
    }
    /// The narrowest the label area gets, however short the options ([`DROPDOWN_MIN_WIDTH`], about
    /// ten characters, by default). The button and the list are never narrower than this plus
    /// their arrow, padding and lead.
    pub fn min_width(mut self, px: f32) -> Self {
        self.min_width = px;
        self
    }
    /// Wrap a label that is wider than [`max_width`](Self::max_width) onto more lines, breaking
    /// inside a word that does not fit, instead of cutting it with an ellipsis (the default).
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }
    /// An element before each label (a colour dot, an icon), in the button and in the list.
    /// `width` is how much room it takes, so the button is wide enough for it.
    pub fn lead(
        mut self,
        width: f32,
        lead: impl Fn(&T) -> Option<Element<'a, M>> + 'a,
    ) -> Self {
        self.lead_width = width;
        self.lead = Some(Box::new(lead));
        self
    }
}

/// `GtkDropDown` over a slice of options. The button and its list are as wide as the widest
/// option, so they do not change size with the value and never cut the one they show: at least
/// [`DROPDOWN_MIN_WIDTH`], at most [`DROPDOWN_MAX_WIDTH`] (see [`DropdownOptions`]).
pub fn dropdown<'a, T, M>(
    options: &'a [T],
    selected: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
) -> Popover<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    dropdown_with(options, selected, on_select, DropdownOptions::default())
}

/// A [`dropdown`] whose current value and items are each led by `lead(option)`: a colour dot or
/// an icon before the label, in the button and in the list. `lead_width` is its width.
pub fn dropdown_lead<'a, T, M>(
    options: &'a [T],
    selected: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
    lead_width: f32,
    lead: impl Fn(&T) -> Option<Element<'a, M>> + 'a,
) -> Popover<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    dropdown_with(
        options,
        selected,
        on_select,
        DropdownOptions::default().lead(lead_width, lead),
    )
}

/// A [`dropdown`] with [`DropdownOptions`].
pub fn dropdown_with<'a, T, M>(
    options: &'a [T],
    selected: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
    opts: DropdownOptions<'a, T, M>,
) -> Popover<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    let m = Metrics::desktop();
    // What the button and the list each add around a label: the arrow, and the list's padding,
    // lead, gap and check. The list is as wide as the button, so the button is sized for the
    // larger, and a label that fits one fits the other.
    let button_extra = m.icon_size + 2.0 * m.space_s;
    let list_extra = 3.0 * m.space_m + m.icon_size + iced::Padding::from(m.popover_padding).x();
    let extra = button_extra.max(list_extra);
    // What a label has room for: the width, less all that.
    let room = opts.max_width - extra - opts.lead_width;
    let label = |t: &T| {
        if opts.wrap {
            t.to_string()
        } else {
            clip(&t.to_string(), room)
        }
    };
    let wrapping = if opts.wrap {
        text::Wrapping::WordOrGlyph
    } else {
        text::Wrapping::None
    };

    let mut shown = row![].spacing(m.space_s).align_y(Vertical::Center);
    if let Some(l) = selected
        .as_ref()
        .and_then(|s| opts.lead.as_ref().and_then(|f| f(s)))
    {
        shown = shown.push(l);
    }
    let current = selected.as_ref().map(&label).unwrap_or_default();
    let visible = row![
        shown.push(text(current).wrapping(wrapping).width(Length::Fill)),
        Icon::GoDown.view(m.icon_size)
    ]
    .spacing(m.space_s)
    .align_y(Vertical::Center);

    // The widest option, laid out and never drawn (see `SizeProbe`). The button takes the width
    // of the widest of its content, and the visible row fills it. The minimum is one more row of
    // that width.
    let gap = if opts.lead_width > 0.0 { m.space_s } else { 0.0 };
    let mut rows: Vec<Element<'a, M>> = options
        .iter()
        .map(|o| {
            row![
                space().width(opts.lead_width),
                text(label(o)).wrapping(wrapping)            ,
                space().width(extra),
            ]
            .spacing(gap)
            .into()
        })
        .collect();
    rows.push(
        space()
            .width(opts.lead_width + gap + opts.min_width + extra)
            .into(),
    );
    let sizer = SizeProbe::new(iced::widget::column(rows));

    // The probe is the base layer, so the button is the size of the widest option; the visible
    // row fills it.
    let content = iced::widget::stack![sizer, visible.width(Length::Fill).height(Length::Fill)];
    let trigger = if opts.wrap {
        container(content).max_width(opts.max_width)
    } else {
        container(content)
            .max_width(opts.max_width)
            .center_y(Length::Fixed(m.button_min_height - m.button_padding.y()))
    };

    let items = options
        .iter()
        .map(|o| {
            menu_item_inner(
                label(o),
                opts.lead.as_ref().and_then(|f| f(o)),
                selected.as_ref() == Some(o),
                Some(on_select(o.clone())),
                opts.wrap,
            )
        })
        .collect::<Vec<_>>();
    Popover::new(trigger, popover(items, 280.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_label_is_cut_with_an_ellipsis_to_fit() {
        assert_eq!(clip("In progress", 200.0), "In progress");
        let cut = clip("A very long status name indeed", 90.0);
        assert!(cut.ends_with('…'));
        assert!(cut.chars().count() <= (90.0 / CHAR_PX) as usize);
        // Room for nothing still shows something.
        assert_eq!(clip("abc", 0.0), "…");
    }
}

/// Content that is laid out to take its size (as wide as the widest row, as tall as the tallest)
/// and draws nothing: how a dropdown learns the size of its widest option without showing it.
struct SizeProbe<'a, M> {
    content: Element<'a, M>,
}

impl<'a, M> SizeProbe<'a, M> {
    fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl<M> iced::advanced::Widget<M, Theme, iced::Renderer> for SizeProbe<'_, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.content.as_widget().tag()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        self.content.as_widget().state()
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.content.as_widget().children()
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        self.content.as_widget().diff(tree);
    }
    fn size(&self) -> iced::Size<Length> {
        iced::Size::new(Length::Shrink, Length::Shrink)
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        // The widest and tallest of the rows: one row's height when they are alike, the tallest
        // when some wrap.
        let node = self.content.as_widget_mut().layout(tree, renderer, limits);
        let tallest = node
            .children()
            .iter()
            .map(|c| c.size().height)
            .fold(0.0_f32, f32::max);
        iced::advanced::layout::Node::with_children(
            iced::Size::new(node.size().width, tallest),
            vec![node],
        )
    }
    fn draw(
        &self,
        _: &iced::advanced::widget::Tree,
        _: &mut iced::Renderer,
        _: &Theme,
        _: &iced::advanced::renderer::Style,
        _: iced::advanced::Layout<'_>,
        _: iced::advanced::mouse::Cursor,
        _: &iced::Rectangle,
    ) {
    }
}

impl<'a, M: 'a> From<SizeProbe<'a, M>> for Element<'a, M> {
    fn from(p: SizeProbe<'a, M>) -> Self {
        Element::new(p)
    }
}
