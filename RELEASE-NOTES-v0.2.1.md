# meshwork v0.2.1

A small release with one theme. The portfolio layer that landed in v0.2.0 turns two things into real, hand-maintained, cross-repo state: `sequence.md` — the ordered overlay of `repo#id` bullets — and `needs:` edges that cross repo boundaries. Hand-maintained state rots and drifts in ways the store's own lifecycle can't observe, so before more repos join the queue, the guards exist first.

## the overlay can't rot silently

`sequence.md` is authored, denormalized state: bullets that must stay coherent with many repos' worth of lifecycle it cannot see. Two guards close the loop:

- **Dangling entries are a lint finding.** A typo'd or deleted id used to be skipped silently — "first ready one wins" just walked past it. Registry-aware `lint` now warns `dangling-sequence` when an entry resolves nowhere it could: an unregistered repo name, or a registered, locally-present store holding no such id. A repo that's simply absent from this machine is *not* dangling — it stays the skipped-repo notice, because absence is not evidence of anything. A warning, not an error: a dangling entry degrades ordering, never readiness.
- **Satisfied entries prune themselves.** Entries whose task is done or dropped in a present repo used to accumulate forever — six months in, the overlay would be mostly dead ids, the clutter problem `archive/` already solved for task files. Now running any `portfolio` verb prunes them, no flag: bullet lines resolving to a terminal task are removed before the query runs; headings, prose, live entries, dangling entries, and entries in absent repos survive byte-for-byte. Every removal is reported — stderr in text mode, a `pruned` list beside `skipped` in `--json` — and since the file is versioned in the portfolio repo, `git diff` is the review surface and the undo.

## drop tells the other side

Only done or dropped satisfies a dependency, and they mean opposite things: done says the work happened, drop says it never will. Inside one repo that distinction is survivable — same person, same head. Across repos, whoever drops `beta#bz-c0r3` silently unblocks every task elsewhere that needed it, with no visibility on either side. `drop` now scans registered, present repos for live inbound cross-repo `needs` on the dropped id and prints each as a warning: this task, over there, was waiting on the thing that just became never-going-to-happen. Repos it can't scan — absent checkout, broken store — are named as unscanned, because silence would read as "all clear." The warnings are advisory; the drop always proceeds.

## getting it

darwin arm64, linux arm64/x86_64, windows x86_64. Pin: put `v0.2.1` in `.meshwork-version`; install to `~/.meshwork/versions/v0.2.1/` (see the meshwork adoption skill). Consuming repos upgrade by editing one file; nothing global to touch.
