//! Outer frame, panel titles and status bar.

use crate::app::AppState;
use crate::ui::frame::{Frame, Style};
use crate::ui::glyphs::GlyphSet;
use crate::ui::layout::Layout;

pub fn draw_box(frame: &mut Frame, layout: &Layout, glyphs: &GlyphSet) {
    let w = layout.cols as i32;
    let h = layout.rows as i32;
    if w < 2 || h < 2 {
        return;
    }

    // Top border with junctions at the two divider columns.
    frame.set(0, 0, glyphs.top_left());
    for c in 1..=w - 2 {
        let ch = if c == layout.div1 || c == layout.div2 {
            glyphs.top_mid()
        } else {
            glyphs.hbar()
        };
        frame.set(c, 0, ch);
    }
    frame.set(w - 1, 0, glyphs.top_right());

    // Vertical dividers and outer sides.
    for r in 1..=h - 2 {
        frame.set(0, r, glyphs.vbar());
        frame.set(layout.div1, r, glyphs.vbar());
        frame.set(layout.div2, r, glyphs.vbar());
        frame.set(w - 1, r, glyphs.vbar());
    }

    // Bottom border with junctions.
    frame.set(0, h - 1, glyphs.bottom_left());
    for c in 1..=w - 2 {
        let ch = if c == layout.div1 || c == layout.div2 {
            glyphs.bottom_mid()
        } else {
            glyphs.hbar()
        };
        frame.set(c, h - 1, ch);
    }
    frame.set(w - 1, h - 1, glyphs.bottom_right());
}

pub fn draw_titles(frame: &mut Frame, layout: &Layout) {
    frame.put_str(layout.lx + 2, 0, "Randloom", layout.lw - 4);
    frame.put_str(layout.mx + 2, 2, "Distribution Graph", layout.mw - 2);
    frame.put_str(layout.rx + 2, 2, "Results", layout.rw - 2);
}

pub fn draw_status(frame: &mut Frame, state: &AppState, layout: &Layout) {
    let width = layout.cols as i32 - 2;
    let style = if state.status.is_empty() {
        Style::PLAIN
    } else {
        Style::BOLD
    };
    frame.put_padded(layout.lx, layout.status_y, width, &state.status, style);
}
