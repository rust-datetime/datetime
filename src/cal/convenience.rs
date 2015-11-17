//! Adds convenience functions to some structs.
//!
//! # Example
//! ```
//! # use datetime::LocalDate;
//! # use datetime::DatePiece;
//! use datetime::convenience::Today;
//! let today:LocalDate = LocalDate::today();
//! ```
use cal::datetime::{LocalDate,LocalDateTime};

/// Adds `LocalDate::today() -> LocalDate`
pub trait Today{
    fn today() -> LocalDate;
}

impl Today for LocalDate{
    fn today() -> LocalDate{
        LocalDateTime::now().date()
    }

}

