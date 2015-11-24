use std::fmt;
use cal::{LocalDate, LocalTime, LocalDateTime, DatePiece, TimePiece};
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


#[cfg(test)]
mod test {
    use super::*;
    use cal::{LocalDate, LocalTime, LocalDateTime, Month};
    use std::string::ToString;

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
