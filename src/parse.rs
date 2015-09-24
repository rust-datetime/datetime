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
        ( ?P<millis> \d{3} )
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

        /// The year component, which should be a **four**-digit string.
        year: &'a str,

        /// The month component, which should be a **two**-digit string.
        month: &'a str,

        /// The day component, which should also be a **two**-digit string.
        day: &'a str,
    },

    /// Year, week-of-year, and day-of-week components.
    YWD {

        /// The year component, which should be a **four**-digit string.
        year: &'a str,

        /// The week-of-year component, which should be a **two**-digit string.
        week: &'a str,

        /// The weekday component, which should also be a **single**-digit
        /// string from 1 (Monday) to 7 (Sunday).
        weekday: &'a str,
    },

    /// Ordinal year and day-of-year components.
    YD {

        /// The year component, which should be a **four**-digit string.
        year: &'a str,

        /// The day component, which should also be a **three**-digit string.
        yearday: &'a str,
    },
}


/// A set of string fields representing time components.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TimeFields<'a> {

    /// Hour and minute components.
    HM {

        /// The hour component, which should be a **two**-digit string.
        hour: &'a str,

        /// The minute component, which should also be a **two**-digit string.
        minute: &'a str,
    },

    /// Hour, minute, and second components.
    HMS {
        /// The hour component, which should be a **two**-digit string.
        hour: &'a str,

        /// The minute component, which should also be a **two**-digit string.
        minute: &'a str,

        /// The second component, which should also also be a **two**-digit
        /// string.
        second: &'a str,
    },

    /// Hour, minute, second, and millisecond components.
    HMSms {
        /// The hour component, which should be a **two**-digit string.
        hour: &'a str,

        /// The minute component, which should also be a **two**-digit string.
        minute: &'a str,

        /// The second component, which should also also be a **two**-digit
        /// string.
        second: &'a str,

        /// The millisecond component, which should be a **three**-digit string.
        millisecond: &'a str,
    }
}


/// A set of string fields representing time zone components.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ZoneFields<'a> {

    /// A single "Z", indicating UTC time.
    Zulu {

        /// The Z!
        z: &'a str,
    },

    Offset {
        /// The sign, a plus or a minus.
        sign: &'a str,

        /// The hours component, which should be a **two**-digit string.
        hours: &'a str,

        /// The minutes component, which should also be a **two**-digit string.
        minutes: Option<&'a str>,
    }
}


/// Parses an ISO 8601 date string into a set of `DateFields`.
/// It accepts the following formats:
///
/// - `yyyy-mm-dd`
/// - `yyyy-Www`
/// - `yyyy-ddd`
///
/// ### Examples
///
/// ```rust
/// use datetime::parse::{DateFields, parse_iso_8601_date};
///
/// let date = DateFields::YMD {
///     year:  "2015",
///     month: "12",
///     day:   "25",
/// };
///
/// assert_eq!(parse_iso_8601_date("2015-12-25"), Ok(date));
/// ```
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


/// Parses an ISO 8601 time string into a set of `TimeFields`.
/// It accepts the following formats:
///
/// - `hh:mm`
/// - `hh:mm:ss`
/// - `hh:mm:ss.SSS`
///
/// ### Examples
///
/// ```rust
/// use datetime::parse::{TimeFields, parse_iso_8601_time};
///
/// let time = TimeFields::HMS {
///     hour:   "17",
///     minute: "30",
///     second: "00",
/// };
///
/// assert_eq!(parse_iso_8601_time("17:30:00"), Ok(time));
/// ```
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


/// Parses an ISO 8601 time zone string into a set of `ZoneFields`.
/// It accepts the following formats:
///
/// - `Z`
/// - `±hh:mm`
/// - `±hh`
///
/// ### Examples
///
/// ```rust
/// use datetime::parse::{ZoneFields, parse_iso_8601_zone};
///
/// let zone = ZoneFields::Offset {
///     sign:    "+",
///     hours:   "02",
///     minutes: Some("30"),
/// };
///
/// assert_eq!(parse_iso_8601_zone("+02:30"), Ok(zone));
/// ```
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

/// Splits the input string around the first 'T' that it finds.
/// Returns `None` if it can't find a 'T'.
fn split_date_and_time(input: &str) -> Option<(&str, &str)> {
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



#[cfg(test)]
mod test {
    pub use super::*;

    mod dates {
        use super::*;

        #[test]
        fn ymd() {
            let expected = DateFields::YMD {
                year: "2014",
                month: "12",
                day: "25",
            };

            assert_eq!(parse_iso_8601_date("2014-12-25"), Ok(expected));
            assert_eq!(parse_iso_8601_date("20141225"), Ok(expected));
        }

        #[test]
        fn ymd_fails() {
            assert!(parse_iso_8601_date("wibble").is_err());
            assert!(parse_iso_8601_date("19424-07-11").is_err());
        }


        #[test]
        fn ywd() {
            let expected = DateFields::YWD {
                year: "2014",
                week: "22",
                weekday: "3",
            };

            assert_eq!(parse_iso_8601_date("2014-W22-3"), Ok(expected));
            assert_eq!(parse_iso_8601_date("2014W223"), Ok(expected));
        }

        #[test]
        fn ywd_fails() {
            assert!(parse_iso_8601_date("2014-W22").is_err());
            assert!(parse_iso_8601_date("2014-w22-3").is_err());
        }


        #[test]
        fn yd() {
            let expected = DateFields::YD {
                year: "2014",
                yearday: "123",
            };

            assert_eq!(parse_iso_8601_date("2014-123"), Ok(expected));
            assert_eq!(parse_iso_8601_date("2014123"), Ok(expected));
        }

        #[test]
        fn yd_fails() {
            assert!(parse_iso_8601_date("2014-12").is_err());
        }
    }

    mod times {
        use super::*;

        #[test]
        fn hm() {
            let expected = TimeFields::HM {
                hour: "14",
                minute: "45",
            };

            assert_eq!(parse_iso_8601_time("14:45"), Ok(expected));
            assert_eq!(parse_iso_8601_time("1445"), Ok(expected));
        }


        #[test]
        fn hms() {
            let expected = TimeFields::HMS {
                hour: "14",
                minute: "45",
                second: "12",
            };

            assert_eq!(parse_iso_8601_time("14:45:12"), Ok(expected));
            assert_eq!(parse_iso_8601_time("144512"), Ok(expected));
        }

        #[test]
        fn hms_ms() {
            let expected = TimeFields::HMSms {
                hour: "14",
                minute: "45",
                second: "12",
                millisecond: "753",
            };

            assert_eq!(parse_iso_8601_time("14:45:12.753"), Ok(expected));
            assert_eq!(parse_iso_8601_time("144512.753"), Ok(expected));
        }

        #[test]
        fn hms_ms_fails() {
            assert!(parse_iso_8601_time("144512753").is_err());
        }
    }

    mod datetimes {
        use super::*;

        #[test]
        fn ymd_hms() {
            let expected_date = DateFields::YMD {
                year: "2001",
                month: "02",
                day: "03",
            };

            let expected_time = TimeFields::HMS {
                hour: "04",
                minute: "05",
                second: "06",
            };

            assert_eq!(parse_iso_8601("2001-02-03T04:05:06"), Ok((expected_date, expected_time)));
            assert_eq!(parse_iso_8601("20010203T040506"), Ok((expected_date, expected_time)));
        }

        #[test]
        fn lowercase_t() {
            assert!(parse_iso_8601_time("2001-02-03T04:05:06").is_err());
        }
    }
}
