use std::error::Error as ErrorTrait;
use std::fmt;
use std::str::FromStr;

use iso8601;

use cal::datetime::{LocalDate, LocalTime, LocalDateTime, Month, Weekday, Error as DateTimeError};
use cal::offset::{Offset, OffsetDateTime, Error as OffsetError};


impl FromStr for LocalDate {
    type Err = Error<DateTimeError>;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match iso8601::date(input) {
            Ok(fields)  => fields_to_date(fields).map_err(Error::Date),
            Err(e)      => Err(Error::Parse(e)),
        }
    }
}

impl FromStr for LocalTime {
    type Err = Error<DateTimeError>;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match iso8601::time(input) {
            Ok(fields)  => fields_to_time(fields).map_err(Error::Date),
            Err(e)      => Err(Error::Parse(e)),
        }
    }
}

impl FromStr for LocalDateTime {
    type Err = Error<DateTimeError>;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let fields = match iso8601::datetime(input) {
            Ok(fields)  => fields,
            Err(e)      => return Err(Error::Parse(e)),
        };

        let date = fields_to_date(fields.date).map_err(Error::Date)?;
        let time = fields_to_time(fields.time).map_err(Error::Date)?;
        Ok(Self::new(date, time))
    }
}

impl FromStr for OffsetDateTime {
    type Err = Error<OffsetError>;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let fields = match iso8601::datetime(input) {
            Ok(fields)  => fields,
            Err(e)      => return Err(Error::Parse(e)),
        };

        let date   = fields_to_date(fields.date).map_err(|e| Error::Date(OffsetError::Date(e)))?;
        let time   = fields_to_time(fields.time).map_err(|e| Error::Date(OffsetError::Date(e)))?;
        let offset = Offset::of_hours_and_minutes(fields.time.tz_offset_hours as i8, fields.time.tz_offset_minutes as i8).map_err(Error::Date)?;
        Ok(offset.transform_date(LocalDateTime::new(date, time)))
    }
}


fn fields_to_date(fields: iso8601::Date) -> Result<LocalDate, DateTimeError> {
    if let iso8601::Date::YMD { year, month, day } = fields {
        let month_variant = Month::from_one(month as i8)?;
        LocalDate::ymd(year as i64, month_variant, day as i8)
    }
    else if let iso8601::Date::Week { year, ww, d } = fields {
        let weekday_variant = Weekday::from_one(d as i8)?;
        LocalDate::ywd(year as i64, ww as i64, weekday_variant)
    }
    else if let iso8601::Date::Ordinal { year, ddd } = fields {
        LocalDate::yd(year as i64, ddd as i64)
    }
    else {
        unreachable!()  // should be unnecessary??
    }
}

fn fields_to_time(fields: iso8601::Time) -> Result<LocalTime, DateTimeError> {
    let h  = fields.hour as i8;
    let m  = fields.minute as i8;
    let s  = fields.second as i8;
    let ms = fields.millisecond as i16;

    LocalTime::hms_ms(h, m, s, ms)
}


#[derive(PartialEq, Debug, Clone)]
pub enum Error<E: ErrorTrait> {
    Date(E),
    Parse(String),
}

impl<E: ErrorTrait> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Date(ref error)    => write!(f, "parsing resulted in an invalid date: {}", error),
            Error::Parse(ref string)  => write!(f, "parse error: {}", string),
        }
    }
}

impl<E: ErrorTrait> ErrorTrait for Error<E> {
    fn cause(&self) -> Option<&dyn ErrorTrait> {
        match *self {
            Error::Date(ref error)  => Some(error),
            Error::Parse(_)         => None,
        }
    }
}
