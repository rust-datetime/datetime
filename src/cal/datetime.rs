use std::cmp::{Ordering, PartialOrd};
use std::error::Error as ErrorTrait;
use std::fmt;
use std::ops::{Add, Sub};
use std::str::FromStr;

use iso8601;

use duration::Duration;
use instant::Instant;
use now;
use util::RangeExt;
use cal::{DatePiece, TimePiece};

use self::Month::*;
use self::Weekday::*;


/// Number of days guaranteed to be in four years.
const DAYS_IN_4Y:   i64 = 365 *   4 +  1;

/// Number of days guaranteed to be in a hundred years.
const DAYS_IN_100Y: i64 = 365 * 100 + 24;

/// Number of days guaranteed to be in four hundred years.
const DAYS_IN_400Y: i64 = 365 * 400 + 97;

/// Number of seconds in a day. As everywhere in this library, leap seconds
/// are simply ignored.
const SECONDS_IN_DAY: i64 = 86400;


/// Number of days between  **1st January, 1970** and **1st March, 2000**.
///
/// This might seem like an odd number to calculate, instead of using the
/// 1st of January as a reference point, but it turs out that by having the
/// reference point immediately after a possible leap-year day, the maths
/// needed to calculate the day/week/month of an instant comes out a *lot*
/// simpler!
///
/// The Gregorian calendar operates on a 400-year cycle, so the combination
/// of having it on a year that's a multiple of 400, and having the leap
/// day at the very end of one of these cycles, means that the calculations
/// are reduced to simple division (of course, with a bit of date-shifting
/// to base a date around this reference point).
///
/// Rust has the luxury of having been started *after* this date. In Win32,
/// the epoch is midnight, the 1st of January, 1601, for much the same
/// reasons - except that it was developed before the year 2000, so they
/// had to go all the way back to the *previous* 400-year multiple.[^win32]
///
/// The only problem is that many people assume the Unix epoch to be
/// midnight on the 1st January 1970, so this value (and any functions that
/// depend on it) aren't exposed to users of this library.
///
/// [^win32]: http://blogs.msdn.com/b/oldnewthing/archive/2009/03/06/9461176.aspx
///
const EPOCH_DIFFERENCE: i64 = (30 * 365      // 30 years between 2000 and 1970...
                               + 7           // plus seven days for leap years...
                               + 31 + 29);   // plus all the days in January and February in 2000.


/// This rather strange triangle is an array of the number of days elapsed
/// at the end of each month, starting at the beginning of March (the first
/// month after the EPOCH above), going backwards, ignoring February.
const TIME_TRIANGLE: &'static [i64; 11] =
    &[31 + 30 + 31 + 30 + 31 + 31 + 30 + 31 + 30 + 31 + 31,  // January
      31 + 30 + 31 + 30 + 31 + 31 + 30 + 31 + 30 + 31,  // December
      31 + 30 + 31 + 30 + 31 + 31 + 30 + 31 + 30,  // November
      31 + 30 + 31 + 30 + 31 + 31 + 30 + 31,  // October
      31 + 30 + 31 + 30 + 31 + 31 + 30,  // September
      31 + 30 + 31 + 30 + 31 + 31,  // August
      31 + 30 + 31 + 30 + 31,  // July
      31 + 30 + 31 + 30,  // June
      31 + 30 + 31,  // May
      31 + 30,  // April
      31]; // March



/// A **local date** is a day-long span on the timeline, *without a time
/// zone*.
#[derive(Eq, Clone, Copy)]
pub struct LocalDate {
    ymd:     YMD,
    yearday: i16,
    weekday: Weekday,
}

/// A **local time** is a time on the timeline that recurs once a day,
/// *without a time zone*.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct LocalTime {
    hour:   i8,
    minute: i8,
    second: i8,
    millisecond: i16,
}

/// A **local date-time** is an exact instant on the timeline, *without a
/// time zone*.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct LocalDateTime {
    date: LocalDate,
    time: LocalTime,
}


impl LocalDate {

    /// Creates a new local date instance from the given year, month, and day
    /// fields.
    ///
    /// The values are checked for validity before instantiation, and
    /// passing in values out of range will return an error.
    ///
    /// ### Examples
    ///
    /// Instantiate the 20th of July 1969 based on its year,
    /// week-of-year, and weekday.
    ///
    /// ```rust
    /// use datetime::{LocalDate, Month, DatePiece};
    ///
    /// let date = LocalDate::ymd(1969, Month::July, 20).unwrap();
    /// assert_eq!(date.year(), 1969);
    /// assert_eq!(date.month(), Month::July);
    /// assert_eq!(date.day(), 20);
    ///
    /// assert!(LocalDate::ymd(2100, Month::February, 29).is_err());
    /// ```
    pub fn ymd(year: i64, month: Month, day: i8) -> Result<LocalDate, Error> {
        YMD { year: year, month: month, day: day }
            .to_days_since_epoch()
            .map(|days| LocalDate::from_days_since_epoch(days - EPOCH_DIFFERENCE))
    }

    /// Creates a new local date instance from the given year and day-of-year
    /// values.
    ///
    /// The values are checked for validity before instantiation, and
    /// passing in values out of range will return an error.
    ///
    /// ### Examples
    ///
    /// Instantiate the 13th of September 2015 based on its year
    /// and day-of-year.
    ///
    /// ```rust
    /// use datetime::{LocalDate, Weekday, Month, DatePiece};
    ///
    /// let date = LocalDate::yd(2015, 0x100).unwrap();
    /// assert_eq!(date.year(), 2015);
    /// assert_eq!(date.month(), Month::September);
    /// assert_eq!(date.day(), 13);
    /// ```
    pub fn yd(year: i64, yearday: i64) -> Result<LocalDate, Error> {
        if yearday.is_within(0..367) {
            let jan_1 = YMD { year: year, month: January, day: 1 };
            let days = try!(jan_1.to_days_since_epoch());
            Ok(LocalDate::from_days_since_epoch(days + yearday - 1 - EPOCH_DIFFERENCE))
        }
        else {
            Err(Error::OutOfRange)
        }
    }

