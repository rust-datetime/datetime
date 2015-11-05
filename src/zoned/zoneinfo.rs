use local::LocalDateTime;
use zoned::{TimeZone, LocalTimes};

#[derive(PartialEq, Debug)]
pub struct Zone<'a> {

    /// This zone's name in the zoneinfo database, such as "America/New_York".
    pub name: &'a str,

    pub transitions: ZoneSet<'a>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ZoneSet<'a> {
    pub first: ZoneDetails<'a>,
    pub rest:  &'a [ (i64, ZoneDetails<'a>) ],
}

#[derive(PartialEq, Debug, Clone)]
pub struct ZoneDetails<'a> {
    pub offset:  i64,
    pub name:    &'a str,
}

impl<'a> ZoneSet<'a> {
    pub fn find(&self, time: i64) -> &ZoneDetails<'a> {
        match self.rest.iter().rev().find(|t| t.0 < time) {
            None     => &self.first,
            Some(zd) => &zd.1,
        }
    }
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
        self.transitions.find(unix_timestamp).offset
    }

    fn name(&self, datetime: LocalDateTime) -> &str {
        let unix_timestamp = datetime.to_instant().seconds();
        self.transitions.find(unix_timestamp).name
    }

    fn is_fixed(&self) -> bool {
        self.transitions.rest.is_empty()
    }

    fn from_local(&self, local: LocalDateTime) -> LocalTimes {
        unimplemented!()
    }
}