//! CSV export: a single value per line, no header, C `%g` formatting.
//!
//! This reproduces the C `output_file` format (including the `.csv`
//! extension, even though the payload is a single column with no commas).

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use chrono::NaiveDateTime;

/// `randloom_export_YYYYMMDD_HHMMSS.csv`, using local wall-clock fields.
pub fn filename(now: NaiveDateTime) -> String {
    format!("randloom_export_{}.csv", now.format("%Y%m%d_%H%M%S"))
}

/// Write one `%g`-formatted value per line.
pub fn write_csv(values: &[f64], path: &Path) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for value in values {
        writeln!(writer, "{}", crate::fmt::g(*value))?;
    }
    writer.flush()
}
