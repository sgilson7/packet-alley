"""Packet Alley: the title and the four chapter backdrops, drawn as TikZ
and compiled by tools/art.py (web/art/title.png, 20 x 11.25 cm;
web/art/backdrops/<name>.png, 16 x 9.6 cm; both at 150 dpi). Never
generated pixels.

The look is vagrancy/Sam_Comm_final.png's by way of Level Twenty's title: a
street of salmon, coral and pink buildings under a warm yellow sky with
outlined puffy clouds, air conditioners, pipes and antennas, and here many
cables strung between the buildings with packets riding them. Chum 3000,
the green mech cat, peeks over the far rooftops. The city pieces (windows,
building, vsign, water_tank, awning, litter, street, calm) are Level
Twenty's art_scenes.py helpers, copied.

Facts each picture must get right (what an image generator gets wrong),
and where this file enforces each:

- no words, letters or digits: the noodle shop's signs are a bowl, bars and
  blocks; tools/art.py's `check_no_text` refuses \\node and \\text.
- packets are envelopes in exactly four colours, teal, coral, gold and
  violet (PACKET_COLOURS, asserted against what the title uses), and every
  packet hangs from a cable: `packet_on` computes its position and tilt
  from the same sag formula that draws the cable (`sag_pt`), so a packet
  cannot float off its wire.
- every cable ends on something: each end is a building's wall or roof, or
  the router's box, or runs off the canvas edge (`check_cable_ends`
  asserts each end is inside a drawn building rectangle, on the router, or
  outside the canvas).
- the router on the rooftop is the router icon (art_icons.router) placed
  whole, so it cannot look one way on the map and another on the title.
- the noodle shop shows a steaming bowl on its sign, red paper lanterns and
  a short split curtain over the door.
- Chum 3000 is the head from Level Twenty's portrait (green riveted plates,
  tall ears, round segmented eyes, a blank forehead plate, orange mouth),
  drawn before the far buildings so that it peeks over their roofs.
- the title's upper-left corner (x < CALM_X, y > CALM_Y: 40% of the width,
  a third of the height) is only sky and clouds, because the page may lay
  a heading over it: everything after the sky is clipped out of it by
  `calm`.
- the backdrops sit behind a network diagram, so they are low contrast and
  calm in the middle: every backdrop is drawn in a softened ink (SOFT_INK)
  with fills mixed toward paper (`pale`), and nothing but ground or sky is
  drawn inside the middle rectangle MIDDLE (`check_middle` asserts every
  detail's anchor is outside it).
- each backdrop shows its chapter: "Frames on the Block" one plaza seen
  from above with roofs round the border and a street along the foot;
  "Routes Across Town" a grid of blocks with roads between them;
  "Handshakes" rooftops at the foot and edges with cables strung between
  them across the top; "Opening Night" a dusky violet and orange sky, pale
  in the middle, string lights across the top and the noodle shop's bowl
  sign at the right edge.
- colour-blind safe: no picture tells two things apart by red against
  green; the backdrops' middles stay light (checked by their mean
  luminance on a greyscale render) so icons and lines on top keep their
  contrast.
- repeatable: every random choice comes from a seeded random.Random.
"""
import math, random

from artlib import (P, f, pts, rect, rrect, circle, ellipse, poly, blob, line, curve, place,
                    clipped, glint, sky, cloud, tube)
import artlib
import art_icons

WALLS = ["wallA", "wallB", "wallC", "wallD", "wallE", "wallF"]
SIGNS = ["red", "teal", "gold", "magenta", "violet", "orange", "pink", "cyan"]
PACKET_COLOURS = ["pteal", "pcoral", "pgold", "pviolet"]


def lw(w):
    return f",line width={f(w)}pt"


def fillo(path, colour, op):
    return f"\\fill[{colour}, opacity={f(op)}] {path};\n"


# --------------------------------------------------------------------------
# city pieces, from Level Twenty's art_scenes.py
# --------------------------------------------------------------------------

def windows(x0, x1, y0, y1, rng, col, pane=(0.42, 0.5), gap=(0.28, 0.32), lit=0.35):
    s = ""
    y = y1 - gap[1] - pane[1]
    while y > y0 + 0.2:
        x = x0 + gap[0]
        while x + pane[0] < x1 - 0.1:
            r = rng.random()
            if r < lit:
                fill = "window"
            elif r < lit + 0.08:
                fill = "teal!50!window"
            else:
                fill = f"{col}!60!windark"
            s += rect(x, y, x + pane[0], y + pane[1], fill, lw(0.6))
            if fill == "window" and rng.random() < 0.3:
                s += line((x + pane[0] * 0.5, y), (x + pane[0] * 0.5, y + pane[1]), lw=0.5)
            x += pane[0] + gap[0]
        y -= pane[1] + gap[1]
    return s


def ac_unit(x, y, s_=1.0):
    o = rect(x, y, x + 0.75 * s_, y + 0.5 * s_, "metallight", lw(0.8))
    o += circle(x + 0.25 * s_, y + 0.25 * s_, 0.17 * s_, "metal", lw(0.6))
    for k in range(3):
        o += line((x + 0.5 * s_, y + (0.12 + k * 0.13) * s_), (x + 0.7 * s_, y + (0.12 + k * 0.13) * s_), lw=0.5)
    return o


def vsign(x, y, w, h, rng, c=None):
    """A vertical shop sign: a coloured board with blocks for lettering."""
    c = c or rng.choice(SIGNS)
    o = rect(x - 0.05, y + h, x + w + 0.05, y + h + 0.12, "metaldark", lw(0.6))
    o += rect(x, y, x + w, y + h, c)
    o += rect(x + 0.08, y + 0.08, x + w - 0.08, y + h - 0.08, f"{c}!55!white", lw(0.6))
    yy = y + h - 0.25
    while yy - 0.35 > y + 0.1:
        o += rect(x + w * 0.25, yy - 0.3, x + w * 0.75, yy, c, lw(0.5))
        yy -= 0.5
    return o


BUILT = []  # (x0, y0, x1, y1) of every building drawn, for check_cable_ends


