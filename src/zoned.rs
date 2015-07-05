use local::{LocalDateTime, DatePiece, TimePiece, Month, Weekday};

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::error::Error;

use duration::Duration;
use tz::{Transition, parse};


pub trait TimeZone: Clone {
    fn adjust(&self, local: LocalDateTime) -> LocalDateTime;

    fn at(&self, local: LocalDateTime) -> ZonedDateTime<Self> {
        ZonedDateTime {
            local: local,
            time_zone: self.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZonedDateTime<TZ> {
    local: LocalDateTime,
    time_zone: TZ,
}

impl<TZ> DatePiece for ZonedDateTime<TZ> where TZ: TimeZone {
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

impl<TZ> TimePiece for ZonedDateTime<TZ> where TZ: TimeZone {
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


/// **Coordinated Universal Time** is the time standard that regulates time
/// across the world. It does not respect daylight saving time, or undergo any
/// historical or political changes, which makes it suitable for using as the
/// 'base time zone' when the actual time zone is not known or relevant.
#[derive(Debug, Clone)]
pub struct UTC;

impl TimeZone for UTC {
    fn adjust(&self, local: LocalDateTime) -> LocalDateTime {
        local  // No adjustment needed! LocalDateTime uses UTC.
    }
}


/// A time zone with a **fixed offset** differs from UTC by a given number of
/// seconds. This is usually given in hours, but occasionally minutes are also
/// specified.
#[derive(Debug, Clone)]
pub struct FixedOffset {
    offset: i32,
}

impl FixedOffset {

    /// Create a new fixed-offset timezone with the given number of seconds.
    ///
    /// Panics if the number of seconds is greater than one day's worth of
    /// seconds (86400) in either direction.
    pub fn of_seconds(seconds: i32) -> FixedOffset {
        if seconds <= -86400 || seconds >= 86400 {
            panic!("Seconds offset greater than one day ({})", seconds)
        }
        else {
            FixedOffset { offset: seconds }
        }
    }

    /// Create a new fixed-offset timezone with the given number of hours and
    /// minutes.
    ///
    /// The values should either be both positive or both negative.
    ///
    /// Panics if the numbers are greater than their unit allows (more than 23
    /// hours or 59 minutes) in either direction, or if the values differ in
    /// sign (such as a positive number of hours with a negative number of
    /// minutes).
    pub fn of_hours_and_minutes(hours: i8, minutes: i8) -> FixedOffset {
        if hours.signum() != minutes.signum() {
            panic!("Hour and minute values differ in sign ({} and {}", hours, minutes);
        }
        else if hours <= -24 || hours >= 24 {
            panic!("Hours offset greater than one day ({})", hours);
        }
        else if minutes <= -60 || minutes >= 60 {
            panic!("Minutes offset greater than one hour ({})", minutes);
        }
        else {
            let hours = hours as i32;
            let minutes = minutes as i32;
            FixedOffset::of_seconds(hours * 24 + minutes * 60)
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


#[cfg(test)]
mod test {
    use super::FixedOffset;

	#[test]
	fn fixed_seconds() {
		FixedOffset::of_seconds(1234);
	}

	#[test] #[should_panic]
	fn fixed_seconds_panic() {
	    FixedOffset::of_seconds(100_000);
	}

	#[test]
	fn fixed_hm() {
	    FixedOffset::of_hours_and_minutes(5, 30);
	}

	#[test]
	fn fixed_hm_negative() {
	    FixedOffset::of_hours_and_minutes(-3, -45);
	}

	#[test] #[should_panic]
	fn fixed_hm_panic() {
	    FixedOffset::of_hours_and_minutes(8, 60);
	}

	#[test] #[should_panic]
	fn fixed_hm_signs() {
	    FixedOffset::of_hours_and_minutes(-4, 30);
	}
}
