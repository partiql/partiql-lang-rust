//! PartiQL's datetime values, as integers: a `DATE` is days since 1970-01-01, a time of day
//! microseconds since midnight, and a timestamp microseconds since 1970-01-01T00:00Z, so that
//! comparing two is comparing integers, and a column of them is an array of integers (Arrow's
//! `Date32`, `Time64(us)` and `Timestamp(us)`, with the offsets, which Arrow keeps per column,
//! an array of their own). Calendar fields are computed from them when read. Each is `Copy` and
//! at most 16 bytes. Dates and timestamps are bounded to what microseconds in an `i64` hold
//! with a day to spare (about 292,000 years either side of 1970, as in PostgreSQL, DuckDB and
//! Arrow), so that no arithmetic on them overflows.

use std::cmp::Ordering;

const MICROS_PER_SECOND: i64 = 1_000_000;
const MICROS_PER_DAY: i64 = 86_400 * MICROS_PER_SECOND;

/// The days since 1970-01-01 a date or timestamp may have, local: as many as microseconds in an
/// `i64` hold, less two, so that a timestamp's last microsecond and its UTC instant (its local
/// time less an offset under a day) fit too. Years -290,308 to 294,247 (-290,308-12-24 to 294,247-01-08). Bounding them keeps the
/// arithmetic on any value, stored ones too, from overflowing.
const MAX_DAYS: i64 = i64::MAX / MICROS_PER_DAY - 2;
const MIN_DAYS: i64 = -MAX_DAYS;

/// The most fractional digits of a second a value holds.
pub const MAX_PRECISION: u8 = 6;

/// The microseconds of a time's or timestamp's last fractional digit at `precision`: its
/// microseconds are a multiple of them, so that the value has the digits it was written with.
fn micro_unit(precision: u8) -> i64 {
    10i64.pow(u32::from(MAX_PRECISION - precision.min(MAX_PRECISION)))
}

/// `nanos` rounded half up to `precision` fractional digits of a second, and truncated to them,
/// in microseconds.
fn rounded_micros(nanos: u32, precision: u8) -> (i64, i64) {
    let unit = micro_unit(precision) * 1_000;
    let nanos = i64::from(nanos);
    (
        (nanos + unit / 2) / unit * unit / 1_000,
        nanos / unit * unit / 1_000,
    )
}

/// A UTC offset in minutes (east positive), or none: a value without a time zone.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Offset(i16);

impl Offset {
    /// No time zone.
    pub const NONE: Offset = Offset(i16::MIN);

    /// An offset of `minutes`, if it is one (less than a day either way).
    pub fn minutes(minutes: i32) -> Option<Offset> {
        match minutes.unsigned_abs() < 24 * 60 {
            true => Some(Offset(minutes as i16)),
            false => None,
        }
    }

    /// The offset in minutes, if there is a time zone.
    pub fn get(self) -> Option<i16> {
        (self != Offset::NONE).then_some(self.0)
    }

    /// The offset as one integer, for storing it: its minutes, or `i16::MIN` for none.
    pub fn to_bits(self) -> i16 {
        self.0
    }

    /// The offset `to_bits` gave, if it is one.
    pub fn from_bits(bits: i16) -> Option<Offset> {
        match bits {
            i16::MIN => Some(Offset::NONE),
            minutes => Offset::minutes(i32::from(minutes)),
        }
    }

    fn micros(self) -> i64 {
        self.get()
            .map_or(0, |m| i64::from(m) * 60 * MICROS_PER_SECOND)
    }
}

impl std::fmt::Debug for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.get() {
            Some(m) => {
                let sign = if m < 0 { '-' } else { '+' };
                write!(
                    f,
                    "{sign}{:02}:{:02}",
                    m.unsigned_abs() / 60,
                    m.unsigned_abs() % 60
                )
            }
            None => write!(f, "none"),
        }
    }
}

/// A `DATE`: days since 1970-01-01.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date(i32);

