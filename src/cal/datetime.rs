//! Dates, times, datetimes, months, and weekdays.

use std::cmp::{Ordering, PartialOrd};
use std::error::Error as ErrorTrait;
use std::fmt;
use std::ops::{Add, Sub};
use std::ops::Deref;
use std::ops::{Range, RangeFrom, RangeTo, RangeFull};
use std::slice::Iter as SliceIter;

use cal::{DatePiece, TimePiece};
use cal::fmt::ISO;
use duration::Duration;
use instant::Instant;
use system::sys_time;
use util::RangeExt;

use self::Month::*;
use self::Weekday::*;


/// A single year.
///
/// This is just a wrapper around `i64` that performs year-related tests.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Year(pub i64);

impl Year {

    /// Returns whether this year is a leap year.
    ///
    /// ### Examples
    ///
    /// ```
    /// use datetime::Year;
    ///
    /// assert_eq!(Year(2000).is_leap_year(), true);
    /// assert_eq!(Year(1900).is_leap_year(), false);
    /// ```
    pub fn is_leap_year(self) -> bool {
        self.leap_year_calculations().1
    }

    /// Returns an iterator over a continuous span of months in this year,
    /// returning year-month pairs.
    ///
    /// This method takes one argument that can be of four different types,
    /// depending on the months you wish to iterate over:
    ///
    /// - The `RangeFull` type (such as `..`), which iterates over every
    ///   month;
    /// - The `RangeFrom` type (such as `April ..`), which iterates over
    ///   the months starting from the month given;
    /// - The `RangeTo` type (such as `.. June`), which iterates over the
    ///   months stopping at *but not including* the month given;
    /// - The `Range` type (such as `April .. June`), which iterates over
    ///   the months starting from the left one and stopping at *but not
    ///   including* the right one.
    ///
    /// ### Examples
    ///
    /// ```
    /// use datetime::Year;
    /// use datetime::Month::{April, June};
    ///
    /// let year = Year(1999);
    /// assert_eq!(year.months(..).count(), 12);
    /// assert_eq!(year.months(April ..).count(), 9);
    /// assert_eq!(year.months(April .. June).count(), 2);
    /// assert_eq!(year.months(.. June).count(), 5);
    /// ```
    pub fn months<S: MonthSpan>(self, span: S) -> YearMonths {
        YearMonths {
            year: self,
            iter: span.get_slice().iter(),
        }
    }

    /// Returns a year-month, pairing this year with the given month.
    ///
    /// ### Examples
    ///
    /// ```
    /// use datetime::{Year, Month};
    ///
    /// let expiry_date = Year(2017).month(Month::February);
    /// assert_eq!(*expiry_date.year, 2017);
    /// assert_eq!(expiry_date.month, Month::February);
    /// ```
    pub fn month(self, month: Month) -> YearMonth {
        YearMonth {
            year: self,
            month,
        }
    }

    /// Performs two related calculations for leap years, returning the
    /// results as a two-part tuple:
    ///
    /// 1. The number of leap years that have elapsed prior to this year;
    /// 2. Whether this year is a leap year or not.
    fn leap_year_calculations(self) -> (i64, bool) {
        let year = self.0 - 2000;

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

impl Deref for Year {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A span of months, which gets used to construct a `YearMonths` iterator.
///
/// See the `months` method of `Year` for more information.
pub trait MonthSpan {

    /// Returns a static slice of `Month` values contained by this span.
    fn get_slice(&self) -> &'static [Month];
}

static MONTHS: &[Month] = &[
    January,  February,  March,
    April,    May,       June,
    July,     August,    September,
    October,  November,  December,
];

impl MonthSpan for RangeFull {
    fn get_slice(&self) -> &'static [Month] {
        MONTHS
    }
}

impl MonthSpan for RangeFrom<Month> {
    fn get_slice(&self) -> &'static [Month] {
        &MONTHS[self.start.months_from_january() ..]
    }
}

impl MonthSpan for RangeTo<Month> {
    fn get_slice(&self) -> &'static [Month] {
        &MONTHS[.. self.end.months_from_january()]
    }
}

impl MonthSpan for Range<Month> {
    fn get_slice(&self) -> &'static [Month] {
        &MONTHS[self.start.months_from_january() .. self.end.months_from_january()]
    }
}


/// An iterator over a continuous span of months in a year.
///
/// Use the `months` method on `Year` to create instances of this iterator.
pub struct YearMonths {
    year: Year,
    iter: SliceIter<'static, Month>,
}

