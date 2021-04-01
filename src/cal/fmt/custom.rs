//! Datetime-to-string routines.

use std::fmt::Display;
use std::io;
use std::io::Write;
use std::str::CharIndices;

use cal::{DatePiece, TimePiece};

use locale;
use pad::{PadStr, Alignment};


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Field<'a> {
    Literal(&'a str),

    Year(NumArguments),
    YearOfCentury(NumArguments),

    MonthName(bool, TextArguments),

    Day(NumArguments),
    WeekdayName(bool, TextArguments),

    Hour(NumArguments),
    Minute(NumArguments),
    Second(NumArguments),
}

impl<'a> Field<'a> {
    fn format<T>(&self, when: &T, w: &mut Vec<u8>, locale: &locale::Time) -> io::Result<()> where T: DatePiece+TimePiece {
        match *self {
            Field::Literal(s)             => w.write_all(s.as_bytes()),
            Field::Year(a)                => a.format(w, when.year()),
            Field::YearOfCentury(a)       => a.format(w, when.year_of_century()),
            Field::MonthName(true, a)     => a.format(w, &locale.long_month_name(when.month() as usize - 1)[..]),
            Field::MonthName(false, a)    => a.format(w, &locale.short_month_name(when.month() as usize - 1)[..]),
            Field::Day(a)                 => a.format(w, when.day()),
            Field::WeekdayName(true, a)   => a.format(w, &locale.long_day_name(when.weekday() as usize)[..]),
            Field::WeekdayName(false, a)  => a.format(w, &locale.short_day_name(when.weekday() as usize)[..]),
            Field::Hour(a)                => a.format(w, when.hour()),
            Field::Minute(a)              => a.format(w, when.minute()),
            Field::Second(a)              => a.format(w, when.second()),
        }
    }
}


#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DateFormat<'a> {
    pub fields: Vec<Field<'a>>,
}


#[derive(PartialEq, Eq, Clone, Debug, Copy)]
pub enum FormatError {
    InvalidChar { c: char, colon: bool, pos: Pos },
    OpenCurlyBrace { open_pos: Pos },
    CloseCurlyBrace { close_pos: Pos },
    MissingField { open_pos: Pos, close_pos: Pos },
    DoubleAlignment { open_pos: Pos, current_alignment: Alignment },
    DoubleWidth { open_pos: Pos, current_width: Width },
}

pub type Width = usize;
pub type Pos = usize;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Arguments {
    pub alignment: Option<Alignment>,
    pub width:     Option<Width>,
    pub pad_char:  Option<char>,
}

impl Arguments {
    pub fn empty() -> Self {
        Self {
            alignment: None,
            width:     None,
            pad_char:  None,
        }
    }

    pub fn set_width(&mut self, width: Width) -> Self {
        self.width = Some(width);
        *self
    }

    pub fn set_alignment(&mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        *self
    }

    pub fn update_width(&mut self, width: Width, open_pos: Pos) -> Result<(), FormatError> {
        match self.width {
            None => { self.width = Some(width); Ok(())},
            Some(existing) => Err(FormatError::DoubleWidth { open_pos, current_width: existing }),
        }
    }

    pub fn update_alignment(&mut self, alignment: Alignment, open_pos: Pos) -> Result<(), FormatError> {
        match self.alignment {
            None => { self.alignment = Some(alignment); Ok(())},
            Some(existing) => Err(FormatError::DoubleAlignment { open_pos, current_alignment: existing }),
        }
    }

    fn format(self, w: &mut Vec<u8>, string: &str) -> io::Result<()> {
        let width     = self.width.unwrap_or(0);
        let pad_char  = self.pad_char.unwrap_or(' ');
        let alignment = self.alignment.unwrap_or(Alignment::Left);
        let s         = string.pad(width, pad_char, alignment, false);

        w.write_all(s.as_bytes())
    }

    pub fn is_empty(&self) -> bool {
        self.alignment.is_none() && self.width.is_none() && self.pad_char.is_none()
    }
}


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct TextArguments(Arguments);

