"""Packet Alley: the nine device icons (web/art/icons/<id>.png, 3 x 3 cm at
150 dpi, transparent), drawn as TikZ and compiled by tools/art.py. Never
generated pixels.

Each network device is a cute city object in vagrancy/Sam_Comm_final.png's
look: flat fills, every shape outlined in the one plum ink, sage, teal and
coral machines. The map shows them at 64-72 px wide, so each is one bold
silhouette that fills most of the square, inked at 1.8 pt (tools/art.py's
INK["icons"]), with small detail only where it says what the device does.

Facts each icon must get right (what an image generator gets wrong), and
where this file enforces each:

- readable at 64-72 px: every icon spans at least 75% of the canvas width
  (the tall server rack two thirds; `span` asserts the body width per
  icon), the ink is 1.8 pt, and
  no detail is drawn finer than 0.5 pt.
- nothing floats: every icon stands on a soft shadow at the canvas foot
  (`shadow`), and every part that rests on another shares its y constant
  (the laptop's base sits on DESK_TOP, the noodle bowl's foot on RACK_TOP,
  the printer's slip starts inside SLOT_Y, the megaphone's post on HUB_TOP).
- host: an open laptop, screen facing the viewer, on a little desk.
- printer: a kitchen ticket printer; the paper slip comes out of the slot on
  top and curls over; its lines are bars, never writing.
- hub: a dull grey box with exactly four ports (PORTS, asserted), every
  port's light lit the same, and a megaphone on top with sound arcs going
  both ways, because a hub repeats every frame to every port.
- switch: a teal box with exactly four ports, each with its own green light,
  and two arrows crossing on its face, because a switch sends each frame
  only to the port where it belongs.
- router: a squat box with exactly two antennas and a compass rose on its
  face: it picks a route between blocks.
- server: a rack of three units with lights, a noodle bowl with chopsticks
  on top, and steam rising from the bowl (not from the rack).
- firewall: bricks laid in running bond (each row offset by half a brick,
  `bricks`), an arched gate with bars in the wall, and a flame above it.
- uplink: a lattice radio mast standing on the ground, a puffy cloud drawn
  behind it (first), and signal arcs on both sides of its tip.
- toaster: a toaster with two slices of toast, exactly one tiny antenna and
  exactly one glowing eye under a slanted lid (mischief), so it reads as a
  gadget gone bad rather than a kitchen toaster.
- colour-blind safe: each icon is told apart by silhouette alone (laptop on
  desk, box with paper slip, box with megaphone, tall box with crossing
  arrows, squat box with two antennas, rack with bowl, brick wall with
  flame, lattice mast with cloud, toaster with toast); checked by a
  greyscale contact sheet. Status lights use the Okabe-Ito set (`oigreen`,
  `oiorange`, `oiyellow`, `oiverm`, `oisky`, `oiblue` in artlib.COLOURS),
  never red against green; every body fill is much lighter than the ink so
  the outline shows in greyscale; the switch's two arrows are one dark
  (oiblue) and one mid (oiorange) so they stay two arrows in greyscale.
- no words, letters or digits (tools/art.py's `check_no_text`); the screens
  and tickets carry bars and dots only.
- transparent background: tools/art.py renders the icons folder with
  pdftocairo -transp, and nothing here paints the whole canvas.
"""
import math

from artlib import P, f, rect, rrect, circle, ellipse, poly, line, curve, place, glint, tube, cloud, leaf
import artlib

S = 3.0  # the canvas is S x S cm
PORTS = 4


def lw(w):
    return f",line width={f(w)}pt"


def fillo(path, colour, op):
    return f"\\fill[{colour}, opacity={f(op)}] {path};\n"


def shadow(x0=0.35, x1=2.65, y=0.2):
    return fillo(f"{P((x0 + x1) / 2, y)} ellipse ({f((x1 - x0) / 2)} and 0.12)", "ink", 0.18)


def span(x0, x1, at_least=0.75):
    assert (x1 - x0) >= at_least * S, f"icon body spans {x1 - x0:.2f} of {S}"