    /// Creates a new local date instance from the given year, week-of-year,
    /// and weekday values.
    ///
    /// The values are checked for validity before instantiation, and
    /// passing in values out of range will return an error.
    ///
    /// ### Examples
    ///
    /// Instantiate the 11th of September 2015 based on its year,
    /// week-of-year, and weekday.
    ///
    /// ```rust
    /// use datetime::{LocalDate, Weekday, Month, DatePiece};
    ///
    /// let date = LocalDate::ywd(2015, 37, Weekday::Friday).unwrap();
    /// assert_eq!(date.year(), 2015);
    /// assert_eq!(date.month(), Month::September);
    /// assert_eq!(date.day(), 11);
    /// assert_eq!(date.weekday(), Weekday::Friday);
    /// ```
    ///
    /// Note that according to the ISO-8601 standard, the year will change
    /// when working with dates early in week 1, or late in week 53:
    ///
    /// ```rust
    /// use datetime::{LocalDate, Weekday, Month, DatePiece};
    ///
    /// let date = LocalDate::ywd(2009, 1, Weekday::Monday).unwrap();
    /// assert_eq!(date.year(), 2008);
    /// assert_eq!(date.month(), Month::December);
    /// assert_eq!(date.day(), 29);
    /// assert_eq!(date.weekday(), Weekday::Monday);
    ///
    /// let date = LocalDate::ywd(2009, 53, Weekday::Sunday).unwrap();
    /// assert_eq!(date.year(), 2010);
    /// assert_eq!(date.month(), Month::January);
    /// assert_eq!(date.day(), 3);
    /// assert_eq!(date.weekday(), Weekday::Sunday);
    /// ```
    pub fn ywd(year: i64, week: i64, weekday: Weekday) -> Result<LocalDate, Error> {
        let jan_4 = YMD { year: year, month: January, day: 4 };
        let correction = days_to_weekday(jan_4.to_days_since_epoch().unwrap() - EPOCH_DIFFERENCE).days_from_monday_as_one() as i64 + 3;

        let yearday = 7 * week + weekday.days_from_monday_as_one() as i64 - correction;

        if yearday <= 0 {
            let (_, is_leap_year) = YMD { year: year - 1, month: January, day: 1 }.leap_year_calculations();
            let days_in_year = if is_leap_year { 366 } else { 365 };
            LocalDate::yd(year - 1, days_in_year + yearday)
        }
        else {
            let (_, is_leap_year) = YMD { year: year, month: January, day: 1 }.leap_year_calculations();
            let days_in_year = if is_leap_year { 366 } else { 365 };

            if yearday >= days_in_year {
                LocalDate::yd(year + 1, yearday - days_in_year)
            }
            else {
                LocalDate::yd(year, yearday)
            }
        }
    }

    /// Computes a LocalDate - year, month, day, weekday, and yearday -
    /// given the number of days that have passed since the EPOCH.
    ///
    /// This is used by all the other constructor functions.
    /// ### Examples
    ///
    /// Instantiate the 25th of September 2015 given its day-of-year (268).
    ///
    /// ```rust
    /// use datetime::{LocalDate, Month, DatePiece};
    ///
    /// let date = LocalDate::yd(2015, 268).unwrap();
    /// assert_eq!(date.year(), 2015);
    /// assert_eq!(date.month(), Month::September);
    /// assert_eq!(date.day(), 25);
    /// ```
    ///
    /// Remember that on leap years, the number of days in a year changes:
    ///
    /// ```rust
    /// use datetime::{LocalDate, Month, DatePiece};
    ///
    /// let date = LocalDate::yd(2016, 268).unwrap();
    /// assert_eq!(date.year(), 2016);
    /// assert_eq!(date.month(), Month::September);
    /// assert_eq!(date.day(), 24);  // not the 25th!
    /// ```
    fn from_days_since_epoch(days: i64) -> LocalDate {

        // The Gregorian calendar works in 400-year cycles, which repeat
        // themselves ever after.
        //
        // This calculation works by finding the number of 400-year,
        // 100-year, and 4-year cycles, then constantly subtracting the
        // number of leftover days.
        let (num_400y_cycles, mut remainder) = split_cycles(days, DAYS_IN_400Y);

        // Calculate the numbers of 100-year cycles, 4-year cycles, and
        // leftover years, continually reducing the number of days left to
        // think about.
        let num_100y_cycles = remainder / DAYS_IN_100Y;
        remainder -= num_100y_cycles * DAYS_IN_100Y;  // remainder is now days left in this 100-year cycle

        let num_4y_cycles = remainder / DAYS_IN_4Y;
        remainder -= num_4y_cycles * DAYS_IN_4Y;  // remainder is now days left in this 4-year cycle

        let mut years = remainder / 365;
        remainder -= years * 365;  // remainder is now days left in this year

        // Leap year calculation goes thusly:
        //
        // 1. If the year is a multiple of 400, it's a leap year.
        // 2. Else, if the year is a multiple of 100, it's *not* a leap year.
        // 3. Else, if the year is a multiple of 4, it's a leap year again!
        //
        // We already have the values for the numbers of multiples at this
        // point, and it's safe to re-use them.
        let days_this_year =
            if years == 0 && !(num_4y_cycles == 0 && num_100y_cycles != 0) { 366 }
                                                                      else { 365 };

        // Find out which number day of the year it is.
        // The 306 here refers to the number of days in a year excluding
        // January and February (which are excluded because of the EPOCH)
        let mut day_of_year = remainder + days_this_year - 306;
        if day_of_year >= days_this_year {
            day_of_year -= days_this_year;  // wrap around for January and February
        }

        // Turn all those cycles into an actual number of years.
        years +=   4 * num_4y_cycles
               + 100 * num_100y_cycles
               + 400 * num_400y_cycles;

        // Work out the month and number of days into the month by scanning
        // the time triangle, finding the month that has the correct number
        // of days elapsed at the end of it.
        // (it's "11 - index" below because the triangle goes backwards)
        let result = TIME_TRIANGLE.iter()
                                  .enumerate()
                                  .find(|&(_, days)| *days <= remainder);

        let (mut month, month_days) = match result {
            Some((index, days)) => (11 - index, remainder - *days),
            None => (0, remainder),  // No month found? Then it's February.
        };

        // Need to add 2 to the month in order to compensate for the EPOCH
        // being in March.
        month += 2;

        if month >= 12 {
            years += 1;   // wrap around for January and February
            month -= 12;  // (yes, again)
        }

        // The check immediately above means we can `unwrap` this, as the
        // month number is guaranteed to be in the range (0..12).
        let month_variant = Month::from_zero(month as i8).unwrap();

        // Finally, adjust the day numbers for human reasons: the first day
        // of the month is the 1st, rather than the 0th, and the year needs
        // to be adjusted relative to the EPOCH.
        LocalDate {
            yearday: (day_of_year + 1) as i16,
            weekday: days_to_weekday(days),
            ymd: YMD {
                year:  years + 2000,
                month: month_variant,
                day:   (month_days + 1) as i8,
            },
        }
    }

