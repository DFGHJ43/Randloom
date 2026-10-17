//! Process entry point: terminal bootstrap, event loop and exit handling.

use std::io::{self, IsTerminal};
use std::process::ExitCode;
use std::time::Duration;

use crate::app::AppState;
use crate::input::{Flow, InputEvent, Key};
use crate::term::{self, Terminal};
use crate::ui::{self, GlyphSet, Layout};

/// Run the TUI and map startup/runtime errors to a process exit code.
pub fn run() -> ExitCode {
    match run_inner() {
        Ok(()) => ExitCode::SUCCESS,
        Err(RunError::Printed) => ExitCode::FAILURE,
        Err(RunError::Io(err)) => {
            eprintln!("randloom: {err}");
            ExitCode::FAILURE
        }
    }
}

enum RunError {
    /// The message was already printed to the right stream.
    Printed,
    Io(io::Error),
}

impl From<io::Error> for RunError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

fn run_inner() -> Result<(), RunError> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        eprintln!("randloom requires an interactive terminal (stdin/stdout is not a TTY).");
        return Err(RunError::Printed);
    }

    let (mut cols, mut rows) = term::size();
    if cols < ui::MIN_COLS || rows < ui::MIN_ROWS {
        print_too_small(cols, rows);
        return Err(RunError::Printed);
    }

    let mut terminal = Terminal::init()?;
    let mut state = AppState::new();
    let mut layout = Layout::compute(cols, rows);
    let glyphs = GlyphSet::new(!ascii_mode());
    let mut dirty = true;

    loop {
        if term::interrupted() {
            break;
        }

        // Poll the size as well as listening for Resize events: this keeps a
        // missed event from leaving a stale/garbled frame on screen.
        let (current_cols, current_rows) = terminal.size();
        if (current_cols, current_rows) != (cols, rows) {
            cols = current_cols;
            rows = current_rows;

            if cols < ui::MIN_COLS || rows < ui::MIN_ROWS {
                show_too_small(&mut terminal, cols, rows)?;
                if wait_for_usable_size(&mut terminal, &mut cols, &mut rows)? {
                    break;
                }
                layout = Layout::compute(cols, rows);
                terminal.clear()?;
                dirty = true;
                continue;
            }

            layout = Layout::compute(cols, rows);
            state.result_scroll = state
                .result_scroll
                .clamp(0, state.max_scroll(layout.result_max_show.max(1) as usize));
            terminal.clear()?;
            dirty = true;
        }

        if dirty {
            let frame = ui::render(&state, &layout, &glyphs);
            terminal.present(&frame)?;
            dirty = false;
        }

        match terminal.poll_event(Duration::from_millis(10))? {
            Some(InputEvent::Key(key)) => {
                dirty = true;
                let flow = state.handle_key(key, layout.result_max_show.max(1) as usize);
                if flow == Flow::Quit {
                    break;
                }
            }
            Some(InputEvent::Resize(_, _)) => {
                // The next loop iteration re-reads the size, recomputes the
                // layout and clears the screen.
                dirty = true;
            }
            Some(InputEvent::Interrupt) => break,
            None => {}
        }
    }

    Ok(())
}

fn ascii_mode() -> bool {
    std::env::var_os("RANDLOOM_ASCII").is_some()
}

fn print_too_small(cols: u16, rows: u16) {
    println!(
        "Terminal too small: {}x{}. Need at least {}x{}.",
        cols,
        rows,
        ui::MIN_COLS,
        ui::MIN_ROWS
    );
}

fn show_too_small(terminal: &mut Terminal, cols: u16, rows: u16) -> Result<(), RunError> {
    let message = format!(
        "Terminal too small: {}x{}. Need at least {}x{}.",
        cols,
        rows,
        ui::MIN_COLS,
        ui::MIN_ROWS
    );
    let frame = ui::too_small(cols.max(1), rows.max(1), &message);
    terminal.present(&frame)?;
    Ok(())
}

/// Wait while the terminal is too small. Returns `true` if the user asked to
/// quit; `false` once the terminal has grown back to a usable size.
fn wait_for_usable_size(
    terminal: &mut Terminal,
    cols: &mut u16,
    rows: &mut u16,
) -> Result<bool, RunError> {
    loop {
        if term::interrupted() {
            return Ok(true);
        }

        let (current_cols, current_rows) = terminal.size();
        if current_cols >= ui::MIN_COLS && current_rows >= ui::MIN_ROWS {
            *cols = current_cols;
            *rows = current_rows;
            return Ok(false);
        }

        if (current_cols, current_rows) != (*cols, *rows) {
            *cols = current_cols;
            *rows = current_rows;
            show_too_small(terminal, current_cols, current_rows)?;
        }

        match terminal.poll_event(Duration::from_millis(200))? {
            Some(InputEvent::Resize(new_cols, new_rows)) => {
                *cols = new_cols;
                *rows = new_rows;
                if new_cols >= ui::MIN_COLS && new_rows >= ui::MIN_ROWS {
                    return Ok(false);
                }
                show_too_small(terminal, new_cols, new_rows)?;
            }
            Some(InputEvent::Key(Key::Char('q') | Key::Char('Q') | Key::Esc | Key::CtrlC)) => {
                return Ok(true);
            }
            Some(InputEvent::Interrupt) => return Ok(true),
            _ => {}
        }
    }
}