def ports(x0, x1, y, light, n=PORTS):
    """n sockets along a box's face between x0 and x1, a light over each."""
    s = ""
    w = (x1 - x0) / n
    for k in range(n):
        cx = x0 + w * (k + 0.5)
        s += rect(cx - 0.17, y, cx + 0.17, y + 0.3, "ink!85!metaldark", lw(1.1))
        s += rect(cx - 0.08, y + 0.2, cx + 0.08, y + 0.3, "metallight", lw(0.7))
        if light:
            s += circle(cx, y + 0.47, 0.075, light, lw(0.8))
    return s


# --------------------------------------------------------------------------
# the nine devices
# --------------------------------------------------------------------------

DESK_TOP = 1.05


def host():
    """A laptop on a little desk."""
    span(0.25, 2.75)
    s = shadow(0.3, 2.7, 0.18)
    # the desk: a slab on two legs with a cross bar
    s += tube([[(0.55, DESK_TOP - 0.1), (0.5, 0.2)], [(2.45, DESK_TOP - 0.1), (2.5, 0.2)]], 0.16, "wooddark")
    s += line((0.55, 0.5), (2.45, 0.5), lw=2.4)
    s += line((0.55, 0.5), (2.45, 0.5), lw=1.0, colour="wooddark")
    s += rrect(0.25, DESK_TOP - 0.2, 2.75, DESK_TOP, "wood", 0.06)
    # the screen, open, facing us
    s += rrect(0.62, DESK_TOP + 0.2, 2.38, 2.65, "metaldark", 0.1)
    s += rrect(0.75, DESK_TOP + 0.33, 2.25, 2.52, "rangedsoft", 0.04, lw(1.0))
    # on the screen: a window bar, an envelope (a packet) and two lines
    s += rect(0.75, 2.3, 2.25, 2.52, "teal", lw(1.0))
    for k, c in enumerate(("oiverm", "oiyellow", "oigreen")):
        s += circle(0.9 + k * 0.17, 2.41, 0.05, c, lw(0.6))
    s += envelope(1.08, 1.62, 0.62, 0.42, "pcoral", 1.0)
    s += rect(1.85, 1.95, 2.1, 2.05, "tealdark", lw(0.6))
    s += rect(1.85, 1.72, 2.1, 1.82, "tealdark", lw(0.6))
    # the keyboard base, sitting on the desk top
    s += poly([(0.42, DESK_TOP), (2.58, DESK_TOP), (2.38, DESK_TOP + 0.22), (0.62, DESK_TOP + 0.22)], "metallight")
    s += line((1.25, DESK_TOP + 0.08), (1.75, DESK_TOP + 0.08), lw=1.0)
    # a mug beside it would crowd the desk; a sticker on the lid edge instead
    return s, (S, S)


def envelope(x0, y0, w, h, fill, lw_=1.0):
    """A closed envelope: the packet's shape everywhere in the game."""
    s = rect(x0, y0, x0 + w, y0 + h, fill, lw(lw_))
    s += f"\\draw[ink, line width={f(lw_)}pt, line join=round] {P(x0, y0 + h)} -- {P(x0 + w / 2, y0 + h * 0.42)} -- {P(x0 + w, y0 + h)};\n"
    return s


SLOT_Y = 1.55