impl Iterator for YearMonths {
    type Item = YearMonth;

    fn next(&mut self) -> Option<YearMonth> {
        self.iter.next().map(|m| YearMonth {
            year: self.year,
            month: *m,
        })
    }
}

impl DoubleEndedIterator for YearMonths {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|m| YearMonth {
            year: self.year,
            month: *m,
        })
    }
}

impl fmt::Debug for YearMonths {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "YearMonths({}, {:?})", self.year.0, self.iter.as_slice())
    }
}

/// A month-year pair.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct YearMonth {
    pub year: Year,
    pub month: Month,
}

impl YearMonth {

    /// Returns the number of days in this month. This can be definitely
    /// known, as the paired year determines whether it’s a leap year, so
    /// there’s no chance of being caught out by February.
    ///
    /// ### Examples
    ///
    /// ```
    /// use datetime::Year;
    /// use datetime::Month::February;
    ///
    /// assert_eq!(Year(2000).month(February).day_count(), 29);
    /// assert_eq!(Year(1900).month(February).day_count(), 28);
    /// ```
    pub fn day_count(&self) -> i8 {
        self.month.days_in_month(self.year.is_leap_year())
    }

    /// Returns an iterator over a continuous span of days in this month,
    /// returning `LocalDate` values.
    ///
    /// ### Examples
    ///
    /// ```
    /// use datetime::Year;
    /// use datetime::Month::September;
    ///
    /// let ym = Year(1999).month(September);
    /// assert_eq!(ym.days(..).count(), 30);
    /// assert_eq!(ym.days(10 ..).count(), 21);
    /// assert_eq!(ym.days(10 .. 20).count(), 10);
    /// assert_eq!(ym.days(.. 20).count(), 19);
    /// ```
    pub fn days<S: DaySpan>(&self, span: S) -> MonthDays {
        MonthDays {
            ym: *self,
            range: span.get_range(self)
        }
    }

    /// Returns a `LocalDate` based on the day of this month.
    ///
    /// This is just a short-cut for the `LocalDate::ymd` constructor.
    pub fn day(&self, day: i8) -> Result<LocalDate, Error> {
        LocalDate::ymd(self.year.0, self.month, day)
    }
}


/// A span of days, which gets used to construct a `MonthDays` iterator.
pub trait DaySpan {

    /// Returns a `Range` of the day numbers specified for the given year-month pair.
    fn get_range(&self, ym: &YearMonth) -> Range<i8>;
}

impl DaySpan for RangeFull {
    fn get_range(&self, ym: &YearMonth) -> Range<i8> {
        1 .. ym.day_count() + 1
    }
}

impl DaySpan for RangeFrom<i8> {
    fn get_range(&self, ym: &YearMonth) -> Range<i8> {
        self.start .. ym.day_count() + 1
    }
}

impl DaySpan for RangeTo<i8> {
    fn get_range(&self, _ym: &YearMonth) -> Range<i8> {
        1 .. self.end
    }
}

impl DaySpan for Range<i8> {
    fn get_range(&self, _ym: &YearMonth) -> Range<i8> {
        self.clone()
    }
}


/// An iterator over a continuous span of days in a month.
///
/// Use the `days` method on `YearMonth` to create instances of this iterator.
#[derive(PartialEq, Debug)]
pub struct MonthDays {
    ym: YearMonth,
    range: Range<i8>,
}

impl Iterator for MonthDays {
    type Item = LocalDate;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().and_then(|d| LocalDate::ymd(self.ym.year.0, self.ym.month, d).ok())
    }
}

impl DoubleEndedIterator for MonthDays {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range.next_back().and_then(|d| LocalDate::ymd(self.ym.year.0, self.ym.month, d).ok())
    }
}


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
/// of having it on a year that’s a multiple of 400, and having the leap
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
/// depend on it) aren’t exposed to users of this library.
///
/// [^win32]: http://blogs.msdn.com/b/oldnewthing/archive/2009/03/06/9461176.aspx
///
const EPOCH_DIFFERENCE: i64 = 30 * 365   // 30 years between 2000 and 1970...
                            + 7          // plus seven days for leap years...
                            + 31 + 29;   // plus all the days in January and February in 2000.


