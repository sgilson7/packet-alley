# Packet Alley

A browser game that teaches TCP/IP by letting you watch packets move. You are
handed a small network in a cartoon city — computers, printers, a noodle shop's
server, hubs, switches and routers — and a job: address a packet so it arrives,
swap a hub for a switch to stop collisions, break a switching loop, give a host
its gateway, fix a routing table, point a packet away from a loop before its
hops run out, find a neighbour's hardware address, open a connection the TCP
way, and finish a transfer over a line that drops frames. You make a change,
press Run, and the fight plays out on the map: every frame rides its cable, and
a log in words says what each device did and why anything was lost.

**Play:** https://sgilson7.github.io/packet-alley/

Built for an instructor teaching TCP/IP who asked, on the lab Slack, for a game
with networking challenges that visualises packets moving on a network. Like the
BT Lab, it is meant to be both a teaching tool and a game: each level pairs a
challenge with the one idea it teaches, a question for a third star, and a
device inspector that shows what a switch or router actually knows.

## What it covers

Three chapters, nine levels, one layer of the stack at a time.

- **Frames on the Block (link layer).** IP and MAC addresses, how a switch
  learns and a hub does not, collisions, and a switching loop that storms.
- **Routes Across Town (network layer).** Gateways, the routing table,
  longest-prefix match, directly-connected blocks, and the TTL that stops a
  routing loop from lasting forever.
- **Handshakes (transport layer).** ARP, the three-way handshake and its
  acknowledgement numbers, and go-back-N retransmission on a lossy link.

Every packet is labelled with a letter and named in a list beside the map, and
every map mark is a symbol, so nothing is told apart by colour alone; the status
colours are the Okabe–Ito set, which stay distinct for colour-blind readers.

### Not in this build

A denial-of-service / flooding chapter was planned (spoofed source addresses, a
SYN flood, and the defences against it — SYN cookies, source-address filtering,
rate limiting). It is not included. Automated safety review repeatedly stopped
the build while that attack mechanic was being written, even though it ran only
inside a closed simulation, and the right response to a safety stop is not to
disguise the same feature and slip it past. An instructor who wants to teach
denial of service should use their own materials or a dedicated lab environment.
`SECOND-ORDER.md` records this in full.

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