def printer():
    """A kitchen ticket printer with a slip curling out of its slot."""
    span(0.35, 2.65)
    s = shadow(0.35, 2.65, 0.2)
    # the slip, drawn first so the printer's lip covers its foot
    x0, x1 = 0.98, 2.02
    s += f"\\filldraw[inked,fill=cream] {P(x0, SLOT_Y - 0.1)} -- {P(x0 - 0.06, 2.35)} .. controls {P(x0 - 0.1, 2.75)} and {P(x0 + 0.2, 2.95)} .. {P(x0 + 0.55, 2.9)} -- {P(x1 + 0.55, 2.9)} .. controls {P(x1 + 0.2, 2.95)} and {P(x1 - 0.1, 2.75)} .. {P(x1 - 0.06, 2.35)} -- {P(x1, SLOT_Y - 0.1)} -- cycle;\n"
    # the curl's underside, rolling forward
    s += f"\\filldraw[inked,fill=paper] {P(x0 + 0.55, 2.9)} -- {P(x1 + 0.55, 2.9)} .. controls {P(x1 + 0.75, 2.88)} and {P(x1 + 0.72, 2.62)} .. {P(x1 + 0.5, 2.62)} -- {P(x0 + 0.5, 2.62)} .. controls {P(x0 + 0.72, 2.62)} and {P(x0 + 0.75, 2.88)} .. cycle;\n"
    # ticket lines: bars, never writing
    for k, (a, b) in enumerate(((0.12, 0.75), (0.12, 0.55), (0.12, 0.85), (0.12, 0.45))):
        y = 2.3 - k * 0.2
        s += rect(x0 + a - 0.03 * k * 0, y, x0 + b, y + 0.07, "ink!70!cream", lw(0.7))
    # the printer body, sage, chunky
    s += rrect(0.35, 0.2, 2.65, SLOT_Y, "sage", 0.2)
    s += rrect(0.35, SLOT_Y - 0.32, 2.65, SLOT_Y, "sagelight", 0.12)
    s += rrect(0.85, SLOT_Y - 0.2, 2.15, SLOT_Y - 0.08, "ink", 0.04, lw(0.8))
    # front: a red tear bar, a button and a green light
    s += rrect(0.6, 0.45, 1.75, 0.95, "sagedark", 0.08, lw(1.2))
    s += rect(0.75, 0.62, 1.6, 0.78, "ink!80!sagedark", lw(0.8))
    s += circle(2.1, 0.8, 0.2, "oiverm")
    s += glint(2.05, 0.86, 0.05)
    s += circle(2.1, 0.4, 0.07, "oigreen", lw(0.8))
    return s, (S, S)


HUB_TOP = 1.35


def hub():
    """A dull grey box with four ports and a megaphone shouting from its
    roof, both ways."""
    span(0.25, 2.75)
    s = shadow(0.3, 2.7, 0.2)
    # the megaphone on its post, pointing up and right
    s += tube([[(1.25, HUB_TOP), (1.25, 1.75)]], 0.14, "hubgreydark")
    mouth = (2.5, 2.35)
    s += poly([(0.85, 1.85), (1.0, 2.15), (2.4, 2.75), (2.6, 1.95)], "gold")
    s += ellipse(2.5, 2.35, 0.17, 0.42, "goldlight", rot=-12)
    s += ellipse(2.52, 2.35, 0.08, 0.22, "ink!80!gold", rot=-12, extra=lw(0.8))
    s += rrect(0.62, 1.78, 0.95, 2.2, "hubgreydark", 0.06)
    s += poly([(1.2, 1.95), (1.4, 1.7), (1.55, 1.75), (1.35, 2.0)], "hubgreydark", lw(1.2))
    # sound arcs: forward from the mouth, and back over the left too
    for k, r in enumerate((0.25, 0.45)):
        a, b = mouth
        s += f"\\draw[ink, line width=1.6pt, line cap=round] {P(a + 0.12 + r * math.cos(math.radians(-40)), b + r * math.sin(math.radians(-40)))} arc[start angle=-40, end angle=50, radius={f(r)}];\n"
    for k, r in enumerate((0.2, 0.38)):
        a, b = 0.6, 2.05
        s += f"\\draw[ink, line width=1.6pt, line cap=round] {P(a - 0.05 + r * math.cos(math.radians(130)), b + r * math.sin(math.radians(130)))} arc[start angle=130, end angle=220, radius={f(r)}];\n"
    # the box
    s += rrect(0.25, 0.2, 2.75, HUB_TOP, "hubgrey", 0.14)
    s += rect(0.25, HUB_TOP - 0.22, 2.75, HUB_TOP - 0.12, "hubgreydark", lw(0.8))
    s += ports(0.45, 2.55, 0.38, "oiyellow")
    return s, (S, S)