/// This rather strange triangle is an array of the number of days elapsed
/// at the end of each month, starting at the beginning of March (the first
/// month after the EPOCH above), going backwards, ignoring February.
const TIME_TRIANGLE: &[i64; 11] =
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
    pub fn ymd(year: i64, month: Month, day: i8) -> Result<Self, Error> {
        YMD { year, month, day }
            .to_days_since_epoch()
            .map(|days| Self::from_days_since_epoch(days - EPOCH_DIFFERENCE))
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
    pub fn yd(year: i64, yearday: i64) -> Result<Self, Error> {
        if yearday.is_within(0..367) {
            let jan_1 = YMD { year, month: January, day: 1 };
            let days = jan_1.to_days_since_epoch()?;
            Ok(Self::from_days_since_epoch(days + yearday - 1 - EPOCH_DIFFERENCE))
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
    pub fn ywd(year: i64, week: i64, weekday: Weekday) -> Result<Self, Error> {
        let jan_4 = YMD { year, month: January, day: 4 };
        let correction = days_to_weekday(jan_4.to_days_since_epoch().unwrap() - EPOCH_DIFFERENCE).days_from_monday_as_one() as i64 + 3;

        let yearday = 7 * week + weekday.days_from_monday_as_one() as i64 - correction;

        if yearday <= 0 {
            let days_in_year = if Year(year - 1).is_leap_year() { 366 } else { 365 };
            Self::yd(year - 1, days_in_year + yearday)
        }
        else {
            let days_in_year = if Year(year).is_leap_year() { 366 } else { 365 };

            if yearday >= days_in_year {
                Self::yd(year + 1, yearday - days_in_year)
            }
            else {
                Self::yd(year, yearday)
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
    fn from_days_since_epoch(days: i64) -> Self {

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

        let mut years = std::cmp::min(remainder / 365, 3);
        remainder -= years * 365;  // remainder is now days left in this year

        // Leap year calculation goes thusly:
        //
        // 1. If the year is a multiple of 400, it’s a leap year.
        // 2. Else, if the year is a multiple of 100, it’s *not* a leap year.
        // 3. Else, if the year is a multiple of 4, it’s a leap year again!
        //
        // We already have the values for the numbers of multiples at this
        // point, and it’s safe to re-use them.
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
        // (it’s “11 - index” below because the triangle goes backwards)
        let result = TIME_TRIANGLE.iter()
                                  .enumerate()
                                  .find(|&(_, days)| *days <= remainder);

        let (mut month, month_days) = match result {
            Some((index, days)) => (11 - index, remainder - *days),
            None => (0, remainder),  // No month found? Then it’s February.
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
        Self {
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
    /// validity!** It’s possible to pass the wrong values in, such as having
    /// a wrong day value for a month, or having the yearday value out of
    /// step. Before using it, check that the values are all correct - or just
    /// use the `date!()` macro, which does this for you at compile-time.
    ///
    /// For this reason, the function is marked as `unsafe`, even though it
    /// (technically) uses unsafe components.
    pub unsafe fn _new_with_prefilled_values(year: i64, month: Month, day: i8, weekday: Weekday, yearday: i16) -> Self {
        Self {
            ymd: YMD { year, month, day },
            weekday,
            yearday,
        }
    }

    // I’m not 100% convinced on using `unsafe` for something that doesn’t
    // technically *need* to be unsafe, but I’ll stick with it for now.
}

impl DatePiece for LocalDate {
    fn year(&self) -> i64 { self.ymd.year }
    fn month(&self) -> Month { self.ymd.month }
    fn day(&self) -> i8 { self.ymd.day }
    fn yearday(&self) -> i16 { self.yearday }
    fn weekday(&self) -> Weekday { self.weekday }
}

impl fmt::Debug for LocalDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LocalDate({})", self.iso())
    }
}

impl PartialEq for LocalDate {
    fn eq(&self, other: &Self) -> bool {
        self.ymd == other.ymd
    }
}

impl PartialOrd for LocalDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ymd.partial_cmp(&other.ymd)
    }
}

impl Ord for LocalDate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ymd.cmp(&other.ymd)
    }
}

impl LocalTime {

    /// Computes the number of hours, minutes, and seconds, based on the
    /// number of seconds that have elapsed since midnight.
    pub fn from_seconds_since_midnight(seconds: i64) -> Self {
        Self::from_seconds_and_milliseconds_since_midnight(seconds, 0)
    }

    /// Computes the number of hours, minutes, and seconds, based on the
    /// number of seconds that have elapsed since midnight.
    pub fn from_seconds_and_milliseconds_since_midnight(seconds: i64, millisecond_of_second: i16) -> Self {
        Self {
            hour:   (seconds / 60 / 60) as i8,
            minute: (seconds / 60 % 60) as i8,
            second: (seconds % 60) as i8,
            millisecond: millisecond_of_second,
        }
    }

    /// Returns the time at midnight, with all fields initialised to 0.
    pub fn midnight() -> Self {
        Self { hour: 0, minute: 0, second: 0, millisecond: 0 }
    }

    /// Creates a new timestamp instance with the given hour and minute
    /// fields. The second and millisecond fields are set to 0.
    ///
    /// The values are checked for validity before instantiation, and
    /// passing in values out of range will return an `Err`.
    pub fn hm(hour: i8, minute: i8) -> Result<Self, Error> {
        if (hour.is_within(0..24) && minute.is_within(0..60))
        || (hour == 24 && minute == 00) {
            Ok(Self { hour, minute, second: 0, millisecond: 0 })
        }
        else {
            Err(Error::OutOfRange)
        }
    }

    /// Creates a new timestamp instance with the given hour, minute, and
    /// second fields. The millisecond field is set to 0.
    ///
    /// The values are checked for validity before instantiation, and
    /// passing in values out of range will return an `Err`.
    pub fn hms(hour: i8, minute: i8, second: i8) -> Result<Self, Error> {
        if (hour.is_within(0..24) && minute.is_within(0..60) && second.is_within(0..60))
        || (hour == 24 && minute == 00 && second == 00) {
            Ok(Self { hour, minute, second, millisecond: 0 })
        }
        else {
            Err(Error::OutOfRange)
        }
    }

    /// Creates a new timestamp instance with the given hour, minute,
    /// second, and millisecond fields.
    ///
    /// The values are checked for validity before instantiation, and
    /// passing in values out of range will return an `Err`.
    pub fn hms_ms(hour: i8, minute: i8, second: i8, millisecond: i16) -> Result<Self, Error> {
        if hour.is_within(0..24)   && minute.is_within(0..60)
        && second.is_within(0..60) && millisecond.is_within(0..1000)
        {
            Ok(Self { hour, minute, second, millisecond })
        }
        else {
            Err(Error::OutOfRange)
        }
    }

    /// Calculate the number of seconds since midnight this time is at,
    /// ignoring milliseconds.
    pub fn to_seconds(self) -> i64 {
        self.hour as i64 * 3600
            + self.minute as i64 * 60
            + self.second as i64
    }
}

impl TimePiece for LocalTime {
    fn hour(&self) -> i8 { self.hour }
    fn minute(&self) -> i8 { self.minute }
    fn second(&self) -> i8 { self.second }
    fn millisecond(&self) -> i16 { self.millisecond }
}

impl fmt::Debug for LocalTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LocalTime({})", self.iso())
    }
}