impl Date {
    /// The date of a year, month (1–12) and day (1–31), if it is one (in the years -290,308
    /// to 294,247).
    pub fn from_ymd(year: i32, month: u8, day: u8) -> Option<Date> {
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            return None;
        }
        // Any `i32` year's days fit an `i64`
        i32::try_from(days_from_civil(year, month, day))
            .ok()
            .and_then(Date::from_days)
    }

    /// The date `days` after 1970-01-01, if it is one (in the years -290,308 to 294,247).
    pub fn from_days(days: i32) -> Option<Date> {
        (MIN_DAYS..=MAX_DAYS)
            .contains(&i64::from(days))
            .then_some(Date(days))
    }

    /// Days since 1970-01-01.
    pub fn days(self) -> i32 {
        self.0
    }

    /// The year, month and day.
    pub fn ymd(self) -> (i32, u8, u8) {
        civil_from_days(i64::from(self.0))
    }
}

/// A time of day, `TIME [WITH TIME ZONE]`: microseconds since midnight (local, as written),
/// with its offset when it has a time zone. `==` is the same value as written (precision
/// included); PartiQL compares times with `compare`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Time {
    micros: i64,
    offset: Offset,
    precision: u8,
}

impl Time {
    /// The time `hour:minute:second.micros`, if it is one, with `precision` fractional digits
    /// of a second (at most 6).
    pub fn from_hms_micro(
        hour: u8,
        minute: u8,
        second: u8,
        micros: u32,
        offset: Offset,
        precision: u8,
    ) -> Option<Time> {
        if hour > 23 || minute > 59 || second > 59 || i64::from(micros) >= MICROS_PER_SECOND {
            return None;
        }
        let seconds = i64::from(hour) * 3600 + i64::from(minute) * 60 + i64::from(second);
        Time::from_parts(
            seconds * MICROS_PER_SECOND + i64::from(micros),
            offset,
            precision.min(MAX_PRECISION),
        )
    }

    /// The time of `micros` since midnight (local), if it is one, as `micros()` and the like
    /// give it back, for storing it.
    pub fn from_parts(micros: i64, offset: Offset, precision: u8) -> Option<Time> {
        let valid = (0..MICROS_PER_DAY).contains(&micros)
            && precision <= MAX_PRECISION
            && micros % micro_unit(precision) == 0;
        valid.then_some(Time {
            micros,
            offset,
            precision,
        })
    }

    /// The time `hour:minute:second` and `nanos` nanoseconds, if it is one: the nanoseconds
    /// rounded half up to `precision` fractional digits, truncated instead where the carry would pass midnight.
    pub fn from_hms_nano(
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
        offset: Offset,
        precision: u8,
    ) -> Option<Time> {
        if nanos >= 1_000_000_000 {
            return None;
        }
        Time::from_hms_micro(hour, minute, second, 0, offset, precision)?.plus_nanos(nanos)
    }

    /// The time `nanos` nanoseconds (less than a second) later, rounded half up to its
    /// precision's digits; truncated instead where the carry would pass midnight.
    fn plus_nanos(self, nanos: u32) -> Option<Time> {
        let (rounded, truncated) = rounded_micros(nanos, self.precision);
        Time::from_parts(self.micros + rounded, self.offset, self.precision)
            .or_else(|| Time::from_parts(self.micros + truncated, self.offset, self.precision))
    }

    /// Microseconds since midnight, local.
    pub fn micros(self) -> i64 {
        self.micros
    }

    pub fn offset(self) -> Offset {
        self.offset
    }

    /// Fractional digits of a second it was written with.
    pub fn precision(self) -> u8 {
        self.precision
    }

    /// The hour, minute, second and microsecond, local.
    pub fn hms_micro(self) -> (u8, u8, u8, u32) {
        hms_micro(self.micros)
    }

    /// The time of day in UTC, by which times with a time zone compare.
    fn utc_micros(self) -> i64 {
        (self.micros - self.offset.micros()).rem_euclid(MICROS_PER_DAY)
    }

    /// The order of two times of the same kind (both with a time zone, compared in UTC, or
    /// both without); `None` for one of each.
    pub fn compare(self, other: Time) -> Option<Ordering> {
        match (self.offset.get(), other.offset.get()) {
            (None, None) => Some(self.micros.cmp(&other.micros)),
            (Some(_), Some(_)) => Some(self.utc_micros().cmp(&other.utc_micros())),
            _ => None,
        }
    }

    /// What the time is, as `compare` sees it: a time without a time zone, or the UTC time
    /// of one with. Equal for equal times.
    pub(crate) fn key(self) -> (bool, i64) {
        match self.offset.get() {
            None => (false, self.micros),
            Some(_) => (true, self.utc_micros()),
        }
    }
}

