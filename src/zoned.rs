use local::{LocalDateTime, DatePiece, TimePiece, Month, Weekday};

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::error::Error;

use duration::Duration;
use tz::{Transition, parse};


pub trait TimeZone: Sized {
    fn adjust(&self, local: LocalDateTime) -> LocalDateTime;

    fn at(&self, local: LocalDateTime) -> ZonedDateTime<Self> {
        ZonedDateTime {
            local: local,
            time_zone: self
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZonedDateTime<'tz, TZ: 'tz> {
    local: LocalDateTime,
    time_zone: &'tz TZ,
}

impl<'tz, TZ> DatePiece for ZonedDateTime<'tz, TZ> where TZ: TimeZone+'tz {
    fn year(&self) -> i64 {
        self.time_zone.adjust(self.local).year()
    }

    fn month(&self) -> Month {
        self.time_zone.adjust(self.local).month()
    }

    fn day(&self) -> i8 {
        self.time_zone.adjust(self.local).day()
    }

    fn yearday(&self) -> i16 {
        self.time_zone.adjust(self.local).yearday()
    }

    fn weekday(&self) -> Weekday {
        self.time_zone.adjust(self.local).weekday()
    }
}

impl<'tz, TZ> TimePiece for ZonedDateTime<'tz, TZ> where TZ: TimeZone {
    fn hour(&self) -> i8 {
        self.time_zone.adjust(self.local).hour()
    }

    fn minute(&self) -> i8 {
        self.time_zone.adjust(self.local).minute()
    }

    fn second(&self) -> i8 {
        self.time_zone.adjust(self.local).second()
    }

    fn millisecond(&self) -> i16 {
        self.time_zone.adjust(self.local).millisecond()
    }
}


#[derive(Debug, Clone)]
pub struct UTC;

impl TimeZone for UTC {
    fn adjust(&self, local: LocalDateTime) -> LocalDateTime {
        local
    }
}


#[derive(Debug, Clone)]
pub struct FixedOffset {
    offset: i32,
}

impl FixedOffset {
    pub fn of_seconds(seconds: i32) -> FixedOffset {
        if seconds > -86400 && seconds < 86400 {
            panic!("Offset greater than one day")
        }
        else {
            FixedOffset { offset: seconds }
        }
    }
}

impl TimeZone for FixedOffset {
    fn adjust(&self, local: LocalDateTime) -> LocalDateTime {
        local + Duration::of(self.offset as i64)
    }
}


#[derive(Debug, Clone)]
pub struct VariableOffset {
    transitions: Vec<Transition>,
}

impl VariableOffset {
    pub fn localtime() -> Result<VariableOffset, Box<Error>> {
        VariableOffset::zoneinfo(&Path::new("/etc/localtime"))
    }

    pub fn zoneinfo(path: &Path) -> Result<VariableOffset, Box<Error>> {
        let mut contents = Vec::new();
        try!(File::open(path).unwrap().read_to_end(&mut contents));
        let mut tz = try!(parse(contents));
        tz.transitions.sort_by(|b, a| a.timestamp.cmp(&b.timestamp));
        Ok(VariableOffset { transitions: tz.transitions })
    }
}

impl TimeZone for VariableOffset {
    fn adjust(&self, local: LocalDateTime) -> LocalDateTime {
        let unix_timestamp = local.to_instant().seconds() as i32;

        match self.transitions.iter().find(|t| t.timestamp < unix_timestamp) {
            None     => local,
            Some(t)  => local + Duration::of(t.local_time_type.offset as i64),
        }
    }
}