impl LocalDateTime {

    /// Computes a complete date-time based on the values in the given
    /// Instant parameter.
    pub fn from_instant(instant: Instant) -> Self {
        Self::at_ms(instant.seconds(), instant.milliseconds())
    }

    /// Computes a complete date-time based on the number of seconds that
    /// have elapsed since **midnight, 1st January, 1970**, setting the
    /// number of milliseconds to 0.
    pub fn at(seconds_since_1970_epoch: i64) -> Self {
        Self::at_ms(seconds_since_1970_epoch, 0)
    }

    /// Computes a complete date-time based on the number of seconds that
    /// have elapsed since **midnight, 1st January, 1970**,
    pub fn at_ms(seconds_since_1970_epoch: i64, millisecond_of_second: i16) -> Self {
        let seconds = seconds_since_1970_epoch - EPOCH_DIFFERENCE * SECONDS_IN_DAY;

        // Just split the input value into days and seconds, and let
        // LocalDate and LocalTime do all the hard work.
        let (days, secs) = split_cycles(seconds, SECONDS_IN_DAY);

        Self {
            date: LocalDate::from_days_since_epoch(days),
            time: LocalTime::from_seconds_and_milliseconds_since_midnight(secs, millisecond_of_second),
        }
    }

    /// Creates a new local date time from a local date and a local time.
    pub fn new(date: LocalDate, time: LocalTime) -> Self {
        Self {
            date,
            time,
        }
    }

    /// Returns the date portion of this date-time stamp.
    pub fn date(&self) -> LocalDate {
        self.date
    }

    /// Returns the time portion of this date-time stamp.
    pub fn time(&self) -> LocalTime {
        self.time
    }

    /// Creates a new date-time stamp set to the current time.
    #[cfg_attr(target_os = "redox", allow(unused_unsafe))]
    pub fn now() -> Self {
        let (s, ms) = unsafe { sys_time() };
        Self::at_ms(s, ms)
    }

