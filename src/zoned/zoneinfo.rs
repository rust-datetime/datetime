use local::LocalDateTime;
use zoned::{TimeZone, LocalTimes};

#[derive(PartialEq, Debug)]
pub struct Zone<'a> {

    /// This zone's name in the zoneinfo database, such as "America/New_York".
    pub name: &'a str,

    /// A static slice of all the timespans that pertain to this zone.
    /// These should be in order of when they end, up until the
    /// currently-applying timespan.
    pub transitions: &'a [Transition<'a>],
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Transition<'name> {
    pub occurs_at:  Option<i64>,
    pub offset:     i64,
    pub name:       &'name str,
}

/// The "type" of time that a time is.
///
/// A time may be followed with a letter, signifying what 'type'
/// of time the timestamp is:
///
/// - **w** for "wall clock" time (the default),
/// - **s** for local standard time,
/// - **u** or **g** or **z** for universal time.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TimeType {

    /// Wall-clock time.
    Wall,

    /// Standard Time.
    Standard,

    /// Universal Co-ordinated Time.
    UTC,
}

impl<'a> TimeZone for Zone<'a> {
    fn offset(&self, datetime: LocalDateTime) -> i64 {
        let unix_timestamp = datetime.to_instant().seconds();
        match self.transitions.iter().rev().find(|t| t.occurs_at.unwrap_or(0) < unix_timestamp) {
            None     => 0,
            Some(t)  => t.offset,
        }
    }

    fn name(&self, datetime: LocalDateTime) -> &str {
        let unix_timestamp = datetime.to_instant().seconds();
        match self.transitions.iter().rev().find(|t| t.occurs_at.unwrap_or(0) < unix_timestamp) {
            None     => "??",
            Some(t)  => t.name,
        }
    }

    fn is_fixed(&self) -> bool {
        unimplemented!()
    }

    fn from_local(&self, local: LocalDateTime) -> LocalTimes {
        unimplemented!()
    }
}