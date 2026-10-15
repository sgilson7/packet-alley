# Four zone-opening lessons: scripted view and sourced copy

Each zone uses a compact three-stage overlay **inside the existing zoomable 2D scene**: (1) select devices and inspect visible state; (2) scrub a prebuilt **demonstration** with a topology distinct from the graded tasks; (3) answer a small transfer prediction by marking a path or element on the map. The lesson closes when the prediction is submitted. Source text remains selectable and readable by assistive technology.

## Source rule

All statements that explain network mechanisms are populated from `data/textbook_source_manifest.json` via `scripts/extract_textbook.py` and stored at build time in `data/textbook_extracts.json`. The extractor picks whole original sentences, never generated rewrites. The UI presents a short excerpt with its textbook section/link/license. Narrative prompts below are original game copy and are not presented as textbook facts.

## Frames on the Block (`block`)

**Exact opening:** It is opening morning in Packet Alley. The noodle shop, its printers, and the nearby desks share one local network. Before anyone can depend on it, you need to find out what each device knows.

**Verbatim mechanics slots:** LAN_IP_VS_MAC, LAN_MAC_TABLE, LAN_LEARNING, LAN_LOOP. These are build-time generated from the textbook source, with per-excerpt source and license controls.

**Exact scene demo instructions:** A demonstration switch connects two sample computers and a test printer on TEST-NET. Click the first computer to see its IP address; click the printer to see its Ethernet address. Trigger one sample frame. Pause it at the switch, inspect its source MAC, then inspect the switch table after the frame. This demonstration uses distinct addresses and links from all scored missions.

**Student action:** first click each relevant object, then advance an animated packet or queue step, then mark one evidence item. Example has its own IDs and intentionally different addresses and topology from the scored network.

**Required conceptual coverage:** Destination IP versus destination MAC; Switch source-MAC learning and unknown-destination flooding; Collision domains and loop-free switching.

**Lesson exit check:** the student has inspected the highlighted records and submitted a prediction on an unseen, non-scored variant; do not show a solved main-level configuration.


## Routes Across Town (`town`)

**Exact opening:** The local network is running. Now the shop needs services on the other side of the city. Each router makes its own forwarding decision. Your job is to read those decisions before changing them.

**Verbatim mechanics slots:** IP_FORWARD, IP_BLACKHOLE, IP_CYCLE. These are build-time generated from the textbook source, with per-excerpt source and license controls.

**Exact scene demo instructions:** A demonstration host on 192.0.2.10/24 sends to 198.51.100.20 through a sample router on 192.0.2.1/24. Click the local prefix, select the router interface, then step through a packet and watch its hop limit decrease. The demonstration is read-only and does not reveal any mission next hops.

**Student action:** first click each relevant object, then advance an animated packet or queue step, then mark one evidence item. Example has its own IDs and intentionally different addresses and topology from the scored network.

**Required conceptual coverage:** Local prefixes and first-hop gateway; Forwarding tables, matching prefixes, next hops; Forward and reverse routes; hop limit.

**Lesson exit check:** the student has inspected the highlighted records and submitted a prediction on an unseen, non-scored variant; do not show a solved main-level configuration.


## Handshakes and Delivery (`wires`)

**Exact opening:** Messages can cross the city. The remaining problem is keeping a conversation open long enough to finish the work. The service desk needs a connection, and the shop needs complete files.

**Verbatim mechanics slots:** ARP_LOCAL, TCP_OPEN, TCP_BYTE_STREAM, GBN_MODEL. These are build-time generated from the textbook source, with per-excerpt source and license controls.

**Exact scene demo instructions:** In a demonstration network, a client at 203.0.113.10 and a server at 203.0.113.20 exchange a SYN, SYN-ACK, and ACK with numbers different from scored levels. Drag each message into a timeline, inspect the two endpoints, then watch one simulated loss in a separate simplified Go-Back-N teaching example. Label the two protocols separately.

**Student action:** first click each relevant object, then advance an animated packet or queue step, then mark one evidence item. Example has its own IDs and intentionally different addresses and topology from the scored network.

**Required conceptual coverage:** Ports, SYN/SYN-ACK/ACK, sequence/acknowledgment values; End-to-end transport versus router forwarding; Simplified Go-Back-N versus actual TCP; acknowledgments and recovery.

**Lesson exit check:** the student has inspected the highlighted records and submitted a prediction on an unseen, non-scored variant; do not show a solved main-level configuration.


## When Too Much Arrives (`storm`)

**Exact opening:** The night market is open. The city is busy, and a small server receives more requests than its link can carry. Your assignment is to observe what fails and explain the evidence.

**Verbatim mechanics slots:** CONGESTION_BUFFER, CONGESTION_QUEUE, CONGESTION_CAPACITY. These are build-time generated from the textbook source, with per-excerpt source and license controls.

**Exact scene demo instructions:** A sample queue receives two ordinary requests each step while a sample link transmits one. Pause and inspect the queue count, then run to a fixed limit. This is an observation of shared capacity, not an attack or attack-building exercise.

**Student action:** first click each relevant object, then advance an animated packet or queue step, then mark one evidence item. Example has its own IDs and intentionally different addresses and topology from the scored network.

**Required conceptual coverage:** Link capacity and finite buffers; Offered load, queue growth, drops and fairness; Congestion is not itself proof of an attack.

**Lesson exit check:** the student has inspected the highlighted records and submitted a prediction on an unseen, non-scored variant; do not show a solved main-level configuration.

## Anti-spoiler test

Run each demo side by side with its chapter's first scored level: neither layout, host address, route table, correct port, predicted outcome nor scenario identity is the same. Verify no content cards use “choose the switch,” “block the spare,” “use 10.x.x.1” or a scoring threshold to reveal the answer. A student who only watched the demonstration must still have to reason from the scored map.
