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

#[derive(Debug)]
pub struct ISOString<'a, T: 'a>(&'a T);

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
        self.date().fmt(f)?;
        write!(f, "T")?;
        self.time().fmt(f)
    }
}

impl ISO for Offset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_utc() {
            write!(f, "Z")
        }
        else {
            f.write_str(if self.is_negative() { "-" } else { "+" })?;

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
