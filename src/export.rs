//! CSV export: a single value per line, no header.
//!
//! The exporter uses the same per-distribution number style as the result
//! list: compact `%g` for integer-valued results, six decimals (scientific
//! notation below `1e-6`) for continuous results. This reproduces the C
//! `output_file` shape (including the `.csv` extension, even though the
//! payload is a single column with no commas).

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use chrono::NaiveDateTime;

use crate::fmt::{self, NumberStyle};

/// `randloom_export_YYYYMMDD_HHMMSS.csv`, using local wall-clock fields.
pub fn filename(now: NaiveDateTime) -> String {
    format!("randloom_export_{}.csv", now.format("%Y%m%d_%H%M%S"))
}

/// Write one formatted value per line.
pub fn write_csv(values: &[f64], path: &Path, style: NumberStyle) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for value in values {
        writeln!(writer, "{}", fmt::number(*value, style))?;
    }
    writer.flush()
}
