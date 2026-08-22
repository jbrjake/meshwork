---
id: mw-3tzfqmq
title: add/set accept control characters that mint an unparseable file
status: open
category: core/authoring
discovered-from: mw-8fmsws3
verify: run cargo test mint_rejects_controls
created: 2026-08-22T01:20Z
---

## log
- 2026-08-22T01:20Z created

## comments
- 2026-08-22T01:20Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Discovered red-testing mw-8fmsws3: 'add' with a raw ESC in the title writes the string through yaml_scalar unescaped, YAML forbids raw controls, and the freshly minted file parses INVALID — the verb reports success while the store gains an invalid row. Same path for set --title/--handoff and block --reason. Fix either by escaping controls in yaml_scalar (double-quoted forms) or refusing the input loudly at the verb; red test mints with ESC and expects a valid file or a refusal, never a silent invalid.
