//! The 24 mountains, 9 palaces, Lo Shu flight and San Yuan periods.

use crate::{Error, Result};
use std::str::FromStr;

/// The 24 mountains in clockwise order, starting at 壬 (345°).
pub const MOUNTAINS: [char; 24] = [
    '壬', '子', '癸', '丑', '艮', '寅', '甲', '卯', '乙', '辰', '巽', '巳', '丙', '午', '丁',
    '未', '坤', '申', '庚', '酉', '辛', '戌', '乾', '亥',
];

/// Lo Shu flight path: center, NW, W, NE, S, N, SW, E, SE.
pub const LUOSHU_PATH: [Palace; 9] = [
    Palace::Center,
    Palace::Qian,
    Palace::Dui,
    Palace::Gen,
    Palace::Li,
    Palace::Kan,
    Palace::Kun,
    Palace::Zhen,
    Palace::Xun,
];

/// Traditional south-up three-by-three display.
pub const GRID: [[Palace; 3]; 3] = [
    [Palace::Xun, Palace::Li, Palace::Kun],
    [Palace::Zhen, Palace::Center, Palace::Dui],
    [Palace::Gen, Palace::Kan, Palace::Qian],
];

/// Cardinal/trigram palaces. Discriminants are *not* flight indices.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Palace {
    Kan,
    Kun,
    Zhen,
    Xun,
    Center,
    Qian,
    Dui,
    Gen,
    Li,
}

impl Palace {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Kan => "坎",
            Self::Kun => "坤",
            Self::Zhen => "震",
            Self::Xun => "巽",
            Self::Center => "中",
            Self::Qian => "乾",
            Self::Dui => "兑",
            Self::Gen => "艮",
            Self::Li => "离",
        }
    }

    pub const fn number(self) -> u8 {
        match self {
            Self::Kan => 1,
            Self::Kun => 2,
            Self::Zhen => 3,
            Self::Xun => 4,
            Self::Center => 5,
            Self::Qian => 6,
            Self::Dui => 7,
            Self::Gen => 8,
            Self::Li => 9,
        }
    }

    pub const fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Self::Kan),
            2 => Some(Self::Kun),
            3 => Some(Self::Zhen),
            4 => Some(Self::Xun),
            5 => Some(Self::Center),
            6 => Some(Self::Qian),
            7 => Some(Self::Dui),
            8 => Some(Self::Gen),
            9 => Some(Self::Li),
            _ => None,
        }
    }

    pub const fn direction(self) -> &'static str {
        match self {
            Self::Kan => "北",
            Self::Kun => "西南",
            Self::Zhen => "东",
            Self::Xun => "东南",
            Self::Center => "中",
            Self::Qian => "西北",
            Self::Dui => "西",
            Self::Gen => "东北",
            Self::Li => "南",
        }
    }

    pub const fn flight_index(self) -> usize {
        match self {
            Self::Center => 0,
            Self::Qian => 1,
            Self::Dui => 2,
            Self::Gen => 3,
            Self::Li => 4,
            Self::Kan => 5,
            Self::Kun => 6,
            Self::Zhen => 7,
            Self::Xun => 8,
        }
    }

    /// Returns the mountain sharing this palace and yuanlong (0..=2).
    pub fn mountain(self, yuanlong: usize) -> Result<Mountain> {
        if yuanlong > 2 || self == Self::Center {
            return Err(Error::InvalidPalace);
        }
        const EIGHT: [Palace; 8] = [
            Palace::Kan,
            Palace::Gen,
            Palace::Zhen,
            Palace::Xun,
            Palace::Li,
            Palace::Kun,
            Palace::Dui,
            Palace::Qian,
        ];
        let palace_index = EIGHT
            .iter()
            .position(|&candidate| candidate == self)
            .ok_or(Error::InvalidPalace)?;
        Ok(Mountain((palace_index * 3 + yuanlong) as u8))
    }
}

/// A validated 24-mountain index.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Mountain(u8);

impl Mountain {
    pub fn from_char(value: char) -> Result<Self> {
        MOUNTAINS
            .iter()
            .position(|&candidate| candidate == value)
            .map(|index| Self(index as u8))
            .ok_or(Error::InvalidMountain)
    }

    pub const fn symbol(self) -> char {
        MOUNTAINS[self.0 as usize]
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }

    pub const fn palace(self) -> Palace {
        const EIGHT: [Palace; 8] = [
            Palace::Kan,
            Palace::Gen,
            Palace::Zhen,
            Palace::Xun,
            Palace::Li,
            Palace::Kun,
            Palace::Dui,
            Palace::Qian,
        ];
        EIGHT[self.0 as usize / 3]
    }

    /// 0=earth, 1=heaven, 2=human yuanlong.
    pub const fn yuanlong(self) -> usize {
        self.0 as usize % 3
    }

    /// Xuankong yin/yang, intentionally distinct from ganzhi yin/yang.
    pub const fn is_yang(self) -> bool {
        let odd_palace = self.palace().number() % 2 == 1;
        if odd_palace {
            self.yuanlong() == 0
        } else {
            self.yuanlong() != 0
        }
    }

    pub const fn opposite(self) -> Self {
        Self((self.0 + 12) % 24)
    }

    /// Neighboring mountain, using a clockwise signed offset.
    pub fn neighbor(self, steps: i32) -> Self {
        Self((i32::from(self.0) + steps).rem_euclid(24) as u8)
    }

    pub const fn center_degrees(self) -> f64 {
        ((345 + 15 * self.0 as u16) % 360) as f64
    }

