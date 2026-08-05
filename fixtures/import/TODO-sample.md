# TODO — sazed (committed sample of the baseline checkbox format, MW-J3)

## Now

- [~] **Fix spill cliff at 600M keys** — p99 collapses past 600M; repro in bench/spill_cliff.rs.
      verify: `cargo test -p sazed-spill -- --exact spill::cliff_600m` exits 0
- [ ] **Extract Arrow conversion seam** — spill tests need fake batches; blocks the cliff fix refactor.
      verify: `cargo test -p sazed-exec seam::` exits 0
- [!] **Bump datafusion to 52** — blocked: 52 unreleased; unblock = release lands, then rerun parquet suite.
      verify: `cargo test -p sazed-exec parquet::` exits 0

## Later

- [ ] **Document the governor knobs** — one page: guarantees vs never-promises,
      including the wakeup interval defaults.
- [x] **Migrate config to toml** — landed 2026-07-29.
      verify: `cargo test -p sazed-config toml::` exits 0
