//! Date-selection scoring as an independent pure Rust API.
//!
//! A caller must supply real, independently calculated almanac facts.
//! This module does not invent lunar dates or ephemeris data.

use crate::ganzhi::ZHI;
use crate::{Error, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Event {
    pub name: &'static str,
    pub keys: &'static [&'static str],
    pub best: &'static str,
    pub acceptable: &'static str,
}

pub const EVENTS: [Event; 11] = [
    Event { name: "入宅", keys: &["入宅", "移徙"], best: "成开", acceptable: "满定" },
    Event { name: "搬家", keys: &["移徙", "入宅"], best: "成开", acceptable: "满定" },
    Event { name: "开业", keys: &["开市", "立券", "交易"], best: "开成", acceptable: "满定" },
    Event { name: "动土", keys: &["动土", "修造"], best: "成开", acceptable: "定" },
    Event { name: "装修", keys: &["修造", "动土"], best: "成开", acceptable: "定" },
    Event { name: "嫁娶", keys: &["嫁娶"], best: "成开", acceptable: "定满" },
    Event { name: "订婚", keys: &["订盟", "纳采"], best: "成开", acceptable: "定满" },
    Event { name: "安葬", keys: &["安葬", "破土"], best: "成开", acceptable: "除" },
    Event { name: "出行", keys: &["出行"], best: "成开", acceptable: "定" },
    Event { name: "签约", keys: &["立券", "交易"], best: "定成", acceptable: "开" },
    Event { name: "安床", keys: &["安床"], best: "定成", acceptable: "开" },
];

pub const MIN_SCORE: i32 = 3;
pub const MAX_RANGE_DAYS: u16 = 400;

pub fn event(name: &str) -> Result<Event> {
    EVENTS.iter().copied().find(|entry| entry.name == name).ok_or(Error::InvalidEvent)
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DayFacts<'a> {
    pub jianchu: &'a str,
    pub chong_zhi: char,
    pub huangdao: bool,
    pub yi: &'a [&'a str],
    pub ji: &'a [&'a str],
    pub jishen: &'a [&'a str],
    pub flags: &'a [&'a str],
    pub suji: &'a [&'a str],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DayScore {
    pub score: i32,
    pub reasons: Vec<String>,
    pub excluded: Vec<String>,
    pub notes: Vec<String>,
}

impl DayScore {
    pub fn eligible(&self) -> bool {
        self.excluded.is_empty() && self.score >= MIN_SCORE
    }
}

/// Score a fully constructed day. Birth years are treated as sexagenary years.
pub fn score_day(facts: &DayFacts<'_>, event: Event, birth_years: &[i32]) -> Result<DayScore> {
    if !ZHI.contains(&facts.chong_zhi) {
        return Err(Error::InvalidGanzhi);
    }
    let mut result = DayScore {
        score: 0,
        reasons: Vec::new(),
        excluded: facts.flags.iter().map(|value| (*value).to_owned()).collect(),
        notes: facts.suji.iter().map(|value| format!("通书俗忌：{value}")).collect(),
    };
    const ANIMALS: [char; 12] = [
        '鼠', '牛', '虎', '兔', '龙', '蛇', '马', '羊', '猴', '鸡', '狗', '猪',
    ];
    for &birth_year in birth_years {
        let index = (birth_year - 4).rem_euclid(12) as usize;
        if facts.chong_zhi == ZHI[index] {
            result.excluded.push(format!(
                "冲{birth_year}年生人（{}{}）",
                ZHI[index], ANIMALS[index],
            ));
        }
    }
    let avoided: Vec<&str> = event.keys.iter().copied()
        .filter(|keyword| facts.ji.contains(keyword))
        .collect();
    if !avoided.is_empty() {
        result.excluded.push(format!("黄历忌{}", avoided.join("/")));
    }
    let recommended: Vec<&str> = event.keys.iter().copied()
        .filter(|keyword| facts.yi.contains(keyword))
        .collect();
    if !recommended.is_empty() {
        result.score += 3;
        result.reasons.push(format!("宜{}", recommended.join("/")));
    }
    if event.best.contains(facts.jianchu) {
        result.score += 2;
        result.reasons.push(format!("{}日", facts.jianchu));
    } else if event.acceptable.contains(facts.jianchu) {
        result.score += 1;
        result.reasons.push(format!("{}日(次)", facts.jianchu));
    }
    if facts.huangdao {
        result.score += 1;
        result.reasons.push("黄道".to_owned());
    }
    let de: Vec<&str> = facts.jishen.iter().copied()
        .filter(|name| ["天德", "月德", "天德合", "月德合", "天赦", "天恩", "母仓"].contains(name))
        .collect();
    if !de.is_empty() {
        let major = de.iter().any(|name| ["天德", "月德", "天赦"].contains(name));
        result.score += if major { 2 } else { 1 };
        result.reasons.push(de.join("、"));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base<'a>() -> DayFacts<'a> {
        DayFacts {
            jianchu: "平",
            chong_zhi: '子',
            ..DayFacts::default()
        }
    }

    #[test]
    fn exact_score_vectors() {
        let ev = event("入宅").unwrap();
        let rich = DayFacts {
            yi: &["入宅"],
            jianchu: "成",
            huangdao: true,
            jishen: &["天德"],
            ..base()
        };
        let result = score_day(&rich, ev, &[]).unwrap();
        assert_eq!(result.score, 8);
        assert!(result.excluded.is_empty());
        assert!(result.eligible());
        let minor = DayFacts { jishen: &["母仓"], ..base() };
        assert_eq!(score_day(&minor, ev, &[]).unwrap().score, 1);
        let secondary = DayFacts { jianchu: "满", ..base() };
        assert_eq!(score_day(&secondary, ev, &[]).unwrap().score, 1);
    }

    #[test]
    fn hard_exclusions_are_separate_from_score() {
        let facts = DayFacts {
            flags: &["月破"],
            ji: &["移徙"],
            yi: &["入宅"],
            ..base()
        };
        let result = score_day(&facts, event("入宅").unwrap(), &[1984]).unwrap();
        assert_eq!(result.excluded, ["月破", "冲1984年生人（子鼠）", "黄历忌移徙"]);
        assert!(!result.eligible());
        assert!(event("拜拜").is_err());
    }
}
