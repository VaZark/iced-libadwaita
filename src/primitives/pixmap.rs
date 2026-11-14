//! Pixel maps: small bitmaps drawn one pixel at a time, with no drawing here.
//!
//! The classic desktop toolkits drew their glyphs (check marks, arrows, window-button symbols)
//! and the round radio box as fixed pixel patterns. A [`PixelMap`] is that pattern as rows of
//! characters; each character names a **tone** that the theme colors:
//!
//! ```text
//! ' ' or '.'  nothing        L  border-light         l  border-light-inner
//! F           the fill       D  border-dark          d  border-dark-inner
//! X           the ink (text color of the widget the map sits in)
//! ```
//!
//! The same module has the other pixel-exact rules a classic look needs: the 1px dither of a
//! scrollbar track, the dotted focus rectangle and the blocks of a progress bar. All of it is
//! pure geometry, so it is tested without a window.

use iced::{Point, Rectangle, Size};

/// What a pixel of a map is colored with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pixel {
    /// `border-light`.
    Light,
    /// `border-dark`.
    Dark,
    /// `border-light-inner`.
    LightInner,
    /// `border-dark-inner`.
    DarkInner,
    /// The fill of the frame the map is drawn for (`[frame.<name>] bg`).
    Fill,
    /// The ink: the text color of the widget the map is in.
    Ink,
}

impl Pixel {
    /// The tone a map character names; `None` for a transparent pixel.
    pub fn of(c: char) -> Option<Self> {
        match c {
            'L' => Some(Self::Light),
            'D' => Some(Self::Dark),
            'l' => Some(Self::LightInner),
            'd' => Some(Self::DarkInner),
            'F' => Some(Self::Fill),
            'X' => Some(Self::Ink),
            _ => None,
        }
    }
}

/// A rectangle of pixels, row by row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelMap {
    rows: Vec<String>,
}

/// A horizontal run of one tone: `width` pixels starting at `(x, y)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Run {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub pixel: Pixel,
}

impl PixelMap {
    /// A map from rows (all the same width; a shorter row is padded with nothing).
    pub fn new(rows: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            rows: rows.into_iter().map(Into::into).collect(),
        }
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as u32
    }
    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.rows.len() as u32
    }
    pub fn size(&self) -> Size {
        Size::new(self.width() as f32, self.height() as f32)
    }
    /// The pixel at `(x, y)`, `None` when transparent or outside.
    pub fn at(&self, x: u32, y: u32) -> Option<Pixel> {
        self.rows
            .get(y as usize)?
            .chars()
            .nth(x as usize)
            .and_then(Pixel::of)
    }

    /// The map as horizontal runs of one tone, so it can be drawn with few rectangles.
    pub fn runs(&self) -> Vec<Run> {
        let mut out = Vec::new();
        for (y, row) in self.rows.iter().enumerate() {
            let mut start: Option<(u32, Pixel)> = None;
            let mut x = 0u32;
            for c in row.chars() {
                let p = Pixel::of(c);
                match (start, p) {
                    (Some((_, a)), Some(b)) if a == b => {}
                    (Some((sx, a)), _) => {
                        out.push(Run { x: sx, y: y as u32, width: x - sx, pixel: a });
                        start = p.map(|b| (x, b));
                    }
                    (None, Some(b)) => start = Some((x, b)),
                    (None, None) => {}
                }
                x += 1;
            }
            if let Some((sx, a)) = start {
                out.push(Run { x: sx, y: y as u32, width: x - sx, pixel: a });
            }
        }
        out
    }

    /// The map placed in the middle of a `width` x `height` map of nothing.
    pub fn centered_in(&self, width: u32, height: u32) -> Self {
        let (w, h) = (self.width(), self.height());
        let (ox, oy) = (width.saturating_sub(w) / 2, height.saturating_sub(h) / 2);
        let mut rows = vec![" ".repeat(width as usize); height as usize];
        for (y, row) in self.rows.iter().enumerate() {
            let y = y as u32 + oy;
            if y >= height {
                break;
            }
            let mut chars: Vec<char> = rows[y as usize].chars().collect();
            for (x, c) in row.chars().enumerate() {
                let x = x as u32 + ox;
                if x < width {
                    chars[x as usize] = c;
                }
            }
            rows[y as usize] = chars.into_iter().collect();
        }
        Self { rows }
    }

    /// `other` drawn over this map (its non-transparent pixels win), at `(x, y)`.
    pub fn overlay(&self, other: &PixelMap, x: u32, y: u32) -> Self {
        let mut rows: Vec<Vec<char>> = self.rows.iter().map(|r| r.chars().collect()).collect();
        for (dy, row) in other.rows.iter().enumerate() {
            for (dx, c) in row.chars().enumerate() {
                if Pixel::of(c).is_some()
                    && let Some(cell) = rows
                        .get_mut(y as usize + dy)
                        .and_then(|r| r.get_mut(x as usize + dx))
                {
                    *cell = c;
                }
            }
        }
        Self {
            rows: rows.into_iter().map(|r| r.into_iter().collect()).collect(),
        }
    }
}