/// A `TIMESTAMP [WITH TIME ZONE]`: microseconds since 1970-01-01T00:00 (in UTC for one with a
/// time zone, which keeps its offset; local for one without), ±292,000 years. `==` is the same value as written (precision
/// included); PartiQL compares timestamps with `compare`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Timestamp {
    micros: i64,
    offset: Offset,
    precision: u8,
}

impl Timestamp {
    /// The timestamp of a local date and time and its `offset`, if it is one, with `precision`
    /// fractional digits of a second (at most 6).
    pub fn from_local(date: Date, time: Time, offset: Offset, precision: u8) -> Option<Timestamp> {
        // A date's days are bounded, so this doesn't overflow
        let local = i64::from(date.days()) * MICROS_PER_DAY + time.micros;
        Timestamp::new(
            local.checked_sub(offset.micros())?,
            offset,
            precision.min(MAX_PRECISION),
        )
    }

    /// The timestamp, if its local date is in the years -9999 to 9999.
    fn new(micros: i64, offset: Offset, precision: u8) -> Option<Timestamp> {
        let local = micros.checked_add(offset.micros())?;
        let days = local.div_euclid(MICROS_PER_DAY);
        // An offset is whole minutes, so the local and UTC micros have the same digits
        let valid = (MIN_DAYS..=MAX_DAYS).contains(&days)
            && precision <= MAX_PRECISION
            && micros % micro_unit(precision) == 0;
        valid.then_some(Timestamp {
            micros,
            offset,
            precision,
        })
    }

    /// The timestamp of `micros` since 1970-01-01T00:00 (UTC with a time zone, else local), if
    /// it is one, as `micros()` and the like give it back, for storing it.
    pub fn from_parts(micros: i64, offset: Offset, precision: u8) -> Option<Timestamp> {
        Timestamp::new(micros, offset, precision)
    }

    /// The timestamp `micros` microseconds later, if it is one.
    pub(crate) fn plus_micros(self, micros: i64) -> Option<Timestamp> {
        Timestamp::new(
            self.micros.checked_add(micros)?,
            self.offset,
            self.precision,
        )
    }

    /// The timestamp of a local date, a time of day to the second and `nanos` nanoseconds
    /// (less than a second, or two for a leap second), and its `offset`, if it is one: the
    /// nanoseconds rounded half up to `precision` fractional digits, truncated instead where the
    /// carry would leave the range a timestamp may have.
    pub fn from_local_nano(
        date: Date,
        (hour, minute, second): (u8, u8, u8),
        nanos: u32,
        offset: Offset,
        precision: u8,
    ) -> Option<Timestamp> {
        if nanos >= 2_000_000_000 {
            return None;
        }
        let time = Time::from_hms_micro(hour, minute, second, 0, offset, precision)?;
        Timestamp::from_local(date, time, offset, precision)?.plus_nanos(nanos)
    }

    /// The timestamp `nanos` nanoseconds (less than a second, or two for a leap second) later,
    /// rounded half up to its precision's digits; truncated instead where the carry would leave
    /// the range a timestamp may have.
    fn plus_nanos(self, nanos: u32) -> Option<Timestamp> {
        let (rounded, truncated) = rounded_micros(nanos, self.precision);
        self.plus_micros(rounded)
            .or_else(|| self.plus_micros(truncated))
    }

    /// Microseconds since 1970-01-01T00:00: UTC with a time zone, else local.
    pub fn micros(self) -> i64 {
        self.micros
    }

    pub fn offset(self) -> Offset {
        self.offset
    }

    /// Fractional digits of a second it was written with.
    pub fn precision(self) -> u8 {
        self.precision
    }

    /// The local date and time, as written.
    pub fn local(self) -> (Date, Time) {
        // Bounded when made, so this doesn't overflow
        let local = self.micros + self.offset.micros();
        let days = local.div_euclid(MICROS_PER_DAY);
        let time = Time {
            micros: local.rem_euclid(MICROS_PER_DAY),
            offset: self.offset,
            precision: self.precision,
        };
        (Date(days as i32), time)
    }

