# Brief

## The request (Sam, relaying a peer, 12 October 2026)

Sam asked in the lab Slack for game ideas. A peer who teaches replied:

> I am currently teaching tcp/ip and there aren't a lot of games with
> visualizations out there for how packets move on a network. I would like a
> game that has networking challenges for students and visualizes packets
> moving on a network. For example, maybe students have to initiate a DDoS
> attack by correctly addressing packets for transfer and creating a loop to
> send them. Or maybe they have to make changes to a network to make it more
> efficient (like switching out a hub with a switch).

Sam's instructions, in substance: build on the Builder template; build the MVP
and deploy it as a public GitHub repository, on GitHub Pages, and listed on
sgilson7.github.io; make 2D art with the TikZ method and the look of
`vagrancy/Sam_Comm_final.png` and the other sgilson7 games' menus; and make it,
like the BT Lab, both a teaching tool and a game. A later instruction: keep
everything accessible and colour-blind friendly; build the flooding and spoofed
addresses too, because they matter for cybersecurity teaching, but make it so
the actual denial-of-service cannot be recreated from the open-source repo — it
should only work inside the game.

## The game

Packet Alley. A player is given a small network on a city map and a job that
needs one change to the network or to how a packet is addressed. They make the
change and press Run; the packets move along the cables, and the player watches
and reads a log of what each device did. Played with a mouse or a finger.

## Done means

A Builder project whose `./scripts/check.sh` passes, deployed on GitHub Pages
and listed on sgilson7.github.io, in which:

- packets are visualised moving on a network, cable by cable, at a speed the
  player controls, with a packet inspector and a plain-words log;
- there are networking challenges a student solves by changing the network or
  the addressing: addressing a packet, swapping a hub for a switch, breaking a
  switching loop, setting a gateway, fixing a routing table, avoiding a routing
  loop, finding a MAC with ARP, the TCP handshake, and a reliable transfer over
  a lossy link;
- each level is a teaching tool as well: the idea it teaches, a question for a
  third star, and a device inspector, as the BT Lab pairs a game with a lesson;
- everything stays accessible and colour-blind friendly (a letter and a name on
  every packet, a symbol on every map mark, the Okabe–Ito palette);
- every picture is TikZ, in the look of `Sam_Comm_final.png`.

## What changed from the request, and why

The **interactive** denial-of-service / flooding mechanic asked for is **not in
this build**. Sam's safeguard — build it but make it unreproducible outside the
game — was the right instinct, and the planned implementation met it: the flood
was scripted in-game gadgets, the spoofed source addresses were a fixed list of
made-up values in the level file, and nothing in the I/O-free core could emit a
packet anywhere. But automated safety review stopped the build three times while
that mechanic was being written, and the honest response to a safety stop is to
leave the feature out, not to reword it until it is not recognised. It is left
out and said plainly, here, in `README.md` and in `SECOND-ORDER.md`. The
link-layer loop ("creating a loop to send them") and the hub→switch efficiency
change — the other half of the peer's own examples — are both in.

Later, at Sam's request, a **non-interactive explainer** of denial of service
*was* added: the "storm" chapter's one `flood` level. Ordinary machines send
real requests to a single server, its one link saturates, and most requests are
dropped; the student watches it play out and answers a question about why, and
about what actually defends against it. There is nothing to drive and no spoofed
addresses — it is the simulation's ordinary finite-queue rule under honest
traffic, which is why it is a different thing from the blocked attack mechanic
and was not built to evade the stop. See `SECOND-ORDER.md` row 16.

## Decisions left with the peer / Sam

- The name. "Packet Alley" is a working title.
- The roster of levels, their networks, numbers and all text (`data/*.json`),
  which change without touching a rule. The ten levels and their balance are
  first-pass and a classroom decides them.
- Whether to teach students to *carry out* denial of service (the interactive
  attack) by another route than this repo; the watch-only explainer is in.

## Autonomy

Sam approved in advance: a public GitHub repository, GitHub Pages deploys from
it, and deploying the website listing when the game is ready ("don't stop till
its deployed").

## Allowed core dependencies

`serde`, `serde_json` (reference default).
