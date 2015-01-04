#![crate_name = "datetime"]
#![crate_type = "dylib"]
#![feature(associated_types, default_type_params, globs, phase)]

extern crate regex;

mod now;
mod parse;
pub mod local;
pub mod instant;
pub mod duration;
