//! Fixed-size cell grid used by the headless renderer.
//!
//! Every glyph used by the UI occupies exactly one terminal cell, so the
//! frame can be modelled as a flat `char + style` grid. The terminal layer
//! converts a frame to ANSI commands; this module never touches a terminal.

/// Minimal character attributes used by the TUI (bold status bar, reverse
/// video for the focused field).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub bold: bool,
    pub reverse: bool,
}

impl Style {
    pub const PLAIN: Self = Self {
        bold: false,
        reverse: false,
    };

    pub const BOLD: Self = Self {
        bold: true,
        reverse: false,
    };

    pub const REVERSE: Self = Self {
        bold: false,
        reverse: true,
    };
}

/// One character cell.
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            style: Style::default(),
        }
    }
}

/// A fixed-size grid of single-width character cells.
pub struct Frame {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl Frame {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); width as usize * height as usize],
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&Cell> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        self.cells
            .get(y as usize * self.width as usize + x as usize)
    }

    pub fn set(&mut self, x: i32, y: i32, ch: char) {
        self.set_styled(x, y, ch, Style::default());
    }

    pub fn set_styled(&mut self, x: i32, y: i32, ch: char, style: Style) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let idx = y as usize * self.width as usize + x as usize;
        self.cells[idx] = Cell { ch, style };
    }

    fn set_checked(&mut self, x: i32, y: i32, ch: char, style: Style) {
        self.set_styled(x, y, ch, style);
    }

    pub fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, ch: char, style: Style) {
        if w <= 0 || h <= 0 {
            return;
        }
        for row in y..y.saturating_add(h) {
            for col in x..x.saturating_add(w) {
                self.set_checked(col, row, ch, style);
            }
        }
    }

    /// Write `text` without padding, clipped to `max_width` cells and the
    /// frame bounds.
    pub fn put_str(&mut self, x: i32, y: i32, text: &str, max_width: i32) {
        self.put_str_styled(x, y, text, max_width, Style::PLAIN);
    }

    pub fn put_str_styled(&mut self, x: i32, y: i32, text: &str, max_width: i32, style: Style) {
        if max_width <= 0 {
            return;
        }
        for (i, ch) in text.chars().take(max_width as usize).enumerate() {
            self.set_checked(x + i as i32, y, ch, style);
        }
    }

    /// Write `text` truncated/padded to exactly `width` cells.
    pub fn put_padded(&mut self, x: i32, y: i32, width: i32, text: &str, style: Style) {
        if width <= 0 {
            return;
        }
        let mut col = x;
        let end = x.saturating_add(width);
        for ch in text.chars().take(width as usize) {
            if col >= end {
                return;
            }
            self.set_checked(col, y, ch, style);
            col += 1;
        }
        while col < end {
            self.set_checked(col, y, ' ', style);
            col += 1;
        }
    }

    /// Plain-text view used by headless snapshot tests. Trailing spaces are
    /// trimmed so snapshots stay readable and line-ending safe.
    pub fn to_string_trimmed(&self) -> String {
        let mut out = String::new();
        for y in 0..self.height {
            if y > 0 {
                out.push('\n');
            }
            let mut row = String::with_capacity(self.width as usize);
            for x in 0..self.width {
                let ch = self.get(x as i32, y as i32).map_or(' ', |cell| cell.ch);
                row.push(ch);
            }
            out.push_str(row.trim_end());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_padded_truncates_and_pads() {
        let mut frame = Frame::new(6, 1);
        frame.put_padded(1, 0, 3, "abcdef", Style::REVERSE);
        assert_eq!(frame.get(0, 0).unwrap().ch, ' ');
        assert_eq!(frame.get(1, 0).unwrap().ch, 'a');
        assert_eq!(frame.get(2, 0).unwrap().ch, 'b');
        assert_eq!(frame.get(3, 0).unwrap().ch, 'c');
        assert_eq!(frame.get(4, 0).unwrap().style, Style::PLAIN);

        frame.put_padded(1, 0, 3, "x", Style::BOLD);
        assert_eq!(frame.get(1, 0).unwrap().ch, 'x');
        assert!(frame.get(1, 0).unwrap().style.bold);
        assert_eq!(frame.get(2, 0).unwrap().ch, ' ');
        assert_eq!(frame.get(3, 0).unwrap().ch, ' ');
    }

    #[test]
    fn put_padded_is_checked_different() {
        let mut frame = Frame::new(2, 1);
        frame.put_padded(1, 0, 10, "xy", Style::default());
        assert_eq!(frame.get(1, 0).unwrap().ch, 'x');
    }

    #[test]
    fn out_of_bounds_writes_are_ignored() {
        let mut frame = Frame::new(2, 2);
        frame.set(-1, 0, 'x');
        frame.set(0, -1, 'x');
        frame.set(2, 0, 'x');
        frame.set(0, 2, 'x');
        assert!(frame.get(-1, 0).is_none());
        assert_eq!(frame.get(0, 0).unwrap().ch, ' ');
    }
}
