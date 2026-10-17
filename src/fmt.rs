//! C `printf("%g")`-compatible formatting.
//!
//! The TUI and the CSV exporter both print numbers with `%g` in the C
//! implementation. Rust's `Display` uses a shortest-round-trip
//! representation instead, so this helper reproduces the C rules:
//! six significant digits by default, fixed notation when the decimal
//! exponent is in `[-4, precision)`, and scientific notation otherwise, with
//! trailing zeros removed.

pub const DEFAULT_PRECISION: usize = 6;

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
