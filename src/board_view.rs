//! Board drawing. All sizes derive from the canvas; nothing assumes 336x536.
//! Discs are squares-in-squares: black is solid, white is a ring (the canvas is
//! 1-bit). Many cells are drawn with batched `fill_rects` calls.

use gam::menu::{PixelColor, Point};
use gam::GlyphStyle;
use othello_core::Board;
use precursor_appkit::screen::Screen;

/// Where the board sits.
#[derive(Clone, Copy)]
pub struct Geometry {
    pub x0: isize,
    pub y0: isize,
    pub cell: isize,
    pub coords: bool,
}

impl Geometry {
    /// Largest board that fits in `width` x `avail_h` below `top`.
    pub fn fit(width: isize, top: isize, avail_h: isize, coords: bool) -> Self {
        let label = if coords { 14 } else { 0 };
        let by_w = (width - 2 * label) / 8;
        let by_h = (avail_h - label) / 8;
        let cell = by_w.min(by_h).max(12);
        let x0 = (width - 8 * cell) / 2;
        Self { x0, y0: top + label, cell, coords }
    }

    fn cell_tl(&self, row: u8, col: u8) -> Point {
        Point::new(self.x0 + col as isize * self.cell, self.y0 + row as isize * self.cell)
    }

    fn rect(&self, row: u8, col: u8, inset: isize) -> (Point, Point) {
        let tl = self.cell_tl(row, col);
        (Point::new(tl.x + inset, tl.y + inset), Point::new(tl.x + self.cell - 1 - inset, tl.y + self.cell - 1 - inset))
    }
}

pub struct View<'a> {
    pub board: &'a Board,
    /// Cursor cell (row, col).
    pub cursor: Option<(u8, u8)>,
    /// Bitboard of squares to mark as legal.
    pub legal: u64,
    pub last: Option<u8>,
}

pub fn draw(screen: &Screen, g: Geometry, v: &View) {
    let size = 8 * g.cell;
    // Grid.
    for i in 0..=8isize {
        screen.line(Point::new(g.x0, g.y0 + i * g.cell), Point::new(g.x0 + size, g.y0 + i * g.cell));
        screen.line(Point::new(g.x0 + i * g.cell, g.y0), Point::new(g.x0 + i * g.cell, g.y0 + size));
    }
    if g.coords {
        for i in 0..8u8 {
            let col = (b'A' + i) as char;
            let tl = g.cell_tl(0, i);
            screen.text_centered(
                Point::new(tl.x, g.y0 - 14),
                Point::new(tl.x + g.cell, g.y0 - 1),
                GlyphStyle::Small,
                &col.to_string(),
            );
            let tl = g.cell_tl(i, 0);
            screen.text_centered(
                Point::new(g.x0 - 14, tl.y + g.cell / 2 - 8),
                Point::new(g.x0 - 1, tl.y + g.cell / 2 + 8),
                GlyphStyle::Small,
                &(i + 1).to_string(),
            );
        }
    }
    // Cursor first: its frame fills the cell, so discs are drawn over it.
    if let Some((r, c)) = v.cursor {
        let (tl, br) = g.rect(r, c, 1);
        screen.frame(tl, br, 3);
    }
    let inset = (g.cell / 7).max(2);
    let ring = (g.cell / 7).max(2) + 2;
    let mut black = Vec::new();
    let mut white_outer = Vec::new();
    let mut white_inner = Vec::new();
    let mut dots = Vec::new();
    for pos in 0..64u8 {
        let (r, c) = (pos / 8, pos % 8);
        let bit = 1u64 << pos;
        if v.board.black & bit != 0 {
            black.push(g.rect(r, c, inset + 2));
        } else if v.board.white & bit != 0 {
            white_outer.push(g.rect(r, c, inset + 2));
            white_inner.push(g.rect(r, c, ring + 2));
        } else if v.legal & bit != 0 {
            let m = g.cell / 2;
            let tl = g.cell_tl(r, c);
            dots.push((Point::new(tl.x + m - 2, tl.y + m - 2), Point::new(tl.x + m + 1, tl.y + m + 1)));
        }
    }
    screen.fill_rects(black, PixelColor::Dark);
    screen.fill_rects(white_outer, PixelColor::Dark);
    screen.fill_rects(white_inner, PixelColor::Light);
    screen.fill_rects(dots, PixelColor::Dark);
    // Last move: a small mark in the centre of the disc, contrasting with it.
    if let Some(p) = v.last {
        if p < 64 {
            let (r, c) = (p / 8, p % 8);
            let tl = g.cell_tl(r, c);
            let m = g.cell / 2;
            let colour = if v.board.black & (1u64 << p) != 0 { PixelColor::Light } else { PixelColor::Dark };
            screen.fill_rects([(Point::new(tl.x + m - 1, tl.y + m - 1), Point::new(tl.x + m, tl.y + m))], colour);
        }
    }
}
