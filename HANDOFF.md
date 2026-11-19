# Rustification handoff — fengshui.skill

> **2026-11-19 · owner-requested main integration · unfinished migration**
>
> **CURRENT STATE: NOT 100% RUST. NOT VERIFIED.**
> The owner explicitly instructed integrating the work into `main` now,
> **with validation deferred**. Do not interpret a merged commit as a claim of
> compilation success, runtime readiness, numerical parity, or complete porting.

## Goal / invariants

- Replace all executable Feng Shui, Ba Zi, almanac, date-selection and
  city Five-Element computations with **native Rust**; no Python fallback,
  no JS/HTML/WebView runtime, no UI work required.
- Make a **reusable typed crate core** (`src/lib.rs` / `kanyu_core`) with a
  thin cross-platform terminal CLI (`src/bin/kanyu.rs`).
  Library business logic must never depend on CLI, stdout, or OS plumbing.
- Keep algorithms deterministic and clearly distinguish **traditional lookup
  conventions** from real-world evidence. Never silently fabricate calendar,
  solar-term or geographical facts.
- Favor modules, reusable structs/enums/results, minimal dependencies, future
  crate extraction, stable JSON schemas and explicit error contracts.
- Keep source only in Git, never commit `target/`, built binaries or caches.
- `tzerokyi/rustryx` is a reference-only archive, not an editing target.
  No graphical frontend is needed in this repository.

## What has been ported

| Rust source | Functionality | Status |
| --- | --- | --- |
| `src/core.rs` | 24 mountains, bearing/compass, Lo Shu, 9 periods | Initial implementation |
| `src/bazhai.rs` | Mingua, eight-mansion, You Nian | Initial implementation |
| `src/feixing.rs` | Xuan Kong flying stars, substitute stars and patterns | Initial implementation |
| `src/zibai.rs` | Annual/monthly stars, year Sha | Initial implementation |
| `src/ganzhi.rs` | Five elements, hidden stems, ten gods, twelve phases, month rulers, stem/branch paired relations, empty branches, seasonal strength table | Partial |
| `src/almanac.rs` | Pure almanac taboo rules, approximate equation-of-time/solar offset | Partial |
| `src/zeri.rs` | Event definitions, hard exclusion and weighted scoring from **supplied** day facts | Partial |
| `src/bin/kanyu.rs` | `doctor`, `mingua`, `bazhai`, `feixing`, `liunian`, `ten-god`, `day-flags`, `zeri-score`; JSON/text | Partial adapter |

The Rust program does not execute Python. However the repository **still
contains** the original Python skill implementations and HTML/JS assets.
They are retained as parity oracles / legacy resources, not Rust equivalents.

## Remaining implementation work (priority order)

### P0 — complete accurate calendar and time primitives

- Port `scripts/kanyu/almanac.py` entirely: ISO/date parser, exact solar
  terms (including Li Chun boundaries), Gan-Zhi month/day/hour, astronomical or
  sourced lunar↔solar conversion, conventional almanac listings and rules.
- Introduce a trustworthy, versioned ephemeris/calendar dataset or native Rust
  calculation engine with explicit supported date range and accuracy.
  The current `src/almanac.rs` **cannot** provide `day_info`.
- Port fixed-offset and IANA time zones including historical DST, ambiguous
  and nonexistent local times, longitude-based true solar time, standard/
  Beijing/local time conversions. Do not silently replace these with
  February 4 / sixth-of-month approximations.
- Preserve day-boundary and solar-term exactness; include fixtures spanning
  Li Chun, month boundaries, leap months, DST switches and overseas births.

### P1 — complete Ba Zi / Gan-Zhi library

- Port `scripts/kanyu/bazi.py`, remaining `scripts/kanyu/ganzhi.py`
  (three-combinations, three-meetings, full relationship graph, shensha,
  xunkong, luck-cycle lookup and traditional annotations), and the
  `scripts/kanyu/data/qiongtong.json` lookup data.
- Define typed `BirthMoment`, `Pillar`, `BaZiChart`, `LuckCycle` and
  structured provenance/assumption fields in stable public API.
- Ensure the independent `skills/bazi-mingli/scripts/kanyu/` Python copy is
  not treated as another source of truth; reconcile against the original.

