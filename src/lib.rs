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
extern crate regex;
extern crate tz;

#[macro_use]
extern crate lazy_static;

mod now;
pub mod parse;
pub mod zoned;
pub mod local;
pub mod instant;
pub mod duration;
pub mod format;
mod util;
