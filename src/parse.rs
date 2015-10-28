use local::{self, LocalDate, LocalTime, LocalDateTime, Month};
use zoned::*;

use iso8601;
use iso8601::Date;
use nom::IResult::*;

/// Parses a ISO 8601 a string into LocalDateTime Object.
pub fn parse_iso_8601(string: &str) -> Result<LocalDateTime, Error> {
    if let Done(_,parsed) =  iso8601::datetime(string.as_bytes()){
        let date = match parsed.date {
            Date::YMD{year, month, day} => LocalDate::ymd(year as i64, Month::from_one(month as i8), day as i8),
            Date::Week{year, ww, d}     => LocalDate::from_weekday(year as i64, ww as i64 , d as i64),
            Date::Ordinal{ year, ddd }  => LocalDate::from_yearday(year as i64, ddd as i64)
        };
        let time = LocalTime::hms_ms(parsed.time.hour as i8, parsed.time.minute as i8, parsed.time.second as i8, 8i16);
        return Ok(
            LocalDateTime::from_date_time(date.unwrap(), time.unwrap())
            );
    }
    Err(Error::InvalidCharacter)
}


/// Parses ISO 8601 Date a string into a LocalDate Object.
///
/// Used by `LocalDate::parse()`
pub fn parse_iso_8601_date(string: &str) -> Result<LocalDate, Error> {
    if let Done(_,parsed) =  iso8601::date(string.as_bytes()){
        return match parsed {
            Date::YMD{year, month, day} => LocalDate::ymd(year as i64, Month::from_one(month as i8), day as i8).map_err(Error::InvalidDate),
            Date::Week{year, ww, d}     => LocalDate::from_weekday(year as i64, ww as i64 , d as i64).map_err(Error::InvalidDate),
            Date::Ordinal{ year, ddd }  => LocalDate::from_yearday(year as i64, ddd as i64).map_err(Error::InvalidDate)
        };
    }

    Err(Error::InvalidCharacter)
}


/// Parses ISO 8601 a string into a ZonedDateTime Object.
///
/// Used by `ZonedDateTime::parse()`
pub fn parse_iso_8601_zoned(string: &str) -> Result<(LocalDateTime, TimeZone), Error> {
    if let Done(_,parsed) =  iso8601::datetime(string.as_bytes()){
        let date = match parsed.date {
            Date::YMD{year, month, day} => LocalDate::ymd(year as i64, Month::from_one(month as i8), day as i8),
            Date::Week{year, ww, d}     => LocalDate::from_weekday(year as i64, ww as i64 , d as i64),
            Date::Ordinal{ year, ddd }  => LocalDate::from_yearday(year as i64, ddd as i64)
        };
        let time = LocalTime::hms_ms(parsed.time.hour as i8, parsed.time.minute as i8, parsed.time.second as i8, 8i16);
        return Ok(
            (LocalDateTime::from_date_time(date.unwrap(), time.unwrap()),
            TimeZone::of_hours_and_minutes((parsed.time.tz_offset/3600) as i8, (parsed.time.tz_offset/60) as i8)))
            ;
    }
    Err(Error::InvalidCharacter)
}

/// Parses ISO 8601 a string into a LocalTime Object.
///
/// Used by `LocalTime::parse()`
pub fn parse_iso_8601_time(string: &str) -> Result<LocalTime, Error> {
    //if string.is_empty() { return Ok(LocalTime::hms(0, 0, 0).unwrap()); }
    if let Done(_,parsed) =  iso8601::time(string.as_bytes()){
        return LocalTime::hms_ms(parsed.hour as i8,
                                 parsed.minute as i8,
                                 parsed.second as i8,
                                 8i16).map_err(Error::InvalidDate);
    }

    Err(Error::InvalidCharacter)
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Error {
    InvalidCharacter,
    InvalidDate(local::Error),
}


#[cfg(test)]
mod test {
    pub use super::{parse_iso_8601_date, Error};
    pub use local::{LocalDate, Month};

    #[test]
    fn date() {
        let date = parse_iso_8601_date("1985-04-12").unwrap();
        assert_eq!(date, LocalDate::ymd(1985, Month::April, 12).unwrap());
    }

    #[test]
    fn fail() {
        let date = parse_iso_8601_date("");
        assert_eq!(date, Err(Error::InvalidCharacter));
    }
}
