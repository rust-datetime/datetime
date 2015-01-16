extern crate libc;

#[cfg(any(target_os = "macos", target_os = "ios"))]
use self::libc::{c_int, timeval, timezone};
use std::ptr::null_mut;

#[cfg(any(target_os = "macos", target_os = "ios"))]
extern {
    fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> c_int;
}

/// Return the current time, as a number of seconds and milliseconds.
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub unsafe fn now() -> (i64, i16) {
    let mut tv = timeval { tv_sec: 0, tv_usec: 0 };
    gettimeofday(&mut tv, null_mut());
    (tv.tv_sec as i64, (tv.tv_usec / 1000) as i16)
}
