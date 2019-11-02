extern crate datetime;
use datetime::{LocalDateTime, Month};
use datetime::{DatePiece, TimePiece};


#[test]
fn a_long_time_ago() {
    let date = LocalDateTime::at(-1_000_000_000);

    assert_eq!(date.year(),   1938);
    assert_eq!(date.month(),  Month::April);
    assert_eq!(date.day(),    24);
    assert_eq!(date.hour(),   22);
    assert_eq!(date.minute(), 13);
    assert_eq!(date.second(), 20);
}


#[test]
fn unix_epoch() {
    let date = LocalDateTime::at(0);

    assert_eq!(date.year(),   1970);
    assert_eq!(date.month(),  Month::January);
    assert_eq!(date.day(),    01);
    assert_eq!(date.hour(),   00);
    assert_eq!(date.minute(), 00);
    assert_eq!(date.second(), 00);
}


#[test]
fn billennium() {
    let date = LocalDateTime::at(1_000_000_000);

    assert_eq!(date.year(),   2001);
    assert_eq!(date.month(),  Month::September);
    assert_eq!(date.day(),    09);
    assert_eq!(date.hour(),   01);
    assert_eq!(date.minute(), 46);
    assert_eq!(date.second(), 40);
}


#[test]
fn numbers() {
    let date = LocalDateTime::at(1_234_567_890);

    assert_eq!(date.year(),   2009);
    assert_eq!(date.month(),  Month::February);
    assert_eq!(date.day(),    13);
    assert_eq!(date.hour(),   23);
    assert_eq!(date.minute(), 31);
    assert_eq!(date.second(), 30);
}


#[test]
fn year_2038_problem() {
    let date = LocalDateTime::at(0x7FFF_FFFF);

    assert_eq!(date.year(),   2038);
    assert_eq!(date.month(),  Month::January);
    assert_eq!(date.day(),    19);
    assert_eq!(date.hour(),   03);
    assert_eq!(date.minute(), 14);
    assert_eq!(date.second(), 07);
}


#[test]
fn the_end_of_time() {
    let date = LocalDateTime::at(0x7FFF_FFFF_FFFF_FFFF);

    assert_eq!(date.year(),   292_277_026_596);
    assert_eq!(date.month(),  Month::December);
    assert_eq!(date.day(),    4);
    assert_eq!(date.hour(),   15);
    assert_eq!(date.minute(), 30);
    assert_eq!(date.second(), 07);
}


#[test]
fn just_some_date() {
    let date = LocalDateTime::at(146096 * 86400);

    assert_eq!(date.year(),   2369);
    assert_eq!(date.month(),  Month::December);
    assert_eq!(date.day(),    31);
    assert_eq!(date.hour(),   00);
    assert_eq!(date.minute(), 00);
    assert_eq!(date.second(), 00);
}

#[test]
fn leap_year_some_date() {
    let date = LocalDateTime::at(1459468800);

    assert_eq!(date.year(),   2016);
    assert_eq!(date.month(),  Month::April);
    assert_eq!(date.day(),    1);
    assert_eq!(date.hour(),   00);
    assert_eq!(date.minute(), 00);
    assert_eq!(date.second(), 00);
}

#[test]
fn leap_year_29th_feb() {
    let date = LocalDateTime::at(1456704000);

    assert_eq!(date.year(),   2016);
    assert_eq!(date.month(),  Month::February);
    assert_eq!(date.day(),    29);
    assert_eq!(date.hour(),   00);
    assert_eq!(date.minute(), 00);
    assert_eq!(date.second(), 00);
}
