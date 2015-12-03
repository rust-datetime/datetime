extern crate datetime;
pub use datetime::{YearMonth, Year};

mod months {
    use super::*;
    use datetime::Month::*;

    #[test]
    fn range_full() {
        let year = Year(2013);
        let months: Vec<_> = year.months(..).collect();
        assert_eq!(months, vec![
            year.month(January),
            year.month(February),
            year.month(March),
            year.month(April),
            year.month(May),
            year.month(June),
            year.month(July),
            year.month(August),
            year.month(September),
            year.month(October),
            year.month(November),
            year.month(December),
        ]);
    }

    #[test]
    fn range_from() {
        let year = Year(2013);
        let months: Vec<_> = year.months(July..).collect();
        assert_eq!(months, vec![
            year.month(July),
            year.month(August),
            year.month(September),
            year.month(October),
            year.month(November),
            year.month(December),
        ]);
    }

    #[test]
    fn range_to() {
        let year = Year(2013);
        let months: Vec<_> = year.months(..July).collect();
        assert_eq!(months, vec![
            year.month(January),
            year.month(February),
            year.month(March),
            year.month(April),
            year.month(May),
            year.month(June),
        ]);
    }

    #[test]
    fn range() {
        let year = Year(2013);
        let months: Vec<_> = year.months(April..July).collect();
        assert_eq!(months, vec![
            year.month(April),
            year.month(May),
            year.month(June),
        ]);
    }

    #[test]
    fn range_empty() {
        let year = Year(2013);
        let months: Vec<_> = year.months(August..August).collect();
        assert!(months.is_empty());
    }

    #[test]
    fn range_singular() {
        let year = Year(2013);
        let months = year.month(April);
        assert_eq!(months, year.month(April));
    }
}

mod days {
    use super::*;
    use datetime::LocalDate;
    use datetime::Month::*;

    #[test]
    fn range_full() {
        let year = Year(2013).month(February);
        let days: Vec<_> = year.days(..).collect();
        let results: Vec<_> = (1..29).map(|d| LocalDate::ymd(2013, February, d).unwrap()).collect();
        assert_eq!(days, results);
    }

    #[test]
    fn range_full_leap_year() {
        let year = Year(2000).month(February);
        let days: Vec<_> = year.days(..).collect();
        let results: Vec<_> = (1..30).map(|d| LocalDate::ymd(2000, February, d).unwrap()).collect();
        assert_eq!(days, results);
    }

    #[test]
    fn range() {
        let year = Year(2008).month(March);
        let days: Vec<_> = year.days(10..20).collect();
        let results: Vec<_> = (10..20).map(|d| LocalDate::ymd(2008, March, d).unwrap()).collect();
        assert_eq!(days, results);
    }

    #[test]
    fn just_for_one_day() {
        let day = Year(1066).month(October).day(14);
        assert_eq!(day, LocalDate::ymd(1066, October, 14));
    }
}

#[test]
fn entire_year() {
    let count = Year(1999).months(..)
                          .flat_map(|m| m.days(..))
                          .count();

    assert_eq!(count, 365);
}

#[test]
fn entire_leap_year() {
    let count = Year(2000).months(..)
                          .flat_map(|m| m.days(..))
                          .count();

    assert_eq!(count, 366);
}
