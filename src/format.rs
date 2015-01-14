use std::io;
use local;
use local::{LocalDate, DatePiece};

#[derive(PartialEq, Eq, Clone, Show)]
pub enum Field<'a> {
    Literal(&'a str),

    Year,
    YearOfCentury,

    MonthName(bool),

    Day,
    DayOfWeekName(bool),
}

impl<'a> Copy for Field<'a> { }

impl<'a> Field<'a> {
    fn format(self, when: LocalDate, w: &mut io::MemWriter) -> io::IoResult<()> {
        match self {
            Field::Literal(s)           => write!(w, "{}", s),
            Field::Year                 => write!(w, "{}", when.year()),
            Field::YearOfCentury        => write!(w, "{}", when.year_of_century()),
            Field::MonthName(true)      => write!(w, "{}", long_month_name(when.month())),
            Field::MonthName(false)     => write!(w, "{}", short_month_name(when.month())),
            Field::Day                  => write!(w, "{}", when.day()),
            Field::DayOfWeekName(true)  => write!(w, "{}", long_day_name(when.weekday())),
            Field::DayOfWeekName(false) => write!(w, "{}", short_day_name(when.weekday())),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Show)]
pub struct DateFormat<'a> {
    pub fields: Vec<Field<'a>>,
}

#[derive(PartialEq, Eq, Clone, Show)]
pub enum FormatError {
    InvalidChar(char, usize),
    OpenCurlyBrace,
    MissingField,
}

impl Copy for FormatError { }

#[derive(PartialEq, Eq, Clone, Show)]
enum Alignment {
    Left,
    Centre,
    Right,
}

struct Arguments {
    alignment: Option<Alignment>,
    width:     Option<usize>,
    pad_char:  Option<char>,
}

impl Arguments {
    pub fn empty() -> Arguments {
        Arguments {
            alignment: None,
            width:     None,
            pad_char:  None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.alignment.is_none() && self.width.is_none() && self.pad_char.is_none()
    }
}

impl<'a> DateFormat<'a> {
    pub fn format(self, when: LocalDate) -> String {
        let mut buf = io::MemWriter::new();
        for bit in self.fields.into_iter() {
            bit.format(when, &mut buf);
        }
        String::from_utf8(buf.into_inner()).unwrap()
    }

    pub fn parse(input: &'a str) -> Result<DateFormat<'a>, FormatError> {
        let mut parser = FormatParser {
            iter: input.char_indices(),
            fields: Vec::new(),
            input: input,
        };

        try! { parser.parse_format_string() };

        Ok(DateFormat { fields: parser.fields })
    }
}

struct FormatParser<'a, I> {
    iter: I,
    fields: Vec<Field<'a>>,
    input: &'a str,
}

impl<'a, I: Iterator<Item=(usize, char)>> FormatParser<'a, I> {
    fn next(&mut self) -> Option<(usize, char)> {
        self.iter.next()
    }

    fn get_input_slice(&self, from: usize, to: Option<usize>) -> Field {
        let slice = match to {
            None =>    self.input.slice_from(from),
            Some(n) => self.input.slice(from, n),
        };

        Field::Literal(slice)
    }

    fn parse_format_string(&mut self) -> Result<(), FormatError> {
        let mut anchor = None;

        loop {
            match self.next() {
                Some((new_pos, '{')) => {
                    if let Some(pos) = anchor {
                        anchor = None;
                        let field = Field::Literal(self.input.slice(pos, new_pos));
                        self.fields.push(field);
                    }

                    let field = try! { self.parse_a_thing() };
                    self.fields.push(field);
                },
                Some((pos, c)) => {
                    if anchor.is_none() {
                        anchor = Some(pos);
                    }
                }
                None => break,
            }
        }

        if let Some(pos) = anchor {
            let field = Field::Literal(self.input.slice_from(pos));
            self.fields.push(field);
        }

        Ok(())
    }

    fn parse_a_thing(&mut self) -> Result<Field<'a>, FormatError> {
        let mut args = Arguments::empty();
        let mut bit = None;

        loop {
            match self.next() {
                Some((_, 'Y')) => { bit = Some(Field::Year); },
                Some((_, 'M')) => { bit = Some(Field::MonthName(true)); },
                Some((_, 'D')) => { bit = Some(Field::Day); },
                Some((_, '}')) => {
                    if let Some(b) = bit {
                        return Ok(b)
                    }
                    else {
                        return Err(FormatError::MissingField);
                    }
                },
                Some(_) => {
                    args.alignment = Some(Alignment::Centre);
                },
                None => return Err(FormatError::OpenCurlyBrace),
            }
        }
    }
}

fn long_month_name(month: local::Month) -> &'static str {
    use local::Month::*;
    match month {
        January   => "January",    February  => "February",
        March     => "March",      April     => "April",
        May       => "May",        June      => "June",
        July      => "July",       August    => "August",
        September => "September",  October   => "October",
        November  => "November",   December  => "December",
    }
}

fn short_month_name(month: local::Month) -> &'static str {
    use local::Month::*;
    match month {
        January   => "Jan", February  => "Feb",
        March     => "Mar", April     => "Apr",
        May       => "May", June      => "Jun",
        July      => "Jul", August    => "Aug",
        September => "Sep", October   => "Oct",
        November  => "Nov", December  => "Dec",
    }
}

fn long_day_name(day: local::Weekday) -> &'static str {
    use local::Weekday::*;
    match day {
        Monday    => "Monday",     Tuesday   => "Tuesday",
        Wednesday => "Wednesday",  Thursday  => "Thursday",
        Friday    => "Friday",     Saturday  => "Saturday",
        Sunday    => "Sunday",

    }
}

fn short_day_name(day: local::Weekday) -> &'static str {
    use local::Weekday::*;
    match day {
        Monday    => "Mon",  Tuesday   => "Tue",
        Wednesday => "Wed",  Thursday  => "Thu",
        Friday    => "Fri",  Saturday  => "Sat",
        Sunday    => "Sun",

    }
}

#[cfg(test)]
mod test {
    pub use super::DateFormat;
    pub use super::Field::*;

    mod parse {
        use super::*;

        #[test]
        fn empty_string() {
            assert_eq!(DateFormat::parse("").unwrap(), DateFormat { fields: vec![] })
        }

        #[test]
        fn entirely_literal() {
            assert_eq!(DateFormat::parse("Date!").unwrap(), DateFormat { fields: vec![ Literal("Date!") ] })
        }

        #[test]
        fn single_element() {
            assert_eq!(DateFormat::parse("{Y}").unwrap(), DateFormat { fields: vec![ Year ] })
        }

        #[test]
        fn two_long_years() {
            assert_eq!(DateFormat::parse("{Y}{Y}").unwrap(), DateFormat { fields: vec![ Year, Year ] })
        }

        #[test]
        fn surrounded() {
            assert_eq!(DateFormat::parse("({D})").unwrap(), DateFormat { fields: vec![ Literal("("), Day, Literal(")") ] })
        }

        #[test]
        fn a_bunch_of_elements() {
            assert_eq!(DateFormat::parse("{Y}-{M}-{D}").unwrap(), DateFormat { fields: vec![ Year, Literal("-"), MonthName(true), Literal("-"), Day ] })
        }
    }
}
