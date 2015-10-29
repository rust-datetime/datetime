use std::fmt;

use duration::Duration;
use instant::Instant;
use local::LocalDateTime;

pub mod factory;
mod offset;
pub mod zoneinfo;


pub trait TimeZone: fmt::Debug {
    fn offset(&self, datetime: LocalDateTime) -> i64;
    fn name(&self, datetime: LocalDateTime) -> &str;
    fn is_fixed(&self) -> bool;

    fn from_local(&self, local: LocalDateTime) -> LocalTimes;

    fn to_zoned(&self, datetime: LocalDateTime) -> LocalDateTime {
        datetime + Duration::of(self.offset(datetime))
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum LocalTimes {
    Impossible,

    Precise(Instant),

    Ambiguous { earlier: Instant, later: Instant },
}

// #[derive(Debug)]
// pub struct ZonedDateTime {
//     pub local: LocalDateTime,
//     pub time_zone: Box<TimeZone<'static>>,
// }