    /// Creates a new datestamp instance with the given year, month, day,
    /// weekday, and yearday fields.
    ///
    /// This function is unsafe because **the values are not checked for
    /// validity!** It's possible to pass the wrong values in, such as having
    /// a wrong day value for a month, or having the yearday value out of
    /// step. Before using it, check that the values are all correct - or just
    /// use the `date!()` macro, which does this for you at compile-time.
    ///
    /// For this reason, the function is marked as `unsafe`, even though it
    /// (technically) uses unsafe components.
    pub unsafe fn _new_with_prefilled_values(year: i64, month: Month, day: i8, weekday: Weekday, yearday: i16) -> LocalDate {
        LocalDate {
            ymd: YMD { year: year, month: month, day: day },
            weekday: weekday,
            yearday: yearday,
        }
    }

    // I'm not 100% convinced on using `unsafe` for something that doesn't
    // technically *need* to be unsafe, but I'll stick with it for now.

    /// Creates a new local date instance by parsing the strings in the given
    /// set of fields.
    pub fn from_fields(fields: iso8601::Date) -> Result<LocalDate, ParseError> {
        if let iso8601::Date::YMD { year, month, day } = fields {
            let month_variant = try!(Month::from_one(month as i8).map_err(ParseError::Date));
            LocalDate::ymd(year as i64, month_variant, day as i8).map_err(ParseError::Date)
        }
        else if let iso8601::Date::Week { year, ww, d } = fields {
            let weekday_variant = try!(Weekday::from_one(d as i8).map_err(ParseError::Date));
            LocalDate::ywd(year as i64, ww as i64, weekday_variant).map_err(ParseError::Date)
        }
        else if let iso8601::Date::Ordinal { year, ddd } = fields {
            LocalDate::yd(year as i64, ddd as i64).map_err(ParseError::Date)
        }
        else {
            unreachable!()  // should be unnecessary??
        }
    }
}

impl fmt::Debug for LocalDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:04}-{:02}-{:02}", self.year(), self.month().months_from_january(), self.day())
    }
}

impl DatePiece for LocalDate {
    fn year(&self) -> i64 { self.ymd.year }
    fn month(&self) -> Month { self.ymd.month }
    fn day(&self) -> i8 { self.ymd.day }
    fn yearday(&self) -> i16 { self.yearday }
    fn weekday(&self) -> Weekday { self.weekday }
}

impl FromStr for LocalDate {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match iso8601::date(input) {
            Ok(fields)  => LocalDate::from_fields(fields),
            Err(e)      => Err(ParseError::Parse(e)),
        }
    }
}

impl PartialEq for LocalDate {
    fn eq(&self, other: &LocalDate) -> bool {
        self.ymd == other.ymd
    }
}

impl PartialOrd for LocalDate {
    fn partial_cmp(&self, other: &LocalDate) -> Option<Ordering> {
        self.ymd.partial_cmp(&other.ymd)
    }
}

impl Ord for LocalDate {
    fn cmp(&self, other: &LocalDate) -> Ordering {
        self.ymd.cmp(&other.ymd)
    }
}

impl LocalTime {

    /// Computes the number of hours, minutes, and seconds, based on the
    /// number of seconds that have elapsed since midnight.
    pub fn from_seconds_since_midnight(seconds: i64) -> LocalTime {
        LocalTime::from_seconds_and_milliseconds_since_midnight(seconds, 0)
    }

    /// Computes the number of hours, minutes, and seconds, based on the
    /// number of seconds that have elapsed since midnight.
    pub fn from_seconds_and_milliseconds_since_midnight(seconds: i64, millisecond_of_second: i16) -> LocalTime {
        LocalTime {
            hour:   (seconds / 60 / 60) as i8,
            minute: (seconds / 60 % 60) as i8,
            second: (seconds % 60) as i8,
            millisecond: millisecond_of_second,
        }
    }

    /// The time at midnight, with all fields initialised to 0.
    pub fn midnight() -> LocalTime {
        LocalTime { hour: 0, minute: 0, second: 0, millisecond: 0 }
    }

    /// Create a new timestamp instance with the given hour and minute
    /// fields. The second and millisecond fields are set to 0.
    ///
    /// The values are checked for validity before instantiation, and
    /// passing in values out of range will return an `Err`.
    pub fn hm(hour: i8, minute: i8) -> Result<LocalTime, Error> {
        if (hour.is_within(0..24) && minute.is_within(0..60))
        || (hour == 24 && minute == 00) {
            Ok(LocalTime { hour: hour, minute: minute, second: 0, millisecond: 0 })
        }
        else {
            Err(Error::OutOfRange)
        }
    }

    /// Create a new timestamp instance with the given hour, minute, and
    /// second fields. The millisecond field is set to 0.
    ///
    /// The values are checked for validity before instantiation, and
    /// passing in values out of range will return an `Err`.
    pub fn hms(hour: i8, minute: i8, second: i8) -> Result<LocalTime, Error> {
        if (hour.is_within(0..24) && minute.is_within(0..60) && second.is_within(0..60))
        || (hour == 24 && minute == 00 && second == 00) {
            Ok(LocalTime { hour: hour, minute: minute, second: second, millisecond: 0 })
        }
        else {
            Err(Error::OutOfRange)
        }
    }

