use local::{LocalDate, LocalTime, LocalDateTime};

use regex::Regex;
pub fn parse_iso_ymd(input: &str) -> Option<(i64, i8, i8)> {
    match Regex::new(r"^(\d{4})-(\d{2})-(\d{2})$").unwrap().captures(input) {
        None => None,
        Some(caps) => {
            Some((caps.at(1).unwrap().parse().unwrap(),
                  caps.at(2).unwrap().parse().unwrap(),
                  caps.at(3).unwrap().parse().unwrap()))
        },
    }
}


/// Splits DateString, TimeString
///
/// for further parsing by `parse_iso_8601_date` and `parse_iso_8601_time`.
pub fn split_iso_8601(string:&str) -> Option<(String, String)>
{
    let split = Regex::new(r"^([^T]*)T?(.*)$").unwrap();
    if split.is_match(&string) {
        let caps = split.captures(&string).unwrap();
        if caps.len() > 1 {
            return Some( (caps.at(1).unwrap().into(), caps.at(2).unwrap().into()) );
        }
    }
    None
}

/// Parses a ISO 8601 strin into LocalDateTime Object.
pub fn parse_iso_8601(string:&str) -> Option<LocalDateTime>
{
    let (date_string, time_string) = split_iso_8601(string).unwrap();
    match (parse_iso_8601_date(&date_string), parse_iso_8601_time(&time_string)) {
        (Some(date),Some(time)) => return Some(LocalDateTime::from_date_time(date,time)),
        _ => None
    }
}


/// Parses ISO 8601 Date strings into LocalDate Object.
pub fn parse_iso_8601_date(string:&str) -> Option<LocalDate>
{
    let week = Regex::new(r"^(\d{4})-W(\d{2})-(\d{1})$").unwrap();
    let ymd  = Regex::new(r"^(\d{4})-?(\d{2})-?(\d{2})$").unwrap();

    if ymd.is_match(&string) {
        return ymd.captures(string).map(|caps|
        LocalDate::new(
            caps.at(1).unwrap().parse().unwrap(), // year
            caps.at(2).unwrap().parse().unwrap(), // month
            caps.at(3).unwrap().parse().unwrap(), // day
            ).unwrap());
    }

    if week.is_match(&string) {
        return week.captures(string).map(|caps|
        LocalDate::from_weekday(
            caps.at(1).unwrap().parse::<i64>().unwrap(),  // year
            caps.at(2).unwrap().parse::<i64>().unwrap(),  // week
            caps.at(3).unwrap().parse::<i64>().unwrap()   // weekday
            ).unwrap());
    }
    None
}

/// Parses ISO 8601 Date strings into LocalTime Object.
pub fn parse_iso_8601_time(string:&str) -> Option<LocalTime>
{
    let exp = Regex::new(r"^(\d{2}):?(\d{2})?:?(?:(\d{2})\.?((?:\d{1,9}))?)?(?:([+-]\d\d)?:?(\d\d)?|(Z))?$").unwrap();
    if exp.is_match(&string) {
        let tup = exp.captures(string).map(|caps| (
                caps.at(1).unwrap_or("00").parse::<i8>().unwrap(), // HH
                caps.at(2).unwrap_or("00").parse::<i8>().unwrap(), // MM
                caps.at(3).unwrap_or("00").parse::<i8>().unwrap(), // SS
                caps.at(4).unwrap_or("00").parse::<i32>().unwrap(), // MS
                caps.at(5).unwrap_or("+00").trim_matches('+').parse::<i32>().unwrap(), // ZH
                caps.at(6).unwrap_or("00").parse::<i32>().unwrap(), // ZM
                caps.at(7).unwrap_or("_"), // "Z"
                caps.at(0).unwrap(), // All
                caps.at(4).unwrap_or("").len(),
                )).unwrap();

        if tup.8%3 != 0 {return None}

        return LocalTime::hms_ms(tup.0,tup.1,tup.2,tup.3 as i16);
    }
    if string.len() == 0 {
        return Some(LocalTime::hms(0,0,0).unwrap());
    }
    None
}

#[cfg(test)]
mod test {
    pub use super::parse_iso_ymd;
    pub use super::parse_iso_8601_date;
    pub use local::LocalDate;

    #[test]
    fn date() {
        let date = parse_iso_ymd("1985-04-12");
        assert_eq!(date, Some((1985, 4, 12)));
        let date = parse_iso_8601_date("1985-04-12");
        assert_eq!(date, LocalDate::new(1985, 4, 12));
    }

    #[test]
    fn fail() {
        let date = parse_iso_ymd("");
        assert_eq!(date, None);
        let date = parse_iso_8601_date("");
        assert_eq!(date, None);
    }
}

// 2014-12-25
// Combined date and time in UTC:   2014-12-25T02:56:40+00:00, 2014-12-25T02:56:40Z
// Week:    2014-W52
// Date with week number:   2014-W52-4
// Ordinal date:    2014-359
