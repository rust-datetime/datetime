use std::borrow::Cow;
use std::collections::VecDeque;

use local::{self, LocalDate, LocalTime, LocalDateTime, DatePiece};
use zoned::{TimeZone, LocalTimes};


#[derive(PartialEq, Debug)]
pub struct Zone<'a> {

    /// This zone's name in the zoneinfo database, such as "America/New_York".
    pub name: &'a str,

    /// A static slice of all the timespans that pertain to this zone.
    /// These should be in order of when they end, up until the
    /// currently-applying timespan.
    pub timespans: &'a [Timespan<'a>],
}

impl<'a> Zone<'a> {

    /// Gets this zone's name in the zoneinfo database.
    pub fn zoneinfo_name(&self) -> &'a str {
        self.name
    }

    fn find_relevant_timespan(&self, date: LocalDate) -> &Timespan {
        for timespan in self.timespans {

            // Timespans without an 'until' datespec will always match.
            // They will be the last ones in the slice, so it can be assumed
            // that if none have matched up until this point, this one will!
            let until_spec = match timespan.end_time {
                Some(u) => u,
                None    => return timespan,
            };

            // If the date is before this span's 'until' date, then it's a
            // match, so return it. Otherwise, continue with the next span.
            if LocalDateTime::at(until_spec).date() > date {
                return timespan;
            }
        }

        // A timespan should *always* end with one with a currently-applying
        // date. If it doesn't, it's an error.
        unreachable!()
    }

    /// Returns an iterator over any transitions that occur **after** the
    /// given time, going forward, in the order in which they occur.
    ///
    /// This is useful for finding out what the next transition is going to
    /// be, given a local date and time.
    pub fn forward_transitions(&self, start_time: LocalDateTime) -> ForwardTransitions {
        ForwardTransitions::new(start_time, self.timespans.iter().collect())
    }
}

impl<'a> TimeZone<'a> for Zone<'a> {
    fn offset(&self, _datetime: LocalDateTime) -> i64 {
        unimplemented!()
    }

    fn name(&'a self, datetime: LocalDateTime) -> Cow<'a, str> {
        let timespan = self.find_relevant_timespan(datetime.date());

        if let Saving::Multiple(ref ruleset) = timespan.saving {
            let current_rule = ruleset.find_relevant_rule(&datetime);

            let letters = match current_rule {
                Some(rule)  => rule.letters.unwrap_or(""),
                None        => "",
            };

            Cow::Owned(timespan.format.replace("%s", letters))
        }
        else {
            // TODO(ogham): replace with IntoCow trait once it stabilises
            Cow::Borrowed(timespan.format)
        }
    }

    fn is_fixed(&self) -> bool {
        self.timespans.len() <= 1
    }

    fn from_local(&self, _local: LocalDateTime) -> LocalTimes {
        unimplemented!()
    }
}

#[derive(PartialEq, Debug)]
enum TransitionState {
    Year(i64),
    Midway(LocalDateTime),
}

#[derive(PartialEq, Debug)]
pub struct ForwardTransitions<'a> {
    time: TransitionState,
    timespans: Vec<&'a Timespan<'a>>,
    output_queue: VecDeque<Transition>,
}

impl<'a> ForwardTransitions<'a> {
    fn new(start_time: LocalDateTime, timespans: Vec<&'a Timespan>) -> ForwardTransitions<'a> {
        ForwardTransitions {
            time: TransitionState::Midway(start_time),
            timespans: timespans,
            output_queue: VecDeque::new(),
        }
    }
}

impl<'a> Iterator for ForwardTransitions<'a> {
    type Item = Transition;

