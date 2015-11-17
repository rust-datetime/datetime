//! Iterators through multiple datetimes.

use std::ops::{Range, RangeFrom, RangeTo, RangeFull};
use std::slice::Iter as SliceIter;

use cal::datetime::{LocalDate, Month, YMD};
use cal::datetime::Error as DateTimeError;


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Year(pub i64);

impl Year {
    pub fn is_leap_year(&self) -> bool {
        YMD { year: self.0, month: Month::January, day: 1 }
            .leap_year_calculations()
            .1
    }

    pub fn months<S: MonthSpan>(&self, span: S) -> YearMonths {
        YearMonths {
            year: self.clone(),
            iter: span.get_slice().iter(),
        }
    }

    pub fn month(&self, month: Month) -> YearMonth {
        YearMonth {
            year: self.clone(),
            month: month,
        }
    }
}


pub trait MonthSpan {
    fn get_slice(&self) -> &'static [Month];
}

impl MonthSpan for RangeFull {
    fn get_slice(&self) -> &'static [Month] {
        MONTHS
    }
}

impl MonthSpan for RangeFrom<Month> {
    fn get_slice(&self) -> &'static [Month] {
        &MONTHS[self.start.months_from_january() ..]
    }
}

impl MonthSpan for RangeTo<Month> {
    fn get_slice(&self) -> &'static [Month] {
        &MONTHS[.. self.end.months_from_january()]
    }
}

impl MonthSpan for Range<Month> {
    fn get_slice(&self) -> &'static [Month] {
        &MONTHS[self.start.months_from_january() .. self.end.months_from_january()]
    }
}

static MONTHS: &'static [Month] = &[
    Month::January,  Month::February,  Month::March,
    Month::April,    Month::May,       Month::June,
    Month::July,     Month::August,    Month::September,
    Month::October,  Month::November,  Month::December,
];


pub struct YearMonths {
    year: Year,
    iter: SliceIter<'static, Month>,
}

impl Iterator for YearMonths {
    type Item = YearMonth;

    fn next(&mut self) -> Option<YearMonth> {
        self.iter.next().map(|m| YearMonth {
            year: self.year,
            month: *m,
        })
    }
}

impl DoubleEndedIterator for YearMonths {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|m| YearMonth {
            year: self.year,
            month: *m,
        })
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct YearMonth {
    year: Year,
    month: Month,
}

impl YearMonth {
    pub fn day_count(&self) -> i8 {
        self.month.days_in_month(self.year.is_leap_year())
    }

    pub fn days<S: DaySpan>(&self, span: S) -> MonthDays {
        MonthDays {
            ym: self,
            range: span.get_range(self)
        }
    }

    pub fn day(&self, day: i8) -> Result<LocalDate, DateTimeError> {
        LocalDate::ymd(self.year.0, self.month, day)
    }
}

pub trait DaySpan {
    fn get_range(&self, ym: &YearMonth) -> Range<i8>;
}

impl DaySpan for RangeFull {
    fn get_range(&self, ym: &YearMonth) -> Range<i8> {
        1 .. ym.day_count() + 1
    }
}

impl DaySpan for RangeFrom<i8> {
    fn get_range(&self, ym: &YearMonth) -> Range<i8> {
        self.start .. ym.day_count() + 1
    }
}

impl DaySpan for RangeTo<i8> {
    fn get_range(&self, _ym: &YearMonth) -> Range<i8> {
        1 .. self.end
    }
}

impl DaySpan for Range<i8> {
    fn get_range(&self, _ym: &YearMonth) -> Range<i8> {
        self.clone()
    }
}




#[derive(PartialEq, Debug)]
pub struct MonthDays<'ym> {
    ym: &'ym YearMonth,
    range: Range<i8>,
}

impl<'ym> Iterator for MonthDays<'ym> {
    type Item = LocalDate;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().and_then(|d| LocalDate::ymd(self.ym.year.0, self.ym.month, d).ok())
    }
}

impl<'ym> DoubleEndedIterator for MonthDays<'ym> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range.next_back().and_then(|d| LocalDate::ymd(self.ym.year.0, self.ym.month, d).ok())
    }
}

#[cfg(test)]
mod test {
    pub use super::*;

    mod months {
        use super::*;
        use cal::datetime::Month::*;

        #[test]
        fn range_full() {
            let year = Year(2013);
            let months: Vec<_> = year.months(..).collect();
            assert_eq!(months, vec![
                YearMonth { year: year, month: January },
                YearMonth { year: year, month: February },
                YearMonth { year: year, month: March },
                YearMonth { year: year, month: April },
                YearMonth { year: year, month: May },
                YearMonth { year: year, month: June },
                YearMonth { year: year, month: July },
                YearMonth { year: year, month: August },
                YearMonth { year: year, month: September },
                YearMonth { year: year, month: October },
                YearMonth { year: year, month: November },
                YearMonth { year: year, month: December },
            ]);
        }

        #[test]
        fn range_from() {
            let year = Year(2013);
            let months: Vec<_> = year.months(July..).collect();
            assert_eq!(months, vec![
                YearMonth { year: year, month: July },
                YearMonth { year: year, month: August },
                YearMonth { year: year, month: September },
                YearMonth { year: year, month: October },
                YearMonth { year: year, month: November },
                YearMonth { year: year, month: December },
            ]);
        }

        #[test]
        fn range_to() {
            let year = Year(2013);
            let months: Vec<_> = year.months(..July).collect();
            assert_eq!(months, vec![
                YearMonth { year: year, month: January },
                YearMonth { year: year, month: February },
                YearMonth { year: year, month: March },
                YearMonth { year: year, month: April },
                YearMonth { year: year, month: May },
                YearMonth { year: year, month: June },
            ]);
        }

        #[test]
        fn range() {
            let year = Year(2013);
            let months: Vec<_> = year.months(April..July).collect();
            assert_eq!(months, vec![
                YearMonth { year: year, month: April },
                YearMonth { year: year, month: May },
                YearMonth { year: year, month: June },
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
            assert_eq!(months, YearMonth { year: year, month: April });
        }
    }

    mod days {
        use super::*;
        use cal::datetime::LocalDate;
        use cal::datetime::Month::*;

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
}