def switch():
    """A teal box with four ports, a green light over each, and two arrows
    crossing on its face."""
    span(0.25, 2.75)
    s = shadow(0.3, 2.7, 0.2)
    s += rrect(0.25, 0.2, 2.75, 2.35, "teal", 0.18)
    s += rrect(0.25, 2.05, 2.75, 2.35, "tealdark", 0.12)
    for x in (0.45, 2.55):
        s += circle(x, 2.2, 0.06, "metallight", lw(0.7))
    # the face panel with the crossing arrows
    s += rrect(0.5, 1.08, 2.5, 1.95, "white!80!rangedsoft", 0.1, lw(1.2))
    # one dark, one mid arrow, so they stay two arrows in greyscale
    s += fat_arrow((0.72, 1.85), (2.3, 1.18), "oiblue", w=0.2, head=0.26)
    s += fat_arrow((0.72, 1.18), (2.3, 1.85), "oiorange", w=0.2, head=0.26)
    # the ports along the bottom, each with its own green light
    s += ports(0.4, 2.6, 0.38, "oigreen")
    return s, (S, S)


def fat_arrow(a, b, fill, w=0.12, head=0.2):
    """A straight outlined arrow from a to b."""
    (ax, ay), (bx, by) = a, b
    L = math.hypot(bx - ax, by - ay)
    ux, uy = (bx - ax) / L, (by - ay) / L
    nx, ny = -uy, ux
    neck = (bx - ux * head * 1.3, by - uy * head * 1.3)
    pts_ = [(ax + nx * w / 2, ay + ny * w / 2), (neck[0] + nx * w / 2, neck[1] + ny * w / 2),
            (neck[0] + nx * head, neck[1] + ny * head), (bx, by),
            (neck[0] - nx * head, neck[1] - ny * head), (neck[0] - nx * w / 2, neck[1] - ny * w / 2),
            (ax - nx * w / 2, ay - ny * w / 2)]
    return poly(pts_, fill, lw(1.2))


ANTENNAS = 2


def router():
    """A squat box with two antennas and a compass rose on its face."""
    span(0.2, 2.8)
    s = shadow(0.25, 2.75, 0.2)
    tips = [(0.55, 2.7), (2.45, 2.7)]
    assert len(tips) == ANTENNAS
    # signal arcs off each tip
    for (x, y), sgn in zip(tips, (-1, 1)):
        for r in (0.18, 0.32):
            start = 90 - sgn * 20 - 50
            s += f"\\draw[ink, line width=1.4pt, line cap=round] {P(x + r * math.cos(math.radians(start)), y + r * math.sin(math.radians(start)))} arc[start angle={start}, end angle={start + 100}, radius={f(r)}];\n"
    s += tube([[(0.7, 1.4), tips[0]], [(2.3, 1.4), tips[1]]], 0.13, "metaldark")
    for (x, y) in tips:
        s += circle(x, y, 0.1, "oiverm", lw(1.2))
    # the box
    s += rrect(0.2, 0.2, 2.8, 1.55, "wallD", 0.25)
    s += rrect(0.2, 1.25, 2.8, 1.55, "wallB", 0.15)
    # lights along the right
    for k, c in enumerate(("oigreen", "oiorange")):
        s += circle(2.45, 0.55 + k * 0.32, 0.075, c, lw(0.8))
    for k, c in enumerate(("oigreen", "oiorange")):
        s += circle(0.55, 0.55 + k * 0.32, 0.075, c, lw(0.8))
    # the compass rose
    cx, cy, R = 1.5, 0.78, 0.5
    s += circle(cx, cy + 0.02, R, "cream", lw(1.4))
    big = []
    for k in range(8):
        a = math.radians(90 + k * 45)
        r = R * 0.88 if k % 2 == 0 else R * 0.22
        big.append((cx + r * math.cos(a), cy + 0.02 + r * math.sin(a)))
    s += poly(big, "oisky!45!white", lw(1.0))
    # the north point picked out: the route chosen
    s += poly([(cx, cy + 0.02 + R * 0.88), (cx - R * 0.16, cy + 0.02 + R * 0.16), (cx + R * 0.16, cy + 0.02 + R * 0.16)], "oiverm", lw(1.0))
    s += circle(cx, cy + 0.02, 0.06, "gold", lw(0.8))
    return s, (S, S)