    /// Create a new timestamp instance with the given hour, minute,
    /// second, and millisecond fields.
    ///
    /// The values are checked for validity before instantiation, and
    /// passing in values out of range will return an `Err`.
    pub fn hms_ms(hour: i8, minute: i8, second: i8, millisecond: i16) -> Result<LocalTime, Error> {
        if hour.is_within(0..24)   && minute.is_within(0..60)
        && second.is_within(0..60) && millisecond.is_within(0..1000)
        {
            Ok(LocalTime { hour: hour, minute: minute, second: second, millisecond: millisecond })
        }
        else {
            Err(Error::OutOfRange)
        }
    }

    /// Calculate the number of seconds since midnight this time is at,
    /// ignoring milliseconds.
    pub fn to_seconds(&self) -> i64 {
        self.hour as i64 * 3600
            + self.minute as i64 * 60
            + self.second as i64
    }

    /// Creates a new local time instance by parsing the strings in the given
    /// set of fields.
    pub fn from_fields(fields: iso8601::Time) -> Result<Self, ParseError> {
        let h  = fields.hour as i8;
        let m  = fields.minute as i8;
        let s  = fields.second as i8;
        let ms = fields.millisecond as i16;

        LocalTime::hms_ms(h, m, s, ms).map_err(ParseError::Date)
    }
}

impl FromStr for LocalTime {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<LocalTime, Self::Err> {
        match iso8601::time(input) {
            Ok(fields)  => LocalTime::from_fields(fields),
            Err(e)      => Err(ParseError::Parse(e)),
        }
    }
}

impl fmt::Debug for LocalTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:01}-{:02}-{:02}", self.hour(), self.minute(), self.second())
    }
}

impl TimePiece for LocalTime {
    fn hour(&self) -> i8 { self.hour }
    fn minute(&self) -> i8 { self.minute }
    fn second(&self) -> i8 { self.second }
    fn millisecond(&self) -> i16 { self.millisecond }
}


impl LocalDateTime {

    /// Computes a complete date-time based on the values in the given
    /// Instant parameter.
    pub fn from_instant(instant: Instant) -> LocalDateTime {
        LocalDateTime::at_ms(instant.seconds(), instant.milliseconds())
    }

    /// Computes a complete date-time based on the number of seconds that
    /// have elapsed since **midnight, 1st January, 1970**, setting the
    /// number of milliseconds to 0.
    pub fn at(seconds_since_1970_epoch: i64) -> LocalDateTime {
        LocalDateTime::at_ms(seconds_since_1970_epoch, 0)
    }

    /// Computes a complete date-time based on the number of seconds that
    /// have elapsed since **midnight, 1st January, 1970**,
    pub fn at_ms(seconds_since_1970_epoch: i64, millisecond_of_second: i16) -> LocalDateTime {
        let seconds = seconds_since_1970_epoch - EPOCH_DIFFERENCE * SECONDS_IN_DAY;

        // Just split the input value into days and seconds, and let
        // LocalDate and LocalTime do all the hard work.
        let (days, secs) = split_cycles(seconds, SECONDS_IN_DAY);

        LocalDateTime {
            date: LocalDate::from_days_since_epoch(days),
            time: LocalTime::from_seconds_and_milliseconds_since_midnight(secs, millisecond_of_second),
        }
    }

    /// Creates a new local date time from a local date and a local time.
    pub fn new(date: LocalDate, time: LocalTime) -> LocalDateTime {
        LocalDateTime {
            date: date,
            time: time,
        }
    }

    /// The date portion of this date-time stamp.
    pub fn date(&self) -> LocalDate {
        self.date
    }

    /// The time portion of this date-time stamp.
    pub fn time(&self) -> LocalTime {
        self.time
    }

    /// Creates a new date-time stamp set to the current time.
    pub fn now() -> LocalDateTime {
        let (s, ms) = unsafe { now::now() };
        LocalDateTime::at_ms(s, ms)
    }

    pub fn to_instant(&self) -> Instant {
        let seconds = self.date.ymd.to_days_since_epoch().unwrap() * SECONDS_IN_DAY + self.time.to_seconds();
        Instant::at_ms(seconds, self.time.millisecond)
    }

    pub fn add_seconds(&self, seconds: i64) -> LocalDateTime {
        Self::from_instant(self.to_instant() + Duration::of(seconds))
    }
}

impl FromStr for LocalDateTime {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<LocalDateTime, Self::Err> {
        let fields = match iso8601::datetime(input) {
            Ok(fields)  => fields,
            Err(e)      => return Err(ParseError::Parse(e)),
        };

        let date = try!(LocalDate::from_fields(fields.date));
        let time = try!(LocalTime::from_fields(fields.time));
        Ok(LocalDateTime { date: date, time: time })
    }
}

impl fmt::Debug for LocalDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}T{:?}", self.date, self.time)
    }
}

impl DatePiece for LocalDateTime {
    fn year(&self) -> i64 { self.date.ymd.year }
    fn month(&self) -> Month { self.date.ymd.month }
    fn day(&self) -> i8 { self.date.ymd.day }
    fn yearday(&self) -> i16 { self.date.yearday }
    fn weekday(&self) -> Weekday { self.date.weekday }
}

impl TimePiece for LocalDateTime {
    fn hour(&self) -> i8 { self.time.hour }
    fn minute(&self) -> i8 { self.time.minute }
    fn second(&self) -> i8 { self.time.second }
    fn millisecond(&self) -> i16 { self.time.millisecond }
}

impl Add<Duration> for LocalDateTime {
    type Output = LocalDateTime;

    fn add(self, duration: Duration) -> LocalDateTime {
        LocalDateTime::from_instant(self.to_instant() + duration)
    }
}

impl Sub<Duration> for LocalDateTime {
    type Output = LocalDateTime;

    fn sub(self, duration: Duration) -> LocalDateTime {
        LocalDateTime::from_instant(self.to_instant() - duration)
    }
}


