#![crate_name = "datetime"]
#![crate_type = "dylib"]
#![feature(globs, phase)]

extern crate regex;

mod now;
mod parse;
pub mod local;
