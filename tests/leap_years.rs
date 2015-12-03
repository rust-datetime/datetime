extern crate datetime;
use datetime::{Year};


#[test]
fn year_1600() {
    assert!(Year(1600).is_leap_year());
}

#[test]
fn year_1900() {
    assert!(Year(1900).is_leap_year() == false);
}

#[test]
fn year_2000() {
    assert!(Year(2000).is_leap_year());
}

#[test]
fn year_2038() {
    assert!(Year(2038).is_leap_year() == false);
}



