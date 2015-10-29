//! Parsing [ISO-8601](w) formats.
//!
//! [w]: https://en.wikipedia.org/wiki/ISO_8601

use std::error::Error as ErrorTrait;
use std::fmt;

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
    Zulu,

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
        Ok(ZoneFields::Zulu)
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


/// Parses an ISO 8601 string into a pair of `DateFields` and `TimeFields`.
/// The formats allowed are any formats accepted by their two parsers, with
/// the strings separated by a 'T' as per the ISO 8601 standard.
///
/// ### Examples
///
/// ```rust
/// use datetime::parse::{DateFields, TimeFields, parse_iso_8601_date_time};
///
/// let date = DateFields::YD {
///     year: "1969",
///     yearday: "201",
/// };
///
/// let time = TimeFields::HM {
///     hour:   "20",
///     minute: "18",
/// };
///
/// assert_eq!(parse_iso_8601_date_time("1969-201T20:18"), Ok((date, time)));
/// ```
pub fn parse_iso_8601_date_time(input: &str) -> Result<(DateFields, TimeFields), Error> {
    if let Some((date_str, time_str)) = split_date_and_time(input) {
        let date_fields = try!(parse_iso_8601_date(date_str));
        let time_fields = try!(parse_iso_8601_time(time_str));
        Ok((date_fields, time_fields))
    }
    else {
        Err(Error::InvalidFormat)
    }
}


/// Parses an ISO 8601 time string into a pair of `TimeFields` and `ZoneFields`.
/// The formats allowed are any formats accepted by their two parsers, and the
/// strings do not need to be separated by any character.
///
/// ### Examples
///
/// ```rust
/// use datetime::parse::{TimeFields, ZoneFields, parse_iso_8601_time_zone};
///
/// let time = TimeFields::HM {
///     hour:   "14",
///     minute: "45",
/// };
///
/// assert_eq!(parse_iso_8601_time_zone("14:45Z"), Ok((time, ZoneFields::Zulu)));
/// ```
pub fn parse_iso_8601_time_zone(input: &str) -> Result<(TimeFields, ZoneFields), Error> {
    if let Some((time_str, zone_str)) = split_time_and_zone(input) {
        let time_fields = try!(parse_iso_8601_time(time_str));
        let zone_fields = try!(parse_iso_8601_zone(zone_str));
        Ok((time_fields, zone_fields))
    }
    else {
        Err(Error::InvalidFormat)
    }
}


/// Parses an ISO 8601 string into a triple of `DateFields`, `TimeFields`, and `ZoneFields`.
/// The formats allowed are any formats accepted by their three parsers. The
/// date and time must be separated by a `T` as per the ISO 8601 standard, and
/// the time and time zone fields cannot be separated by anything.
///
/// ### Examples
///
/// ```rust
/// use datetime::parse::{DateFields, TimeFields, ZoneFields};
/// use datetime::parse::parse_iso_8601_date_time_zone;
///
/// let date = DateFields::YMD {
///     year:  "2001",
///     month: "09",
///     day:   "09",
/// };
///
/// let time = TimeFields::HMS {
///     hour:   "01",
///     minute: "46",
///     second: "40",
/// };
///
/// let zone = ZoneFields::Offset {
///     sign:    "-",
///     hours:   "13",
///     minutes: Some("37"),
/// };
///
/// assert_eq!(parse_iso_8601_date_time_zone("2001-09-09T01:46:40-1337"), Ok((date, time, zone)));
/// ```
pub fn parse_iso_8601_date_time_zone(input: &str) -> Result<(DateFields, TimeFields, ZoneFields), Error> {
    if let Some((date_time_str, zone_str)) = split_time_and_zone(input) {
        if let Some((date_str, time_str)) = split_date_and_time(date_time_str) {
            let date_fields = try!(parse_iso_8601_date(date_str));
            let time_fields = try!(parse_iso_8601_time(time_str));
            let zone_fields = try!(parse_iso_8601_zone(zone_str));

            return Ok((date_fields, time_fields, zone_fields));
        }
    }

    Err(Error::InvalidFormat)
}


