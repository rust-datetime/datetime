//! Datetimes with a variable UTC offset, and time zone calculations.

use std::borrow::Cow;
use std::sync::Arc;

use duration::Duration;
use instant::Instant;
use cal::{LocalDateTime, DatePiece, TimePiece, Month, Weekday};
use util::RangeExt;


/// A **time zone**, which here is a list of timespans, each containing a
/// fixed offset for the current location’s time from UTC.
#[derive(Debug, Clone)]
pub struct TimeZone(pub TimeZoneSource<'static>);

#[derive(Debug, Clone)]
pub enum TimeZoneSource<'a> {
    Static(&'a StaticTimeZone<'a>),
    Runtime(Arc<runtime::OwnedTimeZone>),
}

#[derive(PartialEq, Debug)]
pub struct StaticTimeZone<'a> {

    /// This zone’s name in the zoneinfo database, such as “America/New_York”.
    pub name: &'a str,

    /// The set of timespans used in this time zone.
    pub fixed_timespans: FixedTimespanSet<'a>,
}

impl TimeZone {

    pub fn zone_name(&self) -> Option<&str> {
        match self.0 {
            TimeZoneSource::Static(ref tz)   => Some(tz.name),
            TimeZoneSource::Runtime(ref arc) => arc.name.as_ref().map(|x| &**x),
        }
    }

    /// Returns the total offset from UTC, in seconds, that this time zone
    /// has at the given datetime.
    pub fn offset(&self, datetime: LocalDateTime) -> i64 {
        match self.0 {
            TimeZoneSource::Static(ref tz)   => tz.fixed_timespans.offset(datetime),
            TimeZoneSource::Runtime(ref arc) => arc.fixed_timespans.borrow().offset(datetime),
        }
    }

    /// Returns the time zone abbreviation that this time zone has at the
    /// given datetime. As always, abbreviations are notoriously vague, and
    /// should only be used when referring to a known timezone.
    pub fn name(&self, datetime: LocalDateTime) -> String {
        match self.0 {
            TimeZoneSource::Static(ref tz)   => tz.fixed_timespans.name(datetime),
            TimeZoneSource::Runtime(ref arc) => arc.fixed_timespans.borrow().name(datetime),
        }
    }

    /// Whether this time zone is “fixed”: a fixed time zone has no
    /// transitions, meaning it will always be at the same offset from UTC.
    ///
    /// There are relatively few of these, namely the European timezones
    /// WET, CET, MET, and EET, and the North American timezones EST5EDT,
    /// CST6CDT, MST7MDT, and PST8PDT, none of which actually corresponds to
    /// a geographical location.
    pub fn is_fixed(&self) -> bool {
        match self.0 {
            TimeZoneSource::Static(ref tz)   => tz.fixed_timespans.is_fixed(),
            TimeZoneSource::Runtime(ref arc) => arc.fixed_timespans.borrow().is_fixed(),
        }
    }

    /// Converts a local datetime in UTC to a zoned datetime that uses this
    /// time zone.
    pub fn to_zoned(&self, datetime: LocalDateTime) -> LocalDateTime {
        datetime + Duration::of(self.offset(datetime))
    }

    /// Converts a local datetime that is *already* informally in this time
    /// zone into a zoned datetime that actually uses this time zone.
    ///
    /// For example, say you have the current time for a time zone, but you
    /// *don’t* know what the current offset from UTC is. This method
    /// computes the offset, then *subtracts* rather than adds it, resulting
    /// in a value that gets displayed as the current time. In other words,
    /// calling `hour()` or `year()` or any of the other view methods on one
    /// of the resulting values will *always* return the same as the
    /// datetime initially passed in, no matter what the current offset is.
    ///
    /// This method can return 0, 1, or 2 values, depending on whether the
    /// datetime passed in falls between two timespans (an impossible time)
    /// or overlaps two separate timespans (an ambiguous time). The result
    /// will *almost* always be precise, but there are edge cases you need
    /// to watch out for.
    pub fn convert_local(&self, local: LocalDateTime) -> LocalTimes {
        match self.0 {
            TimeZoneSource::Static(ref tz)   => tz.fixed_timespans.convert_local(local, &self.0),
            TimeZoneSource::Runtime(ref arc) => arc.fixed_timespans.borrow().convert_local(local, &self.0),
        }
    }
}


