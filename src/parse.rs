//! Parsing [ISO-8601](w) formats.
//!
//! [w]: https://en.wikipedia.org/wiki/ISO_8601

use regex::Regex;

/// A set of regexes to test against.
///
/// All of these regexes use the `(?x)` flag, which means they support
/// comments and whitespace directly in the regex string!
lazy_static! {
    static ref YMD_REGEX: Regex = Regex::new(r##"(?x) ^
        ( ?P<year>  \d{4} ) -?
        ( ?P<month> \d{2} ) -?
        ( ?P<day>   \d{2} )
    $ "##).unwrap();

    static ref YWD_REGEX: Regex = Regex::new(r##"(?x) ^
        ( ?P<year>  \d{4} ) -?
      W ( ?P<week>  \d{2} ) -?
        ( ?P<day>   \d{1} )
    $ "##).unwrap();

    static ref YD_REGEX: Regex = Regex::new(r##"(?x) ^
        ( ?P<year>  \d{4} ) -?
        ( ?P<day>   \d{3} )
    $ "##).unwrap();

    static ref HM_REGEX: Regex = Regex::new(r##"(?x) ^
        ( ?P<hour>   \d{2} ) :?
        ( ?P<minute> \d{2} )
    $ "##).unwrap();

    static ref HMS_REGEX: Regex = Regex::new(r##"(?x) ^
        ( ?P<hour>   \d{2} ) :?
        ( ?P<minute> \d{2} ) :?
        ( ?P<second> \d{2} )
    $ "##).unwrap();

    static ref HMS_MS_REGEX: Regex = Regex::new(r##"(?x) ^
        ( ?P<hour>   \d{2} ) :?
        ( ?P<minute> \d{2} ) :?
        ( ?P<second> \d{2} ) .
        ( ?P<millis> \d{3} ) .
    $ "##).unwrap();

    static ref TZ_REGEX: Regex = Regex::new(r##"(?x) ^
        ( ?P<sign>    [+-]  )
        ( ?P<hours>   \d{2} ) :?
        ( ?P<minutes> \d{2} )?
    $ "##).unwrap();
}


/// A set of string fields representing date components.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum DateFields<'a> {

    /// Year, month, and day components.
    YMD {

        /// The year component, which should be a *four*-digit string.
        year: &'a str,

        /// The month component, which should be a *two*-digit string.
        month: &'a str,

        /// The day component, which should also be a *two*-digit string.
        day: &'a str,
    },

    /// Year, week-of-year, and day-of-week components.
    YWD {

        /// The year component, which should be a *four*-digit string.
        year: &'a str,

        /// The week-of-year component, which should be a *two*-digit string.
        week: &'a str,

        /// The weekday component, which should also be a *single*-digit
        /// string from 1 (Monday) to 7 (Sunday).
        weekday: &'a str,
    },

    /// Ordinal year and day-of-year components.
    YD {

        /// The year component, which should be a *four*-digit string.
        year: &'a str,

        /// The day component, which should also be a *three*-digit string.
        yearday: &'a str,
    },
}


#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TimeFields<'a> {

    HM {
        hour: &'a str,
        minute: &'a str,
    },

    HMS {
        hour: &'a str,
        minute: &'a str,
        second: &'a str,
    },

    HMSms {
        hour: &'a str,
        minute: &'a str,
        second: &'a str,
        millisecond: &'a str,
    }
}


#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ZoneFields<'a> {
    Zulu {
        z: &'a str,
    },

    Offset {
        sign: &'a str,
        hours: &'a str,
        minutes: Option<&'a str>,
    }
}


/// Parses an ISO 8601 date string into a set of `DateFields`.
pub fn parse_iso_8601_date(input: &str) -> Result<DateFields, Error> {
    if let Some(caps) = YMD_REGEX.captures(input) {
        Ok(DateFields::YMD {
            year:   caps.name("year").unwrap(),
            month:  caps.name("month").unwrap(),
            day:    caps.name("day").unwrap(),
        })
    }
    else if let Some(caps) = YWD_REGEX.captures(input) {
        Ok(DateFields::YWD {
            year:     caps.name("year").unwrap(),
            week:     caps.name("week").unwrap(),
            weekday:  caps.name("day").unwrap(),
        })
    }
    else if let Some(caps) = YD_REGEX.captures(input) {
        Ok(DateFields::YD {
            year:     caps.name("year").unwrap(),
            yearday:  caps.name("day").unwrap(),
        })
    }
    else {
        Err(Error::InvalidFormat)
    }
}


pub fn parse_iso_8601_time(input: &str) -> Result<TimeFields, Error> {
    if let Some(caps) = HM_REGEX.captures(input) {
        Ok(TimeFields::HM {
            hour:   caps.name("hour").unwrap(),
            minute: caps.name("minute").unwrap(),
        })
    }
    else if let Some(caps) = HMS_REGEX.captures(input) {
        Ok(TimeFields::HMS {
            hour:   caps.name("hour").unwrap(),
            minute: caps.name("minute").unwrap(),
            second: caps.name("second").unwrap(),
        })
    }
    else if let Some(caps) = HMS_MS_REGEX.captures(input) {
        Ok(TimeFields::HMSms {
            hour:   caps.name("hour").unwrap(),
            minute: caps.name("minute").unwrap(),
            second: caps.name("second").unwrap(),
            millisecond: caps.name("millis").unwrap(),
        })
    }
    else {
        Err(Error::InvalidFormat)
    }
}


pub fn parse_iso_8601_zone(input: &str) -> Result<ZoneFields, Error> {
    if input == "Z" {
        Ok(ZoneFields::Zulu { z: input })
    }
    else if let Some(caps) = TZ_REGEX.captures(input) {
        Ok(ZoneFields::Offset {
            sign:    caps.name("sign").unwrap(),
            hours:   caps.name("hours").unwrap(),
            minutes: caps.name("minutes"),  // minutes is Optional!
        })
    }
    else {
        Err(Error::InvalidFormat)
    }
}

pub fn parse_iso_8601(input: &str) -> Result<(DateFields, TimeFields), Error> {
    if let Some((date_str, time_str)) = split_date_and_time(input) {
        let date_fields = try!(parse_iso_8601_date(date_str));
        let time_fields = try!(parse_iso_8601_time(time_str));
        Ok((date_fields, time_fields))
    }
    else {
        Err(Error::InvalidFormat)
    }
}

pub fn split_date_and_time(input: &str) -> Option<(&str, &str)> {
    match input.bytes().position(|c| c == b'T') {
        Some(index) => Some(( &input[.. index], &input[index + 1 ..] )),
        None        => None,
    }
}

/// An error that can occur while trying to parse a date, time, or zone string.
///
/// Unfortunately, as this whole thing is implemented using regexes, there
/// isn't very much that we can do in the way of error diagnostics: either the
/// string matches, or it doesn't.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Error {

    /// The input string didn't match the format required.
    InvalidFormat,
}
