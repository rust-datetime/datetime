#[phase(plugin)]
extern crate regex_macros;

pub fn parse_iso_ymd(input: &str) -> Option<(i64, i8, i8)> {
    match regex!(r"^(\d{4})-(\d{2})-(\d{2})$").captures(input) {
        None => None,
        Some(caps) => {
            Some((caps.at(1).unwrap().parse().unwrap(),
                  caps.at(2).unwrap().parse().unwrap(),
                  caps.at(3).unwrap().parse().unwrap()))
        },
    }
}

#[cfg(test)]
mod test {
    pub use super::parse_iso_ymd;

    #[test]
    fn date() {
        let date = parse_iso_ymd("1985-04-12");
        assert_eq!(date, Some((1985, 4, 12)));
    }

    #[test]
    fn fail() {
        let date = parse_iso_ymd("");
        assert_eq!(date, None)
    }
}

// 2014-12-25
// Combined date and time in UTC:	2014-12-25T02:56:40+00:00, 2014-12-25T02:56:40Z
// Week:	2014-W52
// Date with week number:	2014-W52-4
// Ordinal date:	2014-359
