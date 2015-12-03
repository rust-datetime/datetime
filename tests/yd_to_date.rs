extern crate datetime;
use datetime::{LocalDate, Month};
use datetime::DatePiece;


#[test]
fn day_start_of_year() {
    let date = LocalDate::yd(2015, 1).unwrap();
    assert_eq!(2015, date.year());
    assert_eq!(Month::January, date.month());
    assert_eq!(1, date.day());
}


#[test]
fn from_yearday() {
    for date in vec![
        //LocalDate::ymd(1970, 01 , 01).unwrap(),
        LocalDate::ymd(1971, Month::from_one(01).unwrap(), 01).unwrap(),
        LocalDate::ymd(1973, Month::from_one(01).unwrap(), 01).unwrap(),
        LocalDate::ymd(1977, Month::from_one(01).unwrap(), 01).unwrap(),
        LocalDate::ymd(1989, Month::from_one(11).unwrap(), 10).unwrap(),
        LocalDate::ymd(1990, Month::from_one( 7).unwrap(),  8).unwrap(),
        LocalDate::ymd(2014, Month::from_one( 7).unwrap(), 13).unwrap(),
        LocalDate::ymd(2001, Month::from_one( 2).unwrap(), 03).unwrap(),
    ]{
        let new_date = LocalDate::yd(date.year(), date.yearday() as i64).unwrap();
        assert_eq!(new_date, date);
        assert!(LocalDate::yd(2002, 1).is_ok());

        assert_eq!(new_date.yearday(), date.yearday());
    }
}