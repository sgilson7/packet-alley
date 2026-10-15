# Claude Code: execute the curriculum packet verbatim

**Repository:** https://github.com/sgilson7/packet-alley, work on a feature branch `curriculum-evidence-first`. This packet is not a command to push or deploy. Do not rewrite authored prose. Only implementation wiring, tests and code comments may be written by the agent.

## Stage 1 — Baseline

Clone checkout. Read `BRIEF.md`, `BUILDER-SPEC.md`, `PLAN.md`, `AGENTS.md`, `CLAUDE.md`, `HANDOFF.md`, `SECOND-ORDER.md`; inventory every student-visible copy location in `data/`, `web/`, `crates/`. Save full initial `git rev-parse HEAD` and `./scripts/check.sh` results. Confirm `data/levels.json` really contains the 19 IDs in `data/level_blueprints.json`. Stop and log drift rather than silently map a different level.

## Stage 2 — Provenance and content

Read `09_COPYRIGHT_PROVENANCE.md`. Run `python3 scripts/extract_textbook.py` from this packet on the developer machine (requires access to publicly available textbook pages), generating `data/textbook_extracts.json`. Review every extracted sentence against the book page. Verify each passage's per-file copyright notice and attach proper attribution/ShareAlike notices. Never fetch textbook pages from the student's browser. If a URL/selector changed, add a blocker to `SECOND-ORDER.md`; don't replace the passage with your own technical explanation.

## Stage 3 — Exact narrative copy

Load `data/verbatim_story_copy.json`. Map each known field by `level.id` into the renderer's data path. If the current schema needs new `story_copy` keys, extend it additively; preserve old keys until validated. Use the supplied `opening`, `story`, `task`, `debrief` and prompts **byte for byte**. Apply `data/verbatim_interface_and_technical_routes.json` verbatim. Migrate inaccurate mechanical statements in `concept`, `quiz`, `why` and `copy.json` by replacing them with its explicitly mapped textbook passage IDs, not by inventing explanatory prose. This includes every existing `about_*` and `why_*` key. Title and devices without prescribed replacement remain unchanged unless their label leaks a repair.

## Stage 4 — Four interactive lesson scenes

Build the four demo scene descriptions in `05_ZONE_LESSONS.md`, reusing the current map object selection, hover/keyboard focus, packet render, inspector and animation. Use demonstration-only TEST-NET addresses in `data/level_blueprints.json`; never use a scored level's solution. Overlay original game instructions and render textbook excerpt components with source attribution. Require evidence click and prediction before exiting. No full-screen lecture-only slideshow.

## Stage 5 — Evidence-first level model in Rust

For 19 existing IDs, add explicit typed state stages `Inspect -> Predict -> Configure -> Commit -> Simulate -> Debrief`; record chosen devices/fields, predicted hop/failure and an evidence referent. Prevent outcome/individual-correctness disclosure precommit. Grade a holistic decision in core on the committed deterministic variant; output observed network events and pedagogical progress, not answer-keys in the UI. After an incorrect graded attempt, offer a new *practice* variant (new addresses or equivalent topology) for learning, not repeated grade award on identical two-choice config. Keep deterministic scores and tests.

## Stage 6 — Implement the 19 blueprints

Follow `06_LEVEL_REDESIGN.md` and `data/level_blueprints.json` one level at a time, in the listed order. For every new interaction: provide a map-native manipulation AND accessible keyboard/side-panel equivalent. Do not remove existing animation/zoom/symbol and label paths. Where one graph admits multiple correct spanning trees, grade connectivity and acyclicity, not the chosen physical cable. Where the game model is simplified, label it accurately and do not claim full real-world protocol behavior.

## Stage 7 — Evidence checks

Add Rust property/negative tests for equivalent valid loop fixes, wrong route/prefix, TTL decrement, both directions of ping, ARP neighbor scope, SYN/SYN-ACK/ACK sequence number and port scope, simplified GBN behavior, congestion with legitimate traffic, no attack builder, no hardcoded right-answer index, evidence requirement, deterministic variant and scoring. Browser tests in Chromium/Firefox/WebKit exercise zone intro, device clicks, dragging/selection, keyboard controls, commits, retry and attribution. Check that each scored lesson requires meaningful direct scene use.

## Stage 8 — Audit

Run `./scripts/check.sh`, inspect all results and screenshots, inspect generated `textbook_extracts.json` references, run `python3 scripts/validate_packet.py` from the packet, and show any known divergence. Generate a `HANDOFF_CURRICULUM.md` with: baseline HEAD, new HEAD, changed files, 19-level checklist, objectives covered, exact-copy diff, textbook IDs and licenses, testing results, unresolved educational questions, preview URLs only if actually created.

## Stage 9 — Stop at handoff

Do not deploy on self-judgement. Obtain explicit professor and project-owner review of the playable build's conceptual accuracy, particularly: MAC/IP distinction, ARP placement, scope of GBN simplification, guess-resistance and congestion/DoS wording. Do not use human-user learning gains as a claim without classroom evaluation.

### Mandatory stop triggers

- Textbook selector fails, book license cannot be verified, or text source requires login.
- The Rust core cannot model a proposed scored interaction without violating Builder requirements.
- The student-facing copy does not map 1:1 to provided content.
- `./scripts/check.sh` fails or existing stars/saves cannot be migrated safely.
- Any requested change introduces attack construction or off-origin traffic.

Report blockers truthfully; implement the unaffected work without improvising missing educational content.
