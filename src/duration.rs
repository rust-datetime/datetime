//! Lengths of time on the timeline.

use std::ops::{Add, Sub, Mul};


/// A **duration** is a length of time on the timeline, irrespective of
/// time zone or calendar format, with millisecond precision.
#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub struct Duration {
    seconds: i64,
    milliseconds: i16,
}

impl Duration {

    /// Create a new zero-length duration.
    pub fn zero() -> Self {
        Self { seconds: 0, milliseconds: 0 }
    }

    /// Create a new duration that’s the given number of seconds long.
    pub fn of(seconds: i64) -> Self {
        Self { seconds, milliseconds: 0 }
    }

    /// Create a new duration that’s the given number of seconds and
    /// milliseconds long.
    pub fn of_ms(seconds: i64, milliseconds: i16) -> Self {
        assert!(milliseconds >= 0 && milliseconds <= 999);  // TODO: replace assert with returning Result
        Self { seconds, milliseconds }
    }

    /// Return the seconds and milliseconds portions of the duration as
    /// a 2-element tuple.
    pub fn lengths(&self) -> (i64, i16) {
        (self.seconds, self.milliseconds)
    }

    // I’ve done it like this instead of having separate seconds() and
    // milliseconds() functions, because I think there’s a danger that
    // people will think that milliseconds() returns the *total* length
    // in milliseconds, rather than just this particular portion. This
    // way, it’s clear that there are two separate values being returned.
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Add<Duration> for Duration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let ms = self.milliseconds + rhs.milliseconds;
        if ms >= 1000 {
            Self::of_ms(self.seconds + rhs.seconds + 1, ms - 1000)
        }
        else {
            Self::of_ms(self.seconds + rhs.seconds, ms)
        }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Sub<Duration> for Duration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let ms = self.milliseconds - rhs.milliseconds;
        if ms < 0 {
            Self::of_ms(self.seconds - rhs.seconds - 1, ms + 1000)
        }
        else {
            Self::of_ms(self.seconds - rhs.seconds, ms)
        }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Mul<i64> for Duration {
    type Output = Self;

    fn mul(self, amount: i64) -> Self {
        let ms = self.milliseconds as i64 * amount;
        Self::of_ms(self.seconds * amount + ms / 1000, (ms % 1000) as i16)
    }
}
