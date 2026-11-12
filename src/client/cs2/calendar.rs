//! Dates as the scripts print them, in English (`Class407.aStringArrayArray4849[0]`):
//! `Preference.method7788` ("dd-Mmm-yyyy HH:MM", GMT) and `Static.method13045` ("dd-Mmm-yyyy",
//! the machine's own time zone in the reference; UTC here, the standard library has no time
//! zones).

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// (year, month 1..=12, day) of a day count since 1970-01-01 (civil calendar, proleptic
/// Gregorian, as `java.util.GregorianCalendar` after 1582).
fn civil(days: i64) -> (i64, usize, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month as usize, day)
}

/// `method13045`: "dd-Mmm-yyyy" for a time in ms since 1970.
pub fn date(ms: i64) -> String {
    let (y, m, d) = civil(ms.div_euclid(86_400_000));
    format!("{d:02}-{}-{y}", MONTHS[m - 1])
}

/// `method7788`: "dd-Mmm-yyyy HH:MM" for a time in ms since 1970, in GMT.
pub fn date_time(ms: i64) -> String {
    let minutes = ms.div_euclid(60_000).rem_euclid(1440);
    format!("{} {:02}:{:02}", date(ms), minutes / 60, minutes % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_like_the_reference() {
        assert_eq!(date(0), "01-Jan-1970");
        // Day 11757 since 1970 (the scripts' day 12, counted from 27-Feb-2002) at 13:05.
        assert_eq!(date_time((11_757 * 1440 + 13 * 60 + 5) * 60_000), "11-Mar-2002 13:05");
        assert_eq!(date(951_782_400_000), "29-Feb-2000");
    }
}
