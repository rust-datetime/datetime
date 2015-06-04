//! # Date and Time Formatting
//!
//! There are various competing standards for how a date-time formatting string should look:
//! Unix-style `strftime` with `%` symbols and flags, Joda-style formatters that use the number of
//! letters as their widths, and many others, each with their own idiosyncrasies and subtle
//! differences. Thus, it should come as no surprise that this library invents *another* style of
//! formatting string, designed to mimic the syntax of the `format!` and `println!` macros.
//!
//! In order to format a date, you must first create a *formatter object*, passing in a formatting
//! string. This string is checked for correctness, after which it's guaranteed to always work. For
//! example:
//!
//! ```rust
//! use datetime::format::DateFormat;
//! let formatter = match DateFormat::parse("{_:M} {:D}, {:Y}") {
//!     Ok(f) => f,
//!     Err(e) => panic!("Error in format string: {}", e),
//! };
//! ```
//!
//! If there's a syntax error in the formatting string, the `Err(e)` path will be followed,
//! terminating the program and printing out an error. (It's usually considered better style to
//! handle your errors more gracefully than this.) For this reason, it's preferable to use the
//! `date_format!` macro whenever your formatting string is fixed, as this will check its syntax at
//! compile-time, reporting an error if it is invalid, and removing the need for the `match`
//! construct entirely:
//!
//! ```rust
//! use datetime::format::DateFormat;
//! let formatter = date_format!("{_:M} {:D}, {:Y}");
//! ```
//!
//! ## Locales
//!
//! The second thing you need to be aware of before actually being able to format a date into a
//! string is the concept of *locales*. A Locale object specifies how the months and days should be
//! named. "January" isn't universally understood: in some places it's "janvier"; in others,
//! "Janeiro", or "Ιανουαρίου", or "января", or "一月".
//!
//! To govern which language a formatter should use, a Locale object can be passed in. For more
//! information on how Locale objects work, see the [documentation on the Locale
//! crate](http://bsago.me/doc/locale).
//!
//! If you're in a hurry and just want to format a date *right now* then the simplest way to get
//! around this is to just use the English locale:
//!
//! ```rust
//! use locale;
//! let my_locale = locale::Time::english();
//! ```
//!
//! ## Actually Formatting a Date

use num::Integer;

use std::fmt::Display;
use std::io;
use std::io::Write;
use std::str::CharIndices;

use local::{DatePiece, TimePiece};

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
            Field::MonthName(true, a)     => a.format(w, &locale.long_month_name(when.month().months_from_january())[..]),
            Field::MonthName(false, a)    => a.format(w, &locale.short_month_name(when.month().months_from_january())[..]),
            Field::Day(a)                 => a.format(w, when.day()),
            Field::WeekdayName(true, a)   => a.format(w, &locale.long_day_name(when.weekday().days_from_sunday())[..]),
            Field::WeekdayName(false, a)  => a.format(w, &locale.short_day_name(when.weekday().days_from_sunday())[..]),
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
    pub fn empty() -> Arguments {
        Arguments {
            alignment: None,
            width:     None,
            pad_char:  None,
        }
    }

    pub fn set_width(&mut self, width: Width) -> Arguments {
        self.width = Some(width);
        *self
    }

    pub fn set_alignment(&mut self, alignment: Alignment) -> Arguments {
        self.alignment = Some(alignment);
        *self
    }

    pub fn update_width(&mut self, width: Width, open_pos: Pos) -> Result<(), FormatError> {
        match self.width {
            None => Ok({ self.width = Some(width); }),
            Some(existing) => Err(FormatError::DoubleWidth { open_pos: open_pos, current_width: existing }),
        }
    }

    pub fn update_alignment(&mut self, alignment: Alignment, open_pos: Pos) -> Result<(), FormatError> {
        match self.alignment {
            None => Ok({ self.alignment = Some(alignment); }),
            Some(existing) => Err(FormatError::DoubleAlignment { open_pos: open_pos, current_alignment: existing }),
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

    fn format<N: Integer + Display>(self, w: &mut Vec<u8>, number: N) -> io::Result<()> {
        self.0.format(w, &number.to_string())
    }
}

impl<'a> DateFormat<'a> {
    pub fn format<T>(&self, when: &T, locale: &locale::Time) -> String where T: DatePiece+TimePiece{
        let mut buf = Vec::<u8>::new();

        for field in self.fields.iter() {
            // It's safe to just ignore the error when writing to an in-memory
            // Vec<u8> buffer. If it fails then you have bigger problems
            match field.format(when, &mut buf, locale) { _ => {} }
        }

        String::from_utf8(buf).unwrap()  // Assume UTF-8
    }

    pub fn parse(input: &'a str) -> Result<DateFormat<'a>, FormatError> {
        let mut parser = FormatParser::new(input);
        try! { parser.parse_format_string() };

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
            input:  input,
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

                    let field = try! { self.parse_a_thing(new_pos) };
                    self.fields.push(field);
                },
                Some((new_pos, '}')) => {
                    if let Some((_, '}')) = self.next() {
                        self.collect_up_to_anchor(Some(new_pos));

                        let field = Field::Literal(&self.input[new_pos .. new_pos + 1]);
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
                    self.next();  // ignore result - it's going to be the same!
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
        let mut close_pos;
        let mut first = true;
        let mut long = false;

        loop {
            match self.next() {
                Some((pos, '{')) if first => return Ok(Field::Literal(&self.input[pos .. pos + 1])),
                Some((_, '<')) => { try! { args.update_alignment(Alignment::Left, open_pos) }; continue },
                Some((_, '^')) => { try! { args.update_alignment(Alignment::Middle, open_pos) }; continue },
                Some((_, '>')) => { try! { args.update_alignment(Alignment::Right, open_pos) }; continue },
                Some((_, '0')) => { args.pad_char = Some('0'); continue },
                Some((_, n)) if n.is_digit(10) => { try! { args.update_width(self.parse_number(n), open_pos) }; continue },
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
                        Some((pos, c)) => return Err(FormatError::InvalidChar { c: c, colon: true, pos: pos }),
                        None => return Err(FormatError::OpenCurlyBrace { open_pos: open_pos }),
                    };

                    bit = Some(bitlet);
                },
                Some((pos, '}')) => { close_pos = pos; break; },
                Some((pos, c)) => return Err(FormatError::InvalidChar { c: c, colon: false, pos: pos }),
                None => return Err(FormatError::OpenCurlyBrace { open_pos: open_pos }),
            };

            first = false;
        }

        match bit {
            Some(b) => Ok(b),
            None    => Err(FormatError::MissingField { open_pos: open_pos, close_pos: close_pos }),
        }
    }
}

#[cfg(test)]
mod test {
    pub use super::{DateFormat, FormatError, Field, Arguments, NumArguments, TextArguments};
    pub use super::Field::*;

    pub use pad::Alignment;

    mod parse {
        pub use super::*;

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
