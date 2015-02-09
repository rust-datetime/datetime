#![crate_name = "datetime_macros"]
#![crate_type = "dylib"]
#![feature(core, plugin_registrar, quote, rustc_private)]

//! For complete documentation, see the `datetime` crate.

extern crate syntax;
extern crate rustc;
extern crate datetime;

use syntax::ptr::P;
use syntax::ast;
use syntax::codemap;
use syntax::ext::build::AstBuilder;
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr, DummyResult};
use syntax::parse::token;
use syntax::print::pprust;
use syntax::fold::Folder;
use syntax::ext::quote::rt::ToTokens;
use rustc::plugin::Registry;

use datetime::local::{LocalDate, DatePiece, Month, Weekday};
use datetime::format::{DateFormat, Field, NumArguments, TextArguments};

/// The plugin registrar.
///
/// This function *needs* to have the exact name and signature, otherwise it
/// will not work with no warning.
#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("date", date);
    reg.register_macro("date_format", date_format);
}

/// Implements the `date!()` macro.
fn date(cx: &mut ExtCtxt, sp: codemap::Span, tts: &[ast::TokenTree]) -> Box<MacResult+'static> {
    let format_string = match get_string_argument(cx, tts) {
        Some(f) => f,
        None => return DummyResult::any(sp),
    };

    match LocalDate::parse(format_string.as_slice()) {
        Some(date) => date_to_code(cx, date),
        None => {
            cx.span_err(sp, "Invalid date string");
            DummyResult::any(sp)
        },
    }
}

/// Implements the `date_format!()` macro.
fn date_format(cx: &mut ExtCtxt, sp: codemap::Span, tts: &[ast::TokenTree]) -> Box<MacResult+'static> {
    let format_string = match get_string_argument(cx, tts) {
        Some(f) => f,
        None => return DummyResult::any(sp),
    };

    let formatter = match DateFormat::parse(format_string.as_slice()) {
        Ok(f) => f,
        Err(e) => {
            cx.span_err(sp, format!("Couldn't parse date string `{:?}`", e).as_slice());
            return DummyResult::any(sp);
        }
    };

    let fields = formatter.fields.iter()
                                 .map(|&x| field_to_code(cx, x))
                                 .collect();

    let things = cx.expr_vec(sp, fields);
    MacExpr::new(quote_expr!(cx, datetime::format::DateFormat { fields: vec! $things }))
}

/// The above macros both take one literal string as their only argument. This
/// function extracts it, throwing an error if there's more than one argument,
/// or it isn't a literal string.
fn get_string_argument(cx: &mut ExtCtxt, tts: &[ast::TokenTree]) -> Option<String> {
    let mut parser = cx.new_parser_from_tts(tts);
    let entry = cx.expander().fold_expr(parser.parse_expr());
    let formatter = match entry.node {
        ast::ExprLit(ref lit) => {
            match lit.node {
                ast::LitStr(ref s, _) => s.to_string(),
                _ => return fail(cx, entry.span, pprust::lit_to_string(&**lit)),
            }
        }
        _ => return fail(cx, entry.span, pprust::expr_to_string(&*entry)),
    };

    if !parser.eat(&token::Eof) {
        cx.span_err(parser.span, "only one argument allowed");
        return None;
    }

    Some(formatter)
}

fn fail(cx: &mut ExtCtxt, span: codemap::Span, message: String) -> Option<String> {
    cx.span_err(span, format!("expected string literal but got `{}`", message).as_slice());
    return None;
}

fn month_to_code(cx: &mut ExtCtxt, month: Month) -> P<ast::Expr> {
	match month {
		Month::January   => quote_expr!(cx, datetime::local::Month::January),
		Month::February  => quote_expr!(cx, datetime::local::Month::February),
		Month::March     => quote_expr!(cx, datetime::local::Month::March),
		Month::April     => quote_expr!(cx, datetime::local::Month::April),
		Month::May       => quote_expr!(cx, datetime::local::Month::May),
		Month::June      => quote_expr!(cx, datetime::local::Month::June),
		Month::July      => quote_expr!(cx, datetime::local::Month::July),
		Month::August    => quote_expr!(cx, datetime::local::Month::August),
		Month::September => quote_expr!(cx, datetime::local::Month::September),
		Month::October   => quote_expr!(cx, datetime::local::Month::October),
		Month::November  => quote_expr!(cx, datetime::local::Month::November),
		Month::December  => quote_expr!(cx, datetime::local::Month::December),
	}
}

fn weekday_to_code(cx: &mut ExtCtxt, weekday: Weekday) -> P<ast::Expr> {
    match weekday {
        Weekday::Monday    => quote_expr!(cx, datetime::local::Weekday::Monday),
        Weekday::Tuesday   => quote_expr!(cx, datetime::local::Weekday::Tuesday),
        Weekday::Wednesday => quote_expr!(cx, datetime::local::Weekday::Wednesday),
        Weekday::Thursday  => quote_expr!(cx, datetime::local::Weekday::Thursday),
        Weekday::Friday    => quote_expr!(cx, datetime::local::Weekday::Friday),
        Weekday::Saturday  => quote_expr!(cx, datetime::local::Weekday::Saturday),
        Weekday::Sunday    => quote_expr!(cx, datetime::local::Weekday::Sunday),
    }
}

fn field_to_code(cx: &mut ExtCtxt, field: Field) -> P<ast::Expr> {
    match field {
    	Field::Literal(s)        => quote_expr!(cx, datetime::format::Field::Literal($s)),

    	Field::Year(a)           => { let a = numargs_to_code(cx, a);  quote_expr!(cx, datetime::format::Field::Year($a)) },
    	Field::MonthName(s, a)   => { let a = textargs_to_code(cx, a); quote_expr!(cx, datetime::format::Field::MonthName($s, $a)) },
    	Field::Day(a)            => { let a = numargs_to_code(cx, a);  quote_expr!(cx, datetime::format::Field::Day($a)) },
    	Field::YearOfCentury(a)  => { let a = numargs_to_code(cx, a);  quote_expr!(cx, datetime::format::Field::YearOfCentury($a)) },
    	Field::WeekdayName(s, a) => { let a = textargs_to_code(cx, a); quote_expr!(cx, datetime::format::Field::WeekdayName($s, $a)) },
    }
}

fn date_to_code(cx: &mut ExtCtxt, date: LocalDate) -> Box<MacResult + 'static> {
	let year = date.year();
	let month = month_to_code(cx, date.month());
	let day = date.day();
	let weekday = weekday_to_code(cx, date.weekday());
	let yearday = date.yearday();
	println!("{:?}", date);
	MacExpr::new(quote_expr!(cx, unsafe { datetime::local::LocalDate::_new_with_prefilled_values($year, $month, $day, $weekday, $yearday) }))
}

fn numargs_to_code(cx: &mut ExtCtxt, args: NumArguments) -> P<ast::Expr> {
    let w = args.args.width;
    println!("{:?}", w);
    quote_expr!(cx, datetime::format::NumArguments { args: datetime::format::Arguments { width: None, alignment: None, pad_char: None, } })
}

fn textargs_to_code(cx: &mut ExtCtxt, args: TextArguments) -> P<ast::Expr> {
    let w = args.args.width;
    quote_expr!(cx, datetime::format::TextArguments { args: datetime::format::Arguments { width: None, alignment: None, pad_char: None, } })
}
