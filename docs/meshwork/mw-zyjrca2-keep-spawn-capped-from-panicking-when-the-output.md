---
id: mw-zyjrca2
title: Keep spawn_capped from panicking when the output cap splits a multibyte character in stderr
category: core/verify
docs: [docs/DESIGN-meshwork.md#12b-trust-boundary-verify-is-untrusted-input-ruled-via-mw-mjwfvxn-2026-08-07]
verify: run cargo test spawn_capped_cap_on_multibyte_boundary
status: open
created: 2026-10-02T13:32Z
---
`verify_exec::spawn_capped` appends `&err_tail[..err_tail.len().min(spare)]` — a byte-index slice of a lossy-decoded String. When stdout leaves `spare` bytes and that index lands inside a multibyte character of stderr, the slice panics. Found by reading the code; not yet reproduced — the test should drive stdout to just under the cap with a multibyte stderr tail.

## log
- 2026-10-02T13:32Z created
