//! Graph area: clear + dispatch to the per-distribution renderer.

mod axes;
mod bernoulli;
mod canvas;
mod normal;
mod poisson;
mod uniform;

use crate::app::AppState;
use crate::config::DistType;
use crate::ui::frame::{Frame, Style};
use crate::ui::glyphs::GlyphSet;
use crate::ui::layout::Layout;

pub fn draw(frame: &mut Frame, state: &AppState, layout: &Layout, glyphs: &GlyphSet) {
    // Matches graph.c: rows gy-1 .. gy+gh+2 and columns mx .. mx+mw-1.
    frame.fill(
        layout.mx,
        layout.gy - 1,
        layout.mw,
        layout.gh + 4,
        ' ',
        Style::PLAIN,
    );

    match state.dist {
        DistType::Uniform => uniform::draw(frame, state, layout, glyphs),
        DistType::Normal => normal::draw(frame, state, layout, glyphs),
        DistType::Bernoulli => bernoulli::draw(frame, state, layout, glyphs),
        DistType::Poisson => poisson::draw(frame, state, layout, glyphs),
    }
}
