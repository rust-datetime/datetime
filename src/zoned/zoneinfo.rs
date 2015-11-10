use local::LocalDateTime;
use util::RangeExt;
use zoned::{TimeZone, LocalTimes, ZonedDateTime};


#[derive(PartialEq, Debug, Clone)]
pub struct Zone<'a> {

    /// This zone's name in the zoneinfo database, such as "America/New_York".
    pub name: &'a str,

    pub transitions: ZoneSet<'a>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ZoneSet<'a> {
    pub first: ZoneDetails<'a>,
    pub rest:  &'a [ (i64, ZoneDetails<'a>) ],
}

#[derive(PartialEq, Debug, Clone)]
pub struct ZoneDetails<'a> {
    pub offset:  i64,
    pub name:    &'a str,
}

impl<'a> ZoneSet<'a> {
    pub fn find(&self, time: i64) -> &ZoneDetails {
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
    previous:  Option<(&'a ZoneDetails<'a>, i64)>,
    current:   &'a ZoneDetails<'a>,
    next:      Option<&'a (i64, ZoneDetails<'a>)>,
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

impl<'a> TimeZone<'a> for Zone<'a> {
    fn offset(&self, datetime: LocalDateTime) -> i64 {
        let unix_timestamp = datetime.to_instant().seconds();
        self.transitions.find(unix_timestamp).offset
    }

    fn name(&self, datetime: LocalDateTime) -> &str {
        let unix_timestamp = datetime.to_instant().seconds();
        self.transitions.find(unix_timestamp).name
    }

    fn is_fixed(&self) -> bool {
        self.transitions.rest.is_empty()
    }

    fn convert_local(&self, local: LocalDateTime) -> LocalTimes {
        let unix_timestamp = local.to_instant().seconds();

        let zonify = |offset| ZonedDateTime {
            adjusted: local,
            current_offset: offset,
            time_zone: Box::new(self.clone()) /*as Box<TimeZone + 'a>*/,
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
}


#[cfg(test)]
mod test {
    pub use super::*;
    pub use local::*;
    pub use zoned::{TimeZone, LocalTimes};

    mod zoneset {
        use super::*;
        use zoned::zoneinfo::Surroundings;

        const NONE: ZoneSet<'static> = ZoneSet {
            first: ZoneDetails {
                offset: 0,  // UTC offset 1500, DST offset 0
                name: "ZONE_A",
            },
            rest: &[],
        };

        #[test]
        fn empty() {
            assert_eq!(NONE.find_with_surroundings(1184000000), Surroundings {
                previous: None,
                current: &ZoneDetails {
                    offset: 0,
                    name: "ZONE_A",
                },
                next: None,
            })
        }

        const ONE: ZoneSet<'static> = ZoneSet {
            first: ZoneDetails {
                offset: 0,
                name: "ZONE_A",
            },
            rest: &[
                (1174784400, ZoneDetails {
                    offset: 3600,
                    name: "ZONE_B",
                }),
            ],
        };

        #[test]
        fn just_one_first() {
            assert_eq!(ONE.find_with_surroundings(1184000000), Surroundings {
                previous: Some((
                    &ZoneDetails {
                        offset: 0,
                        name: "ZONE_A",
                    },
                    1174784400,
                )),
                current: &ZoneDetails {
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
                current: &ZoneDetails {
                    offset: 0,
                    name: "ZONE_A",
                },
                next: Some(&(
                    1174784400,
                    ZoneDetails {
                        offset: 3600,
                        name: "ZONE_B",
                    },
                )),
            })
        }

        const MANY: ZoneSet<'static> = ZoneSet {
            first: ZoneDetails {
                offset: 0,
                name: "ZONE_A",
            },
            rest: &[
                (1174784400, ZoneDetails {
                    offset: 3600,
                    name: "ZONE_B",
                }),
                (1193533200, ZoneDetails {
                    offset: 0,
                    name: "ZONE_C",
                }),
            ],
        };

        #[test]
        fn multiple_second() {
            assert_eq!(MANY.find_with_surroundings(1184000000), Surroundings {
                previous: Some((
                    &ZoneDetails {
                        offset: 0,
                        name: "ZONE_A",
                    },
                    1174784400,
                )),
                current: &ZoneDetails {
                    offset: 3600,
                    name: "ZONE_B",
                },
                next: Some(&(
                    1193533200,
                    ZoneDetails {
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
                    &ZoneDetails {
                        offset: 3600,
                        name: "ZONE_B",
                    },
                    1193533200,
                )),
                current: &ZoneDetails {
                    offset: 0,
                    name: "ZONE_C",
                },
                next: None,
            });
        }
    }

    const TEST_ZONESET: Zone<'static> = Zone {
        name: "Test Zoneset",
        transitions: ZoneSet {
            first: ZoneDetails {
                offset: 0,
                name: "ZONE_A",
            },
            rest: &[
                (1206838800, ZoneDetails {
                    offset: 3600,
                    name: "ZONE_B",
                }),
                (1224982800, ZoneDetails {
                    offset: 0,
                    name: "ZONE_A",
                }),
                (1238288400, ZoneDetails {
                    offset: 3600,
                    name: "ZONE_B",
                }),
                (1256432400, ZoneDetails {
                    offset: 0,
                    name: "ZONE_A",
                }),
                (1269738000, ZoneDetails {
                    offset: 3600,
                    name: "ZONE_B",
                }),
                (1288486800, ZoneDetails {
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
