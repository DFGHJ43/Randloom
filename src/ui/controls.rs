//! Left panel: distribution selector, parameter fields and action buttons.

use crate::app::AppState;
use crate::config::{DistType, FieldId, UniformMode};
use crate::ui::frame::{Frame, Style};
use crate::ui::layout::Layout;

fn draw_option(frame: &mut Frame, layout: &Layout, row: i32, dist: DistType, selected: bool) {
    let mark = if selected { "[*]" } else { "[ ]" };
    let text = if layout.lw >= 17 {
        format!("{} {} ({})", mark, dist.display_name(), dist.hint())
    } else {
        format!("{} {}", mark, dist.display_name())
    };
    frame.put_padded(layout.lx + 2, row, layout.lw - 2, &text, Style::PLAIN);
}

/// Checkbox row for the uniform integer/continuous mode. It is aligned with
/// the parameter field labels (same left edge) and drawn one blank row below
/// the last field: `Continuous(C): [ ]` / `Continuous(C): [x]`.
fn draw_uniform_mode(frame: &mut Frame, layout: &Layout, row: i32, mode: UniformMode) {
    let mark = if mode.is_continuous() { "[x]" } else { "[ ]" };
    let full = format!("Continuous(C): {mark}");
    let short = format!("Continuous: {mark}");
    // Drop the `(C)` key hint on narrow panels rather than clipping the row.
    let text = if layout.lw >= full.chars().count() as i32 {
        &full
    } else {
        &short
    };
    frame.put_str(layout.lx, row, text, layout.lw);
}

pub fn draw(frame: &mut Frame, state: &AppState, layout: &Layout) {
    let x = layout.lx;
    let y = 3;

    frame.put_padded(x, y, layout.lw, "Distribution:", Style::PLAIN);

    for (offset, dist) in DistType::ALL.iter().enumerate() {
        draw_option(
            frame,
            layout,
            y + 1 + offset as i32,
            *dist,
            state.dist == *dist,
        );
    }

    let value_w = (layout.lw - 12).max(1);
    let mut ry = y + 6;
    let mut last_field_row = ry - 1;

    for id in FieldId::ALL {
        if !id.visible(state.dist) {
            frame.fill(x, ry, layout.lw, 1, ' ', Style::PLAIN);
            ry += 1;
            continue;
        }

        frame.put_str(x, ry, id.label(), 8);

        frame.set(x + 10, ry, '[');
        let value_style = if id == state.focus {
            Style::REVERSE
        } else {
            Style::PLAIN
        };
        frame.put_padded(
            x + 11,
            ry,
            value_w,
            &state.fields[id as usize].text,
            value_style,
        );
        frame.set(x + 11 + value_w, ry, ']');

        last_field_row = ry;
        ry += 1;
    }

    // The uniform mode checkbox lives below the parameter fields, separated
    // from them by one blank row, between the fields and the action buttons.
    if state.dist == DistType::Uniform {
        draw_uniform_mode(frame, layout, last_field_row + 2, state.params.uniform_mode);
    }

    // Clear any rows the field block did not cover.
    let mut clear_row = ry;
    while clear_row < y + 12 {
        frame.fill(x, clear_row, layout.lw, 1, ' ', Style::PLAIN);
        clear_row += 1;
    }

    let by = y + 14;
    frame.put_str(x, by, "[ Generate ]  G", layout.lw);
    frame.put_str(x, by + 1, "[ Export   ]  E", layout.lw);
    frame.put_str(x, by + 2, "[ Quit     ]  Q", layout.lw);

    let mut row = by + 3;
    while row < layout.rows as i32 - 1 {
        frame.fill(x, row, layout.lw, 1, ' ', Style::PLAIN);
        row += 1;
    }
}