    /// The order of two timestamps of the same kind (both with a time zone, compared as
    /// instants, or both without); `None` for one of each.
    pub fn compare(self, other: Timestamp) -> Option<Ordering> {
        match (self.offset.get().is_some(), other.offset.get().is_some()) {
            (a, b) if a == b => Some(self.micros.cmp(&other.micros)),
            _ => None,
        }
    }

    /// What the timestamp is, as `compare` sees it: equal for equal timestamps.
    pub(crate) fn key(self) -> (bool, i64) {
        (self.offset.get().is_some(), self.micros)
    }
}

fn hms_micro(micros: i64) -> (u8, u8, u8, u32) {
    let seconds = micros / MICROS_PER_SECOND;
    (
        (seconds / 3600) as u8,
        (seconds / 60 % 60) as u8,
        (seconds % 60) as u8,
        (micros % MICROS_PER_SECOND) as u32,
    )
}

fn is_leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 of a proleptic Gregorian date (Howard Hinnant's `days_from_civil`).
fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let (month, day) = (i64::from(month), i64::from(day));
    let year = i64::from(year) - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The proleptic Gregorian date of days since 1970-01-01 (`civil_from_days`).
fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * mp + 2) / 5 + 1) as u8;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year as i32, month, day)
}

// Each datetime fits in 16 bytes, so in a register without making it bigger
const _: () = assert!(std::mem::size_of::<Date>() <= 16);
const _: () = assert!(std::mem::size_of::<Time>() <= 16);
const _: () = assert!(std::mem::size_of::<Timestamp>() <= 16);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_round_trip_through_days() {
        for (y, m, d, days) in [
            (1970, 1, 1, 0),
            (2000, 3, 1, 11_017),
            (1969, 12, 31, -1),
            (1, 1, 1, -719_162),
            (9999, 12, 31, 2_932_896),
            (2024, 2, 29, 19_782),
        ] {
            let date = Date::from_ymd(y, m, d).unwrap();
            assert_eq!(date.days(), days, "{y}-{m}-{d}");
            assert_eq!(date.ymd(), (y, m, d));
        }
        // Every day of four centuries
        for days in -146_097..146_097 {
            let (y, m, d) = Date::from_days(days).unwrap().ymd();
            assert_eq!(Date::from_ymd(y, m, d).unwrap().days(), days);
        }
        assert!(Date::from_ymd(2023, 2, 29).is_none());
        assert!(Date::from_ymd(2023, 13, 1).is_none());
    }

    #[test]
    fn values_are_bounded_to_what_microseconds_hold() {
        assert_eq!(
            Date::from_days(MAX_DAYS as i32).unwrap().ymd(),
            (294_247, 1, 8)
        );
        assert_eq!(
            Date::from_days(MIN_DAYS as i32).unwrap().ymd(),
            (-290_308, 12, 24)
        );
        assert!(Date::from_ymd(294_247, 1, 9).is_none());
        assert!(Date::from_ymd(100_000, 1, 1).is_some());
        assert!(Date::from_days(i32::MAX).is_none());
        // The last day's last microsecond, in any offset, has a UTC instant
        let last = Date::from_days(MAX_DAYS as i32).unwrap();
        let first = Date::from_days(MIN_DAYS as i32).unwrap();
        for minutes in [-1439, 0, 1439] {
            let offset = Offset::minutes(minutes).unwrap();
            let ts = Timestamp::from_local_nano(last, (23, 59, 59), 999_999_000, offset, 6);
            assert!(ts.is_some(), "{minutes}");
            let ts = Timestamp::from_local_nano(first, (0, 0, 0), 0, offset, 0);
            assert_eq!(ts.unwrap().local().0, first);
        }
        // Stored values out of range, whose local time would overflow
        let hour = Offset::minutes(60).unwrap();
        assert!(Timestamp::from_parts(i64::MAX, hour, 0).is_none());
        assert!(Timestamp::from_parts(i64::MIN, Offset::minutes(-60).unwrap(), 0).is_none());
        // Rounding past the last instant truncates instead
        let ts = Timestamp::from_local_nano(last, (23, 59, 59), 999_999_600, Offset::NONE, 6);
        assert_eq!(ts.unwrap().local().1.hms_micro(), (23, 59, 59, 999_999));
        // ... as it does past midnight for a time
        let time = Time::from_hms_micro(23, 59, 59, 0, Offset::NONE, 6).unwrap();
        assert_eq!(
            time.plus_nanos(999_999_600).unwrap().hms_micro(),
            (23, 59, 59, 999_999)
        );
        assert_eq!(
            time.plus_nanos(999_999_400).unwrap().hms_micro(),
            (23, 59, 59, 999_999)
        );
    }

    #[test]
    fn offsets_print_their_sign() {
        assert_eq!(format!("{:?}", Offset::minutes(-30).unwrap()), "-00:30");
        assert_eq!(format!("{:?}", Offset::minutes(-489).unwrap()), "-08:09");
        assert_eq!(format!("{:?}", Offset::minutes(90).unwrap()), "+01:30");
        assert!(Offset::minutes(i32::MIN).is_none());
    }

    #[test]
    fn fractions_round_to_the_precision() {
        let t = Time::from_hms_nano(1, 23, 45, 678_000_000, Offset::NONE, 2).unwrap();
        assert_eq!(t.hms_micro(), (1, 23, 45, 680_000));
        // A value's microseconds have no more digits than its precision
        assert!(Time::from_hms_micro(1, 23, 45, 678_000, Offset::NONE, 2).is_none());
        assert!(Time::from_parts(1_234, Offset::NONE, 3).is_none());
        assert!(Timestamp::from_parts(1_234, Offset::NONE, 3).is_none());
        assert!(Timestamp::from_parts(1_000, Offset::NONE, 3).is_some());
        let date = Date::from_ymd(2000, 1, 1).unwrap();
        let ts = Timestamp::from_local_nano(date, (0, 0, 59), 950_000_000, Offset::NONE, 1);
        assert_eq!(ts.unwrap().local().1.hms_micro(), (0, 1, 0, 0));
    }

    #[test]
    fn fractions_are_less_than_a_second() {
        assert!(Time::from_hms_nano(0, 0, 0, u32::MAX, Offset::NONE, 6).is_none());
        assert!(Time::from_hms_nano(0, 0, 0, 1_000_000_000, Offset::NONE, 6).is_none());
        let date = Date::from_ymd(2000, 1, 1).unwrap();
        // A leap second's fraction is a second or more
        let leap = Timestamp::from_local_nano(date, (23, 59, 59), 1_500_000_000, Offset::NONE, 6);
        assert_eq!(leap.unwrap().local().1.hms_micro(), (0, 0, 0, 500_000));
        assert!(Timestamp::from_local_nano(date, (0, 0, 0), u32::MAX, Offset::NONE, 6).is_none());
    }

    #[test]
    fn timestamps_with_a_time_zone_are_instants() {
        let date = Date::from_ymd(2000, 1, 2).unwrap();
        let at = |h, offset| {
            let time = Time::from_hms_micro(h, 0, 0, 0, offset, 0).unwrap();
            Timestamp::from_local(date, time, offset, 0).unwrap()
        };
        let (utc, plus_one) = (Offset::minutes(0).unwrap(), Offset::minutes(60).unwrap());
        assert_eq!(at(3, utc).compare(at(4, plus_one)), Some(Ordering::Equal));
        assert_eq!(at(3, utc).key(), at(4, plus_one).key());
        assert_eq!(at(4, plus_one).local().1.hms_micro(), (4, 0, 0, 0));
        assert_eq!(at(3, Offset::NONE).compare(at(3, utc)), None);
        // An instant before midnight UTC on the local day
        let (day, time) = at(0, plus_one).local();
        assert_eq!((day, time.hms_micro()), (date, (0, 0, 0, 0)));
        assert_eq!(
            at(0, plus_one).micros(),
            i64::from(date.days()) * MICROS_PER_DAY - 3_600_000_000
        );
    }

    #[test]
    fn times_with_a_time_zone_compare_in_utc() {
        let t = |h, m| Time::from_hms_micro(h, 0, 0, 0, Offset::minutes(m).unwrap(), 0).unwrap();
        assert_eq!(t(1, 60).compare(t(0, 0)), Some(Ordering::Equal));
        assert_eq!(t(0, 60).compare(t(23, 0)), Some(Ordering::Equal));
        assert_eq!(t(2, 0).compare(t(1, 0)), Some(Ordering::Greater));
    }
}
