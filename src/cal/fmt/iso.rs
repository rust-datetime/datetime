use std::fmt;
use cal::{LocalDate, LocalTime, LocalDateTime, DatePiece, TimePiece};
use cal::{Offset, OffsetDateTime};
use util::RangeExt;


pub trait ISO: Sized {
    fn iso(&self) -> ISOString<Self> {
        ISOString(self)
    }

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

struct ISOString<'a, T: 'a>(&'a T);

impl<'a, T> fmt::Display for ISOString<'a, T>
where T: ISO {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ISO::fmt(self.0, f)
    }
}

impl ISO for LocalDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let year = self.year();
        if year.is_within(0 .. 9999) {
            write!(f, "{:04}-{:02}-{:02}", year, self.month() as usize, self.day())
        }
        else {
            write!(f, "{:+05}-{:02}-{:02}", year, self.month() as usize, self.day())
        }
    }
}

impl ISO for LocalTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}:{:02}:{:02}.{:03}", self.hour(), self.minute(), self.second(), self.millisecond())
    }
}

impl ISO for LocalDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(self.date().fmt(f));
        try!(write!(f, "T"));
        self.time().fmt(f)
    }
}

impl ISO for Offset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_utc() {
            write!(f, "Z")
        }
        else {
            try!(f.write_str(if self.is_negative() { "-" } else { "+" }));

            match (self.hours(), self.minutes(), self.seconds()) {
                (h, 0, 0) => write!(f, "{:02}", h.abs()),
                (h, m, 0) => write!(f, "{:02}:{:02}", h.abs(), m.abs()),
                (h, m, s) => write!(f, "{:02}:{:02}:{:02}", h.abs(), m.abs(), s.abs()),
            }
        }
    }
}

impl ISO for OffsetDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.local.iso(), self.offset.iso())
    }
}


#[cfg(test)]
mod test {
    pub use super::ISO;
    pub use std::string::ToString;

    mod datetimes {
        use super::*;
        use cal::{LocalDate, LocalTime, LocalDateTime, Month};

        #[test]
        fn recently() {
            let date = LocalDate::ymd(1600, Month::February, 28).unwrap();
            let debugged = date.iso().to_string();

            assert_eq!(debugged, "1600-02-28");
        }

        #[test]
        fn just_then() {
            let date = LocalDate::ymd(-753, Month::December, 1).unwrap();
            let debugged = date.iso().to_string();

            assert_eq!(debugged, "-0753-12-01");
        }

        #[test]
        fn far_far_future() {
            let date = LocalDate::ymd(10601, Month::January, 31).unwrap();
            let debugged = date.iso().to_string();

            assert_eq!(debugged, "+10601-01-31");
        }

        #[test]
        fn midday() {
            let time = LocalTime::hms(12, 0, 0).unwrap();
            let debugged = time.iso().to_string();

            assert_eq!(debugged, "12:00:00.000");
        }

        #[test]
        fn ascending() {
            let then = LocalDateTime::new(
                        LocalDate::ymd(2009, Month::February, 13).unwrap(),
                        LocalTime::hms(23, 31, 30).unwrap());

            let debugged = then.iso().to_string();

            assert_eq!(debugged, "2009-02-13T23:31:30.000");
        }
    }

    mod offsets {
        use super::*;
        use cal::{Offset};

        #[test]
        fn zulu() {
            let offset = Offset::utc();
            let debugged = offset.iso().to_string();
            assert_eq!(debugged, "Z");
        }

        #[test]
        fn hours() {
            let offset = Offset::of_hours_and_minutes(1, 0).unwrap();
            let debugged = offset.iso().to_string();
            assert_eq!(debugged, "+01");
        }

        #[test]
        fn hours_minutes() {
            let offset = Offset::of_hours_and_minutes(1, 30).unwrap();
            let debugged = offset.iso().to_string();
            assert_eq!(debugged, "+01:30");
        }

        #[test]
        fn dublin_mean_time() {
            let offset = Offset::of_seconds(-25 * 60 - 21).unwrap();
            let debugged = offset.iso().to_string();
            assert_eq!(debugged, "-00:25:21");
        }

        #[test]
        fn offset_date_time() {
            use cal::{LocalDate, LocalTime, LocalDateTime, Month};

            let offset = Offset::of_seconds(25 * 60 + 21).unwrap();

            let then = LocalDateTime::new(
                        LocalDate::ymd(2009, Month::February, 13).unwrap(),
                        LocalTime::hms(23, 31, 30).unwrap());

            let debugged = offset.transform_date(then).iso().to_string();
            assert_eq!(debugged, "2009-02-13T23:31:30.000+00:25:21");
        }
    }
}
