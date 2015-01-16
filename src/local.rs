#[plugin]
extern crate regex_macros;

use std::ops::{Add, Sub};
use std::num::FromPrimitive;

use now;
use parse;
use instant::Instant;
use duration::Duration;

use self::Month::*;

// ---- definitions ----

/// Number of days guaranteed to be in four years.
const DAYS_IN_4Y:   i64 = 365 *   4 +  1;

/// Number of days guaranteed to be in a hundred years.
const DAYS_IN_100Y: i64 = 365 * 100 + 24;

/// Number of days guaranteed to be in four hundred years.
const DAYS_IN_400Y: i64 = 365 * 400 + 97;

/// Number of seconds in a day. As everywhere in this library, leap seconds
/// are simply ignored.
const SECONDS_IN_DAY: i64 = 86400;

/// Number of seconds between **midnight, 1st March, 2000**, and
/// **midnight, 1st January, 1970**.
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
const EPOCH: i64 = SECONDS_IN_DAY * (30 * 365      // 30 years between 2000 and 1970...
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

/// A month of the year, starting with January, and ending with December.
///
/// This is stored as an enum instead of just a number to prevent
/// off-by-one errors: is month 2 February (1-indexed) or March (0-indexed)?
#[derive(FromPrimitive, PartialEq, Eq, PartialOrd, Ord, Show, Clone)]
pub enum Month {
    January, February, March, April, May, June, July,
    August, September, October, November, December,
}

impl Copy for Month { }

/// A named day of the week, starting with Sunday, and ending with Saturday.
///
/// Sunday is day 0. This seems to be a North American thing? It's pretty
/// much an arbitrary choice, and if you don't use the FromPrimitive trait,
/// it won't affect you at all. If you want to change it, the only thing
/// that should be affected is LocalDate::days_to_weekday.
#[derive(FromPrimitive, PartialEq, Eq, Show, Clone)]
pub enum Weekday {
    Sunday, Monday, Tuesday, Wednesday, Thursday, Friday, Saturday,
}

// I'm not going to give weekdays an Ord instance because there's no
// real standard as to whether Sunday should come before Monday, or the
// other way around. Luckily, they don't need one, as the field is
// ignored when comparing LocalDates.

impl Copy for Weekday { }

/// A **local date-time** is an exact instant on the timeline, *without a
/// time zone*.
#[derive(PartialEq, Show, Clone)]
pub struct LocalDateTime {
    date: LocalDate,
    time: LocalTime,
}

impl Copy for LocalDateTime { }

/// A **local date** is a day-long span on the timeline, *without a time
/// zone*.
#[derive(Eq, Show, Clone)]
pub struct LocalDate {
    ymd:     YMD,
    yearday: i16,
    weekday: Weekday,
}

impl Copy for LocalDate { }

/// A **local time** is a time on the timeline that recurs once a day,
/// *without a time zone*.
#[derive(PartialEq, Show, Clone)]
pub struct LocalTime {
    hour:   i8,
    minute: i8,
    second: i8,
    millisecond: i16,
}

impl Copy for LocalTime { }

// ---- implementations ----

/// A **YMD** is an implementation detail of LocalDate. It provides
/// helper methods relating to the construction of LocalDate instances.
///
/// The main difference is that while all LocalDates get checked for
/// validity before they are used, there is no such check for YMD. The
/// interface to LocalDate ensures that it should be impossible to
/// create an instance of the 74th of March, for example, but you're
/// free to create such an instance of YMD. For this reason, it is not
/// exposed to implementors of this library.
#[derive(PartialEq, Eq, Clone, Show)]
struct YMD {
    year:    i64,
    month:   Month,
    day:     i8,
}

impl Copy for YMD { }

impl YMD {
    /// Calculates the number of days that have elapsed since the 1st
    /// January, 1970. Returns the number of days if this datestamp is
    /// valid; None otherwise.
    ///
    /// This method returns an Option instead of exposing is_valid to
    /// the user, because the leap year calculations are used in both
    /// functions, so it makes more sense to only do them once.
    fn to_days_since_epoch(&self) -> Option<i64> {
        let years = self.year - 2000;
        let (leap_days_elapsed, is_leap_year) = self.leap_year_calculations();

        if !self.is_valid(is_leap_year) {
            return None;
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

        Some(days)
    }

    /// Returns whether this datestamp is valid, which basically means
    /// whether the day is in the range allowed by the month.
    ///
    /// Whether the current year is a leap year should already have been
    /// calculated at this point, so the value is passed in rather than
    /// calculating it afresh.
    fn is_valid(&self, is_leap_year: bool) -> bool {
        self.day >= 1 && self.day <= self.month.days_in_month(is_leap_year)
    }

    /// Performs two related calculations for leap years, returning the
    /// results as a two-part tuple:
    ///
    /// 1. The number of leap years that have elapsed prior to this date;
    /// 2. Whether the current year is a leap year or not.
    fn leap_year_calculations(&self) -> (i64, bool) {
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
    FromPrimitive::from_i64(if weekday < 0 { weekday + 7 } else { weekday }).unwrap()
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

impl LocalDate {

    /// Computes a LocalDate - year, month, day, weekday, and yearday -
    /// given the number of days that have passed since the EPOCH.
    ///
    /// This is used by LocalDateTime::at() below.
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

        // Finally, adjust the day numbers for human reasons: the first day
        // of the month is the 1st, rather than the 0th, and the year needs
        // to be adjusted relative to the EPOCH.
        LocalDate {
            yearday: (day_of_year + 1) as i16,
            weekday: days_to_weekday(days),
            ymd: YMD {
                year:  years + 2000,
                month: FromPrimitive::from_uint(month).unwrap(),
                day:   (month_days + 1) as i8,
            },
        }
    }

    /// Creates a new datestamp instance with the given year, month, and
    /// day fields.
    ///
    /// The values are checked for validity before instantiation, and
    /// passing in values out of range will return None.
    pub fn ymd(year: i64, month: Month, day: i8) -> Option<LocalDate> {
        YMD { year: year, month: month, day: day }
            .to_days_since_epoch()
            .map(|days| LocalDate::from_days_since_epoch(days))
    }

    /// Parse an input string matching the ISO-8601 format, returning
    /// the constructed date if successful, and None if unsuccessful.
    pub fn parse(input: &str) -> Option<LocalDate> {
        parse::parse_iso_ymd(input).map(|(y, m, d)| {
            let month = FromPrimitive::from_i8(m).unwrap();
            LocalDate::ymd(y, month, d)
        }).unwrap()
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

    /// Create a new timestamp instance with the given hour, minute, and
    /// second fields. The millisecond field is set to 0.
    ///
    /// The values are checked for validity before instantiation, and
    /// passing in values out of range will return None.
    pub fn hms(hour: i8, minute: i8, second: i8) -> Option<LocalTime> {
        if hour >= 0 && hour <= 23
            && minute >= 0 && minute <= 59
            && second >= 0 && second <= 59
        {
            Some(LocalTime { hour: hour, minute: minute, second: second, millisecond: 0 })
        }
        else {
            None
        }
    }

    /// Create a new timestamp instance with the given hour, minute,
    /// second, and millisecond fields.
    ///
    /// The values are checked for validity before instantiation, and
    /// passing in values out of range will return None.
    pub fn hms_ms(hour: i8, minute: i8, second: i8, millisecond: i16) -> Option<LocalTime> {
        if hour >= 0 && hour <= 23
            && minute >= 0 && minute <= 59
            && second >= 0 && second <= 59
            && millisecond >= 0 && millisecond <= 999
        {
            Some(LocalTime { hour: hour, minute: minute, second: second, millisecond: millisecond })
        }
        else {
            None
        }
    }

    /// Calculate the number of seconds since midnight this time is at,
    /// ignoring milliseconds.
    fn to_seconds(&self) -> i64 {
        self.hour as i64 * 3600
            + self.minute as i64 * 60
            + self.second as i64
    }
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
        let seconds = seconds_since_1970_epoch - EPOCH;

        // Just split the input value into days and seconds, and let
        // LocalDate and LocalTime do all the hard work.
        let (days, secs) = split_cycles(seconds, SECONDS_IN_DAY);

        LocalDateTime {
            date: LocalDate::from_days_since_epoch(days),
            time: LocalTime::from_seconds_and_milliseconds_since_midnight(secs, millisecond_of_second),
        }
    }

    /// Computes a complete date-time based on the number of seconds
    /// *and milliseconds* that have elapsed since **midnight, 1st
    /// January, 1970**.


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
}

impl Month {
    /// The number of days in this month, depending on whether it's a
    /// leap year or not.
    fn days_in_month(&self, leap_year: bool) -> i8 {
        match *self {
            January   => 31, February  => if leap_year { 28 } else { 29 },
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
}

// ---- accessors ----

/// The **date piece** trait is used for date and time values that have
/// date components of years, months, and days.
pub trait DatePiece {

    /// The year, in absolute terms.
    /// This is in human-readable format, so the year 2014 actually has a
    /// year value of 2014, rather than 14 or 114 or anything like that.
    fn year(&self) -> i64;

    /// The month of the year.
    fn month(&self) -> Month;

    /// The day of the month, from 1 to 31.
    fn day(&self) -> i8;

    /// The day of the year, from 1 to 366.
    fn yearday(&self) -> i16;

    /// The day of the week.
    fn weekday(&self) -> Weekday;

    /// The number of years into the century.
    /// This is the same as the last two digits of the year.
    fn year_of_century(&self) -> i64 { self.year() % 100 }

    /// The year number, relative to the year 2000.
    /// Internally, many routines use years relative the year 2000,
    /// rather than the year 0 (well, 1 BCE).
    fn years_from_2000(&self) -> i64 { self.year() - 2000 }

    // I'd ideally like to include 'century' here, but there's some
    // discrepancy over what the result should be: the Gregorian
    // calendar calls the span from 2000 to 2099 the '21st Century', but
    // the ISO-8601 calendar calls it Century 20. I think the only way
    // for people to safely know which one they're going to get is to
    // just get the year value and do the calculation themselves, which
    // is simple enough because it's just a division.
}

/// The **time piece** trait is used for date and time values that have
/// time components of hours, minutes, and seconds.
pub trait TimePiece {

    /// The hour of the day.
    fn hour(&self) -> i8;

    /// The minute of the hour.
    fn minute(&self) -> i8;

    /// The second of the minute.
    fn second(&self) -> i8;

    /// The millisecond of the second.
    fn millisecond(&self) -> i16;
}

impl DatePiece for LocalDate {
    fn year(&self) -> i64 { self.ymd.year }
    fn month(&self) -> Month { self.ymd.month }
    fn day(&self) -> i8 { self.ymd.day }
    fn yearday(&self) -> i16 { self.yearday }
    fn weekday(&self) -> Weekday { self.weekday }
}

impl TimePiece for LocalTime {
    fn hour(&self) -> i8 { self.hour }
    fn minute(&self) -> i8 { self.minute }
    fn second(&self) -> i8 { self.second }
    fn millisecond(&self) -> i16 { self.millisecond }
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

// ---- helper implementations ----

impl PartialEq for LocalDate {
    fn eq(&self, other: &LocalDate) -> bool {
        self.ymd == other.ymd
    }
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


// ---- tests ----

#[cfg(test)]
mod test {
    pub use super::{LocalDateTime, LocalDate, LocalTime, Month, Weekday, YMD};

    mod seconds_to_datetimes {
        pub use super::*;

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
}
