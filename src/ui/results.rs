//! Right panel: generated results and scroll hint.

use crate::app::AppState;
use crate::fmt;
use crate::ui::frame::{Frame, Style};
use crate::ui::layout::Layout;

pub fn draw(frame: &mut Frame, state: &AppState, layout: &Layout) {
    let x = layout.rx;
    let y = 3;
    let max_show = layout.result_max_show.max(1);

    // Clear one extra row for the "(N total, arrows)" hint, matching the C
    // panel's clearing loop.
    frame.fill(x, y, layout.rw, max_show + 3, ' ', Style::PLAIN);

    if state.results.is_empty() {
        frame.put_str(x, y + 1, "(no results)", layout.rw);
        return;
    }

    let count = state.results.len() as i32;
    let mut start = state.result_scroll;
    if start > count - max_show {
        start = count - max_show;
    }
    if start < 0 {
        start = 0;
    }

    let mut i = 0;
    while i < max_show && start + i < count {
        let index = (start + i) as usize;
        let line = format!("{:3}: {}", start + i + 1, fmt::g(state.results[index]));
        frame.put_str(x, y + i + 1, &line, layout.rw);
        i += 1;
    }

    if count > max_show {
        let hint = format!("({} total, arrows)", count);
        frame.put_padded(x, y + max_show + 2, layout.rw, &hint, Style::PLAIN);
    }
}
