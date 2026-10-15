# Current-game audit: factual baseline

## Plot, delivery and controls
Packet Alley is a small city built around the noodle shop, printers, office registers, shop computers and a few servers. There is not yet a coherent chapter-spanning dramatic plot; each level invents a new maintenance problem. The two-dimensional network is already the strongest piece of the game: students select hosts, routers, cables and packet destinations, inspect addresses and device tables, and watch packets move with a plain-words trace. The Rust core simulates deterministic events and scoring; the browser renders them via thin WASM plumbing. Original content is stored in `data/levels.json` and `data/copy.json`.

The design brief records the instructor's desire for visualized packet movement and repair challenges, including replacing a hub with a switch and experimenting with network loops. It also records accessibility requirements. **Retain all of this.**

## Every current level and what it teaches

The IDs, original learning topics, and chapter sequence are drawn from the current source. **The interaction column describes the proposed redesign, not the original shipped implementation.** This distinction matters especially for `address`, which currently exposes an IP destination choice rather than a separate MAC-editing control.

| ID | Existing level | Existing learning objective | Required redesigned interaction | Audit finding |
|---|---|---|---|---|
| `address` | The kitchen order | Distinguish destination IP and delivery on one LAN | Address a packet using separate IP-destination and link-destination inspection | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `hub` | Lunch rush | Compare shared hub collisions versus switched forwarding | Swap central device on map, rerun one committed variant | One short menu can give away the answer. |
| `loop` | The backup line | Explain a broadcast loop and select a loop-free forwarding tree | Block any valid non-bridge cycle edge, not a pre-labelled “spare” | One short menu can give away the answer. |
| `crowd` | Four registers | Diagnose collision domains on a larger segment | Choose device types in two locations | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `backbone` | The duplicate paths | Detect multiple independent switching loops | Block enough ports to obtain connected acyclic forwarding paths | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `floor` | Open the upstairs floor | Combine hub replacement and loop prevention | Fix device roles and forwarding graph | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `gateway` | The other side of the street | Recognize local versus remote subnet and gateway | Set gateway via on-map interface selection, no labelled-correct option | One short menu can give away the answer. |
| `route` | A missing direction | Configure next-hop route and reverse route | Edit router route card with prefix and neighbor selector | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `ttl` | A message going in circles | Explain routing-loop behavior and IP TTL | Choose route edge | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `point` | Two missing settings | Combine first-hop gateway and static route | Set gateway and next hop after a preflight prediction | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `roads` | Market or harbor | Distinguish two destination prefixes and next hops | Drag prefix cards onto egress interfaces | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `across` | The far-side delivery | Combine route chain, return path and hop limit | Edit only missing entries plus initial TTL, certify route table reasoning | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `arp` | An unknown neighbor | Understand ARP request/reply for a local Ethernet address | Pick requested IP via scene, inspect learned mapping | One short menu can give away the answer. |
| `handshake` | Open the service desk | Order TCP SYN/SYN-ACK/ACK and acknowledgement semantics | Drag header flags and values into packet slots, core validates transcript | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `transfer` | A missing page | Explain retransmission and window in a simplified GBN model | Set window and timer after explaining expected repeat work | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `window` | The catalogue download | Explore GBN tradeoff between throughput and retransmission | Commit parameter card, run on shifted deterministic case | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `connect` | A connection across town | Recognize end-to-end TCP over routed IP | Set destination endpoint and handshake transcript | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `reliable` | The final report | Combine routing and lossy reliable transfer | Configure route plus window/timeout | A multi-setting task, but labels or a predictable quiz can still reveal the method. |
| `flood` | The night market crowd | Observe finite queue under concurrent honest demand | Watch-only traffic | Watch-only, no item edit; analysis must precede playback. |

## Confirmed copy/assessment concerns

1. In `address`, the existing story says an IP destination is sufficient for switch delivery; link-layer forwarding needs an Ethernet frame destination and MAC-table reasoning. The network simulation should show both layers clearly.
2. `arp` is placed in chapter `wires` with the TCP material. It belongs conceptually with local neighbor resolution; surface its concept in the link-layer orientation, even if its level ID/order cannot move yet.
3. `transfer`, `window` and `reliable` teach a go-back-N **teaching model**. Do not claim that modern TCP itself is simply go-back-N; label that simulator-specific simplification.
4. Some goals and tasks disclose solutions, e.g. “swap the hub for a switch” and “block the spare cable”. Avoid naming the necessary modification in the assignment.
5. Every current `quiz.answer` field inspected in `data/levels.json` has value `0`. A stable first-option answer is trivial to exploit even without inspecting source.
6. All level answers and configurations are embedded in public static JSON. A public offline program cannot hide every solution from a determined source inspector. Therefore the goal is **no answer disclosure in normal student gameplay** plus authentic problem-solving evidence, not impossible secrecy from a public repository.
7. The single `flood` lesson uses five legitimate clients and a finite queue. It demonstrates **congestion**, not by itself evidence of a malicious DDoS. Keep it observation-only as the brief requires.
8. The goals reward delivery, not always an explanation or prediction of the mechanism; repeated re-run can become an implicit answer oracle.
9. Introductory single-choice tasks undershoot the original instructor's request for high interaction with items in the view. Make direct manipulation the default; keep panels as equivalent accessible controls.
10. Scores and packet-event logs can disclose exact repairs following repeated trials. Keep diagnostic value while not identifying the correct action before the student provides reasoning.

## Existing architecture and safety boundaries

`BRIEF.md` describes the original request and removal of active denial-of-service construction. `BUILDER-SPEC.md` R1–R9 require a deterministic, I/O-free Rust core; decision-free wasm-bindgen shim; static browser renderer; and one predeploy check command. `README.md` lists four chapters, 19 levels, three-engine Playwright checks, symbol/letter packet labels, Okabe–Ito colors, map zoom, and the watch-only final level. These are **baseline requirements**, not redesign invitations.