    fn next(&mut self) -> Option<Self::Item> {
        loop {

            // Use up anything in the output queue first.
            if let Some(t) = self.output_queue.pop_front() {
                return Some(t);
            }

            // We are done if there are no valid timespans left.
            if self.timespans.is_empty() {
                return None;
            }
            else if let TransitionState::Midway(current_time) = self.time {
                if self.timespans[0].has_completed_before(current_time) {
                    self.timespans.remove(0);
                    continue;
                }
            }


            match self.timespans[0].saving {
                Saving::NoSaving => {
                    if let Some(start_time) = self.timespans[0].start_time {
                        self.output_queue.push_back(
                            Transition {
                                occurs_at: LocalDateTime::at(start_time),
                                gmt_offset: self.timespans[0].offset,
                                dst_offset: 0,
                            }
                        );
                    }

                    if let Some(end_time) = self.timespans[0].end_time {
                        let date  = LocalDateTime::at(end_time);
                        self.time = TransitionState::Midway(date);
                    }

                    self.timespans.remove(0);
                },

                Saving::OneOff(amount) => {
                    if let Some(start_time) = self.timespans[0].start_time {
                        self.output_queue.push_back(
                            Transition {
                                occurs_at: LocalDateTime::at(start_time),
                                gmt_offset: self.timespans[0].offset,
                                dst_offset: amount,
                            }
                        );
                    }

                    if let Some(end_time) = self.timespans[0].end_time {
                        let date  = LocalDateTime::at(end_time);
                        self.time = TransitionState::Midway(date);
                    }

                    self.timespans.remove(0);
                },

                Saving::Multiple(rules) => {

                    match self.time {
                        TransitionState::Midway(time) => {
                            let year = time.year();
                            self.output_queue = rules.rules.iter()
                                .filter(|r| r.applies_to_year(year))
                                .map(|r| Transition {
                                    occurs_at: r.absolute_datetime(year),
                                    gmt_offset: self.timespans[0].offset,
                                    dst_offset: r.time_to_add,
                                })
                                .collect();

                            if let Some(start_time) = self.timespans[0].start_time {
                                let start_time = LocalDateTime::at(start_time);
                                if start_time >= time {
                                    self.output_queue.push_front(Transition {
                                        occurs_at:  start_time,
                                        gmt_offset: self.timespans[0].offset,
                                        dst_offset: 0,
                                    });
                                }
                            }

                            self.time = TransitionState::Year(year + 1);
                        },
                        TransitionState::Year(year) => {
                            if self.timespans[0].end_year() == Some(year) {
                                // do the rules up until that point
                                if let Some(end_time) = self.timespans[0].end_time {
                                    self.time = TransitionState::Midway(LocalDateTime::at(end_time));
                                }
                                self.timespans.remove(0);
                                continue;  // could be the last one, but whatev
                            }
                            else {
                                self.output_queue = rules.rules.iter()
                                    .filter(|r| r.applies_to_year(year))
                                    .map(|r| Transition {
                                        occurs_at: r.absolute_datetime(year),
                                        gmt_offset: self.timespans[0].offset,
                                        dst_offset: r.time_to_add,
                                    })
                                    .collect();

                                self.time = TransitionState::Year(year + 1);
                            }

                            if let Some(maximum_year) = rules.maximum_year() {
                                if maximum_year <= year {
                                    self.timespans.remove(0);
                                    continue;
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Transition {
    pub occurs_at: LocalDateTime,
    pub gmt_offset: i64,
    pub dst_offset: i64,
}

#[derive(PartialEq, Debug)]
pub struct Timespan<'a> {
    pub offset: i64,
    pub saving: Saving<'a>,
    pub format: &'a str,

    pub start_time: Option<i64>,
    pub end_time:   Option<i64>,
}

impl<'a> Timespan<'a> {
    fn has_completed_before(&self, date: LocalDateTime) -> bool {
        match self.end_time {
            Some(end_time) => LocalDateTime::at(end_time) < date,
            None           => false,
        }
    }

    fn end_year(&self) -> Option<i64> {
        match self.end_time {
            Some(end_time) => Some(LocalDateTime::at(end_time).year()),
            None           => None,
        }
    }
}


/// The amount of daylight saving time (DST) to apply to this timespan.
#[derive(PartialEq, Debug)]
pub enum Saving<'a> {

    /// Just stick to the base offset.
    NoSaving,

    /// This amount of time should be saved while this timespan is in effect.
    /// (This is the equivalent to there being a single one-off rule with the
    /// given amount of time to save).
    OneOff(i64),

    /// All rules in the referenced ruleset should apply while this timespan
    /// is in effect.
    Multiple(&'a Ruleset<'a>),
}


#[derive(PartialEq, Debug)]
pub struct Ruleset<'a> {
    pub rules: &'a [RuleInfo<'a>],
}

impl<'a> Ruleset<'a> {
    fn find_relevant_rule(&self, datetime: &LocalDateTime) -> Option<&RuleInfo> {
        let mut rules = self.rules.iter()
                                  .map(|r| (r.absolute_datetime(datetime.year()), r))
                                  .collect::<Vec<_>>();

        rules.sort_by(|a, b| b.0.cmp(&a.0));
        rules.iter().find(|r| r.0 < *datetime).map(|t| t.1)
    }

    fn maximum_year(&self) -> Option<i64> {
        use self::YearSpec::*;
        use std::cmp::max;

        let mut current_max: i64 = -1;

        for rule in self.rules {
            match rule.to_year {
                Some(Number(year)) => { current_max = max(current_max, year); },
                Some(Maximum)      => return None,
                Some(Minimum)      => unreachable!("Can't end on a minimum year!"),
                None               => {
                    match rule.from_year {
                        Number(year)  => { current_max = max(current_max, year); },
                        Maximum       => unreachable!("Can't start on a maximum year!"),
                        Minimum       => { /* skip */ },
                    }
                },
            }
        }

        if current_max == -1 {
            None
        }
        else {
            Some(current_max)
        }
    }
}


#[derive(PartialEq, Debug)]
pub struct RuleInfo<'a> {
    pub from_year:   YearSpec,
    pub to_year:     Option<YearSpec>,
    pub month:       MonthSpec,
    pub day:         DaySpec,
    pub time:        i64,
    pub time_to_add: i64,
    pub letters:     Option<&'a str>,
}

impl<'a> RuleInfo<'a> {
    fn applies_to_year(&self, year: i64) -> bool {
        use self::YearSpec::*;

        match (self.from_year, self.to_year) {
            (Number(from), None)             => year == from,
            (Number(from), Some(Maximum))    => year >= from,
            (Number(from), Some(Number(to))) => year >= from && year <= to,
            _ => unreachable!(),
        }
    }

    fn absolute_datetime(&self, year: i64) -> LocalDateTime {
        let date = self.day.to_concrete_date(year, self.month.0);
        let time = LocalTime::from_seconds_since_midnight(self.time);
        LocalDateTime::new(date, time)
    }
}


#[derive(PartialEq, Debug, Copy, Clone)]
pub enum YearSpec {
    Minimum,
    Maximum,
    Number(i64),
}


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct MonthSpec(pub local::Month);


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct WeekdaySpec(pub local::Weekday);


#[derive(PartialEq, Debug, Copy, Clone)]
pub enum DaySpec {
    Ordinal(i8),
    Last(WeekdaySpec),
    LastOnOrBefore(WeekdaySpec, i8),
    FirstOnOrAfter(WeekdaySpec, i8)
}

impl DaySpec {

    /// Converts this day specification to a concrete date, given the year and
    /// month it should occur in.
    fn to_concrete_date(&self, year: i64, month: local::Month) -> LocalDate {
        use local::{LocalDate, Year, DatePiece};

        match *self {
            DaySpec::Ordinal(day)           => LocalDate::ymd(year, month, day as i8).unwrap(),
            DaySpec::Last(w)                => DaySpec::find_weekday(w, Year(year).days_for_month(month).rev()),
            DaySpec::LastOnOrBefore(w, day) => DaySpec::find_weekday(w, Year(year).days_for_month(month).rev().filter(|d| d.day() < day as i8)),
            DaySpec::FirstOnOrAfter(w, day) => DaySpec::find_weekday(w, Year(year).days_for_month(month).skip(day as usize - 1)),
        }
    }

    /// Find the first-occurring day with the given weekday in the iterator.
    /// Panics if it can't find one. It should find one!
    fn find_weekday<I>(weekday: WeekdaySpec, mut iterator: I) -> local::LocalDate
    where I: Iterator<Item=local::LocalDate> {
        use local::DatePiece;

        iterator.find(|date| date.weekday() == weekday.0)
                .expect("Failed to find weekday")
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use datetime::local::{LocalDateTime, LocalDate, LocalTime, Month};

    #[test]
    fn no_transitions() {
        let timespan = Timespan {
            offset: 1234,
            format: "TEST",
            saving: Saving::NoSaving,
            start_time: None,
            end_time: None,
        };

        let zone = Zone {
            name: "Test/Zone",
            timespans: &[ timespan ],
        };

        let mut iterator = zone.forward_transitions(LocalDateTime::at(0));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn one_transition() {
        let timespan_1 = Timespan {
            offset: 1234,
            format: "TEST",
            saving: Saving::NoSaving,
            start_time: None,
            end_time: Some(123456),
        };

        let timespan_2 = Timespan {
            offset: 5678,
            format: "TSET",
            saving: Saving::NoSaving,
            start_time: Some(123456),
            end_time: None,
        };

        let zone = Zone {
            name: "Test/Zone",
            timespans: &[ timespan_1, timespan_2 ],
        };

        let mut iterator = zone.forward_transitions(LocalDateTime::at(0));
        assert_eq!(iterator.next(), Some(Transition {
            occurs_at: LocalDateTime::at(123456),
            gmt_offset: 5678,
            dst_offset: 0,
        }));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn two_transitions() {
        let timespan_1 = Timespan {
            offset: 1234,
            format: "TEST",
            saving: Saving::NoSaving,
            start_time: None,
            end_time: Some(123456),
        };

        let timespan_2 = Timespan {
            offset: 3456,
            format: "TSET",
            saving: Saving::NoSaving,
            start_time: Some(123456),
            end_time: Some(234567),
        };

        let timespan_3 = Timespan {
            offset: 5678,
            format: "ESTE",
            saving: Saving::NoSaving,
            start_time: Some(234567),
            end_time: None,
        };

        let zone = Zone {
            name: "Test/Zone",
            timespans: &[ timespan_1, timespan_2, timespan_3 ],
        };

        let mut iterator = zone.forward_transitions(LocalDateTime::at(0));
        assert_eq!(iterator.next(), Some(Transition {
            occurs_at: LocalDateTime::at(123456),
            gmt_offset: 3456,
            dst_offset: 0,
        }));
        assert_eq!(iterator.next(), Some(Transition {
            occurs_at: LocalDateTime::at(234567),
            gmt_offset: 5678,
            dst_offset: 0,
        }));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn one_rule() {
        let ruleset = Ruleset { rules: &[
            RuleInfo {
                from_year:   YearSpec::Number(1980),
                to_year:     None,
                month:       MonthSpec(Month::February),
                day:         DaySpec::Ordinal(4),
                time:        0,
                time_to_add: 1000,
                letters:     None,
            }
        ] };

        let timespan = Timespan {
            offset: 2000,
            format: "TEST",
            saving: Saving::Multiple(&ruleset),
            start_time: None,
            end_time: None,
        };

        let zone = Zone {
            name: "Test/Zone",
            timespans: &[ timespan ],
        };

        let mut iterator = zone.forward_transitions(LocalDateTime::at(0));
        assert_eq!(iterator.next(), Some(Transition {
            occurs_at: LocalDateTime::new(LocalDate::ymd(1980, Month::February, 4).unwrap(), LocalTime::midnight()),
            gmt_offset: 2000,
            dst_offset: 1000,
        }));

        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn two_rules() {
        let ruleset = Ruleset { rules: &[
            RuleInfo {
                from_year:   YearSpec::Number(1980),
                to_year:     None,
                month:       MonthSpec(Month::February),
                day:         DaySpec::Ordinal(4),
                time:        0,
                time_to_add: 1000,
                letters:     None,
            },
            RuleInfo {
                from_year:   YearSpec::Number(1989),
                to_year:     None,
                month:       MonthSpec(Month::January),
                day:         DaySpec::Ordinal(12),
                time:        0,
                time_to_add: 1500,
                letters:     None,
            },
        ] };

        let timespan = Timespan {
            offset: 2000,
            format: "TEST",
            saving: Saving::Multiple(&ruleset),
            start_time: None,
            end_time: None,
        };

        let zone = Zone {
            name: "Test/Zone",
            timespans: &[ timespan ],
        };

        let mut iterator = zone.forward_transitions(LocalDateTime::at(0));
        assert_eq!(iterator.next(), Some(Transition {
            occurs_at: LocalDateTime::new(LocalDate::ymd(1980, Month::February, 4).unwrap(), LocalTime::midnight()),
            gmt_offset: 2000,
            dst_offset: 1000,
        }));
        assert_eq!(iterator.next(), Some(Transition {
            occurs_at: LocalDateTime::new(LocalDate::ymd(1989, Month::January, 12).unwrap(), LocalTime::midnight()),
            gmt_offset: 2000,
            dst_offset: 1500,
        }));

        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn multiple_changes() {
        let ruleset = Ruleset { rules: &[
            RuleInfo {
                from_year:   YearSpec::Number(1980),
                to_year:     Some(YearSpec::Number(1981)),
                month:       MonthSpec(Month::February),
                day:         DaySpec::Ordinal(4),
                time:        0,
                time_to_add: 1000,
                letters:     None,
            },
        ] };

        let timespan = Timespan {
            offset: 2000,
            format: "TEST",
            saving: Saving::Multiple(&ruleset),
            start_time: None,
            end_time: None,
        };

        let zone = Zone {
            name: "Test/Zone",
            timespans: &[ timespan ],
        };

        let mut iterator = zone.forward_transitions(LocalDateTime::at(0));
        assert_eq!(iterator.next(), Some(Transition {
            occurs_at: LocalDateTime::new(LocalDate::ymd(1980, Month::February, 4).unwrap(), LocalTime::midnight()),
            gmt_offset: 2000,
            dst_offset: 1000,
        }));
        assert_eq!(iterator.next(), Some(Transition {
            occurs_at: LocalDateTime::new(LocalDate::ymd(1981, Month::February, 4).unwrap(), LocalTime::midnight()),
            gmt_offset: 2000,
            dst_offset: 1000,
        }));
        assert_eq!(iterator.next(), None);
    }
}
