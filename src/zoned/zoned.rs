use duration::Duration;
use instant::Instant;
use local::{LocalDateTime, DatePiece, TimePiece, Month, Weekday};
use util::RangeExt;


#[derive(PartialEq, Debug, Clone)]
pub struct TimeZone<'a> {

    /// This zone's name in the zoneinfo database, such as "America/New_York".
    pub name: &'a str,

    pub transitions: FixedTimespanSet<'a>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct FixedTimespanSet<'a> {
    pub first: FixedTimespan<'a>,
    pub rest:  &'a [ (i64, FixedTimespan<'a>) ],
}

#[derive(PartialEq, Debug, Clone)]
pub struct FixedTimespan<'a> {
    pub offset:  i64,
    pub name:    &'a str,
}

impl<'a> FixedTimespanSet<'a> {
    pub fn find(&self, time: i64) -> &FixedTimespan {
        match self.rest.iter().take_while(|t| t.0 < time).last() {
            None     => &self.first,
            Some(zd) => &zd.1,
        }
    }

    fn find_with_surroundings(&self, time: i64) -> Surroundings {
        if let Some((position, _)) = self.rest.iter().enumerate().take_while(|&(_, t)| t.0 < time).last() {
            // There's a matching time in the 'rest' list, so return that
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
            // If there's no matching time in the 'rest' list, it must be
            // the 'first' one.
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

/// The "type" of time that a time is.
///
/// A time may be followed with a letter, signifying what 'type'
/// of time the timestamp is:
///
/// - **w** for "wall clock" time (the default),
/// - **s** for local standard time,
/// - **u** or **g** or **z** for universal time.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TimeType {

    /// Wall-clock time.
    Wall,

    /// Standard Time.
    Standard,

    /// Universal Co-ordinated Time.
    UTC,
}

impl<'a> TimeZone<'a> {
    pub fn offset(&self, datetime: LocalDateTime) -> i64 {
        let unix_timestamp = datetime.to_instant().seconds();
        self.transitions.find(unix_timestamp).offset
    }

    pub fn name(&self, datetime: LocalDateTime) -> &str {
        let unix_timestamp = datetime.to_instant().seconds();
        self.transitions.find(unix_timestamp).name
    }

    pub fn is_fixed(&self) -> bool {
        self.transitions.rest.is_empty()
    }

    pub fn convert_local(&self, local: LocalDateTime) -> LocalTimes {
        let unix_timestamp = local.to_instant().seconds();

        let zonify = |offset| ZonedDateTime {
            adjusted: local,
            current_offset: offset,
            time_zone: self.clone(),
        };

        let timespans = self.transitions.find_with_surroundings(unix_timestamp);

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

    pub fn to_zoned(&self, datetime: LocalDateTime) -> LocalDateTime {
        datetime + Duration::of(self.offset(datetime))
    }
}

#[derive(Debug)]
pub enum LocalTimes<'a> {
    Impossible,

    Precise(ZonedDateTime<'a>),

    Ambiguous { earlier: ZonedDateTime<'a>, later: ZonedDateTime<'a> },
}

impl<'a> LocalTimes<'a> {
    pub fn unwrap_precise(self) -> ZonedDateTime<'a> {
        match self {
            LocalTimes::Precise(p)        => p,
            LocalTimes::Impossible        => panic!("called `LocalTimes::unwrap()` on an `Impossible` value"),
            LocalTimes::Ambiguous { .. }  => panic!("called `LocalTimes::unwrap()` on an `Ambiguous` value: {:?}", self),
        }
    }

    pub fn is_impossible(&self) -> bool {
        match *self {
            LocalTimes::Impossible => true,
            _                      => false,
        }
    }

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
    time_zone: TimeZone<'a>,
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


#[cfg(test)]
mod test {
    pub use super::*;
    pub use local::*;

    mod zoneset {
        use super::*;
        use super::super::Surroundings;

        const NONE: FixedTimespanSet<'static> = FixedTimespanSet {
            first: FixedTimespan {
                offset: 0,
                name: "ZONE_A",
            },
            rest: &[],
        };

        #[test]
        fn empty() {
            assert_eq!(NONE.find_with_surroundings(1184000000), Surroundings {
                previous: None,
                current: &FixedTimespan {
                    offset: 0,
                    name: "ZONE_A",
                },
                next: None,
            })
        }

        const ONE: FixedTimespanSet<'static> = FixedTimespanSet {
            first: FixedTimespan {
                offset: 0,
                name: "ZONE_A",
            },
            rest: &[
                (1174784400, FixedTimespan {
                    offset: 3600,
                    name: "ZONE_B",
                }),
            ],
        };

        #[test]
        fn just_one_first() {
            assert_eq!(ONE.find_with_surroundings(1184000000), Surroundings {
                previous: Some((
                    &FixedTimespan {
                        offset: 0,
                        name: "ZONE_A",
                    },
                    1174784400,
                )),
                current: &FixedTimespan {
                    offset: 3600,
                    name: "ZONE_B",
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
                    name: "ZONE_A",
                },
                next: Some(&(
                    1174784400,
                    FixedTimespan {
                        offset: 3600,
                        name: "ZONE_B",
                    },
                )),
            })
        }

        const MANY: FixedTimespanSet<'static> = FixedTimespanSet {
            first: FixedTimespan {
                offset: 0,
                name: "ZONE_A",
            },
            rest: &[
                (1174784400, FixedTimespan {
                    offset: 3600,
                    name: "ZONE_B",
                }),
                (1193533200, FixedTimespan {
                    offset: 0,
                    name: "ZONE_C",
                }),
            ],
        };

        #[test]
        fn multiple_second() {
            assert_eq!(MANY.find_with_surroundings(1184000000), Surroundings {
                previous: Some((
                    &FixedTimespan {
                        offset: 0,
                        name: "ZONE_A",
                    },
                    1174784400,
                )),
                current: &FixedTimespan {
                    offset: 3600,
                    name: "ZONE_B",
                },
                next: Some(&(
                    1193533200,
                    FixedTimespan {
                        offset: 0,
                        name: "ZONE_C",
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
                        name: "ZONE_B",
                    },
                    1193533200,
                )),
                current: &FixedTimespan {
                    offset: 0,
                    name: "ZONE_C",
                },
                next: None,
            });
        }
    }