impl TextArguments {
    #[cfg(test)]
    fn empty() -> TextArguments {
        TextArguments(Arguments::empty())
    }

    fn format(self, w: &mut Vec<u8>, string: &str) -> io::Result<()> {
        self.0.format(w, string)
    }
}


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct NumArguments(Arguments);

impl NumArguments {
    #[cfg(test)]
    fn empty() -> NumArguments {
        NumArguments(Arguments::empty())
    }

    fn format<N: Display>(self, w: &mut Vec<u8>, number: N) -> io::Result<()> {
        self.0.format(w, &number.to_string())
    }
}

impl<'a> DateFormat<'a> {
    pub fn format<T>(&self, when: &T, locale: &locale::Time) -> String where T: DatePiece+TimePiece{
        let mut buf = Vec::<u8>::new();

        for field in &self.fields {
            // It's safe to just ignore the error when writing to an in-memory
            // Vec<u8> buffer. If it fails then you have bigger problems
            match field.format(when, &mut buf, locale) { _ => {} }
        }

        String::from_utf8(buf).unwrap()  // Assume UTF-8
    }

    pub fn parse(input: &'a str) -> Result<DateFormat<'a>, FormatError> {
        let mut parser = FormatParser::new(input);
        parser.parse_format_string()?;

        Ok(DateFormat { fields: parser.fields })
    }
}


struct FormatParser<'a> {
    iter:   CharIndices<'a>,
    fields: Vec<Field<'a>>,
    input:  &'a str,
    anchor: Option<Pos>,
    peekee: Option<Option<(Pos, char)>>,
}

