#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::num::NonZeroU8;
use time::macros::format_description;
use time::{Duration, UtcOffset};

#[derive(Hash, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DateTime {
    Date(time::Date),
    Time(time::Time, Option<u32>),
    TimeWithTz(time::Time, Option<u32>, time::UtcOffset),
    Timestamp(time::PrimitiveDateTime, Option<u32>),
    TimestampWithTz(time::OffsetDateTime, Option<u32>),
}

impl DateTime {
    pub fn from_hms(hour: u8, minute: u8, second: u8) -> Self {
        DateTime::Time(
            time::Time::from_hms(hour, minute, second).expect("valid time value"),
            None,
        )
    }

    pub fn from_hms_nano(hour: u8, minute: u8, second: u8, nanosecond: u32) -> Self {
        Self::from_hms_nano_offset(hour, minute, second, nanosecond, None)
    }

    pub fn from_hms_nano_tz(
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
        tz_hours: Option<i8>,
        tz_minutes: Option<i8>,
    ) -> Self {
        let offset = match (tz_hours, tz_minutes) {
            (Some(h), Some(m)) => Some(UtcOffset::from_hms(h, m, 0).expect("valid offset")),
            (None, Some(m)) => Some(UtcOffset::from_hms(0, m, 0).expect("valid offset")),
            (Some(h), None) => Some(UtcOffset::from_hms(h, 0, 0).expect("valid offset")),
            _ => None,
        };

        Self::from_hms_nano_offset(hour, minute, second, nanosecond, offset)
    }

    pub fn from_ymd(year: i32, month: NonZeroU8, day: u8) -> Self {
        let month: time::Month = month.get().try_into().expect("valid month");
        let date = time::Date::from_calendar_date(year, month, day).expect("valid ymd");
        DateTime::Date(date)
    }

    pub fn from_ymdhms_nano_offset_minutes(
        year: i32,
        month: NonZeroU8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
        offset: Option<i32>,
    ) -> Self {
        let month: time::Month = month.get().try_into().expect("valid month");
        let date = time::Date::from_calendar_date(year, month, day).expect("valid ymd");
        let time = time_from_hms_nano(hour, minute, second, nanosecond);
        match offset {
            None => DateTime::Timestamp(date.with_time(time), None),
            Some(o) => {
                let offset = UtcOffset::from_whole_seconds(o * 60).expect("offset in range");
                let date = date.with_time(time).assume_offset(offset);
                DateTime::TimestampWithTz(date, None)
            }
        }
    }

    fn from_hms_nano_offset(
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
        offset: Option<UtcOffset>,
    ) -> Self {
        let time = time_from_hms_nano(hour, minute, second, nanosecond);
        match offset {
            Some(offset) => DateTime::TimeWithTz(time, None, offset),
            None => DateTime::Time(time, None),
        }
    }

    pub fn from_yyyy_mm_dd(date: &str) -> Self {
        let format = format_description!("[year]-[month]-[day]");
        let date = time::Date::parse(date, &format).expect("valid date string");
        DateTime::Date(date)
    }

    pub fn from_hh_mm_ss(time: &str, precision: &Option<u32>) -> Self {
        let format = format_description!("[hour]:[minute]:[second].[subsecond]");
        let time = time::Time::parse(time, &format).expect("valid time string");
        DateTime::Time(time, *precision)
    }

    pub fn from_hh_mm_ss_time_zone(time: &str, precision: &Option<u32>) -> Self {
        let time_format = format_description!(
            "[hour]:[minute]:[second].[subsecond][offset_hour]:[offset_minute]"
        );
        let time_part = time::Time::parse(time, &time_format).expect("valid time with time zone");
        let time_format = format_description!(
            "[hour]:[minute]:[second].[subsecond][offset_hour]:[offset_minute]"
        );
        let offset_part = time::UtcOffset::parse(time, &time_format).expect("valid time zone");
        DateTime::TimeWithTz(time_part, *precision, offset_part)
    }

    pub fn from_yyyy_mm_dd_hh_mm_ss(timestamp: &str, precision: &Option<u32>) -> Self {
        let format =
            format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");
        let time =
            time::PrimitiveDateTime::parse(timestamp, &format).expect("valid timestamp string");
        DateTime::Timestamp(time, *precision)
    }

    pub fn from_yyyy_mm_dd_hh_mm_ss_time_zone(timestamp: &str, precision: &Option<u32>) -> Self {
        let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond][offset_hour]:[offset_minute]");
        let time = time::OffsetDateTime::parse(timestamp, &format)
            .expect("valid timestamp string with time zone");
        DateTime::TimestampWithTz(time, *precision)
    }
}

fn time_from_hms_nano(hour: u8, minute: u8, second: u8, nanosecond: u32) -> time::Time {
    time::Time::from_hms_nano(hour, minute, second, nanosecond).expect("valid time value")
}

impl Debug for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DateTime::Date(d) => {
                write!(f, "DATE '{d:?}'")
            }
            DateTime::Time(t, p) => match p {
                None => write!(f, "TIME '{t:?}'"),
                Some(p) => write!(f, "TIME ({p:?}) '{t:?}'"),
            },
            DateTime::TimeWithTz(t, p, tz) => match p {
                None => write!(f, "TIME WITH TIME ZONE '{t:?} {tz:?}'"),
                Some(p) => write!(f, "TIME ({p:?}) WITH TIME ZONE '{t:?} {tz:?}'"),
            },
            DateTime::Timestamp(dt, p) => match p {
                None => write!(f, "TIMESTAMP '{dt:?}'"),
                Some(p) => write!(f, "TIMESTAMP ({p:?}) '{dt:?}'"),
            },
            DateTime::TimestampWithTz(dt, p) => match p {
                None => write!(f, "TIMESTAMP WITH TIME ZONE '{dt:?}'"),
                Some(p) => write!(f, "TIMESTAMP ({p:?}) WITH TIME ZONE '{dt:?}'"),
            },
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

            (DateTime::Time(l, _lp), DateTime::Time(r, _rp)) => l.cmp(r),
            // TODO: sorting using the time precisions
            (DateTime::Time(_, _), _) => Ordering::Less,
            (_, DateTime::Time(_, _)) => Ordering::Greater,

            (DateTime::TimeWithTz(l, _lp, lo), DateTime::TimeWithTz(r, _rp, ro)) => {
                // TODO: sorting using the time precisions
                let lod = Duration::new(lo.whole_seconds() as i64, 0);
                let rod = Duration::new(ro.whole_seconds() as i64, 0);
                let l_adjusted = *l + lod;
                let r_adjusted = *r + rod;
                l_adjusted.cmp(&r_adjusted)
            }
            (DateTime::TimeWithTz(_, _, _), _) => Ordering::Less,
            (_, DateTime::TimeWithTz(_, _, _)) => Ordering::Greater,

            // TODO: sorting using the timestamp precisions
            (DateTime::Timestamp(l, _lp), DateTime::Timestamp(r, _rp)) => l.cmp(r),
            (DateTime::Timestamp(_, _), _) => Ordering::Less,
            (_, DateTime::Timestamp(_, _)) => Ordering::Greater,

            // TODO: sorting using the timestamp precisions
            (DateTime::TimestampWithTz(l, _lp), DateTime::TimestampWithTz(r, _rp)) => l.cmp(r),
        }
    }
}