    /// Direction is clockwise from north; a boundary belongs to the next mountain.
    pub fn from_bearing(degrees: f64) -> Result<(Self, f64)> {
        if !degrees.is_finite() {
            return Err(Error::InvalidBearing);
        }
        let degrees = degrees.rem_euclid(360.0);
        let index = ((degrees - 337.5).rem_euclid(360.0) / 15.0).floor() as u8;
        let mountain = Self(index);
        let offset = (degrees - mountain.center_degrees() + 180.0).rem_euclid(360.0) - 180.0;
        Ok((mountain, offset))
    }
}

impl FromStr for Mountain {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let input = input.trim();
        let mut chars = input.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            if let Ok(mountain) = Self::from_char(c) {
                return Ok(mountain);
            }
        }
        let numeric = input
            .trim_end_matches(|c: char| c == '°' || c == '度')
            .parse::<f64>()
            .map_err(|_| Error::InvalidDirection)?;
        Self::from_bearing(numeric).map(|(mountain, _)| mountain)
    }
}

/// A stable, compact star grid in LUOSHU_PATH order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StarGrid {
    pub stars: [u8; 9],
}

impl StarGrid {
    pub const fn at(self, palace: Palace) -> u8 {
        self.stars[palace.flight_index()]
    }

    pub fn palace_of(self, star: u8) -> Option<Palace> {
        LUOSHU_PATH
            .iter()
            .copied()
            .find(|&palace| self.at(palace) == star)
    }

    pub fn entries(self) -> [(Palace, u8); 9] {
        std::array::from_fn(|index| (LUOSHU_PATH[index], self.stars[index]))
    }
}

/// Forward = +1 each palace; backward = -1.
pub fn fly(center: u8, forward: bool) -> Result<StarGrid> {
    if !(1..=9).contains(&center) {
        return Err(Error::InvalidStar);
    }
    Ok(StarGrid {
        stars: std::array::from_fn(|index| {
            let step = if forward { index as i32 } else { -(index as i32) };
            (i32::from(center) - 1 + step).rem_euclid(9) as u8 + 1
        }),
    })
}

/// Three Yuan, nine periods, with 1864 as period one (Li Chun-adjusted years).
pub fn yun_of_year(lichun_year: i32) -> u8 {
    ((lichun_year - 1864).div_euclid(20).rem_euclid(9) + 1) as u8
}

pub fn yun_range(yun: u8) -> Result<(i32, i32)> {
    if !(1..=9).contains(&yun) {
        return Err(Error::InvalidYun);
    }
    let start = 1864 + (i32::from(yun) - 1) * 20;
    Ok((start, start + 19))
}

pub fn ganzhi_of_year(lichun_year: i32) -> String {
    const GAN: [char; 10] = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
    const ZHI: [char; 12] = [
        '子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥',
    ];
    let gan = GAN[(lichun_year - 4).rem_euclid(10) as usize];
    let zhi = ZHI[(lichun_year - 4).rem_euclid(12) as usize];
    format!("{gan}{zhi}")
}

pub fn zhi_index_of_year(lichun_year: i32) -> usize {
    (lichun_year - 4).rem_euclid(12) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compass_boundaries_and_roundtrip() {
        for (degrees, expected) in [
            (0.0, '子'),
            (352.5, '子'),
            (352.4, '壬'),
            (7.6, '癸'),
            (180.0, '午'),
            (45.0, '艮'),
            (315.0, '乾'),
            (360.0, '子'),
            (-15.0, '壬'),
        ] {
            assert_eq!(Mountain::from_bearing(degrees).unwrap().0.symbol(), expected);
        }
        for mountain in MOUNTAINS {
            let m = Mountain::from_char(mountain).unwrap();
            assert_eq!(Mountain::from_bearing(m.center_degrees()).unwrap(), (m, 0.0));
            assert_eq!(m.opposite().opposite(), m);
            assert_eq!(m.palace().mountain(m.yuanlong()).unwrap(), m);
        }
        assert_eq!(Mountain::from_bearing(187.0).unwrap().1, 7.0);
        assert!(Mountain::from_bearing(f64::NAN).is_err());
    }

    #[test]
    fn yin_yang_and_parsing() {
        let yang: String = MOUNTAINS
            .iter()
            .copied()
            .filter(|&m| Mountain::from_char(m).unwrap().is_yang())
            .collect();
        assert_eq!(yang.chars().count(), 12);
        assert_eq!(Mountain::from_char('壬').unwrap().is_yang(), true);
        assert_eq!(Mountain::from_char('子').unwrap().is_yang(), false);
        assert_eq!(Mountain::from_char('艮').unwrap().is_yang(), true);
        assert_eq!("187.5".parse::<Mountain>().unwrap().symbol(), '丁');
        assert_eq!(" 90度".parse::<Mountain>().unwrap().symbol(), '卯');
        assert!("南".parse::<Mountain>().is_err());
    }

    #[test]
    fn luoshu_and_years() {
        let chart = fly(5, true).unwrap();
        for palace in LUOSHU_PATH {
            assert_eq!(chart.at(palace), palace.number());
        }
        assert_eq!(fly(5, false).unwrap().at(Palace::Qian), 4);
        assert_eq!(fly(1, false).unwrap().at(Palace::Qian), 9);
        assert!(fly(0, true).is_err());
        assert_eq!(ganzhi_of_year(2026), "丙午");
        assert_eq!(yun_of_year(2023), 8);
        assert_eq!(yun_of_year(2024), 9);
        assert_eq!(yun_of_year(2044), 1);
        assert_eq!(yun_range(9).unwrap(), (2024, 2043));
    }
}
