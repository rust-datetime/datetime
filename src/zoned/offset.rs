//! Dates and times paired with a time zone, and time zone definitions.

use std::str::FromStr;

use iso8601;

use duration::Duration;
use local::{LocalDateTime, LocalDate, LocalTime, DatePiece, TimePiece, Month, Weekday};
use local::ParseError as LocalParseError;
use util::RangeExt;


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Offset {
    offset_seconds: Option<i32>,
}

impl Offset {
    fn adjust(&self, local: LocalDateTime) -> LocalDateTime {
        match self.offset_seconds {
            Some(s) => local + Duration::of(s as i64),
            None    => local,
        }
    }

    pub fn utc() -> Offset {
        Offset { offset_seconds: None }
    }

    pub fn of_seconds(seconds: i32) -> Result<Offset, Error> {
        if seconds.is_within(-86400..86401) {
            Ok(Offset { offset_seconds: Some(seconds) })
        }
        else {
            Err(Error::OutOfRange)
        }
    }

    pub fn of_hours_and_minutes(hours: i8, minutes: i8) -> Result<Offset, Error> {
        if (hours.is_positive() && minutes.is_negative())
        || (hours.is_negative() && minutes.is_positive()) {
            Err(Error::SignMismatch)
        }
        else if hours <= -24 || hours >= 24 {
            Err(Error::OutOfRange)
        }
        else if minutes <= -60 || minutes >= 60 {
            Err(Error::OutOfRange)
        }
        else {
            let hours = hours as i32;
            let minutes = minutes as i32;
            Offset::of_seconds(hours * 24 + minutes * 60)
        }
    }
}


#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Error {
    OutOfRange,
    SignMismatch,
}

#[derive(PartialEq, Debug, Clone)]
pub enum ParseError {
    Zone(Error),
    Local(LocalParseError),
    Parse(String),
}


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct OffsetDateTime {
    local: LocalDateTime,
    offset: Offset,
}

impl FromStr for OffsetDateTime {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<OffsetDateTime, Self::Err> {
        let fields = match iso8601::datetime(input) {
            Ok(fields)  => fields,
            Err(e)      => return Err(ParseError::Parse(e)),
        };

        let date   = try!(LocalDate::from_fields(fields.date).map_err(ParseError::Local));
        let time   = try!(LocalTime::from_fields(fields.time).map_err(ParseError::Local));
        let offset = try!(Offset::of_hours_and_minutes(fields.time.tz_offset_hours as i8, fields.time.tz_offset_minutes as i8).map_err(ParseError::Zone));
        Ok(OffsetDateTime {
            local: LocalDateTime::new(date, time),
            offset: offset,
        })
    }
}


impl DatePiece for OffsetDateTime {
    fn year(&self) -> i64 {
        self.offset.adjust(self.local).year()
    }

    fn month(&self) -> Month {
        self.offset.adjust(self.local).month()
    }

    fn day(&self) -> i8 {
        self.offset.adjust(self.local).day()
    }

    fn yearday(&self) -> i16 {
        self.offset.adjust(self.local).yearday()
    }

    fn weekday(&self) -> Weekday {
        self.offset.adjust(self.local).weekday()
    }
}

impl TimePiece for OffsetDateTime {
    fn hour(&self) -> i8 {
        self.offset.adjust(self.local).hour()
    }

    fn minute(&self) -> i8 {
        self.offset.adjust(self.local).minute()
    }

    fn second(&self) -> i8 {
        self.offset.adjust(self.local).second()
    }

    fn millisecond(&self) -> i16 {
        self.offset.adjust(self.local).millisecond()
    }
}


#[cfg(test)]
mod test {
    use super::Offset;

    #[test]
    fn fixed_seconds() {
        assert!(Offset::of_seconds(1234).is_ok());
    }

    #[test]
    fn fixed_seconds_panic() {
        assert!(Offset::of_seconds(100_000).is_err());
    }

    #[test]
    fn fixed_hm() {
        assert!(Offset::of_hours_and_minutes(5, 30).is_ok());
    }

    #[test]
    fn fixed_hm_negative() {
        assert!(Offset::of_hours_and_minutes(-3, -45).is_ok());
    }

    #[test]
    fn fixed_hm_err() {
        assert!(Offset::of_hours_and_minutes(8, 60).is_err());
    }

    #[test]
    fn fixed_hm_signs() {
        assert!(Offset::of_hours_and_minutes(-4, 30).is_err());
    }

    #[test]
    fn fixed_hm_signs_zero() {
        assert!(Offset::of_hours_and_minutes(4, 0).is_ok());
    }
}
