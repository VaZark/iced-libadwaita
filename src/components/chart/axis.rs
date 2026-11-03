//! Gridlines and axes: the frame a cartesian chart sits in.

use super::{Anchor, LABEL_SIZE, ink, label};
use crate::Theme;
use crate::primitives::chart::scale::{Band, Linear};
use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Point, Rectangle};

/// Horizontal gridlines at `ticks`, and the value labels to the left of `plot`.
pub fn value_axis(
    frame: &mut Frame,
    theme: &Theme,
    plot: Rectangle,
    scale: &Linear,
    ticks: &[f32],
    labels: &[String],
) {
    let (text, grid) = ink(theme);
    for (t, l) in ticks.iter().zip(labels) {
        let y = scale.map(*t);
        frame.stroke(
            &Path::line(Point::new(plot.x, y), Point::new(plot.x + plot.width, y)),
            Stroke::default().with_width(1.0).with_color(grid),
        );
        label(
            frame,
            l.as_str(),
            Point::new(plot.x - 8.0, y),
            Anchor::End,
            text,
        );
    }
}

/// Vertical gridlines at `ticks`, and the value labels under `plot` (a horizontal value axis).
pub fn value_axis_bottom(
    frame: &mut Frame,
    theme: &Theme,
    plot: Rectangle,
    scale: &Linear,
    ticks: &[f32],
    labels: &[String],
) {
    let (text, grid) = ink(theme);
    for (t, l) in ticks.iter().zip(labels) {
        let x = scale.map(*t);
        frame.stroke(
            &Path::line(Point::new(x, plot.y), Point::new(x, plot.y + plot.height)),
            Stroke::default().with_width(1.0).with_color(grid),
        );
        label(
            frame,
            l.as_str(),
            Point::new(x, plot.y + plot.height + LABEL_SIZE + 2.0),
            Anchor::Middle,
            text,
        );
    }
}

/// A baseline along the bottom of `plot` and a label under each band. Labels that would collide
/// are thinned to every second, third… one.
pub fn band_axis(
    frame: &mut Frame,
    theme: &Theme,
    plot: Rectangle,
    band: &Band,
    labels: &[String],
) {
    let (text, _) = ink(theme);
    let base = plot.y + plot.height;
    frame.stroke(
        &Path::line(
            Point::new(plot.x, base),
            Point::new(plot.x + plot.width, base),
        ),
        Stroke::default()
            .with_width(1.0)
            .with_color(crate::color::alpha(text, 0.6)),
    );
    let widest = labels
        .iter()
        .map(|l| super::text_width(l))
        .fold(0.0f32, f32::max)
        + 8.0;
    let slot = if band.len() > 1 {
        band.center(1) - band.center(0)
    } else {
        plot.width
    };
    let every = (widest / slot.max(1.0)).ceil().max(1.0) as usize;
    for (i, l) in labels.iter().enumerate().take(band.len()) {
        if i % every == 0 {
            label(
                frame,
                l.as_str(),
                Point::new(band.center(i), base + LABEL_SIZE + 2.0),
                Anchor::Middle,
                text,
            );
        }
    }
}

/// A baseline along the left of `plot` and a label beside each band (a horizontal bar chart).
pub fn band_axis_left(
    frame: &mut Frame,
    theme: &Theme,
    plot: Rectangle,
    band: &Band,
    labels: &[String],
) {
    let (text, _) = ink(theme);
    frame.stroke(
        &Path::line(
            Point::new(plot.x, plot.y),
            Point::new(plot.x, plot.y + plot.height),
        ),
        Stroke::default()
            .with_width(1.0)
            .with_color(crate::color::alpha(text, 0.6)),
    );
    for (i, l) in labels.iter().enumerate().take(band.len()) {
        label(
            frame,
            l.as_str(),
            Point::new(plot.x - 8.0, band.center(i)),
            Anchor::End,
            text,
        );
    }
}

/// The left margin a value axis needs for its tick `labels`.
pub fn label_margin(labels: &[String]) -> f32 {
    labels
        .iter()
        .map(|l| super::text_width(l))
        .fold(0.0f32, f32::max)
        + 14.0
}