/// A set of timespans, separated by the instances at which the timespans
/// change over. There will always be one more timespan than transitions.
#[derive(PartialEq, Debug, Clone)]
pub struct FixedTimespanSet<'a> {

    /// The first timespan, which is assumed to have been in effect up until
    /// the initial transition instant (if any). Each set has to have at
    /// least one timespan.
    pub first: FixedTimespan<'a>,

    /// The rest of the timespans, as a slice of tuples, each containing:
    ///
    /// 1. A transition instant at which the previous timespan ends and the
    ///    next one begins, stored as a Unix timestamp;
    /// 2. The actual timespan to transition into.
    pub rest: &'a [ (i64, FixedTimespan<'a>) ],
}

/// An individual timespan with a fixed offset.
#[derive(PartialEq, Debug, Clone)]
pub struct FixedTimespan<'a> {

    /// The *total* offset in effect during this timespan, in seconds. This
    /// is the sum of the standard offset from UTC (the zone’s standard
    /// time), and any extra daylight-saving offset.
    pub offset: i64,

    /// Whether there was any daylight-saving offset in effect during this
    /// timespan.
    pub is_dst: bool,

    /// The abbreviation in use during this timespan, such as “GMT” or
    /// “PDT”. Abbreviations are notoriously vague, and should only be used
    /// for referring to a known timezone.
    pub name: Cow<'a, str>,
}

impl<'a> FixedTimespanSet<'a> {
    fn find(&self, time: i64) -> &FixedTimespan {
        match self.rest.iter().take_while(|t| t.0 < time).last() {
            None     => &self.first,
            Some(zd) => &zd.1,
        }
    }

    fn offset(&self, datetime: LocalDateTime) -> i64 {
        let unix_timestamp = datetime.to_instant().seconds();
        self.find(unix_timestamp).offset
    }

    fn name(&self, datetime: LocalDateTime) -> String {
        let unix_timestamp = datetime.to_instant().seconds();
        self.find(unix_timestamp).name.to_string()
    }

    fn is_fixed(&self) -> bool {
        self.rest.is_empty()
    }

    fn convert_local(&self, local: LocalDateTime, source: &TimeZoneSource<'a>) -> LocalTimes<'a> {
        let unix_timestamp = local.to_instant().seconds();

        let zonify = |offset| ZonedDateTime {
            adjusted: local,
            current_offset: offset,
            time_zone: source.clone(),
        };

        let timespans = self.find_with_surroundings(unix_timestamp);

        if let Some((previous_zone, previous_transition_time)) = timespans.previous {

            assert!(timespans.current.offset != previous_zone.offset,
                    "Offsets cannot be equal! Is this a non-transition transition?");

            println!("unix timestamp {:?}, previous time {:?}", unix_timestamp, previous_transition_time);

            // Test whether this timestamp is in the *overlap* after the
            // current timespan starts but before the previous one ends.
            if previous_zone.offset > timespans.current.offset
            && (unix_timestamp - previous_transition_time).is_within(timespans.current.offset .. previous_zone.offset) {
                return LocalTimes::Ambiguous {
                    earlier:  zonify(previous_zone.offset),
                    later:    zonify(timespans.current.offset),
                };
            }

            // Test whether this timestamp is in the *space* after the
            // previous timespan ends but before the current one starts.
            if previous_zone.offset < timespans.current.offset
            && (unix_timestamp - previous_transition_time).is_within(previous_zone.offset .. timespans.current.offset) {
                return LocalTimes::Impossible;
            }
        }

        if let Some(&(next_transition_time, ref next_zone)) = timespans.next {

            assert!(timespans.current.offset != next_zone.offset,
                "Offsets cannot be equal! Is this a non-transition transition?");

            println!("unix timestamp {:?}, next time {:?}", unix_timestamp, next_transition_time);
            println!("offset 1 {:?}, offset 2 {:?}", next_zone.offset, timespans.current.offset);

            // Test whether this timestamp is in the *overlap* after the
            // next timespan starts but before the current one ends.
            if timespans.current.offset > next_zone.offset
            && (unix_timestamp - next_transition_time).is_within(next_zone.offset .. timespans.current.offset) {
                return LocalTimes::Ambiguous {
                    earlier:  zonify(timespans.current.offset),
                    later:    zonify(next_zone.offset),
                };
            }

            // Test whether this timestamp is in the *space* after the
            // current timespan ends but before the next one starts.
            if timespans.current.offset < next_zone.offset
            && (unix_timestamp - next_transition_time).is_within(timespans.current.offset .. next_zone.offset) {
                return LocalTimes::Impossible;
            }
        }

        LocalTimes::Precise(zonify(timespans.current.offset))
    }

    fn find_with_surroundings(&self, time: i64) -> Surroundings {
        if let Some((position, _)) = self.rest.iter().enumerate().take_while(|&(_, t)| t.0 < time).last() {
            // There’s a matching time in the ‘rest’ list, so return that
            // time along with the two sets of details around it.

            let previous_details = if position == 0 {
                &self.first
            }
            else {
                &self.rest[position - 1].1
            };

            Surroundings {
                previous:  Some((previous_details, self.rest[position].0)),
                current:   &self.rest[position].1,
                next:      self.rest.get(position + 1),
            }
        }
        else {
            // If there’s no matching time in the ‘rest’ list, it must be
            // the ‘first’ one.
            Surroundings {
                previous: None,
                current:  &self.first,
                next:     self.rest.get(0),
            }
        }
    }
}


#[derive(PartialEq, Debug)]
struct Surroundings<'a> {
    previous:  Option<(&'a FixedTimespan<'a>, i64)>,
    current:   &'a FixedTimespan<'a>,
    next:      Option<&'a (i64, FixedTimespan<'a>)>,
}


