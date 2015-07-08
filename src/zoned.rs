//! Dates and times paired with a time zone, and time zone definitions.

use local::{LocalDateTime, DatePiece, TimePiece, Month, Weekday};
use parse;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::error::Error;

use duration::Duration;
use tz::{Transition, parse};

/// A **time zone** is used to calculate how much to adjust a UTC-based time
/// based on its geographical location.
#[derive(Clone,Debug)]
pub enum TimeZone
{
    UTC,
    FixedOffset{offset: i32},
    VariableOffset{ transitions: Vec<Transition>}
}

/// A **time zone** is used to calculate how much to adjust a UTC-based time
/// based on its geographical location.
impl TimeZone
{
    fn adjust(&self, local: LocalDateTime) -> LocalDateTime
    {
        match self{
            &TimeZone::UTC => { self.adjust_utc(local)},
            &TimeZone::FixedOffset{offset} => { self.adjust_fixed(offset, local)},
            &TimeZone::VariableOffset{ref transitions} => { self.adjust_variable(&transitions, local)},
        }
    }

    fn adjust_utc(&self, local: LocalDateTime) -> LocalDateTime
    {
        local  // No adjustment needed! LocalDateTime uses UTC.
    }

    fn adjust_fixed(&self, offset:i32,  local: LocalDateTime) -> LocalDateTime
    {
        local + Duration::of(offset as i64)
    }

    fn adjust_variable(&self, transitions:&Vec<Transition>, local: LocalDateTime) -> LocalDateTime
    {
        let unix_timestamp = local.to_instant().seconds() as i32;

        // TODO: Replace this with a binary search
        match transitions.iter().find(|t| t.timestamp < unix_timestamp) {
            None     => local,
            Some(t)  => local + Duration::of(t.local_time_type.offset as i64),
        }
    }

    pub fn at(&self, local: LocalDateTime) -> ZonedDateTime
    {
        ZonedDateTime {
            local: local,
            time_zone: self.clone()
        }
    }

    /// Read time zone information in from the user's local time zone.
    pub fn localtime() -> Result<TimeZone, Box<Error>>
    {
        // TODO: replace this with some kind of factory.
        // this won't be appropriate for all systems
        TimeZone::zoneinfo(&Path::new("/etc/localtime"))
    }

    /// Read time zone information in from the file at the given path,
    /// returning a variable offset containing time transitions if successful,
    /// or an error if not.
    pub fn zoneinfo(path: &Path) -> Result<TimeZone, Box<Error>>
    {
        let mut contents = Vec::new();
        try!(File::open(path).unwrap().read_to_end(&mut contents));
        let mut tz = try!(parse(contents));

        // Sort the transitions *backwards* to make it easier to get the first
        // one *after* a specified time.
        tz.transitions.sort_by(|b, a| a.timestamp.cmp(&b.timestamp));

        Ok(TimeZone::VariableOffset{ transitions: tz.transitions })
    }
    /// Create a new fixed-offset timezone with the given number of seconds.
    ///
    /// Panics if the number of seconds is greater than one day's worth of
    /// seconds (86400) in either direction.
    pub fn of_seconds(seconds: i32) -> TimeZone
    {
        if seconds <= -86400 || seconds >= 86400 {
            panic!("Seconds offset greater than one day ({})", seconds)
        }
        else {
            TimeZone::FixedOffset{ offset: seconds }
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
    pub fn of_hours_and_minutes(hours: i8, minutes: i8) -> TimeZone{
        if (hours.is_positive() && minutes.is_negative())
        || (hours.is_negative() && minutes.is_positive()) {
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
            TimeZone::of_seconds(hours * 24 + minutes * 60)
        }
    }
}


/// A time paired with a time zone.
#[derive(Debug, Clone)]
pub struct ZonedDateTime
{
    pub local: LocalDateTime,
    pub time_zone: TimeZone,
}

impl ZonedDateTime
{
    pub fn parse(input: &str) -> Option<ZonedDateTime> {
        parse::parse_iso_8601_zoned(input)
    }
}
impl DatePiece for ZonedDateTime
{
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

impl TimePiece for ZonedDateTime
{
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


#[cfg(test)]
mod test {
    use super::TimeZone;

    #[test]
    fn fixed_seconds() {
        TimeZone::of_seconds(1234);
    }

    #[test] #[should_panic]
    fn fixed_seconds_panic() {
        TimeZone::of_seconds(100_000);
    }

    #[test]
    fn fixed_hm() {
        TimeZone::of_hours_and_minutes(5, 30);
    }

    #[test]
    fn fixed_hm_negative() {
        TimeZone::of_hours_and_minutes(-3, -45);
    }

    #[test] #[should_panic]
    fn fixed_hm_panic() {
        TimeZone::of_hours_and_minutes(8, 60);
    }

    #[test] #[should_panic]
    fn fixed_hm_signs() {
        TimeZone::of_hours_and_minutes(-4, 30);
    }

    #[test]
    fn fixed_hm_signs_zero() {
        TimeZone::of_hours_and_minutes(4, 0);
    }

    use super::{ZonedDateTime};
    use local::{DatePiece,TimePiece};
    #[test]
    fn parse_zoned()
    {
        let foo = ZonedDateTime::parse("2001-W05-6T04:05:06.123");
        assert_eq!(foo.map(|zdt|(
                           zdt.year(),
                           zdt.month() as i8,
                           zdt.day(),
                           zdt.hour(),
                           zdt.minute(),
                           zdt.second(),
                           zdt.millisecond())
        ),Some((2001,02,03, 04,05,06,123)));

        // TODO is this expected behaviour?
        // match
        // (ZonedDateTime::parse("2001-W05-6T04:05:06.123+00:00"),
        //  ZonedDateTime::parse("2001-W05-6T03:05:06.123+01:00"))
        // {
        //     (Some(UTC0),Some(UTC1)) => assert_eq!(UTC0.hour(),UTC1.hour()),
        //     _ => panic!("parsing error")
        // }

        assert!(ZonedDateTime::parse("2001-w05-6t04:05:06.123z").is_none());
        //assert!(ZonedDateTime::parse("2015-06-26T22:57:09Z00:00").is_none());
        //assert!(ZonedDateTime::parse("2015-06-26T22:57:09Z+00:00").is_none());
        assert!(ZonedDateTime::parse("2001-W05-6T04:05:06.123455Z").is_none());

        assert!(ZonedDateTime::parse("2001-02-03T04:05:06+07:00").is_some());
        assert!(ZonedDateTime::parse("20010203T040506+0700").is_some());
        assert!(ZonedDateTime::parse("2001-W05-6T04:05").is_some());
        assert!(ZonedDateTime::parse("2001-W05-6T04:05:06").is_some());
        assert!(ZonedDateTime::parse("2001-W05-6T04:05:06.123").is_some());
        assert!(ZonedDateTime::parse("2001-W05-6T04:05:06.123Z").is_some());
        assert!(ZonedDateTime::parse("2001-W05-6T04:05:06+07").is_some());
        assert!(ZonedDateTime::parse("2001-W05-6T04:05:06+07:00").is_some());
        assert!(ZonedDateTime::parse("2001-W05-6T04:05:06-07:00").is_some());
        assert!(ZonedDateTime::parse("2015-06-26TZ").is_some());
        assert!(ZonedDateTime::parse("2015-06-26").is_some());
        assert!(ZonedDateTime::parse("2015-06-26T22:57:09+00:00").is_some());
        assert!(ZonedDateTime::parse("2015-06-26T22:57:09Z").is_some());

    }
}

