#![crate_name = "datetime"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
//#![warn(missing_docs)]

#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

extern crate locale;
extern crate libc;
extern crate num;
extern crate pad;
extern crate iso8601;

#[cfg(windows)] extern crate kernel32;
#[cfg(windows)] extern crate winapi;


mod cal;
pub use cal::{DatePiece, TimePiece};
pub use cal::datetime::{LocalDate, LocalTime, LocalDateTime, Month, Weekday, Year, YearMonth};
pub use cal::fmt::custom as fmt;
pub use cal::fmt::ISO;  // TODO: replace this with just a 'fmt' import
pub use cal::offset::{Offset, OffsetDateTime};
pub use cal::zone::{TimeZone, ZonedDateTime};
pub use cal::zone as zone;

pub use cal::convenience;

mod duration;
pub use duration::Duration;

mod instant;
pub use instant::Instant;

mod system;
pub use system::sys_timezone;

mod util;
