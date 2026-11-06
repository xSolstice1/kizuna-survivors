"""Generates World 2 (Ember Leaf Frontier) tiles, props, enemies and relic icon as .px sources.

    python3 tools/gen_w2.py

A war-torn ninja frontier forest at night: ash-strewn forest floor with fallen maple
leaves, scorched craters, dirt trails with plank sections, signal torches, supply
crates, stake palisades, smouldering stumps, and an original ninja-war cast.

Writes art/src/w2/*.px and art/src/icons/relic_w2.px. Deterministic (seeded).
"""
import math
import pathlib
import random
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_stage  # noqa: E402
from gen_stage import Canvas  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent.parent / "art/src/w2"
gen_stage.OUT = OUT


class C(Canvas):
    """Canvas with a few extra primitives."""

    def wpx(self, x, y, c):  # wrap-around pixel, for seamless tiles
        self.g[y % self.h][x % self.w] = c

    def wellipse(self, cx, cy, rx, ry, c):
        for y in range(int(cy - ry) - 1, int(cy + ry) + 2):
            for x in range(int(cx - rx) - 1, int(cx + rx) + 2):
                if ((x + 0.5 - cx) / rx) ** 2 + ((y + 0.5 - cy) / ry) ** 2 <= 1:
                    self.wpx(x, y, c)

    def poly(self, pts, c):
        for y in range(self.h):
            for x in range(self.w):
                px, py, inside = x + 0.5, y + 0.5, False
                for i in range(len(pts)):
                    (x1, y1), (x2, y2) = pts[i], pts[i - 1]
                    if (y1 > py) != (y2 > py) and px < (x2 - x1) * (py - y1) / (y2 - y1) + x1:
                        inside = not inside
                if inside:
                    self.g[y][x] = c

    def line(self, x0, y0, x1, y1, c, w=1):
        steps = int(max(abs(x1 - x0), abs(y1 - y0)) * 2) + 1
        for i in range(steps + 1):
            t = i / steps
            x, y = round(x0 + (x1 - x0) * t), round(y0 + (y1 - y0) * t)
            for dy in range(w):
                for dx in range(w):
                    self.px(x + dx - w // 2, y + dy - w // 2, c)

    def get(self, x, y):
        if 0 <= x < self.w and 0 <= y < self.h:
            return self.g[y][x]
        return None

    def recolor(self, src, dst, region=None):
        x0, y0, x1, y1 = region or (0, 0, self.w - 1, self.h - 1)
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                if self.get(x, y) == src:
                    self.g[y][x] = dst


# ───────────────────────── Ground ─────────────────────────
GROUND = {
    "A": "#271c1b", "B": "#2e211e", "D": "#211818", "H": "#352622",   # forest floor
    "a": "#3a3332", "b": "#2f2a2b",                                   # ash
    "R": "#4e2019", "O": "#62301a", "r": "#3c1d18", "Y": "#6c4420",   # fallen maple leaves
    "E": "#a8461c",                                                   # dim ember
}

LEAVES = [
    [".X.", "XXX", "X.X"],
    ["X.X", ".X.", "XXX", ".X."],
    [".X.", "XXX", ".X."],
    ["XX", "XX"],
    ["X..", "XXX", "..X"],
]


def floor(c, rng, leaves=14, ash=12, embers=0):
    c.rect(0, 0, 31, 31, "A")
    for _ in range(9):
        c.wellipse(rng.uniform(0, 32), rng.uniform(0, 32), rng.uniform(2, 5), rng.uniform(1.5, 3.5),
                   rng.choice("BBDH"))
    for _ in range(90):
        c.wpx(rng.randrange(32), rng.randrange(32), rng.choice("ABDDB"))
    for _ in range(ash):  # little ash drifts
        x, y = rng.randrange(32), rng.randrange(32)
        for _ in range(rng.randint(2, 5)):
            c.wpx(x, y, rng.choice("abb"))
            x += rng.choice((-1, 0, 1))
            y += rng.choice((0, 0, 1))
    for _ in range(leaves):
        pat = rng.choice(LEAVES)
        col = rng.choice("RROrrY")
        x0, y0 = rng.randrange(32), rng.randrange(32)
        for j, row in enumerate(pat):
            for i, ch in enumerate(row):
                if ch == "X":
                    c.wpx(x0 + i, y0 + j, col)
        if rng.random() < 0.5:
            c.wpx(x0 + 1, y0 + len(pat), "r")  # stem
    for _ in range(embers):
        c.wpx(rng.randrange(32), rng.randrange(32), "E")


def path_tile(c, rng):
    c.pal.update({"P": "#3b2b22", "p": "#33251e", "q": "#45332a", "W": "#4a3322", "w": "#3a2618",
                  "v": "#56402a", "g": "#1c1414", "n": "#6a6260"})
    floor(c, rng, leaves=6, ash=4)
    # Cyclic edge jitter so the column tiles vertically.
    phase = rng.uniform(0, 6.28)
    for y in range(32):
        jl = round(1.4 * math.sin(y / 32 * 2 * math.pi * 2 + phase) + 0.8 * math.sin(y / 32 * 2 * math.pi * 3))
        jr = round(1.4 * math.sin(y / 32 * 2 * math.pi * 2 + phase + 2) + 0.8 * math.cos(y / 32 * 2 * math.pi * 3))
        xl, xr = 6 + jl, 25 + jr
        c.rect(xl, y, xr, y, "P")
        c.px(xl - 1, y, "p")
        c.px(xr + 1, y, "p")
    for _ in range(70):
        x, y = rng.randint(8, 23), rng.randrange(32)
        c.px(x, y, rng.choice("ppqq"))
    for _ in range(5):
        x, y = rng.randint(8, 23), rng.randrange(32)
        c.px(x, y, "n")  # pebbles
    # Wooden plank section (3 planks across the trail).
    for k, y0 in enumerate((17, 21, 25)):
        xl, xr = 7 + rng.randint(-1, 1), 24 + rng.randint(-1, 1)
        c.rect(xl, y0, xr, y0 + 2, "W")
        c.rect(xl, y0, xr, y0, "v")
        c.rect(xl, y0 + 3, xr, y0 + 3, "g")
        for x in range(xl + 2, xr - 1, 5):
            c.px(x + rng.randint(0, 2), y0 + 1, "w")  # grain
        c.px(xl + 1, y0 + 1, "n")
        c.px(xr - 1, y0 + 1, "n")  # nails
    c.rect(7, 16, 24, 16, "g")
    # a leaf or two blown onto the trail
    for _ in range(2):
        x, y = rng.randint(9, 22), rng.randint(2, 12)
        c.px(x, y, "R"); c.px(x + 1, y, "O"); c.px(x, y + 1, "R")


def crater(c, rng):
    c.pal.update({"X": "#181213", "x": "#110c0e", "s": "#463c3a", "F": "#ff8a2a", "f": "#ffc456",
                  "e": "#d0541e"})
    c.ellipse(16, 16.5, 12, 9.5, "a")
    c.ellipse(16, 16.5, 11, 8.5, "s")
    c.ellipse(16, 16.5, 10, 7.5, "X")
    c.ellipse(16, 17, 7, 5, "x")
    for _ in range(25):
        a = rng.uniform(0, 2 * math.pi)
        r = rng.uniform(11.5, 14)
        c.px(round(16 + math.cos(a) * r), round(16.5 + math.sin(a) * r * 0.78), rng.choice("abs"))
    # glowing cracks radiating from the centre
    for a in (0.3, 1.5, 2.6, 3.7, 4.9):
        x, y = 16.0, 17.0
        for t in range(rng.randint(5, 8)):
            x += math.cos(a) * 1.0 + rng.uniform(-0.4, 0.4)
            y += math.sin(a) * 0.7 + rng.uniform(-0.3, 0.3)
            c.px(round(x), round(y), "e" if t > 2 else "F")
    for x, y in ((15, 17), (17, 16), (16, 18), (13, 15), (19, 19)):
        c.px(x, y, "f")
    c.px(16, 17, "F")
    for _ in range(6):
        c.px(rng.randint(8, 24), rng.randint(11, 22), rng.choice("eF"))


# ───────────────────────── Props ─────────────────────────
WOOD = {"W": "#6a4a2e", "w": "#4a3220", "v": "#8a6440", "u": "#2e2016"}
FIRE = {"F": "#ff8a2a", "f": "#ffd060", "Z": "#fff4c0", "e": "#c8461c"}


def prop_a():
    # Signal lantern pole: a tall post with a cross-arm, a caged paper lantern and a torch flame on top.
    c = C(16, 32, {**WOOD, **FIRE, "L": "#e8a050", "l": "#c06a2a", "M": "#5a5660", "S": "#4a4448",
                   "s": "#38333a", "R": "#a02a24"})
    c.rect(7, 6, 8, 29, "W")
    c.rect(8, 6, 8, 29, "w")
    for y in (12, 18, 24):
        c.px(7, y, "v")
    # cross arm + hanging lantern
    c.rect(4, 9, 12, 9, "w")
    c.rect(4, 8, 12, 8, "W")
    # caged fire bowl at top
    c.rect(4, 5, 11, 7, "M")
    c.rect(5, 7, 10, 7, "S")
    c.poly([(5, 5), (7, 0.5), (8, -0.5), (9, 1), (11, 5)], "F")
    c.poly([(6.5, 5), (7.8, 1.8), (9.6, 5)], "f")
    c.px(8, 4, "Z")
    c.px(5, 2, "e"); c.px(11, 2, "F")
    # paper lantern hanging from the right arm
    c.px(11, 10, "u")
    c.rect(10, 11, 13, 16, "L")
    c.rect(10, 11, 13, 11, "u"); c.rect(10, 16, 13, 16, "u")
    c.rect(11, 12, 12, 15, "f")
    c.px(10, 13, "l"); c.px(13, 13, "l")
    c.rect(11, 14, 12, 14, "R")  # signal mark
    # rope wrap
    c.rect(7, 20, 8, 20, "v"); c.rect(7, 22, 8, 22, "v")
    # base: stone footing, 8 wide, 4 deep
    c.rect(4, 28, 11, 31, "S")
    c.rect(4, 28, 11, 28, "M")
    c.rect(5, 31, 11, 31, "s")
    c.px(6, 29, "s"); c.px(9, 30, "s")
    c.save("prop_a", "Signal lantern pole with torch (W2)", outline=True)


def prop_b():
    # Supply crates and a barrel, lashed with rope.
    c = C(32, 32, {**WOOD, "T": "#7a5434", "t": "#5a3c24", "Q": "#94704a", "N": "#c8a870", "n": "#8e7448",
                   "I": "#4a4650", "i": "#6e6a74", "R": "#8a2a22", "M": "#a8a4ac"})
    # barrel (left)
    c.ellipse(9, 22, 6.5, 9.5, "T")
    c.rect(3, 13, 15, 31, ".")
    c.rect(4, 14, 14, 31, "T")
    c.rect(3, 16, 15, 29, "T")
    c.rect(11, 14, 14, 31, "t")
    c.rect(5, 14, 5, 30, "Q")
    for y in (16, 22, 28):
        c.rect(3, y, 15, y, "I")
        c.px(5, y, "i")
    c.rect(5, 13, 13, 13, "u")
    c.rect(6, 12, 12, 12, "t")
    # big crate (right, bottom)
    c.rect(14, 19, 29, 31, "W")
    c.rect(14, 19, 29, 19, "v")
    c.rect(14, 19, 14, 31, "w"); c.rect(29, 19, 29, 31, "w")
    c.rect(14, 25, 29, 25, "w")
    c.line(15, 20, 28, 30, "w")
    c.line(15, 30, 28, 20, "v")
    # small crate stacked on top
    c.rect(16, 9, 27, 18, "T")
    c.rect(16, 9, 27, 9, "Q")
    c.rect(16, 9, 16, 18, "t"); c.rect(27, 9, 27, 18, "t")
    c.rect(16, 13, 27, 14, "t")
    c.rect(19, 15, 24, 17, "u")
    c.rect(20, 16, 23, 16, "R")  # stamped clan mark
    # rope lashing
    c.line(22, 8, 22, 31, "N")
    c.rect(14, 22, 29, 22, "N")
    for y in range(9, 31, 3):
        c.px(22, y, "n")
    for x in range(15, 29, 3):
        c.px(x, 22, "n")
    c.rect(20, 7, 24, 8, "N")
    c.px(25, 8, "n"); c.px(26, 9, "N")
    # a coil of rope and a kunai stuck in the crate
    c.ellipse(3.5, 30.5, 2.5, 1.5, "N")
    c.px(3, 30, "n")
    c.line(28, 15, 30, 13, "M")
    c.px(27, 16, "u")
    c.save("prop_b", "Supply crates and barrel with rope (W2)", outline=True)


def prop_c():
    # Palisade of sharpened stakes with rope bindings, some tips charred and smouldering.
    rng = random.Random(22)
    c = C(48, 32, {**WOOD, **FIRE, "N": "#b89a64", "n": "#7e6640", "X": "#2a2224", "x": "#3e3436",
                   "S": "#3a3440"})
    xs = list(range(4, 44, 5))
    for i, x0 in enumerate(xs):
        top = 4 + rng.randint(0, 6) + (3 if i % 2 else 0)
        tip = top - 4
        c.rect(x0, top, x0 + 3, 30, "W")
        c.rect(x0 + 3, top, x0 + 3, 30, "w")
        c.rect(x0, top, x0, 30, "v")
        c.poly([(x0, top + 0.5), (x0 + 1.6, tip), (x0 + 2.4, tip), (x0 + 4, top + 0.5)], "W")
        c.px(x0 + 2, tip + 1, "v")
        for y in range(top + 3, 30, 4):
            c.px(x0 + 1 + rng.randint(0, 1), y, "w")
        if rng.random() < 0.45:  # charred tip
            for y in range(tip, top + 3):
                for x in range(x0, x0 + 4):
                    if c.get(x, y) not in (".", None):
                        c.px(x, y, "X" if (x + y) % 3 else "x")
            c.px(x0 + 1, top, "F")
            c.px(x0 + 2, top + 1, "e")
            if rng.random() < 0.6:
                c.px(x0 + 2, tip + 1, "f")
    # two horizontal crossbeams bound with rope
    for y in (13, 23):
        c.rect(2, y, 45, y + 1, "w")
        c.rect(2, y, 45, y, "W")
        for x0 in xs:
            c.rect(x0 + 1, y - 1, x0 + 2, y + 2, "N")
            c.px(x0 + 2, y, "n")
    # grounded base: packed earth
    c.rect(3, 30, 44, 31, "S")
    c.rect(3, 30, 44, 30, "u")
    c.save("prop_c", "Palisade barricade of sharpened stakes (W2)", outline=True)


def prop_d():
    # Burnt tree stump with glowing ember cracks and spreading roots.
    rng = random.Random(23)
    c = C(24, 24, {**FIRE, "X": "#2a2224", "x": "#3a3032", "y": "#4c3e3a", "s": "#1e181a",
                   "W": "#5a3e2a", "a": "#6a6466"})
    c.rect(6, 7, 17, 21, "X")
    c.rect(6, 7, 7, 21, "x")
    c.rect(16, 7, 17, 21, "s")
    # roots
    c.poly([(6, 17), (2, 22), (6, 22), (8, 19)], "X")
    c.poly([(17, 16), (21, 22), (16, 22), (15, 19)], "s")
    c.poly([(10, 19), (8, 23), (14, 23), (13, 19)], "X")
    c.rect(3, 22, 20, 22, "X")
    # jagged broken top
    c.poly([(6, 8), (7, 3), (9, 6), (11, 1), (13, 6), (15, 4), (17, 8)], "X")
    c.poly([(6, 8), (7, 3), (8, 7)], "x")
    c.px(11, 2, "y"); c.px(11, 3, "y")
    # inner ring top (exposed wood, burnt)
    c.rect(8, 7, 15, 8, "W")
    c.rect(10, 7, 13, 7, "e")
    c.px(11, 8, "F")
    # glowing bark cracks
    for x, y0, n in ((9, 10, 6), (13, 12, 6), (15, 9, 3)):
        x1 = x
        for t in range(n):
            c.px(x1, y0 + t, "F" if t % 2 else "e")
            x1 += rng.choice((-1, 0, 0, 1))
    c.px(9, 12, "f"); c.px(13, 14, "f"); c.px(12, 20, "e")
    # bark texture
    for _ in range(12):
        x, y = rng.randint(6, 17), rng.randint(9, 21)
        if c.get(x, y) == "X":
            c.px(x, y, "y" if rng.random() < 0.3 else "x")
    # ash and rising embers
    c.px(4, 21, "a"); c.px(19, 21, "a")
    c.px(4, 2, "F"); c.px(18, 1, "f"); c.px(15, 0, "e")
    c.save("prop_d", "Burnt tree stump with embers (W2)", outline=True)


# ───────────────────────── Enemies ─────────────────────────
def fodder_a():
    # Straw Puppet Soldier: a straw training dummy animated, ragged cloth, wooden spear.
    c = C(16, 16, {"Y": "#d0a650", "y": "#9a7434", "j": "#ecd08a", "S": "#b8955c", "s": "#86683c",
                   "C": "#8a2c26", "c": "#5c1c1a", "E": "#ff4a2a", "W": "#6a4a2a", "M": "#c8ccd4",
                   "k": "#3a2a1a"})
    # spear behind
    c.line(13, 2, 13, 13, "W")
    c.px(13, 1, "M"); c.px(13, 0, "M")
    # head: stitched sack
    c.ellipse(8, 4, 3.6, 3.2, "S")
    c.rect(5, 5, 10, 6, "s")
    c.rect(5, 2, 11, 5, "S")
    c.px(6, 1, "j"); c.px(8, 0, "j"); c.px(10, 1, "j"); c.px(4, 2, "j")
    c.px(9, 4, "E"); c.px(11, 4, "E")
    c.px(10, 6, "k"); c.px(9, 6, "k"); c.px(11, 6, "k")
    c.px(7, 3, "s")  # stitch
    # straw arms
    c.rect(3, 8, 12, 8, "Y")
    c.px(2, 8, "j"); c.px(2, 9, "y")
    c.px(12, 7, "Y"); c.px(12, 9, "y")
    # straw body + ragged tabard
    c.rect(5, 7, 10, 12, "Y")
    c.rect(6, 7, 10, 11, "C")
    c.rect(9, 7, 10, 11, "c")
    c.px(6, 12, "C"); c.px(8, 12, "c"); c.px(10, 12, "C")
    c.rect(5, 9, 10, 9, "y")  # rope belt
    c.px(5, 10, "j"); c.px(5, 11, "j")
    # straw legs
    c.rect(6, 12, 7, 14, "Y"); c.px(6, 14, "y")
    c.rect(9, 13, 10, 14, "Y"); c.px(10, 14, "y")
    c.px(5, 14, "j"); c.px(11, 14, "j")
    c.save("fodder_a", "Straw Puppet Soldier (W2 fodder)", outline=True)


def fodder_b():
    # Leaf Shinobi: lean masked ninja in green-brown wrappings, crouched to dash.
    c = C(16, 16, {"G": "#5a6a34", "g": "#3c4824", "h": "#7e8c46", "k": "#1e2016", "E": "#f8f080",
                   "R": "#b05a2a", "r": "#7a3a1c", "M": "#c8ccd4", "B": "#3a2a1c"})
    # trailing head-wrap tail
    c.px(5, 3, "R"); c.px(4, 3, "R"); c.px(3, 4, "r"); c.px(2, 4, "R"); c.px(1, 5, "r")
    # head
    c.ellipse(9.5, 4.5, 3.2, 3.2, "G")
    c.rect(7, 2, 11, 2, "h")
    c.rect(8, 4, 12, 4, "k")
    c.px(10, 4, "E"); c.px(12, 4, "E")
    c.rect(9, 6, 12, 6, "g")
    c.px(6, 4, "R"); c.px(6, 5, "r")
    # crouched torso
    c.poly([(4, 7.5), (11, 6.5), (12, 11), (5, 11.5)], "G")
    c.rect(4, 8, 7, 11, "g")
    c.rect(6, 7, 10, 7, "h")
    c.line(6, 10, 11, 9, "B")  # sash
    # forward arm with short blade held low
    c.line(11, 8, 13, 10, "G")
    c.px(14, 10, "B")
    c.line(14, 11, 15, 12, "M")
    # back leg extended, front leg bent
    c.rect(2, 12, 6, 13, "g")
    c.rect(1, 14, 3, 14, "k")
    c.rect(8, 11, 10, 13, "G")
    c.rect(9, 11, 10, 12, "h")
    c.rect(8, 14, 11, 14, "k")
    c.save("fodder_b", "Leaf Shinobi (W2 fodder)", outline=True)


def ranged():
    # Kunai Thrower: ninja with a raised throwing hand and a bandolier of blades.
    c = C(16, 16, {"N": "#4a4a6e", "n": "#30304a", "h": "#6a6a92", "S": "#e0b894", "E": "#ffe060",
                   "R": "#c0302a", "r": "#801c1c", "L": "#7a5030", "M": "#d4dae4", "W": "#ffffff",
                   "k": "#18141e"})
    # hood tails
    c.px(3, 3, "R"); c.px(2, 4, "R"); c.px(1, 4, "r"); c.px(2, 5, "r")
    # head
    c.ellipse(7, 4.2, 3.2, 3.2, "N")
    c.rect(5, 2, 9, 2, "R")
    c.rect(4, 2, 4, 3, "r")
    c.rect(7, 4, 10, 4, "S")
    c.px(9, 4, "E")
    c.rect(7, 5, 10, 5, "n")
    # body
    c.rect(4, 7, 9, 12, "N")
    c.rect(4, 7, 5, 12, "n")
    c.rect(6, 7, 9, 7, "h")
    # bandolier with blades
    c.line(4, 7, 9, 12, "L")
    c.px(5, 8, "M"); c.px(7, 10, "M"); c.px(9, 12, "M")
    # back arm
    c.rect(2, 8, 3, 11, "n")
    c.px(2, 11, "S")
    # raised throwing arm
    c.line(9, 7, 11, 4, "N", w=2)
    c.px(12, 3, "S"); c.px(11, 2, "S")
    c.line(12, 2, 14, 0, "M")
    c.px(14, 0, "W")
    c.px(13, 3, "M"); c.px(14, 3, "M")  # second kunai between fingers
    # legs
    c.rect(4, 13, 5, 14, "n")
    c.rect(8, 13, 9, 14, "n")
    c.rect(3, 14, 5, 14, "k"); c.rect(8, 14, 10, 14, "k")
    c.save("ranged", "Kunai Thrower (W2 ranged)", outline=True)


def blinker():
    # Smoke Ninja: a ninja half-dissolved into grey smoke.
    rng = random.Random(24)
    c = C(16, 16, {"D": "#3a3646", "d": "#26222e", "h": "#56506a", "E": "#8af0ff", "e": "#d8fcff",
                   "S": "#8a8898", "s": "#5e5c6e", "L": "#bebccc"})
    # smoke body
    for cx, cy, rx, ry, col in ((6, 11, 3.5, 2.5, "s"), (10, 12, 3.2, 2.2, "S"), (7, 13.5, 4.5, 1.6, "S"),
                                (4, 9, 2.5, 2, "s"), (11, 14, 2.5, 1, "s")):
        c.ellipse(cx, cy, rx, ry, col)
    # head + torso (the right/front half is still solid)
    c.ellipse(9, 4.5, 3.2, 3.2, "D")
    c.rect(7, 2, 10, 2, "h")
    c.rect(8, 4, 12, 4, "d")
    c.px(10, 4, "E"); c.px(12, 4, "e")
    c.poly([(6, 7), (12, 7), (12, 11), (7, 11)], "D")
    c.rect(10, 7, 12, 10, "h")
    c.line(12, 8, 14, 10, "D")
    c.px(14, 11, "E")  # glowing fingertip seal
    # dissolve the back half into wisps
    for y in range(16):
        for x in range(16):
            ch = c.get(x, y)
            if ch in ("D", "h", "d") and x < 8 and rng.random() < 0.45 + (8 - x) * 0.05:
                c.px(x, y, rng.choice("SsL"))
            elif ch in ("S", "s") and rng.random() < 0.18:
                c.px(x, y, ".")
    for x, y in ((2, 5), (1, 7), (3, 6), (13, 13), (2, 12)):
        c.px(x, y, "L")
    c.save("blinker", "Smoke Ninja (W2 blinker)", outline=True)


def elite():
    # Iron Mask Ronin: hulking armoured ronin, iron demon mask, a huge blade.
    c = C(32, 32, {"A": "#2e2a38", "a": "#4a4458", "q": "#625a74", "H": "#4a3e3a", "h": "#342a28",
                   "I": "#767684", "i": "#aeaebe", "J": "#4a4a56", "B": "#e0d4b4", "E": "#ff4a2a",
                   "R": "#a02a2a", "M": "#d0d6e2", "m": "#8a92a4", "G": "#c89a3a", "D": "#1e1a24",
                   "S": "#c09474", "k": "#14101a"})
    # wild dark mane behind
    c.poly([(11, 3), (5, 6), (3, 12), (7, 11), (6, 15), (11, 12)], "D")
    # legs: wide hakama
    c.poly([(9, 22), (22, 22), (25, 29), (17, 29), (15.5, 25), (14, 29), (6, 29)], "H")
    c.poly([(9, 22), (13, 22), (11, 29), (6, 29)], "h")
    c.rect(5, 29, 10, 30, "k"); c.rect(18, 29, 25, 30, "k")
    c.rect(6, 29, 9, 29, "J"); c.rect(19, 29, 24, 29, "J")
    # torso armour (do)
    c.rect(8, 12, 22, 22, "A")
    for y in (14, 17, 20):
        c.rect(8, y, 22, y, "a")
        for x in range(9, 22, 3):
            c.px(x, y + 1, "R")
    c.rect(8, 22, 22, 22, "G")
    # shoulder guards
    c.rect(4, 11, 10, 18, "a")
    c.rect(4, 11, 10, 11, "q")
    for y in (13, 16):
        c.rect(4, y, 10, y, "A")
        c.px(5, y + 1, "R"); c.px(8, y + 1, "R")
    c.rect(20, 11, 26, 16, "a")
    c.rect(20, 11, 26, 11, "q")
    c.rect(20, 14, 26, 14, "A")
    c.px(22, 15, "R"); c.px(25, 15, "R")
    # left arm
    c.rect(5, 18, 8, 21, "A")
    c.rect(5, 22, 8, 23, "S")
    # iron demon mask
    c.ellipse(16.5, 7, 5.2, 5.2, "I")
    c.rect(12, 4, 21, 4, "i")
    c.rect(13, 3, 20, 3, "i")
    c.rect(12, 5, 21, 5, "J")
    c.rect(14, 6, 16, 6, "k"); c.rect(18, 6, 20, 6, "k")
    c.px(15, 6, "E"); c.px(16, 6, "E"); c.px(19, 6, "E"); c.px(20, 6, "E")
    c.rect(17, 7, 17, 8, "J")
    c.rect(14, 10, 20, 10, "k")
    c.px(15, 10, "B"); c.px(19, 10, "B"); c.px(15, 11, "B"); c.px(19, 11, "B")
    c.rect(13, 8, 13, 9, "J"); c.rect(21, 8, 21, 9, "J")
    # horns
    c.line(13, 3, 11, 0, "B"); c.px(12, 2, "B")
    c.line(20, 3, 22, 0, "B"); c.px(21, 2, "B")
    # right arm + huge blade
    c.rect(20, 16, 24, 20, "A")
    c.rect(21, 17, 24, 17, "a")
    c.line(21, 22, 24, 19, "R", w=2)
    c.rect(23, 18, 26, 19, "G")
    c.px(24, 18, "S"); c.px(25, 19, "S")
    c.line(25, 17, 30, 2, "M", w=2)
    c.line(26, 17, 31, 2, "m")
    c.line(25, 16, 29, 4, "i")
    c.px(30, 1, "M")
    c.save("elite", "Iron Mask Ronin (W2 elite)", outline=True)


def mini():
    # Kagerou, Twin-Blade Captain: elite ninja captain, two short blades, flowing scarf, red visor.
    c = C(32, 32, {"N": "#2a2e48", "n": "#1a1c2e", "h": "#424a6c", "P": "#5a6280", "p": "#8a94b4",
                   "R": "#d02a3a", "r": "#801424", "V": "#ff3a4a", "v": "#ffc0c0", "M": "#dae0ea",
                   "m": "#8a92a4", "W": "#ffffff", "G": "#c89a3a", "S": "#d8b090", "k": "#10101a"})
    # flowing scarf, trailing back to the left
    for i in range(16):
        x = 15 - i
        y = 10 + round(math.sin(i * 0.55) * 2.2) - i // 6
        c.rect(x, y, x, y + 2 - (i > 11), "R" if (i // 3) % 2 == 0 else "r")
    c.px(0, 7, "R"); c.px(1, 8, "r")
    for i in range(10):  # second, shorter tail
        x = 14 - i
        y = 13 + round(math.sin(i * 0.7 + 1) * 1.5)
        c.rect(x, y, x, y + 1, "r")
    # legs: dash stance
    c.line(19, 20, 23, 25, "N", w=3)
    c.line(23, 25, 24, 29, "N", w=3)
    c.rect(22, 25, 24, 28, "P")
    c.rect(22, 29, 26, 30, "k")
    c.line(15, 20, 12, 25, "n", w=3)
    c.line(12, 25, 9, 29, "n", w=3)
    c.rect(10, 25, 11, 27, "P")
    c.rect(6, 29, 10, 30, "k")
    # torso
    c.poly([(13, 10.5), (21.5, 10.5), (21, 20.5), (14, 20.5)], "N")
    c.rect(15, 12, 20, 16, "P")
    c.rect(15, 12, 20, 12, "p")
    c.rect(18, 13, 20, 13, "p")
    c.rect(14, 18, 21, 19, "R")
    c.px(17, 18, "G"); c.px(17, 19, "G")
    # shoulder plate
    c.rect(19, 10, 23, 12, "P")
    c.rect(19, 10, 23, 10, "p")
    # head: hood with metal brow plate and glowing visor
    c.ellipse(17.5, 5.5, 4.3, 4.6, "N")
    c.rect(14, 2, 20, 2, "h")
    c.rect(15, 3, 21, 3, "m")
    c.rect(16, 5, 22, 6, "V")
    c.rect(16, 5, 16, 6, "r")
    c.rect(19, 5, 21, 5, "v")
    c.px(22, 5, "W")
    c.rect(17, 7, 21, 9, "n")
    c.rect(19, 8, 21, 8, "h")
    # scarf wrap at neck
    c.rect(14, 9, 21, 10, "R")
    c.rect(14, 10, 21, 10, "r")
    # forward arm, blade thrust forward
    c.line(21, 13, 25, 16, "N", w=2)
    c.rect(24, 15, 25, 15, "P")
    c.px(26, 16, "S"); c.px(26, 15, "S")
    c.px(26, 14, "G"); c.px(27, 15, "G")
    c.line(27, 14, 30, 10, "M", w=2)
    c.line(27, 13, 30, 9, "W")
    c.px(31, 9, "m")
    # back arm, blade in reverse grip
    c.line(14, 12, 11, 17, "n", w=2)
    c.px(11, 18, "S"); c.px(10, 18, "S")
    c.px(10, 17, "G")
    c.line(10, 19, 5, 22, "M", w=2)
    c.line(9, 18, 5, 21, "m")
    c.save("mini", "Kagerou, Twin-Blade Captain (W2 mini-boss)", outline=True)


def boss():
    # Great Ember Toad: enormous war toad, saddle-shrine on its back, glowing warts, pipe smoke.
    rng = random.Random(25)
    c = C(48, 48, {"T": "#7a3424", "t": "#5a2418", "q": "#9a4a2e", "U": "#d0aa76", "u": "#9a7850",
                   "O": "#ff8a2a", "o": "#ffd060", "Y": "#ffd040", "k": "#160e10", "G": "#c89a3a",
                   "g": "#8a6424", "S": "#9a98a6", "s": "#6e6c7c", "D": "#2e2634", "d": "#46404e",
                   "Rr": "#b02a22", "C": "#7a1a2a", "c": "#541220", "W": "#5a3a22", "L": "#d8c08a",
                   "F": "#ff6a1a"})
    c.pal["R"] = c.pal.pop("Rr")
    # back haunch
    c.ellipse(11, 37, 9, 8, "t")
    # body
    c.ellipse(23, 33, 19, 12, "T")
    c.ellipse(21, 29, 15, 7, "q")
    c.ellipse(23, 36, 19, 9, "T")
    # head
    c.ellipse(35, 28, 11, 8, "T")
    c.ellipse(35, 25, 9, 4, "q")
    # belly / throat
    c.ellipse(32, 38, 11, 6.5, "U")
    c.ellipse(32, 40, 10, 4.5, "u")
    c.rect(24, 36, 40, 36, "U")
    # eye bulge
    c.ellipse(37, 19.5, 4.6, 4, "T")
    c.ellipse(37.5, 19.5, 3, 2.6, "Y")
    c.rect(36, 19, 39, 19, "k")
    c.px(38, 18, "o")
    c.rect(34, 16, 40, 16, "q")
    # mouth
    for x in range(25, 46):
        y = 31 - round(((x - 25) / 20) ** 1.6 * 3)
        c.px(x, y, "k")
        c.px(x, y + 1, "t")
    c.px(45, 26, "T"); c.px(46, 27, "T")
    c.px(43, 23, "k"); c.px(44, 23, "k")  # nostril
    # pipe (kiseru) from mouth corner with smoke
    c.line(41, 30, 45, 27, "G")
    c.line(42, 30, 45, 28, "g")
    c.rect(44, 24, 46, 26, "G")
    c.rect(44, 24, 46, 24, "g")
    c.px(45, 23, "O")
    for cx, cy, rx, ry in ((45, 20, 1.8, 1.4), (43.5, 16, 2.4, 1.8), (45, 11, 2.2, 1.6), (43, 7, 1.8, 1.3)):
        c.ellipse(cx, cy, rx, ry, "S")
        c.px(round(cx), round(cy) - 1, "s")
    # front leg
    c.poly([(33, 38), (40, 37), (43, 45), (31, 45)], "T")
    c.poly([(33, 38), (35, 38), (34, 45), (31, 45)], "t")
    for x in (32, 36, 40, 43):
        c.rect(x - 1, 45, x + 1, 46, "u")
    # back foot
    c.rect(3, 44, 16, 45, "t")
    c.rect(2, 45, 16, 46, "t")
    for x in (3, 8, 13):
        c.rect(x, 46, x + 2, 46, "u")
    # glowing warts
    for _ in range(26):
        x, y = rng.randint(4, 40), rng.randint(22, 42)
        if c.get(x, y) in ("T", "q", "t") and c.get(x + 1, y) in ("T", "q", "t"):
            c.px(x, y, "O")
            if rng.random() < 0.5:
                c.px(x + 1, y, "O")
                c.px(x, y - 1, "o")
    # saddle blanket with gold trim
    c.poly([(9, 22), (28, 22), (31, 30), (6, 30)], "C")
    c.poly([(9, 22), (12, 22), (9, 30), (6, 30)], "c")
    c.rect(7, 29, 30, 30, "G")
    for x in range(8, 30, 3):
        c.px(x, 31, "g")
    c.line(28, 25, 33, 34, "L")  # girth rope
    # shrine on the saddle
    c.rect(10, 19, 27, 22, "W")
    c.rect(10, 19, 27, 19, "L")
    c.rect(12, 11, 13, 19, "R")
    c.rect(24, 11, 25, 19, "R")
    c.rect(14, 12, 23, 18, "k")
    c.ellipse(18.5, 15.5, 2.2, 2.6, "O")
    c.rect(18, 15, 19, 16, "o")
    c.rect(14, 12, 23, 12, "L")
    for x in (15, 18, 21):
        c.px(x, 13, "L")  # shide streamers
    c.poly([(5, 11.5), (32, 11.5), (27, 6), (10, 6)], "D")
    c.rect(9, 7, 28, 8, "d")
    c.rect(11, 5, 26, 5, "G")
    c.px(5, 10, "G"); c.px(4, 9, "G"); c.px(32, 10, "G"); c.px(33, 9, "G")
    c.rect(17, 3, 20, 4, "G")
    c.px(18, 2, "F"); c.px(19, 1, "O")
    c.save("boss", "Great Ember Toad (W2 boss)", outline=True)


def maple(cx, cy, s):
    """Maple leaf polygon (7 lobes), pointing up."""
    pts = []
    lobes = [(-90, 1.0), (-140, 0.55), (-165, 0.85), (160, 0.45), (120, 0.6), (90, 0.28),
             (60, 0.6), (20, 0.45), (-15, 0.85), (-40, 0.55)]
    lobes = sorted(lobes, key=lambda p: p[0])
    for ang, r in lobes:
        a = math.radians(ang)
        pts.append((cx + math.cos(a) * s * r, cy + math.sin(a) * s * r))
    # add notches between lobes
    out = []
    for i, p in enumerate(pts):
        q = pts[(i + 1) % len(pts)]
        out.append(p)
        out.append((cx + (p[0] + q[0] - 2 * cx) * 0.22, cy + (p[1] + q[1] - 2 * cy) * 0.22))
    return out


def final():
    # Shogun of the Burning Leaf: black-and-crimson lacquered armour, blazing maple crest,
    # burning cape, war fan.
    rng = random.Random(26)
    c = C(48, 48, {"A": "#1e1a24", "a": "#3a3244", "q": "#564c64", "R": "#c02030", "r": "#7a1420",
                   "G": "#d8a840", "g": "#8a6424", "C": "#8a1420", "c": "#560c18", "F": "#ff7a2a",
                   "f": "#ffd060", "Z": "#fff4c0", "E": "#ffe040", "M": "#3a1418", "W": "#a8a4b0",
                   "k": "#0e0a10", "O": "#ff5a1a"})
    # burning cape behind
    c.poly([(15, 13), (32, 13), (30, 44), (6, 46), (2, 36), (8, 24)], "C")
    c.poly([(15, 13), (20, 13), (14, 45), (6, 46), (2, 36), (8, 24)], "c")
    # flames licking along the cape's trailing edges
    for i in range(14):
        t = i / 13
        bx, by = 2 + t * 4 + (1 - t) * 0, 36 + t * 10
        bx = 8 - 6 * (1 - abs(t - 0.4) * 1.5) if t < 0.4 else 2 + (t - 0.4) * 7
        by = 24 + t * 22
        h = rng.randint(2, 4)
        c.line(bx, by, bx - h * 0.8, by - h, "F")
        c.px(round(bx), round(by), "O")
        if i % 2 == 0:
            c.px(round(bx - 1), round(by - 1), "f")
    for x in range(6, 31, 2):
        y = 45 - (x - 6) * 0.04
        h = rng.randint(1, 4)
        c.line(x, y, x - 1, y - h, "F")
        c.px(x - 1, round(y - h), "f")
        c.px(x, round(y), "O")
    for _ in range(14):
        x, y = rng.randint(4, 28), rng.randint(30, 44)
        if c.get(x, y) in ("C", "c"):
            c.px(x, y, rng.choice("FOr"))
    # legs (greaves + sabatons)
    for x0 in (17, 26):
        c.rect(x0, 36, x0 + 4, 44, "A")
        c.rect(x0, 36, x0, 44, "a")
        for y in (38, 41):
            c.rect(x0, y, x0 + 4, y, "q")
        c.rect(x0 - 1, 44, x0 + 5, 46, "A")
        c.rect(x0 - 1, 44, x0 + 5, 44, "G")
    # thigh-armour skirt (kusazuri)
    c.poly([(14, 29), (35, 29), (37, 38), (12, 38)], "A")
    for y in (31, 34, 37):
        c.rect(12, y, 37, y, "a")
        for x in range(14, 37, 3):
            c.px(x, y - 1, "R")
    c.rect(12, 38, 37, 38, "G")
    for x in (18, 24, 30):
        c.rect(x, 30, x, 38, "k")
    # torso (do)
    c.rect(15, 16, 33, 29, "A")
    c.rect(15, 16, 16, 29, "a")
    for y in (19, 22, 25, 28):
        c.rect(15, y, 33, y, "a")
        for x in range(17, 33, 3):
            c.px(x, y + 1, "R")
    c.rect(20, 17, 30, 18, "R")
    # gold maple emblem on the chest
    c.poly(maple(25.5, 23, 4.2), "G")
    c.px(25, 23, "f")
    # shoulder guards (sode)
    for x0, x1 in ((8, 16), (32, 40)):
        c.rect(x0, 15, x1, 24, "A")
        c.rect(x0, 15, x1, 15, "G")
        for y in (17, 20, 23):
            c.rect(x0, y, x1, y, "a")
            for x in range(x0 + 1, x1, 3):
                c.px(x, y + 1, "R")
    # left arm
    c.rect(11, 24, 15, 30, "a")
    c.rect(11, 31, 15, 33, "A")
    c.rect(11, 31, 15, 31, "G")
    # helmet neck guard (shikoro)
    c.poly([(16, 9), (32, 9), (35, 15.5), (13, 15.5)], "A")
    for y in (11, 13, 15):
        c.rect(14, y, 34, y, "a")
        for x in range(15, 34, 2):
            c.px(x, y - 1, "R")
    # helmet bowl
    c.ellipse(24.5, 9, 7, 5.5, "A")
    c.rect(18, 6, 30, 6, "q")
    c.rect(17, 9, 32, 9, "G")
    # menpo mask with glowing eyes
    c.rect(22, 10, 31, 15, "M")
    c.rect(22, 11, 31, 11, "k")
    c.px(26, 11, "E"); c.px(27, 11, "E"); c.px(30, 11, "E")
    c.rect(24, 13, 31, 13, "W")
    c.rect(25, 14, 30, 14, "k")
    c.px(31, 12, "M"); c.px(32, 12, "M")
    # gold horns (kuwagata) and the blazing maple-leaf crest
    c.line(19, 7, 13, 0, "G", w=2)
    c.line(30, 7, 36, 0, "G", w=2)
    c.poly(maple(24.5, 5.5, 8.0), "O")
    c.poly(maple(24.5, 5.5, 6.4), "F")
    c.poly(maple(24.5, 5.8, 3.8), "f")
    c.px(24, 4, "Z"); c.px(24, 5, "Z")
    c.line(24.5, 6, 24.5, 9, "O")
    for x, y in ((18, 1), (31, 0), (21, 0), (28, 2)):
        c.px(x, y, rng.choice("Ff"))
    # right arm and war fan (gunbai)
    c.line(36, 23, 39, 28, "a", w=3)
    c.rect(38, 28, 41, 30, "A")
    c.rect(38, 28, 41, 28, "G")
    c.line(40, 30, 42, 21, "k", w=2)
    c.ellipse(42.5, 15, 5, 6.5, "G")
    c.ellipse(42.5, 15, 4, 5.5, "A")
    c.ellipse(42.5, 15, 3, 4.5, "R")
    c.ellipse(42.5, 14.5, 1.8, 1.8, "f")
    c.px(42, 14, "Z")
    c.rect(41, 21, 44, 21, "G")
    # embers drifting up from the cape
    for x, y in ((4, 20), (7, 15), (3, 28), (10, 10), (1, 31)):
        c.px(x, y, rng.choice("Ff"))
    c.save("final", "Shogun of the Burning Leaf (W2 final boss)", outline=True)


def relic():
    from gen_icons import Icon
    # Ember Leaf Headband: dark cloth headband with a metal plate engraved with a flame-leaf.
    i = Icon({"C": "#3a3448", "c": "#241e2e", "h": "#544a66", "M": "#b4bcc8", "m": "#6e7684",
              "W": "#eef2f8", "O": "#ff7a2a", "o": "#ffd060", "k": "#2a2a34"})
    i.poly([(0.5, 6.5), (8, 5.2), (15.5, 6.5), (15.5, 10.5), (8, 9.2), (0.5, 10.5)], "C")
    i.line(1, 6, 15, 6, "h")
    i.px(1, 10, "c"); i.px(14, 10, "c"); i.px(15, 10, "c")
    # trailing knot tails (left)
    i.poly([(1, 9), (0.5, 15.5), (3, 15), (3.5, 10)], "c")
    i.poly([(2.5, 9.5), (5, 14.5), (7, 13.5), (4.5, 9.5)], "C")
    i.px(4, 11, "h")
    # metal plate
    i.rect(5, 4, 11, 10, "M")
    i.rect(5, 10, 11, 10, "m")
    i.rect(11, 4, 11, 10, "m")
    i.rect(5, 4, 10, 4, "W")
    i.px(6, 5, "m"); i.px(10, 9, "k"); i.px(6, 9, "k"); i.px(10, 5, "m")
    # engraved flame-leaf
    i.poly([(8, 4.6), (10.2, 7.4), (9.2, 9.6), (6.8, 9.6), (5.8, 7.4)], "O")
    i.px(8, 5, "o"); i.px(7, 8, "o"); i.px(8, 7, "o")
    i.px(8, 9, "m")
    i.save("relic_w2", "Relic: Ember Leaf Headband (W2)")


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    c = C(32, 32, dict(GROUND)); floor(c, random.Random(201), leaves=11, ash=10)
    c.save("ground_a", "Forest floor with fallen maple leaves (W2)")
    c = C(32, 32, dict(GROUND)); floor(c, random.Random(202), leaves=5, ash=22, embers=2)
    c.save("ground_b", "Ash-strewn forest floor (W2)")
    c = C(32, 32, dict(GROUND)); floor(c, random.Random(203), leaves=8, ash=8); crater(c, random.Random(204))
    c.save("decal", "Scorched crater with glowing embers (W2)")
    c = C(32, 32, dict(GROUND)); path_tile(c, random.Random(205))
    c.save("path", "Packed dirt trail with plank section (W2)")
    for f in (prop_a, prop_b, prop_c, prop_d, fodder_a, fodder_b, ranged, blinker, elite, mini, boss, final):
        f()
    relic()
    print("wrote", len(list(OUT.glob("*.px"))), "W2 sprites to", OUT, "+ icons/relic_w2.px")


if __name__ == "__main__":
    main()