RACK_TOP = 1.95


def server():
    """The Noodle Shop's web server: a rack with lights and a steaming bowl."""
    span(0.5, 2.5, 0.66)
    s = shadow(0.45, 2.55, 0.18)
    # the rack
    s += rrect(0.5, 0.18, 2.5, RACK_TOP, "metal", 0.12)
    for k in range(3):
        y0 = 0.33 + k * 0.53
        s += rrect(0.65, y0, 2.35, y0 + 0.42, "metallight", 0.06, lw(1.2))
        for j in range(3):
            s += line((0.8 + j * 0.16, y0 + 0.1), (0.8 + j * 0.16, y0 + 0.32), lw=1.0)
        lights = [("oigreen", "oiorange"), ("oiorange", "oigreen"), ("oigreen", "oigreen")][k]
        for j, c in enumerate(lights):
            s += circle(1.95 + j * 0.2, y0 + 0.21, 0.07, c, lw(0.8))
        s += rect(1.35, y0 + 0.15, 1.7, y0 + 0.27, "ink!80!metaldark", lw(0.6))
    rim = 2.62
    # steam rising from the bowl
    for x, h in ((1.2, 0.3), (1.5, 0.36), (1.8, 0.3)):
        s += curve([(x, rim + 0.1), (x + 0.1, rim + 0.1 + h * 0.35), (x - 0.08, rim + 0.1 + h * 0.7), (x + 0.04, rim + 0.1 + h)], lw=1.6, colour="ink!45!white")
    # chopsticks behind the rim
    s += tube([[(1.75, rim - 0.1), (2.6, 2.98)], [(1.9, rim - 0.15), (2.78, 2.88)]], 0.07, "woodlight")
    # the bowl, deep, its foot on the rack's top
    s += rect(1.25, RACK_TOP, 1.75, RACK_TOP + 0.1, "bowl", lw(1.2))
    s += f"\\filldraw[inked,fill=bowl] {P(0.72, rim)} .. controls {P(0.74, 2.1)} and {P(1.1, RACK_TOP + 0.08)} .. {P(1.5, RACK_TOP + 0.08)} .. controls {P(1.9, RACK_TOP + 0.08)} and {P(2.26, 2.1)} .. {P(2.28, rim)} -- cycle;\n"
    s += rect(0.76, 2.3, 2.24, 2.4, "oiverm", ",draw=none")
    s += ellipse(1.5, rim, 0.78, 0.14, "broth")
    for k in range(4):
        x = 1.05 + k * 0.25
        s += curve([(x, rim - 0.04), (x + 0.07, rim + 0.05), (x + 0.15, rim - 0.04)], lw=1.0, colour="noodle!70!wooddark")
    s += circle(1.85, rim + 0.02, 0.07, "greenC", lw(0.8))
    return s, (S, S)


