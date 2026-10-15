# Curriculum handoff — the evidence-first redesign (Phase 2)

15 October 2026. This is the handoff the curriculum packet (Stage 8) asked for:
what was built, what was checked, and what still needs a human's judgement.

## Baseline and new build

- Baseline HEAD (branch point): `bb401aa`.
- New HEAD: `e42c9c1`, branch `evidence-first`.
- Changed (outside the packet): 15 files, +1538 / −85 — the core, the shim, the
  page, the data, the tests. The engine's determinism and the 57-star campaign
  are preserved.

## What shipped, by stage

1. **Core evidence-first model.** A level may carry `evidence` (ids to inspect),
   a `predict` block (a prompt and options, each a verifiable `Claim`), and
   fixed practice `variants`. `graded()` checks a claim against the recorded run
   — never a stored option index (PA8). A run will not start until the evidence
   is inspected and a prediction committed (PA5); the final star needs a pass
   **and** a correct prediction; a practice variant changes the numbers and
   awards nothing (PA6).
2. **Page flow.** The devices to inspect glow; opening one records evidence; a
   prediction panel commits an outcome; "Commit and run" is disabled until both
   are in; the result shows whether the prediction matched and offers "Practice
   a different case." Keyboard and side-panel paths kept; no page numeric literal
   but 0/1; no inline style (PA13).
3. **Four demonstration scenes** (`demo_block`, `demo_town`, `demo_wires`,
   `demo_storm`), one per chapter, on TEST-NET addresses (192.0.2/198.51.100/
   203.0.113), watch-first, Teacher mode skips (PA2, PA3). Each requires evidence
   and a prediction before it is left; none awards a star.
4. **All 19 levels migrated** to evidence + prediction. flood's watch-only quiz
   became a prediction. Practice variants on the eight single-block levels
   (plus address) give a fresh case on a miss across the link and transport
   chapters.
5. **Textbook concept panel + Go-Back-N badge.** Each chapter's brief shows a
   verbatim CC BY-SA 3.0 passage with attribution; the three go-back-N levels
   carry a "simplified teaching model, not full TCP" badge (PA11).

## The 19-level checklist

All 19 pass with their intended choices **and** their prediction is graded right
(`cargo test -p pa_core --test campaign -- --nocapture`): address, hub, loop,
crowd, backbone, floor, gateway, route, ttl, point, roads, across, arp,
handshake, transfer, window, connect, reliable, flood.

## Objectives covered

Destination IP vs MAC and switch learning; collision domains and loop-free
switching; local vs remote and the gateway; forwarding tables and next hops;
the hop limit and routing loops; ARP's local scope; the three-way handshake;
go-back-N loss recovery (labelled simplified); congestion from honest demand.

## Textbook sources and licence

`data/textbook_extracts.json` holds 13 verified passages (source URL, exact
text, SHA-256, extraction date) from *Computer Networking: Principles, Protocols
and Practice* (Bonaventure, `cnp3/ebook`), **CC BY-SA 3.0 Unported** (verified in
the repo README). Attribution is in `LICENSES.md` and inline on every quote.
Two manifest anchors (`CONGESTION_BUFFER`, `CONGESTION_CAPACITY`) drifted in the
3rd edition and are unused; re-source them before wiring more storm copy.

## Testing results

- `cargo test -p pa_core`: 33 tests green (9 unit, 11 boundary, 5 campaign, 8
  evidence-first). Property/negative coverage: the same option is right or wrong
  by the run (no answer index); gating; the final-star rule; practice awards
  nothing; each scene watches and grades; a shipped variant is winnable.
- `./scripts/check.sh`: PASSED in Chromium, Firefox and WebKit — the browser
  test watches the block scene, plays the level through inspect → predict →
  commit, and checks the textbook excerpt and its CC BY-SA attribution render.
- `python3 packet_alley_curriculum_packet/scripts/validate_packet.py`: packet
  structure OK; the answer-leak lint passes.

## Unresolved educational questions (for Ally)

Per the packet's Stage 9, the playable build still wants a conceptual review:
the MAC/IP framing in `address`; ARP placement (link-layer resolution, in the
transport chapter by level id); how far the go-back-N simplification should be
pushed; and the congestion-vs-DoS wording in `flood`. The interactive DoS /
spoofing mechanic stays out, as throughout.

## Not done (fast-follow)

Practice variants on the routed/address-embedding levels (gateway, route, ttl,
point, roads, arp, across, connect, reliable) — the mechanism supports them
(per-variant edit-option overrides) but the data is not written yet; a miss on
those re-offers the same case. The remaining textbook passages beyond the 13
can be wired into per-level concept notes.
