//! Dates and times paired with a time zone, and time zone definitions.

use local::{LocalDateTime, LocalDate, LocalTime, DatePiece, TimePiece, Month, Weekday};
use local::ParseError as LocalParseError;
use parse;
use util::RangeExt;

use std::num::ParseIntError;
use std::str::FromStr;

use duration::Duration;


/// A **time zone** is used to calculate how much to adjust a UTC-based time
/// based on its geographical location.
#[derive(Clone, Debug)]
pub enum Offset {
    UTC,
    FixedOffset { offset: i32 },
}

/// A **time zone** is used to calculate how much to adjust a UTC-based time
/// based on its geographical location.
impl Offset {
    fn adjust(&self, local: LocalDateTime) -> LocalDateTime {
        match *self {
            Offset::UTC                    => local,
            Offset::FixedOffset { offset } => local + Duration::of(offset as i64)
        }
    }

    /// Create a new fixed-offset timezone with the given number of seconds.
    ///
    /// Returns an error if the number of seconds is greater than one day's
    /// worth of seconds (86400) in either direction.
    pub fn of_seconds(seconds: i32) -> Result<Offset, Error> {
        if seconds.is_within(-86400..86401) {
            Ok(Offset::FixedOffset { offset: seconds })
        }
        else {
            Err(Error::OutOfRange)
        }
    }

    /// Create a new fixed-offset timezone with the given number of hours and
    /// minutes.
    ///
    /// The values should either be both positive or both negative.
    ///
    /// Returns an error if the numbers are greater than their unit allows
    /// (more than 23 hours or 59 minutes) in either direction, or if the
    /// values differ in sign (such as a positive number of hours with a
    /// negative number of minutes).
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

    pub fn from_fields(fields: parse::ZoneFields) -> Result<Offset, ParseError> {
        let parse = |input: &str| input.parse().map_err(ParseError::Number);

        let result = match fields {
            parse::ZoneFields::Zulu => return Ok(Offset::UTC),
            parse::ZoneFields::Offset { sign: "+", hours, minutes: None } => Offset::of_hours_and_minutes( try!(parse(hours)), 0),
            parse::ZoneFields::Offset { sign: "-", hours, minutes: None } => Offset::of_hours_and_minutes(-try!(parse(hours)), 0),
            parse::ZoneFields::Offset { sign: "+", hours, minutes: Some(mins) } => Offset::of_hours_and_minutes( try!(parse(hours)),  try!(parse(mins))),
            parse::ZoneFields::Offset { sign: "-", hours, minutes: Some(mins) } => Offset::of_hours_and_minutes(-try!(parse(hours)), -try!(parse(mins))),
            _ => unreachable!(),  // this definitely should be unreachable: the regex only checks for [Z+-].
        };

        result.map_err(ParseError::Zone)
    }
}

impl FromStr for Offset {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Offset, Self::Err> {
        match parse::parse_iso_8601_zone(input) {
            Ok(fields)  => Offset::from_fields(fields),
            Err(e)      => Err(ParseError::Parse(e)),
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
    Date(LocalParseError),
    Number(ParseIntError),
    Parse(parse::Error),
}


#[derive(Debug, Clone)]
pub struct OffsetDateTime {
    local: LocalDateTime,
    time_zone: Offset,
}

impl FromStr for OffsetDateTime {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<OffsetDateTime, Self::Err> {
        let (date_fields, time_fields, zone_fields) = try!(parse::parse_iso_8601_date_time_zone(input).map_err(ParseError::Parse));
        let date = try!(LocalDate::from_fields(date_fields).map_err(ParseError::Date));
        let time = try!(LocalTime::from_fields(time_fields).map_err(ParseError::Date));
        let zone = try!(Offset::from_fields(zone_fields));
        Ok(OffsetDateTime { local: LocalDateTime::new(date, time), time_zone: zone })
    }
}


impl DatePiece for OffsetDateTime {
    fn year(&self) -> i64 {
        self.time_zone.adjust(self.local).year()
    }

    fn month(&self) -> Month {
        self.time_zone.adjust(self.local).month()
    }

    fn day(&self) -> i8 {
        self.time_zone.adjust(self.local).day()
    }

    fn yearday(&self) -> i16 {
        self.time_zone.adjust(self.local).yearday()
    }

    fn weekday(&self) -> Weekday {
        self.time_zone.adjust(self.local).weekday()
    }
}

impl TimePiece for OffsetDateTime {
    fn hour(&self) -> i8 {
        self.time_zone.adjust(self.local).hour()
    }

    fn minute(&self) -> i8 {
        self.time_zone.adjust(self.local).minute()
    }

    fn second(&self) -> i8 {
        self.time_zone.adjust(self.local).second()
    }

    fn millisecond(&self) -> i16 {
        self.time_zone.adjust(self.local).millisecond()
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
