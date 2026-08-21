---
id: mw-nzeezr8
title: "`dep add` corrupts block-style `needs:` frontmatter, reports success, and `lint --fix` cannot repair it"
category: core/format
labels: [bug]
verify: run cargo test dep_add_block_style_needs
status: done
created: 2026-08-19T19:15Z
seq: 10
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
```

## log
- 2026-08-19T19:15Z created
- 2026-08-21T19:00Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-21T19:08Z doing→done — verify exit 0 @ c3c042f+3

## comments
- 2026-08-21T19:08Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Fixed at the source: dep add/rm now edit through the block-aware set_list, and remove_scalar drops the indented block under a removed key. Repairing stores already damaged by older pinned binaries is filed separately as mw-csdzc20.
