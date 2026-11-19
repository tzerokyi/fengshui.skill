//! Xuan Kong flying stars, replacement stars and chart-pattern identification.

use crate::core::{fly, Mountain, Palace, StarGrid, LUOSHU_PATH};
use crate::{Error, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StarEntry {
    pub n: u8,
    pub by: Mountain,
    pub forward: bool,
    pub center: u8,
}

impl StarEntry {
    pub const fn replaced(self) -> bool {
        self.center != self.n
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeixingChart {
    pub yun: u8,
    pub sitting: Mountain,
    pub facing: Mountain,
    pub sitting_palace: Palace,
    pub facing_palace: Palace,
    pub tigua: bool,
    pub yunpan: StarGrid,
    pub shan: StarGrid,
    pub xiang: StarGrid,
    pub shan_entry: StarEntry,
    pub xiang_entry: StarEntry,
    pub patterns: Vec<&'static str>,
}

pub fn star_direction(n: u8, reference: Mountain) -> Result<(bool, Mountain)> {
    let by = if n == 5 {
        reference
    } else {
        Palace::from_number(n)
            .ok_or(Error::InvalidStar)?
            .mountain(reference.yuanlong())?
    };
    Ok((by.is_yang(), by))
}

fn replacement_star(by: Mountain) -> u8 {
    let mountain = by.symbol();
    if "子癸甲申".contains(mountain) {
        1
    } else if "坤壬乙卯未".contains(mountain) {
        2
    } else if "戌乾亥辰巽巳".contains(mountain) {
        6
    } else if "艮丙辛酉丑".contains(mountain) {
        7
    } else {
        debug_assert!("寅午庚丁".contains(mountain));
        9
    }
}

fn entry(grid: StarGrid, palace: Palace, reference: Mountain, tigua: bool) -> Result<StarEntry> {
    let n = grid.at(palace);
    let (forward, by) = star_direction(n, reference)?;
    let center = if tigua && n != 5 {
        replacement_star(by)
    } else {
        n
    };
    Ok(StarEntry {
        n,
        by,
        forward,
        center,
    })
}

pub fn feixing_chart(yun: u8, sitting: Mountain, tigua: bool) -> Result<FeixingChart> {
    if !(1..=9).contains(&yun) {
        return Err(Error::InvalidYun);
    }
    let facing = sitting.opposite();
    let sitting_palace = sitting.palace();
    let facing_palace = facing.palace();
    let yunpan = fly(yun, true)?;
    let shan_entry = entry(yunpan, sitting_palace, sitting, tigua)?;
    let xiang_entry = entry(yunpan, facing_palace, facing, tigua)?;
    let mut chart = FeixingChart {
        yun,
        sitting,
        facing,
        sitting_palace,
        facing_palace,
        tigua,
        yunpan,
        shan: fly(shan_entry.center, shan_entry.forward)?,
        xiang: fly(xiang_entry.center, xiang_entry.forward)?,
        shan_entry,
        xiang_entry,
        patterns: Vec::new(),
    };
    chart.patterns = chart_patterns(&chart);
    Ok(chart)
}

fn consecutive(stars: [u8; 3]) -> bool {
    (1..=9).any(|start| {
        let mut possible = [
            start,
            start % 9 + 1,
            (start + 1) % 9 + 1,
        ];
        possible.sort_unstable();
        let mut actual = stars;
        actual.sort_unstable();
        possible == actual
    })
}

pub fn chart_patterns(chart: &FeixingChart) -> Vec<&'static str> {
    let yun = chart.yun;
    let sitting = chart.sitting_palace;
    let facing = chart.facing_palace;
    let shan = chart.shan;
    let xiang = chart.xiang;
    let yunpan = chart.yunpan;
    let mut patterns = Vec::new();
    if shan.at(sitting) == yun && xiang.at(facing) == yun {
        patterns.push("旺山旺向（当令山星到坐、向星到向）");
    } else if shan.at(facing) == yun && xiang.at(sitting) == yun {
        patterns.push("上山下水（当令山星到向、向星到坐）");
    } else if shan.at(facing) == yun && xiang.at(facing) == yun {
        patterns.push("双星到向（当令山向二星同到向首）");
    } else if shan.at(sitting) == yun && xiang.at(sitting) == yun {
        patterns.push("双星到坐（当令山向二星同到坐山）");
    }

    for (name, star) in [
        ("山星", chart.shan_entry),
        ("向星", chart.xiang_entry),
    ] {
        if star.center == 5 {
            patterns.push(match (name, star.forward) {
                ("山星", true) => "山星伏吟（五入中顺飞，与洛书元旦盘相同）",
                ("山星", false) => "山星反吟（五入中逆飞，与洛书元旦盘相反）",
                ("向星", true) => "向星伏吟（五入中顺飞，与洛书元旦盘相同）",
                _ => "向星反吟（五入中逆飞，与洛书元旦盘相反）",
            });
        }
    }
    if LUOSHU_PATH
        .iter()
        .all(|&palace| shan.at(palace) + yunpan.at(palace) == 10)
    {
        patterns.push("山星与运星合十");
    }
    if LUOSHU_PATH
        .iter()
        .all(|&palace| xiang.at(palace) + yunpan.at(palace) == 10)
    {
        patterns.push("向星与运星合十");
    }
    if LUOSHU_PATH.iter().all(|&palace| {
        let mut triplet = [
            yunpan.at(palace),
            shan.at(palace),
            xiang.at(palace),
        ];
        triplet.sort_unstable();
        [[1, 4, 7], [2, 5, 8], [3, 6, 9]].contains(&triplet)
    }) {
        patterns.push("父母三般卦（每宫运山向三星成 147/258/369）");
    }
    if LUOSHU_PATH.iter().all(|&palace| {
        consecutive([
            yunpan.at(palace),
            shan.at(palace),
            xiang.at(palace),
        ])
    }) {
        patterns.push("连珠三般卦（每宫运山向三星相连）");
    }
    patterns
}

pub fn main_pattern(chart: &FeixingChart) -> Option<&'static str> {
    match chart.patterns.first().copied() {
        Some("旺山旺向（当令山星到坐、向星到向）") => Some("旺山旺向"),
        Some("上山下水（当令山星到向、向星到坐）") => Some("上山下水"),
        Some("双星到向（当令山向二星同到向首）") => Some("双星到向"),
        Some("双星到坐（当令山向二星同到坐山）") => Some("双星到坐"),
        _ => None,
    }
}

/// Conventional school-dependent star status; not a physical risk assessment.
pub fn star_status(star: u8, yun: u8) -> Result<&'static str> {
    if !(1..=9).contains(&star) {
        return Err(Error::InvalidStar);
    }
    if !(1..=9).contains(&yun) {
        return Err(Error::InvalidYun);
    }
    if star == yun {
        return Ok("当令旺星");
    }
    let delta = (i32::from(star) - i32::from(yun)).rem_euclid(9);
    Ok(match delta {
        1 => "生气（下运旺）",
        2 => "次生气",
        8 => "退气",
        _ if star == 5 => "五黄煞",
        7 => "衰气",
        _ => "死气",
    })
}