### P2 — date selection end-to-end

- Finish `scripts/kanyu/zeri.py` as `src/zeri.rs`: construct real
  `DayInfo` from the calendar provider, walk inclusive date ranges,
  reject over-400-day requests, hard exclusions, notes, sort good/bad, report
  period-dependent Sha. Existing `score_day` is only the pure-rule layer.
- Match `tests/test_zeri.py` and `tests/test_almanac.py` reference vectors.

### P3 — city Five-Element geographical engine (no UI)

- Port **all computational** logic under
  `skills/city-wuxing-map/scripts/{city_model,generate,build_full_city}.py`
  to Rust, with modular input/geometry/classification/output APIs and a
  native CLI command/subcommand.
- Support the documented geodata formats, complete claimed coverage,
  spatial queries, deterministic classifications, source provenance,
  confidence and attribution; do not invent data for absent coverage.
- Keep legacy `assets/map.html`, Leaflet JS and rendered examples only
  as non-runtime references until output compatibility is sorted.
  A native Rust **frontend is out of scope**; focus on data/engine only.

### P4 — CLI, packaging, parity and cleanup

- Port any remaining active `scripts/kanyu/cli.py`, `render.py` and
  `scripts/fengshui.py` behaviors that constitute CLI/data APIs.
  Traditional narrative/HTML rendering is **not** a mandate to build a new UI.
- Add complete commands and structured `--json` contracts, strict argument
  validation, stable failures (exit code 2 for invalid input), help/examples.
- Route all skill instructions to the Rust binary and remove Python runtime
  dependency **only when** the matching Rust behavior exists.
- Make reusable Rust crate interfaces independent of project installation.
  Maintain Windows portability, path handling and Unicode support.
- Review dependencies for minimalism; keep publication disabled until
  verified licensing and third-party geography-data attribution are resolved.

## Mandatory validation later (owner explicitly deferred)

Current state: **No verified Rust build, Clippy, rustfmt, or binary smoke
result**. Rust self-hosted runner jobs were queued/pending when this
handoff was written. Passing Python/Agent Skills CI is *not* Rust verification.

Run on the self-hosted Linux Actions runner with 12 Cargo jobs and
persistent cache; the workflow is `.github/workflows/rust-native.yml`:

```sh
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release --bin kanyu
cargo run --locked --bin kanyu -- doctor --json
cargo run --locked --bin kanyu -- ten-god --day-stem 甲 --other-stem 庚 --json
cargo run --locked --bin kanyu -- day-flags --year-zhi 午 --month-zhi 巳 --day-gz 癸巳 --jianchu 建 --json
cargo run --locked --bin kanyu -- feixing --sit 子 --yun 8 --json
```

**Important:** Previously added Rust source was not yet run through rustfmt.
A follow-up must commit exactly formatted source before relying on strict
Clippy/test/build gates. Audit `src/bin/kanyu.rs` argument parser and newly
added commands; verify `src/zeri.rs` day scoring and all Rust unit tests.
Compare computed output to Python reference fixtures, especially
`tests/test_{core,bazhai,feixing,zibai,ganzhi,almanac,zeri}.py`.
Add Windows target build/runtime coverage in a later phase.

Only once a gate actually runs and passes may it be described as passing.

## Release / license constraints

The fork contains no clear top-level LICENSE. `Cargo.toml` deliberately
uses `publish = false`. Do not infer permission to publish to crates.io,
relicense original content, or distribute embedded geographic datasets
without examining their licenses and attributions.

## Source-of-truth and next agent instructions

- Primary native code: `src/`, `Cargo.toml`; stage subsequent Rust changes
  in small coherent feature branches. The owner approved **this one
  exception** to normal validation-before-main policy; future changes
  should still format and validate before merging unless explicitly told
  otherwise.
- Progress matrix: `docs/RUSTIFY_STATUS.md`. This handoff is the current
  outstanding-work checklist; update both as modules are completed.
- Original Python rules: `scripts/kanyu/`; bazi copy:
  `skills/bazi-mingli/scripts/kanyu/`; city logic:
  `skills/city-wuxing-map/scripts/`.
- Python source/tests are **temporary references**, not acceptable Rust
  runtime fallbacks. Do not delete them prematurely or represent the
  package as 100% migrated.
