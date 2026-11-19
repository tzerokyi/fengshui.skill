//! Deterministic, headless feng shui computations.
//!
//! The crate implements traditional rule systems, not predictions of physical
//! outcomes. All year-based calculations expect a *Li Chun adjusted year*;
//! the astronomical date/time conversion is not silently approximated.
//!
//! The public API is independent of a CLI, Python, HTML, and the operating system.

pub mod almanac;
pub mod bazhai;
pub mod core;
pub mod feixing;
pub mod ganzhi;
pub mod zibai;
pub mod zeri;

use std::fmt;

/// Stable validation errors shared by the library and CLI adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidMountain,
    InvalidDirection,
    InvalidBearing,
    InvalidGender,
    InvalidPalace,
    InvalidStar,
    InvalidYun,
    InvalidMonthIndex,
    InvalidGanzhi,
    InvalidDayOfYear,
    InvalidEvent,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidMountain => "unknown 24-mountain name",
            Self::InvalidDirection => "direction must be a 24-mountain name or compass degrees",
            Self::InvalidBearing => "bearing must be finite",
            Self::InvalidGender => "gender must be m/f or 男/女",
            Self::InvalidPalace => "invalid palace or yuanlong",
            Self::InvalidStar => "star must be 1..=9",
            Self::InvalidYun => "yun must be 1..=9",
            Self::InvalidMonthIndex => "solar month index must be 1..=12 (寅=1)",
            Self::InvalidGanzhi => "invalid Gan-Zhi stem, branch or pillar",
            Self::InvalidDayOfYear => "day ordinal must be 1..=366",
            Self::InvalidEvent => "unknown event for date selection",
        })
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
