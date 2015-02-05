#![crate_name = "datetime"]
#![crate_type = "dylib"]
#![feature(collections, core, io, libc, plugin)]

extern crate regex;

mod now;
mod parse;
pub mod local;
pub mod instant;
pub mod duration;
pub mod format;
