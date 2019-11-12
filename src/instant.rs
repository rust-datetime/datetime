//! Exact points on a timeline.

use std::fmt;
use std::ops::{Add, Sub};

use system::sys_time;
use duration::Duration;


/// An **instant** is an exact point on the timeline, irrespective of time
/// zone or calendar format, with millisecond precision.
///
/// Internally, this is represented by a 64-bit integer of seconds, and a
/// 16-bit integer of milliseconds. This means that it will overflow (and thus
/// be unsuitable for) instants past GMT 15:30:08, Sunday 4th December,
/// 292,277,026,596 (yes, that’s a year)
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Instant {
    seconds: i64,
    milliseconds: i16,
}

impl Instant {

    /// Creates a new Instant set to the number of seconds since the Unix
    /// epoch, and zero milliseconds.
    pub fn at(seconds: i64) -> Self {
        Self::at_ms(seconds, 0)
    }

    /// Creates a new Instant set to the number of seconds since the
    /// Unix epoch, along with the number of milliseconds so far this
    /// second.
    pub fn at_ms(seconds: i64, milliseconds: i16) -> Self {
        Self { seconds, milliseconds }
    }

    /// Creates a new Instant set to the computer’s current time.
    #[cfg_attr(target_os = "redox", allow(unused_unsafe))]
    pub fn now() -> Self {
        let (seconds, milliseconds) = unsafe { sys_time() };
        Self { seconds, milliseconds }
    }

    /// Creates a new Instant set to the Unix epoch.
    pub fn at_epoch() -> Self {
        Self::at(0)
    }

    /// Returns the number of seconds at this instant
    pub fn seconds(&self) -> i64 {
        self.seconds
    }

    /// Returns the number of milliseconds at this instant
    pub fn milliseconds(&self) -> i16 {
        self.milliseconds
    }
}

impl fmt::Debug for Instant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Instant({}s/{}ms)", self.seconds, self.milliseconds)
    }
}

impl Add<Duration> for Instant {
    type Output = Self;

    fn add(self, duration: Duration) -> Self {
        let (seconds, milliseconds) = duration.lengths();
        Self {
            seconds: self.seconds + seconds,
            milliseconds: self.milliseconds + milliseconds,
        }
    }
}

impl Sub<Duration> for Instant {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self {
        let (seconds, milliseconds) = duration.lengths();
        Self {
            seconds: self.seconds - seconds,
            milliseconds: self.milliseconds - milliseconds,
        }
    }
}
