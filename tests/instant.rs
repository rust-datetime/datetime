extern crate datetime;
use datetime::Instant;


#[test]
fn seconds() {
    assert_eq!(Instant::at(3), Instant::at_ms(3, 0))
}

#[test]
fn milliseconds() {
    assert_eq!(Instant::at_ms(3, 333).milliseconds(), 333)
}

#[test]
fn epoch() {
    assert_eq!(Instant::at_epoch().seconds(), 0)
}

#[test]
fn sanity() {
    // Test that the system call has worked at all.
    // If this fails then you have gone back in time, or something?
    assert!(Instant::now().seconds() != 0)
}
