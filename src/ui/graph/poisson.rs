//! Poisson distribution PMF graph: a discrete bar chart.

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
    let lambda = state.params.lambda;

    if lambda <= 0.0 {
        return;
    }

    axes::draw(frame, gx, gy, gh, gw, glyphs);

    let title = format!("Poisson PMF  lambda={:.1}", lambda);
    frame.put_padded(gx, gy - 1, layout.mw - 2, &title, Style::PLAIN);

    let mut k_max = (lambda + 4.0 * lambda.sqrt()) as i32;
    if k_max < lambda as i32 + 1 {
        k_max = lambda as i32 + 1;
    }

    let mut pmf: Vec<f64> = Vec::with_capacity(64);
    pmf.push((-lambda).exp());
    let mut k = 1;
    while k <= k_max && pmf.len() < 64 {
        let prev = *pmf.last().expect("pmf has at least one value");
        pmf.push(prev * lambda / k as f64);
        k += 1;
    }

    let mut pmf_max = pmf[0];
    for &value in &pmf {
        if value > pmf_max {
            pmf_max = value;
        }
    }
    if pmf_max <= 0.0 {
        pmf_max = 1.0;
    }

    let canvas_rows = gh - 1;
    let sub_w = gw;
    let sub_h = 2 * canvas_rows;
    let count = pmf.len() as i32;

    let step = if count > sub_w {
        (count + sub_w - 1) / sub_w
    } else {
        1
    };
    let shown = (count + step - 1) / step;
    let pitch = (sub_w / shown).max(1);
    let gap = if pitch >= 2 { 1 } else { 0 };
    let bar_dw = pitch - gap;
    let mut x = (sub_w - (shown * pitch - gap)) / 2;

    if let Some(mut cv) = Canvas::new(gw, canvas_rows) {
        let mut i = 0;
        while i < count {
            let value = pmf[i as usize];
            let mut h = ((value / pmf_max) * (sub_h - 1) as f64 + 0.5) as i32;
            if h < 1 && value > 0.0 {
                h = 1;
            }
            if h > sub_h - 1 {
                h = sub_h - 1;
            }

            let mut w = bar_dw;
            if x + w > sub_w {
                w = sub_w - x;
            }
            if w > 0 {
                cv.rect(x, sub_h - h, w, h);
            }

            x += pitch;
            i += step;
        }
        cv.render(frame, gy + 1, gx, glyphs);
    }

    let baseline = gy + gh + 1;
    frame.put_str(gx, baseline, "0", gw);

    let k_text = k_max.to_string();
    let k_len = k_text.len() as i32;
    frame.put_str(gx + gw - k_len, baseline, &k_text, k_len);

    let lambda_col = ((lambda / k_max as f64) * (gw - 1) as f64) as i32;
    if (0..gw).contains(&lambda_col) {
        frame.put_str(gx + lambda_col, baseline, "L", gw - lambda_col);
    }

    let legend = format!("{:.3}", pmf_max);
    frame.put_str(layout.mx, gy, &legend, layout.mw);
}
