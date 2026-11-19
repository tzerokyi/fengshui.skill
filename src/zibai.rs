//! Annual/monthly Purple-White stars and yearly Sha directions.

use crate::core::{fly, ganzhi_of_year, zhi_index_of_year};
use crate::{Error, Result};

const ZHI: [char; 12] = [
    '子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥',
];

const ZHI_DIR: [&str; 12] = [
    "北", "东北", "东北", "东", "东南", "东南", "南", "西南", "西南", "西", "西北", "西北",
];

/// The caller must supply a Li-Chun-adjusted year.
pub fn annual_center(year: i32) -> u8 {
    (10 - year.rem_euclid(9)).rem_euclid(9) as u8 + 1
}

/// Month index is seasonal, not Gregorian: 寅 month=1, 丑 month=12.
pub fn monthly_center(lichun_year: i32, month_index: u8) -> Result<u8> {
    if !(1..=12).contains(&month_index) {
        return Err(Error::InvalidMonthIndex);
    }
    let branch = zhi_index_of_year(lichun_year);
    let start = match branch {
        0 | 3 | 6 | 9 => 8,
        1 | 4 | 7 | 10 => 5,
        _ => 2,
    };
    Ok((start - 1 - (i32::from(month_index) - 1)).rem_euclid(9) as u8 + 1)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YearSha {
    pub year: i32,
    pub ganzhi: String,
    pub center: u8,
    pub taisui: String,
    pub suipo: String,
    pub sansha: String,
    pub wuhuang: &'static str,
    pub erhei: &'static str,
}

pub fn year_sha(year: i32) -> YearSha {
    let branch = zhi_index_of_year(year);
    let opposite = (branch + 6) % 12;
    let (direction, mountains) = match branch {
        0 | 4 | 8 => ("南", "巳午未"),
        2 | 6 | 10 => ("北", "亥子丑"),
        3 | 7 | 11 => ("西", "申酉戌"),
        _ => ("东", "寅卯辰"),
    };
    let annual = fly(annual_center(year), true).expect("1..=9");
    YearSha {
        year,
        ganzhi: ganzhi_of_year(year),
        center: annual_center(year),
        taisui: format!("{}（{}山）", ZHI_DIR[branch], ZHI[branch]),
        suipo: format!("{}（{}山）", ZHI_DIR[opposite], ZHI[opposite]),
        sansha: format!("{direction}（{mountains}）"),
        wuhuang: annual.palace_of(5).expect("permutation").direction(),
        erhei: annual.palace_of(2).expect("permutation").direction(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annual_monthly_vectors() {
        for (year, expected) in [
            (2017, 1),
            (2020, 7),
            (2022, 5),
            (2023, 4),
            (2024, 3),
            (2025, 2),
            (2026, 1),
            (2027, 9),
        ] {
            assert_eq!(annual_center(year), expected);
        }
        assert_eq!(monthly_center(2026, 1).unwrap(), 8);
        assert_eq!(monthly_center(2024, 1).unwrap(), 5);
        assert_eq!(monthly_center(2025, 1).unwrap(), 2);
        assert_eq!(monthly_center(2026, 9).unwrap(), 9);
        assert!(monthly_center(2026, 0).is_err());
        assert!(monthly_center(2026, 13).is_err());
    }

    #[test]
    fn year_sha_vectors() {
        let year = year_sha(2026);
        assert_eq!(year.ganzhi, "丙午");
        assert_eq!(year.center, 1);
        assert_eq!(year.wuhuang, "南");
        assert_eq!(year.erhei, "西北");
        assert!(year.sansha.starts_with('北'));
        assert!(year.suipo.starts_with('北'));
        assert!(year_sha(2024).sansha.starts_with('南'));
        assert!(year_sha(2025).sansha.starts_with('东'));
    }

    #[test]
    fn three_yuan_cycle_against_classic_reference() {
        for year in 1684..2224 {
            let offset = (year - 1864_i32).rem_euclid(180);
            let yuan = (offset / 60) as usize;
            let step = offset % 60;
            let starts = [1, 4, 7];
            let classic = (starts[yuan] - 1 - step).rem_euclid(9) + 1;
            assert_eq!(i32::from(annual_center(year)), classic);
        }
    }
}