def building(x0, x1, top, rng, col=None, base=0.0, signs=True, roof=True, win=True):
    col = col or rng.choice(WALLS)
    BUILT.append((x0, base - 0.5, x1, top + 0.18))
    s = rect(x0, base - 0.5, x1, top, col)
    s += rect(x0 - 0.1, top, x1 + 0.1, top + 0.18, f"{col}!75!ink")
    if win:
        s += windows(x0, x1, base + 0.4, top, rng, col)
    if signs and rng.random() < 0.6:
        sx = rng.choice([x0 + 0.1, x1 - 0.65])
        sy = rng.uniform(base + 1.2, max(base + 1.3, top - 3.0))
        s += vsign(sx, sy, 0.55, min(2.6, top - sy - 0.2), rng)
    if rng.random() < 0.7:
        s += ac_unit(rng.uniform(x0 + 0.1, max(x0 + 0.2, x1 - 0.9)), rng.uniform(base + 0.8, max(base + 0.9, top - 1.0)), 0.9)
    if rng.random() < 0.5:  # a pipe down the front
        px = rng.uniform(x0 + 0.2, x1 - 0.2)
        s += tube([[(px, top), (px, base + 0.3)]], 0.12, "metallight")
    if roof:
        for k in range(rng.randint(0, 2)):
            ax = rng.uniform(x0 + 0.2, x1 - 0.2)
            ah = rng.uniform(0.4, 0.9)
            s += line((ax, top + 0.18), (ax, top + 0.18 + ah), lw=0.9)
            s += line((ax - 0.2, top + 0.18 + ah * 0.7), (ax + 0.2, top + 0.18 + ah * 0.7), lw=0.8)
        if rng.random() < 0.4:
            tx = rng.uniform(x0 + 0.3, max(x0 + 0.4, x1 - 1.2))
            s += water_tank(tx, top + 0.18, 0.7)
    return s


def water_tank(x, y, s_):
    o = ""
    for dx in (0.15, 0.85):
        o += line((x + dx * s_, y), (x + dx * s_, y + 0.6 * s_), lw=1.0)
    o += line((x + 0.15 * s_, y + 0.3 * s_), (x + 0.85 * s_, y + 0.3 * s_), lw=0.7)
    o += rrect(x, y + 0.6 * s_, x + s_, y + 1.5 * s_, "wood", 0.08 * s_)
    for k in range(3):
        yy = y + (0.8 + k * 0.25) * s_
        o += line((x, yy), (x + s_, yy), lw=0.6)
    o += poly([(x - 0.05 * s_, y + 1.5 * s_), (x + 0.5 * s_, y + 1.85 * s_), (x + 1.05 * s_, y + 1.5 * s_)], "wooddark")
    return o


def string_lights(a, b, sag, n, rng):
    s = cable(a, b, sag, lw=0.8)
    for k in range(1, n):
        x, y = sag_pt(a, b, sag, k / n)
        c = rng.choice(["gold", "pink", "cyan", "goldlight"])
        s += fillo(f"{P(x, y - 0.12)} circle (0.2)", c, 0.3)
        s += circle(x, y - 0.12, 0.08, c, lw(0.5))
    return s


def awning(x0, x1, y, col, stripe="white", drop=0.7):
    s = poly([(x0, y), (x1, y), (x1 + 0.25, y - drop), (x0 - 0.25, y - drop)], col)
    n = max(2, int((x1 - x0) / 0.45))
    for k in range(0, n, 2):
        a = x0 + (x1 - x0) * k / n
        b = x0 + (x1 - x0) * (k + 1) / n
        ta = (a - (x0 + x1) / 2) / ((x1 - x0) / 2) * 0.25
        tb = (b - (x0 + x1) / 2) / ((x1 - x0) / 2) * 0.25
        s += poly([(a, y), (b, y), (b + tb, y - drop), (a + ta, y - drop)], stripe, lw(0.6))
    for k in range(n + 1):
        x = x0 - 0.25 + (x1 - x0 + 0.5) * k / n
        s += f"\\filldraw[inked,fill={col}] {P(x - 0.2, y - drop)} arc[start angle=180,end angle=360,radius=0.2] -- cycle;\n"
    return s


def crate(x, y, w, h, fill="wood"):
    s = rect(x, y, x + w, y + h, fill)
    s += line((x, y), (x + w, y + h), lw=0.7)
    s += rect(x, y + h * 0.42, x + w, y + h * 0.58, f"{fill}!80!ink", lw(0.6))
    return s


def can(x, y, r=0.15, fill="sagelight", rot_=0):
    return f"\\filldraw[inked,fill={fill},rotate around={{{f(rot_)}:{P(x, y)}}}] {P(x - r, y - r * 1.5)} rectangle {P(x + r, y + r * 1.5)};\n"


def litter(rng, W, y0, y1, n, avoid=None):
    s = ""
    for k in range(n):
        x, y = rng.uniform(0, W), rng.uniform(y0, y1)
        if avoid and avoid[0] < x < avoid[2] and avoid[1] < y < avoid[3]:
            continue
        r = rng.random()
        if r < 0.35:
            s += can(x, y, rng.uniform(0.09, 0.14), rng.choice(["sagelight", "red", "goldlight", "metallight"]), rng.uniform(-80, 80))
        elif r < 0.6:
            s += f"\\filldraw[inked,fill=cream,rotate around={{{f(rng.uniform(-30, 30))}:{P(x, y)}}}] {P(x - 0.2, y - 0.12)} rectangle {P(x + 0.2, y + 0.12)};\n"
        elif r < 0.8:
            s += crate(x, y, rng.uniform(0.4, 0.7), rng.uniform(0.3, 0.5), rng.choice(["wallA", "woodlight", "wallF"]))
        else:
            s += circle(x, y, 0.05, "ink!60!wood")
    return s


def street(W, gy, vx, col="concrete"):
    """A paved street seen from the middle of the road, slabs running away."""
    s = rect(-0.5, -0.5, W + 0.5, gy, col)
    s += line((-0.5, gy), (W + 0.5, gy), lw=artlib.CUR_INK)
    for k in range(-14, 15):
        x = vx + k * 1.6
        s += f"\\draw[{col}!70!ink, line width=0.6pt] {P(vx + k * 0.25, gy)} -- {P(x + k * 1.2, -0.5)};\n"
    y, step = gy, 0.25
    while y > -0.5:
        s += f"\\draw[{col}!70!ink, line width=0.6pt] {P(-0.5, y)} -- {P(W + 0.5, y)};\n"
        y -= step
        step *= 1.45
    return s


