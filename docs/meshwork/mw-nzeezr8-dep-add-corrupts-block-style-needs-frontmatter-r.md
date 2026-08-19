---
id: mw-nzeezr8
title: "`dep add` corrupts block-style `needs:` frontmatter, reports success, and `lint --fix` cannot repair it"
category: core/format
labels: [bug]
verify: cargo test dep_add_block_style_needs
status: open
created: 2026-08-19T19:15Z
---
`meshwork dep add` rewrites `needs:` in **flow style** without removing the existing **block-style** entries, producing invalid YAML. It then prints a success line. `lint --fix` does not repair it.

Block style is what `add --batch` preserves, so any store seeded by a batch import is exposed. (`add --needs` writes flow style, which is why this doesn't show up in the common path.)

## Repro

```sh
mkdir /tmp/r && cd /tmp/r && git init -q . && meshwork init
A=$(meshwork add "task alpha" | head -1)
B=$(meshwork add "task beta"  | head -1)

# a task whose needs: is block style, as --batch writes it
cat > /tmp/b.md <<'EOF'

## log
- 2026-08-19T19:15Z created
