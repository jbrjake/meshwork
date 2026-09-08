# The cost baseline

Tokens per session, per task by category, and main chain against subagents, mined from the
session transcripts of the nine registered repos and the code root. The setup-cost matrix's
sibling. Data through **2026-09-08 13:55Z**; the corpus is live while the
miner runs, so a re-run drifts by units. Read-only miner: `python3 scripts/mine_cost.py --write`
regenerates this file; `--check` is its self-test.

## Headlines (each number's denominator in its section below)

- A session processes a median **461k fresh tokens** (p90 882k) over a median 137 API messages, at a median cache hit rate of **99%** (n=473 sessions).
- **20% of all fresh tokens are spent on subagents**, in 46 of 473 sessions; a session that fans out costs a median 894k against 423k for one that does not.
- Per task, by category (n ≥ 7 tasks): medians span **24.2×**, from `skill` to `kwaan` — the prioritization ladder's rung-0 question (*does cost vary ≥ 3× across categories?*) is answered **yes** on 8 categories.
- Sessions that loaded the skill (≥ 10 meshwork calls) spend a median **21.0k fresh tokens per meshwork call** against 28.3k without it (n=94/108); hand-edits per session do not move (median 1 both ways).
- Hand-edits of task files cost **650 API turns** in 210 sessions — 615k output tokens (1% of all output) on turns carrying 150M tokens of context, nearly all of it cache reads.
- Meshwork calls whose result took more than 5 minutes: **72**, 16.4 h of wall time in 52 sessions (longest 172 min). 20 of them were `start`/`close`/`verify` (5.7 h — a build, or an approval hold); the other 52 (10.7 h) were verbs that run nothing ({'comment': 13, 'lint': 10, 'set': 7, 'show': 5, 'q': 3, 'portfolio': 3}) and can only be the owner holding an approval: §1.6's wait, at the permission prompt.

## Method