def calm(W, H, cx, cy, body):
    """Everything outside the corner x < cx, y > cy."""
    path = f"{P(cx, H + 1)} -- {P(W + 1, H + 1)} -- {P(W + 1, -1)} -- {P(-1, -1)} -- {P(-1, cy)} -- {P(cx, cy)} -- cycle"
    return clipped(path, body)


# --------------------------------------------------------------------------
# cables and packets
# --------------------------------------------------------------------------

def sag_pt(a, b, sag, t):
    """The point a fraction t along a cable from a to b that sags by sag
    at its middle (a parabola, close enough to a catenary)."""
    (ax, ay), (bx, by) = a, b
    return (ax + (bx - ax) * t, ay + (by - ay) * t - sag * 4 * t * (1 - t))


def sag_angle(a, b, sag, t):
    (ax, ay), (bx, by) = a, b
    return math.degrees(math.atan2((by - ay) - sag * 4 * (1 - 2 * t), bx - ax))


def cable(a, b, sag, lw=0.9, colour="ink"):
    return curve([sag_pt(a, b, sag, k / 12) for k in range(13)], lw=lw, colour=colour, tension=0.5)


def envelope_body(c):
    """A packet: an envelope hanging from a pulley at the origin."""
    s = line((0, 0), (0, -0.2), lw=1.0)
    s += rect(-0.42, -0.74, 0.42, -0.18, c)
    s += f"\\draw[ink, line width=0.9pt, line join=round] {P(-0.42, -0.18)} -- {P(0, -0.5)} -- {P(0.42, -0.18)};\n"
    s += circle(0, 0, 0.09, "metallight", lw(0.8))
    return s


def packet_on(a, b, sag, t, c, s_=0.8):
    assert c in PACKET_COLOURS, c
    x, y = sag_pt(a, b, sag, t)
    return place(x, y, s_, envelope_body(c), rot=sag_angle(a, b, sag, t) * 0.6)


def anchor(p):
    """The little insulator a cable is tied to."""
    return circle(p[0], p[1], 0.07, "metallight", lw(0.7))


def roof_at(x):
    """The roof line of the lowest-drawn building standing at x."""
    tops = [y1 - 0.18 for (x0, y0, x1, y1) in BUILT if x0 <= x <= x1]
    return max(tops)


def check_cable_ends(cables, W, H, extra=()):
    boxes = BUILT + list(extra)
    for a, b, _ in cables:
        for (x, y) in (a, b):
            off = x < 0 or x > W or y < 0 or y > H
            on = any(x0 - 0.15 <= x <= x1 + 0.15 and y0 - 0.15 <= y <= y1 + 0.15 for (x0, y0, x1, y1) in boxes)
            assert off or on, f"cable end {(x, y)} is tied to nothing"


# --------------------------------------------------------------------------
# Chum 3000 and the noodle shop
# --------------------------------------------------------------------------

def chum_head():
    """The mech cat's head from Level Twenty's portrait (art_units.chum),
    without its scrap of city: 6 cm wide, chin at y = 0.1."""
    g, gd, gl = "chum", "chumdark", "chumlight"
    s = poly([(1.05, 3.2), (0.6, 5.85), (2.35, 4.6)], g)
    s += poly([(1.25, 3.6), (0.95, 5.25), (2.0, 4.5)], gd)
    s += poly([(4.95, 3.2), (5.4, 5.85), (3.65, 4.6)], g)
    s += poly([(4.75, 3.6), (5.05, 5.25), (4.0, 4.5)], gd)
    s += f"\\filldraw[inked,fill={g}] {P(0.7, 2.9)} .. controls {P(0.6, 4.9)} and {P(5.4, 4.9)} .. {P(5.3, 2.9)} .. controls {P(5.2, 1.0)} and {P(4.0, 0.1)} .. {P(3.0, 0.1)} .. controls {P(2.0, 0.1)} and {P(0.8, 1.0)} .. cycle;\n"
    for (x, y, rx, ry) in ((1.3, 2.4, 0.35, 0.22), (4.6, 2.2, 0.3, 0.2), (2.2, 0.9, 0.3, 0.18), (4.0, 0.9, 0.25, 0.17), (3.9, 4.15, 0.3, 0.15)):
        s += ellipse(x, y, rx, ry, gl, extra=lw(0.7))
    s += curve([(0.9, 3.4), (2.0, 3.0), (3.0, 2.95), (4.0, 3.0), (5.1, 3.4)], lw=0.9)
    s += rrect(2.0, 3.55, 4.0, 4.2, gl, 0.1)
    for (x, y) in ((2.12, 3.67), (3.88, 3.67), (2.12, 4.08), (3.88, 4.08)):
        s += circle(x, y, 0.06, "metal", lw(0.5))
    s += rect(2.35, 3.8, 3.65, 3.95, "orange", lw(0.6))
    for ex in (1.85, 4.15):
        s += circle(ex, 2.45, 0.62, gl)
        s += circle(ex, 2.45, 0.42, "cyan!35!white")
        s += line((ex - 0.42, 2.45), (ex + 0.42, 2.45), lw=0.8)
        s += line((ex, 2.03), (ex, 2.87), lw=0.8)
        s += circle(ex - 0.1, 2.45, 0.14, "ink")  # glancing left, at the street
        s += glint(ex + 0.02, 2.6, 0.06)
    for k in range(3):
        s += line((2.75, 2.15 - k * 0.13), (3.25, 2.15 - k * 0.13), lw=1.2)
    s += f"\\filldraw[inked,fill=orange] {P(1.75, 1.35)} .. controls {P(2.4, 0.55)} and {P(3.6, 0.55)} .. {P(4.25, 1.35)} .. controls {P(3.6, 1.15)} and {P(2.4, 1.15)} .. cycle;\n"
    for k in range(5):
        x = 2.15 + k * 0.42
        s += poly([(x - 0.13, 1.22), (x, 0.98), (x + 0.13, 1.22)], "white")
    for k in range(14):
        a = math.radians(200 + k * (140 / 13))
        s += circle(3.0 - 2.05 * math.cos(a), 2.6 - 1.9 * math.sin(a), 0.06, "metal", lw(0.5))
    return s


