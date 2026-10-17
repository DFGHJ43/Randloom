//! Encode a [`Frame`] into a single ANSI byte buffer.
//!
//! The frame is written row by row with explicit cursor moves. Style changes
//! emit a reset followed by the needed attributes; all cells are single-width
//! characters, so no width bookkeeping is required.

use std::io;

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Attribute, Print, SetAttribute};

use crate::ui::{Frame, Style};

pub fn encode(frame: &Frame, out: &mut Vec<u8>) -> io::Result<()> {
    out.clear();

    for y in 0..frame.height() {
        queue!(out, MoveTo(0, y))?;
        queue!(out, SetAttribute(Attribute::Reset))?;
        let mut style = Style::default();

        for x in 0..frame.width() {
            let cell = match frame.get(x as i32, y as i32) {
                Some(cell) => cell,
                None => continue,
            };

            if cell.style != style {
                queue!(out, SetAttribute(Attribute::Reset))?;
                if cell.style.bold {
                    queue!(out, SetAttribute(Attribute::Bold))?;
                }
                if cell.style.reverse {
                    queue!(out, SetAttribute(Attribute::Reverse))?;
                }
                style = cell.style;
            }

            queue!(out, Print(cell.ch))?;
        }
    }

    queue!(out, SetAttribute(Attribute::Reset))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::Style;

    #[test]
    fn encodes_plain_and_reverse_cells() {
        let mut frame = Frame::new(2, 1);
        frame.set(0, 0, 'A');
        frame.set_styled(1, 0, 'B', Style::REVERSE);

        let mut out = Vec::new();
        encode(&frame, &mut out).expect("encode frame");
        let text = String::from_utf8(out).expect("ANSI output is UTF-8");

        assert!(text.contains('A'));
        assert!(text.contains('B'));
        assert!(text.contains("\u{1b}[7m"), "reverse video SGR missing");
        assert!(text.contains("\u{1b}[0m"), "style reset missing");
    }
}
