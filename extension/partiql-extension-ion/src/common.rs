#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Encoding {
    Ion,
    PartiqlEncodedAsIon,
}

pub(crate) const BAG_ANNOT: &str = "$bag";
pub(crate) const TIME_ANNOT: &str = "$time";
pub(crate) const DATE_ANNOT: &str = "$date";
pub(crate) const MISSING_ANNOT: &str = "$missing";

pub(crate) const TIME_PART_HOUR_KEY: &str = "hour";
pub(crate) const TIME_PART_MINUTE_KEY: &str = "minute";
pub(crate) const TIME_PART_SECOND_KEY: &str = "second";
pub(crate) const TIME_PART_TZ_HOUR_KEY: &str = "timezone_hour";
pub(crate) const TIME_PART_TZ_MINUTE_KEY: &str = "timezone_minute";

pub(crate) const RE_SET_TIME_PARTS: [&str; 5] = [
    "^hour$",
    "^minute$",
    "^second$",
    "^timezone_hour$",
    "^timezone_minute$",
];
pub(crate) const TIME_PARTS_HOUR: usize = 0;
pub(crate) const TIME_PARTS_MINUTE: usize = 1;
pub(crate) const TIME_PARTS_SECOND: usize = 2;
pub(crate) const TIME_PARTS_TZ_HOUR: usize = 3;
pub(crate) const TIME_PARTS_TZ_MINUTE: usize = 4;
