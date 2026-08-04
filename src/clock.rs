//! Date stamping: UTC civil date, with the `MESHWORK_TODAY` override so
//! golden-file tests (import, add) stay byte-stable across days.

use std::time::{SystemTime, UNIX_EPOCH};

/// Today as `YYYY-MM-DD` (UTC), or the `MESHWORK_TODAY` override verbatim.
#[must_use]
pub fn today() -> String {
    if let Ok(fixed) = std::env::var("MESHWORK_TODAY") {
        let fixed = fixed.trim();
        if !fixed.is_empty() {
            return fixed.to_string();
        }
    }
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    civil_from_days(i64::try_from(secs / 86_400).unwrap_or(0))
}

/// Days-since-epoch → civil date (Howard Hinnant's algorithm).
fn civil_from_days(z: i64) -> String {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}
