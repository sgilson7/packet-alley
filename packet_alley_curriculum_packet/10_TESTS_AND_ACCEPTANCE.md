# Validation plan: instructional claims need evidence

The existing repo's Builder CI is necessary, not sufficient. Extend it without weakening prior checks.

## Unit / Rust-core acceptance

| Test | Required behavior |
|---|---|
| `grade_requires_evidence` | A valid device change without prior prediction and selected evidence cannot earn the graded completion. |
| `case_commit_is_single_use` | Re-Run does not re-award the same scored variant. |
| `practice_variant_differs` | Same user case produces deterministic but different practice numbers/topology. |
| `no_precommit_answer_oracle` | No per-edit correct/incorrect feedback or route solution is returned before committed evaluation. |
| `alternate_spanning_tree_passes` | Two distinct valid connected acyclic forwarding sets both pass. |
| `invalid_spanning_tree_fails` | Disconnecting all paths fails. |
| `ip_vs_mac_fields_separate` | Local frames and network-layer destinations remain distinct. |
| `arp_is_link_local` | Neighbor query stays on the broadcast domain. |
| `route_return_and_prefix` | Route checks include forward/reverse path and most-specific destination match where the model supports it. |
| `tcp_model_is_labelled` | Go-Back-N teaching example is never presented as exact TCP. |
| `storm_is_watch_only` | No exposed attack construction, spoofing, or arbitrary traffic injection. |
| `no_answer_index_pattern` | No always-zero multiple-choice correct index and no direct route answer in displayed prompts. |

## Browser observation tasks

For each of 4 zone openings: inspect, interact with a demonstration packet or queue, predict, and enter scored level without copy revealing the scored configuration. Check keyboard and touchscreen-equivalent controls. For each of 19 levels: verify three meaningful scene interactions, one preflight evidence action, one prediction, one scored commit, honest event log, accessible symbols, zoom behavior, debrief only after run, and a new practice case after error. Run Chromium, Firefox, WebKit as the existing check requires.

## Source/prose checks

- 19 keys exactly match repo `data/levels.json`; 4 chapters match current IDs; no titles/interstitial copy invented by implementation agent.
- `data/textbook_extracts.json` contains every source ID in `data/textbook_source_manifest.json`, with source URL/license/full original sentence and content hash.
- Scientific explanations are either original verbatim excerpts from cited book or factual readouts of current simulator data; story prose contains neither protocol rules nor completed solutions.
- Game-only simulation assumptions are labelled, and differences from real protocols are review items rather than buried.
- No fixed-index quiz choices, pre-run “right/wrong” badges, answer-colored edges or hint text naming the exact selection.

## Human signoff required

1. Professor inspects all 19 missions and four examples for suitability and evidence-based assessment.
2. Networking specialist signs off MAC/IP, ARP, route matching, TCP handshake and GBN simplification.
3. Accessibility reader checks all scene manipulations in nonvisual/keyboard modes.
4. Owner approves final deployment; `./scripts/check.sh` and tests do not by themselves prove learning gains.

**Result in this packet:** JSON and packet-structure validation can be run here; the live project's Rust, WASM and browser checks cannot be claimed until the agent runs the repository locally.
