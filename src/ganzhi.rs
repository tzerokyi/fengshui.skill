//! Traditional Gan-Zhi lookup and relationship rules.
//!
//! This is a deterministic lookup implementation, not a claim of objectively
//! measurable destiny. No strength, fate or auspiciousness is inferred here.

use crate::{Error, Result};

pub const GAN: [char; 10] = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
pub const ZHI: [char; 12] = [
    '子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥',
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Element {
    Wood,
    Fire,
    Earth,
    Metal,
    Water,
}

impl Element {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wood => "木",
            Self::Fire => "火",
            Self::Earth => "土",
            Self::Metal => "金",
            Self::Water => "水",
        }
    }

    pub const fn produces(self) -> Self {
        match self {
            Self::Wood => Self::Fire,
            Self::Fire => Self::Earth,
            Self::Earth => Self::Metal,
            Self::Metal => Self::Water,
            Self::Water => Self::Wood,
        }
    }

    pub const fn controls(self) -> Self {
        match self {
            Self::Wood => Self::Earth,
            Self::Earth => Self::Water,
            Self::Water => Self::Fire,
            Self::Fire => Self::Metal,
            Self::Metal => Self::Wood,
        }
    }
}

fn gan_index(value: char) -> Result<usize> {
    GAN.iter()
        .position(|&candidate| candidate == value)
        .ok_or(Error::InvalidGanzhi)
}

fn zhi_index(value: char) -> Result<usize> {
    ZHI.iter()
        .position(|&candidate| candidate == value)
        .ok_or(Error::InvalidGanzhi)
}

pub fn wuxing_of(value: char) -> Result<Element> {
    match value {
        '甲' | '乙' | '寅' | '卯' => Ok(Element::Wood),
        '丙' | '丁' | '巳' | '午' => Ok(Element::Fire),
        '戊' | '己' | '辰' | '戌' | '丑' | '未' => Ok(Element::Earth),
        '庚' | '辛' | '申' | '酉' => Ok(Element::Metal),
        '壬' | '癸' | '亥' | '子' => Ok(Element::Water),
        _ => Err(Error::InvalidGanzhi),
    }
}

pub fn is_yang(value: char) -> Result<bool> {
    if let Ok(index) = gan_index(value) {
        Ok(index % 2 == 0)
    } else {
        Ok(zhi_index(value)? % 2 == 0)
    }
}

/// Conventionally used hidden stems in principal/intermediate/residual order.
pub fn hidden_stems(branch: char) -> Result<&'static str> {
    const TABLE: [&str; 12] = [
        "癸", "己癸辛", "甲丙戊", "乙", "戊乙癸", "丙庚戊",
        "丁己", "己丁乙", "庚壬戊", "辛", "戊辛丁", "壬甲",
    ];
    Ok(TABLE[zhi_index(branch)?])
}

/// Other stem's ten-god relation to the day stem.
pub fn ten_god(day_stem: char, other_stem: char) -> Result<&'static str> {
    gan_index(day_stem)?;
    gan_index(other_stem)?;
    let a = wuxing_of(day_stem)?;
    let b = wuxing_of(other_stem)?;
    let same = is_yang(day_stem)? == is_yang(other_stem)?;
    Ok(if a == b {
        if same { "比肩" } else { "劫财" }
    } else if a.produces() == b {
        if same { "食神" } else { "伤官" }
    } else if a.controls() == b {
        if same { "偏财" } else { "正财" }
    } else if b.controls() == a {
        if same { "七杀" } else { "正官" }
    } else if same {
        "偏印"
    } else {
        "正印"
    })
}

pub const TWELVE_PHASES: [&str; 12] = [
    "长生", "沐浴", "冠带", "临官", "帝旺", "衰",
    "病", "死", "墓", "绝", "胎", "养",
];

