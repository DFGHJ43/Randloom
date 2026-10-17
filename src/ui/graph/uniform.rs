//! Uniform distribution graph: a flat density bar.
//!
//! The integer mode keeps the C panel unchanged. The continuous mode draws the
//! same flat rectangle, but its labels come from the real-valued bounds and the
//! density legend is `1 / (max - min)`.

use crate::app::AppState;
use crate::config::UniformMode;
use crate::fmt;
use crate::ui::frame::{Frame, Style};
use crate::ui::glyphs::GlyphSet;
use crate::ui::graph::axes;
use crate::ui::graph::canvas::Canvas;
use crate::ui::layout::Layout;

pub fn draw(frame: &mut Frame, state: &AppState, layout: &Layout, glyphs: &GlyphSet) {
    match state.params.uniform_mode {
        UniformMode::Integer => draw_integer(frame, state, layout, glyphs),
        UniformMode::Continuous => draw_continuous(frame, state, layout, glyphs),
    }
}

fn draw_integer(frame: &mut Frame, state: &AppState, layout: &Layout, glyphs: &GlyphSet) {
    let gx = layout.gpx;
    let gy = layout.gy;
    let gw = layout.gpw;
    let gh = layout.gh;
    let (min, max) = state.params.uniform_i32_bounds();

    if min >= max {
        return;
    }

    axes::draw(frame, gx, gy, gh, gw, glyphs);

    let title = format!("Uniform PDF  [{}, {}]", min, max);
    frame.put_padded(gx, gy - 1, layout.mw - 2, &title, Style::PLAIN);

    draw_flat_bar(frame, layout, glyphs);

    let baseline = gy + gh + 1;
    frame.put_str(gx, baseline, &min.to_string(), gw);
    let max_text = max.to_string();
    let max_len = max_text.len() as i32;
    frame.put_str(gx + gw - max_len, baseline, &max_text, max_len);

    let pdf_val = 1.0 / (max as f64 - min as f64);
    let legend = format!("{:.2}", pdf_val);
    frame.put_str(layout.mx, gy, &legend, layout.mw);
}

fn draw_continuous(frame: &mut Frame, state: &AppState, layout: &Layout, glyphs: &GlyphSet) {
    let gx = layout.gpx;
    let gy = layout.gy;
    let gw = layout.gpw;
    let gh = layout.gh;
    let (min, max) = state.params.uniform_real_bounds();

    if !(min.is_finite() && max.is_finite()) || max <= min {
        return;
    }

    axes::draw(frame, gx, gy, gh, gw, glyphs);

    let title = format!("Uniform PDF  [{}, {}]", fmt::g(min), fmt::g(max));
    frame.put_padded(gx, gy - 1, layout.mw - 2, &title, Style::PLAIN);

    draw_flat_bar(frame, layout, glyphs);

    let baseline = gy + gh + 1;
    let min_text = fmt::g(min);
    frame.put_str(gx, baseline, &min_text, gw);
    let max_text = fmt::g(max);
    let max_len = max_text.len() as i32;
    frame.put_str(gx + gw - max_len, baseline, &max_text, max_len);

    let pdf_val = 1.0 / (max - min);
    frame.put_str(layout.mx, gy, &fmt::g(pdf_val), layout.mw);
}

/// Fill the whole plot with the constant uniform density.
fn draw_flat_bar(frame: &mut Frame, layout: &Layout, glyphs: &GlyphSet) {
    if let Some(mut cv) = Canvas::new(layout.gpw, layout.gh - 1) {
        cv.rect(0, 0, cv.cols(), cv.rows() * 2);
        cv.render(frame, layout.gy + 1, layout.gpx, glyphs);
    }
}
