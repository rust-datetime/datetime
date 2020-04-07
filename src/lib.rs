#![warn(missing_copy_implementations)]
//#![warn(missing_docs)]
#![warn(nonstandard_style)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused)]

extern crate locale;
extern crate num_traits;
extern crate pad;
extern crate iso8601;

#[cfg(windows)] extern crate winapi;


mod cal;
pub use cal::{DatePiece, TimePiece};
pub use cal::datetime::{LocalDate, LocalTime, LocalDateTime, Month, Weekday, Year, YearMonth};
pub use cal::fmt::custom as fmt;
pub use cal::fmt::iso::ISO;  // TODO: replace this with just a 'fmt' import
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
