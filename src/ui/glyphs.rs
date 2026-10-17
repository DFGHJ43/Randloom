//! Unicode and ASCII glyph sets.
//!
//! Every glyph is exactly one terminal cell wide. The ASCII set is the
//! fallback for terminals/fonts where the block and box-drawing characters
//! cannot be shown.

/// Unicode vs ASCII glyph set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlyphSet {
    pub unicode: bool,
}

impl GlyphSet {
    pub fn new(unicode: bool) -> Self {
        Self { unicode }
    }

    pub fn hbar(&self) -> char {
        if self.unicode { '─' } else { '-' }
    }

    pub fn vbar(&self) -> char {
        if self.unicode { '│' } else { '|' }
    }

    pub fn top_left(&self) -> char {
        if self.unicode { '┌' } else { '+' }
    }

    pub fn top_mid(&self) -> char {
        if self.unicode { '┬' } else { '+' }
    }

    pub fn top_right(&self) -> char {
        if self.unicode { '┐' } else { '+' }
    }

    pub fn bottom_left(&self) -> char {
        if self.unicode { '└' } else { '+' }
    }

    pub fn bottom_mid(&self) -> char {
        if self.unicode { '┴' } else { '+' }
    }

    pub fn bottom_right(&self) -> char {
        if self.unicode { '┘' } else { '+' }
    }

    pub fn arrow(&self) -> char {
        if self.unicode { '→' } else { '>' }
    }

    /// Glyph for one half-block canvas cell (top/bottom/both bits).
    pub fn block(&self, mask: u8) -> char {
        if !self.unicode {
            return '#';
        }
        match mask & 0b11 {
            0b11 => '█',
            0b01 => '▀',
            0b10 => '▄',
            _ => ' ',
        }
    }
}
