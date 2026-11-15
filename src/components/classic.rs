//! Classic marks: check boxes and radio boxes drawn pixel by pixel, for themes that ask for them.
//!
//! The toolkit's [`check_button`](crate::components::check_button) and
//! [`radio_button`](crate::components::radio_button) are iced widgets a theme can only restyle.
//! A theme that wants the classic sunken box with a tick (`[frame.check]`) or the round radio
//! box (`[frame.radio]`) sets a `border-style` for the frame; [`themed_check`] and
//! [`themed_radio`] then draw the marks themselves, and return the stock widget when it does not.
//!
//! Tokens, per frame (`[frame.check]`, `[frame.radio]`): `size` (the mark's edge, px: 13 and 12
//! unset), `bg` (the box fill), `glyph` (`tick`, `cross`, `square` for a check box; `dot` for a
//! radio box; `none`), and the bevel colors `border-light`, `border-dark`, `border-light-inner`,
//! `border-dark-inner`. A disabled mark's glyph and label are the `border-dark-inner` gray, and
//! with `[typography] disabled-style=emboss` the label gets a white copy one pixel down and right.

use crate::components::Pixels;
use crate::primitives::pixmap::{self, PixelMap};
use crate::{Element, Metrics};
use iced::alignment::Vertical;
use iced::widget::{button, row, stack, text};

/// Whether the installed theme draws check and radio boxes itself: `[frame.check]` or
/// `[frame.radio]` has a drawn `border-style`.
pub fn drawn_marks() -> bool {
    crate::components::bevel_of("frame.check").is_some()
        || crate::components::bevel_of("frame.radio").is_some()
}

fn size_of(section: &str, default: u32) -> u32 {
    crate::theme_config::current()
        .number(section, "size")
        .map_or(default, |s| s.round().max(4.0) as u32)
}

fn glyph_of(section: &str, default: &str) -> Option<PixelMap> {
    let cfg = crate::theme_config::current();
    pixmap::glyph(cfg.text(section, "glyph").unwrap_or(default))
}

/// The map of a check box: the sunken box, with its glyph when `checked`.
pub fn check_map(checked: bool) -> PixelMap {
    let size = size_of("frame.check", 13);
    compose_mark(
        pixmap::check_box(size),
        checked,
        glyph_of("frame.check", "tick"),
    )
}

/// The map of a radio box: the round box, with its dot when `selected`.
pub fn radio_map(selected: bool) -> PixelMap {
    let size = size_of("frame.radio", 12);
    compose_mark(
        pixmap::radio_box(size),
        selected,
        glyph_of("frame.radio", "dot"),
    )
}

fn compose_mark(boxed: PixelMap, selected: bool, glyph: Option<PixelMap>) -> PixelMap {
    match (selected, glyph) {
        (true, Some(g)) => {
            // Oversized glyphs start at the origin and are clipped by overlay.
            let ox = boxed.width().saturating_sub(g.width()) / 2;
            let oy = boxed.height().saturating_sub(g.height()) / 2;
            boxed.overlay(&g, ox, oy)
        }
        _ => boxed,
    }
}

fn tone(section: &str, key: &str, default: u32) -> iced::Color {
    crate::theme_config::current()
        .color(section, key)
        .unwrap_or_else(|| crate::color::hex(default))
}

/// A label that is disabled: the classic gray, and embossed (a white copy one pixel down and
/// right) when `[typography] disabled-style=emboss`.
pub fn disabled_label<'a, M: 'a>(label: impl Into<String>, section: &str) -> Element<'a, M> {
    let label = label.into();
    let gray = tone(section, "border-dark-inner", 0x808080);
    let cfg = crate::theme_config::current();
    if cfg.text("typography", "disabled-style") == Some("emboss") {
        let white = tone(section, "border-light", 0xffffff);
        stack![
            iced::widget::container(text(label.clone()).color(white)).padding(iced::Padding {
                top: 1.0,
                left: 1.0,
                right: 0.0,
                bottom: 0.0
            }),
            text(label).color(gray),
        ]
        .into()
    } else {
        text(label).color(gray).into()
    }
}

fn mark_row<'a, M: 'a>(mark: Element<'a, M>, label: Element<'a, M>) -> Element<'a, M> {
    row![mark, label]
        .spacing(Metrics::desktop().space_s)
        .align_y(Vertical::Center)
        .into()
}

