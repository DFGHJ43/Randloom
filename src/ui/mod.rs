//! Headless rendering model for the Randloom TUI.
//!
//! The UI builds a [`Frame`] (a grid of single-width cells plus the two
//! attributes the C version used: bold and reverse video). The terminal
//! layer is responsible for painting that frame; this module never touches a
//! terminal or a clock, which makes it fully testable with snapshots.
//!
//! Layout and graph geometry are a direct port of `legacy/c/src/tui.c`,
//! `controls.c` and `graph/*.c`.

mod chrome;
mod controls;
mod frame;
mod glyphs;
mod graph;
mod layout;
mod results;

use crate::app::AppState;

pub use frame::{Cell, Frame, Style};
pub use glyphs::GlyphSet;
pub use layout::{Layout, MIN_COLS, MIN_ROWS};

/// Render one complete frame.
pub fn render(state: &AppState, layout: &Layout, glyphs: &GlyphSet) -> Frame {
    let mut frame = Frame::new(layout.cols, layout.rows);

    chrome::draw_box(&mut frame, layout, glyphs);
    chrome::draw_titles(&mut frame, layout);
    controls::draw(&mut frame, state, layout);
    results::draw(&mut frame, state, layout);
    graph::draw(&mut frame, state, layout, glyphs);
    chrome::draw_status(&mut frame, state, layout);

    frame
}

/// Headless convenience wrapper used by snapshot tests.
pub fn render_to_string(state: &AppState, cols: u16, rows: u16, unicode: bool) -> String {
    let layout = Layout::compute(cols, rows);
    render(state, &layout, &GlyphSet::new(unicode)).to_string_trimmed()
}

/// Frame shown while the terminal is smaller than the minimum size. The
/// run loop clears the screen first, exactly like the C overlay. The message
/// wraps at the frame width, which mirrors what a too-narrow terminal does
/// with the C version's plain `printf`.
pub fn too_small(cols: u16, rows: u16, message: &str) -> Frame {
    let mut frame = Frame::new(cols, rows);
    if cols == 0 || rows == 0 {
        return frame;
    }

    let width = cols as i32;
    let mut x = 0;
    let mut y = 0;
    for ch in message.chars() {
        if ch == '\n' {
            x = 0;
            y += 1;
            continue;
        }
        if x >= width {
            x = 0;
            y += 1;
        }
        if y >= rows as i32 {
            break;
        }
        frame.set(x, y, ch);
        x += 1;
    }
    frame
}