/// The result of converting a *local* time to a *zoned* time with the same
/// time components. See `TimeZone::convert_local` for more information.
#[derive(Debug)]
pub enum LocalTimes<'a> {

    /// This local time is impossible (when a time occurs between two
    /// timespans, which should never be shown on a wall clock).
    Impossible,

    /// This local time can be defined unambiguously.
    Precise(ZonedDateTime<'a>),

    /// This local time is ambiguous (when a time overlaps two timespans,
    /// which happens twice on a wall clock rather than once).
    Ambiguous { earlier: ZonedDateTime<'a>, later: ZonedDateTime<'a> },
}

impl<'a> LocalTimes<'a> {

    /// Extracts the *precise* zoned date time, if present; **panics otherwise**.
    ///
    /// It is almost always preferable to use pattern matching on a
    /// `LocalTimes` value and handle the impossible/ambiguous cases
    /// explicitly, rather than risking a panic.
    pub fn unwrap_precise(self) -> ZonedDateTime<'a> {
        match self {
            LocalTimes::Precise(p)        => p,
            LocalTimes::Impossible        => panic!("called `LocalTimes::unwrap()` on an `Impossible` value"),
            LocalTimes::Ambiguous { .. }  => panic!("called `LocalTimes::unwrap()` on an `Ambiguous` value: {:?}", self),
        }
    }

    /// Returns whether this local times result is impossible (when a time
    /// occurs between two timespans, which should never be shown on a wall
    /// clock).
    pub fn is_impossible(&self) -> bool {
        match *self {
            LocalTimes::Impossible => true,
            _                      => false,
        }
    }

    /// Returns whether this local times result is ambiguous (when a time
    /// overlaps two timespans, which happens twice on a wall clock rather
    /// than once).
    pub fn is_ambiguous(&self) -> bool {
        match *self {
            LocalTimes::Ambiguous { .. } => true,
            _                            => false,
        }
    }
}


#[derive(Debug)]
pub struct ZonedDateTime<'a> {
    adjusted: LocalDateTime,
    current_offset: i64,
    time_zone: TimeZoneSource<'a>,
}

impl<'a> ZonedDateTime<'a> {
    pub fn to_instant(&self) -> Instant {
        (self.adjusted - Duration::of(self.current_offset)).to_instant()
    }
}

impl<'a> DatePiece for ZonedDateTime<'a> {
    fn year(&self) -> i64 { self.adjusted.year() }
    fn month(&self) -> Month { self.adjusted.month() }
    fn day(&self) -> i8 { self.adjusted.day() }
    fn yearday(&self) -> i16 { self.adjusted.yearday() }
    fn weekday(&self) -> Weekday { self.adjusted.weekday() }
}

impl<'a> TimePiece for ZonedDateTime<'a> {
    fn hour(&self) -> i8 { self.adjusted.hour() }
    fn minute(&self) -> i8 { self.adjusted.minute() }
    fn second(&self) -> i8 { self.adjusted.second() }
    fn millisecond(&self) -> i16 { self.adjusted.millisecond() }
}


/// The “type” of time that a transition is specified in.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TimeType {

    /// Wall-clock time: a transition specified when the current time in
    /// that zone, including any daylight-saving matches, matches the
    /// transition’s time spec.
    Wall,

    /// Standard Time: a transition specified when the *standard* time in
    /// that zone, which excludes any daylight-saving offset, matches the
    /// transition’s time spec.
    Standard,

    /// UTC: a transition specified when the time in UTC matches the
    /// transition’s time spec.
    UTC,
}