impl<'a> FormatParser<'a> {
    fn new(input: &'a str) -> FormatParser<'a> {
        FormatParser {
            iter:   input.char_indices(),
            fields: Vec::new(),
            input,
            anchor: None,
            peekee: None,
        }
    }

    fn next(&mut self) -> Option<(Pos, char)> {
        match self.peekee {
            Some(p) => {
                self.peekee = None;
                p
            },
            None => { self.iter.next() },
        }
    }

    fn peek(&mut self) -> Option<(Pos, char)> {
        match self.peekee {
            Some(thing) => thing,
            None => {
                self.peekee = Some(self.iter.next());
                self.peek()
            }
        }
    }

    fn collect_up_to_anchor(&mut self, position: Option<Pos>) {
        if let Some(pos) = self.anchor {
            self.anchor = None;
            let text = match position {
                Some(new_pos) => &self.input[pos..new_pos],
                None          => &self.input[pos..],
            };
            self.fields.push(Field::Literal(text));
        }
    }

    fn parse_format_string(&mut self) -> Result<(), FormatError> {
        loop {
            match self.next() {
                Some((new_pos, '{')) => {
                    self.collect_up_to_anchor(Some(new_pos)	);

                    let field = self.parse_a_thing(new_pos)?;
                    self.fields.push(field);
                },
                Some((new_pos, '}')) => {
                    if let Some((_, '}')) = self.next() {
                        self.collect_up_to_anchor(Some(new_pos));

                        let field = Field::Literal(&self.input[new_pos ..=new_pos]);
                        self.fields.push(field);
                    }
                    else {
                        return Err(FormatError::CloseCurlyBrace { close_pos: new_pos });
                    }
                },
                Some((pos, _)) => {
                    if self.anchor.is_none() {
                        self.anchor = Some(pos);
                    }
                }
                None => break,
            }
        }

        // Finally, collect any literal characters after the last date field
        // that haven't been turned into a Literal field yet.
        self.collect_up_to_anchor(None);
        Ok(())
    }

    // The Literal strings are just slices of the original formatting string,
    // which shares a lifetime with the formatter object, requiring fewer
    // allocations. The parser is clever and combines consecutive literal
    // strings.
    //
    // However, because they're slices, we can't transform them
    // to escape {{ and }} characters. So instead, up to three adjacent
    // Literal fields can be used to serve '{' or '}' characters, including
    // one that's the *first character* of the "{{" part. This means it can
    // still use slices.

    fn parse_number(&mut self, just_parsed_character: char) -> usize {
        let mut buf = just_parsed_character.to_string();

        loop {
            if let Some((_, n)) = self.peek() {
                if n.is_digit(10) {
                    buf.push(n);
                    let _ = self.next();  // ignore result - it's going to be the same!
                }
                else {
                    break;
                }
            }
            else {
                break;
            }
        }

        buf.parse().unwrap()
    }

    fn parse_a_thing(&mut self, open_pos: Pos) -> Result<Field<'a>, FormatError> {
        let mut args = Arguments::empty();
        let mut bit = None;
        let close_pos;
        let mut first = true;
        let mut long = false;

        loop {
            match self.next() {
                Some((pos, '{')) if first => return Ok(Field::Literal(&self.input[pos ..=pos])),
                Some((_, '<')) => { args.update_alignment(Alignment::Left, open_pos)?; continue },
                Some((_, '^')) => { args.update_alignment(Alignment::Middle, open_pos)?; continue },
                Some((_, '>')) => { args.update_alignment(Alignment::Right, open_pos)?; continue },
                Some((_, '0')) => { args.pad_char = Some('0'); continue },
                Some((_, n)) if n.is_digit(10) => { args.update_width(self.parse_number(n), open_pos)?; continue },
                Some((_, '_')) => { long = true; },
                Some((_, ':')) => {
                    let bitlet = match self.next() {
                        Some((_, 'Y')) => Field::Year(NumArguments(args)),
                        Some((_, 'y')) => Field::YearOfCentury(NumArguments(args)),
                        Some((_, 'M')) => Field::MonthName(long, TextArguments(args)),
                        Some((_, 'D')) => Field::Day(NumArguments(args)),
                        Some((_, 'E')) => Field::WeekdayName(long, TextArguments(args)),
                        Some((_, 'h')) => Field::Hour(NumArguments(args)),
                        Some((_, 'm')) => Field::Minute(NumArguments(args)),
                        Some((_, 's')) => Field::Second(NumArguments(args)),
                        Some((pos, c)) => return Err(FormatError::InvalidChar { c, colon: true, pos }),
                        None => return Err(FormatError::OpenCurlyBrace { open_pos }),
                    };

                    bit = Some(bitlet);
                },
                Some((pos, '}')) => { close_pos = pos; break; },
                Some((pos, c)) => return Err(FormatError::InvalidChar { c, colon: false, pos }),
                None => return Err(FormatError::OpenCurlyBrace { open_pos }),
            };

            first = false;
        }

        match bit {
            Some(b) => Ok(b),
            None    => Err(FormatError::MissingField { open_pos, close_pos }),
        }
    }
}


#[cfg(test)]
mod test {
    pub(crate) use super::{DateFormat, FormatError, Arguments, NumArguments, TextArguments};
    pub(crate) use super::Field::*;
    pub(crate) use pad::Alignment;

    mod parse {
        use super::*;

        macro_rules! test {
            ($name: ident: $input: expr => $result: expr) => {
                #[test]
                fn $name() {
                    assert_eq!(DateFormat::parse($input), $result)
                }
            };
        }

        test!(empty_string: ""                      => Ok(DateFormat { fields: vec![] }));
        test!(entirely_literal: "Date!"             => Ok(DateFormat { fields: vec![ Literal("Date!") ] }));
        test!(single_element: "{:Y}"                => Ok(DateFormat { fields: vec![ Year(NumArguments::empty()) ] }));
        test!(two_long_years: "{:Y}{:Y}"            => Ok(DateFormat { fields: vec![ Year(NumArguments::empty()), Year(NumArguments::empty()) ] }));
        test!(surrounded: "({:D})"                  => Ok(DateFormat { fields: vec![ Literal("("), Day(NumArguments::empty()), Literal(")") ] }));
        test!(a_bunch_of_elements: "{:Y}-{:M}-{:D}" => Ok(DateFormat { fields: vec![ Year(NumArguments::empty()), Literal("-"), MonthName(false, TextArguments::empty()), Literal("-"), Day(NumArguments::empty()) ] }));

