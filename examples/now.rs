extern crate datetime;
extern crate locale;
use datetime::format::DateFormat;
use datetime::local::LocalDateTime;
use datetime::zoned::{VariableOffset, TimeZone};

use std::path::Path;
use std::env;

fn main() {
    let format = DateFormat::parse("{2>:D} {:M} {:Y}, {:h}:{02>:m}:{02>:s}").unwrap();
    let now = LocalDateTime::now();
    println!("It is {} in UTC", format.format(&now, &locale::Time::english()));

    let localtime = match VariableOffset::localtime() {
        Ok(t) => t,
        Err(e) => { println!("Error: {}", e); return },
    };

    let then = localtime.at(now);
    println!("It is {} in your local time zone", format.format(&then, &locale::Time::english()));

    for arg in env::args().skip(1) {
        let path = Path::new(&arg);
        let localtime = match VariableOffset::zoneinfo(&path) {
            Ok(t) => t,
            Err(e) => { println!("Error: {}", e); continue },
        };

        let then = localtime.at(now);
        println!("It is {} in {}", format.format(&then, &locale::Time::english()), path.file_name().unwrap().to_string_lossy());
    }
}