def bowl(x, y, r, steam=True, steam_colour="white"):
    """A noodle bowl, rim centred at (x, y), r wide each side."""
    s = ""
    if steam:
        for dx, h in ((-0.45, 0.8), (0.0, 1.05), (0.45, 0.8)):
            xx = x + dx * r
            s += curve([(xx, y + 0.15 * r), (xx + 0.15 * r, y + h * 0.35 * r), (xx - 0.12 * r, y + h * 0.7 * r), (xx + 0.06 * r, y + h * r)], lw=1.6, colour=steam_colour)
    s += tube([[(x + 0.35 * r, y - 0.05 * r), (x + 1.05 * r, y + 0.75 * r)], [(x + 0.5 * r, y - 0.1 * r), (x + 1.2 * r, y + 0.6 * r)]], 0.07 * r * 1.4, "woodlight")
    s += f"\\filldraw[inked,fill=bowl] {P(x - r, y)} .. controls {P(x - 0.95 * r, y - 0.6 * r)} and {P(x - 0.5 * r, y - 0.75 * r)} .. {P(x, y - 0.75 * r)} .. controls {P(x + 0.5 * r, y - 0.75 * r)} and {P(x + 0.95 * r, y - 0.6 * r)} .. {P(x + r, y)} -- cycle;\n"
    s += f"\\fill[red] {P(x - 0.88 * r, y - 0.3 * r)} rectangle {P(x + 0.88 * r, y - 0.38 * r)};\n"
    s += ellipse(x, y, r, 0.17 * r, "broth")
    for k in range(4):
        xx = x - 0.55 * r + k * 0.33 * r
        s += curve([(xx, y - 0.04 * r), (xx + 0.09 * r, y + 0.07 * r), (xx + 0.18 * r, y - 0.04 * r)], lw=1.0, colour="noodle!70!wooddark")
    return s


def envelope_shape(c):
    """A loose envelope centred on the origin."""
    s = rect(-0.42, -0.28, 0.42, 0.28, c)
    s += f"\\draw[ink, line width=0.9pt, line join=round] {P(-0.42, 0.28)} -- {P(0, -0.04)} -- {P(0.42, 0.28)};\n"
    return s


def pigeon():
    s = ellipse(0, 0, 0.3, 0.2, "metallight")
    s += circle(0.25, 0.22, 0.13, "metallight", lw(0.7))
    s += poly([(0.37, 0.23), (0.5, 0.19), (0.37, 0.15)], "gold", lw(0.5))
    s += line((-0.05, -0.2), (-0.05, -0.32), lw=0.8)
    s += line((0.08, -0.2), (0.08, -0.32), lw=0.8)
    return s


def telegraph_pole(x, y0, top):
    s = tube([[(x, y0), (x, top)]], 0.24, "wood")
    for y, half in ((top - 0.35, 0.85), (top - 0.9, 0.85)):
        s += rect(x - half, y - 0.08, x + half, y + 0.08, "wooddark", lw(0.9))
        for dx in (-half + 0.1, half - 0.1):
            s += rect(x + dx - 0.05, y + 0.08, x + dx + 0.05, y + 0.22, "metallight", lw(0.6))
    s += rrect(x - 0.3, top - 2.0, x + 0.3, top - 1.5, "metal", 0.06)
    return s


def noodle_cart(x, y):
    """A street cart: a counter on wheels, a striped canopy, a steaming pot."""
    s = ""
    for dx in (0.15, 2.85):
        s += tube([[(x + dx, y + 1.3), (x + dx, y + 2.6)]], 0.1, "metaldark")
    s += awning(x - 0.1, x + 3.1, y + 2.85, "teal", drop=0.5)
    # the pot and its steam
    for dx in (0.9, 1.3, 1.7):
        s += curve([(x + dx, y + 1.85), (x + dx + 0.1, y + 2.05), (x + dx - 0.08, y + 2.25)], lw=1.4, colour="white")
    s += rrect(x + 0.7, y + 1.3, x + 1.9, y + 1.8, "metallight", 0.08)
    s += rect(x + 0.6, y + 1.75, x + 2.0, y + 1.88, "metal", lw(0.8))
    s += bowl(x + 2.4, y + 1.48, 0.25, steam=False)
    s += rect(x, y + 0.45, x + 3.0, y + 1.3, "wallC")
    s += rect(x + 0.2, y + 0.6, x + 2.8, y + 1.15, "wallC!70!white", lw(0.7))
    s += lantern(x + 2.75, y + 2.0, 0.55)
    for dx in (0.6, 2.4):
        s += circle(x + dx, y + 0.35, 0.33, "metaldark")
        s += circle(x + dx, y + 0.35, 0.12, "metallight", lw(0.6))
    return s


def lantern(x, y, s_=1.0):
    o = line((x, y + 0.45 * s_), (x, y + 0.75 * s_), lw=0.9)
    o += rect(x - 0.12 * s_, y + 0.38 * s_, x + 0.12 * s_, y + 0.48 * s_, "ink")
    o += ellipse(x, y, 0.3 * s_, 0.42 * s_, "red")
    o += ellipse(x, y, 0.13 * s_, 0.42 * s_, "redlight", extra=lw(0.6))
    o += rect(x - 0.12 * s_, y - 0.48 * s_, x + 0.12 * s_, y - 0.38 * s_, "ink")
    o += line((x, y - 0.48 * s_), (x, y - 0.7 * s_), lw=0.8, colour="gold")
    return o


