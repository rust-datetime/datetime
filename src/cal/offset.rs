//! Datetimes with a fixed UTC offset.

use std::error::Error as ErrorTrait;
use std::fmt;

use duration::Duration;
use cal::{DatePiece, TimePiece};
use cal::datetime::{LocalDateTime, Month, Weekday, Error as DateTimeError};
use cal::fmt::ISO;
use util::RangeExt;


#[derive(PartialEq, Copy, Clone)]
pub struct Offset {
    offset_seconds: Option<i32>,
}

impl Offset {
    fn adjust(self, local: LocalDateTime) -> LocalDateTime {
        match self.offset_seconds {
            Some(s) => local + Duration::of(s as i64),
            None    => local,
        }
    }

    pub fn utc() -> Self {
        Self { offset_seconds: None }
    }

    pub fn of_seconds(seconds: i32) -> Result<Self, Error> {
        if seconds.is_within(-86400..86401) {
            Ok(Self { offset_seconds: Some(seconds) })
        }
        else {
            Err(Error::OutOfRange)
        }
    }

    pub fn of_hours_and_minutes(hours: i8, minutes: i8) -> Result<Self, Error> {
        if (hours.is_positive() && minutes.is_negative())
        || (hours.is_negative() && minutes.is_positive()) {
            Err(Error::SignMismatch)
        }
        else if hours <= -24 || hours >= 24 || minutes <= -60 || minutes >= 60 {
            Err(Error::OutOfRange)
        }
        else {
            let hours = hours as i32;
            let minutes = minutes as i32;
            Self::of_seconds(hours * (60 * 60) + minutes * 60)
        }
    }

    pub fn transform_date(self, local: LocalDateTime) -> OffsetDateTime {
        OffsetDateTime {
            local,
            offset: self,
        }
    }

    pub fn is_utc(self) -> bool {
        self.offset_seconds.is_none()
    }

    pub fn is_negative(self) -> bool {
        self.hours().is_negative() || self.minutes().is_negative() || self.seconds().is_negative()
    }

    pub fn hours(self) -> i8 {
        match self.offset_seconds {
            Some(s) => (s / 60 / 60) as i8,
            None => 0,
        }
    }

    pub fn minutes(self) -> i8 {
        match self.offset_seconds {
            Some(s) => (s / 60 % 60) as i8,
            None => 0,
        }
    }

    pub fn seconds(self) -> i8 {
        match self.offset_seconds {
            Some(s) => (s % 60) as i8,
            None => 0,
        }
    }
}

impl fmt::Debug for Offset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Offset({})", self.iso())
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Error {
    OutOfRange,
    SignMismatch,
    Date(DateTimeError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::OutOfRange    => write!(f, "offset field out of range"),
            Error::SignMismatch  => write!(f, "sign mismatch"),
            Error::Date(_)       => write!(f, "datetime field out of range"),
        }
    }
}

impl ErrorTrait for Error {
    fn cause(&self) -> Option<&dyn ErrorTrait> {
        if let Error::Date(ref e) = *self {
            Some(e)
        }
        else {
            None
        }
    }
}


#[derive(PartialEq, Copy, Clone)]
pub struct OffsetDateTime {
    pub local: LocalDateTime,
    pub offset: Offset,
}

impl DatePiece for OffsetDateTime {
    fn year(&self) -> i64 {
        self.offset.adjust(self.local).year()
    }

    fn month(&self) -> Month {
        self.offset.adjust(self.local).month()
    }

    fn day(&self) -> i8 {
        self.offset.adjust(self.local).day()
    }

    fn yearday(&self) -> i16 {
        self.offset.adjust(self.local).yearday()
    }

    fn weekday(&self) -> Weekday {
        self.offset.adjust(self.local).weekday()
    }
}

impl TimePiece for OffsetDateTime {
    fn hour(&self) -> i8 {
        self.offset.adjust(self.local).hour()
    }

    fn minute(&self) -> i8 {
        self.offset.adjust(self.local).minute()
    }

    fn second(&self) -> i8 {
        self.offset.adjust(self.local).second()
    }

    fn millisecond(&self) -> i16 {
        self.offset.adjust(self.local).millisecond()
    }
}

impl fmt::Debug for OffsetDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OffsetDateTime({})", self.iso())
    }
}

#[cfg(test)]
mod test {
    use super::Offset;

    #[test]
    fn fixed_seconds() {
        assert!(Offset::of_seconds(1234).is_ok());
    }

    #[test]
    fn fixed_seconds_panic() {
        assert!(Offset::of_seconds(100_000).is_err());
    }

    #[test]
    fn fixed_hm() {
        assert!(Offset::of_hours_and_minutes(5, 30).is_ok());
    }

    #[test]
    fn fixed_hm_negative() {
        assert!(Offset::of_hours_and_minutes(-3, -45).is_ok());
    }

    #[test]
    fn fixed_hm_err() {
        assert!(Offset::of_hours_and_minutes(8, 60).is_err());
    }

    #[test]
    fn fixed_hm_signs() {
        assert!(Offset::of_hours_and_minutes(-4, 30).is_err());
    }

    #[test]
    fn fixed_hm_signs_zero() {
        assert!(Offset::of_hours_and_minutes(4, 0).is_ok());
    }

    #[test]
    fn debug_zulu() {
        let offset = Offset::utc();
        let debugged = format!("{:?}", offset);
        assert_eq!(debugged, "Offset(Z)");
    }

    #[test]
    fn debug_offset() {
        let offset = Offset::of_seconds(-25 * 60 - 21).unwrap();
        let debugged = format!("{:?}", offset);
        assert_eq!(debugged, "Offset(-00:25:21)");
    }

    #[test]
    fn debug_offset_date_time() {
        use cal::{LocalDate, LocalTime, LocalDateTime, Month};

        let offset = Offset::of_seconds(25 * 60 + 21).unwrap();

        let then = LocalDateTime::new(
                    LocalDate::ymd(2009, Month::February, 13).unwrap(),
                    LocalTime::hms(23, 31, 30).unwrap());

        let debugged = format!("{:?}", offset.transform_date(then));
        assert_eq!(debugged, "OffsetDateTime(2009-02-13T23:31:30.000+00:25:21)");
    }
}
