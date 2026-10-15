# Handoff

12 October 2026. Packet Alley, the MVP.

## What exists

| What | Where |
| --- | --- |
| The game, live | https://sgilson7.github.io/packet-alley/ |
| Repository (public, MIT) | https://github.com/sgilson7/packet-alley |
| Website listing | https://sgilson7.github.io/projects/packet-alley/ and a tile on /projects/ |
| Deploys | `.github/workflows/deploy.yml`: a push to `main` runs `./scripts/check.sh` (Chromium) and deploys `dist/` to Pages only if it passes |

What the request asked for is in: packets visualised moving on the network,
cable by cable, with a packet inspector and a plain-words log; networking
challenges a student solves by changing the network or the addressing; each
level a teaching tool as well (its lesson, a device inspector, a diagnosis of a
failed run, and a quiz question for a third star); accessible and colour-blind
friendly throughout; every picture TikZ, after `Sam_Comm_final.png`.

Nineteen levels, four chapters. Each teaching chapter now has a short intro set
and a harder advanced follow-up set of three (configure a larger network by
click-choice; ramps to one technical finale — SECOND-ORDER row 17). Frames on
the Block (link layer — addressing, hub vs switch, a switching loop; then
collision domains, a redundant backbone, a whole segment), Routes Across Town
(network layer — gateway, routing table, TTL loop; then a junction's two roads
and a four-router crossing), Handshakes (transport — ARP, the handshake,
go-back-N; then window/timeout tuning, the handshake across a router, a reliable
routed transfer), and When Too Much Arrives (availability — one watch-only
explainer of a server overwhelmed by a flood of ordinary requests). The map also
has an accessibility zoom (slider + `+`/`−` + reset; SECOND-ORDER row 18).

## The denial-of-service chapter: what is and is not here

Added at the peer's request: a **non-interactive explainer**, the `flood` level
in the "storm" chapter. Five ordinary hosts send real requests to one server,
the single switch→server cable saturates, and ~65 of ~85 frames drop. You watch
it and answer a question; there are no edits, no spoofed addresses, and no
mechanic a student drives to attack — it is the simulation's ordinary
finite-queue rule under honest traffic (SECOND-ORDER row 16).

What stays **out** is the *interactive* attack: a student-built flood with
spoofed source addresses. The design met Sam's "cannot be recreated from the
repo" requirement, but automated safety review stopped the build on it three
times, and a safety stop is answered by leaving the feature out, not by
disguising it. In `BRIEF.md`, `PLAN.md` P11, `SECOND-ORDER.md` row 2 and
`README.md`. The loop and the hub→switch change — the peer's other two
examples — are built.

## Checks run, and results

| Command | Result |
| --- | --- |
| `cargo test --workspace` | core unit tests, 11 boundary tests, a 5-test level probe: all pass |
| `./scripts/check.sh` (local, three engines) | format, clippy, tests, release WebAssembly build, import audit, browser check in Chromium, Firefox and WebKit: PASS (see below) |
| `cargo test -p pa_core --test campaign` | every level passes with its intended choices; none passes before a change; the quiz scores |
| `./scripts/check.sh` final local run | BUILDER CHECK PASSED: browser check in Chromium, Firefox and WebKit |
| `./scripts/check.sh` after the advanced-mission tier + map zoom | BUILDER CHECK PASSED in Chromium, Firefox and WebKit (19-level campaign probe green; zoom verified by screenshot at 3×) |
| `./scripts/check.sh` after v2 (edit on the map, curved cables, wider palette, rewritten prose) | BUILDER CHECK PASSED in Chromium, Firefox and WebKit; campaign green (choices unchanged); the browser check now drives the on-map pointing gesture (SECOND-ORDER rows 20–23) |
| `./scripts/check.sh` after the curriculum content rewrite (verbatim story/task/debrief for all 19 levels, chapter openings, textbook-sourced device blurbs) | BUILDER CHECK PASSED in Chromium, Firefox and WebKit; `cargo test -p pa_core` green; campaign choices unchanged; `python3 packet_alley_curriculum_packet/scripts/validate_packet.py` passes the answer-leak lint |
| CI on `main` (Chromium) | check: success, deploy: success |
| `BUILDER_ORIGIN=https://sgilson7.github.io/packet-alley tests/browser/check.py` | PASSED in Chromium against the live site |
| Website `make check`, `make packet`, `make gate`, `make deploy` | all passed; the listing is live and filed as a classroom tool |

## The curriculum packet (Sam's `packet_alley_curriculum_packet`)

An evidence-first redesign handoff. **Phase 1, the content layer, is applied and
deployed:** every level's title, story, task and debrief is the packet's verbatim
copy, the chapter intros are its openings, and the hub/switch/router inspectors
now quote the open textbook (CC BY-SA 3.0, attributed in `LICENSES.md` and
`data/textbook_extracts.json`). Data only; the deterministic engine, the pinned
campaign choices and the quizzes are untouched. See `SECOND-ORDER.md` rows 24–25.

**Phase 2 is gated and NOT built** (the packet's Stage 9 / R9 say not to
self-authorize it, and ask for professor review): the typed
Inspect→Predict→Configure→Commit→Simulate→Debrief core state machine,
prediction-gated scoring with per-case practice variants, the four replayable
demonstration scenes, the quiz→prediction change, the textbook concept panel and
new property tests. Contracts for it are in `packet_alley_curriculum_packet/`.
See `SECOND-ORDER.md` row 26.

## Waiting on the peer / Sam

1. **The name.** "Packet Alley" is a working title.
2. **The levels and their balance.** Nineteen levels, their networks, numbers and
   all text are first-pass (`data/levels.json`, `data/copy.json`). One probe
   checks each is winnable with the intended change; a classroom decides the
   rest.
3. **The denial-of-service unit.** If wanted, it needs another route than this
   repo (see above).
4. **Phones.** The page lays out at phone width and takes taps, but nobody has
   played it on a phone.

## Known gaps

- No sound and no online mode (neither was asked for; a run needs no second
  player).
- Three extra device icons and a night backdrop are in the build but unused
  (drawn before the flood chapter was dropped); harmless.
- Mutation testing is an optional audit (`scripts/mutants.sh`), not part of the
  everyday check.

## Where things are

`BRIEF.md` (the request), `PLAN.md` (each decision and the words it answers),
`SECOND-ORDER.md` (what came up), `HANDOFF.md` (this). The core:
`crates/core/src/` — `ip.rs` (addresses), `level.rs` (the level format and
edits), `sim.rs` (the whole simulation), `game.rs` (the one door the page
calls), `api.rs` (the JSON wrapper). The page is `web/`; the art pipeline is
`tools/`.