pub fn palace_flags(shan: u8, xiang: u8, annual: Option<u8>) -> Vec<&'static str> {
    let mut flags = Vec::new();
    if shan == 5 || xiang == 5 || annual == Some(5) {
        flags.push("五黄");
    }
    let has_two = shan == 2 || xiang == 2 || annual == Some(2);
    let has_five = shan == 5 || xiang == 5 || annual == Some(5);
    if has_two && has_five {
        flags.push("二五交加");
    }
    flags
}

pub fn jian_warnings(sitting: Mountain, offset: f64) -> Result<Vec<String>> {
    if !offset.is_finite() {
        return Err(Error::InvalidBearing);
    }
    let mut warnings = Vec::new();
    if offset.abs() >= 6.0 {
        let adjacent = sitting.neighbor(if offset > 0.0 { 1 } else { -1 });
        let kind = if adjacent.palace() == sitting.palace() {
            "小空亡，同宫异元龙"
        } else {
            "大空亡，出卦"
        };
        warnings.push(format!(
            "⚠ 接近{}/{}交界（{}），立向不清，宜复测",
            sitting.symbol(),
            adjacent.symbol(),
            kind
        ));
    }
    if offset.abs() > 3.0 {
        warnings.push(
            "⚠ 兼向超过 3°，多数玄空派主张用替卦（--tigua）；各派兼向度数界限不一，宜两盘并参"
                .to_owned(),
        );
    }
    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{GRID, MOUNTAINS};

    fn sitting(value: char) -> Mountain {
        Mountain::from_char(value).unwrap()
    }

    fn reading(grid: StarGrid) -> Vec<u8> {
        GRID.into_iter().flatten().map(|palace| grid.at(palace)).collect()
    }

    #[test]
    fn eight_yun_zi_wu_reference() {
        let chart = feixing_chart(8, sitting('子'), false).unwrap();
        assert_eq!(reading(chart.yunpan), [7, 3, 5, 6, 8, 1, 2, 4, 9]);
        assert_eq!(reading(chart.shan), [3, 8, 1, 2, 4, 6, 7, 9, 5]);
        assert_eq!(reading(chart.xiang), [4, 8, 6, 5, 3, 1, 9, 7, 2]);
        assert_eq!(main_pattern(&chart), Some("双星到向"));
        assert_eq!(chart.shan_entry.by.symbol(), '巽');
        assert!(chart.shan_entry.forward);
        assert_eq!(chart.xiang_entry.by.symbol(), '卯');
        assert!(!chart.xiang_entry.forward);
    }

    #[test]
    fn replacements_and_five() {
        let chart = feixing_chart(8, sitting('子'), true).unwrap();
        assert_eq!(chart.shan.at(Palace::Center), 6);
        assert_eq!(chart.xiang.at(Palace::Center), 2);
        assert!(chart.shan_entry.replaced());
        let chart = feixing_chart(9, sitting('子'), true).unwrap();
        assert_eq!(chart.shan_entry.n, 5);
        assert!(!chart.shan_entry.replaced());
        for name in MOUNTAINS {
            assert!((1..=9).contains(&replacement_star(sitting(name))));
        }
    }

    #[test]
    fn chart_patterns_across_24_mountains() {
        let expected_wang7 = "卯乙辰酉辛戌";
        let expected_wang8 = "丑巽巳未乾亥";
        let expected_shang7 = "甲巽巳庚乾亥";
        let expected_shang8 = "艮寅辰坤申戌";
        for yun in 1..=9 {
            for symbol in MOUNTAINS {
                let mountain = sitting(symbol);
                let pattern = main_pattern(&feixing_chart(yun, mountain, false).unwrap());
                if yun == 7 || yun == 8 {
                    assert_eq!(
                        pattern == Some("旺山旺向"),
                        (if yun == 7 { expected_wang7 } else { expected_wang8 }).contains(symbol)
                    );
                    assert_eq!(
                        pattern == Some("上山下水"),
                        (if yun == 7 { expected_shang7 } else { expected_shang8 }).contains(symbol)
                    );
                }
                if yun == 9 {
                    assert!(matches!(pattern, Some("双星到向" | "双星到坐")));
                }
                let reversed = main_pattern(
                    &feixing_chart(yun, mountain.opposite(), false).unwrap(),
                );
                let expected = match pattern {
                    Some("双星到向") => Some("双星到坐"),
                    Some("双星到坐") => Some("双星到向"),
                    other => other,
                };
                assert_eq!(reversed, expected, "yun={yun} mountain={symbol}");
            }
        }
    }

    #[test]
    fn flags_and_warnings() {
        assert_eq!(palace_flags(5, 2, None), ["五黄", "二五交加"]);
        assert_eq!(palace_flags(1, 2, Some(5)), ["五黄", "二五交加"]);
        assert!(palace_flags(1, 3, Some(4)).is_empty());
        assert!(jian_warnings(sitting('子'), 2.0).unwrap().is_empty());
        assert_eq!(jian_warnings(sitting('子'), 4.0).unwrap().len(), 1);
        assert!(jian_warnings(sitting('子'), 7.0).unwrap()[0].contains("小空亡"));
        assert!(jian_warnings(sitting('癸'), 7.0).unwrap()[0].contains("大空亡"));
        assert_eq!(star_status(5, 9).unwrap(), "五黄煞");
    }
}
