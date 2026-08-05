=== wo-2ga2-bump-datafusion-to-52.md
---
id: wo-2ga2
title: Bump datafusion to 52
status: blocked
verify: "cargo test -p sazed-exec parquet::"
seq: 30
created: 2026-08-04
blocked-reason: "blocked: 52 unreleased; unblock = release lands, then rerun parquet suite."
---
blocked: 52 unreleased; unblock = release lands, then rerun parquet suite.

## log
- 2026-08-04 imported from TODO.md

=== wo-beaw-document-the-governor-knobs.md
---
id: wo-beaw
title: Document the governor knobs
status: open
created: 2026-08-04
---
one page: guarantees vs never-promises,
including the wakeup interval defaults.

## log
- 2026-08-04 imported from TODO.md

=== wo-qe34-fix-spill-cliff-at-600m-keys.md
---
id: wo-qe34
title: Fix spill cliff at 600M keys
status: doing
verify: cargo test -p sazed-spill -- --exact spill::cliff_600m
seq: 10
created: 2026-08-04
---
p99 collapses past 600M; repro in bench/spill_cliff.rs.

## log
- 2026-08-04 imported from TODO.md

=== wo-te8w-migrate-config-to-toml.md
---
id: wo-te8w
title: Migrate config to toml
status: done
verify: "cargo test -p sazed-config toml::"
created: 2026-08-04
---
landed 2026-07-29.

## log
- 2026-08-04 imported from TODO.md

=== wo-wgsr-extract-arrow-conversion-seam.md
---
id: wo-wgsr
title: Extract Arrow conversion seam
status: open
verify: "cargo test -p sazed-exec seam::"
seq: 20
created: 2026-08-04
---
spill tests need fake batches; blocks the cliff fix refactor.

## log
- 2026-08-04 imported from TODO.md