// ------------------------------------------------------------------ glyphs

/// The glyph a `glyph` token names, or `None` for `none` and anything unknown.
pub fn glyph(name: &str) -> Option<PixelMap> {
    let rows: &[&str] = match name {
        // The classic check mark, 7 x 7.
        "tick" => &[
            "......X", ".....XX", "X...XXX", "XX.XXX.", "XXXXX..", ".XXX...", "..X....",
        ],
        "cross" => &[
            "X.....X", ".X...X.", "..X.X..", "...X...", "..X.X..", ".X...X.", "X.....X",
        ],
        // A solid square, for a check box that fills instead of ticking.
        "square" => &["XXXXX", "XXXXX", "XXXXX", "XXXXX", "XXXXX"],
        // The radio dot, 4 x 4 with its corners cut.
        "dot" => &[".XX.", "XXXX", "XXXX", ".XX."],
        _ => return None,
    };
    Some(PixelMap::new(rows.iter().copied()))
}

/// The arrow of a scrollbar button pointing `dir` (`up`, `down`, `left`, `right`): 7 x 4 (or 4 x 7).
pub fn arrow(dir: &str) -> Option<PixelMap> {
    let rows: &[&str] = match dir {
        "up" => &["...X...", "..XXX..", ".XXXXX.", "XXXXXXX"],
        "down" => &["XXXXXXX", ".XXXXX.", "..XXX..", "...X..."],
        "left" => &["...X", "..XX", ".XXX", "XXXX", ".XXX", "..XX", "...X"],
        "right" => &["X...", "XX..", "XXX.", "XXXX", "XXX.", "XX..", "X..."],
        _ => return None,
    };
    Some(PixelMap::new(rows.iter().copied()))
}

/// The glyph of a window button in the classic set (`minimize`, `maximize`, `restore`, `close`).
pub fn window_glyph(name: &str) -> Option<PixelMap> {
    let rows: &[&str] = match name {
        // A bar at the bottom of the cell.
        "minimize" => &["......", "......", "......", "......", "......", "XXXXXX", "XXXXXX"],
        // A box with a thick top edge.
        "maximize" => &[
            "XXXXXXXXX",
            "XXXXXXXXX",
            "X.......X",
            "X.......X",
            "X.......X",
            "X.......X",
            "X.......X",
            "XXXXXXXXX",
        ],
        // Two overlapping boxes.
        "restore" => &[
            "..XXXXXX",
            "..XXXXXX",
            "..X....X",
            "XXXXXX.X",
            "XXXXXX.X",
            "X....XXX",
            "X....X..",
            "X....X..",
            "XXXXXX..",
        ],
        "close" => &[
            "XX....XX",
            ".XX..XX.",
            "..XXXX..",
            "...XX...",
            "..XXXX..",
            ".XX..XX.",
            "XX....XX",
        ],
        _ => return None,
    };
    Some(PixelMap::new(rows.iter().copied()))
}

// ------------------------------------------------------------------ boxes

