//! Time stamping, with the `MESHWORK_TODAY` override (returned verbatim) so
//! golden-file tests (import, add) stay byte-stable across days. Minting
//! stamps carry UTC minute resolution (mw-zp1h12d, §15.8: a minting rule,
//! never validation — the parser accepts date-only forever): union merges
//! of same-day appends keep a recoverable order, and same-day identical
//! comments stop hash-colliding in the mirror.

use std::time::{SystemTime, UNIX_EPOCH};

fn override_stamp() -> Option<String> {
    let fixed = std::env::var("MESHWORK_TODAY").ok()?;
    let fixed = fixed.trim();
    (!fixed.is_empty()).then(|| fixed.to_string())
}

fn epoch_secs() -> i64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    i64::try_from(secs).unwrap_or(0)
}

/// Today as `YYYY-MM-DD` (UTC), or the `MESHWORK_TODAY` override verbatim.
#[must_use]
pub fn today() -> String {
    match override_stamp() {
        Some(fixed) => fixed,
        None => civil_from_days(epoch_secs() / 86_400),
    }
}

/// Minting stamp: `YYYY-MM-DDTHH:MMZ` (UTC minute resolution), or the
/// `MESHWORK_TODAY` override verbatim (mw-zp1h12d).
#[must_use]
pub fn stamp() -> String {
    if let Some(fixed) = override_stamp() {
        return fixed;
    }
    let secs = epoch_secs();
    let day = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    format!(
        "{}T{:02}:{:02}Z",
        civil_from_days(day),
        rem / 3_600,
        (rem % 3_600) / 60
    )
}

/// Whole days from `from` to `to`, each an ISO date or datetime (only
/// the `YYYY-MM-DD` prefix is read); `None` when either fails to parse.
/// Negative when `from` is later — callers judging staleness treat that
/// as "not stale", never as an error (dates are as-written, MW-A6).
#[must_use]
pub fn days_between(from: &str, to: &str) -> Option<i64> {
    Some(days_from_civil(to)? - days_from_civil(from)?)
}

/// Civil date prefix → days-since-epoch (inverse of `civil_from_days`,
/// same Hinnant construction).
pub(crate) fn days_from_civil(date: &str) -> Option<i64> {
    let date = date.get(..10)?;
    let mut parts = date.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_roundtrip_and_day_diffs() {
        for z in [-1, 0, 1, 19_000, 20_688, 60_000] {
            let date = civil_from_days(z);
            assert_eq!(days_from_civil(&date), Some(z), "{date}");
        }
        assert_eq!(days_between("2026-08-01", "2026-08-20"), Some(19));
        assert_eq!(days_between("2026-08-20T09:00Z", "2026-08-20"), Some(0));
        assert_eq!(days_between("2026-09-01", "2026-08-20"), Some(-12));
        assert_eq!(days_between("imported", "2026-08-20"), None);
    }
}
