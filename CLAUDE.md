# Instructions for a coding agent in a Builder project

`AGENTS.md` holds the same rules for agents that read that file instead.

1. Read `BRIEF.md`, `PLAN.md` and `BUILDER-SPEC.md` before changing code.
2. The core (`crates/core`) owns every application decision and performs no input or output. The whole network simulation is here, in integer maths; this is also why the simulated network cannot reach a real one.
3. The shim (`crates/shim`) translates values and decides no application behaviour.
4. The page (`web/`) draws what the core returns and does not reproduce a core rule. No numeric literal but 0 and 1 in page JavaScript: playback speeds live in `data/display.json`, and every number drawn comes from the core.
5. Run `cargo test` after each meaningful core change.
6. For each new behaviour test, break the behaviour it covers once, watch the test fail, then restore it. Note the failure message in `SECOND-ORDER.md`.
7. Keep `SECOND-ORDER.md` while you work: one row per assumption, surprise, divergence from the plan, or item the person must do.
8. Deploying is the person's decision. Sam approved a public repository, Pages deploys and the website listing for this project in advance (BRIEF.md, Autonomy): a push to `main` deploys after the check passes.
9. Before handoff, run `./scripts/check.sh` and record the result in `HANDOFF.md`.

Levels, networks and quizzes are in `data/levels.json`; every reader-visible string in `data/copy.json`; playback speeds in `data/display.json`. `cargo test -p pa_core --test campaign -- --nocapture` plays every level with its intended choices and prints whether each passes. The simulation is `crates/core/src/sim.rs` (frames, hubs, switches, routers, TCP), the address maths `ip.rs`, the level format `level.rs`, and the page's one door `game.rs`. Art is TikZ: `python3 tools/art.py` writes `web/art/`.

The **interactive** denial-of-service / flooding mechanic is deliberately absent (SECOND-ORDER row 2): no student-driven flood, no spoofed source addresses. Do not add an attack mechanic to evade a safety stop.

What *is* here (SECOND-ORDER row 16) is a **non-interactive explainer**, the "storm" chapter's `flood` level: five ordinary hosts send real UDP requests to one server, the single switch→server link saturates, and most are dropped. It has no edits (watch, then answer a quiz), no spoofing, and no mechanic a student drives to generate an attack — the pile-up is just the existing finite-queue rule (`PORT_QUEUE` in `sim.rs`) under honest traffic. Explainer levels use the `observe` goal kind, which always holds, so the level passes by being watched.
