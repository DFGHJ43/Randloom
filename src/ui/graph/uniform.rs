//! Uniform distribution PDF graph: a flat solid bar.

use crate::app::AppState;
use crate::ui::frame::{Frame, Style};
use crate::ui::glyphs::GlyphSet;
use crate::ui::graph::axes;
use crate::ui::graph::canvas::Canvas;
use crate::ui::layout::Layout;

pub fn draw(frame: &mut Frame, state: &AppState, layout: &Layout, glyphs: &GlyphSet) {
    let gx = layout.gpx;
    let gy = layout.gy;
    let gw = layout.gpw;
    let gh = layout.gh;
    let min = state.params.min;
    let max = state.params.max;

    if min >= max {
        return;
    }

    axes::draw(frame, gx, gy, gh, gw, glyphs);

    let title = format!("Uniform PDF  [{}, {}]", min, max);
    frame.put_padded(gx, gy - 1, layout.mw - 2, &title, Style::PLAIN);

    if let Some(mut cv) = Canvas::new(gw, gh - 1) {
        cv.rect(0, 0, cv.cols(), cv.rows() * 2);
        cv.render(frame, gy + 1, gx, glyphs);
    }

    let baseline = gy + gh + 1;
    frame.put_str(gx, baseline, &min.to_string(), gw);
    let max_text = max.to_string();
    let max_len = max_text.len() as i32;
    frame.put_str(gx + gw - max_len, baseline, &max_text, max_len);

    let pdf_val = 1.0 / (max as f64 - min as f64);
    let legend = format!("{:.2}", pdf_val);
    frame.put_str(layout.mx, gy, &legend, layout.mw);
}
