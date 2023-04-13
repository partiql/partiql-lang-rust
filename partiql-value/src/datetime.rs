#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::num::NonZeroU8;
use time::{Duration, UtcOffset};

#[derive(Hash, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DateTime {
    Date(time::Date),
    Time(time::Time),
    TimeWithTz(time::Time, time::UtcOffset),
    Timestamp(time::PrimitiveDateTime),
    TimestampWithTz(time::OffsetDateTime),
}

impl DateTime {
    pub fn from_hms(hour: u8, minute: u8, second: u8) -> Self {
        DateTime::Time(time::Time::from_hms(hour, minute, second).expect("valid time value"))
    }

    pub fn from_hmfs(hour: u8, minute: u8, second: f64) -> Self {
        Self::from_hmfs_offset(hour, minute, second, None)
    }

    pub fn from_hmfs_tz(
        hour: u8,
        minute: u8,
        second: f64,
        tz_hours: Option<i8>,
        tz_minutes: Option<i8>,
    ) -> Self {
        let offset = match (tz_hours, tz_minutes) {
            (Some(h), Some(m)) => Some(UtcOffset::from_hms(h, m, 0).expect("valid offset")),
            (None, Some(m)) => Some(UtcOffset::from_hms(0, m, 0).expect("valid offset")),
            (Some(h), None) => Some(UtcOffset::from_hms(h, 0, 0).expect("valid offset")),
            _ => None,
        };

        Self::from_hmfs_offset(hour, minute, second, offset)
    }

    pub fn from_ymd(year: i32, month: NonZeroU8, day: u8) -> Self {
        let month: time::Month = month.get().try_into().expect("valid month");
        let date = time::Date::from_calendar_date(year, month, day).expect("valid ymd");
        DateTime::Date(date)
    }

    pub fn from_ymdhms_offset_minutes(
        year: i32,
        month: NonZeroU8,
        day: u8,
        hour: u8,
        minute: u8,
        second: f64,
        offset: Option<i32>,
    ) -> Self {
        let month: time::Month = month.get().try_into().expect("valid month");
        let date = time::Date::from_calendar_date(year, month, day).expect("valid ymd");
        let time = time_from_hmfs(hour, minute, second);
        match offset {
            None => DateTime::Timestamp(date.with_time(time)),
            Some(o) => {
                let offset = UtcOffset::from_whole_seconds(o * 60).expect("offset in range");
                let date = date.with_time(time).assume_offset(offset);
                DateTime::TimestampWithTz(date)
            }
        }
    }

    fn from_hmfs_offset(hour: u8, minute: u8, second: f64, offset: Option<UtcOffset>) -> Self {
        let time = time_from_hmfs(hour, minute, second);
        match offset {
            Some(offset) => DateTime::TimeWithTz(time, offset),
            None => DateTime::Time(time),
        }
    }
}

fn time_from_hmfs(hour: u8, minute: u8, second: f64) -> time::Time {
    let millis = (second.fract() * 1e9) as u32;
    let second = second.trunc() as u8;
    time::Time::from_hms_nano(hour, minute, second, millis).expect("valid time value")
}

impl Debug for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DateTime::Date(d) => {
                write!(f, "DATE '{d:?}'")
            }
            DateTime::Time(t) => {
                write!(f, "TIME '{t:?}'")
            }
            DateTime::TimeWithTz(t, tz) => {
                write!(f, "TIME WITH TIME ZONE '{t:?} {tz:?}'")
            }
            DateTime::Timestamp(dt) => {
                write!(f, "TIMESTAMP '{dt:?}'")
            }
            DateTime::TimestampWithTz(dt) => {
                write!(f, "TIMESTAMP WITH TIME ZONE '{dt:?}'")
            }
        }
    }
}

impl PartialOrd for DateTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DateTime {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (DateTime::Date(l), DateTime::Date(r)) => l.cmp(r),
            (DateTime::Date(_), _) => Ordering::Less,
            (_, DateTime::Date(_)) => Ordering::Greater,

            (DateTime::Time(l), DateTime::Time(r)) => l.cmp(r),
            (DateTime::Time(_), _) => Ordering::Less,
            (_, DateTime::Time(_)) => Ordering::Greater,

            (DateTime::TimeWithTz(l, lo), DateTime::TimeWithTz(r, ro)) => {
                let lod = Duration::new(lo.whole_seconds() as i64, 0);
                let rod = Duration::new(ro.whole_seconds() as i64, 0);
                let l_adjusted = *l + lod;
                let r_adjusted = *r + rod;
                l_adjusted.cmp(&r_adjusted)
            }
            (DateTime::TimeWithTz(_, _), _) => Ordering::Less,
            (_, DateTime::TimeWithTz(_, _)) => Ordering::Greater,

            (DateTime::Timestamp(l), DateTime::Timestamp(r)) => l.cmp(r),
            (DateTime::Timestamp(_), _) => Ordering::Less,
            (_, DateTime::Timestamp(_)) => Ordering::Greater,

            (DateTime::TimestampWithTz(l), DateTime::TimestampWithTz(r)) => l.cmp(r),
        }
    }
}
