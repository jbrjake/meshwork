# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Through PLAN 1.6 (2026-08-04). The full DESIGN §6 surface exists in clap, in §6 order: 21 top-level verbs; `dep {add,rm}`, `mirror {push,status}`, `portfolio {ready,next,q,seq}`, `import {todo}` sub-surfaces pinned too. Unbuilt verbs never pretend: `mirror`→"lands at M3", `portfolio`→"M2", `import todo`→"PLAN 1.7", `show --docs`→"M4 (4.1)" — all exit 1 with the pointer. `e2e::cli_surface_frozen` parses --help and compares the verb lists verbatim: the non-goals fence is now machine-enforced (a new verb fails CI until an owner ruling amends REQUIREMENTS §3). TRACE: D4 done (17 planned).

**Decisions:** none new (surface was frozen by DESIGN; this makes it executable).

**Open threads:** replace the stubs::import error with the real implementation next item.

**Next concrete step:** PLAN 1.7 — `import todo`: baseline checkbox format (`[ ]`/`[~]`/`[x]`/`[!]`, `verify:` lines, `## Now` ordering → seq) from a committed sazed-format sample → golden task set (J3). Use MESHWORK_TODAY+MESHWORK_ID_SEED for byte-stable goldens.
verify: `cargo test e2e::import_todo_golden` exits 0.
