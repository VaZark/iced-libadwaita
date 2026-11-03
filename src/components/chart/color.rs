//! Categorical series colors: the GNOME palette's hues, in an order that keeps neighbours apart,
//! lighter on a dark scheme.

use crate::{Scheme, Theme};
use iced::Color;

const LIGHT: [u32; 8] = [
    0x3584e4, 0xe66100, 0x2ec27e, 0x9141ac, 0xf5c211, 0xc64600, 0x1a5fb4, 0x986a44,
];
const DARK: [u32; 8] = [
    0x78aeed, 0xffa348, 0x8ff0a4, 0xdc8add, 0xf9f06b, 0xff7800, 0x99c1f1, 0xcdab8f,
];

fn rgb(v: u32) -> Color {
    Color::from_rgb8((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

/// The color of series `i`. It repeats after eight.
pub fn series(theme: &Theme, i: usize) -> Color {
    let table = match theme.semantic.scheme {
        Scheme::Light => &LIGHT,
        Scheme::Dark => &DARK,
    };
    rgb(table[i % table.len()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_repeat_after_eight() {
        let t = Theme::light();
        assert_eq!(series(&t, 0), series(&t, 8));
        assert_ne!(series(&t, 0), series(&t, 1));
        assert_ne!(series(&t, 0), series(&Theme::dark(), 0));
    }
}
