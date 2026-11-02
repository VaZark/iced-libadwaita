//! `DateInput`: a date entry with a calendar popover.
//!
//! The text is typed as `YYYY-MM-DD` (parsed by [`Date::parse`]); the calendar button opens a
//! month grid to pick a day. Everything is controlled: you keep the text, the month on show and
//! the chosen date, and the widget reports `on_input(text)`, `on_pick(date)` and
//! `on_view(first-of-month)`. Text that does not parse, or is outside `min`/`max`, draws the
//! entry in its error state. Date arithmetic is dependency-free ([`Date`]).
//!
//! ```ignore
//! DateInput::new(&self.text, self.view, Message::Text)
//!     .selected(self.date)
//!     .on_pick(Message::Picked)
//!     .on_view(Message::View)
//! ```
//!
//! The calendar stays open while you page through months or pick a day (click outside or press
//! Escape to close it).

use super::dropdown::{Align, Appearance, Dropdown, popover};
use super::{Element, EntryState, entry};
use crate::primitives::date::{Date, MONTHS, WEEKDAYS};
use crate::style::button as sb;
use crate::{Icon, Metrics};
use iced::Length;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, space, text};

pub struct DateInput<'a, M> {
    text: &'a str,
    view: Date,
    on_input: Option<Box<dyn Fn(String) -> M + 'a>>,
    on_pick: Option<Box<dyn Fn(Date) -> M + 'a>>,
    on_view: Option<Box<dyn Fn(Date) -> M + 'a>>,
    selected: Option<Date>,
    min: Option<Date>,
    max: Option<Date>,
    disabled: bool,
    width: Length,
    open_initially: bool,
    metrics: Metrics,
}

impl<'a, M> DateInput<'a, M> {
    /// `text` is the entry's content, `view` any date in the month the calendar shows.
    pub fn new(text: &'a str, view: Date, on_input: impl Fn(String) -> M + 'a) -> Self {
        Self {
            text,
            view: view.month_start(),
            on_input: Some(Box::new(on_input)),
            on_pick: None,
            on_view: None,
            selected: None,
            min: None,
            max: None,
            disabled: false,
            width: Length::Fixed(220.0),
            open_initially: false,
            metrics: Metrics::desktop(),
        }
    }
    /// The chosen date, highlighted in the calendar.
    pub fn selected(mut self, d: Option<Date>) -> Self {
        self.selected = d;
        self
    }
    /// A day was clicked in the calendar.
    pub fn on_pick(mut self, f: impl Fn(Date) -> M + 'a) -> Self {
        self.on_pick = Some(Box::new(f));
        self
    }
    /// The previous / next month button was pressed; gets the first day of the new month.
    pub fn on_view(mut self, f: impl Fn(Date) -> M + 'a) -> Self {
        self.on_view = Some(Box::new(f));
        self
    }
    pub fn min(mut self, d: Date) -> Self {
        self.min = Some(d);
        self
    }
    pub fn max(mut self, d: Date) -> Self {
        self.max = Some(d);
        self
    }
    /// A disabled input is drawn dimmed and cannot be edited or opened.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn open_initially(mut self, open: bool) -> Self {
        self.open_initially = open;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    fn allowed(&self, d: Date) -> bool {
        self.min.is_none_or(|m| d >= m) && self.max.is_none_or(|m| d <= m)
    }

    /// Validation of the typed text: empty is neutral, an unparsable or out-of-range date is an
    /// error.
    pub fn text_state(&self) -> EntryState {
        if self.text.trim().is_empty() {
            return EntryState::Normal;
        }
        match Date::parse(self.text) {
            Some(d) if self.allowed(d) => EntryState::Normal,
            _ => EntryState::Error,
        }
    }
}

impl<'a, M: Clone + 'a> From<DateInput<'a, M>> for Element<'a, M> {
    fn from(mut d: DateInput<'a, M>) -> Self {
        let m = d.metrics;
        let state = d.text_state();
        let mut field = entry::<M>("YYYY-MM-DD", d.text)
            .state(state)
            .width(Length::Fill)
            .metrics(m);
        if !d.disabled
            && let Some(f) = d.on_input.take()
        {
            field = field.on_input(f);
        }
        let v = d.view;
        let nav = |icon: Icon, to: Date, on: &Option<Box<dyn Fn(Date) -> M + 'a>>| {
            let mut b = button(icon.view(m.icon_size))
                .padding(m.space_s)
                .style(sb::flat);
            if let Some(f) = on {
                b = b.on_press(f(to));
            }
            b
        };
        let header = row![
            nav(Icon::GoPrevious, v.prev_month(), &d.on_view),
            container(text(format!("{} {}", MONTHS[v.month as usize - 1], v.year)))
                .width(Length::Fill)
                .align_x(Horizontal::Center),
            nav(Icon::GoNext, v.next_month(), &d.on_view),
        ]
        .align_y(Vertical::Center);
        let cell = 34.0;
        let weekdays = row(WEEKDAYS.iter().map(|w| {
            container(text(*w).size(m.space_m).style(crate::style::text::dim))
                .center_x(Length::Fixed(cell))
                .into()
        }));
        let mut grid = column![weekdays].spacing(2);
        for week in v.month_grid() {
            grid = grid.push(
                row(week.iter().map(|slot| match slot {
                    None => space().width(cell).height(cell).into(),
                    Some(day) => {
                        let date = Date { day: *day, ..v };
                        let chosen = d.selected == Some(date);
                        let mut b = button(
                            container(text(day.to_string()))
                                .center_x(Length::Fixed(cell - 4.0))
                                .center_y(Length::Fixed(cell - 4.0)),
                        )
                        .padding(0)
                        .style(sb::menu_item(chosen));
                        if d.allowed(date)
                            && let Some(f) = &d.on_pick
                        {
                            b = b.on_press(f(date));
                        }
                        b.into()
                    }
                }))
                .spacing(2),
            );
        }
        let calendar = popover(
            [Element::from(column![header, grid].spacing(m.space_s))],
            400.0,
        );
        let opener = Dropdown::custom(Icon::GoDown.view(m.icon_size), calendar)
            .appearance(Appearance::Button)
            .align(Align::End)
            .popover_width(cell * 7.0 + 2.0 * 6.0 + 12.0 + 12.0)
            .keep_open(true)
            .disabled(d.disabled)
            .open_initially(d.open_initially);
        row![field, opener]
            .spacing(m.space_s)
            .align_y(Vertical::Center)
            .width(d.width)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u8, dd: u8) -> Date {
        Date::new(y, m, dd).unwrap()
    }

    #[test]
    fn text_state_flags_bad_and_out_of_range_dates() {
        let mk = |t| DateInput::<()>::new(t, d(2026, 11, 1), |_| ()).min(d(2026, 1, 1));
        assert_eq!(mk("").text_state(), EntryState::Normal);
        assert_eq!(mk("2026-11-02").text_state(), EntryState::Normal);
        assert_eq!(mk("2026-13-02").text_state(), EntryState::Error);
        assert_eq!(mk("2025-12-31").text_state(), EntryState::Error);
    }
}