/// A **YMD** is an implementation detail of LocalDate. It provides
/// helper methods relating to the construction of LocalDate instances.
///
/// The main difference is that while all LocalDates get checked for
/// validity before they are used, there is no such check for YMD. The
/// interface to LocalDate ensures that it should be impossible to
/// create an instance of the 74th of March, for example, but you're
/// free to create such an instance of YMD. For this reason, it is not
/// exposed to implementors of this library.
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Debug, Copy)]
pub struct YMD {
    pub year:    i64,
    pub month:   Month,
    pub day:     i8,
}

impl YMD {

    /// Calculates the number of days that have elapsed since the 1st
    /// January, 1970. Returns the number of days if this datestamp is
    /// valid; None otherwise.
    ///
    /// This method returns a Result instead of exposing is_valid to
    /// the user, because the leap year calculations are used in both
    /// functions, so it makes more sense to only do them once.
    pub fn to_days_since_epoch(&self) -> Result<i64, Error> {
        let years = self.year - 2000;
        let (leap_days_elapsed, is_leap_year) = self.leap_year_calculations();

        if !self.is_valid(is_leap_year) {
            return Err(Error::OutOfRange);
        }

        // Work out the number of days from the start of 1970 to now,
        // which is a multiple of the number of years...
        let days = years * 365

            // Plus the number of days between the start of 2000 and the
            // start of 1970, to make up the difference because our
            // dates start at 2000 and instants start at 1970...
            + 10958

            // Plus the number of leap years that have elapsed between
            // now and the start of 2000...
            + leap_days_elapsed

            // Plus the number of days in all the months leading up to
            // the current month...
            + self.month.days_before_start() as i64

            // Plus an extra leap day for *this* year...
            + if is_leap_year && self.month >= March { 1 } else { 0 }

            // Plus the number of days in the month so far! (Days are
            // 1-indexed, so we make them 0-indexed here)
            + (self.day - 1) as i64;

        Ok(days)
    }

    /// Returns whether this datestamp is valid, which basically means
    /// whether the day is in the range allowed by the month.
    ///
    /// Whether the current year is a leap year should already have been
    /// calculated at this point, so the value is passed in rather than
    /// calculating it afresh.
    pub fn is_valid(&self, is_leap_year: bool) -> bool {
        self.day >= 1 && self.day <= self.month.days_in_month(is_leap_year)
    }

    /// Performs two related calculations for leap years, returning the
    /// results as a two-part tuple:
    ///
    /// 1. The number of leap years that have elapsed prior to this date;
    /// 2. Whether the current year is a leap year or not.
    pub fn leap_year_calculations(&self) -> (i64, bool) {
        let year = self.year - 2000;

        // This calculation is the reverse of LocalDate::from_days_since_epoch.
        let (num_400y_cycles, mut remainder) = split_cycles(year, 400);

        // Standard leap-year calculations, performed on the remainder
        let currently_leap_year = remainder == 0 || (remainder % 100 != 0 && remainder % 4 == 0);

        let num_100y_cycles = remainder / 100;
        remainder -= num_100y_cycles * 100;

        let leap_years_elapsed = remainder / 4
            + 97 * num_400y_cycles  // There are 97 leap years in 400 years
            + 24 * num_100y_cycles  // There are 24 leap years in 100 years
            - if currently_leap_year { 1 } else { 0 };

        (leap_years_elapsed, currently_leap_year)
    }
}

/// Computes the weekday, given the number of days that have passed
/// since the EPOCH.
fn days_to_weekday(days: i64) -> Weekday {
    // March 1st, 2000 was a Wednesday, so add 3 to the number of days.
    let weekday = (days + 3) % 7;

    // We can unwrap since we've already done the bounds checking.
    Weekday::from_zero(if weekday < 0 { weekday + 7 } else { weekday } as i8).unwrap()
}

/// Split a number of years into a number of year-cycles, and the number
/// of years left over that don't fit into a cycle. This is also used
/// for day-cycles.
///
/// This is essentially a division operation with the result and the
/// remainder, with the difference that a negative value gets 'wrapped
/// around' to be a positive value, owing to the way the modulo operator
/// works for negative values.
fn split_cycles(number_of_periods: i64, cycle_length: i64) -> (i64, i64) {
    let mut cycles    = number_of_periods / cycle_length;
    let mut remainder = number_of_periods % cycle_length;

    if remainder < 0 {
        remainder += cycle_length;
        cycles    -= 1;
    }

    (cycles, remainder)
}


#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Error {
    OutOfRange,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        "datetime field out of range"
    }
}


#[derive(PartialEq, Debug, Clone)]
pub enum ParseError {
    Date(Error),
    Parse(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            ParseError::Date(ref error)   => write!(f, "{}: {}", self.description(), error),
            ParseError::Parse(ref string) => write!(f, "{}: {}", self.description(), string),
        }
    }
}

impl ErrorTrait for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::Date(_)     => "parsing resulted in an invalid date",
            ParseError::Parse(_)    => "parse error",
        }
    }

    fn cause(&self) -> Option<&ErrorTrait> {
        match *self {
            ParseError::Date(ref error)   => Some(error),
            ParseError::Parse(_)          => None,
        }
    }
}

/// A month of the year, starting with January, and ending with December.
///
/// This is stored as an enum instead of just a number to prevent
/// off-by-one errors: is month 2 February (1-indexed) or March (0-indexed)?
/// In this case, it's 1-indexed, to have January become 1 when you use
/// `as i32` in code.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum Month {
    January =  1, February =  2, March     =  3,
    April   =  4, May      =  5, June      =  6,
    July    =  7, August   =  8, September =  9,
    October = 10, November = 11, December  = 12,
}

impl Month {

    /// The number of days in this month, depending on whether it's a
    /// leap year or not.
    pub fn days_in_month(&self, leap_year: bool) -> i8 {
        match *self {
            January   => 31, February  => if leap_year { 29 } else { 28 },
            March     => 31, April     => 30,
            May       => 31, June      => 30,
            July      => 31, August    => 31,
            September => 30, October   => 31,
            November  => 30, December  => 31,
        }
    }

