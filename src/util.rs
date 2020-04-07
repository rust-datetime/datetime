//! Misc stuff.

use std::ops::Range;


// TODO: replace this with the `range_contains` feature when it’s OK to use

pub(crate) trait RangeExt {

    /// Returns whether this value exists within the given range of values.
    fn is_within(&self, range: Range<Self>) -> bool where Self: Sized;
}

// Define RangeExt on *anything* that can be compared, though it’s only
// really ever used for numeric ranges...

impl<T> RangeExt for T where T: PartialOrd<T> {
    fn is_within(&self, range: Range<Self>) -> bool {
        *self >= range.start && *self < range.end
    }
}