/// A check box with a label: drawn by the theme's `[frame.check]` when it has a bevel, else the
/// stock check button. `on_toggle` `None` disables it.
pub fn themed_check<'a, M: Clone + 'a>(
    label: impl Into<String>,
    checked: bool,
    on_toggle: Option<impl Fn(bool) -> M + 'a>,
) -> Element<'a, M> {
    let label = label.into();
    if crate::components::bevel_of("frame.check").is_none() {
        let mut c = crate::components::check_button(label, checked);
        if let Some(f) = on_toggle {
            c = c.on_toggle(f);
        }
        return c.into();
    }
    let enabled = on_toggle.is_some();
    let mark: Element<'a, M> = Pixels::new(check_map(checked), "frame.check")
        .disabled(!enabled)
        .into();
    let text_el: Element<'a, M> = if enabled {
        text(label).into()
    } else {
        disabled_label(label, "frame.check")
    };
    let content = mark_row(mark, text_el);
    button(content)
        .padding(0)
        .style(|theme: &crate::Theme, _| button::Style {
            text_color: theme.semantic.window.fg,
            ..Default::default()
        })
        .on_press_maybe(on_toggle.map(|f| f(!checked)))
        .into()
}

/// A radio box with a label in a group: drawn by `[frame.radio]` when it has a bevel, else the
/// stock radio button.
pub fn themed_radio<'a, V, M>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_select: impl Fn(V) -> M + 'a,
) -> Element<'a, M>
where
    V: Copy + Eq + 'a,
    M: Clone + 'a,
{
    let label = label.into();
    if crate::components::bevel_of("frame.radio").is_none() {
        return crate::components::radio_button(label, value, selected, on_select).into();
    }
    let on = selected == Some(value);
    let mark: Element<'a, M> = Pixels::new(radio_map(on), "frame.radio").into();
    button(mark_row(mark, text(label).into()))
        .padding(0)
        .style(|theme: &crate::Theme, _| button::Style {
            text_color: theme.semantic.window.fg,
            ..Default::default()
        })
        .on_press(on_select(value))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_marks_clip_oversized_glyphs_without_underflow() {
        for size in 4..=8 {
            for boxed in [pixmap::check_box(size), pixmap::radio_box(size)] {
                for name in ["tick", "cross", "square", "dot"] {
                    let glyph = pixmap::glyph(name).expect("built-in glyph");
                    let expected = boxed.overlay(
                        &glyph,
                        boxed.width().saturating_sub(glyph.width()) / 2,
                        boxed.height().saturating_sub(glyph.height()) / 2,
                    );
                    let on = compose_mark(boxed.clone(), true, Some(glyph.clone()));
                    assert_eq!((on.width(), on.height()), (boxed.width(), boxed.height()));
                    assert_eq!(on, expected);
                    assert_eq!(compose_mark(boxed.clone(), false, Some(glyph)), boxed);
                    assert_eq!(compose_mark(boxed.clone(), true, None), boxed);
                }
            }
        }
    }

    #[test]
    fn without_a_theme_the_marks_are_the_stock_widgets() {
        assert!(!drawn_marks());
        let _: Element<'_, u8> = themed_check("Tile", true, Some(|_| 1u8));
        let _: Element<'_, u8> = themed_radio("Center", 1u8, Some(1u8), |_| 2u8);
    }

    #[test]
    fn the_maps_have_the_classic_sizes_and_a_glyph_only_when_on() {
        assert_eq!(
            (check_map(false).width(), check_map(false).height()),
            (13, 13)
        );
        assert_eq!(
            (radio_map(true).width(), radio_map(true).height()),
            (12, 12)
        );
        let on = check_map(true);
        let off = check_map(false);
        assert_ne!(on, off);
        // The tick (7 x 7, centered) puts its top-right pixel at (9, 3).
        assert_eq!(on.at(9, 3), Some(pixmap::Pixel::Ink));
        assert_eq!(off.at(9, 3), Some(pixmap::Pixel::Fill));
        assert_eq!(radio_map(true).at(5, 5), Some(pixmap::Pixel::Ink));
        assert_eq!(radio_map(false).at(5, 5), Some(pixmap::Pixel::Fill));
    }
}
