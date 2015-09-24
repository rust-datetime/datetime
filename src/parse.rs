use local::{self, LocalDate, LocalTime, LocalDateTime, Month};
use zoned::*;

use regex::Regex;


/// Splits Date String, Time String
///
/// for further parsing by `parse_iso_8601_date` and `parse_iso_8601_time`.
pub fn split_iso_8601(string: &str) -> Result<(&str, &str), Error> {
    let split = Regex::new(r"^([^T]*)T?(.*)$").unwrap();

    if split.is_match(&string) {
        let caps = split.captures(&string).unwrap();
        if caps.len() > 1 {
            return Ok((caps.at(1).unwrap().into(), caps.at(2).unwrap().into()));
        }
    }

    Err(Error::InvalidCharacter)
}

/// Parses a ISO 8601 a string into LocalDateTime Object.
pub fn parse_iso_8601(string: &str) -> Result<LocalDateTime, Error> {
    let (date_string, time_string) = try!(split_iso_8601(string));

    match (parse_iso_8601_date(&date_string), parse_iso_8601_time(&time_string)) {
        (Ok(date), Ok(time)) => Ok(LocalDateTime::from_date_time(date, time)),
        _ => Err(Error::InvalidCharacter)
    }
}


/// Parses ISO 8601 Date a string into a LocalDate Object.
///
/// Used by `LocalDate::parse()`
pub fn parse_iso_8601_date(string: &str) -> Result<LocalDate, Error> {
    let week = Regex::new(r##"(?x)^
        (\d{4})   # year
        -W(\d{2}) # number of week
        -(\d{1})  # day in week (1..7)//}
        $"##).unwrap();

    let ymd = Regex::new(r##"(?x)^
        (\d{4})   # year
        -?(\d{2}) # month
        -?(\d{2}) # day
        $"##).unwrap();

    if ymd.is_match(&string) {
        if let Some(caps) = ymd.captures(string) {
            let year       = caps.at(1).unwrap().parse().unwrap();
            let month_num  = caps.at(2).unwrap().parse().unwrap();
            let month      = Month::from_one(month_num);
            let day        = caps.at(3).unwrap().parse().unwrap();

            LocalDate::ymd(year, month, day).map_err(Error::InvalidDate)
        }
        else {
            Err(Error::InvalidCharacter)
        }
    }
    else if week.is_match(&string) {
        if let Some(caps) = week.captures(string) {
            let year   = caps.at(1).unwrap().parse().unwrap();
            let month  = caps.at(2).unwrap().parse().unwrap();
            let day    = caps.at(3).unwrap().parse().unwrap();

            LocalDate::from_weekday(year, month, day).map_err(Error::InvalidDate)
        }
        else {
            Err(Error::InvalidCharacter)
        }
    }
    else {
        Err(Error::InvalidCharacter)
    }
}

/// Parses ISO 8601 a string into a ZonedDateTime Object.
///
/// Used by `ZonedDateTime::parse()`
pub fn parse_iso_8601_zoned(string: &str) -> Result<(LocalDateTime, TimeZone), Error> {
    let (date_string, time_string) = try!(split_iso_8601(string));

    match (parse_iso_8601_date(&date_string), parse_iso_8601_tuple(&time_string)) {
        (Ok(date), Ok((hour, minute, second, millisecond, zh, zm, z))) => {
            if let Ok(time) = LocalTime::hms_ms(hour, minute, second, millisecond as i16) {
                let time_zone = if z == "Z" {
                    TimeZone::UTC
                }
                else {
                    TimeZone::of_hours_and_minutes(zh, zm)
                };

                Ok((LocalDateTime::from_date_time(date, time), time_zone))
            }
            else {
                Err(Error::InvalidCharacter)
            }
        }
        (Ok(date), Err(Error::InvalidCharacter)) => {
            if let Ok(time) = LocalTime::hms(0, 0, 0) {
                Ok((LocalDateTime::from_date_time(date, time), TimeZone::UTC))
            }
            else {
                Err(Error::InvalidCharacter)
            }
        }
        _ => Err(Error::InvalidCharacter)
    }
}

/// Parses ISO 8601 a string into a LocalTime Object.
///
/// Used by `LocalTime::parse()`
pub fn parse_iso_8601_time(string: &str) -> Result<LocalTime, Error> {
    if string.is_empty() {
        return Ok(LocalTime::hms(0, 0, 0).unwrap());
    }

    if let Ok((hour, minute, second, millisecond, _zh, _zm, _z)) = parse_iso_8601_tuple(string) {
        return LocalTime::hms_ms(hour, minute, second, millisecond as i16).map_err(Error::InvalidDate);
    }

    Err(Error::InvalidCharacter)
}

// implementation detail
fn parse_iso_8601_tuple(string: &str) -> Result<(i8,i8,i8,i32,i8,i8,&str), Error> {
    let exp = Regex::new(r##"(?x) ^
        (\d{2}) :?     # hour
        (\d{2})? :?    # minute

        (?:
            (\d{2})         # second
            \.?
            ((?:\d{1,9}))?  # millisecond
        )?

        (?:                 # time zone offset:
            (Z) |           # or just Z for UTC
            ([+-]\d\d)? :?  # hour and
            (\d\d)?         # minute,
        )?
    $"##).ok().expect("Regex Broken");

    if exp.is_match(&string) {
        if let Some(caps) = exp.captures(string) {
            Ok((
                caps.at(1).unwrap_or("00").parse::<i8>().unwrap(), // HH
                caps.at(2).unwrap_or("00").parse::<i8>().unwrap(), // MM
                caps.at(3).unwrap_or("00").parse::<i8>().unwrap(), // SS
                caps.at(4).unwrap_or("000").parse::<i32>().unwrap(), // MS
                caps.at(6).unwrap_or("+00").trim_matches('+').parse::<i8>().unwrap(), // ZH
                caps.at(7).unwrap_or("00").parse::<i8>().unwrap(), // ZM
                caps.at(5).unwrap_or("_"), // "Z"
            ))

            //TODO: check this with the rfc3339 standard
            //if tup.3 > 0 && &format!("{}", tup.3).len() %3 != 0{ return Err(Error::InvalidCharacter)}
        }
        else {
            Err(Error::InvalidCharacter)
        }
    }
    else {
        Err(Error::InvalidCharacter)
    }
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

// 2014-12-25
// Combined date and time in UTC:   2014-12-25T02:56:40+00:00, 2014-12-25T02:56:40Z
// Week:    2014-W52
// Date with week number:   2014-W52-4
// Ordinal date:    2014-359
