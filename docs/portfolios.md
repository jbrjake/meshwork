# meshwork portfolios

If you're not a fan of giant monorepos, you probably have a few projects that connect to each other, like a reusable module or a client/server pairing. And that means you probably have tasks in one that require work in another. If you clone them all locally, meshwork can see across them.

## portfolio setup

You can set up a portfolio to look at multiple projects together (see [pathing](#portfolio-pathing) below), with a tiny git repo holding a `repos.toml`:

```toml
[[repo]]
name = "alpha"
remote = "git@github.com:example/alpha.git"

[[repo]]
name = "beta"
remote = "git@github.com:example/beta.git"

[[repo]]
name = "gamma"
remote = "git@github.com:example/gamma.git"
```

## portfolio usage

`portfolio ready` shows tasks from all repos in the .toml that you've got locally:

```
$ meshwork portfolio ready | head -4
portfolio: skipped gamma — no checkout at /Users/dev/Documents/code/gamma
beta#bz-s3q1  Schema qualifier cleanup
alpha#az-n33d  Publish spill report
alpha#az-x9b2  Cross-repo consumer bump
alpha#az-r3l8  Document spill knobs
```

## inter-dependencies

Dependencies cross repos: the beta repo shipped its reader rewrite (`bz-c0r3`, done), and the alpha repo's consumer bump depends on it:

```
$ grep needs: docs/meshwork/az-x9b2-cross-repo-consumer-bump.md
needs: [beta#bz-c0r3]
$ meshwork why az-x9b2
az-x9b2: nothing blocking — every hard dep is done/dropped
```

That works from inside individual repos, no portfolio command involved. If it can't find the other repo on disk, it'll let you know:

```
$ meshwork why az-x9b2
az-x9b2 blocked by 1:
- beta#bz-c0r3 (unresolved — absent or unregistered repo)
```

Only a done/dropped task on the other side satisfies the dependency.

## cross-prioritization

The portfolio repo can also hold a `sequence.md`, a list of `repo#id` bullets under cosmetic section headings:

```markdown
## Tranche 1 — spill cliff before anything

- alpha#az-t5k1
- beta#bz-r34d

## Tranche 2 — reporting

- alpha#az-n33d
```

`portfolio next` answers the session-start question across everything: what single task is next? The first *ready* sequenced task wins; `az-t5k1` is already claimed as `doing`, so:

```
$ meshwork portfolio next
portfolio: skipped gamma — no checkout at /Users/dev/Documents/code/gamma
beta#bz-r34d  Retry policy for fetch
```

Ready tasks missing from the sequence fall back to `repos.toml` order, then per-repo `seq`. Resequencing an entire portfolio is editing one small file in one small repo, reviewed and diffed like everything else.

## unified querying

`portfolio q` is the same SQL surface with a `repo` column.

## portfolio pathing

Per-machine checkout paths live in a gitignored `repos.local.toml` (default: `~/Documents/code/<name>`; the portfolio dir itself defaults to `~/Documents/code/portfolio`, `MESHWORK_PORTFOLIO` overrides).

## portfolio performance

Cold, `ready` over a 1K-task store, and the union across 20 repos, both answer in ~30ms.