    /// The number of days that have elapsed in a year *before* this
    /// month begins, with no leap year check.
    fn days_before_start(&self) -> i16 {
        match *self {
            January =>   0, February =>  31, March     =>  59,
            April   =>  90, May      => 120, June      => 151,
            July    => 181, August   => 212, September => 243,
            October => 273, November => 304, December  => 334,
        }
    }

    pub fn months_from_january(&self) -> usize {
        match *self {
            January =>   0, February =>   1, March     =>  2,
            April   =>   3, May      =>   4, June      =>  5,
            July    =>   6, August   =>   7, September =>  8,
            October =>   9, November =>  10, December  => 11,
        }
    }

    /// Return the month based on a number, with January as Month 1, February
    /// as Month 2, and so on.
    ///
    /// ```rust
    /// use datetime::Month;
    /// assert_eq!(Month::from_one(5), Ok(Month::May));
    /// assert!(Month::from_one(0).is_err());
    /// ```
    pub fn from_one(month: i8) -> Result<Month, Error> {
        Ok(match month {
             1 => January,   2 => February,   3 => March,
             4 => April,     5 => May,        6 => June,
             7 => July,      8 => August,     9 => September,
            10 => October,  11 => November,  12 => December,
             _ => return Err(Error::OutOfRange),
        })
    }

    /// Return the month based on a number, with January as Month 0, February
    /// as Month 1, and so on.
    ///
    /// ```rust
    /// use datetime::Month;
    /// assert_eq!(Month::from_zero(5), Ok(Month::June));
    /// assert!(Month::from_zero(12).is_err());
    /// ```
    pub fn from_zero(month: i8) -> Result<Month, Error> {
        Ok(match month {
            0 => January,   1 => February,   2 => March,
            3 => April,     4 => May,        5 => June,
            6 => July,      7 => August,     8 => September,
            9 => October,  10 => November,  11 => December,
            _ => return Err(Error::OutOfRange),
        })
    }
}


/// A named day of the week, starting with Sunday, and ending with Saturday.
///
/// Sunday is Day 0. This seems to be a North American thing? It's pretty
/// much an arbitrary choice, and as you can't use the from_zero method,
/// it won't affect you at all. If you want to change it, the only thing
/// that should be affected is LocalDate::days_to_weekday.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Weekday {
    Sunday, Monday, Tuesday, Wednesday, Thursday, Friday, Saturday,
}

// I'm not going to give weekdays an Ord instance because there's no
// real standard as to whether Sunday should come before Monday, or the
// other way around. Luckily, they don't need one, as the field is
// ignored when comparing LocalDates.

impl Weekday {
    fn days_from_monday_as_one(&self) -> i8 {
        match *self {
            Sunday => 7,   Monday => 1,
            Tuesday => 2,  Wednesday => 3,
            Thursday => 4, Friday => 5,
            Saturday => 6,
        }
    }

    /// Return the weekday based on a number, with Sunday as Day 0, Monday as
    /// Day 1, and so on.
    ///
    /// ```rust
    /// use datetime::Weekday;
    /// assert_eq!(Weekday::from_zero(4), Ok(Weekday::Thursday));
    /// assert!(Weekday::from_zero(7).is_err());
    /// ```
    pub fn from_zero(weekday: i8) -> Result<Weekday, Error> {
        Ok(match weekday {
            0 => Sunday,     1 => Monday,    2 => Tuesday,
            3 => Wednesday,  4 => Thursday,  5 => Friday,
            6 => Saturday,   _ => return Err(Error::OutOfRange),
        })
    }

    pub fn from_one(weekday: i8) -> Result<Weekday, Error> {
        Ok(match weekday {
            7 => Sunday,     1 => Monday,    2 => Tuesday,
            3 => Wednesday,  4 => Thursday,  5 => Friday,
            6 => Saturday,   _ => return Err(Error::OutOfRange),
        })
    }
}

#[cfg(test)]
mod test {
    pub use super::{LocalDateTime, LocalDate, LocalTime, Month, Weekday};
    pub use cal::iter::Year;
    pub use cal::DatePiece;
    pub use std::str::FromStr;
    use super::YMD;

    mod seconds_to_datetimes {
        pub use super::*;
        use super::super::YMD;

        #[test]
        fn before_time() {
            let date = LocalDateTime::at(-1_000_000_000_i64);
            let res = LocalDateTime {
                date: LocalDate {
                    ymd: YMD { year: 1938, month: Month::April, day: 24, },
                    weekday: Weekday::Sunday, yearday: 114,
                },
                time: LocalTime {
                    hour: 22, minute: 13, second: 20, millisecond: 0,
                },
            };

            assert_eq!(date, res)
        }

        #[test]
        fn start_of_magic() {
            let date = LocalDateTime::at(0_i64);
            let res = LocalDateTime {
                date: LocalDate {
                    ymd: YMD { year: 1970, month: Month::January, day: 1, },
                    weekday: Weekday::Thursday, yearday: 1,
                },
                time: LocalTime::midnight(),
            };

            assert_eq!(date, res)
        }

        #[test]
        fn billennium() {
            let date = LocalDateTime::at(1_000_000_000_i64);
            let res = LocalDateTime {
                date: LocalDate {
                    ymd: YMD { year: 2001, month: Month::September, day: 9, },
                    weekday: Weekday::Sunday, yearday: 252,
                },
                time: LocalTime {
                    hour: 1, minute: 46, second: 40, millisecond: 0,
                },
            };

            assert_eq!(date, res)
        }

        #[test]
        fn numbers() {
            let date = LocalDateTime::at(1_234_567_890_i64);
            let res = LocalDateTime {
                date: LocalDate {
                    ymd: YMD { year: 2009, month: Month::February, day: 13, },
                    weekday: Weekday::Friday, yearday: 44,
                },
                time: LocalTime {
                    hour: 23, minute: 31, second: 30, millisecond: 0,
                },
            };

            assert_eq!(date, res)
        }

        #[test]
        fn year_2038_problem() {
            let date = LocalDateTime::at(0x7FFF_FFFF_i64);
            let res = LocalDateTime {
                date: LocalDate {
                    ymd: YMD { year: 2038, month: Month::January, day: 19, },
                    weekday: Weekday::Tuesday, yearday: 19,
                },
                time: LocalTime {
                    hour: 3, minute: 14, second: 7, millisecond: 0,
                },
            };

            assert_eq!(date, res)
        }

