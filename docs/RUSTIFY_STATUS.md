# Native Rust migration — source mapping and gates

## Ownership and scope

Source snapshot: fork main commit `a996a21745d0adc7fa343f208dc02920c2a7da8f` (2026-11-19).
This is a **staged migration merged into main by explicit owner request (2026-11-19)**, not a claim of full parity. The original Agent
Skills packaging, Python implementation, reference documents and regression
fixtures remain a stable oracle until each subsystem is validated against it.
No HTML/JavaScript/Python is executed by the new Rust binary, which does not
launch Python subprocesses.

| Original source | Native Rust module | Status | Oracle |
| --- | --- | --- | --- |
| `scripts/kanyu/core.py` | `src/core.rs` | Ported | Compass boundaries, 24 mountains, Lo Shu, periods |
| `scripts/kanyu/bazhai.py` | `src/bazhai.rs` | Ported | Ming Gua, eight mansion symmetry, 3 Yuan |
| `scripts/kanyu/feixing.py` | `src/feixing.rs` | Ported | 8/9 periods, 24 mountain patterns, replacement stars |
| `scripts/kanyu/zibai.py` | `src/zibai.rs` | Ported | Annual/monthly center, year Sha, 3 Yuan |
| `scripts/kanyu/almanac.py` | `src/almanac.rs` | Partial: day taboos and solar offset only | Lunar/calendar/solar-term exactness still pending |
| `scripts/kanyu/ganzhi.py` | `src/ganzhi.rs` | Partial: stems, ten gods, phases, pairs, seasons, empty branches | Full natal relations and shensha pending |
| `scripts/kanyu/bazi.py` | — | Pending | Solar time, DST, zones, pillars, luck cycles |
| `scripts/kanyu/zeri.py` | `src/zeri.rs` | Partial: scoring and exclusion rules | Date-range selection and actual almanac facts pending |
| `scripts/kanyu/render.py` | CLI text adapter | Partial | No legacy narrative/snapshot parity claim |
| `scripts/kanyu/pages.py`, `assets/` | — | Not ported | Interactive HTML excluded from Rust runtime |
| `skills/city-wuxing-map/scripts/` | — | Pending | Full coverage, geometry, provenance |
| `skills/bazi-mingli/scripts/kanyu/` | — | Legacy copy | No second independent Rust implementation |

Map examples and geodata carry source/licensing obligations and confidence
limitations. Any native map core must preserve these, not invent physical
five-element measurements.

## Public API and adapters

One crate, `kanyu_core`: typed deterministic functions; no I/O, FFI,
Python or external dependencies. The CLI `kanyu` in `src/bin/kanyu.rs`
calls that same library. No window, browser, server or daemon is initialized.
Once permissions are cleared the core can be published separately from its
CLI without moving application logic.

**Year contract:** `mingua`, `yun_of_year`, `annual_center` and
`monthly_center` accept years **already adjusted at Li Chun**. We do NOT
silently convert Gregorian birthdays with a guessed February 4 boundary.
Month indices are solar-term months (寅=1), not Gregorian months.

**CLI contract:** `--help`, `--version`, `doctor`, `mingua`,
`bazhai`, `feixing`, `liunian`, `ten-god`, `day-flags`, `zeri-score`, `--json`. JSON schema version 1.
Stdout contains only results, stderr diagnostics, invalid arguments exit 2.
Operations are pure/read-only so no dry-run switch is needed. A future optional
MCP stdio adapter must delegate to the library API, not fork its algorithms.

## Verification gates

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release --bin kanyu

cargo run --locked --bin kanyu -- doctor --json
cargo run --locked --bin kanyu -- feixing --sit 子 --yun 8 --json
cargo run --locked --bin kanyu -- mingua --year 1990 --gender f --json
cargo run --locked --bin kanyu -- liunian --year 2026 --json
```

Parity unit tests follow `tests/test_core.py`, `tests/test_bazhai.py`,
`tests/test_feixing.py`, `tests/test_sanyuan.py` and
`tests/test_zibai.py`; they do not prove missing calendar/Ba Zi/map features.
Compare paired Python/Rust fixtures for every subsequent module before
replacing the legacy entrypoint.

## Publication and ownership

The fork's source tree has no LICENSE file. Accordingly the Rust package is
`publish = false` until the upstream rights and required attribution are
verified. Do not assume an SPDX license for code whose permission is unknown.

## Native delivery

Only source/manifests belong in Git. Keep `target/` and binaries out.
Self-hosted Linux Actions with 12 Cargo jobs and persistent target cache is
the intended build lane. Windows compilation/runtime remains an explicit
future gate, not an assumption. Future feature-branch promotion normally requires a green
exact-SHA rustfmt, strict Clippy, tests and linked CLI JSON smoke test.
For this initial Rustification import, the owner explicitly ordered main
integration before validation. **Build/test/parity are NOT verified**.
See [HANDOFF.md](../HANDOFF.md) for the outstanding implementation backlog.
