---
id: mw-ryshdw8
title: "Correct the README's raw table count — it says six, and q --help lists seven now that covers exists"
category: meta/readme
docs: [README.md#but-you-can-query-it-like-a-database]
verify: lacks README.md /six raw tables/
status: open
created: 2026-09-24T01:13Z
---
README says "On top of the six raw tables sits a derived layer of thirteen views". `q --help` lists seven tables: tasks, edges, labels, comments, log, repos, covers. The views are still computed from the first six (FORMAT.md §Views), so the sentence only needs the count.

README is the owner's voice: make the edit, leave it uncommitted for the owner's review.

## log
- 2026-09-24T01:13Z created
