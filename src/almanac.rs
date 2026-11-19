//! No-runtime-dependency almanac rules (not a replacement for ephemerides).
//!
//! Does NOT make claims about precise Li Chun, lunar conversions or full
//! almanac listings without an explicit astronomical data provider.

use crate::ganzhi::{clash, GAN, ZHI};
use crate::{Error, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DayFlags {
    /// Hard exclusions from month-breaking and the four abandoned days.
    pub flags: Vec<&'static str>,
    /// Additional conventional taboos to report but not hard-exclude.
    pub conventional: Vec<&'static str>,
}

/// Four abandoned days by solar-term season.
fn is_sifei(month_branch: char, day: &str) -> Result<bool> {
    let pairs = match month_branch {
        '寅' | '卯' | '辰' => ["庚申", "辛酉"],
        '巳' | '午' | '未' => ["壬子", "癸亥"],
        '申' | '酉' | '戌' => ["甲寅", "乙卯"],
        '亥' | '子' | '丑' => ["丙午", "丁巳"],
        _ => return Err(Error::InvalidGanzhi),
    };
    Ok(pairs.contains(&day))
}

fn shousi(month_branch: char) -> Result<char> {
    const VALUES: [char; 12] = [
        '卯', '酉', '戌', '辰', '亥', '巳',
        '子', '午', '丑', '未', '寅', '申',
    ];
    let index = ZHI
        .iter()
        .position(|&branch| branch == month_branch)
        .ok_or(Error::InvalidGanzhi)?;
    Ok(VALUES[index])
}

/// Pure rules from the Python oracle's almanac.day_flags.
pub fn day_flags(
    year_branch: char,
    month_branch: char,
    day_pillar: &str,
    jianchu: &str,
) -> Result<DayFlags> {
    let mut chars = day_pillar.chars();
    let day_stem = chars.next().ok_or(Error::InvalidGanzhi)?;
    let day_branch = chars.next().ok_or(Error::InvalidGanzhi)?;
    if chars.next().is_some() || !GAN.contains(&day_stem) {
        return Err(Error::InvalidGanzhi);
    }
    if !ZHI.contains(&year_branch)
        || !ZHI.contains(&month_branch)
        || !ZHI.contains(&day_branch)
    {
        return Err(Error::InvalidGanzhi);
    }
    let mut flags = Vec::new();
    let mut conventional = Vec::new();
    if jianchu == "破" {
        flags.push("月破");
    }
    if is_sifei(month_branch, day_pillar)? {
        flags.push("正四废");
    }
    if clash(year_branch, day_branch)? {
        conventional.push("岁破日");
    }
    if shousi(month_branch)? == day_branch {
        conventional.push("受死");
    }
    Ok(DayFlags {
        flags,
        conventional,
    })
}

/// Approximate equation of time in minutes. Not a precise astronomical table.
pub fn equation_of_time(day_of_year: u16) -> Result<f64> {
    if !(1..=366).contains(&day_of_year) {
        return Err(Error::InvalidDayOfYear);
    }
    let b = (360.0 * (f64::from(day_of_year) - 81.0) / 364.0).to_radians();
    Ok(9.87 * (2.0 * b).sin() - 7.53 * b.cos() - 1.5 * b.sin())
}

/// Traditional longitude-corrected solar time offset from a standard meridian.
pub fn solar_offset_minutes(
    day_of_year: u16,
    longitude: f64,
    standard_meridian: f64,
) -> Result<f64> {
    if !longitude.is_finite()
        || !standard_meridian.is_finite()
        || !(-180.0..=180.0).contains(&longitude)
        || !(-180.0..=180.0).contains(&standard_meridian)
    {
        return Err(Error::InvalidBearing);
    }
    Ok((longitude - standard_meridian) * 4.0 + equation_of_time(day_of_year)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_flags_match_reference() {
        assert_eq!(day_flags('午', '巳', "丁亥", "破").unwrap().flags, ["月破"]);
        assert_eq!(day_flags('午', '巳', "戊子", "危").unwrap().conventional, ["岁破日"]);
        assert_eq!(day_flags('午', '巳', "癸巳", "建").unwrap().conventional, ["受死"]);
        assert_eq!(day_flags('午', '寅', "庚申", "破").unwrap().flags, ["月破", "正四废"]);
        assert!(day_flags('午', '巳', "甲午", "成").unwrap().flags.is_empty());
        assert!(day_flags('午', '巳', "bad", "成").is_err());
    }

    #[test]
    fn solar_offset_reference() {
        // June 1 in a non-leap year: day 152.
        let center = solar_offset_minutes(152, 120.0, 120.0).unwrap();
        assert!(center.abs() < 17.0);
        let west = solar_offset_minutes(152, 116.0, 120.0).unwrap();
        assert!((west - center + 16.0).abs() < 1e-8);
    }
}