pub mod runtime {
    use super::{FixedTimespan, FixedTimespanSet};

    #[derive(PartialEq, Debug)]
    pub struct OwnedTimeZone {
        pub name: Option<String>,
        pub fixed_timespans: OwnedFixedTimespanSet,
    }

    #[derive(PartialEq, Debug)]
    pub struct OwnedFixedTimespanSet {
        pub first: FixedTimespan<'static>,
        pub rest: Vec<(i64, FixedTimespan<'static>)>,
    }

    impl OwnedFixedTimespanSet {
        pub fn borrow(&self) -> FixedTimespanSet {
            FixedTimespanSet {
                first: self.first.clone(),
                rest: &*self.rest,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::Surroundings;
    use std::borrow::Cow;

    const NONE: FixedTimespanSet<'static> = FixedTimespanSet {
        first: FixedTimespan {
            offset: 0,
            is_dst: false,
            name: Cow::Borrowed("ZONE_A"),
        },
        rest: &[],
    };

    #[test]
    fn empty() {
        assert_eq!(NONE.find_with_surroundings(1184000000), Surroundings {
            previous: None,
            current: &FixedTimespan {
                offset: 0,
                is_dst: false,
                name: Cow::Borrowed("ZONE_A"),
            },
            next: None,
        })
    }

    const ONE: FixedTimespanSet<'static> = FixedTimespanSet {
        first: FixedTimespan {
            offset: 0,
            is_dst: false,
            name: Cow::Borrowed("ZONE_A"),
        },
        rest: &[
            (1174784400, FixedTimespan {
                offset: 3600,
                is_dst: false,
                name: Cow::Borrowed("ZONE_B"),
            }),
        ],
    };

    #[test]
    fn just_one_first() {
        assert_eq!(ONE.find_with_surroundings(1184000000), Surroundings {
            previous: Some((
                &FixedTimespan {
                    offset: 0,
                    is_dst: false,
                    name: Cow::Borrowed("ZONE_A"),
                },
                1174784400,
            )),
            current: &FixedTimespan {
                offset: 3600,
                is_dst: false,
                name: Cow::Borrowed("ZONE_B"),
            },
            next: None,
        });
    }

    #[test]
    fn just_one_other() {
        assert_eq!(ONE.find_with_surroundings(1174000000), Surroundings {
            previous: None,
            current: &FixedTimespan {
                offset: 0,
                is_dst: false,
                name: Cow::Borrowed("ZONE_A"),
            },
            next: Some(&(
                1174784400,
                FixedTimespan {
                    offset: 3600,
                    is_dst: false,
                    name: Cow::Borrowed("ZONE_B"),
                },
            )),
        })
    }

    const MANY: FixedTimespanSet<'static> = FixedTimespanSet {
        first: FixedTimespan {
            offset: 0,
            is_dst: false,
            name: Cow::Borrowed("ZONE_A"),
        },
        rest: &[
            (1174784400, FixedTimespan {
                offset: 3600,
                is_dst: false,
                name: Cow::Borrowed("ZONE_B"),
            }),
            (1193533200, FixedTimespan {
                offset: 0,
                is_dst: false,
                name: Cow::Borrowed("ZONE_C"),
            }),
        ],
    };

    #[test]
    fn multiple_second() {
        assert_eq!(MANY.find_with_surroundings(1184000000), Surroundings {
            previous: Some((
                &FixedTimespan {
                    offset: 0,
                    is_dst: false,
                    name: Cow::Borrowed("ZONE_A"),
                },
                1174784400,
            )),
            current: &FixedTimespan {
                offset: 3600,
                is_dst: false,
                name: Cow::Borrowed("ZONE_B"),
            },
            next: Some(&(
                1193533200,
                FixedTimespan {
                    offset: 0,
                    is_dst: false,
                    name: Cow::Borrowed("ZONE_C"),
                }
            )),
        });
    }

    #[test]
    fn multiple_last() {
        assert_eq!(MANY.find_with_surroundings(1200000000), Surroundings {
            previous: Some((
                &FixedTimespan {
                    offset: 3600,
                    is_dst: false,
                    name: Cow::Borrowed("ZONE_B"),
                },
                1193533200,
            )),
            current: &FixedTimespan {
                offset: 0,
                is_dst: false,
                name: Cow::Borrowed("ZONE_C"),
            },
            next: None,
        });
    }
}
