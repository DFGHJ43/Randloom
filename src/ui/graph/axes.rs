//! Shared graph axes.

use crate::ui::frame::Frame;
use crate::ui::glyphs::GlyphSet;

/// Draw the y-axis in column `gx - 1` (rows `gy + 1 .. gy + gh - 1`) and the
/// x-axis on row `gy + gh` (columns `gx .. gx + gw - 1`), with a corner at
/// the origin and an arrow at the right end. Row `gy` stays free for each
/// graph's value legend.
pub fn draw(frame: &mut Frame, gx: i32, gy: i32, gh: i32, gw: i32, glyphs: &GlyphSet) {
    for row in gy + 1..gy + gh {
        frame.set(gx - 1, row, glyphs.vbar());
    }
    frame.set(gx - 1, gy + gh, glyphs.bottom_left());

    for i in 0..gw {
        frame.set(gx + i, gy + gh, glyphs.hbar());
    }
    frame.set(gx + gw, gy + gh, glyphs.arrow());
}
