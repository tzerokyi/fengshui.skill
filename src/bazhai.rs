//! Ba Zhai: Ming Gua and the Eight Wandering Stars.

use crate::core::Palace;
use crate::{Error, Result};
use std::str::FromStr;

pub const BAZHAI_ORDER: [Palace; 8] = [
    Palace::Qian,
    Palace::Kan,
    Palace::Gen,
    Palace::Zhen,
    Palace::Xun,
    Palace::Li,
    Palace::Kun,
    Palace::Dui,
];

const YOUNIAN_SONG: [&str; 8] = [
    "六天五祸绝延生",
    "五天生延绝祸六",
    "六绝祸生延天五",
    "延生祸绝五天六",
    "天五六祸生绝延",
    "六五绝延祸生天",
    "天延绝生祸五六",
    "生祸延绝六五天",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Gender {
    Male,
    Female,
}

impl Gender {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Male => "m",
            Self::Female => "f",
        }
    }
}

impl FromStr for Gender {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "m" | "male" | "男" | "男命" => Ok(Self::Male),
            "f" | "female" | "女" | "女命" => Ok(Self::Female),
            _ => Err(Error::InvalidGender),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HouseGroup {
    East,
    West,
}

impl HouseGroup {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::East => "东四",
            Self::West => "西四",
        }
    }
}

pub const fn east_west(gua: Palace) -> HouseGroup {
    match gua {
        Palace::Kan | Palace::Li | Palace::Zhen | Palace::Xun => HouseGroup::East,
        _ => HouseGroup::West,
    }
}

/// The caller must supply an already Li-Chun-adjusted year.
pub fn mingua_raw(lichun_year: i32, gender: Gender) -> u8 {
    let n = match gender {
        Gender::Male => (11 - lichun_year.rem_euclid(9)).rem_euclid(9),
        Gender::Female => (lichun_year.rem_euclid(9) + 4).rem_euclid(9),
    };
    if n == 0 {
        9
    } else {
        n as u8
    }
}

/// Returns the effective Gua number; male 5 => 2, female 5 => 8.
pub fn mingua(lichun_year: i32, gender: Gender) -> u8 {
    match (mingua_raw(lichun_year, gender), gender) {
        (5, Gender::Male) => 2,
        (5, Gender::Female) => 8,
        (number, _) => number,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Mingua {
    pub lichun_year: i32,
    pub gender: Gender,
    pub raw: u8,
    pub number: u8,
    pub gua: Palace,
    pub group: HouseGroup,
}

impl Mingua {
    pub fn new(lichun_year: i32, gender: Gender) -> Self {
        let raw = mingua_raw(lichun_year, gender);
        let number = mingua(lichun_year, gender);
        let gua = Palace::from_number(number).expect("mingua always maps into 1..=9 except 5");
        Self {
            lichun_year,
            gender,
            raw,
            number,
            gua,
            group: east_west(gua),
        }
    }

    pub const fn jigong(self) -> &'static str {
        if self.raw != 5 {
            ""
        } else {
            match self.gender {
                Gender::Male => "男命五寄坤",
                Gender::Female => "女命五寄艮",
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct YounianEntry {
    pub name: &'static str,
    pub palace: Palace,
    pub auspicious: bool,
}

/// Entries are returned in the stable order 伏位, 生气, 延年, 天医,
/// 祸害, 六煞, 五鬼, 绝命.
pub fn younian(gua: Palace) -> Result<[YounianEntry; 8]> {
    let origin = BAZHAI_ORDER
        .iter()
        .position(|&palace| palace == gua)
        .ok_or(Error::InvalidPalace)?;
    let mut entries = [YounianEntry {
        name: "伏位",
        palace: gua,
        auspicious: true,
    }; 8];
    const NAMES: [&str; 8] = [
        "伏位", "生气", "延年", "天医", "祸害", "六煞", "五鬼", "绝命",
    ];
    for (offset, token) in YOUNIAN_SONG[origin].chars().enumerate() {
        let (index, auspicious) = match token {
            '生' => (1, true),
            '延' => (2, true),
            '天' => (3, true),
            '祸' => (4, false),
            '六' => (5, false),
            '五' => (6, false),
            '绝' => (7, false),
            _ => unreachable!("static younian table"),
        };
        entries[index] = YounianEntry {
            name: NAMES[index],
            palace: BAZHAI_ORDER[(origin + offset + 1) % 8],
            auspicious,
        };
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mingua_vectors_and_five_rule() {
        for (year, gender, expected) in [
            (1985, Gender::Male, 6),
            (1985, Gender::Female, 9),
            (1990, Gender::Male, 1),
            (1990, Gender::Female, 8),
            (1988, Gender::Male, 3),
            (1988, Gender::Female, 3),
            (2000, Gender::Male, 9),
            (2000, Gender::Female, 6),
            (1986, Gender::Male, 2),
            (1977, Gender::Female, 1),
            (2009, Gender::Male, 9),
        ] {
            assert_eq!(mingua(year, gender), expected, "{year} {gender:?}");
        }
        assert_eq!(Mingua::new(1986, Gender::Male).jigong(), "男命五寄坤");
        assert_eq!(Mingua::new(1990, Gender::Female).jigong(), "女命五寄艮");
        for year in 1684..2224 {
            let sum = i32::from(mingua_raw(year, Gender::Male))
                + i32::from(mingua_raw(year, Gender::Female));
            assert_eq!(sum.rem_euclid(9), 6);
        }
    }

    #[test]
    fn younian_kan_and_symmetric() {
        let kan = younian(Palace::Kan).unwrap();
        assert_eq!(kan[1].palace, Palace::Xun);
        assert_eq!(kan[2].palace, Palace::Li);
        assert_eq!(kan[3].palace, Palace::Zhen);
        assert_eq!(kan[4].palace, Palace::Dui);
        assert_eq!(kan[5].palace, Palace::Qian);
        assert_eq!(kan[6].palace, Palace::Gen);
        assert_eq!(kan[7].palace, Palace::Kun);
        for gua in BAZHAI_ORDER {
            let entries = younian(gua).unwrap();
            for entry in entries {
                let other = younian(entry.palace).unwrap();
                let mirror = other.iter().find(|candidate| candidate.name == entry.name);
                assert_eq!(mirror.unwrap().palace, gua);
                assert_eq!(east_west(gua) == east_west(entry.palace), entry.auspicious);
            }
        }
        assert!(younian(Palace::Center).is_err());
    }
}
