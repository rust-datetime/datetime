use std::fmt;

use duration::Duration;
use instant::Instant;
use local::{LocalDateTime, DatePiece, TimePiece, Month, Weekday};

pub mod factory;
mod offset;
pub mod zoneinfo;


pub trait TimeZone<'a>: fmt::Debug {
    fn offset(&self, datetime: LocalDateTime) -> i64;
    fn name(&self, datetime: LocalDateTime) -> &str;
    fn is_fixed(&self) -> bool;

    fn convert_local(&self, local: LocalDateTime) -> LocalTimes;

    fn to_zoned(&self, datetime: LocalDateTime) -> LocalDateTime {
        datetime + Duration::of(self.offset(datetime))
    }
}

#[derive(Debug)]
pub enum LocalTimes<'a> {
    Impossible,

    Precise(ZonedDateTime<'a>),

    Ambiguous { earlier: ZonedDateTime<'a>, later: ZonedDateTime<'a> },
}

impl<'a> LocalTimes<'a> {
    pub fn unwrap_precise(self) -> ZonedDateTime<'a> {
        match self {
            LocalTimes::Precise(p)        => p,
            LocalTimes::Impossible        => panic!("called `LocalTimes::unwrap()` on an `Impossible` value"),
            LocalTimes::Ambiguous { .. }  => panic!("called `LocalTimes::unwrap()` on an `Ambiguous` value: {:?}", self),
        }
    }

    pub fn is_impossible(&self) -> bool {
        match *self {
            LocalTimes::Impossible => true,
            _                      => false,
        }
    }

    pub fn is_ambiguous(&self) -> bool {
        match *self {
            LocalTimes::Ambiguous { .. } => true,
            _                            => false,
        }
    }
}

#[derive(Debug)]
pub struct ZonedDateTime<'a> {
    adjusted: LocalDateTime,
    current_offset: i64,
    time_zone: Box<TimeZone<'a> + 'a>,
}

impl<'a> ZonedDateTime<'a> {
    pub fn to_instant(&self) -> Instant {
        (self.adjusted - Duration::of(self.current_offset)).to_instant()
    }
}

impl<'a> DatePiece for ZonedDateTime<'a> {
    fn year(&self) -> i64 { self.adjusted.year() }
    fn month(&self) -> Month { self.adjusted.month() }
    fn day(&self) -> i8 { self.adjusted.day() }
    fn yearday(&self) -> i16 { self.adjusted.yearday() }
    fn weekday(&self) -> Weekday { self.adjusted.weekday() }
}

impl<'a> TimePiece for ZonedDateTime<'a> {
    fn hour(&self) -> i8 { self.adjusted.hour() }
    fn minute(&self) -> i8 { self.adjusted.minute() }
    fn second(&self) -> i8 { self.adjusted.second() }
    fn millisecond(&self) -> i16 { self.adjusted.millisecond() }
}