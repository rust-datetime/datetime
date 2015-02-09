extern crate libc;

#[cfg(any(target_os = "macos", target_os = "ios"))]
extern {
    fn gettimeofday(tp: *mut libc::timeval, tzp: *mut libc::timezone) -> libc::c_int;
}

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "ios")))]
extern {
    fn clock_gettime(clk_id: libc::c_int, tp: *mut libc::timespec) -> libc::c_int;
}

/// Return the current time, as a number of seconds and milliseconds.
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub unsafe fn now() -> (i64, i16) {
    use std::ptr::null_mut;

    let mut tv = libc::timeval { tv_sec: 0, tv_usec: 0 };
    gettimeofday(&mut tv, null_mut());
    (tv.tv_sec as i64, (tv.tv_usec / 1000) as i16)
}

/// Return the current time, as a number of seconds and milliseconds.
#[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
pub unsafe fn now() -> (i64, i16) {
    let mut tv = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    clock_gettime(libc::CLOCK_REALTIME, &mut tv);
    (tv.tv_sec as i64, (tv.tv_nsec / 1000) as i16)
}
