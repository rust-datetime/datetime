//! Dates and times paired with a time zone, and time zone definitions.

use local::{LocalDateTime, DatePiece, TimePiece, Month, Weekday};

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::error::Error;

use duration::Duration;
use tz::{Transition, parse};


/// A **time zone** is used to calculate how much to adjust a UTC-based time
/// based on its geographical location.
pub trait TimeZone: Clone {

    /// Adjust this local time by a number of seconds.
    ///
    /// Although the return value is a `LocalDateTime`, this actual type is
    /// never exposed.
    fn adjust(&self, local: LocalDateTime) -> LocalDateTime;

    /// Create a `ZonedDateTime` instance using a clone of this time zone.
    fn at(&self, local: LocalDateTime) -> ZonedDateTime<Self> {
        ZonedDateTime {
            local: local,
            time_zone: self.clone()
        }
    }
}


/// A time paired with a time zone.
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


/// A time zone with a **variable offset** differs from UTC by a variable
/// amount that depends on the date, such as for political reasons when a
/// country decides to change. By encoding all the transitions, it's possible
/// to adjust times *after* the transition time while leaving the dates
/// *before* it unaffected.
#[derive(Debug, Clone)]
pub struct VariableOffset {
    transitions: Vec<Transition>,
}

impl VariableOffset {

    /// Read time zone information in from the user's local time zone.
    pub fn localtime() -> Result<VariableOffset, Box<Error>> {
        // TODO: replace this with some kind of factory.
        // this won't be appropriate for all systems
        VariableOffset::zoneinfo(&Path::new("/etc/localtime"))
    }

    /// Read time zone information in from the file at the given path,
    /// returning a variable offset containing time transitions if successful,
    /// or an error if not.
    pub fn zoneinfo(path: &Path) -> Result<VariableOffset, Box<Error>> {
        let mut contents = Vec::new();
        try!(File::open(path).unwrap().read_to_end(&mut contents));
        let mut tz = try!(parse(contents));

        // Sort the transitions *backwards* to make it easier to get the first
        // one *after* a specified time.
        tz.transitions.sort_by(|b, a| a.timestamp.cmp(&b.timestamp));

        Ok(VariableOffset { transitions: tz.transitions })
    }
}

impl TimeZone for VariableOffset {
    fn adjust(&self, local: LocalDateTime) -> LocalDateTime {
        let unix_timestamp = local.to_instant().seconds() as i32;

        // TODO: Replace this with a binary search
        match self.transitions.iter().find(|t| t.timestamp < unix_timestamp) {
            None     => local,
            Some(t)  => local + Duration::of(t.local_time_type.offset as i64),
        }
    }
}


/// An enum of anything that could be a time zone, for cases when you don't
/// know in advance which type of time zone you'll need.
pub enum AnyTimeZone {
    UTC(UTC),
    Fixed(FixedOffset),
    Variable(VariableOffset),
}

impl TimeZone for AnyTimeZone {
    fn adjust(&self, local: LocalDateTime) -> LocalDateTime {
        pub use self::AnyTimeZone::*;

        match self {
            &UTC(utc)     => utc.adjust(local),
            &Fixed(f)     => f.adjust(local),
            &Variable(v)  => v.adjust(local),
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