/// A square check box of `size` px: a sunken frame of two rings (outer top-left `d`, outer
/// bottom-right `L`; inner top-left `D`, inner bottom-right `l`) around a fill. `size` is at
/// least 4.
pub fn check_box(size: u32) -> PixelMap {
    let size = size.max(4);
    let n = size as usize;
    let mut rows = Vec::with_capacity(n);
    for y in 0..n {
        let mut row = String::with_capacity(n);
        for x in 0..n {
            let (xr, yb) = (n - 1 - x, n - 1 - y);
            let c = if y == 0 || x == 0 {
                if x == n - 1 || y == n - 1 { 'L' } else { 'd' }
            } else if yb == 0 || xr == 0 {
                'L'
            } else if y == 1 || x == 1 {
                if x == n - 2 || y == n - 2 { 'l' } else { 'D' }
            } else if yb == 1 || xr == 1 {
                'l'
            } else {
                'F'
            };
            row.push(c);
        }
        rows.push(row);
    }
    PixelMap::new(rows)
}

/// A round radio box of `size` px (at least 8): the check box's two rings bent into a circle, so
/// the top-left half is dark and the bottom-right half light, around a fill.
pub fn radio_box(size: u32) -> PixelMap {
    let size = size.max(8);
    let n = size as i32;
    let c = (n as f32 - 1.0) / 2.0;
    // A pixel is on ring `k` (0 outer) when its distance from the middle rounds to the radius.
    let ring = |x: i32, y: i32| -> Option<u8> {
        let (dx, dy) = (x as f32 - c, y as f32 - c);
        let d = (dx * dx + dy * dy).sqrt();
        let r = n as f32 / 2.0;
        if d <= r - 0.5 && d > r - 1.5 {
            Some(0)
        } else if d <= r - 1.5 && d > r - 2.5 {
            Some(1)
        } else if d <= r - 2.5 {
            Some(2)
        } else {
            None
        }
    };
    let mut rows = Vec::with_capacity(size as usize);
    for y in 0..n {
        let mut row = String::new();
        for x in 0..n {
            // The diagonal that splits light from dark runs from the bottom-left to the top-right.
            let dark_side = x + y <= n - 1;
            row.push(match ring(x, y) {
                Some(0) => if dark_side { 'd' } else { 'L' },
                Some(1) => if dark_side { 'D' } else { 'l' },
                Some(_) => 'F',
                None => ' ',
            });
        }
        rows.push(row);
    }
    PixelMap::new(rows)
}

// ------------------------------------------------------------------ dither

/// Whether the pixel `(x, y)` of a dithered area is the *second* color: a checkerboard of 1px
/// cells, the track of a classic scrollbar.
pub fn dither_second(x: u32, y: u32) -> bool {
    (x + y) % 2 == 1
}

/// RGBA bytes (`width * height * 4`) of a dithered area, `first` on even cells and `second` on
/// odd ones, to draw as one image.
pub fn dither_rgba(width: u32, height: u32, first: [u8; 4], second: [u8; 4]) -> Vec<u8> {
    let mut out = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            out.extend_from_slice(if dither_second(x, y) { &second } else { &first });
        }
    }
    out
}

// ------------------------------------------------------------------ dotted focus

/// The pixels of a dotted 1px rectangle of `size`: every second pixel of its outline, starting
/// with the top-left corner, as the classic focus rectangle. Returned as 1x1 rectangles relative
/// to the rectangle's top-left; empty for a size under 2px.
pub fn dotted_rect(size: Size) -> Vec<Rectangle> {
    let (w, h) = (size.width.floor() as i32, size.height.floor() as i32);
    if w < 2 || h < 2 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut put = |x: i32, y: i32| {
        if (x + y) % 2 == 0 {
            out.push(Rectangle::new(Point::new(x as f32, y as f32), Size::new(1.0, 1.0)));
        }
    };
    for x in 0..w {
        put(x, 0);
        put(x, h - 1);
    }
    for y in 1..h - 1 {
        put(0, y);
        put(w - 1, y);
    }
    out
}

// ------------------------------------------------------------------ progress blocks