        #[test]
        fn the_end_of_time() {
            let date = LocalDateTime::at(0x7FFF_FFFF_FFFF_FFFF_i64);
            let res = LocalDateTime {
                date: LocalDate {
                    ymd: YMD { year: 292_277_026_596, month: Month::December, day: 4, },
                    weekday: Weekday::Sunday, yearday: 339,
                },
                time: LocalTime {
                    hour: 15, minute: 30, second: 7, millisecond: 0,
                },
            };

            assert_eq!(date, res)
        }

        #[test]
        fn just_another_date() {
            let date = LocalDateTime::at(146096 * 86400);
            let res = LocalDateTime {
                date: LocalDate {
                    ymd: YMD { year: 2369, month: Month::December, day: 31, },
                    weekday: Weekday::Wednesday, yearday: 365,
                },
                time: LocalTime::midnight(),
            };

            assert_eq!(date, res)
        }
    }

    mod ymd_to_datetimes {
        use super::*;

        #[test]
        fn the_distant_past() {
            let date = LocalDate::ymd(7, Month::April, 1).unwrap();
            assert_eq!(7, date.year());
            assert_eq!(Month::April, date.month());
            assert_eq!(1, date.day());
        }

        #[test]
        fn the_distant_present() {
            let date = LocalDate::ymd(2015, Month::January, 16).unwrap();
            assert_eq!(2015, date.year());
            assert_eq!(Month::January, date.month());
            assert_eq!(16, date.day());
        }

        #[test]
        fn the_distant_future() {
            let date = LocalDate::ymd(1048576, Month::October, 13).unwrap();
            assert_eq!(1048576, date.year());
            assert_eq!(Month::October, date.month());
            assert_eq!(13, date.day());
        }
    }

    #[test]
    fn start_of_year_day() {
        let date = LocalDate::ymd(2015, Month::January, 1).unwrap();
        assert_eq!(date.yearday(), 1);
    }

    #[test]
    fn end_of_year_day() {
        let date = LocalDate::ymd(2015, Month::December, 31).unwrap();
        assert_eq!(date.yearday(), 365);
    }

    #[test]
    fn end_of_leap_year_day() {
        let date = LocalDate::ymd(2016, Month::December, 31).unwrap();
        assert_eq!(date.yearday(), 366);
    }

    #[test]
    fn day_start_of_year() {
        let date = LocalDate::yd(2015, 1).unwrap();
        assert_eq!(2015, date.year());
        assert_eq!(Month::January, date.month());
        assert_eq!(1, date.day());
    }


    #[test]
    fn leap_year_1600() {
        let date = YMD { year: 1600, month: Month::January, day: 1 };
        assert!(date.leap_year_calculations().1 == true)
    }

    #[test]
    fn leap_year_1900() {
        let date = YMD { year: 1900, month: Month::January, day: 1 };
        assert!(date.leap_year_calculations().1 == false)
    }

    #[test]
    fn some_leap_years() {
        for year in [2004,2008,2012,2016].iter() {
            assert!(LocalDate::ymd(*year, Month::February, 29).is_ok());
            assert!(LocalDate::ymd(*year + 1, Month::February, 29).is_err());
        }
        assert!(LocalDate::ymd(1600,Month::February,29).is_ok());
        assert!(LocalDate::ymd(1601,Month::February,29).is_err());
        assert!(LocalDate::ymd(1602,Month::February,29).is_err());
    }


    #[test]
    fn parse_iso_ymd() {
        let date_option = LocalDate::from_str("2015-06-26");
        assert!(date_option.is_ok());
        let date = date_option.unwrap();
        assert!(date.year() == 2015);
        assert!(date.month() == Month::June);
        assert!(date.day() == 26);
    }

    #[test]
    fn new() {
        for year in 1..3000 {
            assert!(LocalDate::ymd(year, Month::from_one( 1).unwrap(), 32).is_err()); assert!(LocalDate::ymd(year, Month::from_one( 2).unwrap(), 30).is_err()); assert!(LocalDate::ymd(year, Month::from_one( 3).unwrap(), 32).is_err());
            assert!(LocalDate::ymd(year, Month::from_one( 4).unwrap(), 31).is_err()); assert!(LocalDate::ymd(year, Month::from_one( 5).unwrap(), 32).is_err()); assert!(LocalDate::ymd(year, Month::from_one( 6).unwrap(), 31).is_err());
            assert!(LocalDate::ymd(year, Month::from_one( 7).unwrap(), 32).is_err()); assert!(LocalDate::ymd(year, Month::from_one( 8).unwrap(), 32).is_err()); assert!(LocalDate::ymd(year, Month::from_one( 9).unwrap(), 31).is_err());
            assert!(LocalDate::ymd(year, Month::from_one(10).unwrap(), 32).is_err()); assert!(LocalDate::ymd(year, Month::from_one(11).unwrap(), 31).is_err()); assert!(LocalDate::ymd(year, Month::from_one(12).unwrap(), 32).is_err());
        }
    }

    #[test]
    fn to_from_days_since_epoch() {
        let epoch_difference: i64 = 30 * 365 + 7 + 31 + 29;  // see EPOCH_DIFFERENCE
        for date in  vec![
            LocalDate::ymd(1970, Month::from_one(01).unwrap(), 01).unwrap(),
            LocalDate::ymd(  01, Month::from_one(01).unwrap(), 01).unwrap(),
            LocalDate::ymd(1971, Month::from_one(01).unwrap(), 01).unwrap(),
            LocalDate::ymd(1973, Month::from_one(01).unwrap(), 01).unwrap(),
            LocalDate::ymd(1977, Month::from_one(01).unwrap(), 01).unwrap(),
            LocalDate::ymd(1989, Month::from_one(11).unwrap(), 10).unwrap(),
            LocalDate::ymd(1990, Month::from_one( 7).unwrap(),  8).unwrap(),
            LocalDate::ymd(2014, Month::from_one( 7).unwrap(), 13).unwrap(),
            LocalDate::ymd(2001, Month::from_one( 2).unwrap(), 03).unwrap()
        ]{
            assert_eq!( date,
                LocalDate::from_days_since_epoch(
                    date.ymd.to_days_since_epoch().unwrap() - epoch_difference));
        }
    }

