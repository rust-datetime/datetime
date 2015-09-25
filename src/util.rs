use std::ops::Range;


pub trait RangeExt {
    fn is_within(&self, range: Range<Self>) -> bool where Self: Sized;
}

impl<T> RangeExt for T where T: PartialOrd<T> {
    fn is_within(&self, range: Range<Self>) -> bool {
        *self >= range.start && *self < range.end
    }
}