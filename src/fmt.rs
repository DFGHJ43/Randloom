//! Number formatting for results and CSV export.
//!
//! Integer-valued distributions keep C's `printf("%g")` output (see [`g`]),
//! while the continuous distributions use a fixed six-decimal display. Very
//! small values switch to scientific notation so they never collapse to
//! `0.000000`.

pub const DEFAULT_PRECISION: usize = 6;

/// Decimal places used by the continuous-result display.
pub const DECIMAL_PLACES: usize = 6;

/// Magnitudes below this threshold use scientific notation, because six
/// decimal places would render them as `0.000000`.
pub const SCIENTIFIC_THRESHOLD: f64 = 1e-6;

/// Number style used for one result value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NumberStyle {
    /// C `%g` output, used by the integer-valued distributions.
    #[default]
    Compact,
    /// Six decimal places, scientific notation below [`SCIENTIFIC_THRESHOLD`].
    Decimal6,
}

/// Format one result value with the distribution's number style.
pub fn number(value: f64, style: NumberStyle) -> String {
    match style {
        NumberStyle::Compact => g(value),
        NumberStyle::Decimal6 => decimal6(value),
    }
}

/// Format a value with exactly six decimal places, or with scientific
/// notation when `|value| < 1e-6` (e.g. `1.234568e-07`).
pub fn decimal6(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-inf".to_string()
        } else {
            "inf".to_string()
        };
    }
    if value != 0.0 && value.abs() < SCIENTIFIC_THRESHOLD {
        return scientific(value, DECIMAL_PLACES);
    }
    format!("{:.*}", DECIMAL_PLACES, value)
}

/// Scientific notation with `decimals` mantissa digits and a signed,
/// at-least-two-digit exponent (`1.234568e-07`, `1.000000e+300`).
fn scientific(value: f64, decimals: usize) -> String {
    let formatted = format!("{:.*e}", decimals, value);
    let (mantissa, exponent) = formatted
        .split_once('e')
        .expect("Rust's LowerExp formatting always contains an exponent");
    let exponent: i32 = exponent
        .parse()
        .expect("Rust's LowerExp formatting always uses a decimal exponent");
    format!(
        "{}e{}{:02}",
        mantissa,
        if exponent < 0 { '-' } else { '+' },
        exponent.unsigned_abs()
    )
}

/// Format a value like C's `printf("%g", value)`.
pub fn g(value: f64) -> String {
    g_precision(value, DEFAULT_PRECISION)
}

/// Format a value like C's `printf("%.*g", precision, value)`.
pub fn g_precision(value: f64, precision: usize) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-inf".to_string()
        } else {
            "inf".to_string()
        };
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0".to_string()
        } else {
            "0".to_string()
        };
    }

    let negative = value.is_sign_negative();
    let abs = value.abs();
    let precision = precision.max(1);

    // Let Rust round to the requested number of significant digits once, then
    // use the resulting exponent for the fixed/scientific decision. This also
    // handles carries such as 9.999999 -> 1.00000e1.
    let scientific = format!("{:.*e}", precision - 1, abs);
    let (mantissa, exponent) = scientific
        .split_once('e')
        .expect("Rust's LowerExp formatting always contains an exponent");
    let exponent: i32 = exponent
        .parse()
        .expect("Rust's LowerExp formatting always uses a decimal exponent");

    let body = if exponent < -4 || exponent >= precision as i32 {
        format!(
            "{}e{}{:02}",
            trim_zeros(mantissa),
            if exponent < 0 { '-' } else { '+' },
            exponent.unsigned_abs()
        )
    } else {
        let decimals = (precision as i32 - (exponent + 1)).max(0) as usize;
        trim_zeros(&format!("{:.*}", decimals, abs)).to_string()
    };

    if negative { format!("-{body}") } else { body }
}

/// Remove trailing fractional zeros (and a bare decimal point) from a
/// already-formatted number.
fn trim_zeros(text: &str) -> &str {
    if text.contains('.') {
        let trimmed = text.trim_end_matches('0');
        if let Some(without_dot) = trimmed.strip_suffix('.') {
            without_dot
        } else {
            trimmed
        }
    } else {
        text
    }
}