    const TEST_ZONESET: TimeZone<'static> = TimeZone {
        name: "Test Zoneset",
        transitions: FixedTimespanSet {
            first: FixedTimespan {
                offset: 0,
                name: "ZONE_A",
            },
            rest: &[
                (1206838800, FixedTimespan {
                    offset: 3600,
                    name: "ZONE_B",
                }),
                (1224982800, FixedTimespan {
                    offset: 0,
                    name: "ZONE_A",
                }),
                (1238288400, FixedTimespan {
                    offset: 3600,
                    name: "ZONE_B",
                }),
                (1256432400, FixedTimespan {
                    offset: 0,
                    name: "ZONE_A",
                }),
                (1269738000, FixedTimespan {
                    offset: 3600,
                    name: "ZONE_B",
                }),
                (1288486800, FixedTimespan {
                    offset: 0,
                    name: "ZONE_A",
                }),
            ]
        }
    };

    #[test]
    fn construction() {
        let test_date = LocalDateTime::new(
            LocalDate::ymd(2010, Month::June, 9).unwrap(),
            LocalTime::hms(15, 15, 0).unwrap(),
        );

        let zone = TEST_ZONESET;
        assert_eq!(zone.offset(test_date), 3600);

        let zoned_date = zone.convert_local(test_date).unwrap_precise();
        assert_eq!(zoned_date.year(), 2010);
        assert_eq!(zoned_date.hour(), 15);

        let instant = LocalDateTime::new(
            LocalDate::ymd(2010, Month::June, 9).unwrap(),
            LocalTime::hms(14, 15, 0).unwrap(),
        ).to_instant();

        assert_eq!(instant, zoned_date.to_instant());
    }

    #[test]
    fn ambiguity() {
        let test_date = LocalDateTime::new(
            LocalDate::ymd(2010, Month::October, 31).unwrap(),
            LocalTime::hms(1, 15, 0).unwrap(),
        );

        let zone = TEST_ZONESET;
        let converted = zone.convert_local(test_date);
        assert!(converted.is_ambiguous(),
            "Local time {:?} should be ambiguous", converted);
    }

    #[test]
    fn impossible() {
        let test_date = LocalDateTime::new(
            LocalDate::ymd(2010, Month::March, 28).unwrap(),
            LocalTime::hms(1, 15, 0).unwrap(),
        );

        let zone = TEST_ZONESET;
        let converted = zone.convert_local(test_date);
        assert!(converted.is_impossible(),
            "Local time {:?} should be impossible", converted);
    }
}
