//! Parameter model, field buffers, parsing and field visibility.
//!
//! This module is deliberately terminal-free so it can be unit tested and
//! shared by the app, UI and distribution layers.

use std::fmt;

/// Number of editable fields. Keep in sync with [`FieldId::ALL`].
pub const FIELD_COUNT: usize = 8;

/// Maximum length of a field buffer (including the terminating NUL in the C
/// implementation; Rust strings need no terminator, so the usable length is
/// `FIELD_BUF_LEN - 1`).
pub const FIELD_BUF_LEN: usize = 32;

/// Maximum number of generated results.
pub const MAX_RESULTS: usize = 1000;

/// Upper bound for count, matching the C implementation.
pub const COUNT_MAX: i32 = MAX_RESULTS as i32;

/// Upper bound for Poisson lambda; keeps the graph's `k_max` cast safe.
pub const LAMBDA_MAX: f64 = 1e9;

/// The four supported distributions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DistType {
    #[default]
    Uniform,
    Normal,
    Bernoulli,
    Poisson,
}

impl DistType {
    pub const ALL: [DistType; 4] = [
        DistType::Uniform,
        DistType::Normal,
        DistType::Bernoulli,
        DistType::Poisson,
    ];

    /// Lower-case name used in status messages.
    pub fn name(self) -> &'static str {
        match self {
            DistType::Uniform => "uniform",
            DistType::Normal => "normal",
            DistType::Bernoulli => "bernoulli",
            DistType::Poisson => "poisson",
        }
    }

    /// Capitalised name used in the distribution selector.
    pub fn display_name(self) -> &'static str {
        match self {
            DistType::Uniform => "Uniform",
            DistType::Normal => "Normal",
            DistType::Bernoulli => "Bernoulli",
            DistType::Poisson => "Poisson",
        }
    }

    /// Key hint shown next to the distribution name.
    pub fn hint(self) -> char {
        match self {
            DistType::Uniform => 'U',
            DistType::Normal => 'N',
            DistType::Bernoulli => 'B',
            DistType::Poisson => 'P',
        }
    }
}

impl fmt::Display for DistType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

/// Uniform sampling mode: discrete integers (the C-parity default) or
/// continuous real numbers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UniformMode {
    /// Integers in the inclusive `[min, max]` range.
    #[default]
    Integer,
    /// Real numbers in the half-open `[min, max)` range.
    Continuous,
}

impl UniformMode {
    /// The other mode; used by the `C` toggle key.
    pub fn toggled(self) -> Self {
        match self {
            UniformMode::Integer => UniformMode::Continuous,
            UniformMode::Continuous => UniformMode::Integer,
        }
    }

    pub fn is_continuous(self) -> bool {
        matches!(self, UniformMode::Continuous)
    }
}

/// Identifies one editable field. The order is the on-screen order and the
/// Tab order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum FieldId {
    Count = 0,
    Seed,
    Min,
    Max,
    Mean,
    StdDev,
    Prob,
    Lambda,
}

impl FieldId {
    pub const ALL: [FieldId; FIELD_COUNT] = [
        FieldId::Count,
        FieldId::Seed,
        FieldId::Min,
        FieldId::Max,
        FieldId::Mean,
        FieldId::StdDev,
        FieldId::Prob,
        FieldId::Lambda,
    ];

    /// Eight-column label, matching the C layout.
    pub fn label(self) -> &'static str {
        match self {
            FieldId::Count => "Count:  ",
            FieldId::Seed => "Seed:   ",
            FieldId::Min => "Min:    ",
            FieldId::Max => "Max:    ",
            FieldId::Mean => "Mean:   ",
            FieldId::StdDev => "StdDev: ",
            FieldId::Prob => "Prob:   ",
            FieldId::Lambda => "Lambda: ",
        }
    }

    /// Whether this field accepts a typed character. Seed is digits-only;
    /// numeric fields also accept `-` and `.`.
    pub fn accepts(self, c: char) -> bool {
        if self == FieldId::Seed {
            c.is_ascii_digit()
        } else {
            c.is_ascii_digit() || c == '-' || c == '.'
        }
    }

    /// Count and Seed are global; the others depend on the distribution.
    pub fn visible(self, dist: DistType) -> bool {
        match self {
            FieldId::Count | FieldId::Seed => true,
            FieldId::Min | FieldId::Max => dist == DistType::Uniform,
            FieldId::Mean | FieldId::StdDev => dist == DistType::Normal,
            FieldId::Prob => dist == DistType::Bernoulli,
            FieldId::Lambda => dist == DistType::Poisson,
        }
    }
}

/// A single-line editable field.
#[derive(Clone, Debug, Default)]
pub struct FieldBuf {
    pub text: String,
    pub cursor: usize,
}

