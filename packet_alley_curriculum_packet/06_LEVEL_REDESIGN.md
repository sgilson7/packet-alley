# Nineteen level-specific interaction and objective contracts

The following is **implementation specification**, while exact human-facing prose is `data/verbatim_story_copy.json`. The ID is a stable internal key; do not rename or reorder existing levels without audited migration of save data. Every test must verify the cognitive act and not merely packet delivery.

### `address` — The kitchen order

- **Original objective:** Distinguish destination IP and delivery on one LAN.
- **Revised measurable objective:** Separate destination IP from Ethernet destination MAC; inspect a learned or flooded forwarding decision.
- **Required scene evidence:** Host IP, MAC, local prefix; switch source-MAC table before and after an observed demonstration.
- **Student prediction before Run:** Mark who should receive the IP packet and which MAC is used on the local frame.
- **Direct manipulation:** Address a packet using separate IP-destination and link-destination inspection; require evidence card.
- **Book exercise/model reference:** `E-LAN-5` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `hub` — Lunch rush

- **Original objective:** Compare shared hub collisions versus switched forwarding.
- **Revised measurable objective:** Predict a collision from simultaneous arrivals and select topology based on observed evidence.
- **Required scene evidence:** Arrival timestamps and hub forwarding inspector.
- **Student prediction before Run:** Predict collision location and cite shared-medium evidence.
- **Direct manipulation:** Swap central device on map, rerun one committed variant.
- **Book exercise/model reference:** `E-LAN-5` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `loop` — The backup line

- **Original objective:** Explain a broadcast loop and select a loop-free forwarding tree.
- **Revised measurable objective:** Identify cycles, preserve connectivity and distinguish forwarding/blocking ports.
- **Required scene evidence:** Cycle highlighting; MAC-table state; edge/port inspector.
- **Student prediction before Run:** Mark the cycle and identify an alternate reachable path.
- **Direct manipulation:** Block any valid non-bridge cycle edge, not a pre-labelled “spare”.
- **Book exercise/model reference:** `E-LAN-3` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `crowd` — Four registers

- **Original objective:** Diagnose collision domains on a larger segment.
- **Revised measurable objective:** Compare collision domains, multiple devices and forwarding evidence.
- **Required scene evidence:** Collision-domain overlays for all four hosts.
- **Student prediction before Run:** Mark all collision domains before edits.
- **Direct manipulation:** Choose device types in two locations; verify every register.
- **Book exercise/model reference:** `E-LAN-5` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `backbone` — The duplicate paths

- **Original objective:** Detect multiple independent switching loops.
- **Revised measurable objective:** Construct a connected acyclic forwarding subgraph rather than block named links.
- **Required scene evidence:** Port/link roles; connected-components preview (not solver).
- **Student prediction before Run:** Highlight all cycles on map.
- **Direct manipulation:** Block enough ports to obtain connected acyclic forwarding paths.
- **Book exercise/model reference:** `E-LAN-3` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `floor` — Open the upstairs floor

- **Original objective:** Combine hub replacement and loop prevention.
- **Revised measurable objective:** Commission a LAN using an evidence-based checklist.
- **Required scene evidence:** Two collision domains, redundant path and end-host delivery evidence.
- **Student prediction before Run:** Predict the number of independent faults and their consequences.
- **Direct manipulation:** Fix device roles and forwarding graph; completion requires all sources.
- **Book exercise/model reference:** `E-LAN-6` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `gateway` — The other side of the street

- **Original objective:** Recognize local versus remote subnet and gateway.
- **Revised measurable objective:** Use prefixes to decide local delivery versus first-hop routing.
- **Required scene evidence:** IP/prefix and router interface cards; packet encapsulation inspector.
- **Student prediction before Run:** Mark the local subnet and which interface shares it.
- **Direct manipulation:** Set gateway via on-map interface selection, no labelled-correct option.
- **Book exercise/model reference:** `E-NET-1` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `route` — A missing direction

- **Original objective:** Configure next-hop route and reverse route.
- **Revised measurable objective:** Read both forwarding tables and trace request/reply paths.
- **Required scene evidence:** R1 route table, connected networks on R2, reverse path.
- **Student prediction before Run:** Mark forward and return next hops.
- **Direct manipulation:** Edit router route card with prefix and neighbor selector.
- **Book exercise/model reference:** `E-NET-1` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `ttl` — A message going in circles

- **Original objective:** Explain routing-loop behavior and IP TTL.
- **Revised measurable objective:** Trace a loop and predict where TTL expires.
- **Required scene evidence:** Router tables, TTL decrement trace, path cycle.
- **Student prediction before Run:** Predict expiration hop from current TTL.
- **Direct manipulation:** Choose route edge; do not treat higher TTL as a repair for a loop.
- **Book exercise/model reference:** `E-NET-2` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `point` — Two missing settings

- **Original objective:** Combine first-hop gateway and static route.
- **Revised measurable objective:** Distinguish a host forwarding decision from a router forwarding decision.
- **Required scene evidence:** Source subnet and near-router route table.
- **Student prediction before Run:** Identify two distinct decision points with evidence.
- **Direct manipulation:** Set gateway and next hop after a preflight prediction.
- **Book exercise/model reference:** `E-NET-1` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `roads` — Market or harbor