        test!(missing_field: "{}"                              => Err(FormatError::MissingField { open_pos: 0, close_pos: 1 }));
        test!(invalid_char: "{a}"                              => Err(FormatError::InvalidChar { c: 'a', colon: false, pos: 1 }));
        test!(invalid_char_after_colon: "{:7}"                 => Err(FormatError::InvalidChar { c: '7', colon: true, pos: 2 }));
        test!(open_curly_brace: "{"                            => Err(FormatError::OpenCurlyBrace { open_pos: 0 }));
        test!(mystery_close_brace: "}"                         => Err(FormatError::CloseCurlyBrace { close_pos: 0 }));
        test!(another_mystery_close_brace: "This is a test: }" => Err(FormatError::CloseCurlyBrace { close_pos: 16 }));

        test!(escaping_open: "{{"  => Ok(DateFormat { fields: vec![ Literal("{") ] }));
        test!(escaping_close: "}}" => Ok(DateFormat { fields: vec![ Literal("}") ] }));

        test!(escaping_middle: "The character {{ is my favourite!" => Ok(DateFormat { fields: vec![ Literal("The character "), Literal("{"), Literal(" is my favourite!") ] }));
        test!(escaping_middle_2: "It's way better than }}."        => Ok(DateFormat { fields: vec![ Literal("It's way better than "), Literal("}"), Literal(".") ] }));

        mod alignment {
            use super::*;

            test!(left:   "{<:Y}" => Ok(DateFormat { fields: vec![ Year(NumArguments(Arguments::empty().set_alignment(Alignment::Left))) ]}));
            test!(right:  "{>:Y}" => Ok(DateFormat { fields: vec![ Year(NumArguments(Arguments::empty().set_alignment(Alignment::Right))) ]}));
            test!(middle: "{^:Y}" => Ok(DateFormat { fields: vec![ Year(NumArguments(Arguments::empty().set_alignment(Alignment::Middle))) ]}));
        }

        mod alignment_fails {
            use super::*;

            test!(double_left:  "{<<:Y}" => Err(FormatError::DoubleAlignment { open_pos: 0, current_alignment: Alignment::Left }));
            test!(double_right: "{>>:Y}" => Err(FormatError::DoubleAlignment { open_pos: 0, current_alignment: Alignment::Right }));
            test!(left_right: "{<>:Y}"   => Err(FormatError::DoubleAlignment { open_pos: 0, current_alignment: Alignment::Left }));
            test!(right_middle: "{>^:Y}" => Err(FormatError::DoubleAlignment { open_pos: 0, current_alignment: Alignment::Right }));
        }

        mod width {
            use super::*;

            test!(width_2: "{>2:D}"                 => Ok(DateFormat { fields: vec![ Day(NumArguments(Arguments::empty().set_width(2).set_alignment(Alignment::Right))) ] }));
            test!(width_3: "{>3:D}"                 => Ok(DateFormat { fields: vec![ Day(NumArguments(Arguments::empty().set_width(3).set_alignment(Alignment::Right))) ] }));
            test!(width_10: "{>10:D}"               => Ok(DateFormat { fields: vec![ Day(NumArguments(Arguments::empty().set_width(10).set_alignment(Alignment::Right))) ] }));
            test!(width_10_other: "{10>:D}"         => Ok(DateFormat { fields: vec![ Day(NumArguments(Arguments::empty().set_width(10).set_alignment(Alignment::Right))) ] }));
            test!(width_123456789: "{>123456789:D}" => Ok(DateFormat { fields: vec![ Day(NumArguments(Arguments::empty().set_width(123456789).set_alignment(Alignment::Right))) ] }));
        }
    }
}
