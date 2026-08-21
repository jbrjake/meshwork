---
id: mw-3gpdbbh
title: "`add --batch` splits on document separators inside fenced code blocks, creating phantom tasks"
category: core/import
labels: [bug]
verify: run cargo test batch_ignores_separators_in_fenced_code
status: open
created: 2026-08-19T19:18Z
seq: 20
---
The `--batch` parser splits input on a line containing only three hyphens, without tracking fenced-code state. A task body that documents YAML frontmatter — a bug report with a repro, a doc task showing a task-file example, anything quoting meshwork's own format — gets silently split into extra phantom tasks.

Observed live while filing `[[mw-nzeezr8]]`: a bug report whose repro contained a fenced `cat > /tmp/b.md` heredoc with an example task document produced **two** tasks. The phantom one inherited the example's `title: task delta` and its placeholder `needs: "REPLACE_WITH_A"`, which then failed `lint` as a dangling edge.

The failure is quiet in the worst way: `add --batch` is documented as atomic ("all files or none"), and it *was* atomic — it just created a different set of files than the author wrote. Nothing in the output distinguishes "2 tasks, as you intended" from "2 tasks, one of which is a fragment of the other's body."

## Fix

Track fenced-code state (``` and ~~~, including longer runs and info strings) while scanning for separators, and only split at a separator that is outside a fence. Same scan is likely needed anywhere else the store parses a document boundary.

Cheap guard worth having regardless: after splitting, reject a batch where any produced document lacks a `title:`, rather than filing it with an empty or inherited one.

## Note

This is self-referential in a way that will keep biting: meshwork's own repo is where people document meshwork's file format, so its store is the most likely place in any portfolio to contain task bodies that quote task frontmatter.

## log
- 2026-08-19T19:18Z created
