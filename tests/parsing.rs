extern crate datetime;
use datetime::{LocalDateTime, Weekday, Month, LocalDate};

extern crate rustc_serialize;
use rustc_serialize::json::Json;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;


fn open_test_file() -> String {
    let path = Path::new("./tests/examples.json");
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file
    };

    let mut s = String::new();
    let file_content = match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_)    => s
    };

    file_content
}

#[test]
fn iso_formats(){
    assert_eq!(LocalDateTime::from_str("2001-02-03T04:05:06+07:00").unwrap(), LocalDateTime::from_str("20010203T040506+0700").unwrap());
    assert_eq!(LocalDateTime::from_str("2001-02-03T04:05:06+07:00").unwrap(), LocalDateTime::from_str("2001-W05-6T04:05:06+07:00").unwrap());
    assert_eq!(LocalDateTime::from_str("20010203T040506+0700").unwrap(), LocalDateTime::from_str("2001-W05-6T04:05:06+07:00").unwrap());
}


#[test]
/// comprehensive test that compares
fn date_fromweekday_vs_new_vs_parse() {
    if let Json::Array(examples) = Json::from_str(&open_test_file()).unwrap() {
        for example in examples {
            println!("{:?}", example);
            if let Json::Array(ref example) = example {

                // reading fields from examples.json
                let ex0 = example[0].as_string().unwrap();
                let ex1 = example[1].as_array().unwrap();
                let ex2 = example[0].as_string().unwrap();
                let ex3 = example[3].as_array().unwrap();
                let (wyear, week, wday) = (ex1[0].as_i64().unwrap(), ex1[1].as_i64().unwrap(), ex1[2].as_i64().unwrap());
                let (year, month, day)  = (ex3[0].as_i64().unwrap(), ex3[1].as_i64().unwrap(), ex3[2].as_i64().unwrap());

                // instantiating 4 equivalent date in 5 different ways
                println!("{:?}", ex0);
                let date_fwd_s = LocalDate::from_str(&ex0).unwrap();
                let wday =  Weekday::from_one(wday as i8).unwrap();
                let date_fwd_t = LocalDate::ywd(wyear, week, wday).unwrap();
                let date_new_s = LocalDate::from_str(&ex2).unwrap();
                let date_new_t = LocalDate::ymd(year, Month::from_one(month as i8).unwrap(), day as i8).unwrap();
                let date_parse = LocalDate::from_str(&ex0).unwrap();

                // 5 way comparison
                assert_eq!(date_fwd_t, date_new_t);
                assert_eq!(date_new_t, date_fwd_s);
                assert_eq!(date_fwd_s, date_new_s);
                assert_eq!(date_fwd_s, date_parse);
            }
        }
    }
}

