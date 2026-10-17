//! Normal distribution PDF graph: a filled bell curve.

use std::f64::consts::PI;

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
    let mu = state.params.mean;
    let sigma = state.params.stddev;

    if sigma <= 0.0 {
        return;
    }

    axes::draw(frame, gx, gy, gh, gw, glyphs);

    let title = format!("Normal PDF  mu={:.1} sigma={:.1}", mu, sigma);
    frame.put_padded(gx, gy - 1, layout.mw - 2, &title, Style::PLAIN);

    let x_min = mu - 3.0 * sigma;
    let x_max = mu + 3.0 * sigma;
    let pdf_max = 1.0 / (sigma * (2.0 * PI).sqrt());

    if let Some(mut cv) = Canvas::new(gw, gh - 1) {
        let sub_w = cv.cols();
        let sub_h = cv.rows() * 2;

        for x in 0..sub_w {
            let t = if sub_w > 1 {
                x as f64 / (sub_w - 1) as f64
            } else {
                0.0
            };
            let xv = x_min + (x_max - x_min) * t;
            let z = (xv - mu) / sigma;
            let pdf = (-0.5 * z * z).exp() / (sigma * (2.0 * PI).sqrt());

            let mut h = ((pdf / pdf_max) * (sub_h - 1) as f64 + 0.5) as i32;
            h = h.clamp(0, sub_h - 1);

            cv.vspan(x, sub_h - 1 - h, sub_h - 1);
        }
        cv.render(frame, gy + 1, gx, glyphs);
    }

    let baseline = gy + gh + 1;
    let x_min_text = format!("{:.0}", x_min);
    frame.put_str(gx, baseline, &x_min_text, gw);

    let x_max_text = format!("{:.0}", x_max);
    let max_len = x_max_text.len() as i32;
    frame.put_str(gx + gw - max_len, baseline, &x_max_text, max_len);

    let mu_col = ((mu - x_min) / (x_max - x_min) * (gw - 1) as f64) as i32;
    if (0..gw).contains(&mu_col) {
        frame.put_str(gx + mu_col, baseline, "mu", gw - mu_col);
    }

    let legend = format!("{:.3}", pdf_max);
    frame.put_str(layout.mx, gy, &legend, layout.mw);
}
