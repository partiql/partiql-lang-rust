/// The encoding to use when decoding/encoding Ion to/from PartiQL [`partiql_value::Value`]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Encoding {
    /// 'Unlifted'/'Unlowered' Ion to/from PartiQL [`partiql_value::Value`]. [`Vpartiql_value::alue`]s that do not have a direct
    /// Ion analog will result in an error (e.g. PartiQL [`partiql_value::Value`] has a 'bag' type, but Ion does not,
    /// so attempting encode a 'bag').
    Ion,
    /// PartiQL encoded into Ion as supported by `partiql-tests` and by other implementations.
    /// (e.g., a PartiQL bag is encoded as an Ion list annotated with "$bag").
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
