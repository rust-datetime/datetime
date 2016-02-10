extern crate datetime;
use datetime::{LocalDate, Month};
use datetime::DatePiece;


#[test]
fn start_of_year_day() {
    let date = LocalDate::ymd(2015, Month::January, 1).unwrap();
    assert_eq!(date.yearday(), 1);
}


#[test]
fn end_of_year_day() {
    let date = LocalDate::ymd(2015, Month::December, 31).unwrap();
    assert_eq!(date.yearday(), 365);
}


#[test]
fn end_of_leap_year_day() {
    let date = LocalDate::ymd(2016, Month::December, 31).unwrap();
    assert_eq!(date.yearday(), 366);
}


#[test]
fn yearday() {
    for year in 1..2058 {
        assert_eq!( LocalDate::ymd(year, Month::from_one(01).unwrap(), 31).unwrap().yearday() + 1,
                    LocalDate::ymd(year, Month::from_one(02).unwrap(), 01).unwrap().yearday());
        assert_eq!( LocalDate::ymd(year, Month::from_one(03).unwrap(), 31).unwrap().yearday() + 1,
                    LocalDate::ymd(year, Month::from_one(04).unwrap(), 01).unwrap().yearday());
        assert_eq!( LocalDate::ymd(year, Month::from_one(04).unwrap(), 30).unwrap().yearday() + 1,
                    LocalDate::ymd(year, Month::from_one(05).unwrap(), 01).unwrap().yearday());
        assert!(    LocalDate::ymd(year, Month::from_one(12).unwrap(), 31).unwrap().yearday() > 0);
    }
    assert_eq!( LocalDate::ymd(1600, Month::from_one(02).unwrap(), 29).unwrap().yearday() + 1, // leap year
                LocalDate::ymd(1600, Month::from_one(03).unwrap(), 01).unwrap().yearday());
    assert_eq!( LocalDate::ymd(1601, Month::from_one(02).unwrap(), 28).unwrap().yearday() + 1, // no leap year
                LocalDate::ymd(1601, Month::from_one(03).unwrap(), 01).unwrap().yearday());
}