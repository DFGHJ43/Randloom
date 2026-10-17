//! Application state and user actions.
//!
//! This layer is terminal-free: it consumes [`Key`] events and owns the
//! parameter/results state. The run loop and terminal layer are responsible
//! for delivering events and drawing the resulting state.

use std::path::Path;

use crate::config::{COUNT_MAX, DistType, FIELD_COUNT, FieldBuf, FieldId, Params};
use crate::dist;
use crate::export;
use crate::input::{Flow, Key};
use crate::rng;

/// Maximum number of results kept in memory (same as the C build).
pub const MAX_RESULTS: usize = 1000;

/// Complete mutable state of one TUI session.
///
/// Fields are public so the headless render/snapshot tests can construct a
/// state without a terminal. This is not intended as a stable library API.
pub struct AppState {
    pub dist: DistType,
    pub params: Params,
    pub results: Vec<f64>,
    pub result_scroll: i32,
    pub focus: FieldId,
    pub fields: [FieldBuf; FIELD_COUNT],
    pub status: String,
    pub running: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let fields = crate::config::initial_fields();
        let params = Params::from_fields(&fields);
        Self {
            dist: DistType::Uniform,
            params,
            results: Vec::with_capacity(MAX_RESULTS),
            result_scroll: 0,
            focus: FieldId::Count,
            fields,
            status: String::new(),
            running: true,
        }
    }

    /// Replace one field's text (test/helper API) and reparse parameters.
    pub fn set_field_text(&mut self, id: FieldId, text: &str) {
        self.fields[id as usize] = FieldBuf::new(text);
        self.reparse();
    }

    /// Re-run parsing and validation after an edit.
    pub fn reparse(&mut self) {
        self.params = Params::from_fields(&self.fields);
    }

    /// Switch distribution, resetting focus to Count like the C UI.
    pub fn set_dist(&mut self, dist: DistType) {
        self.dist = dist;
        self.focus = FieldId::Count;
        self.status = format!("Switched to {}.", dist.display_name());
    }

    /// Generate `params.count` values with the configured/auto seed.
    pub fn generate(&mut self) {
        self.reparse();

        let n = self.params.count.clamp(1, COUNT_MAX) as usize;
        let now = rng::seed_from_clock();
        let seed = rng::effective_seed(self.params.seed, now);
        let mut generator = rng::Rng::from_seed(seed);

        self.results.clear();
        self.result_scroll = 0;
        dist::generate(self.dist, &self.params, &mut generator, &mut self.results);

        let auto = if self.params.seed == 0 { " (auto)" } else { "" };
        self.status = format!(
            "Gen {} {} numbers. Seed {}{}.",
            n,
            self.dist.name(),
            seed,
            auto
        );
    }

    /// Export the current results to a timestamped CSV file in the CWD.
    pub fn export(&mut self) {
        if self.results.is_empty() {
            self.status = "Nothing to export!".to_string();
            return;
        }

        let now = chrono::Local::now().naive_local();
        let name = export::filename(now);
        match export::write_csv(&self.results, Path::new(&name)) {
            Ok(()) => {
                self.status = format!("Exported {} to CSV.", self.results.len());
            }
            Err(_) => {
                self.status = "Export failed!".to_string();
            }
        }
    }

    /// Handle one key press. `visible_result_rows` is the number of result
    /// lines currently visible in the right panel (used to clamp scrolling).
    pub fn handle_key(&mut self, key: Key, visible_result_rows: usize) -> Flow {
        match key {
            Key::Char('q') | Key::Char('Q') | Key::Esc | Key::CtrlC => {
                self.running = false;
                Flow::Quit
            }
            Key::Char('u') | Key::Char('U') => {
                self.set_dist(DistType::Uniform);
                Flow::Continue
            }
            Key::Char('n') | Key::Char('N') => {
                self.set_dist(DistType::Normal);
                Flow::Continue
            }
            Key::Char('b') | Key::Char('B') => {
                self.set_dist(DistType::Bernoulli);
                Flow::Continue
            }
            Key::Char('p') | Key::Char('P') => {
                self.set_dist(DistType::Poisson);
                Flow::Continue
            }
            Key::Char('g') | Key::Char('G') => {
                self.generate();
                Flow::Continue
            }
            Key::Char('e') | Key::Char('E') => {
                self.export();
                Flow::Continue
            }
            Key::Tab => {
                self.advance_focus();
                Flow::Continue
            }
            Key::Up => {
                if !self.results.is_empty() && self.result_scroll > 0 {
                    self.result_scroll -= 1;
                }
                Flow::Continue
            }
            Key::Down => {
                let max = self.max_scroll(visible_result_rows);
                if !self.results.is_empty() && self.result_scroll < max {
                    self.result_scroll += 1;
                }
                Flow::Continue
            }
            Key::Left => {
                self.fields[self.focus as usize].move_left();
                Flow::Continue
            }
            Key::Right => {
                self.fields[self.focus as usize].move_right();
                Flow::Continue
            }
            Key::Backspace => {
                self.fields[self.focus as usize].backspace();
                self.reparse();
                Flow::Continue
            }
            Key::Char(c) => {
                if self.focus.accepts(c) {
                    self.fields[self.focus as usize].insert(c);
                    self.reparse();
                }
                Flow::Continue
            }
            Key::Enter => Flow::Continue,
        }
    }

    /// Maximum valid `result_scroll` for a viewport height.
    pub fn max_scroll(&self, visible_result_rows: usize) -> i32 {
        let visible = visible_result_rows.min(i32::MAX as usize) as i32;
        (self.results.len() as i32 - visible).max(0)
    }

    fn advance_focus(&mut self) {
        let mut next = self.focus;
        for _ in 0..FIELD_COUNT {
            next = FieldId::ALL[(next as usize + 1) % FIELD_COUNT];
            if next.visible(self.dist) {
                self.focus = next;
                return;
            }
        }
    }
}