def noodle_shop(x0, x1, top, base, rng):
    """A coral shopfront with a bowl sign, lanterns and a split curtain."""
    col = "wallC"
    s = building(x0, x1, top, rng, col=col, base=base, signs=False, roof=False, win=False)
    s += windows(x0, x1, base + 2.6, top, rng, col)
    # the shop floor: a counter window and a door with a short curtain
    s += rect(x0 + 0.25, base, x0 + 1.6, base + 1.6, "window!70!cream")
    s += rect(x0 + 0.25, base + 0.45, x0 + 1.6, base + 0.6, "wood", lw(0.8))
    s += bowl(x0 + 0.7, base + 0.75, 0.22, steam=False)
    s += bowl(x0 + 1.2, base + 0.75, 0.22, steam=False)
    dx0, dx1 = x0 + 1.9, x1 - 0.35
    s += rect(dx0, base - 0.3, dx1, base + 1.9, "ink!80!wooddark")
    n = 3
    for k in range(n):
        a = dx0 + (dx1 - dx0) * k / n
        b = dx0 + (dx1 - dx0) * (k + 1) / n
        s += rect(a + 0.02, base + 1.15, b - 0.02, base + 1.9, "teal")
        s += circle((a + b) / 2, base + 1.55, 0.09, "white", lw(0.6))
    s += awning(x0 + 0.1, x1 - 0.1, base + 2.45, "red")
    # the bowl sign: a round board on a bracket, steaming
    sx, sy = x0 - 0.75, base + 3.6
    s += line((x0, sy + 0.95), (sx, sy + 0.95), lw=2.2)
    s += line((sx, sy + 0.95), (sx, sy + 0.75), lw=1.2)
    s += circle(sx, sy, 0.78, "gold")
    s += circle(sx, sy, 0.62, "cream", lw(0.9))
    s += bowl(sx, sy + 0.02, 0.42, steam_colour="ink!60!cream")
    # a vertical sign of blocks above the awning, and lanterns either side
    s += vsign(x1 - 0.75, base + 2.9, 0.6, 2.4, rng, "teal")
    s += lantern(x0 + 0.35, base + 1.6, 0.8)
    s += lantern(x1 - 0.2, base + 1.6, 0.8)
    return s


# --------------------------------------------------------------------------
# the title
# --------------------------------------------------------------------------

def title():
    W, H = 20.0, 11.25
    CALM_X, CALM_Y = 8.0, 7.6
    rng = random.Random(2027)
    BUILT.clear()
    base = sky(W, H)
    for (x, y, sc) in ((1.6, 10.3, 0.75), (5.8, 9.2, 0.55), (12.8, 10.4, 0.8), (18.2, 10.6, 0.6)):
        base += cloud(x, y, sc)
    s = ""
    gy = 3.4
    # Chum 3000 peeks over the far roofs, drawn first so they cover his chin
    s += place(7.75, 3.6, 0.85, chum_head())
    # far buildings in the middle of the street, low enough to show his eyes
    x = 7.3
    while x < 13.4:
        w = rng.uniform(1.2, 1.8)
        s += building(x, x + w, rng.uniform(4.9, 5.3), rng, col=rng.choice(["wallD", "wallB", "stone"]), base=gy, signs=False, roof=False)
        x += w + 0.08
    # near buildings: low on the left, under the calm corner; tall on the right
    s += building(-0.5, 2.4, 7.1, rng, col="wallA", base=gy - 0.3, roof=False)
    s += building(2.3, 4.7, 6.3, rng, col="wallF", base=gy - 0.3, roof=False)
    s += building(4.6, 7.5, 5.0, rng, col="wallE", base=gy - 0.3, signs=False, roof=False)
    s += building(15.6, 18.0, 9.2, rng, col="wallB", base=gy - 0.3)
    s += building(17.9, 20.5, 10.6, rng, col="wallA", base=gy - 0.3)
    s += water_tank(18.6, 10.78, 0.9)
    s += noodle_shop(13.2, 15.7, 7.4, gy - 0.3, rng)
    s += water_tank(14.2, 7.58, 0.75)
    # the router on the low roof, where the cables meet
    rs = 0.6
    rx, ry = 5.3, 5.18 - 0.2 * rs
    s += place(rx, ry, rs, art_icons.router()[0])
    router_box = (rx + 0.2 * rs, ry + 0.2 * rs, rx + 2.8 * rs, ry + 1.55 * rs)
    R = (rx + 2.7 * rs, ry + 1.1 * rs)
    Rlow = (rx + 2.7 * rs, ry + 0.5 * rs)
    Rl = (rx + 0.3 * rs, ry + 1.1 * rs)
    # a telegraph pole at the front left, its arms on the street side
    pole_x, pole_top = 1.7, 7.35
    pole_box = (pole_x - 0.9, 0.6, pole_x + 0.9, pole_top)
    A1, A2 = (pole_x + 0.75, 7.0), (pole_x - 0.75, 6.45)
    cables = [
        (R, (15.6, 8.3), 0.9),
        (R, (13.2, 6.6), 0.45),
        (Rlow, (13.2, 5.6), 0.35),
        (A1, Rl, 0.35),
        (A2, (-0.6, 6.2), 0.15),
        ((pole_x + 0.75, 6.45), (4.7, 5.6), 0.2),
        ((15.7, 6.9), (17.9, 8.0), 0.3),
        ((15.7, 8.8), (17.9, 10.1), 0.25),
        ((18.0, 9.5), (20.6, 8.6), 0.3),
        ((13.3, 7.1), (15.6, 7.5), 0.2),
        (A1, (15.6, 8.7), 1.35),
    ]
    check_cable_ends(cables, W, H, [router_box, pole_box])
    wires = ""
    for a, b, sag in cables:
        wires += cable(a, b, sag)
        for p in (a, b):
            wires += anchor(p)
    wires += string_lights((7.5, 5.0), (13.2, 5.8), 0.5, 10, rng)
    used = [(0, 0.28, "pteal"), (0, 0.66, "pcoral"), (1, 0.55, "pgold"), (2, 0.5, "pviolet"),
            (3, 0.5, "pteal"), (10, 0.2, "pgold"), (10, 0.82, "pviolet"), (6, 0.5, "pteal"),
            (8, 0.45, "pcoral"), (7, 0.5, "pgold"), (5, 0.5, "pcoral")]
    assert sorted({c for _, _, c in used}) == sorted(PACKET_COLOURS)
    for k, t, c in used:
        a, b, sag = cables[k]
        wires += packet_on(a, b, sag, t, c)
    # the street, then the pole standing in it
    st = street(W, gy, 10.4)
    st += litter(rng, W, 0.2, gy - 0.4, 18, avoid=(14.8, 0.0, 19.6, 2.6))
    st += ellipse(10.2, 0.9, 0.9, 0.28, "metal") + ellipse(10.2, 0.9, 0.65, 0.18, "metaldark", extra=lw(0.7))
    st += telegraph_pole(pole_x, 0.6, pole_top)
    # a crate of envelopes waiting by the pole
    st += crate(2.6, 0.6, 1.3, 0.8, "woodlight")
    for k, c in enumerate(PACKET_COLOURS):
        st += place(2.75 + k * 0.3, 1.55, 0.55, envelope_shape(c), rot=(-12 + k * 9))
    # the noodle cart in front of the shop, and two pigeons
    st += noodle_cart(15.3, 0.5)
    for (px, py, fl) in ((11.6, 1.2, False), (12.3, 1.0, True)):
        st += place(px, py, 1.0, pigeon(), flip=fl)
    assert CALM_X >= 0.4 * W and H - CALM_Y >= 0.3 * H
    return base + calm(W, H, CALM_X, CALM_Y, s + st + wires), (W, H)


