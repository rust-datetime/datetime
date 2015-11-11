use cal::datetime::{LocalDate, Month, YMD};

/// TODO: Make the YMD constructor able to use this
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Year(pub i64);

impl Year {
    pub fn is_leap_year(&self) -> bool {
        YMD { year: self.0, month: Month::January, day: 1 }
            .leap_year_calculations()
            .1
    }

    pub fn days_for_month(&self, month: Month) -> DaysForMonth {
        DaysForMonth {
            year: self,
            month: month,
            day: 1,
            max: month.days_in_month(self.is_leap_year()),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct DaysForMonth<'year> {
    year: &'year Year,
    month: Month,
    day: i8,
    max: i8,
}

impl<'year> Iterator for DaysForMonth<'year> {
    type Item = LocalDate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.day <= self.max {
            let date = LocalDate::ymd(self.year.0, self.month, self.day).unwrap();  // Can this ever be invalid?
            self.day += 1;
            Some(date)
        }
        else {
            None
        }
    }
}

impl<'year> DoubleEndedIterator for DaysForMonth<'year> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.day <= self.max {
            let date = LocalDate::ymd(self.year.0, self.month, self.max).unwrap();  // ditto
            self.max -= 1;
            Some(date)
        }
        else {
            None
        }
    }
}

