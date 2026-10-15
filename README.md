# Packet Alley

A browser game that teaches TCP/IP by letting you watch packets move. You are
handed a small network in a cartoon city — computers, printers, a noodle shop's
server, hubs, switches and routers — and a job: address a packet so it arrives,
swap a hub for a switch to stop collisions, break a switching loop, give a host
its gateway, fix a routing table, point a packet away from a loop before its
hops run out, find a neighbour's hardware address, open a connection the TCP
way, and finish a transfer over a line that drops frames. You change the network
on the map itself — click a device to swap it, click a waiting packet and point
at the machine to send it to, click a cable to block it — then press Run, and
the fight plays out: every frame rides its cable, and a log in words says what
each device did and why anything was lost.

**Play:** https://sgilson7.github.io/packet-alley/

Built for an instructor teaching TCP/IP who asked, on the lab Slack, for a game
with networking challenges that visualises packets moving on a network. Like the
BT Lab, it is meant to be both a teaching tool and a game: each level pairs a
challenge with the one idea it teaches, a question for a third star, and a
device inspector that shows what a switch or router actually knows.

## What it covers

Four chapters, nineteen levels, roughly one layer of the stack at a time. Each
teaching chapter opens with a set of short, one- or two-choice missions — see
what happens — and then a tougher follow-up set of three, where you configure a
whole larger network: more machines, more hops, more decisions, one wrong choice
anywhere and the goal fails. The difficulty ramps, and only the last mission of
each chapter gets genuinely technical.

- **Frames on the Block (link layer).** IP and MAC addresses, how a switch
  learns and a hub does not, collisions, and a switching loop that storms — then
  splitting collision domains across more machines, breaking several loops on a
  redundant backbone, and commissioning a whole segment at once.
- **Routes Across Town (network layer).** Gateways, the routing table,
  longest-prefix match, directly-connected blocks, and the TTL that stops a
  routing loop from lasting forever — then pointing two roads apart at a
  junction and routing a packet, hop by hop, clear across a four-router town.
- **Handshakes (transport layer).** ARP, the three-way handshake and its
  acknowledgement numbers, and go-back-N retransmission on a lossy link — then
  tuning window and timeout for a bigger file, opening a connection across a
  router, and running a reliable transfer across town over a lossy last mile.
- **When Too Much Arrives (availability).** One watch-only explainer: a flood
  of ordinary requests overwhelms a single server's link, most are dropped, and
  a question draws out why — and what actually defends against it. See below.

Every change is made on the map — click an edit badge to open a chooser right
there, or point at a glowing machine to send a packet to it — with the same
choices mirrored in a side panel you can hide. Every choice is a click, never
typed text, so no one has to hand-enter an address, and no one can pass by
guessing a two-button menu. Every packet is labelled with a letter and named in a list beside the
map, and every map mark is a symbol, so nothing is told apart by colour alone;
the status colours are the Okabe–Ito set, which stay distinct for colour-blind
readers. The map has a zoom slider (with `+`/`−` and reset) and scrolls when
enlarged, so a crowded network can be read up close; browser zoom works too.

### Denial of service

The fourth chapter, **When Too Much Arrives**, is a single *explainer* level you
watch rather than play: five ordinary machines send real requests to one small
server, the one cable from the switch to the server saturates, and most requests
are dropped. You watch it play out and answer a question about why — the picture
of a service overwhelmed, and why spreading load and filtering at the edge are
the defences. There is nothing to drive: it uses the simulation's ordinary
finite-queue rule under honest traffic, with no spoofed addresses.

What is **not** here is an *interactive* attack: no student-built flood, no
spoofed source addresses, no SYN flood to assemble. That mechanic was planned,
but automated safety review repeatedly stopped the build while it was being
written — even inside a closed simulation — and the right response to a safety
stop is not to disguise the same feature and slip it past. An instructor who
wants students to *carry out* a denial-of-service attack should use their own
materials or a dedicated lab environment. `SECOND-ORDER.md` records this in full.

## How it is built

A [Builder](https://github.com/sgilson7/builder-setting) project: every rule is
in a Rust core (`crates/core`) in integer maths — the address arithmetic, the
whole network simulation, the scoring — compiled to WebAssembly through a shim
that decides nothing (`crates/shim`), and a static page (`web/`) that draws what
the core returns and holds no rule of its own. The simulated network is values
in the core; nothing in the core performs input or output, so it cannot reach a
real network. Every picture is drawn in TikZ (`tools/`), after vagrancy's
`Sam_Comm_final.png` and the other sgilson7 games' menus.

    ./scripts/setup.sh         # once: Playwright and three browser engines
    ./scripts/check.sh         # format, lint, tests, build, WebAssembly audit, browsers
    python3 scripts/serve.py   # then open http://127.0.0.1:8000
    python3 tools/art.py       # redraw web/art/ (needs pdflatex and pdftocairo)

`BRIEF.md` has the request as it was given, `PLAN.md` the decisions taken from
it, `SECOND-ORDER.md` what came up while building, and `HANDOFF.md` what was
checked. The levels, their networks and all reader-visible text are data
(`data/levels.json`, `data/copy.json`) and change without touching a rule.

MIT licence; see `LICENSE`.
