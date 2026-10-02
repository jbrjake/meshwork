---
id: mw-xvgs08n
title: "Make scripts/demo.sh play the quick-start loop it claims — DSL verifies, no --approve, no trivial verify, end on the archived file"
category: meta/demo
seq: 220
verify: "all(lacks scripts/demo.sh /--approve/, lacks scripts/demo.sh /--verify \"true\"/, lacks scripts/demo.sh /test -f/, contains scripts/demo.sh /docs\\/meshwork\\/archive/)"
docs:
  - docs/PLAN-demo-notes.md#meshwork-integration
  - README.md#quick-start
status: done
created: 2026-10-02T15:21Z
---
README.md says `./scripts/demo.sh` plays the quick-start loop, but the script differs from it in three ways:
- It uses a shell verify (`test -f repro.log`); make it `exists repro.log`.
- It passes `close --approve`, which does nothing because `add` approves the verify it was typed with.
- Its second task verifies `true`, which trips lint's verify-trivial and start's already-green warning. Give it a verify that fails until that task's work is done.

It should also end by printing the archived task file, as the quick-start does.

The gate never runs demo.sh. Run it and read the whole output before closing. README.md stays as it is.

## log
- 2026-10-02T15:21Z created
- 2026-10-02T16:18Z open→doing — claimed by claude (6fa0dcc3-62c5-4a7b-a6b1-de25f58287de)
- 2026-10-02T16:20Z doing→done — verify exit 0 @ b774f5c+4
