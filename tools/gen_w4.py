"""Generates World 4 (Rainward 13: rain-soaked abandoned city ward at night) art as .px sources.

    python3 tools/gen_w4.py

Writes art/src/w4/*.px and art/src/icons/relic_w4.px. Deterministic (seeded).
"""
import math
import pathlib
import random
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_stage  # noqa: E402
from gen_stage import Canvas  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent.parent / "art/src/w4"
gen_stage.OUT = OUT


# ── helpers ──────────────────────────────────────────────────────────────

def poly(c, pts, col):
    for y in range(c.h):
        for x in range(c.w):
            px, py, inside = x + 0.5, y + 0.5, False
            for i in range(len(pts)):
                (x1, y1), (x2, y2) = pts[i], pts[i - 1]
                if (y1 > py) != (y2 > py) and px < (x2 - x1) * (py - y1) / (y2 - y1) + x1:
                    inside = not inside
            if inside:
                c.g[y][x] = col


def line(c, x0, y0, x1, y1, col):
    steps = int(max(abs(x1 - x0), abs(y1 - y0)) * 2) + 1
    for i in range(steps + 1):
        t = i / steps
        c.px(round(x0 + (x1 - x0) * t), round(y0 + (y1 - y0) * t), col)


def disc(c, cx, cy, r, col):
    for y in range(int(cy - r) - 1, int(cy + r) + 2):
        for x in range(int(cx - r) - 1, int(cx + r) + 2):
            if (x + 0.5 - cx) ** 2 + (y + 0.5 - cy) ** 2 <= r * r:
                c.px(x, y, col)


def thick(c, pts, r0, r1, col, steps=40):
    """Stamp discs along a polyline, radius tapering r0 -> r1."""
    segs = list(zip(pts, pts[1:]))
    for si, ((x0, y0), (x1, y1)) in enumerate(segs):
        for i in range(steps + 1):
            t = i / steps
            u = (si + t) / len(segs)
            disc(c, x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, r0 + (r1 - r0) * u, col)


def wpx(c, x, y, col):
    c.g[y % c.h][x % c.w] = col


def periodic_noise(rng, size=32, cell=8):
    n = size // cell
    lat = [[rng.random() for _ in range(n)] for _ in range(n)]

    def sm(t):
        return t * t * (3 - 2 * t)

    out = [[0.0] * size for _ in range(size)]
    for y in range(size):
        for x in range(size):
            gx, gy = x / cell, y / cell
            ix, iy = int(gx), int(gy)
            fx, fy = sm(gx - ix), sm(gy - iy)
            a = lat[iy % n][ix % n]
            b = lat[iy % n][(ix + 1) % n]
            cc = lat[(iy + 1) % n][ix % n]
            d = lat[(iy + 1) % n][(ix + 1) % n]
            out[y][x] = (a * (1 - fx) + b * fx) * (1 - fy) + (cc * (1 - fx) + d * fx) * fy
    return out


# ── ground ───────────────────────────────────────────────────────────────

ASPHALT = {
    "C": "#16191f",  # deepest
    "A": "#1b1f27",  # base
    "B": "#20252e",  # lighter
    "H": "#2a303b",  # aggregate speck
    "S": "#262d3a",  # wet blue sheen
    "s": "#2f3848",  # bright sheen
    "r": "#2a1c24",  # faint red moon reflection
}


