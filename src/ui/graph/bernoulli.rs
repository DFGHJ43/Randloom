//! Bernoulli distribution PMF graph: two bars for P(0) and P(1).

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
    let mut p = state.params.prob;
    p = p.clamp(0.0, 1.0);

    axes::draw(frame, gx, gy, gh, gw, glyphs);

    let title = format!("Bernoulli PMF  p={:.2}", p);
    frame.put_padded(gx, gy - 1, layout.mw - 2, &title, Style::PLAIN);

    let canvas_rows = gh - 1;
    let sub_h = 2 * canvas_rows;

    let bar_cw = (6 * gw / 28).max(1);
    let bar0_cx = 4 * gw / 28;
    let bar1_cx = 20 * gw / 28;

    let mut h0 = ((1.0 - p) * (sub_h - 1) as f64 + 0.5) as i32;
    let mut h1 = (p * (sub_h - 1) as f64 + 0.5) as i32;
    if h0 < 2 {
        h0 = 2;
    }
    if h1 < 2 {
        h1 = 2;
    }
    if h0 > sub_h - 1 {
        h0 = sub_h - 1;
    }
    if h1 > sub_h - 1 {
        h1 = sub_h - 1;
    }

    if let Some(mut cv) = Canvas::new(gw, canvas_rows) {
        cv.rect(bar0_cx, sub_h - h0, bar_cw, h0);
        cv.rect(bar1_cx, sub_h - h1, bar_cw, h1);
        cv.render(frame, gy + 1, gx, glyphs);
    }

    let bar0_x = gx + bar0_cx;
    let bar1_x = gx + bar1_cx;
    let baseline = gy + gh + 1;

    frame.put_str(bar0_x + (bar_cw - 1) / 2, baseline, "0", 1);
    frame.put_str(bar1_x + (bar_cw - 1) / 2, baseline, "1", 1);

    frame.put_str(layout.mx, gy, "1", 1);
    frame.put_str(layout.mx, gy + gh, "0", 1);

    let mut row0 = gy + 1 + (sub_h - h0) / 2 - 1;
    let mut row1 = gy + 1 + (sub_h - h1) / 2 - 1;
    if row0 < gy {
        row0 = gy;
    }
    if row1 < gy {
        row1 = gy;
    }

    frame.put_str(bar0_x, row0, &format!("{:.2}", 1.0 - p), layout.mw);
    frame.put_str(bar1_x, row1, &format!("{:.2}", p), layout.mw);
}