    pub fn to_instant(&self) -> Instant {
        let seconds = self.date.ymd.to_days_since_epoch().unwrap() * SECONDS_IN_DAY + self.time.to_seconds();
        Instant::at_ms(seconds, self.time.millisecond)
    }

    pub fn add_seconds(&self, seconds: i64) -> Self {
        Self::from_instant(self.to_instant() + Duration::of(seconds))
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

impl fmt::Debug for LocalDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LocalDateTime({})", self.iso())
    }
}

impl Add<Duration> for LocalDateTime {
    type Output = Self;

    fn add(self, duration: Duration) -> Self {
        Self::from_instant(self.to_instant() + duration)
    }
}

impl Sub<Duration> for LocalDateTime {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self {
        Self::from_instant(self.to_instant() - duration)
    }
}


/// A **YMD** is an implementation detail of `LocalDate`. It provides
/// helper methods relating to the construction of `LocalDate` instances.
///
/// The main difference is that while all `LocalDate` values get checked
/// for validity before they are used, there is no such check for `YMD`.
/// The interface to `LocalDate` ensures that it should be impossible to
/// create an instance of the 74th of March, for example, but you’re
/// free to create such an instance of `YMD`. For this reason, it is not
/// exposed to implementors of this library.
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Debug, Copy)]
struct YMD {
    year:    i64,
    month:   Month,
    day:     i8,
}

impl YMD {

    /// Calculates the number of days that have elapsed since the 1st
    /// January, 1970. Returns the number of days if this datestamp is
    /// valid; None otherwise.
    ///
    /// This method returns a Result instead of exposing is_valid to
    /// the user, because the leap year calculations are used in both
    /// functions, so it makes more sense to only do them once.
    fn to_days_since_epoch(&self) -> Result<i64, Error> {
        let years = self.year - 2000;
        let (leap_days_elapsed, is_leap_year) = Year(self.year).leap_year_calculations();

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
    fn is_valid(&self, is_leap_year: bool) -> bool {
        self.day >= 1 && self.day <= self.month.days_in_month(is_leap_year)
    }
}

/// Computes the weekday, given the number of days that have passed
/// since the EPOCH.
fn days_to_weekday(days: i64) -> Weekday {
    // March 1st, 2000 was a Wednesday, so add 3 to the number of days.
    let weekday = (days + 3) % 7;

    // We can unwrap since we’ve already done the bounds checking.
    Weekday::from_zero(if weekday < 0 { weekday + 7 } else { weekday } as i8).unwrap()
}

/// Split a number of years into a number of year-cycles, and the number
/// of years left over that don’t fit into a cycle. This is also used
/// for day-cycles.
///
/// This is essentially a division operation with the result and the
/// remainder, with the difference that a negative value gets ‘wrapped
/// around’ to be a positive value, owing to the way the modulo operator
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "datetime field out of range")
    }
}

impl ErrorTrait for Error {
}


/// A month of the year, starting with January, and ending with December.
///
/// This is stored as an enum instead of just a number to prevent
/// off-by-one errors: is month 2 February (1-indexed) or March (0-indexed)?
/// In this case, it’s 1-indexed, to have January become 1 when you use
/// `as i32` in code.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum Month {
    January =  1, February =  2, March     =  3,
    April   =  4, May      =  5, June      =  6,
    July    =  7, August   =  8, September =  9,
    October = 10, November = 11, December  = 12,
}

#[allow(clippy::match_same_arms)]
impl Month {

    /// Returns the number of days in this month, depending on whether it’s
    /// a leap year or not.
    pub fn days_in_month(self, leap_year: bool) -> i8 {
        match self {
            January   => 31, February  => if leap_year { 29 } else { 28 },
            March     => 31, April     => 30,
            May       => 31, June      => 30,
            July      => 31, August    => 31,
            September => 30, October   => 31,
            November  => 30, December  => 31,
        }
    }

    /// Returns the number of days that have elapsed in a year *before* this
    /// month begins, with no leap year check.
    fn days_before_start(self) -> i16 {
        match self {
            January =>   0, February =>  31, March     =>  59,
            April   =>  90, May      => 120, June      => 151,
            July    => 181, August   => 212, September => 243,
            October => 273, November => 304, December  => 334,
        }
    }

    pub fn months_from_january(self) -> usize {
        match self {
            January =>   0, February =>   1, March     =>  2,
            April   =>   3, May      =>   4, June      =>  5,
            July    =>   6, August   =>   7, September =>  8,
            October =>   9, November =>  10, December  => 11,
        }
    }

