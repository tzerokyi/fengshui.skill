//! Zero-dependency CLI adapter for kanyu_core.
//!
//! JSON schema version 1; stdout is data, stderr is diagnostics.

use kanyu_core::almanac::day_flags;
use kanyu_core::ganzhi::ten_god;
use kanyu_core::zeri::{event, score_day, DayFacts};
use kanyu_core::bazhai::{east_west, younian, Gender, Mingua};
use kanyu_core::core::{yun_of_year, Mountain, Palace, StarGrid, GRID, LUOSHU_PATH};
use kanyu_core::feixing::{feixing_chart, jian_warnings, main_pattern, FeixingChart, StarEntry};
use kanyu_core::zibai::{annual_center, monthly_center, year_sha, YearSha};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

const HELP: &str = "\
kanyu: native deterministic feng shui core (schema v1)

Usage:
  kanyu <command> [options] [--json]
  kanyu --help
  kanyu --version

Commands:
  doctor        Inspect native capabilities (no display or network)
  ten-god       Gan-Zhi ten-god lookup
  day-flags     Pure date taboo lookup (no ephemeris)
  zeri-score    Score externally supplied almanac day facts
  mingua        Ming Gua for an already Li-Chun-adjusted birth year
  bazhai        House/person Eight Mansions (Ba Zhai)
  feixing       Xuan Kong flying-star chart
  liunian       Annual/monthly Purple-White stars and yearly Sha directions

Examples:
  kanyu ten-god --day-stem 甲 --other-stem 庚 --json
  kanyu day-flags --year-zhi 午 --month-zhi 巳 --day-gz 癸巳 --jianchu 建 --json
  kanyu mingua --year 1990 --gender f --json
  kanyu bazhai --facing 180 --year 1990 --gender f --json
  kanyu feixing --sit 子 --yun 8 --annual 2026 --json
  kanyu feixing --facing 180 --year 2018 --tigua --json
  kanyu liunian --year 2026 --span 5 --json

Important:
  --year on mingua/bazhai and --year on feixing is a Li-Chun-adjusted
  effective year, NOT an unconverted Gregorian birthday or occupancy date.
  --month-index is a solar-term month (寅=1 ... 丑=12).
  Date/time, bazi, zeri, city maps and HTML rendering are NOT ported yet.
  No implicit 4 February approximation is used.
";

#[derive(Debug, Default)]
struct Args {
    values: BTreeMap<String, String>,
    switches: BTreeSet<String>,
}

impl Args {
    fn parse(args: &[String], allowed: &[&str], switches: &[&str]) -> Result<Self, String> {
        let mut parsed = Self::default();
        let mut index = 0;
        while index < args.len() {
            let option = &args[index];
            if option == "--json" || switches.contains(&option.as_str()) {
                if !parsed.switches.insert(option.clone()) {
                    return Err(format!("duplicate flag: {option}"));
                }
                index += 1;
            } else if allowed.contains(&option.as_str()) {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| format!("missing value for {option}"))?;
                if value.starts_with("--") {
                    return Err(format!("missing value for {option}"));
                }
                if parsed.values.insert(option.clone(), value.clone()).is_some() {
                    return Err(format!("duplicate option: {option}"));
                }
                index += 2;
            } else {
                return Err(format!("unknown option: {option}"));
            }
        }
        Ok(parsed)
    }

    fn has(&self, name: &str) -> bool {
        self.switches.contains(name)
    }

    fn value(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }

    fn required(&self, name: &str) -> Result<&str, String> {
        self.value(name)
            .ok_or_else(|| format!("missing required option {name}"))
    }

    fn number(&self, name: &str) -> Result<Option<i32>, String> {
        self.value(name)
            .map(|value| {
                value
                    .parse::<i32>()
                    .map_err(|_| format!("{name} must be an integer"))
            })
            .transpose()
    }

    fn required_number(&self, name: &str) -> Result<i32, String> {
        self.number(name)?
            .ok_or_else(|| format!("missing required option {name}"))
    }
}

fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            character if character < '\u{20}' => {
                write!(out, "\\u{:04x}", character as u32).expect("in-memory formatting");
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn object(fields: &[(&str, String)]) -> String {
    let entries: Vec<String> = fields
        .iter()
        .map(|(key, value)| format!("{}:{}", quote(key), value))
        .collect();
    format!("{{{}}}", entries.join(","))
}

fn array(values: &[String]) -> String {
    format!("[{}]", values.join(","))
}

fn strings(values: impl IntoIterator<Item = String>) -> String {
    array(&values.into_iter().map(|value| quote(&value)).collect::<Vec<_>>())
}

fn star_grid_json(grid: StarGrid) -> String {
    let entries: Vec<(&str, String)> = LUOSHU_PATH
        .iter()
        .map(|&palace| (palace.name(), grid.at(palace).to_string()))
        .collect();
    object(&entries)
}

fn younian_json(palace: Palace) -> Result<String, String> {
    let entries = younian(palace).map_err(|error| error.to_string())?;
    Ok(object(
        &entries
            .iter()
            .map(|entry| (entry.name, quote(entry.palace.name())))
            .collect::<Vec<_>>(),
    ))
}

fn mingua_json(info: Mingua) -> Result<String, String> {
    Ok(object(&[
        ("lichun_year", info.lichun_year.to_string()),
        ("gender", quote(info.gender.as_str())),
        ("raw", info.raw.to_string()),
        ("number", info.number.to_string()),
        ("gua", quote(info.gua.name())),
        ("group", quote(info.group.as_str())),
        ("jigong", quote(info.jigong())),
        ("younian", younian_json(info.gua)?),
    ]))
}

fn person(args: &Args) -> Result<Mingua, String> {
    let year = args.required_number("--year")?;
    let gender: Gender = args
        .required("--gender")?
        .parse()
        .map_err(|error: kanyu_core::Error| error.to_string())?;
    Ok(Mingua::new(year, gender))
}

fn parse_direction(input: &str) -> Result<(Mountain, Option<f64>), String> {
    let trimmed = input.trim();
    let numeric = trimmed
        .trim_end_matches(|c: char| c == '°' || c == '度')
        .parse::<f64>()
        .ok();
    let mountain = trimmed
        .parse::<Mountain>()
        .map_err(|error| format!("{error}: {input}"))?;
    Ok((mountain, numeric.map(|degrees| degrees.rem_euclid(360.0))))
}

fn sitting(args: &Args) -> Result<(Mountain, Option<f64>), String> {
    match (args.value("--sit"), args.value("--facing")) {
        (Some(_), Some(_)) => Err("provide exactly one of --sit or --facing".to_owned()),
        (Some(value), None) => {
            let (mountain, degrees) = parse_direction(value)?;
            Ok((mountain, degrees))
        }
        (None, Some(value)) => {
            let (facing, degrees) = parse_direction(value)?;
            Ok((facing.opposite(), degrees.map(|d| (d + 180.0).rem_euclid(360.0))))
        }
        (None, None) => Err("provide --sit or --facing".to_owned()),
    }
}

fn star_entry_json(entry: StarEntry) -> String {
    object(&[
        ("n", entry.n.to_string()),
        ("by", quote(&entry.by.symbol().to_string())),
        ("forward", entry.forward.to_string()),
        ("center", entry.center.to_string()),
        ("replaced", entry.replaced().to_string()),
    ])
}

fn feixing_json(
    chart: &FeixingChart,
    annual: Option<u8>,
    monthly: Option<u8>,
    sitting_degrees: Option<f64>,
) -> Result<String, String> {
    let warnings = if let Some(degrees) = sitting_degrees {
        let offset = Mountain::from_bearing(degrees)
            .map_err(|error| error.to_string())?
            .1;
        jian_warnings(chart.sitting, offset).map_err(|error| error.to_string())?
    } else {
        Vec::new()
    };
    let pattern = main_pattern(chart).map(quote).unwrap_or_else(|| "null".to_owned());
    Ok(object(&[
        ("schema_version", "1".to_owned()),
        ("yun", chart.yun.to_string()),
        ("sitting", quote(&chart.sitting.symbol().to_string())),
        ("facing", quote(&chart.facing.symbol().to_string())),
        ("sitting_palace", quote(chart.sitting_palace.name())),
        ("facing_palace", quote(chart.facing_palace.name())),
        ("tigua", chart.tigua.to_string()),
        ("yunpan", star_grid_json(chart.yunpan)),
        ("shan", star_grid_json(chart.shan)),
        ("xiang", star_grid_json(chart.xiang)),
        ("shan_entry", star_entry_json(chart.shan_entry)),
        ("xiang_entry", star_entry_json(chart.xiang_entry)),
        ("patterns", strings(chart.patterns.iter().map(|value| value.to_string()))),
        ("main_pattern", pattern),
        ("annual_center", annual.map_or_else(|| "null".to_owned(), |n| n.to_string())),
        ("month_center", monthly.map_or_else(|| "null".to_owned(), |n| n.to_string())),
        ("sitting_degrees", sitting_degrees.map_or_else(|| "null".to_owned(), |n| n.to_string())),
        ("warnings", strings(warnings)),
    ]))
}

fn year_sha_json(year: &YearSha) -> Result<String, String> {
    let months = (1..=12)
        .map(|month| monthly_center(year.year, month).map(|n| n.to_string()))
        .collect::<kanyu_core::Result<Vec<_>>>()
        .map_err(|error| error.to_string())?;
    Ok(object(&[
        ("year", year.year.to_string()),
        ("ganzhi", quote(&year.ganzhi)),
        ("center", year.center.to_string()),
        ("taisui", quote(&year.taisui)),
        ("suipo", quote(&year.suipo)),
        ("sansha", quote(&year.sansha)),
        ("wuhuang", quote(year.wuhuang)),
        ("erhei", quote(year.erhei)),
        ("months", array(&months)),
    ]))
}

fn run(argv: &[String]) -> Result<String, String> {
    if argv.is_empty() || argv[0] == "--help" || argv[0] == "-h" {
        return Ok(HELP.to_owned());
    }
    if argv[0] == "--version" {
        return Ok(format!("kanyu {}", env!("CARGO_PKG_VERSION")));
    }
    let command = argv[0].as_str();
    if argv[1..].iter().any(|arg| arg == "--help" || arg == "-h") {
        return Ok(HELP.to_owned());
    }
    match command {
        "doctor" => {
            let args = Args::parse(&argv[1..], &[], &[])?;
            if args.has("--json") {
                Ok(object(&[
                    ("schema_version", "1".to_owned()),
                    ("runtime", quote("native-rust")),
                    ("crate", quote("kanyu-core")),
                    (
                        "implemented",
                        strings(["mingua", "bazhai", "feixing", "liunian", "ten-god", "day-flags", "zeri-score"].map(|s| s.to_owned())),
                    ),
                    (
                        "not_ported",
                        strings(["day", "zeri", "bazi", "city-wuxing-map", "html-render"].map(|s| s.to_owned())),
                    ),
                    ("mcp_transport", "false".to_owned()),
                ]))
            } else {
                Ok("Native Rust core: mingua, bazhai, feixing, liunian, ten-god, day-flags, zeri-score\nPending: day, zeri, bazi, city-wuxing-map, HTML renderer\nMCP adapter: not installed".to_owned())
            }
        }
        "ten-god" => {
            let args = Args::parse(&argv[1..], &["--day-stem", "--other-stem"], &[])?;
            let single = |name: &str| -> Result<char, String> {
                let value = args.required(name)?;
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => Ok(c),
                    _ => Err(format!("{name} must contain one Gan character")),
                }
            };
            let day = single("--day-stem")?;
            let other = single("--other-stem")?;
            let result = ten_god(day, other).map_err(|error| error.to_string())?;
            if args.has("--json") {
                Ok(object(&[
                    ("schema_version", "1".to_owned()),
                    ("command", quote("ten-god")),
                    ("day_stem", quote(&day.to_string())),
                    ("other_stem", quote(&other.to_string())),
                    ("relation", quote(result)),
                ]))
            } else {
                Ok(format!("{day}日干对{other}：{result}"))
            }
        }
        "day-flags" => {
            let args = Args::parse(&argv[1..], &["--year-zhi", "--month-zhi", "--day-gz", "--jianchu"], &[])?;
            let single = |name: &str| -> Result<char, String> {
                let value = args.required(name)?;
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => Ok(c),
                    _ => Err(format!("{name} must contain one Zhi character")),
                }
            };
            let flags = day_flags(
                single("--year-zhi")?,
                single("--month-zhi")?,
                args.required("--day-gz")?,
                args.required("--jianchu")?,
            ).map_err(|error| error.to_string())?;
            if args.has("--json") {
                Ok(object(&[
                    ("schema_version", "1".to_owned()),
                    ("command", quote("day-flags")),
                    ("flags", strings(flags.flags.iter().map(|s| s.to_string()))),
                    ("conventional", strings(flags.conventional.iter().map(|s| s.to_string()))),
                ]))
            } else {
                Ok(format!("硬排除:{} 俗忌:{}", flags.flags.join("、"), flags.conventional.join("、")))
            }
        }
        "zeri-score" => {
            let args = Args::parse(
                &argv[1..],
                &["--event", "--jianchu", "--chong-zhi", "--yi", "--ji", "--jishen",
                  "--flags", "--suji", "--birth-years"],
                &["--huangdao"],
            )?;
            let ev = event(args.required("--event")?).map_err(|error| error.to_string())?;
            let branch = args.required("--chong-zhi")?;
            let mut chars = branch.chars();
            let chong_zhi = match (chars.next(), chars.next()) {
                (Some(c), None) => c,
                _ => return Err("--chong-zhi must be one Zhi character".to_owned()),
            };
            let csv = |option: &str| -> Vec<&str> {
                args.value(option)
                    .unwrap_or("")
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect()
            };
            let yi = csv("--yi");
            let ji = csv("--ji");
            let jishen = csv("--jishen");
            let flags = csv("--flags");
            let suji = csv("--suji");
            let birth_years = csv("--birth-years").into_iter()
                .map(|n| n.parse::<i32>().map_err(|_| format!("invalid birth year: {n}")))
                .collect::<Result<Vec<_>,_>>()?;
            let facts = DayFacts {
                jianchu: args.required("--jianchu")?,
                chong_zhi,
                huangdao: args.has("--huangdao"),
                yi: &yi,
                ji: &ji,
                jishen: &jishen,
                flags: &flags,
                suji: &suji,
            };
            let result = score_day(&facts, ev, &birth_years)
                .map_err(|error| error.to_string())?;
            if args.has("--json") {
                Ok(object(&[
                    ("schema_version", "1".to_owned()),
                    ("command", quote("zeri-score")),
                    ("event", quote(ev.name)),
                    ("score", result.score.to_string()),
                    ("eligible", result.eligible().to_string()),
                    ("reasons", strings(result.reasons)),
                    ("excluded", strings(result.excluded)),
                    ("notes", strings(result.notes)),
                ]))
            } else {
                Ok(format!(
                    "{}: {}점, 후보={}, 제외={}, 근거={}",
                    ev.name, result.score, result.eligible(),
                    result.excluded.join("、"), result.reasons.join("、"),
                ))
            }
        }
        "mingua" => {
            let args = Args::parse(&argv[1..], &["--year", "--gender"], &[])?;
            let result = person(&args)?;
            if args.has("--json") {
                Ok(object(&[
                    ("schema_version", "1".to_owned()),
                    ("command", quote("mingua")),
                    ("person", mingua_json(result)?),
                ]))
            } else {
                Ok(format!(
                    "{} ({}) {}命卦 {} / {} {}",
                    result.lichun_year,
                    result.gender.as_str(),
                    result.number,
                    result.gua.name(),
                    result.group.as_str(),
                    result.jigong()
                ))
            }
        }
        "bazhai" => {
            let args = Args::parse(
                &argv[1..],
                &["--sit", "--facing", "--year", "--gender"],
                &[],
            )?;
            let house = if args.value("--sit").is_some() || args.value("--facing").is_some() {
                Some(sitting(&args)?.0)
            } else {
                None
            };
            let person = if args.value("--year").is_some() {
                Some(person(&args)?)
            } else if args.value("--gender").is_some() {
                return Err("--gender requires --year".to_owned());
            } else {
                None
            };
            if house.is_none() && person.is_none() {
                return Err("provide --sit/--facing and/or --year with --gender".to_owned());
            }
            let mut entries = vec![
                ("schema_version", "1".to_owned()),
                ("command", quote("bazhai")),
            ];
            if let Some(mountain) = house {
                let palace = mountain.palace();
                entries.push((
                    "house",
                    object(&[
                        ("sitting", quote(&mountain.symbol().to_string())),
                        ("facing", quote(&mountain.opposite().symbol().to_string())),
                        ("gua", quote(palace.name())),
                        ("group", quote(east_west(palace).as_str())),
                        ("younian", younian_json(palace)?),
                    ]),
                ));
            }
            if let Some(info) = person {
                entries.push(("person", mingua_json(info)?));
                if let Some(mountain) = house {
                    entries.push((
                        "match",
                        (east_west(mountain.palace()) == info.group).to_string(),
                    ));
                }
            }
            if args.has("--json") {
                Ok(object(&entries))
            } else {
                Ok(format!(
                    "八宅: {}",
                    entries
                        .iter()
                        .skip(2)
                        .map(|(name, value)| format!("{name}={value}"))
                        .collect::<Vec<_>>()
                        .join(" ")
                ))
            }
        }
        "feixing" => {
            let args = Args::parse(
                &argv[1..],
                &[
                    "--sit", "--facing", "--yun", "--year", "--annual",
                    "--month-index", "--month-year",
                ],
                &["--tigua"],
            )?;
            let (mountain, sitting_degrees) = sitting(&args)?;
            let yun = if let Some(n) = args.number("--yun")? {
                if args.value("--year").is_some() {
                    return Err("provide either --yun or --year, not both".to_owned());
                }
                u8::try_from(n).map_err(|_| "--yun must be 1..=9".to_owned())?
            } else {
                yun_of_year(args.required_number("--year")?)
            };
            let chart = feixing_chart(yun, mountain, args.has("--tigua"))
                .map_err(|error| error.to_string())?;
            let annual = args.number("--annual")?.map(annual_center);
            let month = match (args.number("--month-year")?, args.number("--month-index")?) {
                (None, None) => None,
                (Some(year), Some(index)) => {
                    let index = u8::try_from(index)
                        .map_err(|_| "--month-index must be 1..=12".to_owned())?;
                    Some(monthly_center(year, index).map_err(|error| error.to_string())?)
                }
                _ => return Err("--month-year and --month-index must be provided together".to_owned()),
            };
            if args.has("--json") {
                feixing_json(&chart, annual, month, sitting_degrees)
            } else {
                let mut rows = Vec::new();
                for row in GRID {
                    rows.push(
                        row.iter()
                            .map(|&palace| {
                                format!(
                                    "{}:{}/{}/{}",
                                    palace.name(),
                                    chart.yunpan.at(palace),
                                    chart.shan.at(palace),
                                    chart.xiang.at(palace),
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("  "),
                    );
                }
                Ok(format!(
                    "{}山{}向 {}运\n{}\n格局: {}",
                    chart.sitting.symbol(),
                    chart.facing.symbol(),
                    chart.yun,
                    rows.join("\n"),
                    main_pattern(&chart).unwrap_or("其他格局")
                ))
            }
        }
        "liunian" => {
            let args = Args::parse(&argv[1..], &["--year", "--span"], &[])?;
            let year = args.required_number("--year")?;
            let span = args.number("--span")?.unwrap_or(1);
            if !(1..=100).contains(&span) {
                return Err("--span must be 1..=100".to_owned());
            }
            let end = year.checked_add(span)
                .ok_or_else(|| "--year plus --span overflows".to_owned())?;
            let results = (year..end).map(year_sha).collect::<Vec<_>>();
            if args.has("--json") {
                let entries = results.iter().map(year_sha_json).collect::<Result<Vec<_>, _>>()?;
                Ok(object(&[
                    ("schema_version", "1".to_owned()),
                    ("command", quote("liunian")),
                    ("years", array(&entries)),
                ]))
            } else {
                Ok(results
                    .iter()
                    .map(|item| format!(
                        "{} {} 中宫{} 太岁{} 岁破{} 三煞{} 五黄{} 二黑{}",
                        item.year, item.ganzhi, item.center, item.taisui, item.suipo,
                        item.sansha, item.wuhuang, item.erhei
                    ))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }
        }
        _ => Err(format!("unknown command: {command}; use --help")),
    }
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match run(&args) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("kanyu: {error}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(args: &[&str]) -> String {
        run(&args.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>()).unwrap()
    }

    #[test]
    fn json_escaping_and_structural_shape() {
        assert_eq!(quote("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"");
        assert_eq!(quote("\u{0001}"), "\"\\u0001\"");
        let payload = cmd(&["feixing", "--sit", "子", "--yun", "8", "--json"]);
        assert!(payload.starts_with("{\"schema_version\":1,\"yun\":8,"));
        assert!(payload.contains("\"main_pattern\":\"双星到向\""));
        assert!(payload.contains("\"sitting\":\"子\""));
        assert!(payload.contains("\"yunpan\":{\"中\":8"));
        let mg = cmd(&["mingua", "--year", "1990", "--gender", "f", "--json"]);
        assert!(mg.contains("\"number\":8"));
        assert!(mg.contains("\"jigong\":\"女命五寄艮\""));
        let years = cmd(&["liunian", "--year", "2026", "--span", "1", "--json"]);
        assert!(years.contains("\"wuhuang\":\"南\""));
    }

    #[test]
    fn invalid_options_fail_without_output() {
        assert!(run(&["feixing", "--sit", "子", "--yun", "0"].map(|s| s.to_owned())).is_err());
        assert!(run(&["mingua", "--year", "2026"].map(|s| s.to_owned())).is_err());
        assert!(run(&["feixing", "--sit", "子", "--yun", "8", "--month-index", "1"].map(|s| s.to_owned())).is_err());
        assert!(run(&["liunian", "--year", "2026", "--span", "999999"].map(|s| s.to_owned())).is_err());
        assert!(run(&["doctor", "--secret"].map(|s| s.to_owned())).is_err());
    }
}
