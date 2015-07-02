extern crate datetime;
use datetime::local::*;
use datetime::parse::*;

extern crate rustc_serialize;
use rustc_serialize::json::Json;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;


#[test]
fn fwd_new_equivalent(){
    let mut count = 0;

    let path = Path::new("./tests/examples.json");
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
        Ok(file) => file
    };

    let mut s = String::new();
    let file_content = match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, Error::description(&why)),
        Ok(_) => s
    };


    if let Json::Array(examples) = Json::from_str(&file_content).unwrap(){
        let len = examples.len();
        for example in examples{
            if let Json::Array(ref example) = example{
                let ex0 = example[0].as_string().unwrap();
                let ex1 = example[1].as_array().unwrap();
                let ex2 = example[0].as_string().unwrap();
                let ex3 = example[3].as_array().unwrap();
                let (wyear, week, wday) = ( ex1[0].as_i64().unwrap(), ex1[1].as_i64().unwrap(), ex1[2].as_i64().unwrap());
                let (year, month, day) = ( ex3[0].as_i64().unwrap(), ex3[1].as_i64().unwrap(), ex3[2].as_i64().unwrap());

                let date_fwd_s = parse_iso_8601_date(&ex0);
                let date_fwd_t = LocalDate::from_weekday(wyear, week, wday);
                let date_new_s = parse_iso_8601_date(&ex2);
                let date_new_t = LocalDate::new(year, month as i8, day as i8);

                if date_fwd_t == date_new_t
                && date_fwd_s == date_new_s {count +=1;}
                else{ println!("{:?}\n{:?}\n", date_fwd_t , date_new_t);
                      println!("{:?}\n{:?}\n", date_fwd_s , date_new_s); }
            }
        }
        println!("{}% ok", (count as f64)/(len as f64)* 100f64);
    }
}

#[test]

fn time(){
    let strings = [
        // {{{
        ("2001-02-03T04:05:06+07:00",    Some((2001,02,03, 04,05,06,00))),//,07,00)),
        ("20010203T040506+0700",         Some((2001,02,03, 04,05,06,00))),//,07,00)),
        ("2001-W05-6T04",                Some((2001,02,03, 04,00,00,00))),//,07,00)),
        ("2002-W05-6T04",                Some((2002,02,02, 04,00,00,00))),//,07,00)),
        ("2003-W05-6T04",                Some((2003,02,01, 04,00,00,00))),//,07,00)),
        ("2001-W05-6T04:05",             Some((2001,02,03, 04,05,00,00))),//,07,00)),
        ("2001-W05-6T04:05:06",          Some((2001,02,03, 04,05,06,00))),//,07,00)),
        ("2001-W05-6T04:05:06.1",        None),
        ("2001-W05-6T04:05:06.12",       None),
        ("2001-W05-6T04:05:06.123",      Some((2001,02,03, 04,05,06,123))),//,07,00)),
        ("2001-W05-6T04:05:06.1234",     None),
        ("2001-W05-6T04:05:06.12345",    None),
        ("2001-W05-6T04:05:06.12345Z",   None),
        ("2001-W05-6T04:05:06.123Z",     Some((2001,02,03, 04,05,06,123))),//,07,00)),
        ("2001-W05-6T04:05:06+07",       Some((2001,02,03, 04,05,06,00))),//,07,00)),
        ("2001-W05-6T04:05:06+07:00",    Some((2001,02,03, 04,05,06,00))),//,07,00)),
        ("2001-W05-6T04:05:06-07:00",    Some((2001,02,03, 04,05,06,00))),//,07,00)),
        ("2015-06-26TZ",                 None                     ),//,07,00)),  // wrong
        ("2015-06-26",                   Some((2015,06,26, 00,00,00,00))),//,07,00)),  // Date
        ("2015-06-26T22:57:09+00:00",    Some((2015,06,26, 22,57,09,00))),//,07,00)),  // Combined date and time in UTC
        ("2015-06-26T22:57:09Z+00:00",   None                     ),//,07,00)),  // wrong
        ("2015-06-26T22:57:09+Z00:00",   None                     ),//,07,00)),  // wrong
        ("2015-06-26T22:57:09Z00:00",    None                     ),//,07,00)),  // wrong
        ("2015-06-26T22:57:09Z",         Some((2015,06,26, 22,57,09,00))),//,07,00)),  //
        //("2015-W26",                     Some((2015,))),  // Week
        ("2015-W26-5",                   Some((2015,06,26, 00,00,00,00))),  // Date with week number
      //("2015-177",                      true)   // Ordinal date
        // }}}
    ];
    for tup in strings.iter(){
        let string  = tup.0;
        let known = tup.1;
        let parsed = parse_iso_8601(&string).map(|d| (
            d.year(), d.month() as i32, d.day(),
            d.hour(), d.minute(), d.second(), d.millisecond()
        ));
        assert_eq!(parsed,known);
}
}

