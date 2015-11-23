use std::fmt;
use cal::{LocalDate, LocalTime, LocalDateTime, DatePiece, TimePiece};
use util::RangeExt;

impl fmt::Debug for LocalDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let year = self.year();
        if year.is_within(0 .. 9999) {
            write!(f, "LocalDate({:04}-{:02}-{:02})", year, self.month() as usize, self.day())
        }
        else {
            write!(f, "LocalDate({:+05}-{:02}-{:02})", year, self.month() as usize, self.day())
        }
    }
}

impl fmt::Debug for LocalTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LocalTime({:02}:{:02}:{:02}.{:03})", self.hour(), self.minute(), self.second(), self.millisecond())
    }
}

impl fmt::Debug for LocalDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}T{:?}", self.date(), self.time())
    }
}