# --------------------------------------------------------------------------
# the four backdrops: pale, soft-inked, calm in the middle
# --------------------------------------------------------------------------

BW, BH = 16.0, 9.6
MIDDLE = (3.0, 2.2, 13.0, 7.4)  # nothing but ground or sky inside this
SOFT_INK = "8C7A82"  # the plum ink mixed 45% toward paper


def soft():
    return f"\\definecolor{{ink}}{{HTML}}{{{SOFT_INK}}}\n"


def pale(c, k=45):
    return f"{c}!{k}!paper"


def check_middle(anchors):
    x0, y0, x1, y1 = MIDDLE
    for (x, y) in anchors:
        assert not (x0 < x < x1 and y0 < y < y1), f"detail at {(x, y)} is in the calm middle"


def roof(x0, y0, x1, y1, col, rng, anchors):
    """A flat roof seen from above: a parapet, and roof clutter."""
    anchors.append(((x0 + x1) / 2, (y0 + y1) / 2))
    s = rect(x0, y0, x1, y1, pale(col, 55))
    s += rect(x0 + 0.15, y0 + 0.15, x1 - 0.15, y1 - 0.15, pale(col, 30), lw(0.6))
    for k in range(rng.randint(1, 3)):
        w, h = rng.uniform(0.4, 0.8), rng.uniform(0.3, 0.55)
        if x1 - x0 - 0.4 - w <= 0 or y1 - y0 - 0.4 - h <= 0:
            continue
        x = rng.uniform(x0 + 0.25, x1 - 0.25 - w)
        y = rng.uniform(y0 + 0.25, y1 - 0.25 - h)
        r = rng.random()
        if r < 0.45:
            s += rect(x, y, x + w, y + h, pale("metallight", 70), lw(0.6))
            s += circle(x + h / 2, y + h / 2, h * 0.32, pale("metal", 60), lw(0.5))
        elif r < 0.75:
            s += circle(x + w / 2, y + w / 2 * 0.8, w * 0.45, pale("wood", 55), lw(0.7))
            s += circle(x + w / 2, y + w / 2 * 0.8, w * 0.3, pale("wood", 35), lw(0.5))
        else:
            s += rect(x, y, x + w, y + h, pale("cyan!30!white", 70), lw(0.6))
            s += line((x, y), (x + w, y + h), lw=0.5)
    return s


def tree_top(x, y, r, anchors, fill="greenC"):
    anchors.append((x, y))
    s = ""
    for k in range(6):
        a = 2 * math.pi * k / 6
        s += circle(x + r * 0.45 * math.cos(a), y + r * 0.45 * math.sin(a), r * 0.6, pale(fill, 55))
    s += f"\\fill[{pale(fill, 55)}] {P(x, y)} circle ({f(r * 0.6)});\n"
    s += circle(x - r * 0.15, y + r * 0.15, r * 0.22, pale("greenA", 45), lw(0.5))
    return s


def block():
    """Frames on the Block: one plaza from above, roofs round the border,
    a street along the foot."""
    rng = random.Random(41)
    anchors = []
    s = soft()
    s += f"\\fill[paper] (0,0) rectangle {P(BW, BH)};\n"
    # the plaza's slabs, faint
    for k in range(1, 21):
        s += f"\\draw[concrete!35!paper, line width=0.5pt] {P(k * 0.8, 1.6)} -- {P(k * 0.8, 8.2)};\n"
    for k in range(9):
        y = 1.6 + k * 0.8
        s += f"\\draw[concrete!35!paper, line width=0.5pt] {P(0, y)} -- {P(BW, y)};\n"
    # the street along the foot, with a crossing at each end
    s += rect(-0.3, -0.3, BW + 0.3, 1.25, pale("asphalt", 35))
    s += rect(-0.3, 1.25, BW + 0.3, 1.55, pale("kerb", 55))
    for k in range(10):
        s += rect(1.0 + k * 1.5, 0.55, 1.75 + k * 1.5, 0.65, pale("roadline", 70), lw(0.4))
    for x0 in (13.0,):
        for k in range(5):
            s += rect(x0 + k * 0.4, 0.0, x0 + k * 0.4 + 0.22, 1.2, "paper", lw(0.5))
    anchors += [(1.0, 0.6), (13.8, 0.6)]
    # roofs along the top and down both sides
    x = -0.3
    while x < BW:
        w = rng.uniform(1.6, 2.6)
        s += roof(x, rng.uniform(8.2, 8.5), x + w, BH + 0.3, rng.choice(WALLS), rng, anchors)
        x += w + 0.05
    for (xa, xb) in ((-0.3, 1.5), (14.5, BW + 0.3)):
        y = 1.7
        while y < 8.4:
            h = rng.uniform(1.3, 2.2)
            s += roof(xa, y, xb + rng.uniform(-0.15, 0.15), min(y + h, 8.5), rng.choice(WALLS), rng, anchors)
            y += h + 0.05
    # trees, benches and planters just inside the border
    for (x, y) in ((2.3, 7.5), (13.7, 7.5), (2.3, 2.3), (13.7, 2.3), (5.2, 7.75), (10.8, 7.75)):
        s += tree_top(x, y, 0.5, anchors)
    for (x, y) in ((7.0, 7.75), (8.6, 7.75)):
        anchors.append((x, y))
        s += rrect(x, y - 0.12, x + 0.9, y + 0.12, pale("wood", 55), 0.05, lw(0.7))
    for (x, y) in ((4.0, 1.95), (11.4, 1.95)):
        anchors.append((x, y))
        s += rect(x, y - 0.18, x + 0.6, y + 0.18, pale("woodlight", 55), lw(0.7))
        for k in range(3):
            s += circle(x + 0.12 + k * 0.18, y, 0.09, pale(rng.choice(["pink", "gold", "red"]), 55), lw(0.4))
    check_middle(anchors)
    return s, (BW, BH)


