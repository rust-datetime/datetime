#![crate_name = "datetime"]
#![crate_type = "dylib"]
#![feature(libc, plugin)]
#![plugin(regex_macros)]

extern crate locale;
extern crate num;
extern crate pad;
extern crate regex;
extern crate tz;

mod now;
mod parse;
pub mod zoned;
pub mod local;
pub mod instant;
pub mod duration;
pub mod format;