/// Stem-specific life phase. Yin stems run backward.
pub fn changsheng(stem: char, branch: char) -> Result<&'static str> {
    const START: [char; 10] = ['亥', '午', '寅', '酉', '寅', '酉', '巳', '子', '申', '卯'];
    let stem_index = gan_index(stem)?;
    let start = zhi_index(START[stem_index])? as i32;
    let branch_index = zhi_index(branch)? as i32;
    let step = if stem_index % 2 == 0 {
        branch_index - start
    } else {
        start - branch_index
    };
    Ok(TWELVE_PHASES[step.rem_euclid(12) as usize])
}

/// Seasonal hidden-current ruling sequence; days count from 1 after the Jie.
pub fn siling(month_branch: char, day_n: u32) -> Result<&'static str> {
    let steps: &[(&str, u32)] = match month_branch {
        '寅' => &[("艮土", 5), ("丙火", 5), ("甲木", 20)],
        '卯' => &[("甲木", 7), ("乙木", 23)],
        '辰' => &[("乙木", 7), ("壬水", 5), ("戊土", 18)],
        '巳' => &[("戊土", 7), ("庚金", 5), ("丙火", 18)],
        '午' => &[("丙火", 7), ("丁火", 23)],
        '未' => &[("丁火", 7), ("甲木", 5), ("己土", 18)],
        '申' => &[("坤土", 5), ("壬水", 5), ("庚金", 20)],
        '酉' => &[("庚金", 7), ("辛金", 23)],
        '戌' => &[("辛金", 7), ("丙火", 5), ("戊土", 18)],
        '亥' => &[("戊土", 5), ("甲木", 5), ("壬水", 20)],
        '子' => &[("壬水", 7), ("癸水", 23)],
        '丑' => &[("癸水", 7), ("庚金", 5), ("己土", 18)],
        _ => return Err(Error::InvalidGanzhi),
    };
    if day_n == 0 {
        return Err(Error::InvalidDayOfYear);
    }
    let mut running = 0;
    for &(name, duration) in steps {
        running += duration;
        if day_n <= running {
            return Ok(name);
        }
    }
    Ok(steps.last().expect("fixed non-empty table").0)
}

fn unordered_pair(a: char, b: char, choices: &[(char, char)]) -> bool {
    choices.iter().any(|&(x, y)| (a == x && b == y) || (a == y && b == x))
}

const GAN_HE: [(char, char); 5] = [
    ('甲', '己'), ('乙', '庚'), ('丙', '辛'), ('丁', '壬'), ('戊', '癸'),
];
const LIUHE: [(char, char); 6] = [
    ('子', '丑'), ('寅', '亥'), ('卯', '戌'), ('辰', '酉'), ('巳', '申'), ('午', '未'),
];
const LIUHAI: [(char, char); 6] = [
    ('子', '未'), ('丑', '午'), ('寅', '巳'), ('卯', '辰'), ('申', '亥'), ('酉', '戌'),
];
const XING: [(char, char); 7] = [
    ('寅', '巳'), ('巳', '申'), ('申', '寅'),
    ('丑', '戌'), ('戌', '未'), ('未', '丑'), ('子', '卯'),
];

pub fn clash(a: char, b: char) -> Result<bool> {
    Ok((zhi_index(a)? as i32 - zhi_index(b)? as i32).rem_euclid(12) == 6)
}

/// Relations for a pair of stems or a pair of branches. Mixed pairs are errors.
pub fn pair_relations(a: char, b: char) -> Result<Vec<&'static str>> {
    if gan_index(a).is_ok() && gan_index(b).is_ok() {
        return Ok(if unordered_pair(a, b, &GAN_HE) {
            vec!["合"]
        } else {
            Vec::new()
        });
    }
    zhi_index(a)?;
    zhi_index(b)?;
    let mut relations = Vec::new();
    if unordered_pair(a, b, &LIUHE) {
        relations.push("六合");
    }
    if clash(a, b)? {
        relations.push("冲");
    }
    if a == b && "辰午酉亥".contains(a) {
        relations.push("自刑");
    } else if unordered_pair(a, b, &XING) {
        relations.push("刑");
    }
    if unordered_pair(a, b, &LIUHAI) {
        relations.push("害");
    }
    Ok(relations)
}

