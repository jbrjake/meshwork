---
id: mw-tcb792a
title: "Fix the README's typos and its ask demo — the spill fix waits on the ask, and the ask closes on its own verify in the asker's tree"
status: done
category: meta/readme
verify: "all(contains README.md /^\\$ meshwork dep add sa-38wd6se --needs sa-z1ecwc8$/, contains README.md /^\\$ meshwork close sa-z1ecwc8$/, lacks README.md porfolio)"
docs:
  - README.md#that-generates-its-own-work
created: 2026-09-24T01:09Z
---

Owner request in session: fix typos and gross technical errors in the README without expanding it or changing its voice. Outside feedback on the ask demo: it files sa-z1ecwc8 because the spill fix needs the knob, yet `ready` still lists sa-38wd6se, so nothing ties the dependent work to the ask; and the ask's verify (`contains config/engine.toml wakeup_ms`) names a file neither tree has, so lint reports verify-path-missing and the ask can never close. The demo adds `dep add sa-38wd6se --needs sa-z1ecwc8`, and back in demo, after the answer is done, writes config/engine.toml and closes the ask, which puts the spill fix back in `ready`.

## log
- 2026-09-24T01:09Z created
- 2026-09-24T01:19Z open→done — verify exit 0 @ 46b0726+5