/// Splits the input string around the first 'T' that it finds into date and
/// time components. Returns `None` if it can't find a 'T'. The 'T' is
/// necessary as the date/time delimiter.
fn split_date_and_time(input: &str) -> Option<(&str, &str)> {
    match input.find('T') {
        Some(index) => Some(( &input[.. index], &input[index + 1 ..] )),
        None        => None,
    }
}

/// Splits the input string up to the first 'Z', '+', or '-' that it finds
/// into time and time zone components. Returns `None` if it can't find any of
/// those characters. Time zones must begin with one of those characters,
/// which is why it's used to split them here.
fn split_time_and_zone(input: &str) -> Option<(&str, &str)> {
    match input.rfind(|c| c == 'Z' || c == '+' || c == '-') {
        Some(index) => Some((  &input[.. index], &input[index ..] )),
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        "invalid format"
    }
}

#[cfg(test)]
mod test {
    pub use super::*;

    macro_rules! test_parse {
        ($name:ident: $left:expr => $right:expr) => {
            #[test]
            fn $name() {
                assert_eq!($left, $right);
            }
        }
    }

    mod ymd {
        use super::*;

        static EXPECTED: DateFields<'static> = DateFields::YMD {
            year: "2014",
            month: "12",
            day: "25",
        };

        test_parse!(hyphens:    parse_iso_8601_date("2014-12-25")  => Ok(EXPECTED));
        test_parse!(no_hyphens: parse_iso_8601_date("20141225")    => Ok(EXPECTED));
        test_parse!(far_future: parse_iso_8601_date("19424-07-11") => Err(Error::InvalidFormat));
        test_parse!(wibble:     parse_iso_8601_date("wibble")      => Err(Error::InvalidFormat));
    }

    mod ywd {
        use super::*;

        static EXPECTED: DateFields<'static> = DateFields::YWD {
            year: "2014",
            week: "22",
            weekday: "3",
        };

        test_parse!(hyphens:    parse_iso_8601_date("2014-W22-3")  => Ok(EXPECTED));
        test_parse!(no_hyphens: parse_iso_8601_date("2014W223")    => Ok(EXPECTED));
        test_parse!(no_day:     parse_iso_8601_date("2014-W22")    => Err(Error::InvalidFormat));
        test_parse!(lowercase:  parse_iso_8601_date("2014-w22-3")  => Err(Error::InvalidFormat));
        test_parse!(blarg:      parse_iso_8601_date("blarg")       => Err(Error::InvalidFormat));
    }

    mod yd {
        use super::*;

        static EXPECTED: DateFields<'static> = DateFields::YD {
            year: "2014",
            yearday: "123",
        };

        test_parse!(hyphens:    parse_iso_8601_date("2014-123")  => Ok(EXPECTED));
        test_parse!(no_hyphens: parse_iso_8601_date("2014123")   => Ok(EXPECTED));
        test_parse!(two_digits: parse_iso_8601_date("2014-12")   => Err(Error::InvalidFormat));
        test_parse!(fizzle:     parse_iso_8601_date("fizzle")    => Err(Error::InvalidFormat));
    }

    mod hm {
        use super::*;

        static EXPECTED: TimeFields<'static> = TimeFields::HM {
            hour: "14",
            minute: "45",
        };

        test_parse!(colons: parse_iso_8601_time("14:45") => Ok(EXPECTED));
        test_parse!(nolons: parse_iso_8601_time("1445")  => Ok(EXPECTED));
    }

    mod hms {
        use super::*;

        static EXPECTED: TimeFields<'static> = TimeFields::HMS {
            hour:   "14",
            minute: "45",
            second: "12",
        };

        test_parse!(colons: parse_iso_8601_time("14:45:12") => Ok(EXPECTED));
        test_parse!(nolons: parse_iso_8601_time("144512")   => Ok(EXPECTED));
    }

    mod hms_ms {
        use super::*;

        static EXPECTED: TimeFields<'static> = TimeFields::HMSms {
            hour:   "14",
            minute: "45",
            second: "12",
            millisecond: "753",
        };

        test_parse!(colons:  parse_iso_8601_time("14:45:12.753") => Ok(EXPECTED));
        test_parse!(nolons:  parse_iso_8601_time("144512.753")   => Ok(EXPECTED));
        test_parse!(extra:   parse_iso_8601_time("144512.7538")  => Err(Error::InvalidFormat));
        test_parse!(fewer:   parse_iso_8601_time("144512.75")    => Err(Error::InvalidFormat));
        test_parse!(dotless: parse_iso_8601_time("144512753")    => Err(Error::InvalidFormat));
    }

    mod datetimes {
        use super::*;

        static EXPECTED: (DateFields<'static>, TimeFields<'static>) = (
            DateFields::YMD {
                year:  "2001",
                month: "02",
                day:   "03",
            },
            TimeFields::HMS {
                hour:   "04",
                minute: "05",
                second: "06",
            },
        );

        test_parse!(hyphens:    parse_iso_8601_date_time("2001-02-03T04:05:06") => Ok(EXPECTED));
        test_parse!(no_hyphens: parse_iso_8601_date_time("20010203T040506")     => Ok(EXPECTED));
        test_parse!(lowercase:  parse_iso_8601_date_time("2001-02-03t04:05:06") => Err(Error::InvalidFormat));
    }

    mod zones {
        use super::*;

        static EXPECTED: ZoneFields<'static> = ZoneFields::Offset {
            sign:    "+",
            hours:   "11",
            minutes: Some("15"),
        };

        test_parse!(zulu:  parse_iso_8601_zone("Z") => Ok(ZoneFields::Zulu));
        test_parse!(lower: parse_iso_8601_zone("z") => Err(Error::InvalidFormat));

        test_parse!(hyphen:    parse_iso_8601_zone("+11:15")  => Ok(EXPECTED));
        test_parse!(no_hyphen: parse_iso_8601_zone("+1115")   => Ok(EXPECTED));
        test_parse!(no_sign:   parse_iso_8601_zone("11:15")   => Err(Error::InvalidFormat));
        test_parse!(both:      parse_iso_8601_zone("Z11:15")  => Err(Error::InvalidFormat));
    }

    mod timezones {
        use super::*;

        static EXPECTED: (TimeFields<'static>, ZoneFields<'static>) = (
            TimeFields::HMS {
                hour:   "04",
                minute: "05",
                second: "06",
            },
            ZoneFields::Offset {
                sign:    "-",
                hours:   "11",
                minutes: Some("15"),
            },
        );

        test_parse!(delimiters:     parse_iso_8601_time_zone("04:05:06-11:15") => Ok(EXPECTED));
        test_parse!(no_delimiters:  parse_iso_8601_time_zone("040506-1115")    => Ok(EXPECTED));
    }

    mod datetimezones {
        use super::*;

        static EXPECTED: (DateFields<'static>, TimeFields<'static>, ZoneFields<'static>) = (
            DateFields::YMD {
                year:  "2001",
                month: "09",
                day:   "09",
            },
            TimeFields::HMS {
                hour:   "01",
                minute: "46",
                second: "40",
            },
            ZoneFields::Offset {
                sign:    "-",
                hours:   "13",
                minutes: Some("37"),
            },
        );

        test_parse!(moon:     parse_iso_8601_date_time_zone("2001-09-09T01:46:40-1337") => Ok(EXPECTED));
        test_parse!(timeless: parse_iso_8601_date_time_zone("2001-09-09TZ") => Err(Error::InvalidFormat));
    }
}
