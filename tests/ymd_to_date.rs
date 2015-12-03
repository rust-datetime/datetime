extern crate datetime;
use datetime::{LocalDate, Month};
use datetime::{DatePiece};


#[test]
fn the_distant_past() {
    let date = LocalDate::ymd(7, Month::April, 1).unwrap();

    assert_eq!(date.year(),  7);
    assert_eq!(date.month(), Month::April);
    assert_eq!(date.day(),   1);
}


#[test]
fn the_distant_present() {
    let date = LocalDate::ymd(2015, Month::January, 16).unwrap();

    assert_eq!(date.year(),  2015);
    assert_eq!(date.month(), Month::January);
    assert_eq!(date.day(),   16);
}


#[test]
fn the_distant_future() {
    let date = LocalDate::ymd(1048576, Month::October, 13).unwrap();

    assert_eq!(date.year(), 1048576);
    assert_eq!(date.month(), Month::October);
    assert_eq!(date.day(), 13);
}