/// The two missing earthly branches in a 10-day Gan-Zhi cycle.
pub fn xunkong(pillar: &str) -> Result<String> {
    let mut characters = pillar.chars();
    let stem = gan_index(characters.next().ok_or(Error::InvalidGanzhi)?)?;
    let branch = zhi_index(characters.next().ok_or(Error::InvalidGanzhi)?)?;
    if characters.next().is_some() || stem % 2 != branch % 2 {
        return Err(Error::InvalidGanzhi);
    }
    let branch_first = (branch as i32 - stem as i32).rem_euclid(12) as usize;
    let first = ZHI[(branch_first + 10) % 12];
    let second = ZHI[(branch_first + 11) % 12];
    Ok(format!("{first}{second}"))
}

/// Traditional 旺相休囚死, in Wood/Fire/Earth/Metal/Water order.
pub fn wangxiang(month_branch: char) -> Result<[(Element, &'static str); 5]> {
    let commanding = match month_branch {
        '寅' | '卯' | '辰' => Element::Wood,
        '巳' | '午' => Element::Fire,
        '未' => Element::Earth,
        '申' | '酉' | '戌' => Element::Metal,
        '亥' | '子' | '丑' => Element::Water,
        _ => return Err(Error::InvalidGanzhi),
    };
    let elements = [
        Element::Wood,
        Element::Fire,
        Element::Earth,
        Element::Metal,
        Element::Water,
    ];
    Ok(elements.map(|element| {
        let state = if element == commanding {
            "旺"
        } else if commanding.produces() == element {
            "相"
        } else if element.produces() == commanding {
            "休"
        } else if element.controls() == commanding {
            "囚"
        } else {
            "死"
        };
        (element, state)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_gods_and_hidden_stems() {
        let expected = [
            "比肩", "劫财", "食神", "伤官", "偏财",
            "正财", "七杀", "正官", "偏印", "正印",
        ];
        for (stem, wanted) in GAN.into_iter().zip(expected) {
            assert_eq!(ten_god('甲', stem).unwrap(), wanted);
        }
        assert_eq!(ten_god('丁', '庚').unwrap(), "正财");
        assert_eq!(ten_god('丁', '癸').unwrap(), "七杀");
        assert_eq!(hidden_stems('申').unwrap(), "庚壬戊");
    }

    #[test]
    fn twelve_phases_and_siling() {
        for (stem, birth, death) in [
            ('甲', '亥', '午'), ('乙', '午', '亥'), ('丙', '寅', '酉'),
            ('戊', '寅', '酉'), ('丁', '酉', '寅'), ('己', '酉', '寅'),
            ('庚', '巳', '子'), ('辛', '子', '巳'), ('壬', '申', '卯'), ('癸', '卯', '申'),
        ] {
            assert_eq!(changsheng(stem, birth).unwrap(), "长生");
            assert_eq!(changsheng(stem, death).unwrap(), "死");
        }
        assert_eq!(siling('寅', 5).unwrap(), "艮土");
        assert_eq!(siling('寅', 6).unwrap(), "丙火");
        assert_eq!(siling('寅', 11).unwrap(), "甲木");
        assert_eq!(siling('寅', 31).unwrap(), "甲木");
    }

    #[test]
    fn pairs_seasons_and_xunkong() {
        assert_eq!(pair_relations('子', '午').unwrap(), ["冲"]);
        assert_eq!(pair_relations('子', '丑').unwrap(), ["六合"]);
        assert_eq!(pair_relations('寅', '巳').unwrap(), ["刑", "害"]);
        assert_eq!(pair_relations('午', '午').unwrap(), ["自刑"]);
        assert_eq!(pair_relations('甲', '己').unwrap(), ["合"]);
        assert_eq!(xunkong("甲子").unwrap(), "戌亥");
        assert_eq!(xunkong("癸酉").unwrap(), "戌亥");
        assert_eq!(xunkong("甲戌").unwrap(), "申酉");
        assert_eq!(xunkong("癸亥").unwrap(), "子丑");
        let states = wangxiang('未').unwrap();
        assert_eq!(states[0].1, "囚");
        assert_eq!(states[2].1, "旺");
    }
}
