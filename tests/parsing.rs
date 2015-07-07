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
fn iso_formats(){
    assert_eq!( parse_iso_8601( "2001-02-03T04:05:06+07:00").unwrap(), parse_iso_8601( "20010203T040506+0700").unwrap());
    assert_eq!( parse_iso_8601( "2001-02-03T04:05:06+07:00").unwrap(), parse_iso_8601( "2001-W05-6T04:05:06+07:00").unwrap());
    assert_eq!( parse_iso_8601( "20010203T040506+0700").unwrap(), parse_iso_8601( "2001-W05-6T04:05:06+07:00").unwrap());
}

fn open_test_file() -> String{

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
    file_content
}

#[test]
/// comprehensive test that compares
fn date_fromweekday_vs_new_vs_parse(){

    if let Json::Array(examples) = Json::from_str(&open_test_file()).unwrap(){
        for example in examples{
            if let Json::Array(ref example) = example{

                // reading fields from examples.json
                let ex0 = example[0].as_string().unwrap();
                let ex1 = example[1].as_array().unwrap();
                let ex2 = example[0].as_string().unwrap();
                let ex3 = example[3].as_array().unwrap();
                let (wyear, week, wday) = ( ex1[0].as_i64().unwrap(), ex1[1].as_i64().unwrap(), ex1[2].as_i64().unwrap());
                let (year, month, day) = ( ex3[0].as_i64().unwrap(), ex3[1].as_i64().unwrap(), ex3[2].as_i64().unwrap());


                // instantiating 4 equivalent date in 5 different ways
                let date_fwd_s = parse_iso_8601_date(&ex0);
                let date_fwd_t = LocalDate::from_weekday(wyear, week, wday);
                let date_new_s = parse_iso_8601_date(&ex2);
                let date_new_t = LocalDate::new(year, month as i8, day as i8);
                let date_parse = LocalDate::parse(&ex0);

                // 5 way comparison
                assert_eq!( date_fwd_t, date_new_t );
                assert_eq!( date_new_t, date_fwd_s );
                assert_eq!( date_fwd_s, date_new_s );
                assert_eq!( date_fwd_s, date_parse );

            }
        }
    }
}

#[test]
fn time_parse_vs_new(){
    let strings = [
        // {{{
        ("2001-02-03T04:05:06+07:00",    Some((2001,02,03, 04,05,06,00, 07,00))),
        ("20010203T040506+0700",         Some((2001,02,03, 04,05,06,00, 07,00))),
        ("2001-W05-6T04",                Some((2001,02,03, 04,00,00,00, 07,00))),
        ("2002-W05-6T04",                Some((2002,02,02, 04,00,00,00, 07,00))),
        ("2003-W05-6T04",                Some((2003,02,01, 04,00,00,00, 07,00))),
        ("2001-W05-6T04:05",             Some((2001,02,03, 04,05,00,00, 07,00))),
        ("2001-W05-6T04:05:06",          Some((2001,02,03, 04,05,06,00, 07,00))),
        ("2001-W05-6T04:05:06.1",        None),
        ("2001-W05-6T04:05:06.12",       None),
        ("2001-W05-6T04:05:06.123",      Some((2001,02,03, 04,05,06,123,07,00))),
        ("2001-W05-6T04:05:06.1234",     None),
        ("2001-W05-6T04:05:06.12345",    None),
        ("2001-W05-6T04:05:06.12345Z",   None),
        ("2001-w05-6t04:05:06.123z",     None),
        ("2001-W05-6T04:05:06.123Z",     Some((2001,02,03, 04,05,06,123,07,00))),
        ("2001-W05-6T04:05:06+07",       Some((2001,02,03, 04,05,06,00, 07,00))),
        ("2001-W05-6T04:05:06+07:00",    Some((2001,02,03, 04,05,06,00, 07,00))),
        ("2001-W05-6T04:05:06-07:00",    Some((2001,02,03, 04,05,06,00, 07,00))),
        ("2015-06-26TZ",                 None),
        ("2015-06-26",                   Some((2015,06,26, 00,00,00,00, 07,00))),  // Date
        ("2015-06-26T22:57:09+00:00",    Some((2015,06,26, 22,57,09,00, 07,00))),  // Combined date and time in UTC
        ("2015-06-26T22:57:09Z+00:00",   None),
        ("2015-06-26T22:57:09+Z00:00",   None),
        ("2015-06-26T22:57:09Z00:00",    None),
        ("2015-06-26T22:57:09Z",         Some((2015,06,26, 22,57,09,00, 07,00))),  //
      //("2015-W26",                     Some((2015,))),  // Week
        ("2015-W26-5",                   Some((2015,06,26, 00,00,00,00, 00,00))),  // Date with week number
      //("2015-177",                     Some)   // Ordinal date
        // }}}
        ];

    for tup in strings.iter(){
        let string  = tup.0;

        let known = match tup.1{
            Some(known) => Some((known.0,known.1,known.2,known.3,known.4,known.5,known.6)),
            None => None };

        //datetime
        let parsed0 = parse_iso_8601(&string).map(|d| (
                d.year(), d.month() as i32, d.day(),
                d.hour(), d.minute(), d.second(), d.millisecond()));
        println!("{:?} {:?}", parsed0, known );
        assert_eq!(parsed0,known);

        let parsed1 = LocalDateTime::parse(&string).map(|d| (
                d.year(), d.month() as i32, d.day(),
                d.hour(), d.minute(), d.second(), d.millisecond()));
        assert_eq!(parsed1,known);

        // date and time
        if let Some((dstring,tstring)) = split_iso_8601(&string){

            let parsed0 = parse_iso_8601_date(&dstring);
            let parsed1 = LocalDate::parse(&dstring);
            if let Some(known) = tup.1{
                assert_eq!(parsed0,LocalDate::new(known.0, known.1 as i8, known.2));
                assert_eq!(parsed1,LocalDate::new(known.0, known.1 as i8, known.2));
            }

            let parsed0 = parse_iso_8601_time(&tstring);
            let parsed1 = LocalTime::parse(&tstring);
            if let Some(known) = tup.1{
                assert_eq!(parsed0,LocalTime::hms_ms(known.3, known.4, known.5, known.6 as i16));
                assert_eq!(parsed1,LocalTime::hms_ms(known.3, known.4, known.5, known.6 as i16));
            }
        }
    }
}

//#[test]
//fn zoned_time(){
//
//}