def town():
    """Routes Across Town: a grid of blocks with roads between them."""
    rng = random.Random(42)
    anchors = []
    s = soft()
    s += f"\\fill[{pale('asphalt', 25)}] (0,0) rectangle {P(BW, BH)};\n"
    xs = [(-0.5, 4.6), (5.6, 10.4), (11.4, BW + 0.5)]
    ys = [(-0.5, 2.6), (3.6, 6.0), (7.0, BH + 0.5)]
    # road centre lines, dashed and faint
    for x in (5.1, 10.9):
        s += f"\\draw[{pale('roadline', 80)}, line width=1.2pt, dash pattern=on 6pt off 6pt] {P(x, -0.3)} -- {P(x, BH + 0.3)};\n"
    for y in (3.1, 6.5):
        s += f"\\draw[{pale('roadline', 80)}, line width=1.2pt, dash pattern=on 6pt off 6pt] {P(-0.3, y)} -- {P(BW + 0.3, y)};\n"
    tints = ["sage", "wallD", "stone", "wallF", "greenC"]
    for i, (x0, x1) in enumerate(xs):
        for j, (y0, y1) in enumerate(ys):
            middle = i == 1 and j == 1
            col = "greenC" if middle else tints[(i * 3 + j) % len(tints)]
            s += rrect(x0, y0, x1, y1, pale("kerb", 50), 0.3)
            s += rrect(x0 + 0.18, y0 + 0.18, x1 - 0.18, y1 - 0.18, pale(col, 28), 0.2, lw(0.6))
            if middle:
                continue  # the middle block is an empty green
            # roofs round the block's outer side only
            if j == 2:
                x = x0 + 0.3
                while x < x1 - 0.8:
                    w = min(rng.uniform(1.0, 1.7), x1 - 0.3 - x)
                    s += roof(x, max(y0 + 0.3, 8.1), x + w, y1 - 0.3, rng.choice(WALLS), rng, anchors)
                    x += w + 0.08
            if j == 0:
                x = x0 + 0.3
                while x < x1 - 0.8:
                    w = min(rng.uniform(1.0, 1.7), x1 - 0.3 - x)
                    s += roof(x, y0 + 0.3, x + w, min(y1 - 0.3, 1.5), rng.choice(WALLS), rng, anchors)
                    x += w + 0.08
            if j == 1 and i in (0, 2):
                xa, xb = (x0 + 0.3, 2.2) if i == 0 else (13.8, x1 - 0.3)
                y = y0 + 0.3
                while y < y1 - 0.6:
                    h = min(rng.uniform(0.8, 1.2), y1 - 0.3 - y)
                    s += roof(xa, y, xb, y + h, rng.choice(WALLS), rng, anchors)
                    y += h + 0.08
    # trees at the outer corners of the middle green, and along the edges
    for (x, y) in ((1.4, 8.4), (14.7, 1.0)):
        s += tree_top(x, y, 0.45, anchors)
    # crossings at the four inner junctions, at their outer side only
    for (x, y) in ((5.1, 2.0), (10.9, 2.0), (5.1, 7.6), (10.9, 7.6)):
        anchors.append((x, y))
        for k in range(4):
            xx = x - 0.4 + k * 0.24
            s += rect(xx, y - 0.0 - 0.6 if y < 4 else y + 0.0, xx + 0.13, y if y < 4 else y + 0.6, "paper", lw(0.4))
    check_middle(anchors)
    return s, (BW, BH)


