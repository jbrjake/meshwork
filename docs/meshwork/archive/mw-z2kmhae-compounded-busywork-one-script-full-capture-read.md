---
id: mw-z2kmhae
title: "Compounded busywork: one script, full capture, README headline number"
status: done
category: meta/readme
verify: python3 scripts/busywork-tokens.py --selftest
docs:
  - README.md#-the-numbers
created: 2026-08-13T01:13Z
---

## log
- 2026-08-13T01:13Z created
- 2026-08-13T01:14Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-13T01:31Z doing→done — verify exit 0 @ 8b141b2+9

## comments
- 2026-08-13T01:30Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] MEASURED 2026-08-12, full-capture recompute. Owner ruling: the compounded number is the headline — content is re-paid on every request after it lands, so early busywork (the onboarding read) is the expensive kind. scripts/busywork-tokens.py replaces admin-tokens.py + turned-tokens.py: one classifier, flat + compounded, capture now includes assistant text/thinking paragraphs about the tracker, tracker lines/diff-sections embedded in other calls' traffic, user messages, attachments/system at line granularity, and subagent transcripts (<session>/subagents/) compounding in their own chains. Whole-call classification tests the call TARGET (heredoc bodies stripped, Path()/open() evidence kept) — product-code edits citing TODO.md in a comment no longer count whole. Numbers (before -> after): sazed flat 32.6K (26.6%) -> 8.5K (7.0%), compounded 4.19M/39.7M (10.6%) -> 0.99M/48.3M (2.1%), 34 post sessions; leras flat 31.0K (30.1%) -> 10.0K (10.5%), compounded 2.34M/24.7M (9.5%) -> 0.50M/18.1M (2.8%), 3 ordinary post sessions. Excluded: migration sessions (sazed 8b7f479f, leras 6f063ba1), sub-100KB, leras sweeps faba7815/4e5b1f04 (54%/56% flat busywork), sazed 42ec1a8b (live, still appending at measure time). Note the honest direction: fuller capture RAISED the post-era numbers more than pre (old replay row said 4.6x/5.7x less; true compounded is 4.2x/4.7x less absolute).