def asphalt(c, seed):
    rng = random.Random(seed)
    nz = periodic_noise(rng, 32, 8)
    nz2 = periodic_noise(rng, 32, 16)
    for y in range(32):
        for x in range(32):
            v = nz[y][x] * 0.6 + nz2[y][x] * 0.4 + rng.uniform(-0.12, 0.12)
            c.g[y][x] = "C" if v < 0.36 else ("A" if v < 0.62 else "B")
    for _ in range(26):
        wpx(c, rng.randrange(32), rng.randrange(32), "H")
    for _ in range(10):
        wpx(c, rng.randrange(32), rng.randrange(32), "C")
    # horizontal wet sheen streaks (reflections of distant lights)
    for _ in range(5):
        x, y, n = rng.randrange(32), rng.randrange(32), rng.randint(3, 7)
        for k in range(n):
            wpx(c, x + k, y, "s" if k in (n // 2,) else "S")
    for _ in range(2):
        x, y = rng.randrange(32), rng.randrange(32)
        for k in range(rng.randint(2, 4)):
            wpx(c, x + k, y, "r")


def ground():
    c = Canvas(32, 32, ASPHALT)
    asphalt(c, 41)
    c.save("ground_a", "Rainward 13: wet night asphalt")

    c = Canvas(32, 32, {**ASPHALT, "k": "#121419"})
    asphalt(c, 42)
    # a hairline crack, wrapped so it tiles
    rng = random.Random(9)
    x, y = 6, 20
    for _ in range(22):
        wpx(c, x, y, "k")
        if rng.random() < 0.3:
            wpx(c, x, y + 1, "k")
        x += 1
        y += rng.choice((-1, 0, 0, 1))
    # oily sheen patch
    for (dx, dy, col) in ((0, 0, "S"), (1, 0, "s"), (2, 0, "S"), (-1, 1, "S"), (0, 1, "S"), (3, 1, "S")):
        wpx(c, 22 + dx, 8 + dy, col)
    c.save("ground_b", "Rainward 13: cracked wet asphalt")

    # decal: puddle reflecting the red moon
    c = Canvas(32, 32, {**ASPHALT, "W": "#11141b", "E": "#2c3546", "R": "#5a1620",
                        "Q": "#8e222e", "q": "#c8323c", "L": "#3a4458"})
    asphalt(c, 43)
    pud = Canvas(32, 32, {})
    pud.ellipse(16, 17, 11, 6.5, "W")
    pud.ellipse(10, 14, 5, 3.5, "W")
    pud.ellipse(22, 20, 6, 3.5, "W")
    for y in range(32):
        for x in range(32):
            if pud.g[y][x] == "W":
                c.g[y][x] = "W"
                if any(not (0 <= x + dx < 32 and 0 <= y + dy < 32) or pud.g[y + dy][x + dx] == "."
                       for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                    c.g[y][x] = "E"
    # red moon reflection with cloud bands across it
    c.ellipse(17, 16, 3.6, 3.0, "R")
    c.ellipse(17, 16, 2.6, 2.1, "Q")
    c.ellipse(17.6, 15.6, 1.3, 1.0, "q")
    for x in range(12, 23):
        if c.g[17][x] in "RQq":
            c.g[17][x] = "R"
    for x in range(9, 13):
        c.px(x, 19, "R")
    for x in range(20, 25):
        c.px(x, 21, "R")
    # ripple rings from raindrops
    for (cx, cy) in ((10, 14), (23, 20)):
        for dx, dy in ((-2, 0), (2, 0), (-1, -1), (0, -1), (1, -1), (-1, 1), (0, 1), (1, 1)):
            c.px(cx + dx, cy + dy, "L")
    c.save("decal", "Rainward 13: puddle reflecting the red moon")

    # path: cracked concrete sidewalk with kerbs on both sides
    c = Canvas(32, 32, {"P": "#2b2f37", "p": "#272a31", "O": "#30343d", "J": "#1c1f25",
                        "k": "#1f2228", "T": "#3a3f4a", "U": "#22252c", "S": "#343c4c", "M": "#2a3a30"})
    rng = random.Random(44)
    for y in range(32):
        for x in range(32):
            c.g[y][x] = rng.choice("PPPPPpO")
    for y in (15, 31):
        c.rect(3, y, 28, y, "J")
    c.rect(16, 0, 16, 14, "J")
    for x in (0, 31):
        c.rect(x, 0, x, 31, "U")
    for x in (1, 30):
        c.rect(x, 0, x, 31, "T")
    for x in (2, 29):
        c.rect(x, 0, x, 31, "J")
    for y in range(0, 32, 8):  # kerb stone joints
        c.px(1, y, "U"); c.px(30, y + 4, "U"); c.px(0, y, "J"); c.px(31, y + 4, "J")
    for sx, sy, n in ((6, 2, 9), (20, 18, 10), (9, 20, 6)):
        x, y = sx, sy
        for _ in range(n):
            c.px(x, y, "k")
            x += rng.choice((-1, 0, 1, 1))
            y += rng.choice((1, 1, 0))
    for x, y in ((5, 9), (6, 9), (24, 6), (25, 6), (26, 6), (12, 26), (13, 26)):
        c.px(x, y, "S")  # wet sheen
    for x, y in ((3, 14), (3, 13), (28, 30), (28, 29), (4, 30)):
        c.px(x, y, "M")  # moss in the joints
    c.save("path", "Rainward 13: cracked concrete sidewalk with kerbs")


# ── props ────────────────────────────────────────────────────────────────

def props():
    # prop_a: flickering sodium street lamp, 16x32
    c = Canvas(16, 32, {"G": "#3c424e", "g": "#272b33", "h": "#59606e", "Y": "#fff0b0",
                        "O": "#ffb347", "o": "#e07a28", "D": "#5a3a1a"})
    c.rect(6, 6, 8, 29, "G")
    c.rect(6, 6, 6, 29, "g")
    c.rect(8, 6, 8, 29, "h")
    c.rect(5, 18, 9, 19, "g")           # access panel collar
    c.rect(4, 27, 10, 29, "G")
    c.rect(4, 30, 11, 31, "g")
    c.rect(4, 27, 4, 29, "g")
    # arm curving right
    for x, y in ((7, 5), (8, 4), (9, 3), (10, 3), (11, 3), (12, 3)):
        c.px(x, y, "G")
    c.px(7, 4, "G"); c.px(8, 3, "G")
    # lamp head
    c.rect(9, 1, 15, 2, "g")
    c.rect(10, 0, 14, 0, "h")
    c.rect(10, 3, 15, 3, "o")
    c.rect(11, 4, 14, 4, "O")
    c.rect(12, 4, 13, 4, "Y")
    c.rect(11, 5, 14, 5, "o")
    c.px(12, 5, "O")
    # peeling flyer on pole
    c.rect(6, 12, 8, 14, "D")
    c.save("prop_a", "Flickering sodium street lamp", outline=True)

    # prop_b: overflowing dumpster with trash bags, 32x32
    c = Canvas(32, 32, {"G": "#2f5040", "g": "#1f3529", "L": "#447058", "R": "#6a3a24",
                        "B": "#262a33", "b": "#363c48", "w": "#566078", "W": "#7a92b0",
                        "T": "#2a2a2e", "Y": "#a89a3a", "N": "#6a5a44"})
    poly(c, [(3, 13), (29, 13), (27, 29), (5, 29)], "G")
    c.rect(3, 13, 29, 15, "L")
    c.rect(3, 15, 29, 15, "g")
    for x in (9, 16, 23):
        c.rect(x, 16, x, 28, "g")
    c.rect(5, 26, 27, 28, "g")
    for x, y in ((6, 18), (7, 19), (20, 22), (21, 23), (13, 24), (25, 19)):
        c.px(x, y, "R")  # rust
    c.rect(10, 20, 15, 21, "Y")  # faded hazard label
    c.rect(11, 20, 11, 21, "g"); c.rect(13, 20, 13, 21, "g")
    # lid propped open at the back
    poly(c, [(4, 12), (28, 12), (26, 7), (6, 7)], "g")
    c.rect(6, 7, 26, 7, "L")
    # trash bags piled over the rim and spilling out
    for cx, cy, rx, ry in ((10, 11, 5, 4), (19, 10, 6, 4.5), (25, 12, 4, 3), (14, 8, 3.5, 3)):
        c.ellipse(cx, cy, rx, ry, "B")
        c.ellipse(cx - 1, cy - 1, rx * 0.5, ry * 0.4, "b")
        c.px(int(cx) - 2, int(cy) - 2, "w")
    c.px(19, 5, "B"); c.px(19, 4, "b")  # bag knot
    c.ellipse(28.5, 26.5, 3.5, 3.2, "B")
    c.ellipse(27.5, 25.5, 1.5, 1.2, "b")
    c.ellipse(3, 27, 3, 2.5, "B")
    c.px(2, 26, "w")
    c.rect(22, 9, 23, 11, "N")  # soggy cardboard
    # wheels
    c.rect(7, 29, 9, 30, "T"); c.rect(22, 29, 24, 30, "T")
    c.save("prop_b", "Overflowing dumpster with trash bags", outline=True)

    # prop_c: chain-link fence with a KEEP OUT sign, 48x32 (mesh stays see-through)
    c = Canvas(48, 32, {"M": "#4a5262", "m": "#353b47", "P": "#5a6272", "p": "#383e4a",
                        "R": "#a8262c", "r": "#6e1a20", "W": "#d8d4c8", "w": "#9a968c",
                        "X": "#7a6248", "Z": "#2a2e36"})
    for y in range(4, 29):
        for x in range(2, 46):
            if (x + y) % 4 == 0 or (x - y) % 4 == 0:
                c.px(x, y, "M" if y % 2 else "m")
    # a peeled-back hole in the mesh
    for y in range(20, 28):
        for x in range(31, 39):
            if (x - 35) ** 2 + (y - 25) ** 2 < 11:
                c.px(x, y, ".")
    for x, y in ((31, 21), (32, 20), (38, 21), (37, 20), (30, 24)):
        c.px(x, y, "m")
    c.rect(1, 3, 46, 4, "P")
    c.rect(1, 4, 46, 4, "p")
    for x in (1, 23, 45):
        c.rect(x, 1, x + 1, 31, "P")
        c.rect(x, 1, x, 31, "p")
        c.rect(x - 1, 30, x + 2, 31, "Z")
    # barbed wire on top
    for x in range(1, 47):
        c.px(x, 1 + (x // 2) % 2, "m")
        if x % 4 == 0:
            c.px(x, 0, "M")
    # KEEP OUT sign
    c.rect(9, 9, 20, 17, "W")
    c.rect(9, 9, 20, 11, "R")
    c.rect(9, 17, 20, 17, "w")
    for k, x in enumerate((10, 13, 16)):  # glyph blocks
        c.rect(x, 10, x + 1, 10, "W")
    for x0, x1 in ((10, 14), (16, 19)):
        c.rect(x0, 13, x1, 13, "r")
    c.rect(10, 15, 18, 15, "r")
    c.px(20, 16, "X"); c.px(19, 16, "X"); c.px(20, 15, "X")  # rust streak
    c.px(9, 9, "m"); c.px(20, 9, "m")  # bolts
    # outline only the solid parts (posts, rail, sign), not the mesh
    solid = set("PpRrWwXZ")
    g = c.g
    out = [row[:] for row in g]
    for y in range(32):
        for x in range(48):
            if g[y][x] not in solid and any(
                0 <= x + dx < 48 and 0 <= y + dy < 32 and g[y + dy][x + dx] in solid
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))
            ):
                out[y][x] = "K"
    c.g = out
    c.save("prop_c", "Chain-link fence with KEEP OUT sign")

    # prop_d: traffic cone + broken umbrella, 24x24
    c = Canvas(24, 24, {"O": "#e86a24", "o": "#a84418", "W": "#e8e4dc", "w": "#a8a49c",
                        "B": "#2c2a36", "b": "#423f52", "M": "#7a7e8a", "g": "#2a2e36"})
    c.rect(2, 21, 13, 22, "g")
    c.rect(3, 21, 12, 21, "o")
    poly(c, [(4.5, 21), (11.5, 21), (8.6, 4), (7.4, 4)], "O")
    poly(c, [(4.5, 21), (7, 21), (7.6, 4), (7.4, 4)], "o")
    c.rect(4, 10, 11, 11, "W"); c.rect(4, 10, 6, 11, "w")
    c.rect(4, 15, 11, 16, "W"); c.rect(4, 15, 5, 16, "w")
    c.rect(7, 3, 8, 4, "O")
    # broken umbrella lying on its side to the right
    poly(c, [(12, 22), (14, 15), (18, 12), (23, 14), (22, 22)], "B")
    for (x0, y0, x1, y1) in ((13, 21, 17, 13), (16, 22, 19, 13), (19, 22, 21, 14)):
        line(c, x0, y0, x1, y1, "b")
    line(c, 16, 15, 22, 9, "M")  # snapped spoke poking out
    line(c, 14, 22, 15, 20, "M")
    c.rect(12, 22, 23, 22, "B")
    c.px(22, 8, "M"); c.px(23, 8, "M")
    c.save("prop_d", "Traffic cone and broken umbrella", outline=True)


# ── enemies ──────────────────────────────────────────────────────────────

def fodder():
    # fodder_a: Drowned Rat
    c = Canvas(16, 16, {"B": "#4e4a5c", "b": "#36323f", "H": "#7a7a94", "T": "#b07484",
                        "t": "#7a4c5a", "E": "#ff3a3a", "Y": "#ffd0a0", "P": "#e090a0", "W": "#8ac0e0"})
    c.px(0, 9, "t"); c.px(0, 10, "t")
    line(c, 0, 11, 2, 12, "t")
    line(c, 2, 12, 3, 12, "T")
    c.ellipse(7, 11, 5.2, 3.4, "B")
    c.ellipse(7, 12.6, 4.4, 1.6, "b")
    c.ellipse(12, 10.3, 3.2, 2.4, "B")
    poly(c, [(13, 9), (16, 11), (13, 12)], "B")
    c.px(15, 11, "P")
    c.rect(10, 7, 11, 8, "B"); c.px(10, 8, "T")
    for x in (4, 6, 8):  # wet matted fur spikes
        c.px(x, 8, "H"); c.px(x + 1, 7, "B")
    c.px(5, 9, "H"); c.px(9, 9, "H")
    c.px(12, 9, "Y"); c.px(13, 9, "E")
    c.rect(4, 14, 5, 14, "b"); c.rect(9, 14, 10, 14, "b")
    c.px(11, 13, "b")
    c.outline()
    c.px(14, 13, "W"); c.px(3, 6, "W")  # drips (after outline)
    c.save("fodder_a", "Drowned Rat")

    # fodder_b: Rain Ghoul
    c = Canvas(16, 16, {"J": "#3e4a66", "j": "#29324a", "h": "#5a6a94", "S": "#cfd3da",
                        "s": "#8c94a2", "D": "#121420", "E": "#ff4a4a", "W": "#8ac0e0"})
    poly(c, [(3, 14), (3.5, 8), (6, 4), (10, 3), (12.5, 6), (12, 11), (10, 14)], "J")
    poly(c, [(3, 14), (3.5, 8), (6, 4), (6.5, 14)], "j")
    c.ellipse(10.5, 5.5, 3.6, 3.2, "J")
    c.ellipse(11.7, 6.3, 2.3, 2.3, "D")
    c.ellipse(12.4, 6.6, 1.6, 1.9, "S")
    c.px(11, 7, "s")
    c.px(13, 6, "E"); c.px(12, 6, "E")
    c.px(13, 8, "s")
    for x, y in ((8, 3), (7, 4), (9, 4)):
        c.px(x, y, "h")
    # long dangling arm to the ground
    line(c, 11, 8, 13, 11, "J")
    line(c, 13, 11, 14, 13, "S")
    c.px(13, 12, "s")
    c.px(15, 14, "S"); c.px(14, 14, "S"); c.px(15, 13, "s")
    line(c, 4, 9, 2, 13, "s")
    c.px(2, 14, "S")
    c.rect(5, 13, 6, 15, "j"); c.rect(9, 13, 10, 15, "j")
    c.px(6, 15, "s"); c.px(10, 15, "s")
    c.outline()
    c.px(8, 1, "W"); c.px(4, 3, "W")
    c.save("fodder_b", "Rain Ghoul")


def ranged_blinker():
    # ranged: Gutter Spitter
    c = Canvas(16, 16, {"G": "#5a6c3c", "g": "#3a4828", "H": "#8aa058", "w": "#2e3a20",
                        "M": "#22140f", "A": "#c8f03a", "a": "#f4ff9a", "Y": "#ffe040", "I": "#4a4e58"})
    c.ellipse(7, 9.5, 6.4, 5.2, "G")
    c.ellipse(6, 11.5, 5.2, 3, "g")
    c.ellipse(5.5, 7, 2.6, 1.8, "H")
    for x, y in ((3, 10), (8, 12), (5, 13), (10, 8)):
        c.px(x, y, "w")
    c.rect(11, 9, 13, 11, "M")
    c.px(13, 9, "G")
    c.px(12, 10, "A")
    c.px(12, 12, "A"); c.px(12, 13, "A")
    c.rect(14, 8, 15, 9, "A"); c.px(15, 8, "a")
    for x, y in ((9, 6), (11, 7), (7, 5)):
        c.px(x, y, "Y")
    c.rect(1, 6, 3, 6, "I"); c.px(2, 5, "I")  # rusty pipe stub in its back
    c.rect(3, 14, 4, 14, "g"); c.rect(9, 14, 10, 14, "g")
    c.outline()
    c.px(13, 15, "A")
    c.save("ranged", "Gutter Spitter")

    # blinker: Umbrella Stalker
    c = Canvas(16, 16, {"U": "#2e2c40", "u": "#5a5880", "D": "#0f0d16", "C": "#2c2640",
                        "c": "#4c4070", "M": "#8a8a9a", "E": "#ff3a50", "e": "#ffb0b8", "W": "#8ac0e0"})
    for y in range(1, 5):
        for x in range(16):
            if ((x + 0.5 - 8) / 7.8) ** 2 + ((y + 0.5 - 5) / 4.2) ** 2 <= 1:
                c.px(x, y, "U")
    for x in (0, 3, 6, 9, 12, 15):
        c.px(x, 5, "U")
    for x in range(2, 15):
        c.px(x, 2 if 5 < x < 11 else 3, "u") if x in (4, 8, 12) else None
    c.rect(4, 1, 11, 1, "u")
    c.px(8, 0, "M")
    c.rect(5, 5, 11, 7, "D")
    c.px(9, 6, "E"); c.px(11, 6, "E"); c.px(10, 6, "e")
    c.px(10, 6, "D")
    c.rect(6, 8, 10, 13, "C")
    c.rect(6, 8, 6, 13, "c"); c.rect(10, 9, 10, 13, "c")
    c.rect(8, 5, 8, 9, "M")
    c.px(9, 9, "c")
    c.rect(7, 14, 7, 15, "C"); c.rect(9, 14, 9, 15, "C")
    c.outline()
    c.px(1, 7, "W"); c.px(14, 8, "W"); c.px(15, 10, "W")
    c.save("blinker", "Umbrella Stalker")


def elite():
    c = Canvas(32, 32, {"R": "#8a7d3c", "r": "#5e5428", "L": "#b8a858", "D": "#100e14",
                        "E": "#ffd040", "X": "#6a1818", "x": "#9a2626", "P": "#c4bcaa", "p": "#8e8676",
                        "S": "#c09a84", "M": "#b0b8c2", "m": "#6a707a", "B": "#24242a", "W": "#8ac0e0"})
    poly(c, [(5, 30), (6, 14), (10, 10), (22, 10), (27, 14), (27, 30)], "R")
    poly(c, [(5, 30), (6, 14), (10, 10), (11, 30)], "r")
    c.ellipse(16, 7, 6.5, 6, "R")
    c.ellipse(13.5, 6.5, 3, 4.5, "r")
    c.ellipse(18.3, 8, 3.6, 3.6, "D")
    c.px(18, 8, "E"); c.px(20, 8, "E")
    c.rect(17, 10, 20, 10, "S")  # jaw in shadow
    c.px(18, 10, "D")
    for y in (2, 3):
        c.px(16 + y, y, "L")
    c.rect(22, 5, 22, 7, "L")
    # dirty butcher's apron
    c.rect(14, 15, 23, 29, "P")
    c.rect(14, 15, 15, 29, "p")
    c.rect(16, 13, 21, 14, "p")
    for (x, y, rx, ry) in ((19, 20, 2.2, 1.6), (16.5, 25, 1.8, 2.2), (22, 27, 1.5, 1.4)):
        c.ellipse(x, y, rx, ry, "X")
    c.px(19, 19, "x"); c.px(16, 24, "x"); c.px(19, 22, "X"); c.px(19, 23, "X")
    c.rect(26, 15, 26, 28, "L")
    for y in (17, 22, 27):
        c.rect(7, y, 11, y, "R")
    # back arm
    c.rect(2, 14, 5, 23, "r")
    c.rect(2, 24, 5, 26, "S")
    # front arm with meat hook
    poly(c, [(22, 11), (26, 12), (29, 19), (26, 21), (23, 16)], "R")
    c.rect(26, 20, 29, 22, "S")
    c.rect(25, 21, 30, 21, "m")  # T-grip
    c.rect(28, 23, 28, 26, "M")
    c.px(29, 23, "m")
    line(c, 28, 27, 26, 29, "M")
    c.px(25, 28, "M"); c.px(24, 27, "M"); c.px(24, 26, "x")
    c.px(29, 26, "m")
    # boots
    c.rect(7, 29, 13, 31, "B"); c.rect(18, 29, 24, 31, "B")
    c.outline()
    for x, y in ((9, 1), (3, 9), (29, 9), (12, 13)):
        c.px(x, y, "W")
    c.px(24, 30, "x")  # blood drip at hook tip... on floor
    c.save("elite", "Ward Butcher")


def mini():
    c = Canvas(32, 32, {"H": "#2a2a3a", "i": "#474a62", "b": "#6a2030", "S": "#b89a88", "s": "#8a6e62",
                        "V": "#5af0ff", "v": "#2a9cc0", "w": "#e8ffff", "C": "#7a6448", "c": "#52432f",
                        "L": "#a48a64", "G": "#d0a840", "B": "#20202a", "M": "#3a3c4a", "m": "#6e7282",
                        "Z": "#c8d2dc", "z": "#f0faff", "W": "#8ac0e0"})
    # trench coat body
    poly(c, [(7, 30), (9, 13), (22, 13), (25, 30)], "C")
    poly(c, [(7, 30), (9, 13), (12, 13), (11, 30)], "c")
    c.rect(22, 16, 23, 29, "L")
    line(c, 13, 13, 16, 20, "c")
    line(c, 19, 13, 16, 20, "L")
    c.rect(16, 21, 16, 30, "c")
    c.rect(9, 20, 23, 21, "c")
    c.rect(15, 20, 17, 21, "G")
    c.px(16, 20, "c")
    for y in (15, 17):
        c.px(18, y, "G")
    # popped collar
    poly(c, [(10, 14), (11, 9), (14, 12)], "C")
    poly(c, [(21, 14), (21, 9), (18, 12)], "L")
    # head
    c.rect(12, 7, 19, 12, "S")
    c.rect(12, 7, 13, 12, "s")
    c.rect(13, 8, 20, 9, "v")
    c.rect(14, 8, 19, 8, "V")
    c.rect(16, 9, 19, 9, "V")
    c.px(19, 8, "w"); c.px(20, 8, "V")
    c.rect(15, 11, 18, 11, "s")
    # hat
    c.rect(7, 5, 23, 6, "H")
    c.rect(11, 1, 20, 5, "H")
    c.rect(12, 1, 19, 1, "i")
    c.rect(11, 4, 20, 4, "b")
    c.rect(19, 2, 20, 3, "i")
    c.rect(7, 6, 23, 6, "i")
    c.px(15, 0, "H"); c.px(16, 0, "H")
    # back arm
    c.rect(6, 14, 8, 24, "c")
    c.rect(6, 25, 8, 26, "S")
    # front arm holding the opened briefcase-weapon
    poly(c, [(21, 13), (24, 14), (26, 20), (23, 21), (21, 16)], "C")
    c.rect(23, 20, 25, 21, "S")
    c.rect(22, 21, 29, 26, "M")
    c.rect(22, 21, 29, 21, "m")
    c.rect(22, 24, 29, 24, "v")
    c.px(25, 24, "V"); c.px(26, 24, "V")
    c.rect(29, 22, 29, 26, "m")
    # unfolded blade rising from the case
    for k in range(17):
        x = 27 + k * 0.25
        y = 20 - k
        c.px(round(x), y, "Z")
        c.px(round(x) + 1, y, "z" if k % 4 else "V")
    c.rect(26, 20, 29, 20, "m")
    c.px(31, 3, "z")
    # legs
    c.rect(11, 29, 13, 31, "B"); c.rect(18, 29, 20, 31, "B")
    c.rect(18, 31, 21, 31, "B")
    c.outline()
    for x, y in ((4, 4), (25, 8), (3, 18), (5, 1)):
        c.px(x, y, "W")
    c.save("mini", "The Night Investigator")


def boss():
    c = Canvas(48, 48, {"I": "#3a3e48", "i": "#5c626e", "D": "#0b0c10", "G": "#2e5a48", "g": "#1d3c31",
                        "H": "#4c8a68", "h": "#9ad8a8", "Y": "#f0e050", "y": "#b0a030", "O": "#e06020",
                        "p": "#200c10", "M": "#120a10", "T": "#d8d0b8", "R": "#6a3a24", "W": "#8ac0e0"})
    # manhole rim
    c.ellipse(24, 43.5, 22, 4.5, "I")
    c.ellipse(24, 42.5, 19, 3.2, "D")
    for x in range(6, 42, 5):
        c.px(x, 46, "i")
    c.rect(10, 40, 38, 40, "i")
    # slime mass
    c.ellipse(24, 28, 15, 15, "G")
    c.ellipse(24, 15, 12, 11, "G")
    c.ellipse(17, 30, 7, 13, "g")
    c.ellipse(15, 14, 4, 8, "g")
    c.ellipse(30, 10, 4, 3, "H")
    c.ellipse(33, 22, 3, 6, "H")
    # tentacle arms
    thick(c, [(12, 30), (5, 26), (3, 18), (6, 11)], 3.2, 1.2, "g")
    thick(c, [(36, 30), (43, 26), (45, 18), (42, 10)], 3.2, 1.2, "G")
    thick(c, [(42, 27), (45, 19)], 0.8, 0.6, "H")
    # embedded drain grate
    c.rect(9, 33, 15, 37, "I")
    for x in (10, 12, 14):
        c.rect(x, 34, x, 36, "D")
    c.rect(36, 34, 39, 36, "R")
    # big central eye + many small eyes
    c.ellipse(24, 15, 5, 4, "Y")
    c.ellipse(24.5, 15, 2.8, 2.8, "O")
    c.rect(24, 13, 24, 17, "p")
    c.px(22, 13, "h")
    for (x, y, r) in ((14, 10, 1.8), (33, 15, 1.9), (17, 22, 1.6), (31, 25, 2.2), (12, 25, 1.4),
                      (36, 9, 1.3), (20, 6, 1.3), (29, 5, 1.2), (38, 20, 1.3), (8, 15, 1.2)):
        disc(c, x, y, r, "Y")
        c.px(int(x), int(y), "p")
        if r > 1.5:
            c.px(int(x) - 1, int(y) - 1, "y")
    # mouth with teeth
    c.ellipse(24, 32, 9, 3.6, "M")
    for x in range(17, 31, 3):
        poly(c, [(x, 29), (x + 2, 29), (x + 1, 31.5)], "T")
    for x in range(18, 30, 3):
        poly(c, [(x, 35), (x + 2, 35), (x + 1, 32.8)], "T")
    # drips running over the rim
    for x, y0, y1 in ((11, 38, 42), (18, 41, 44), (30, 41, 45), (37, 37, 41)):
        c.rect(x, y0, x, y1, "G")
        c.px(x, y1, "H")
    for x, y in ((26, 3), (19, 9), (35, 30), (27, 38)):
        c.px(x, y, "h")
    c.outline()
    for x, y in ((3, 33), (44, 34), (6, 39), (41, 4), (2, 8)):
        c.px(x, y, "h")
    c.save("boss", "Mother of Gutters")


def final():
    c = Canvas(48, 48, {"R": "#c02030", "r": "#6a0c18", "Q": "#ff4a58", "N": "#1e0a10",
                        "W": "#ece6dc", "w": "#b4aca2", "k": "#4a3a40", "E": "#ff2a3a",
                        "C": "#5a0e1a", "c": "#2c0810", "L": "#8a1a2a", "B": "#a01424",
                        "M": "#d8b060", "S": "#8aa8c8"})
    # umbrella-like halo canopy behind the head
    hx, hy = 24, 12
    for y in range(48):
        for x in range(48):
            dx, dy = x + 0.5 - hx, y + 0.5 - hy
            d = math.hypot(dx, dy)
            if 10.5 <= d <= 15 and dy < 6:
                c.px(x, y, "R")
    for k in range(9):
        a = math.pi + k * math.pi / 8
        a = math.pi * 0.92 + k * (math.pi * 1.16) / 8
        tx, ty = hx + 17.5 * math.cos(a), hy + 17.5 * math.sin(a)
        line(c, hx + 6 * math.cos(a), hy + 6 * math.sin(a), hx + 15 * math.cos(a), hy + 15 * math.sin(a), "r")
        line(c, hx + 15 * math.cos(a), hy + 15 * math.sin(a), tx, ty, "Q")
    for y in range(48):
        for x in range(48):
            d = math.hypot(x + 0.5 - hx, y + 0.5 - hy)
            if 14.3 <= d <= 15 and y < hy + 6 and c.g[y][x] == "R":
                c.px(x, y, "Q")
    # long coat
    poly(c, [(7, 45), (14, 21), (18, 18), (30, 18), (34, 21), (41, 45)], "C")
    poly(c, [(7, 45), (14, 21), (18, 18), (20, 45)], "c")
    for x0, x1 in ((22, 20), (27, 29), (32, 36)):
        line(c, x0, 24, x1, 45, "L" if x0 > 24 else "c")
    line(c, 24, 19, 24, 45, "c")
    # collar spikes
    poly(c, [(15, 21), (17, 13), (21, 19)], "c")
    poly(c, [(33, 21), (31, 13), (27, 19)], "C")
    c.px(31, 14, "L"); c.px(30, 16, "L")
    # glowing blood core
    c.ellipse(24, 26, 2.6, 3, "B")
    c.ellipse(24, 25.5, 1.4, 1.6, "Q")
    # sleeves + liquid tendrils
    thick(c, [(15, 22), (11, 30), (9, 36)], 2.6, 2.4, "c")
    thick(c, [(33, 22), (37, 30), (39, 36)], 2.6, 2.4, "C")
    thick(c, [(9, 37), (5, 40), (2, 36), (3, 31)], 1.6, 0.6, "B")
    thick(c, [(39, 37), (43, 39), (46, 34), (44, 28), (46, 24)], 1.6, 0.6, "B")
    thick(c, [(38, 38), (40, 44)], 1.2, 0.6, "B")
    line(c, 43, 38, 45, 34, "Q"); line(c, 5, 39, 3, 36, "Q")
    # hem dissolving into blood-rain tendrils
    for x, y1 in ((9, 47), (13, 46), (17, 47), (22, 46), (27, 47), (31, 46), (35, 47), (39, 46)):
        c.rect(x, 44, x + 1, y1, "B")
        c.px(x, y1, "Q")
    c.rect(8, 45, 40, 45, "r")
    # head: dark hood + cracked porcelain mask
    c.ellipse(24, 11.5, 6, 7, "N")
    c.ellipse(24.6, 12, 4.4, 5.4, "W")
    c.ellipse(22.2, 12, 2, 4.6, "w")
    c.rect(22, 11, 23, 11, "E"); c.rect(26, 11, 27, 11, "E")
    c.px(27, 10, "Q")
    line(c, 27, 12, 27, 16, "B")  # blood tear
    line(c, 24, 6, 25, 9, "k"); line(c, 25, 9, 24, 12, "k"); c.px(26, 8, "k")
    c.rect(23, 15, 26, 15, "k")
    c.px(22, 16, "k")
    c.rect(22, 18, 26, 18, "M")  # gold clasp
    c.outline()
    # falling red rain around him (after outline)
    for x, y in ((3, 6), (4, 7), (44, 5), (45, 6), (6, 24), (42, 15), (1, 16), (47, 42), (2, 44)):
        c.px(x, y, "Q" if (x + y) % 2 else "S")
    c.save("final", "The Crimson Rain Lord")


def relic():
    from gen_icons import Icon
    i = Icon({"G": "#d0a040", "g": "#8a6420", "H": "#f6d878", "P": "#d8ccb0", "p": "#a89c80",
              "r": "#8a2020", "s": "#5a4a3a"})
    i.circle(4.5, 9.5, 3.4, "G")
    i.circle(4.5, 9.5, 1.6, ".")
    i.px(3, 7, "H"); i.px(2, 8, "H")
    i.line(7, 11, 13, 14, "G")
    i.line(7, 12, 13, 15, "g")
    i.rect(11, 15, 12, 15, "G"); i.px(12, 13, "G")
    i.px(10, 12, "H")
    i.rect(10, 14, 10, 15, "G")
    i.line(5, 6, 8, 4, "s")  # string to the tag
    i.rect(8, 0, 14, 6, "P")
    i.rect(8, 6, 14, 6, "p")
    i.px(8, 0, "p")
    i.rect(10, 1, 10, 5, "r"); i.px(9, 2, "r")      # "1"
    i.rect(12, 1, 13, 1, "r"); i.px(13, 2, "r")     # "3"
    i.rect(12, 3, 13, 3, "r"); i.px(13, 4, "r"); i.rect(12, 5, 13, 5, "r")
    i.save("relic_w4", "Ward 13 Key")


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    ground()
    props()
    fodder()
    ranged_blinker()
    elite()
    mini()
    boss()
    final()
    relic()
    print("wrote", len(list(OUT.glob("*.px"))), "w4 sprites to", OUT, "+ icons/relic_w4.px")


if __name__ == "__main__":
    main()