def firewall():
    """A brick wall segment with an arched barred gate and a flame above."""
    span(0.2, 2.8)
    s = shadow(0.2, 2.8, 0.2)
    x0, x1, y0, y1 = 0.2, 2.8, 0.2, 2.05
    rows, bw = 5, 0.52
    rh = (y1 - y0) / rows
    # the flame first, so the wall's top covers its foot
    s += f"\\filldraw[inked,fill=hot] {P(1.0, y1)} .. controls {P(0.85, 2.45)} and {P(1.25, 2.55)} .. {P(1.2, 2.95)} .. controls {P(1.45, 2.7)} and {P(1.5, 2.6)} .. {P(1.55, 2.5)} .. controls {P(1.65, 2.65)} and {P(1.85, 2.8)} .. {P(1.85, 2.95)} .. controls {P(2.15, 2.6)} and {P(2.1, 2.3)} .. {P(2.0, y1)} -- cycle;\n"
    s += f"\\filldraw[inked,fill=gold{lw(1.0)}] {P(1.2, y1)} .. controls {P(1.15, 2.3)} and {P(1.4, 2.4)} .. {P(1.45, 2.6)} .. controls {P(1.6, 2.45)} and {P(1.85, 2.3)} .. {P(1.8, y1)} -- cycle;\n"
    # the mortar bed, then bricks in running bond
    s += clipped_rect(x0, y0, x1, y1, rect(x0, y0, x1, y1, "mortar") + bricks(x0, x1, y0, rows, rh, bw))
    # a coping course along the top
    s += rect(x0 - 0.05, y1, x1 + 0.05, y1 + 0.16, "stonedark")
    # the gate: an arch with bars
    gx0, gx1, gtop = 1.12, 1.88, 1.05
    r = (gx1 - gx0) / 2
    gate = f"{P(gx0, y0)} -- {P(gx0, gtop)} arc[start angle=180, end angle=0, radius={f(r)}] -- {P(gx1, y0)} -- cycle"
    s += f"\\filldraw[inked,fill=ink!85!wooddark] {gate};\n"
    for k in range(1, 4):
        x = gx0 + (gx1 - gx0) * k / 4
        top = gtop + math.sqrt(max(0.0, r * r - (x - gx0 - r) ** 2))
        s += line((x, y0 + 0.02), (x, top - 0.02), lw=2.6, colour="ink")
        s += line((x, y0 + 0.02), (x, top - 0.02), lw=1.2, colour="metallight")
    s += line((gx0 + 0.04, 0.75), (gx1 - 0.04, 0.75), lw=2.6, colour="ink")
    s += line((gx0 + 0.04, 0.75), (gx1 - 0.04, 0.75), lw=1.2, colour="metallight")
    return s, (S, S)


def bricks(x0, x1, y0, rows, rh, bw):
    """Running bond: odd rows start half a brick to the left."""
    s = ""
    for r in range(rows):
        off = 0.0 if r % 2 == 0 else -bw / 2
        x = x0 + off
        while x < x1:
            fill = "brick" if (r + int((x - x0) / bw)) % 3 else "brickdark!60!brick"
            s += rrect(x + 0.025, y0 + r * rh + 0.025, x + bw - 0.025, y0 + (r + 1) * rh - 0.025, fill, 0.03, lw(1.1))
            x += bw
    return s


def clipped_rect(x0, y0, x1, y1, body):
    return f"\\begin{{scope}}\\clip {P(x0, y0)} rectangle {P(x1, y1)};\n{body}\\end{{scope}}\n" + f"\\draw[ink, line width=\\Ink, line join=round] {P(x0, y0)} rectangle {P(x1, y1)};\n"