    /// Returns the month based on a number, with January as **Month 1**,
    /// February as **Month 2**, and so on.
    ///
    /// ```rust
    /// use datetime::Month;
    /// assert_eq!(Month::from_one(5), Ok(Month::May));
    /// assert!(Month::from_one(0).is_err());
    /// ```
    pub fn from_one(month: i8) -> Result<Self, Error> {
        Ok(match month {
             1 => January,   2 => February,   3 => March,
             4 => April,     5 => May,        6 => June,
             7 => July,      8 => August,     9 => September,
            10 => October,  11 => November,  12 => December,
             _ => return Err(Error::OutOfRange),
        })
    }

    /// Returns the month based on a number, with January as **Month 0**,
    /// February as **Month 1**, and so on.
    ///
    /// ```rust
    /// use datetime::Month;
    /// assert_eq!(Month::from_zero(5), Ok(Month::June));
    /// assert!(Month::from_zero(12).is_err());
    /// ```
    pub fn from_zero(month: i8) -> Result<Self, Error> {
        Ok(match month {
            0 => January,   1 => February,   2 => March,
            3 => April,     4 => May,        5 => June,
            6 => July,      7 => August,     8 => September,
            9 => October,  10 => November,  11 => December,
            _ => return Err(Error::OutOfRange),
        })
    }
}


/// A named day of the week.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Weekday {
    Sunday, Monday, Tuesday, Wednesday, Thursday, Friday, Saturday,
}

// Sunday is Day 0. This seems to be a North American thing? It’s pretty
// much an arbitrary choice, and as you can’t use the `from_zero` method,
// it won’t affect you at all. If you want to change it, the only thing
// that should be affected is `LocalDate::days_to_weekday`.
//
// I’m not going to give weekdays an Ord instance because there’s no
// real standard as to whether Sunday should come before Monday, or the
// other way around. Luckily, they don’t need one, as the field is
// ignored when comparing LocalDates.

impl Weekday {
    fn days_from_monday_as_one(self) -> i8 {
        match self {
            Sunday   => 7,  Monday    => 1,
            Tuesday  => 2,  Wednesday => 3,
            Thursday => 4,  Friday    => 5,
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
    pub fn from_zero(weekday: i8) -> Result<Self, Error> {
        Ok(match weekday {
            0 => Sunday,     1 => Monday,    2 => Tuesday,
            3 => Wednesday,  4 => Thursday,  5 => Friday,
            6 => Saturday,   _ => return Err(Error::OutOfRange),
        })
    }

    pub fn from_one(weekday: i8) -> Result<Self, Error> {
        Ok(match weekday {
            7 => Sunday,     1 => Monday,    2 => Tuesday,
            3 => Wednesday,  4 => Thursday,  5 => Friday,
            6 => Saturday,   _ => return Err(Error::OutOfRange),
        })
    }
}


/// Misc tests that don’t seem to fit anywhere.
#[cfg(test)]
mod test {
    pub(crate) use super::{LocalDateTime, LocalDate, LocalTime, Month};


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

    mod debug {
        use super::*;

        #[test]
        fn recently() {
            let date = LocalDate::ymd(1600, Month::February, 28).unwrap();
            let debugged = format!("{:?}", date);

            assert_eq!(debugged, "LocalDate(1600-02-28)");
        }

        #[test]
        fn just_then() {
            let date = LocalDate::ymd(-753, Month::December, 1).unwrap();
            let debugged = format!("{:?}", date);

            assert_eq!(debugged, "LocalDate(-0753-12-01)");
        }

        #[test]
        fn far_far_future() {
            let date = LocalDate::ymd(10601, Month::January, 31).unwrap();
            let debugged = format!("{:?}", date);

            assert_eq!(debugged, "LocalDate(+10601-01-31)");
        }

        #[test]
        fn midday() {
            let time = LocalTime::hms(12, 0, 0).unwrap();
            let debugged = format!("{:?}", time);

            assert_eq!(debugged, "LocalTime(12:00:00.000)");
        }

        #[test]
        fn ascending() {
            let then = LocalDateTime::new(
                        LocalDate::ymd(2009, Month::February, 13).unwrap(),
                        LocalTime::hms(23, 31, 30).unwrap());
            let debugged = format!("{:?}", then);

            assert_eq!(debugged, "LocalDateTime(2009-02-13T23:31:30.000)");
        }
    }
}
