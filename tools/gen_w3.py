"""Generates World 3 (Pale Citadel of Souls) tiles, props and enemies as .px sources in art/src/w3/.

    python3 tools/gen_w3.py

A white-walled spirit citadel in the afterlife: moonlit slate courtyards, black-tiled roofs,
blue-white spirit flames and drifting souls. Deterministic (seeded).
Also writes art/src/icons/relic_w3.px (Pale Mask Shard).
"""
import math
import pathlib
import random
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_stage  # noqa: E402
from gen_stage import Canvas  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent.parent / "art/src/w3"
gen_stage.OUT = OUT


# ── helpers ──────────────────────────────────────────────────────────────────

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


def line(c, x0, y0, x1, y1, col, w=1):
    steps = int(max(abs(x1 - x0), abs(y1 - y0)) * 2) + 1
    for i in range(steps + 1):
        t = i / steps
        x, y = round(x0 + (x1 - x0) * t), round(y0 + (y1 - y0) * t)
        for dx in range(w):
            for dy in range(w):
                c.px(x + dx - w // 2, y + dy - w // 2, col)


def wpx(c, x, y, col):
    """Wrapping pixel (for seamless tiles)."""
    c.g[y % c.h][x % c.w] = col


def wrect(c, x0, y0, x1, y1, col):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            wpx(c, x, y, col)


def recolor(c, frm, to, region=None):
    x0, y0, x1, y1 = region or (0, 0, c.w - 1, c.h - 1)
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            if 0 <= x < c.w and 0 <= y < c.h and c.g[y][x] in frm:
                c.g[y][x] = to


def shade_right(c, body, shade, frac=0.7, region=None):
    """Turn the right-most part of each run of `body` into `shade` (simple form shading)."""
    for y in range(c.h):
        xs = [x for x in range(c.w) if c.g[y][x] == body]
        if not xs:
            continue
        cut = xs[0] + (xs[-1] - xs[0]) * frac
        for x in xs:
            if x > cut:
                c.g[y][x] = shade


# ── ground ───────────────────────────────────────────────────────────────────

GROUND = {
    "A": "#2a3041", "B": "#2e3546", "C": "#272c3c",  # slab tones (moonlit slate)
    "M": "#1b1f2b", "H": "#353d51", "D": "#222735",  # mortar, top-edge highlight, wear/crack
}


def slabs(c, rng, split=0.0):
    """Two rows of 16x16 slabs, second row offset by 8; drawn with wrapping so it tiles."""
    for row in range(2):
        off = 8 if row else 0
        for col in range(2):
            x0, y0 = col * 16 + off, row * 16
            tone = rng.choice("AABBC")
            wrect(c, x0, y0, x0 + 15, y0 + 15, tone)
            # speckle texture
            for _ in range(14):
                wpx(c, x0 + rng.randint(1, 14), y0 + rng.randint(1, 14), rng.choice("ABCD".replace(tone, "")))
            # moonlit top edge + left edge (low contrast)
            wrect(c, x0, y0, x0 + 14, y0, "H")
            wrect(c, x0, y0 + 1, x0, y0 + 14, "H")
            if rng.random() < split:  # a slab broken into two halves
                wrect(c, x0, y0 + 7, x0 + 14, y0 + 7, "M")
                wrect(c, x0 + 1, y0 + 8, x0 + 14, y0 + 8, "H")
            # mortar
            wrect(c, x0, y0 + 15, x0 + 15, y0 + 15, "M")
            wrect(c, x0 + 15, y0, x0 + 15, y0 + 15, "M")
            wpx(c, x0, y0 + 15, "M")
            if rng.random() < 0.45:  # hairline crack
                x, y = x0 + rng.randint(3, 11), y0 + rng.randint(2, 6)
                for _ in range(rng.randint(3, 6)):
                    wpx(c, x, y, "D")
                    x, y = x + rng.choice((-1, 0, 1)), y + 1


def ground():
    rng = random.Random(303)
    c = Canvas(32, 32, GROUND)
    slabs(c, rng)
    c.save("ground_a", "W3 ground: moonlit slate courtyard slabs")

    c = Canvas(32, 32, {**GROUND, "S": "#2c3a4a"})
    slabs(c, random.Random(331), split=0.6)
    # faint bluish soul-stain where spirits linger
    srng = random.Random(9)
    for _ in range(10):
        x, y = srng.randint(4, 27), srng.randint(4, 27)
        if c.g[y][x] in "ABC":
            c.g[y][x] = "S"
    c.save("ground_b", "W3 ground variant: weathered split slabs")

    # decal: ground_a slabs with a cracked, glowing spirit sigil in the middle
    c = Canvas(32, 32, {**GROUND, "G": "#4f9be0", "g": "#2f5578", "W": "#a8dcff"})
    slabs(c, random.Random(303))
    cx, cy = 16, 16
    for y in range(32):
        for x in range(32):
            d = math.hypot(x + 0.5 - cx, y + 0.5 - cy)
            if 9.3 <= d < 10.3:
                c.g[y][x] = "g"
            elif 6.0 <= d < 6.9:
                c.g[y][x] = "G" if (x + y) % 3 else "g"
    # six-pointed sigil (two triangles)
    for k in range(2):
        pts = [(cx + 6.2 * math.cos(a), cy + 6.2 * math.sin(a))
               for a in (k * math.pi / 3 - math.pi / 2 + i * 2 * math.pi / 3 for i in range(3))]
        for i in range(3):
            line(c, pts[i][0] - 0.5, pts[i][1] - 0.5, pts[i - 1][0] - 0.5, pts[i - 1][1] - 0.5, "g")
    # runes around the ring
    for i in range(8):
        a = i * math.pi / 4 + 0.39
        c.px(round(cx - 0.5 + 8.1 * math.cos(a)), round(cy - 0.5 + 8.1 * math.sin(a)), "G")
    c.rect(15, 15, 16, 16, "W")
    c.px(15, 14, "G"); c.px(16, 17, "G"); c.px(14, 16, "G"); c.px(17, 15, "G")
    # the slab is cracked through the sigil
    for path in (((16, 16), (11, 9), (8, 5), (7, 3)), ((16, 16), (22, 20), (25, 25)), ((16, 16), (20, 10), (23, 8))):
        for (x0, y0), (x1, y1) in zip(path, path[1:]):
            line(c, x0, y0, x1, y1, "M")
    for x, y in ((13, 12), (18, 18), (19, 12), (12, 11), (21, 19)):
        c.px(x, y, "G")
    c.save("decal", "W3 decal: cracked slab with glowing spirit sigil")

    # path: long polished walkway of darker slabs between pale curb stones
    P = {"P": "#1d2231", "Q": "#212738", "R": "#2b3449", "S": "#36435e",
         "M": "#141822", "E": "#363d50", "e": "#2c3242"}
    c = Canvas(32, 32, P)
    prng = random.Random(77)
    c.rect(0, 0, 31, 31, "P")
    for row in range(4):
        y0 = row * 8
        tone = "PQ"[row % 2]
        c.rect(3, y0, 28, y0 + 7, tone)
        c.rect(3, y0 + 7, 28, y0 + 7, "M")
        seam = 11 if row % 2 else 20
        c.rect(seam, y0, seam, y0 + 6, "M")
        # polished sheen: diagonal streaks of reflected moonlight
        for k in range(3):
            sx = 5 + ((row * 7 + k * 3) % 18)
            c.px(sx, y0 + 2 + k, "R")
            c.px(sx + 1, y0 + 1 + k, "R")
        if row % 2 == 0:
            c.px(24, y0 + 3, "S"); c.px(25, y0 + 2, "R")
        for _ in range(4):
            c.px(prng.randint(4, 27), y0 + prng.randint(1, 5), "R" if prng.random() < 0.3 else "Q" if tone == "P" else "P")
    # curb stones on both sides (lighter, 16px long blocks)
    for side in (0, 29):
        c.rect(side, 0, side + 2, 31, "E")
        for y0 in (0, 16):
            c.rect(side, y0 + 15, side + 2, y0 + 15, "M")
            c.rect(side, y0, side + 2, y0, "e")
        c.rect(side + (2 if side == 0 else 0), 0, side + (2 if side == 0 else 0), 31, "M")
    c.save("path", "W3 path: polished dark-slab walkway with curb stones")


# ── props ────────────────────────────────────────────────────────────────────

FLAME = {"F": "#3d8ee8", "f": "#8fd0ff", "W": "#eef9ff"}
STONE = {"S": "#a9aec0", "L": "#c8ccda", "s": "#7d8398", "d": "#5a5f72"}
ROOF = {"T": "#1e2030", "t": "#2d3045", "r": "#464c66"}


def flame(c, cx, top, bottom, rx):
    """Upright teardrop spirit flame."""
    h = bottom - top
    for y in range(top, bottom + 1):
        t = (y - top) / h
        w = rx * math.sin(min(1.0, t * 1.25) * math.pi / 2) * (1 - max(0, t - 0.8) * 2.5)
        c.rect(round(cx - w), y, round(cx + w) - 1, y, "F")
        if t > 0.25:
            c.rect(round(cx - w * 0.6), y, round(cx + w * 0.6) - 1, y, "f")
        if t > 0.5:
            c.rect(round(cx - w * 0.28), y, round(cx + w * 0.28) - 1, y, "W")


def props():
    # prop_a: spirit-flame brazier on a stand
    c = Canvas(16, 32, {**FLAME, "I": "#45465c", "i": "#2a2a3a", "J": "#6a6c86"})
    flame(c, 8, 1, 13, 5.5)
    c.px(4, 3, "F"); c.px(11, 2, "f"); c.px(12, 5, "F"); c.px(3, 7, "F")
    poly(c, [(1.5, 12), (14.5, 12), (12, 17), (4, 17)], "I")
    c.rect(1, 12, 14, 12, "J")
    c.rect(4, 16, 11, 16, "i")
    c.rect(3, 13, 12, 13, "f")  # glow inside the bowl rim
    c.rect(4, 13, 11, 13, "W")
    c.rect(7, 17, 8, 28, "I")
    c.rect(8, 17, 8, 28, "i")
    c.rect(6, 21, 9, 22, "J")
    c.rect(6, 22, 9, 22, "i")
    poly(c, [(5, 27), (11, 27), (13, 31), (3, 31)], "I")
    c.rect(4, 31, 12, 31, "i")
    c.px(6, 28, "J"); c.px(5, 29, "J")
    c.save("prop_a", "W3 prop: spirit-flame brazier", outline=True)

    # prop_b: stone gate pillar segment with black roof-tile cap
    c = Canvas(32, 32, {**STONE, **ROOF, "G": "#4f9be0", "g": "#2f5578"})
    c.rect(8, 11, 23, 28, "S")
    c.rect(8, 11, 10, 28, "L")
    c.rect(20, 11, 23, 28, "s")
    for y in (16, 22):
        c.rect(8, y, 23, y, "d")
    c.rect(9, 13, 21, 13, "s")
    # carved sigil (faint glow)
    c.rect(14, 17, 17, 17, "g"); c.rect(15, 18, 16, 20, "G"); c.rect(14, 21, 17, 21, "g")
    # cracks
    line(c, 19, 23, 21, 27, "d"); line(c, 11, 11, 12, 14, "d")
    # plinth
    c.rect(5, 27, 26, 31, "s")
    c.rect(5, 27, 26, 27, "L")
    c.rect(6, 28, 25, 28, "S")
    c.rect(4, 30, 27, 31, "d")
    # roof cap: upswept eaves, tile ridges
    poly(c, [(0, 5), (4, 8), (27, 8), (31, 5), (31, 7), (27, 11), (4, 11), (0, 7)], "T")
    poly(c, [(5, 3), (26, 3), (28, 8), (3, 8)], "t")
    c.rect(6, 1, 25, 3, "T")
    c.rect(4, 0, 6, 1, "T"); c.rect(25, 0, 27, 1, "T")
    c.rect(7, 2, 24, 2, "r")
    for x in range(5, 28, 3):
        c.rect(x, 4, x, 8, "T")
        c.px(x + 1, 4, "r")
    c.rect(3, 9, 28, 9, "r")
    c.rect(8, 11, 23, 11, "d")
    c.save("prop_b", "W3 prop: gate pillar with black tile cap", outline=True)

    # prop_c: low white wall segment with black-tiled top
    c = Canvas(48, 32, {**STONE, **ROOF, "W": "#b4b8c8", "w": "#959aad", "v": "#7a8094"})
    c.rect(4, 12, 43, 27, "W")
    c.rect(4, 12, 43, 14, "v")  # shadow under the eave
    c.rect(4, 15, 43, 15, "w")
    c.rect(39, 15, 43, 27, "w")
    rngc = random.Random(5)
    for _ in range(18):  # weathering stains
        x, y = rngc.randint(6, 40), rngc.randint(17, 26)
        c.px(x, y, "w")
    for x in range(8, 40, 2):  # rain streaks below the eave
        if rngc.random() < 0.5:
            c.rect(x, 15, x, 15 + rngc.randint(2, 5), "w")
    # lattice vent
    c.rect(20, 17, 27, 22, "d")
    for x in (21, 23, 25):
        c.rect(x, 17, x, 22, "s")
    c.rect(20, 19, 27, 19, "s")
    c.rect(20, 17, 27, 17, "v")
    # crack
    line(c, 32, 16, 34, 20, "v"); line(c, 34, 20, 33, 24, "v")
    # stone foundation
    c.rect(3, 26, 44, 31, "s")
    for x0 in range(3, 44, 7):
        c.rect(x0, 26, x0, 31, "d")
        c.rect(x0 + 1, 26, x0 + 5, 26, "S")
    c.rect(3, 29, 44, 29, "d")
    for x0 in range(6, 44, 7):
        c.rect(x0, 30, x0, 31, "d")
    # tile roof
    poly(c, [(0, 7), (4, 9), (43, 9), (47, 7), (47, 10), (44, 13), (3, 13), (0, 10)], "T")
    poly(c, [(4, 3), (43, 3), (45, 9), (2, 9)], "t")
    c.rect(3, 1, 44, 3, "T")
    c.rect(1, 0, 3, 1, "T"); c.rect(44, 0, 46, 1, "T")
    c.rect(4, 2, 43, 2, "r")
    for x in range(3, 46, 3):
        c.rect(x, 4, x, 9, "T")
        c.px(x + 1, 4, "r")
    c.rect(2, 11, 45, 11, "r")
    for x in range(3, 45, 3):
        c.px(x, 12, "t")
    c.save("prop_c", "W3 prop: white citadel wall with black-tiled top", outline=True)

    # prop_d: pile of broken bone-white masks and rubble
    c = Canvas(24, 24, {"R": "#5a6070", "r": "#454a5a", "q": "#353a48",
                        "B": "#d9d4c2", "b": "#b3ad9a", "n": "#8a8574", "E": "#14121c", "G": "#6fc0ff"})
    c.ellipse(12, 21, 9, 3.5, "r")
    c.ellipse(6, 19, 3.5, 3, "R"); c.ellipse(17, 19.5, 4, 2.8, "R")
    c.ellipse(13, 21.5, 3, 2, "q")
    c.rect(4, 22, 19, 23, "q")
    # big half-mask lying tilted
    c.ellipse(11, 14, 6, 5.5, "B")
    poly(c, [(13, 8), (18, 12), (16, 15), (12, 12)], ".")  # broken-off chunk
    c.ellipse(9, 13, 1.6, 1.6, "E"); c.ellipse(13.5, 13, 1.4, 1.5, "E")
    c.px(9, 13, "G")
    c.rect(9, 17, 13, 17, "n")
    for x in (9, 11, 13):
        c.px(x, 18, "b")
    shade_right(c, "B", "b", 0.75)
    line(c, 12, 9, 10, 11, "n"); line(c, 10, 11, 11, 13, "n")
    # small shard + second mask fragment
    poly(c, [(15, 13), (20, 12), (21, 17), (16, 18)], "B")
    c.px(18, 14, "E"); c.px(19, 14, "E"); c.px(19, 16, "n")
    poly(c, [(2, 15), (6, 14), (6, 18), (2, 19)], "b")
    c.px(4, 16, "E")
    c.px(20, 20, "B"); c.px(3, 21, "b"); c.px(15, 21, "b")
    c.save("prop_d", "W3 prop: pile of broken bone-white masks", outline=True)


# ── enemies ──────────────────────────────────────────────────────────────────

def enemies():
    # fodder_a: Lost Soul — small teardrop soul flame with a sad face (tail streams back/left)
    c = Canvas(16, 16, {"F": "#3a7fd6", "f": "#7cc4ff", "W": "#dff3ff", "E": "#101c36", "T": "#ffffff"})
    c.ellipse(9, 10.5, 5.6, 5, "F")
    poly(c, [(4, 9), (1, 1), (8, 5.5), (13, 7.5)], "F")
    poly(c, [(7, 6), (9, 1), (10, 6)], "F")
    c.ellipse(9.3, 10.6, 4.4, 3.9, "f")
    poly(c, [(5, 9), (2.5, 3.5), (8, 7)], "f")
    c.px(6, 8, "W"); c.px(7, 8, "W"); c.px(6, 9, "W"); c.px(5, 7, "T")
    # sad face
    c.rect(8, 10, 8, 11, "E"); c.rect(11, 10, 11, 11, "E")
    c.px(7, 9, "E"); c.px(12, 9, "E")      # drooping brows
    c.px(9, 14, "E"); c.rect(10, 13, 11, 13, "E"); c.px(12, 14, "E")
    c.px(8, 12, "T")  # tear
    c.save("fodder_a", "W3 fodder: Lost Soul", outline=True)

    # fodder_b: Masked Hound — four-legged shadow beast with a cracked bone mask
    c = Canvas(16, 16, {"N": "#2e2742", "n": "#1f1a2e", "L": "#463c62",
                        "B": "#e2dccb", "b": "#aaa390", "E": "#7fe8ff", "X": "#14121c"})
    c.ellipse(7, 9, 5.5, 3.4, "N")
    c.rect(2, 7, 4, 7, "L"); c.rect(4, 6, 10, 6, "L"); c.rect(3, 8, 10, 8, "L")
    poly(c, [(2, 6), (0, 4), (1, 7), (0, 9), (3, 9)], "N")  # smoky tail
    c.px(0, 3, "L")
    poly(c, [(4, 6), (5, 3), (6, 6)], "N"); poly(c, [(7, 6), (8, 3), (9, 6)], "N")  # smoke spines
    for x in (2, 4, 9, 11):
        c.rect(x, 11, x + 1, 14, "N" if x in (4, 11) else "n")
        c.px(x + 1, 14, "n")
    c.rect(2, 11, 12, 11, "n")
    # head + mask
    c.ellipse(12, 7, 3.5, 3, "N")
    poly(c, [(10, 4), (14, 4), (16, 7), (16, 9), (12, 10), (10, 8)], "B")
    c.rect(13, 9, 15, 9, "b"); c.px(10, 7, "b"); c.px(11, 8, "b")
    c.px(12, 6, "X"); c.rect(13, 6, 14, 6, "E"); c.px(13, 5, "X")
    line(c, 14, 4, 15, 7, "X")  # crack
    c.px(15, 9, "X")
    poly(c, [(10, 4), (11, 1), (12, 4)], "b")  # ear horn
    c.save("fodder_b", "W3 fodder: Masked Hound", outline=True)

    # ranged: Pale Archer — ghostly archer in tattered white robes with a spirit bow
    c = Canvas(16, 16, {"R": "#cfd7e8", "r": "#98a4bf", "q": "#6c7690", "X": "#161a28",
                        "E": "#8fe0ff", "G": "#5aa8ff", "g": "#cdeaff"})
    poly(c, [(4, 5), (10, 5), (11, 13), (2, 13)], "R")
    for x, d in ((2, 14), (3, 15), (5, 14), (6, 15), (8, 14), (9, 15), (10, 14)):
        c.rect(x, 13, x, d, "R")
    c.ellipse(7, 4, 3.3, 3.5, "R")        # hood
    c.ellipse(8, 4.5, 2, 2.2, "X")        # shadowed face
    c.px(8, 4, "E"); c.px(9, 4, "E")
    shade_right(c, "R", "r", 0.55)
    c.rect(3, 8, 4, 13, "q")             # back fold of the robe
    c.rect(6, 7, 11, 8, "r")             # drawing arm
    c.rect(10, 8, 11, 8, "R")
    # bow: glowing arc on the right
    for y in range(1, 15):
        x = 13 + round(1.6 * math.sin((y - 1) / 13 * math.pi))
        c.px(x, y, "G")
    c.rect(12, 1, 12, 14, "g")           # string
    c.rect(6, 7, 15, 7, "g"); c.px(15, 6, "g")   # spirit arrow
    c.px(5, 7, "E")
    c.save("ranged", "W3 ranged: Pale Archer", outline=True)

    # blinker: Shade Dancer — thin black silhouette, white mask, trailing ribbons
    c = Canvas(16, 16, {"N": "#28213a", "L": "#5a4c82", "M": "#ece8f4", "m": "#b6b0c6",
                        "X": "#14121c", "E": "#5ad0ff", "V": "#d2dcf6", "v": "#6f8fe6"})
    c.ellipse(11, 2.5, 2, 2.4, "N")                  # head
    line(c, 11, 4, 10, 10, "N", 2)                   # long thin torso
    line(c, 11, 5, 14, 1, "N")                       # arm raised forward
    line(c, 10, 5, 7, 4, "N"); c.px(6, 4, "N")       # arm swept back
    line(c, 10, 10, 11, 15, "N"); c.px(12, 15, "N")  # standing leg on tiptoe
    line(c, 9, 10, 5, 12, "N"); c.px(4, 12, "N")     # leg kicked back
    line(c, 9, 9, 8, 11, "N")                        # skirt flare
    for x, y in ((10, 5), (10, 6), (10, 7), (12, 3), (11, 12), (7, 11), (13, 2)):
        c.px(x, y, "L")                              # rim light
    c.rect(11, 1, 13, 3, "M")                        # white mask
    c.px(12, 0, "M"); c.px(13, 4, "m"); c.px(11, 3, "m")
    c.px(12, 2, "E"); c.px(13, 2, "X")
    c.outline()
    # two ribbons streaming back from the hand and the waist (unoutlined, they glow faintly)
    for (xe, ye, amp, drift, col) in ((5, 4, 1.2, -0.25, "V"), (8, 9, 1.2, 0.3, "v")):
        for x in range(0, xe + 1):
            y = ye + round(amp * math.sin((xe - x) * 0.6) + drift * (xe - x))
            for yy in ((y, y + 1) if x % 4 == 1 else (y,)):
                if c.g[yy][x] in ".K":
                    c.g[yy][x] = col
    c.save("blinker", "W3 blinker: Shade Dancer")

    # elite: Bone Sentinel — towering skeletal guard in pale armour with a halberd
    c = Canvas(32, 32, {"B": "#ddd8c8", "b": "#b0aa98", "n": "#7e7968",
                        "A": "#9aa6bc", "L": "#c6cfe0", "a": "#6a7488", "q": "#4a5266",
                        "C": "#2a2d40", "E": "#6fd8ff", "X": "#14121c",
                        "H": "#4a3a3a", "S": "#d8e2f0", "s": "#8e9ab0"})
    # halberd (behind right arm): shaft + axe blade + spike
    c.rect(25, 3, 25, 31, "H")
    c.rect(25, 0, 25, 3, "S")
    poly(c, [(26, 3), (30, 2), (31, 6), (30, 10), (26, 8)], "S")
    poly(c, [(28, 4), (30, 3), (30.6, 6), (30, 9), (28, 7)], "s")
    poly(c, [(24, 5), (21, 6), (24, 7)], "s")
    # legs: bone shins, armoured thighs
    c.rect(10, 22, 13, 25, "A"); c.rect(17, 22, 20, 25, "a")
    c.rect(11, 26, 12, 29, "B"); c.rect(18, 26, 19, 29, "b")
    c.rect(9, 30, 13, 31, "a"); c.rect(17, 30, 21, 31, "q")
    # tattered dark tabard
    poly(c, [(11, 17), (20, 17), (21, 25), (18, 23), (15, 26), (13, 23), (10, 25)], "C")
    # torso: ribcage between breastplate halves
    c.rect(10, 9, 21, 17, "A")
    c.rect(10, 9, 12, 17, "L")
    c.rect(19, 9, 21, 17, "a")
    c.rect(13, 10, 18, 16, "X")
    for y in (10, 12, 14):
        c.rect(13, y, 18, y, "B")
    c.rect(15, 10, 16, 16, "b")   # spine
    c.rect(10, 17, 21, 17, "a")
    # pauldrons
    c.ellipse(8, 10, 3.5, 2.6, "L"); c.ellipse(23, 10, 3.5, 2.6, "A")
    c.rect(5, 11, 11, 11, "a"); c.rect(20, 11, 26, 11, "q")
    # arms: left hangs, right grips halberd
    c.rect(6, 12, 7, 19, "B"); c.rect(6, 20, 7, 21, "b")
    c.rect(22, 12, 23, 16, "b"); c.rect(23, 16, 25, 17, "B"); c.rect(24, 15, 26, 18, "b")
    # skull in a crested helm
    c.ellipse(16, 5, 4.2, 4.2, "B")
    c.rect(13, 7, 19, 9, "B")
    c.rect(14, 8, 18, 8, "n")
    for x in (14, 16, 18):
        c.px(x, 9, "n")
    c.rect(14, 4, 15, 6, "X"); c.rect(17, 4, 18, 6, "X")
    c.px(15, 5, "E"); c.px(18, 5, "E"); c.px(14, 5, "E")
    c.px(16, 7, "X")
    poly(c, [(11.5, 4), (16, 0), (20.5, 4)], "A")   # helm
    c.rect(12, 3, 20, 3, "L")
    c.rect(15, 0, 16, 2, "L")
    c.rect(12, 4, 12, 6, "a"); c.rect(20, 4, 20, 6, "a")
    c.save("elite", "W3 elite: Bone Sentinel", outline=True)

    # mini: Captain of the Ninth Gate — gate-shaped tower shield, long spear, white cloak
    c = Canvas(32, 32, {"W": "#d6dbe6", "w": "#a8afc2", "v": "#7a8298",
                        "N": "#2c3350", "n": "#1f2438", "P": "#46507a", "Y": "#c9b27a", "y": "#8a7648",
                        "S": "#b4b9c9", "s": "#828799", "T": "#1e2030", "t": "#3a3f58",
                        "D": "#141826", "G": "#5ab4ff", "g": "#2f5f90", "E": "#9fe8ff", "H": "#5a4636", "B": "#e4ecf8"})
    # spear held upright in the back hand
    c.rect(5, 4, 5, 30, "H")
    poly(c, [(5.5, 0), (7, 4), (5.5, 6), (4, 4)], "B")
    c.rect(4, 6, 6, 6, "Y")
    c.px(4, 7, "W"); c.px(6, 8, "W")  # tassel
    # white cloak flaring left
    poly(c, [(8, 8), (15, 8), (16, 29), (11, 30), (7, 28), (2, 30), (4, 20)], "W")
    poly(c, [(4, 20), (8, 12), (9, 28), (7, 28), (2, 30)], "w")
    for x in (5, 8):
        c.rect(x, 24, x, 29, "v")
    # legs / greaves
    c.rect(11, 25, 13, 29, "N"); c.rect(14, 25, 16, 29, "n")
    c.rect(10, 30, 13, 31, "n"); c.rect(14, 30, 17, 31, "D")
    # armoured torso
    c.rect(9, 11, 17, 24, "N")
    c.rect(9, 11, 11, 24, "P")
    c.rect(9, 18, 17, 19, "Y"); c.rect(9, 19, 17, 19, "y")
    for y in (21, 23):
        c.rect(9, y, 17, y, "n")
    c.ellipse(9, 11, 3, 2.2, "P"); c.rect(7, 12, 11, 12, "Y")
    c.rect(6, 12, 7, 17, "N")  # back arm to spear
    c.rect(5, 16, 7, 17, "Y")
    # head: kabuto-like helm with a tall crest and face guard (original design)
    c.ellipse(13, 7, 3.6, 3.4, "N")
    c.rect(11, 6, 16, 9, "D")
    c.rect(13, 7, 16, 7, "E")  # visor glow
    c.rect(9, 5, 17, 5, "Y")
    poly(c, [(12, 4), (13, 0), (14, 4)], "Y")
    poly(c, [(13, 3), (17, 0), (15, 4)], "Y")
    c.rect(10, 9, 16, 10, "W")   # cloak collar
    # gate-shaped shield (front, right side): black tile lintel, pale posts, iron doors, sigil
    sx0, sx1 = 17, 30
    poly(c, [(15.5, 11), (31.5, 11), (30, 14), (17, 14)], "T")
    c.rect(16, 11, 31, 11, "t")
    c.px(15, 10, "T"); c.px(31, 10, "T")
    c.rect(sx0, 14, sx0 + 1, 29, "S"); c.rect(sx1 - 1, 14, sx1, 29, "s")
    c.rect(sx0 + 2, 14, sx1 - 2, 29, "D")
    c.rect(sx0 + 2, 15, sx1 - 2, 15, "s")
    c.rect(23, 16, 23, 29, "n")  # door split
    for y in (18, 25):
        for x in (20, 22, 25, 27):
            c.px(x, y, "t")
    c.ellipse(23.5, 21.5, 3.2, 3.2, "g")
    c.ellipse(23.5, 21.5, 2.2, 2.2, "D")
    c.rect(23, 19, 24, 24, "G"); c.rect(21, 21, 26, 22, "G")
    c.rect(23, 21, 24, 22, "E")
    c.rect(sx0, 29, sx1, 29, "s")
    c.rect(sx0, 30, sx1, 30, "T")
    c.save("mini", "W3 mini-boss: Captain of the Ninth Gate", outline=True)

    boss()
    final()


def boss():
    # The Hollow Monarch: masked beast-king of black smoke and bone, horn crown, hollow chest, blue core
    c = Canvas(48, 48, {"N": "#221c33", "n": "#16121f", "L": "#3b3258", "l": "#2e2745",
                        "B": "#e4dece", "b": "#b8b19e", "o": "#857f6e",
                        "X": "#0c0a12", "E": "#6fd8ff", "C": "#4ab0ff", "c": "#2a5f9a", "W": "#ffffff", "w": "#a8e4ff"})
    rng = random.Random(48)
    # smoke body: hunched mass
    c.ellipse(22, 27, 15, 13, "N")
    c.ellipse(17, 22, 11, 9, "N")
    poly(c, [(8, 34), (36, 34), (40, 47), (33, 43), (27, 47), (21, 43), (15, 47), (10, 43), (5, 47)], "N")
    # smoke wisps rising from the back
    for (x, y, r) in ((6, 18, 3), (4, 25, 2.5), (9, 12, 2.5), (3, 33, 2.5), (12, 9, 2)):
        c.ellipse(x, y, r, r, "l")
    c.ellipse(16, 18, 9, 6, "L")
    c.ellipse(14, 16, 6, 3, "l")
    for _ in range(40):
        x, y = rng.randint(6, 38), rng.randint(16, 44)
        if c.g[y][x] == "N":
            c.g[y][x] = rng.choice("ln")
    shade_right(c, "N", "n", 0.85)
    # bone shoulder plates / spines along back
    for k, (x, y) in enumerate(((9, 15), (13, 12), (18, 11))):
        poly(c, [(x, y + 3), (x - 2, y - 3), (x + 3, y + 2)], "b")
    c.ellipse(29, 21, 6, 3.5, "B")
    c.rect(24, 22, 34, 23, "b")
    # hollow chest with blue core
    c.ellipse(24, 30, 7, 6.5, "o")
    c.ellipse(24, 30, 6, 5.5, "X")
    for y in (26, 30, 34):  # ribs crossing the hole
        c.rect(18, y, 30, y, "b")
    c.rect(18, 26, 30, 26, "B")
    c.ellipse(24, 30, 3, 3, "c")
    c.ellipse(24, 30, 2.2, 2.2, "C")
    c.ellipse(24, 30, 1.2, 1.2, "W")
    c.px(24, 26, "w"); c.px(24, 34, "w"); c.px(20, 30, "c"); c.px(28, 30, "c")
    # arms: smoky with long bone claws reaching forward-right
    poly(c, [(31, 23), (38, 26), (42, 34), (38, 36), (34, 30)], "N")
    c.ellipse(41, 35, 3, 2.5, "n")
    for k in range(4):
        line(c, 39 + k, 37, 40 + k * 1.5, 42 + (k % 2), "B")
    line(c, 43, 36, 46, 40, "B")
    poly(c, [(10, 26), (6, 32), (5, 40), (9, 40), (12, 32)], "l")
    for k in range(3):
        line(c, 5 + k * 2, 40, 4 + k * 2, 44, "b")
    # head: big bone mask, facing right
    c.ellipse(32, 13, 8, 7, "B")
    poly(c, [(33, 15), (44, 14), (46, 18), (42, 21), (32, 21)], "B")  # snout
    shade_right(c, "B", "b", 0.8)
    c.rect(28, 18, 44, 18, "o")
    for x in range(33, 45, 2):  # teeth
        c.rect(x, 19, x, 20, "B")
        c.px(x + 1, 19, "X")
    c.rect(33, 21, 42, 21, "o")
    poly(c, [(29, 10), (34, 9), (36, 12), (31, 13)], "X")  # eye socket
    c.rect(32, 10, 34, 11, "E"); c.px(33, 10, "W")
    poly(c, [(38, 11), (41, 11), (41, 13), (38, 13)], "X")
    c.px(40, 12, "E")
    line(c, 28, 7, 31, 13, "o"); line(c, 31, 13, 29, 17, "o")   # mask crack
    c.px(30, 14, "C")
    for y in range(14, 18):
        c.px(26 + (y % 2), y, "o")
    # crown of horns
    for (bx, tx, ty) in ((25, 19, 0), (29, 27, 1), (33, 34, 0), (37, 42, 1), (40, 47, 6)):
        poly(c, [(bx - 2, 9), (tx, ty), (bx + 2, 9)], "B")
        line(c, bx + 1, 8, tx + (1 if tx > bx else 0), ty + 2, "b")
    c.rect(25, 7, 40, 8, "o")
    c.save("boss", "W3 boss: The Hollow Monarch", outline=True)


def final():
    # Seraph of the Pale Throne: six white wings, halo of blades, glowing blue eyes, throne fragment
    c = Canvas(48, 48, {"W": "#eef1f8", "w": "#c2cadb", "v": "#8e98b2", "u": "#5f6884",
                        "R": "#f6f8fc", "r": "#b9c2d6", "S": "#80869c", "s": "#5c6276", "d": "#3c4052", "L": "#a9aec0",
                        "Y": "#ffe9a8", "y": "#c8a860", "H": "#dfe8ff", "h": "#6f98d8",
                        "E": "#5ad8ff", "X": "#1a1f33", "F": "#f0e4d8", "f": "#c8b8b0", "G": "#3d8ee8"})
    cx = 24
    # throne fragment (broken dark-pale stone back + stepped seat) so the white figure pops
    poly(c, [(13, 22), (16, 17), (19, 20), (22, 15), (26, 18), (29, 14), (33, 19), (35, 22), (35, 40), (13, 40)], "S")
    c.rect(13, 22, 15, 40, "s"); c.rect(33, 22, 35, 40, "s")
    c.rect(8, 38, 40, 42, "S"); c.rect(8, 38, 40, 38, "L")
    c.rect(6, 43, 42, 47, "s"); c.rect(6, 43, 42, 43, "S")
    c.rect(6, 46, 42, 47, "d")
    line(c, 30, 43, 33, 47, "d"); line(c, 14, 39, 12, 42, "d"); line(c, 38, 39, 40, 42, "d")
    c.rect(10, 40, 12, 41, "G"); c.px(36, 45, "G"); c.px(20, 44, "G")  # glowing sigil fragments

    def wing(root, tip, flen, cols, lead, n=6):
        (rx, ry), (tx, ty) = root, tip
        dx, dy = tx - rx, ty - ry
        L = math.hypot(dx, dy)
        ax, ay = dx / L, dy / L
        nx, ny = -ay, ax
        if ny < 0:
            nx, ny = -nx, -ny  # feathers hang downward
        for k in range(1, n + 1):
            t = k / n
            bx, by = rx + dx * t, ry + dy * t
            ln = flen * (0.55 + 0.6 * t)
            fx, fy = nx + ax * 0.55 * t, ny + ay * 0.55 * t
            m = math.hypot(fx, fy)
            fx, fy = fx / m, fy / m
            ex, ey = bx + fx * ln, by + fy * ln
            poly(c, [(bx - ax * 2.2, by - ay * 2.2), (bx + ax * 1.6, by + ay * 1.6),
                     (ex + ax * 0.6, ey + ay * 0.6), (ex - ax * 1.0, ey - ay * 1.0)], cols[k % 2])
            line(c, bx, by, ex - fx * 0.5, ey - fy * 0.5, cols[2])
        line(c, rx, ry, tx, ty, lead, 2)
        c.px(round(tx), round(ty), "R")

    # six wings, back-to-front: low pair, raised pair, wide middle pair
    for side in (-1, 1):
        wing((cx + side * 3, 26), (cx + side * 20, 33), 6, ("v", "u", "u"), "w", 5)
    for side in (-1, 1):
        wing((cx + side * 3, 18), (cx + side * 21, 2), 8, ("w", "v", "v"), "W", 6)
    for side in (-1, 1):
        wing((cx + side * 3, 21), (cx + side * 23, 17), 9, ("W", "w", "v"), "R", 6)
    # robed body seated on the throne
    poly(c, [(19, 21), (29, 21), (31, 39), (17, 39)], "R")
    poly(c, [(25, 22), (29, 21), (31, 39), (26, 39)], "r")
    c.rect(17, 35, 32, 39, "R"); c.rect(27, 35, 32, 39, "r")
    c.rect(17, 39, 32, 39, "h")                      # blue-trimmed hem
    c.rect(24, 28, 24, 38, "h")                      # front trim
    for x in (20, 22, 28, 30):
        c.rect(x, 31, x, 38, "w")
    c.rect(19, 25, 29, 25, "y"); c.rect(23, 24, 25, 27, "Y")  # gold clasp
    c.px(24, 25, "E")
    # arms resting, hands on knees
    c.rect(17, 22, 18, 32, "w"); c.rect(30, 22, 31, 32, "v")
    c.rect(17, 33, 19, 34, "F"); c.rect(29, 33, 31, 34, "F")
    # head with veil
    c.ellipse(cx, 16, 3.5, 4, "F")
    c.rect(21, 18, 27, 19, "F")
    c.rect(26, 15, 27, 19, "f")
    c.rect(20, 11, 28, 13, "W")
    c.rect(19, 13, 21, 21, "W"); c.rect(27, 13, 29, 21, "w")
    c.rect(21, 12, 27, 12, "h")                      # circlet
    c.px(24, 12, "E")
    c.rect(22, 16, 23, 16, "E"); c.rect(25, 16, 26, 16, "E")  # glowing eyes
    c.px(22, 15, "H"); c.px(26, 15, "H")
    c.rect(23, 19, 25, 19, "f")
    c.rect(20, 20, 28, 21, "R")                      # collar
    # halo of blades: gold ring behind the head with outward blade points
    for y in range(c.h):
        for x in range(c.w):
            d = math.hypot(x + 0.5 - cx, y + 0.5 - 15.5)
            if 7.5 <= d < 8.5 and y < 20 and c.g[y][x] in ".vwWuRrh":
                c.g[y][x] = "Y" if (x + y) % 3 else "y"
    for i in range(9):
        a = math.pi + i * math.pi / 8
        bx, by = cx + 8.5 * math.cos(a), 15.5 + 8.5 * math.sin(a)
        tx_, ty_ = cx + 13.5 * math.cos(a), 15.5 + 13.5 * math.sin(a)
        px_, py_ = -math.sin(a) * 1.1, math.cos(a) * 1.1
        poly(c, [(bx + px_, by + py_), (tx_, ty_), (bx - px_, by - py_)], "H")
        line(c, bx, by, (bx + tx_) / 2, (by + ty_) / 2, "h")
    c.save("final", "W3 final boss: Seraph of the Pale Throne", outline=True)


def relic():
    import gen_icons
    from gen_icons import Icon
    i = Icon({"B": "#e6e1d2", "b": "#b3ad9a", "n": "#7e7968", "C": "#5ab4ff", "W": "#dff4ff", "X": "#14121c"})
    i.poly([(3, 2), (10, 1), (14, 4), (13, 9), (11, 14), (6, 15), (2, 10), (1, 5)], "B")
    i.poly([(10, 1), (14, 4), (13, 9), (11, 14), (9, 13), (11, 8)], "b")
    i.poly([(4, 5), (8, 5), (8, 8), (5, 8)], "X")       # eye hole
    i.px(6, 6, "C")
    i.line(10, 1, 9, 5, "C"); i.line(9, 5, 11, 8, "C"); i.line(11, 8, 8, 11, "C"); i.line(8, 11, 9, 15, "C")
    i.px(9, 6, "W"); i.px(10, 8, "W"); i.px(9, 10, "W")
    i.line(3, 11, 6, 12, "n")
    i.save("relic_w3", "Relic: Pale Mask Shard")
    _ = gen_icons


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    ground()
    props()
    enemies()
    relic()
    print("wrote", len(list(OUT.glob("*.px"))), "w3 sprites to", OUT)


if __name__ == "__main__":
    main()