    #[test]
    fn from_yearday() {
        for date in vec![
            //LocalDate::ymd(1970, 01 , 01).unwrap(),
            LocalDate::ymd(1971, Month::from_one(01).unwrap(), 01).unwrap(),
            LocalDate::ymd(1973, Month::from_one(01).unwrap(), 01).unwrap(),
            LocalDate::ymd(1977, Month::from_one(01).unwrap(), 01).unwrap(),
            LocalDate::ymd(1989, Month::from_one(11).unwrap(), 10).unwrap(),
            LocalDate::ymd(1990, Month::from_one( 7).unwrap(),  8).unwrap(),
            LocalDate::ymd(2014, Month::from_one( 7).unwrap(), 13).unwrap(),
            LocalDate::ymd(2001, Month::from_one( 2).unwrap(), 03).unwrap(),
        ]{
            let new_date = LocalDate::yd(date.year(), date.yearday() as i64).unwrap();
            assert_eq!(new_date, date);
            assert!(LocalDate::yd(2002, 1).is_ok());

            assert_eq!(new_date.yearday(), date.yearday());
        }
    }

    #[test]
    fn yearday() {
        for year in 1..2058 {
            assert_eq!( LocalDate::ymd(year, Month::from_one(01).unwrap(), 31).unwrap().yearday() + 1,
                        LocalDate::ymd(year, Month::from_one(02).unwrap(), 01).unwrap().yearday());
            assert_eq!( LocalDate::ymd(year, Month::from_one(03).unwrap(), 31).unwrap().yearday() + 1,
                        LocalDate::ymd(year, Month::from_one(04).unwrap(), 01).unwrap().yearday());
            assert_eq!( LocalDate::ymd(year, Month::from_one(04).unwrap(), 30).unwrap().yearday() + 1,
                        LocalDate::ymd(year, Month::from_one(05).unwrap(), 01).unwrap().yearday());
            assert!(    LocalDate::ymd(year, Month::from_one(12).unwrap(), 31).unwrap().yearday() > 0);
        }
        assert_eq!( LocalDate::ymd(1600, Month::from_one(02).unwrap(), 29).unwrap().yearday() + 1, // leap year
                    LocalDate::ymd(1600, Month::from_one(03).unwrap(), 01).unwrap().yearday());
        assert_eq!( LocalDate::ymd(1601, Month::from_one(02).unwrap(), 28).unwrap().yearday() + 1, // no leap year
                    LocalDate::ymd(1601, Month::from_one(03).unwrap(), 01).unwrap().yearday());

    }

    #[test]
    fn parse_month() {
        assert_eq!( LocalDate::from_str("2015-01-26").unwrap().month(), Month::January);
        assert_eq!( LocalDate::from_str("1970-01-26").unwrap().month(), Month::January);
        assert_eq!( LocalDate::from_str("1969-01-26").unwrap().month(), Month::January);
    }

    #[test]
    fn leap_year_2000() {
        let date = YMD { year: 2000, month: Month::January, day: 1 };
        assert!(date.leap_year_calculations().1 == true)
    }

    mod datetimes_to_seconds {
        pub use super::*;

        #[test]
        fn test_1970() {
            let date = LocalDateTime::at(0);
            let res = date.to_instant().seconds();

            assert_eq!(res, 0)
        }

        #[test]
        fn test_1971() {
            let date = LocalDateTime::at(86400);
            let res = date.to_instant().seconds();

            assert_eq!(res, 86400)
        }

        #[test]
        fn test_1972() {
            let date = LocalDateTime::at(86400 * 365 * 2);
            let res = date.to_instant().seconds();

            assert_eq!(0, 86400 * 365 * 2 - res)
        }

        #[test]
        fn test_1973() {
            let date = LocalDateTime::at(86400 * (365 * 3 + 1));
            let res = date.to_instant().seconds();

            assert_eq!(0, 86400 * (365 * 3 + 1) - res)
        }

        #[test]
        fn some_date() {
            let date = LocalDateTime::at(1234567890);
            let res = date.to_instant().seconds();

            assert_eq!(1234567890, res)
        }

        #[test]
        fn far_far_future() {
            let date = LocalDateTime::at(54321234567890);
            let res = date.to_instant().seconds();

            assert_eq!(54321234567890, res)
        }

        #[test]
        fn the_distant_past() {
            let date = LocalDateTime::at(-54321234567890);
            let res = date.to_instant().seconds();

            assert_eq!(-54321234567890, res)
        }
    }

    mod arithmetic {
        use super::*;
        use duration::Duration;

        #[test]
        fn addition() {
            let date = LocalDateTime::at(10000);
            assert_eq!(LocalDateTime::at(10001), date + Duration::of(1))
        }

        #[test]
        fn subtraction() {
            let date = LocalDateTime::at(100000000);
            assert_eq!(LocalDateTime::at(99999999), date - Duration::of(1))
        }
    }

    mod spans {
        use super::*;

        #[test]
        fn iterator() {
            let year = Year(2016);
            let mut days = year.days_for_month(Month::February);

            for i in 1..30 {
                assert_eq!(days.next().unwrap(), LocalDate::ymd(2016, Month::February, i).unwrap());
            }

            assert!(days.next().is_none());
        }

        #[test]
        fn iterator_back() {
            let year = Year(2014);
            let mut days = year.days_for_month(Month::February).rev();

            for i in (1..29).rev() {
                assert_eq!(days.next().unwrap(), LocalDate::ymd(2014, Month::February, i).unwrap());
            }

            assert!(days.next().is_none());
        }

        #[test]
        fn double() {
            let year = Year(2012);

            let mut days = year.days_for_month(Month::February);
            assert_eq!(days.next().unwrap(), LocalDate::ymd(2012, Month::February, 1).unwrap());
            assert_eq!(days.next_back().unwrap(), LocalDate::ymd(2012, Month::February, 29).unwrap());
        }
    }
}