def uplink():
    """A lattice radio mast with a puffy cloud behind it: the wider internet."""
    span(0.2, 2.8)
    s = ""
    # the cloud, behind everything
    s += cloud(1.35, 2.05, 0.82, "cream")
    s += shadow(0.6, 2.4, 0.18)
    # the mast: two legs narrowing to the tip, braced
    top = (1.5, 2.55)
    L0, R0 = (0.8, 0.2), (2.2, 0.2)
    L1, R1 = (1.38, 2.35), (1.62, 2.35)
    braces = 5
    for k in range(braces):
        t0, t1 = k / braces, (k + 1) / braces
        la = (L0[0] + (L1[0] - L0[0]) * t0, L0[1] + (L1[1] - L0[1]) * t0)
        ra = (R0[0] + (R1[0] - R0[0]) * t0, R0[1] + (R1[1] - R0[1]) * t0)
        lb = (L0[0] + (L1[0] - L0[0]) * t1, L0[1] + (L1[1] - L0[1]) * t1)
        rb = (R0[0] + (R1[0] - R0[0]) * t1, R0[1] + (R1[1] - R0[1]) * t1)
        s += line(la, rb, lw=1.3)
        s += line(ra, lb, lw=1.3)
        s += line(lb, rb, lw=1.5)
    s += tube([[L0, L1], [R0, R1]], 0.11, "red")
    s += tube([[(1.5, 2.3), top]], 0.08, "metaldark")
    # dishes on the mast
    s += ellipse(1.18, 1.55, 0.13, 0.24, "white", rot=0)
    s += ellipse(1.83, 1.2, 0.13, 0.24, "white", rot=0)
    # the beacon and its signal both ways
    s += circle(top[0], top[1] + 0.05, 0.12, "gold", lw(1.2))
    for r in (0.25, 0.42):
        s += f"\\draw[ink, line width=1.5pt, line cap=round] {P(top[0] + r * math.cos(math.radians(-35)), top[1] + 0.05 + r * math.sin(math.radians(-35)))} arc[start angle=-35, end angle=35, radius={f(r)}];\n"
        s += f"\\draw[ink, line width=1.5pt, line cap=round] {P(top[0] + r * math.cos(math.radians(145)), top[1] + 0.05 + r * math.sin(math.radians(145)))} arc[start angle=145, end angle=215, radius={f(r)}];\n"
    # a concrete foot
    s += rect(0.6, 0.12, 2.4, 0.3, "concrete", lw(1.4))
    return s, (S, S)


def toaster():
    """A smart toaster gone bad: one tiny antenna, one glowing sly eye."""
    span(0.3, 2.7)
    s = shadow(0.3, 2.7, 0.2)
    top = 1.95
    # two slices of toast poking up
    for x in (0.75, 1.6):
        s += rrect(x, top - 0.3, x + 0.65, top + 0.42, "woodlight", 0.14)
        s += rrect(x + 0.08, top - 0.3, x + 0.57, top + 0.34, "goldlight!60!woodlight", 0.1, lw(0.8))
    # the antenna, tiny, on the right shoulder
    s += tube([[(2.35, top - 0.05), (2.5, 2.6)]], 0.06, "metaldark")
    s += circle(2.5, 2.64, 0.1, "oipurple", lw(1.2))
    # the body
    s += rrect(0.3, 0.25, 2.7, top, "pcoral", 0.4)
    s += rrect(0.3, 0.25, 2.7, 0.55, "wallE", 0.2)
    for x in (0.75, 1.6):
        s += rrect(x - 0.03, top - 0.1, x + 0.68, top + 0.02, "ink", 0.04, lw(0.8))
    # the lever on the side
    s += rect(2.7, 1.05, 2.82, 1.55, "metaldark", lw(1.0))
    s += rrect(2.72, 1.35, 2.95, 1.5, "metallight", 0.04, lw(1.0))
    # the eye: a glow, a white, a pupil, and a slanted lid
    cx, cy = 1.5, 1.12
    s += fillo(f"{P(cx, cy)} circle (0.58)", "magenta", 0.25)
    s += circle(cx, cy, 0.4, "white", lw(1.6))
    s += circle(cx + 0.06, cy - 0.04, 0.2, "magenta", lw(1.2))
    s += circle(cx + 0.06, cy - 0.04, 0.08, "ink", lw(0.6))
    s += glint(cx + 0.12, cy + 0.02, 0.04)
    s += f"\\filldraw[inked,fill=wallE] {P(cx - 0.47, cy + 0.32)} -- {P(cx + 0.47, cy + 0.04)} -- {P(cx + 0.42, cy + 0.32)} arc[start angle=40, end angle=140, radius=0.55] -- cycle;\n"
    # a smirk
    s += curve([(1.1, 0.68), (1.5, 0.66), (1.85, 0.78)], lw=1.6)
    return s, (S, S)


PICTURES = {f"icons/{k}": v for k, v in {
    "host": host, "printer": printer, "hub": hub, "switch": switch, "router": router,
    "server": server, "firewall": firewall, "uplink": uplink, "toaster": toaster,
}.items()}
