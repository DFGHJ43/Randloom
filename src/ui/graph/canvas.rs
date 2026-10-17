//! Half-block canvas used by the distribution graphs.
//!
//! A character cell holds two vertically stacked half-cells:
//!
//! ```text
//! bit 0 (top)    => ▀
//! bit 1 (bottom) => ▄
//! both           => █
//! ```
//!
//! The ASCII fallback renders every touched cell as `#`, matching the C
//! implementation.

use crate::ui::frame::Frame;
use crate::ui::glyphs::GlyphSet;

pub struct Canvas {
    cols: i32,
    rows: i32,
    cells: Vec<u8>,
}

impl Canvas {
    /// Allocate a cols x rows canvas. Returns `None` for an empty size.
    pub fn new(cols: i32, rows: i32) -> Option<Self> {
        if cols <= 0 || rows <= 0 {
            return None;
        }
        Some(Self {
            cols,
            rows,
            cells: vec![0; cols as usize * rows as usize],
        })
    }

    pub fn cols(&self) -> i32 {
        self.cols
    }

    pub fn rows(&self) -> i32 {
        self.rows
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.cols || y >= self.rows * 2 {
            return None;
        }
        Some((y >> 1) as usize * self.cols as usize + x as usize)
    }

    /// Set one half-cell. Out-of-bounds coordinates are ignored.
    pub fn set(&mut self, x: i32, y: i32) {
        if let Some(idx) = self.index(x, y) {
            self.cells[idx] |= if y & 1 == 0 { 0b01 } else { 0b10 };
        }
    }

    /// Set every half-cell in column `x` between `y0` and `y1` (inclusive,
    /// either order).
    pub fn vspan(&mut self, x: i32, y0: i32, y1: i32) {
        let (mut top, bottom) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
        while top <= bottom {
            self.set(x, top);
            top += 1;
        }
    }

    /// Fill a half-cell rectangle whose top-left half-cell is `(x0, y0)`.
    pub fn rect(&mut self, x0: i32, y0: i32, w: i32, h: i32) {
        if w <= 0 || h <= 0 {
            return;
        }
        for y in y0..y0.saturating_add(h) {
            for x in x0..x0.saturating_add(w) {
                self.set(x, y);
            }
        }
    }

    /// Draw the canvas with its top-left cell at screen cell `(row, col)`.
    /// Empty cells are skipped; the caller has already cleared the area.
    pub fn render(&self, frame: &mut Frame, row: i32, col: i32, glyphs: &GlyphSet) {
        let cols = self.cols as usize;
        for r in 0..self.rows {
            let start = r as usize * cols;
            for (c, &mask) in self.cells[start..start + cols].iter().enumerate() {
                if mask != 0 {
                    frame.set(col + c as i32, row + r, glyphs.block(mask));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_canvas_is_rejected() {
        assert!(Canvas::new(0, 5).is_none());
        assert!(Canvas::new(5, 0).is_none());
        assert!(Canvas::new(-1, 5).is_none());
    }

    #[test]
    fn masks_distinguish_top_and_bottom_half_cells() {
        let mut cv = Canvas::new(2, 2).unwrap();
        cv.set(0, 0); // top of cell (0,0)
        cv.set(1, 1); // bottom of cell (1,0)
        cv.set(0, 2); // top of cell (0,1)
        cv.set(1, 3); // bottom of cell (1,1)

        let mut frame = Frame::new(2, 2);
        cv.render(&mut frame, 0, 0, &GlyphSet::new(true));
        assert_eq!(frame.get(0, 0).unwrap().ch, '▀');
        assert_eq!(frame.get(1, 0).unwrap().ch, '▄');
        assert_eq!(frame.get(0, 1).unwrap().ch, '▀');
        assert_eq!(frame.get(1, 1).unwrap().ch, '▄');
    }

    #[test]
    fn both_halves_render_full_block_and_ascii_hash() {
        let mut cv = Canvas::new(1, 1).unwrap();
        cv.set(0, 0);
        cv.set(0, 1);

        let mut unicode = Frame::new(1, 1);
        cv.render(&mut unicode, 0, 0, &GlyphSet::new(true));
        assert_eq!(unicode.get(0, 0).unwrap().ch, '█');

        let mut ascii = Frame::new(1, 1);
        cv.render(&mut ascii, 0, 0, &GlyphSet::new(false));
        assert_eq!(ascii.get(0, 0).unwrap().ch, '#');
    }

    #[test]
    fn out_of_bounds_and_clipping_are_safe() {
        let mut cv = Canvas::new(2, 1).unwrap();
        cv.set(-1, 0);
        cv.set(0, -1);
        cv.set(2, 0);
        cv.set(0, 2);
        cv.rect(-1, -1, 4, 4); // clips to the whole 2x2 half-cell canvas

        let mut frame = Frame::new(2, 1);
        cv.render(&mut frame, 0, 0, &GlyphSet::new(true));
        assert_eq!(frame.get(0, 0).unwrap().ch, '█');
        assert_eq!(frame.get(1, 0).unwrap().ch, '█');
    }

    #[test]
    fn vspan_normalises_reversed_coordinates() {
        let mut cv = Canvas::new(1, 2).unwrap();
        cv.vspan(0, 3, 1); // bottom up to half-cell 1
        let mut frame = Frame::new(1, 2);
        cv.render(&mut frame, 0, 0, &GlyphSet::new(true));
        assert_eq!(frame.get(0, 0).unwrap().ch, '▄');
        assert_eq!(frame.get(0, 1).unwrap().ch, '█');
    }
}