/// The blocks of a segmented progress bar: how many whole blocks of `block` px with `gap` px
/// between them fill `fraction` (0 to 1) of `width` px, as `(x, width)` pairs from the left.
pub fn progress_blocks(width: f32, fraction: f32, block: f32, gap: f32) -> Vec<(f32, f32)> {
    let (block, gap) = (block.max(1.0), gap.max(0.0));
    let fill = width * fraction.clamp(0.0, 1.0);
    let step = block + gap;
    let n = ((fill + gap) / step).floor() as usize;
    (0..n).map(|i| (i as f32 * step, block)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_map_has_its_size_and_pixels() {
        let m = PixelMap::new(["X.", ".D"]);
        assert_eq!((m.width(), m.height()), (2, 2));
        assert_eq!(m.at(0, 0), Some(Pixel::Ink));
        assert_eq!(m.at(1, 0), None);
        assert_eq!(m.at(1, 1), Some(Pixel::Dark));
        assert_eq!(m.at(5, 5), None);
    }

    #[test]
    fn runs_merge_equal_neighbours_and_split_on_a_change() {
        let m = PixelMap::new(["XXD.X"]);
        let r = m.runs();
        assert_eq!(r.len(), 3);
        assert_eq!((r[0].x, r[0].width, r[0].pixel), (0, 2, Pixel::Ink));
        assert_eq!((r[1].x, r[1].width, r[1].pixel), (2, 1, Pixel::Dark));
        assert_eq!((r[2].x, r[2].width), (4, 1));
    }

    #[test]
    fn the_classic_glyphs_have_their_classic_sizes() {
        let size = |m: PixelMap| (m.width(), m.height());
        assert_eq!(size(glyph("tick").unwrap()), (7, 7));
        assert_eq!(size(glyph("dot").unwrap()), (4, 4));
        assert_eq!(size(window_glyph("maximize").unwrap()), (9, 8));
        assert_eq!(size(window_glyph("close").unwrap()), (8, 7));
        assert_eq!(size(window_glyph("restore").unwrap()), (8, 9));
        assert_eq!(size(window_glyph("minimize").unwrap()), (6, 7));
        assert_eq!(size(arrow("up").unwrap()), (7, 4));
        assert_eq!(size(arrow("left").unwrap()), (4, 7));
        assert!(glyph("none").is_none() && arrow("sideways").is_none());
    }

    #[test]
    fn the_minimize_glyph_is_a_two_pixel_bar_on_the_bottom_row() {
        let m = window_glyph("minimize").unwrap();
        assert_eq!(m.at(0, 4), None);
        assert_eq!(m.at(0, 5), Some(Pixel::Ink));
        assert_eq!(m.at(5, 6), Some(Pixel::Ink));
    }

    #[test]
    fn the_maximize_glyph_has_a_two_pixel_top_edge() {
        let m = window_glyph("maximize").unwrap();
        assert!((0..9).all(|x| m.at(x, 0) == Some(Pixel::Ink) && m.at(x, 1) == Some(Pixel::Ink)));
        assert_eq!(m.at(4, 2), None);
        assert_eq!(m.at(0, 2), Some(Pixel::Ink));
        assert!((0..9).all(|x| m.at(x, 7) == Some(Pixel::Ink)));
    }

    #[test]
    fn a_check_box_is_two_sunken_rings_around_a_fill() {
        let m = check_box(13);
        assert_eq!((m.width(), m.height()), (13, 13));
        // Outer: top-left dark inner-tone gray, bottom-right white.
        assert_eq!(m.at(0, 0), Some(Pixel::DarkInner));
        assert_eq!(m.at(6, 0), Some(Pixel::DarkInner));
        assert_eq!(m.at(0, 6), Some(Pixel::DarkInner));
        assert_eq!(m.at(12, 0), Some(Pixel::Light));
        assert_eq!(m.at(12, 12), Some(Pixel::Light));
        assert_eq!(m.at(0, 12), Some(Pixel::Light));
        assert_eq!(m.at(6, 12), Some(Pixel::Light));
        // Inner: top-left black, bottom-right light gray.
        assert_eq!(m.at(1, 1), Some(Pixel::Dark));
        assert_eq!(m.at(11, 11), Some(Pixel::LightInner));
        assert_eq!(m.at(11, 1), Some(Pixel::LightInner));
        // A 9 x 9 fill.
        assert_eq!(m.at(2, 2), Some(Pixel::Fill));
        assert_eq!(m.at(10, 10), Some(Pixel::Fill));
        let fills = (0..13).flat_map(|y| (0..13).map(move |x| (x, y))).filter(|(x, y)| m.at(*x, *y) == Some(Pixel::Fill)).count();
        assert_eq!(fills, 81);
    }

    #[test]
    fn a_radio_box_is_round_dark_top_left_and_light_bottom_right() {
        let m = radio_box(12);
        assert_eq!((m.width(), m.height()), (12, 12));
        // The corners are outside the circle.
        assert_eq!(m.at(0, 0), None);
        assert_eq!(m.at(11, 11), None);
        // The top edge is dark, the bottom edge light, the left dark, the right light.
        assert_eq!(m.at(5, 0), Some(Pixel::DarkInner));
        assert_eq!(m.at(5, 11), Some(Pixel::Light));
        assert_eq!(m.at(0, 5), Some(Pixel::DarkInner));
        assert_eq!(m.at(11, 5), Some(Pixel::Light));
        assert_eq!(m.at(5, 1), Some(Pixel::Dark));
        assert_eq!(m.at(5, 10), Some(Pixel::LightInner));
        assert_eq!(m.at(5, 5), Some(Pixel::Fill));
        // Symmetric under a half turn (swapping the tones).
        let swap = |p: Pixel| match p {
            Pixel::DarkInner => Pixel::Light,
            Pixel::Light => Pixel::DarkInner,
            Pixel::Dark => Pixel::LightInner,
            Pixel::LightInner => Pixel::Dark,
            p => p,
        };
        for y in 0..12 {
            for x in 0..12 {
                assert_eq!(m.at(x, y).map(swap).is_some(), m.at(11 - x, 11 - y).is_some());
            }
        }
    }

    #[test]
    fn centering_and_overlay_place_a_glyph_in_a_box() {
        let boxed = check_box(13);
        let tick = glyph("tick").unwrap();
        let both = boxed.overlay(&tick, 3, 3);
        assert_eq!(both.at(2, 2), Some(Pixel::Fill));
        assert_eq!(both.at(3 + 6, 3), Some(Pixel::Ink));
        let c = tick.centered_in(13, 13);
        assert_eq!((c.width(), c.height()), (13, 13));
        assert_eq!(c.at(3, 3), None.or(c.at(3, 3)));
        assert_eq!(c.at(3 + 6, 3), Some(Pixel::Ink));
    }

    #[test]
    fn the_dither_alternates_in_both_directions() {
        assert!(!dither_second(0, 0) && dither_second(1, 0) && dither_second(0, 1) && !dither_second(1, 1));
        let px = dither_rgba(2, 2, [1, 1, 1, 255], [2, 2, 2, 255]);
        assert_eq!(&px[0..4], &[1, 1, 1, 255]);
        assert_eq!(&px[4..8], &[2, 2, 2, 255]);
        assert_eq!(&px[8..12], &[2, 2, 2, 255]);
        assert_eq!(px.len(), 16);
    }

    #[test]
    fn a_dotted_rect_has_every_second_outline_pixel() {
        let d = dotted_rect(Size::new(6.0, 4.0));
        // The outline of 6 x 4 has 16 pixels; half are drawn.
        assert_eq!(d.len(), 8);
        assert!(d.iter().any(|r| r.x == 0.0 && r.y == 0.0));
        assert!(d.iter().all(|r| (r.x + r.y) % 2.0 == 0.0));
        assert!(dotted_rect(Size::new(1.0, 5.0)).is_empty());
    }

    #[test]
    fn progress_blocks_fill_in_whole_steps() {
        // 100 px, half full, 8 px blocks 2 px apart: 50 px holds 5 blocks (5*8 + 4*2 = 48).
        let b = progress_blocks(100.0, 0.5, 8.0, 2.0);
        assert_eq!(b.len(), 5);
        assert_eq!(b[0], (0.0, 8.0));
        assert_eq!(b[4], (40.0, 8.0));
        assert!(progress_blocks(100.0, 0.0, 8.0, 2.0).is_empty());
        assert_eq!(progress_blocks(100.0, 1.0, 8.0, 2.0).len(), 10);
        assert_eq!(progress_blocks(100.0, 2.0, 8.0, 2.0).len(), 10);
    }
}
#[cfg(test)]
mod tmp_print {
    #[test]
    fn print_radio() {
        let m = super::radio_box(12);
        for y in 0..12 { let r: String = (0..12).map(|x| match m.at(x,y) { None=>'.', Some(super::Pixel::DarkInner)=>'d', Some(super::Pixel::Dark)=>'D', Some(super::Pixel::Light)=>'L', Some(super::Pixel::LightInner)=>'l', Some(super::Pixel::Fill)=>'F', _=>'?' }).collect(); eprintln!("R {r}"); }
    }
}
