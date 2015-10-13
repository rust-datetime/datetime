use std::borrow::Cow;
use std::fmt;

use instant::Instant;
use local::LocalDateTime;

mod offset;
pub mod zoneinfo;


pub trait TimeZone<'a>: fmt::Debug {
    fn offset(&self, datetime: LocalDateTime) -> i64;
    fn name(&'a self, datetime: LocalDateTime) -> Cow<'a, str>;
    fn is_fixed(&self) -> bool;

    fn from_local(&self, local: LocalDateTime) -> LocalTimes;

    fn to_local(&self, _instant: Instant) -> ZonedDateTime {
        unimplemented!()
        // instant + self.offset(instant)
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum LocalTimes {
    Impossible,

    Precise(Instant),

    Ambiguous { earlier: Instant, later: Instant },
}

#[derive(Debug)]
pub struct ZonedDateTime {
    pub local: LocalDateTime,
    pub time_zone: Box<TimeZone<'static>>,
}