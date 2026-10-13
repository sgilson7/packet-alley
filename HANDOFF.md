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

Nine levels, three chapters: Frames on the Block (link layer — addressing,
hub vs switch, a switching loop), Routes Across Town (network layer — gateway,
routing table, TTL loop), Handshakes (transport — ARP, the three-way handshake,
go-back-N on a lossy link).

## What is deliberately not here

The denial-of-service / flooding chapter the peer asked for. The design met
Sam's "cannot be recreated from the repo" requirement, but automated safety
review stopped the build on it three times, and a safety stop is answered by
leaving the feature out, not by disguising it. In `BRIEF.md`, `PLAN.md` P11,
`SECOND-ORDER.md` row 2 and `README.md`. The loop and the hub→switch change —
the peer's other two examples — are built.

## Checks run, and results

| Command | Result |
| --- | --- |
| `cargo test --workspace` | 24 unit tests, 11 boundary tests, 4-test level probe: all pass |
| `./scripts/check.sh` (local, three engines) | format, clippy, tests, release WebAssembly build, import audit, browser check in Chromium, Firefox and WebKit: PASS (see below) |
| `cargo test -p pa_core --test campaign` | every level passes with its intended choices; none passes before a change; the quiz scores |
| `./scripts/check.sh` final local run | BUILDER CHECK PASSED: browser check in Chromium, Firefox and WebKit |
| CI on `main` (Chromium) | check: success, deploy: success |
| `BUILDER_ORIGIN=https://sgilson7.github.io/packet-alley tests/browser/check.py` | PASSED in Chromium against the live site |
| Website `make check`, `make packet`, `make gate`, `make deploy` | all passed; the listing is live and filed as a classroom tool |

## Waiting on the peer / Sam

1. **The name.** "Packet Alley" is a working title.
2. **The levels and their balance.** Nine levels, their networks, numbers and
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
