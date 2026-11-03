//! The tooltip box: a title and rows of `swatch  name  value`, kept inside the chart.

use super::{Anchor, LABEL_SIZE, label, text_width};
use crate::Theme;
use iced::widget::canvas::{Fill, Frame, Path, Stroke};
use iced::{Color, Point, Rectangle, Size};

/// One row of a tooltip.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub color: Option<Color>,
    pub name: String,
    pub value: String,
}

impl Row {
    pub fn new(color: Option<Color>, name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            color,
            name: name.into(),
            value: value.into(),
        }
    }
}

const PAD: f32 = 8.0;
const LINE: f32 = 18.0;

/// The size a tooltip with this `title` and `rows` takes.
pub fn measure(title: Option<&str>, rows: &[Row]) -> Size {
    let name = rows
        .iter()
        .map(|r| text_width(&r.name))
        .fold(0.0f32, f32::max);
    let value = rows
        .iter()
        .map(|r| text_width(&r.value))
        .fold(0.0f32, f32::max);
    let swatch = if rows.iter().any(|r| r.color.is_some()) {
        16.0
    } else {
        0.0
    };
    let body = swatch + name + if value > 0.0 { 16.0 + value } else { 0.0 };
    let w = body.max(title.map_or(0.0, text_width)) + PAD * 2.0;
    let lines = rows.len() + usize::from(title.is_some());
    Size::new(
        w,
        lines as f32 * LINE + PAD * 2.0 - (LINE - LABEL_SIZE) * 0.0,
    )
}

/// The top-left corner for a tooltip of `size` near `anchor`: right of and above it, flipped
/// to stay inside `bounds`.
pub fn place(anchor: Point, size: Size, bounds: Size) -> Point {
    let gap = 12.0;
    let x = if anchor.x + gap + size.width > bounds.width {
        anchor.x - gap - size.width
    } else {
        anchor.x + gap
    };
    let y = if anchor.y - gap - size.height < 0.0 {
        anchor.y + gap
    } else {
        anchor.y - gap - size.height
    };
    Point::new(
        x.max(0.0),
        y.max(0.0).min((bounds.height - size.height).max(0.0)),
    )
}

/// Draws the tooltip near `anchor`.
pub fn draw(frame: &mut Frame, theme: &Theme, anchor: Point, title: Option<&str>, rows: &[Row]) {
    let size = measure(title, rows);
    let at = place(anchor, size, frame.size());
    let pop = theme.semantic.popover;
    let rect = Path::rounded_rectangle(at, size, 6.0.into());
    frame.fill(&rect, Fill::from(pop.bg));
    frame.stroke(
        &rect,
        Stroke::default()
            .with_width(1.0)
            .with_color(theme.semantic.border),
    );
    let _ = Rectangle::new(at, size);
    let mut y = at.y + PAD + LINE / 2.0;
    if let Some(t) = title {
        label(frame, t, Point::new(at.x + PAD, y), Anchor::Start, pop.fg);
        y += LINE;
    }
    let swatch = rows.iter().any(|r| r.color.is_some());
    for r in rows {
        let mut x = at.x + PAD;
        if let Some(c) = r.color {
            frame.fill(
                &Path::rounded_rectangle(Point::new(x, y - 5.0), Size::new(10.0, 10.0), 2.0.into()),
                Fill::from(c),
            );
        }
        if swatch {
            x += 16.0;
        }
        label(
            frame,
            r.name.as_str(),
            Point::new(x, y),
            Anchor::Start,
            pop.fg,
        );
        label(
            frame,
            r.value.as_str(),
            Point::new(at.x + size.width - PAD, y),
            Anchor::End,
            pop.fg,
        );
        y += LINE;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_flips_to_stay_inside() {
        let size = Size::new(100.0, 40.0);
        let bounds = Size::new(300.0, 200.0);
        let p = place(Point::new(50.0, 100.0), size, bounds);
        assert!(p.x > 50.0 && p.y < 100.0);
        let p = place(Point::new(290.0, 10.0), size, bounds);
        assert!(p.x + size.width <= 290.0 && p.y > 10.0);
    }

    #[test]
    fn it_grows_with_its_rows() {
        let one = measure(Some("Jan"), &[Row::new(None, "Sales", "12")]);
        let two = measure(
            Some("Jan"),
            &[Row::new(None, "Sales", "12"), Row::new(None, "Cost", "4")],
        );
        assert!(two.height > one.height);
    }
}