- **Original objective:** Distinguish two destination prefixes and next hops.
- **Revised measurable objective:** Apply prefix-based forwarding for multiple destinations; add an overlap case.
- **Required scene evidence:** Destination prefix cards, R1 interfaces, two router paths.
- **Student prediction before Run:** Assign each destination to matching prefix first.
- **Direct manipulation:** Drag prefix cards onto egress interfaces; include overlapping-prefix variant.
- **Book exercise/model reference:** `E-NET-3` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `across` — The far-side delivery

- **Original objective:** Combine route chain, return path and hop limit.
- **Revised measurable objective:** Predict every forwarding step and TTL consumption.
- **Required scene evidence:** All route tables, hop count and reverse paths.
- **Student prediction before Run:** Draw all hops with TTL after each router.
- **Direct manipulation:** Edit only missing entries plus initial TTL, certify route table reasoning.
- **Book exercise/model reference:** `E-NET-2` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `arp` — An unknown neighbor

- **Original objective:** Understand ARP request/reply for a local Ethernet address.
- **Revised measurable objective:** Explain IP-to-MAC discovery and scope; separate from transport-layer connection state.
- **Required scene evidence:** ARP cache, broadcast reach, responding host.
- **Student prediction before Run:** Mark which machines receive a broadcast and which can answer.
- **Direct manipulation:** Pick requested IP via scene, inspect learned mapping; avoid showing answer in prompt.
- **Book exercise/model reference:** `E-LAN-5` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `handshake` — Open the service desk

- **Original objective:** Order TCP SYN/SYN-ACK/ACK and acknowledgement semantics.
- **Revised measurable objective:** Predict connection state and next expected sequence number, including ports.
- **Required scene evidence:** Endpoint state panels, TCP four-tuple and sequence/ack ledger.
- **Student prediction before Run:** Place three handshake events on timeline and derive ACK.
- **Direct manipulation:** Drag header flags and values into packet slots, core validates transcript.
- **Book exercise/model reference:** `E-TCP-1` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `transfer` — A missing page

- **Original objective:** Explain retransmission and window in a simplified GBN model.
- **Revised measurable objective:** Compare a simplified Go-Back-N teaching model with real TCP; predict loss recovery.
- **Required scene evidence:** Timeline of part numbers, window and timeout; explicit “simplified Go-Back-N” badge.
- **Student prediction before Run:** Mark lost part and predicted resend set.
- **Direct manipulation:** Set window and timer after explaining expected repeat work.
- **Book exercise/model reference:** `E-REL-1` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `window` — The catalogue download

- **Original objective:** Explore GBN tradeoff between throughput and retransmission.
- **Revised measurable objective:** Use trace measurements to justify window/timeout, contrast TCP congestion-control concepts.
- **Required scene evidence:** Timeline with RTT estimate, loss schedule, retransmit counts.
- **Student prediction before Run:** Predict comparative completion and redundant transmissions.
- **Direct manipulation:** Commit parameter card, run on shifted deterministic case.
- **Book exercise/model reference:** `E-REL-1` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `connect` — A connection across town

- **Original objective:** Recognize end-to-end TCP over routed IP.
- **Revised measurable objective:** Preserve endpoint addresses and port/ACK semantics across routers.
- **Required scene evidence:** TCP/IP header layers before and after router, flags/ports.
- **Student prediction before Run:** Mark which fields may change at each hop and which are end-to-end.
- **Direct manipulation:** Set destination endpoint and handshake transcript; inspect multi-layer path.
- **Book exercise/model reference:** `E-TCP-1` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `reliable` — The final report

- **Original objective:** Combine routing and lossy reliable transfer.
- **Revised measurable objective:** Plan an end-to-end delivery with correct route and modeled loss recovery.
- **Required scene evidence:** Route/next hop, loss trace, sender timeline.
- **Student prediction before Run:** Predict route and recovery cost before run.
- **Direct manipulation:** Configure route plus window/timeout; certify clean transfer.
- **Book exercise/model reference:** `E-REL-1` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


### `flood` — The night market crowd

- **Original objective:** Observe finite queue under concurrent honest demand.
- **Revised measurable objective:** Separate congestion from intentional denial of service; reason about capacity, fairness, and service loss.
- **Required scene evidence:** Queue depth, link service rate, requests accepted/dropped, loss causes.
- **Student prediction before Run:** Predict which link backs up and identify queue evidence.
- **Direct manipulation:** Watch-only traffic; nonattack analysis and optional non-operational capacity thought experiment.
- **Book exercise/model reference:** `E-SHARE-1` in `03_TEXTBOOK_EXERCISE_MAP.md`.
- **Acceptance:** success requires matching target outcomes **and** a precommitted evidence-based prediction validated by the Rust core; old stars/progress must remain legible.


## Integrity boundaries

1. Changing the parameter using a menu with only two choices is **not** evidence of learning. Test a changed scenario with new valid numbers/ports and require reasoning that transfers.
2. Quiz correctness is not determined by the index of an option supplied in publicly readable JSON; answer position must not remain constant. Use source-backed construction/evidence checks where possible.
3. Display post-run observational causes (drop, queue, TTL decrement), but withhold prescriptive “click this exact option next” messages on a scored case.
4. Do not make an incorrect attempt a permanent barrier to teaching; practice scenarios permit safe retries and an instructor override.
5. For every scenario, find at least one nonworking alternative and a distinct equivalently valid solution if multiple spanning trees exist; score graph invariants rather than hardcoded edges.
