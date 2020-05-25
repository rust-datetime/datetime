//! ISO-8601 date and time calculations, which use years, months, days,
//! hours, minutes, and seconds.

pub(crate) mod datetime;
pub(crate) mod fmt;
pub(crate) mod offset;
#[cfg(feature="parse")] pub(crate) mod parse;
pub mod zone;
pub mod convenience;

use self::datetime::{LocalDate, LocalTime, LocalDateTime, Weekday, Month};
use self::offset::{Offset, OffsetDateTime};


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

    // I’d ideally like to include “century” here, but there’s some
    // discrepancy over what the result should be: the Gregorian
    // calendar calls the span from 2000 to 2099 the “21st Century”, but
    // the ISO-8601 calendar calls it Century 20. I think the only way
    // for people to safely know which one they’re going to get is to
    // just get the year value and do the calculation themselves, which
    // is simple enough because it’s just a division.
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