def wires_backdrop():
    """Handshakes: rooftops at the foot and edges, cables strung across
    the top between the two tall buildings."""
    rng = random.Random(43)
    BUILT.clear()
    anchors = []
    s = soft()
    s += sky(BW, BH, "paper", pale("skyhi", 60), pale("sky", 50), a=0.85, b=0.22)
    for (x, y, sc) in ((1.8, 9.0, 0.5), (14.0, 9.1, 0.45)):
        anchors.append((x, y))
        s += cloud(x, y, sc, pale("cloud", 60))
    # the low skyline along the foot
    x = -0.3
    while x < BW:
        w = rng.uniform(1.1, 1.8)
        top = rng.uniform(1.2, 2.0)
        anchors.append((x + w / 2, top))
        s += building(x, x + w, top, rng, col=pale(rng.choice(WALLS), 50), base=0.3, signs=False, roof=rng.random() < 0.5, win=True)
        x += w + rng.uniform(0.0, 0.2)
    # the two tall buildings at the edges
    s += building(-0.5, 2.0, 7.4, rng, col=pale("wallA", 50), base=0.3, signs=False, roof=False)
    s += building(14.0, 16.5, 7.8, rng, col=pale("wallC", 50), base=0.3, signs=False, roof=False)
    s += water_tank(0.3, 7.58, 0.9)
    anchors += [(0.75, 4.0), (15.2, 4.0)]
    # a mast on each tall roof: the two ends of the handshake
    for (mx, my) in ((1.4, 7.58), (14.9, 7.98)):
        s += line((mx, my), (mx, my + 1.0), lw=1.6)
        s += line((mx - 0.25, my + 0.7), (mx + 0.25, my + 0.7), lw=1.1)
        s += circle(mx, my + 1.05, 0.09, pale("red", 60), lw(0.7))
        BUILT.append((mx - 0.3, my, mx + 0.3, my + 1.1))
    cables = [
        ((1.4, 8.3), (14.9, 8.7), 0.9),
        ((2.0, 7.0), (14.0, 7.3), 0.7),
        ((-0.5, 6.0), (0.0, 6.0), 0.0),
        ((2.0, 6.0), (2.9, roof_at(2.9)), 0.25),
        ((14.0, 6.2), (13.1, roof_at(13.1)), 0.25),
        ((4.0, roof_at(4.0)), (6.6, roof_at(6.6)), 0.3),
        ((7.4, roof_at(7.4)), (9.4, roof_at(9.4)), 0.25),
        ((10.2, roof_at(10.2)), (12.4, roof_at(12.4)), 0.3),
        ((16.5, 5.0), (16.0, 5.0), 0.0),
    ]
    cables = [c for c in cables if c[2] > 0]
    check_cable_ends(cables, BW, BH)
    for a, b, sag in cables:
        s += cable(a, b, sag, lw=1.0)
        s += anchor(a) + anchor(b)
    # birds on the top cable near the ends, packets near the masts
    for t in (0.12, 0.17, 0.85):
        x, y = sag_pt(*cables[0][:2], cables[0][2], t)
        anchors.append((x, y))
        s += ellipse(x, y + 0.14, 0.18, 0.13, pale("metallight", 70), extra=lw(0.7))
        s += circle(x + 0.14, y + 0.27, 0.08, pale("metallight", 70), lw(0.6))
    for (k, t, c) in ((1, 0.06, "pteal"), (1, 0.94, "pcoral")):
        a, b, sag = cables[k]
        anchors.append(sag_pt(a, b, sag, t))
        x, y = sag_pt(a, b, sag, t)
        s += place(x, y, 0.65, envelope_body(pale(c, 60)), rot=sag_angle(a, b, sag, t) * 0.6)
    # wires drop into the middle only as the two long top cables, which sag
    # no lower than MIDDLE's top edge
    for a, b, sag in cables[:2]:
        assert min(sag_pt(a, b, sag, k / 20)[1] for k in range(21)) >= MIDDLE[3] - 1.2
    check_middle(anchors)
    return s, (BW, BH)


def night():
    """Opening Night: a dusky evening street, string lights across the top,
    a noodle shop sign at the edge, pale in the middle."""
    rng = random.Random(44)
    BUILT.clear()
    anchors = []
    s = soft()
    # violet at the top, a pale lilac middle, an orange glow over the roofs
    s += sky(BW, BH, pale("dusk", 75), "duskhi!70!paper", pale("dusklow", 65), a=0.82, b=0.3)
    s += f"\\fill[{pale('dusklow', 40)}] (0,{f(BH * 0.3)}) rectangle {P(BW, BH * 0.38)};\n"
    # a moon and a few stars, high and at the sides
    s += circle(14.6, 8.9, 0.42, "cream")
    s += circle(14.78, 9.02, 0.36, pale("dusk", 75), ",draw=none")
    anchors.append((14.6, 8.9))
    for (x, y) in ((2.6, 9.2), (4.1, 8.8), (6.0, 9.3), (9.8, 9.25), (11.7, 8.85), (12.9, 9.35)):
        anchors.append((x, y))
        s += circle(x, y, 0.05, "cream", lw(0.5))
    # the low skyline along the foot, windows lit warm
    x = -0.3
    while x < BW:
        w = rng.uniform(1.1, 1.8)
        top = rng.uniform(1.2, 2.0)
        anchors.append((x + w / 2, top))
        s += building(x, x + w, top, rng, col=pale(rng.choice(["duskwall", "wallF", "violet!40!wallC"]), 55), base=0.3, signs=False, roof=False)
        x += w + rng.uniform(0.0, 0.2)
    # the tall buildings at the edges; the noodle shop's on the right
    s += building(-0.5, 2.0, 7.2, rng, col=pale("duskwall", 60), base=0.3, signs=False, roof=False)
    s += building(14.0, 16.5, 7.6, rng, col=pale("wallC", 55), base=0.3, signs=False, roof=False)
    anchors += [(0.75, 4.0), (15.2, 4.0)]
    # the noodle shop's signs: a round bowl sign on a bracket, a vertical
    # board of blocks, and lanterns; all of it at the right edge
    sx, sy = 13.3, 5.3
    anchors.append((sx, sy))
    s += line((14.0, sy + 0.95), (sx, sy + 0.95), lw=2.0)
    s += line((sx, sy + 0.95), (sx, sy + 0.75), lw=1.1)
    s += fillo(f"{P(sx, sy)} circle (1.05)", "gold", 0.18)
    s += circle(sx, sy, 0.75, pale("gold", 70))
    s += circle(sx, sy, 0.6, "cream", lw(0.8))
    s += bowl(sx, sy + 0.02, 0.4, steam_colour="ink!50!cream")
    s += vsign(15.0, 3.4, 0.7, 3.4, rng, pale("teal", 70))
    s += awning(14.1, 16.3, 2.6, pale("red", 60), drop=0.6)
    for x in (14.3, 16.0):
        s += lantern(x, 1.75, 0.8)
    anchors += [(15.35, 5.0), (15.2, 2.3)]
    # string lights between the tall buildings, sagging no lower than the
    # top of the calm middle
    strands = [((2.0, 8.6), (14.0, 8.9), 0.8, 18), ((2.0, 7.8), (14.0, 7.6), 1.0, 16)]
    for a, b, sag, n in strands:
        assert min(sag_pt(a, b, sag, k / 20)[1] for k in range(21)) >= MIDDLE[3] - 1.0
        s += string_lights(a, b, sag, n, rng)
        s += anchor(a) + anchor(b)
    check_middle(anchors)
    return s, (BW, BH)


PICTURES = {
    "title": title,
    "backdrops/block": block,
    "backdrops/town": town,
    "backdrops/wires": wires_backdrop,
    "backdrops/night": night,
}
