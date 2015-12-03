extern crate datetime;
use datetime::{LocalDateTime, Duration};


#[test]
fn addition() {
    let date = LocalDateTime::at(10000);
    assert_eq!(LocalDateTime::at(10001), date + Duration::of(1))
}

#[test]
fn subtraction() {
    let date = LocalDateTime::at(100000000);
    assert_eq!(LocalDateTime::at(99999999), date - Duration::of(1))
}
