# Textbook exercises -> 2D interactive mechanics

The exercises below are cited with **exact source locations**. Two very short examples can be quoted in this packet; the longer originals remain at the linked CC-licensed source pages. The interactive versions are **new level designs**, not literal reproductions of the textbook's solutions or figures. The agent must never copy exercises that are hidden behind login, and must not put teacher-side answers into a student zone.

## Exact short source examples

From the book's LAN exercises (exercise 3): **“Compute the Spanning Tree of this network.”** Source: https://beta.computer-networking.info/syllabus/default/exercises/lan.html, item 3. This is a brief excerpt; use the complete exercise with diagram from the source when reviewing the design, with attribution.

From the book's TCP basics exercises introduction: **“A TCP connection always starts with a three-way handshake.”** Source: https://beta.computer-networking.info/syllabus/default/exercises/tcp.html. The page's individual interactive exercises require login, so we do not quote or scrape them.

## Blueprint translations

| Textbook location | Original cognitive task | 2D game translation | Not acceptable |
|---|---|---|---|
| LAN exercises 1–3, switched topology / spanning tree | Work out active and blocked ports from a network diagram | Tap each switch port; flag which to block; verify connected and acyclic graph with a pre-run prediction | A button whose label says “block the correct spare” |
| LAN exercise 2, failed link | Update port/address tables after a connection fails | Toggle a failed link in the demonstration; drag MAC-table records to their new learning ports | Spoil the correct port in mission text |
| LAN exercise 5, traceroute/frames/packets/segments | Explain successive layer headers along a path | Step packet along cable; inspect frame/IP/transport headers on each hop | Make only the IP destination visible |
| LAN exercise 6, connectivity | Diagnose whether two hosts can communicate, and fix it | Select a source and destination, predict first failure, then repair devices on-map | Trial-send on every candidate with scored feedback |
| [Building a network](https://beta.computer-networking.info/syllabus/default/principles/network.html), datagram/forwarding table examples | Follow every hop and detect black holes/loops | Draw next-hop arrows and read table entries; predict TTL at each hop | Show a glowing solved path on unopened scored task |
| [TCP basics](https://beta.computer-networking.info/syllabus/default/exercises/tcp.html), handshake exercises | Infer SYN/SYN-ACK/ACK and seq/ack state | Drag messages into timeline and build endpoint state ledger | Always choose the first answer; hand out `y+1` as an option annotated “correct” |
| [TCP-2 packetdrill demonstrations](https://beta.computer-networking.info/syllabus/default/exercises/tcp-2.html) | Predict stack response to a received segment | Inject only a simulated safe message into a miniature *in-game* endpoint, then inspect changed TCP state | Pretend simulator implements full Linux stack |
| [Reliable transfer](https://beta.computer-networking.info/syllabus/default/principles/reliability.html), go-back-N | Predict frames resent after simulated loss | Rearrange numbered cards into time slots and compare cumulative ACKs | Claim this simplified protocol is actual contemporary TCP |
| [Sharing resources](https://beta.computer-networking.info/syllabus/default/principles/sharing.html), congestion examples | Compare offered load, finite buffers, and delivery | Tap queue and capacity gauges during honest traffic, predict loss site and request count | Let students construct attack traffic or give attack recipes |

## Instructor constraint translation

**High interactivity:** for each level, at least three independently meaningful scene interactions: inspect a device/packet, select or annotate evidence directly on the map, manipulate a device/port/route/segment, and run/step through the result. Example scenes must be playable or scrubbable in the 2D view; no modal-wall lecture slides.

**Not guess-and-check:** before a scored run, students must make a visible prediction and cite map-based evidence. Scored submissions get one commit per preselected case. A wrong commit opens a fresh *practice* case with a different address layout/topology; retries are encouraged for learning but do not retroactively award the scored case. Ordinary post-run diagnostics discuss what happened without naming the exact replacement click.

**No direct answer leak:** zone examples use distinct numbers and graph structures from all scored levels; lesson passages describe rules at the conceptual level; options are not annotated with their correctness; no star quiz has a fixed answer position; no correct-route glow prior to student commitment. Public static code still remains inspectable, so do not claim impossible anti-cheat secrecy.