impl FieldBuf {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor = text.len();
        Self { text, cursor }
    }

    /// Insert one ASCII character at the cursor. Silently ignores non-ASCII
    /// characters and overflow beyond the C-compatible buffer length.
    pub fn insert(&mut self, c: char) {
        if !c.is_ascii() || self.text.len() >= FIELD_BUF_LEN - 1 {
            return;
        }
        self.text.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        self.text.remove(self.cursor);
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// Parsed and validated parameters.
///
/// `min`/`max` are real numbers so the continuous uniform mode can use
/// fractional bounds; the integer uniform mode truncates them toward zero
/// (see [`Params::uniform_i32_bounds`]). The uniform mode is not part of a
/// field buffer, so [`Params::from_fields`] always yields the default
/// [`UniformMode::Integer`]; the application copies the toggled mode back in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub count: i32,
    pub seed: u64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub stddev: f64,
    pub prob: f64,
    pub lambda: f64,
    pub uniform_mode: UniformMode,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            count: 1,
            seed: 0,
            min: 0.0,
            max: 100.0,
            mean: 0.0,
            stddev: 1.0,
            prob: 0.5,
            lambda: 1.0,
            uniform_mode: UniformMode::Integer,
        }
    }
}

impl Params {
    /// Parse all fields, then apply the same clamps as the C implementation.
    pub fn from_fields(fields: &[FieldBuf; FIELD_COUNT]) -> Self {
        let mut params = Self {
            count: parse_i32(&fields[FieldId::Count as usize].text),
            seed: parse_u64(&fields[FieldId::Seed as usize].text),
            min: parse_f64(&fields[FieldId::Min as usize].text),
            max: parse_f64(&fields[FieldId::Max as usize].text),
            mean: parse_f64(&fields[FieldId::Mean as usize].text),
            stddev: parse_f64(&fields[FieldId::StdDev as usize].text),
            prob: parse_f64(&fields[FieldId::Prob as usize].text),
            lambda: parse_f64(&fields[FieldId::Lambda as usize].text),
            uniform_mode: UniformMode::Integer,
        };
        params.validate();
        params
    }

    /// Apply the C clamp rules. Kept public so callers can re-validate a
    /// hand-constructed `Params` (used by tests).
    pub fn validate(&mut self) {
        self.count = self.count.clamp(1, COUNT_MAX);
        self.prob = self.prob.clamp(0.0, 1.0);
        if !self.stddev.is_finite() || self.stddev <= 0.0 {
            self.stddev = 1.0;
        }
        if !self.lambda.is_finite() || self.lambda <= 0.0 {
            self.lambda = 1.0;
        }
        if self.lambda > LAMBDA_MAX {
            self.lambda = LAMBDA_MAX;
        }
        if !self.mean.is_finite() {
            self.mean = 0.0;
        }
        if !self.min.is_finite() {
            self.min = 0.0;
        }
        if !self.max.is_finite() {
            self.max = 100.0;
        }
    }

    /// Integer endpoints for the discrete uniform mode.
    ///
    /// Fractional field values truncate toward zero, matching how C `strtol`
    /// parsed the old integer fields; values outside the `i32` range saturate.
    pub fn uniform_i32_bounds(&self) -> (i32, i32) {
        (trunc_i32(self.min), trunc_i32(self.max))
    }

    /// Sorted real endpoints for the continuous uniform mode. The sampler
    /// swaps reversed bounds, just like the integer mode.
    pub fn uniform_real_bounds(&self) -> (f64, f64) {
        if self.min > self.max {
            (self.max, self.min)
        } else {
            (self.min, self.max)
        }
    }
}

/// Truncate a real value toward zero and saturate it to the `i32` range.
fn trunc_i32(value: f64) -> i32 {
    value.trunc().clamp(i32::MIN as f64, i32::MAX as f64) as i32
}

/// Default field contents, matching the C initialization.
pub fn initial_fields() -> [FieldBuf; FIELD_COUNT] {
    [
        FieldBuf::new("1"),
        FieldBuf::new("0"),
        FieldBuf::new("0"),
        FieldBuf::new("100"),
        FieldBuf::new("0.0"),
        FieldBuf::new("1.0"),
        FieldBuf::new("0.5"),
        FieldBuf::new("1.0"),
    ]
}

/// Parse a signed integer with C `strtol`-like tolerance. Empty/invalid input
/// yields 0; overflow saturates at the `i32` bounds.
pub fn parse_i32(text: &str) -> i32 {
    let text = text.trim();
    if text.is_empty() {
        return 0;
    }
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => match text.strip_prefix('+') {
            Some(rest) => (false, rest),
            None => (false, text),
        },
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return 0;
    }
    let mut value: i128 = 0;
    let cap = i32::MAX as i128 + 1;
    for b in digits.bytes() {
        value = value.saturating_mul(10).saturating_add((b - b'0') as i128);
        if value > cap {
            value = cap;
        }
    }
    let value = if negative { -value } else { value };
    value.clamp(i32::MIN as i128, i32::MAX as i128) as i32
}

/// Parse an unsigned integer; empty/invalid input yields 0 and overflow
/// saturates at `u64::MAX` (matching `strtoull` behaviour).
pub fn parse_u64(text: &str) -> u64 {
    let text = text.trim();
    if text.is_empty() {
        return 0;
    }
    let text = text.strip_prefix('+').unwrap_or(text);
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return 0;
    }
    let mut value: u128 = 0;
    for b in text.bytes() {
        value = value.saturating_mul(10).saturating_add((b - b'0') as u128);
        if value > u64::MAX as u128 {
            value = u64::MAX as u128;
        }
    }
    value as u64
}

/// Parse a float; empty/invalid/non-finite input yields 0.0.
pub fn parse_f64(text: &str) -> f64 {
    let value = text.trim().parse::<f64>().unwrap_or(0.0);
    if value.is_finite() { value } else { 0.0 }
}