- **Session** = one transcript file. **Message** = one assistant API response, identified by `message.id`; a response streams as several records that repeat the same `usage`, so it is counted once (73000 duplicate records skipped). Synthetic API-error records carry zero usage and are skipped.
- **Fresh tokens** = `input_tokens + cache_creation_input_tokens + output_tokens`: what the API processed anew. **Cache hit rate** = `cache_read ÷ (cache_read + input + cache_creation)`, main chain. Cache reads are never summed into fresh.
- **Main chain vs subagents** = the session's transcript against the `subagents/agent-*.jsonl` files beside it (every record there is `isSidechain`). A subagent message is attributed to the task the main chain was on at the message's stamp.
- **Task attribution** = tokens from a main-chain `start <id>` until that id's `close` or the next `start`; before the first start or after a close they are *between tasks*. Categories are the store's, two levels, per-repo `q` over every store including archives.
- **Hand-edit turn** = a message carrying an Edit/Write on `docs/meshwork/*.md` or a heredoc/sed/perl into one. **Long call** = a meshwork Bash call whose `tool_result` landed more than five minutes after the call.
- **After the first emphatic meshwork prompt** = usage from the first prompt after turn 1 that is emphatic (field study §1.1's definition) and names meshwork or a task id, to the end of the session.

## Dataset (the denominators)

473 of 485 transcripts carry usage, 2026-07-31 → 2026-09-08, 71,014 API messages.

| repo | sessions | med fresh / session | p90 | total fresh | med output | med messages | med cache hit | subagent share | sessions with subagents |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| sazed | 177 | 475k | 829k | 90.2M | 131k | 175 | 99% | 2% | 4 |
| portfolio | 32 | 390k | 3759k | 39.2M | 103k | 70 | 97% | 65% | 10 |
| leras | 58 | 545k | 996k | 34.0M | 125k | 100 | 98% | 7% | 3 |
| code | 43 | 457k | 1549k | 27.2M | 85k | 90 | 96% | 44% | 16 |
| meshwork | 51 | 373k | 749k | 26.3M | 108k | 100 | 98% | 28% | 8 |
| tensoon | 40 | 492k | 873k | 19.8M | 148k | 107 | 98% | 1% | 1 |
| marasi-applied-r-and-d | 30 | 581k | 882k | 16.1M | 159k | 212 | 99% | 0% | 1 |
| marasi | 26 | 522k | 870k | 15.4M | 141k | 82 | 98% | 26% | 3 |
| wyndam | 9 | 207k | 682k | 2.1M | 71k | 72 | 98% | 0% | 0 |
| oreseur | 7 | 120k | 302k | 1.1M | 36k | 44 | 96% | 0% | 0 |
| **all** | 473 | 461k | 882k | 271.4M | 121k | 137 | 99% | 20% | 46 |

## Cache

Per-session main-chain hit rate (n=473): p10 95%, median 99%, p90 99%; corpus-wide 99%. Caching is near-universal, so the fresh-token figures above are the cost that varies. The 9 sessions below 50% are short: code/c1eda219 (0%, 1 msgs), leras/59503e08 (0%, 1 msgs), sazed/6492e292 (36%, 1 msgs), tensoon/391752f7 (37%, 1 msgs), sazed/b493c42a (40%, 1 msgs), tensoon/7b6341c3 (42%, 1 msgs), code/74161c35 (46%, 2 msgs), portfolio/fdf3e6a0 (46%, 2 msgs), oreseur/cc84eee4 (47%, 1 msgs).

## Main chain vs subagents

| | value |
|---|---|
| sessions that spawned subagents | 46 of 473 |
| fresh tokens on subagents, corpus | 53.1M of 271.4M (20%) |
| subagent share inside fan-out sessions | 69% |
| median fresh, fan-out vs single-chain sessions | 894k vs 423k |
| subagents per fan-out session | median 4, p90 10 |

The field study's own method — 39 sessions read by analyst subagents from the portfolio and code-root directories — is inside these numbers, which is why those two repos carry the largest subagent shares.

## Per task, by category

266 tasks received attributed tokens (39.2M fresh; 232.3M more fell between tasks); 1 attributed ids resolve to no store. Per task: median 58k, p90 370k. Categories with n ≥ 7:

| category | tasks | med fresh / task | p90 | total |
|---|---:|---:|---:|---:|
| (none) | 87 | 118k | 605k | 17.2M |
| core/format | 16 | 18k | 58k | 0.5M |
| core/lifecycle | 12 | 34k | 78k | 0.6M |
| skill | 12 | 9k | 41k | 0.2M |
| core/verify | 11 | 30k | 121k | 0.6M |
| kwaan | 9 | 227k | 888k | 2.4M |
| rig/exec | 7 | 165k | 455k | 1.2M |
| meta/distribution | 7 | 17k | 62k | 0.2M |

Spread of medians across those 8 categories: **24.2×** (`skill` → `kwaan`). Rung 0's threshold is 3×.

## Skill loaded vs not

| sessions | n | med fresh / session | med fresh / meshwork call | med meshwork calls | med hand-edits | med --help |
|---|---:|---:|---:|---:|---:|---:|
| any meshwork call, skill loaded | 114 | 479k | 23.0k | 18 | 1 | 1 |
| any meshwork call, not loaded | 174 | 461k | 35.4k | 12 | 1 | 1 |
| ≥ 10 calls, skill loaded | 94 | 490k | 21.0k | 21 | 1 | 1 |
| ≥ 10 calls, not loaded | 108 | 494k | 28.3k | 17 | 1 | 2 |

## The findings, priced

| finding (field study) | price |
|---|---|
| 723 hand-edits of task files (§2.3) | 650 API turns in 210 sessions; 615k output tokens (1% of all output), 150M tokens of context re-sent |
| the unscoped verify — an 18-minute compile against a 5-minute timeout (§2.5.2–3) | 20 `start`/`close`/`verify` calls past 5 min, 5.7 h of wall time, by repo {'sazed': 6, 'leras': 5, 'meshwork': 4, 'marasi': 2, 'marasi-applied-r-and-d': 2, 'tensoon': 1}; plus 52 calls on verbs that run nothing (10.7 h) — approval holds |
| the sixth-time eruptions (§2.1, §2.3) | tensoon/f1040972 772k fresh, 468k of it after the first emphatic prompt; marasi-applied-r-and-d/58b837fb 740k fresh, 72k of it after the first emphatic prompt; sazed/72773264 602k fresh, 31k of it after the first emphatic prompt; sazed/51e60dbd 458k fresh, 313k of it after the first emphatic prompt; sazed/1b167145 348k fresh, 50k of it after the first emphatic prompt; sazed/94b76cfa 273k fresh, 177k of it after the first emphatic prompt |
| every session with an emphatic meshwork prompt | 29 sessions spent 7.2M fresh tokens after their first one — a median 44% of the session |
| 61% of sessions never load the skill (§1.2) | per meshwork call, 28.3k fresh without the skill vs 21.0k with it (≥ 10-call sessions, n=108/94) |
| fan-out (ask A8) | 20% of the corpus's fresh tokens; 46 sessions |
| cache hit rate (ask A4) | median 99% per session, p10 95%; 9 sessions under half — not where the cost varies |

## Caveats

- Transcripts exist only on this machine and are prunable; the meshwork repo's sessions dogfood the CLI against fixture stores, so its attributed ids include throwaway tasks.
- Half of all closed work was never `start`ed (field study §1.2), so per-task attribution covers the claimed half; unclaimed work is in *between tasks*. A session that starts a task and never closes it attributes everything after the start to it.
- Token counts are the API's; no price list is applied. Cache reads are billed but are not fresh work, which is why they are reported apart.
- Verb and edit detection is regex over commands, as in the field study; a heredoc that mentions a verb in prose can miscount. Hand-edits and meshwork calls made by subagents are not counted (the field study's scripts read the session transcript only); their tokens are.
- A long call's duration includes any approval hold in front of it; the split by verb is the only separation the transcript offers.
