extern crate datetime;
use datetime::zone::{StaticTimeZone, FixedTimespanSet, FixedTimespan, TimeZoneSource, TimeZone};
use datetime::{LocalDateTime, LocalDate, LocalTime, Month, DatePiece, TimePiece};
use std::borrow::Cow;


const TEST_ZONESET: &'static StaticTimeZone<'static> = &StaticTimeZone {
    name: "Test Zoneset",
    fixed_timespans: FixedTimespanSet {
        first: FixedTimespan {
            offset: 0,
            is_dst: false,
            name: Cow::Borrowed("ZONE_A"),
        },
        rest: &[
            (1206838800, FixedTimespan {
                offset: 3600,
                is_dst: false,
                name: Cow::Borrowed("ZONE_B"),
            }),
            (1224982800, FixedTimespan {
                offset: 0,
                is_dst: false,
                name: Cow::Borrowed("ZONE_A"),
            }),
            (1238288400, FixedTimespan {
                offset: 3600,
                is_dst: false,
                name: Cow::Borrowed("ZONE_B"),
            }),
            (1256432400, FixedTimespan {
                offset: 0,
                is_dst: false,
                name: Cow::Borrowed("ZONE_A"),
            }),
            (1269738000, FixedTimespan {
                offset: 3600,
                is_dst: false,
                name: Cow::Borrowed("ZONE_B"),
            }),
            (1288486800, FixedTimespan {
                offset: 0,
                is_dst: false,
                name: Cow::Borrowed("ZONE_A"),
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

    let zone = TimeZone(TimeZoneSource::Static(TEST_ZONESET));
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

    let zone = TimeZone(TimeZoneSource::Static(TEST_ZONESET));
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

    let zone = TimeZone(TimeZoneSource::Static(TEST_ZONESET));
    let converted = zone.convert_local(test_date);
    assert!(converted.is_impossible(),
        "Local time {:?} should be impossible", converted);
}
