"""Generates World 5 (Shatterline Crossing) art as .px sources in art/src/w5/.

    python3 tools/gen_w5.py

A cursed city crossing at night after a catastrophe: shattered asphalt veined with
purple curse energy, broken signals, wrecked cars, and the curses that roam there.
All designs are original. Deterministic (seeded), so re-running produces identical files.
"""
import math
import pathlib
import random
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_stage  # noqa: E402
from gen_stage import Canvas as _Canvas  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent.parent / "art/src/w5"
gen_stage.OUT = OUT


class Canvas(_Canvas):
    def wpx(self, x, y, c):
        """Wrapping pixel (for seamless tiles)."""
        self.g[y % self.h][x % self.w] = c

    def get(self, x, y):
        if 0 <= x < self.w and 0 <= y < self.h:
            return self.g[y][x]
        return "."

    def line(self, x0, y0, x1, y1, c, wrap=False):
        steps = int(max(abs(x1 - x0), abs(y1 - y0)) * 2) + 1
        for i in range(steps + 1):
            t = i / steps
            x, y = round(x0 + (x1 - x0) * t), round(y0 + (y1 - y0) * t)
            (self.wpx if wrap else self.px)(x, y, c)

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

    def recolor(self, src, dst, x0=0, y0=0, x1=None, y1=None):
        x1 = self.w - 1 if x1 is None else x1
        y1 = self.h - 1 if y1 is None else y1
        for y in range(max(0, y0), min(self.h - 1, y1) + 1):
            for x in range(max(0, x0), min(self.w - 1, x1) + 1):
                if self.g[y][x] == src:
                    self.g[y][x] = dst

    def shade_right(self, base, shade, frac=0.3):
        """Darken the right-most part of every run of `base` in each row... used as a cheap form shade."""
        for y in range(self.h):
            xs = [x for x in range(self.w) if self.g[y][x] == base]
            if not xs:
                continue
            cut = max(xs) - int((max(xs) - min(xs)) * frac)
            for x in xs:
                if x > cut:
                    self.g[y][x] = shade


# ───────────────────────── Ground ─────────────────────────
ASPH = {
    "A": "#1c1922", "B": "#211d28", "C": "#18151d",   # asphalt tones
    "H": "#2a2532",                                   # aggregate highlight
    "M": "#110e15",                                   # crack (deep)
    "v": "#2c1a3c", "V": "#4a2468",                   # curse vein: dim, mid
}


def asphalt(c, seed):
    rng = random.Random(seed)
    for y in range(32):
        for x in range(32):
            r = rng.random()
            c.g[y][x] = "A" if r < 0.55 else "B" if r < 0.8 else "C" if r < 0.97 else "H"
    # soft blotches (wrapping) so it isn't pure noise
    for _ in range(5):
        cx, cy, rad = rng.randint(0, 31), rng.randint(0, 31), rng.randint(3, 6)
        tone = rng.choice("BC")
        for dy in range(-rad, rad + 1):
            for dx in range(-rad, rad + 1):
                if dx * dx + dy * dy <= rad * rad and rng.random() < 0.55:
                    c.wpx(cx + dx, cy + dy, tone)
    return rng


def crack(c, rng, x, y, length, vein=True, glow="V"):
    """Random-walk crack with an inner curse vein; wraps around the tile."""
    dx = rng.choice((-1, 1))
    for i in range(length):
        c.wpx(x, y, "M")
        if vein and i % 3 != 2:
            c.wpx(x + 1, y, "v")
        if vein and rng.random() < 0.18:
            c.wpx(x, y, glow)
        if rng.random() < 0.15:  # branch stub
            bx, by = x, y
            for _ in range(rng.randint(2, 4)):
                bx += rng.choice((-1, 1))
                by += rng.choice((0, 1))
                c.wpx(bx, by, "M")
        if rng.random() < 0.5:
            x += dx
        if rng.random() < 0.25:
            dx = -dx
        y += 1


def ground():
    c = Canvas(32, 32, ASPH)
    rng = asphalt(c, 51)
    crack(c, rng, 6, 3, 14)
    crack(c, rng, 22, 18, 10, vein=False)
    c.save("ground_a", "W5 cracked night asphalt")

    c = Canvas(32, 32, ASPH)
    rng = asphalt(c, 52)
    # a patched square of slightly newer asphalt
    for y in range(9, 21):
        for x in range(12, 26):
            if c.g[y][x] != "H":
                if (x - 12 + (y * 7) % 3) > 1 and (25 - x + (y * 5) % 3) > 1:
                    c.g[y][x] = "C" if (x * 3 + y) % 7 else "A"
    crack(c, rng, 28, 22, 12)
    crack(c, rng, 3, 10, 7, vein=False)
    c.save("ground_b", "W5 patched asphalt with curse vein")

    # Decal: a fragment of crosswalk stripe split by a glowing curse crack.
    pal = {**ASPH, "W": "#3a3644", "w": "#2e2b37", "P": "#9a4ae0", "Q": "#d8a0ff"}
    c = Canvas(32, 32, pal)
    rng = asphalt(c, 53)
    for y in range(6, 26):
        for x in range(5, 27):
            jag = (x + y * 3) % 7 == 0
            if (x < 7 or x > 24) and jag:
                continue
            if rng.random() < 0.93:
                c.g[y][x] = "W" if rng.random() < 0.8 else "w"
    # broken corner chunks
    c.rect(5, 6, 8, 8, "A"); c.rect(23, 23, 26, 25, "B")
    # glowing crack through the stripe
    x = 4
    for y in range(3, 29):
        c.px(x + 10, y, "M")
        c.px(x + 11, y, "P")
        if 8 < y < 24:
            c.px(x + 12, y, "M")
            if y % 4 == 0:
                c.px(x + 11, y, "Q")
        if y % 3 == 0:
            x += rng.choice((-1, 1, 1, 0))
    # side fissure
    for k in range(6):
        c.px(15 + k, 15 + (k // 2), "M")
        c.px(15 + k, 14 + (k // 2), "V")
    c.save("decal", "W5 crosswalk fragment with glowing curse crack")

    # Path: dimmed zebra stripes running vertically (tiles top-to-bottom).
    pal = {**ASPH, "W": "#34303c", "w": "#2c2933"}
    c = Canvas(32, 32, pal)
    rng = asphalt(c, 54)
    for x0 in (4, 20):
        for y in range(32):
            for x in range(x0, x0 + 8):
                r = rng.random()
                if r < 0.9:
                    c.g[y][x] = "W" if r < 0.75 else "w"
    crack(c, rng, 16, 20, 9, vein=True)
    c.save("path", "W5 dimmed zebra crossing stripes (vertical)")


# ───────────────────────── Props ─────────────────────────
def props():
    # prop_a: broken traffic signal pole, head knocked askew, red + amber lit.
    pal = {"G": "#5a5866", "g": "#3c3a46", "h": "#7a7888", "D": "#26242c", "d": "#16141a",
           "R": "#ff3a3a", "r": "#a02020", "Y": "#ffb030", "y": "#8a5a10", "O": "#2a3a2a", "W": "#ffd0c0"}
    c = Canvas(16, 32, pal)
    c.rect(4, 28, 11, 31, "g"); c.rect(4, 28, 11, 28, "G"); c.rect(5, 31, 10, 31, "d")
    c.rect(7, 10, 8, 27, "G"); c.rect(8, 10, 8, 27, "g"); c.px(7, 14, "h"); c.px(7, 20, "h")
    # bent arm
    c.line(8, 10, 10, 7, "G"); c.line(8, 11, 10, 8, "g")
    # signal housing, tilted
    c.poly([(7, 1), (13, 0), (15, 13), (9, 14)], "D")
    c.poly([(8, 2), (12, 1), (14, 12), (10, 13)], "d")
    c.ellipse(11, 3.5, 1.8, 1.6, "R"); c.px(10, 3, "W")
    c.ellipse(11.8, 7.5, 1.8, 1.6, "Y")
    c.ellipse(12.6, 11.4, 1.6, 1.4, "O")
    # visors
    c.line(9, 2, 13, 1, "D"); c.line(10, 6, 14, 5, "D"); c.line(11, 10, 15, 9, "D")
    # dangling wire
    for y in range(13, 21):
        c.px(4 + (y % 3 == 0), y, "d")
    c.px(4, 21, "Y")
    c.save("prop_a", "Broken traffic signal (red/amber lit)", outline=True)

    # prop_b: wrecked compact car, side view facing right.
    pal = {"R": "#7a2a34", "r": "#521a24", "q": "#9a3a44", "M": "#2a2830", "m": "#46444e", "T": "#151318",
           "W": "#3a4a6a", "w": "#6a8ab0", "L": "#ffe08a", "l": "#ff5a3a", "P": "#a050ff", "S": "#8a8a96"}
    c = Canvas(32, 32, pal)
    # body
    c.poly([(1, 22), (2, 16), (8, 15), (12, 9), (22, 9), (27, 15), (31, 17), (31, 25), (1, 26)], "R")
    c.poly([(1, 23), (31, 22), (31, 26), (1, 26)], "r")
    c.line(2, 17, 30, 18, "q")
    # crumpled roof dent
    c.poly([(14, 9), (18, 9), (17, 12), (15, 11)], "r")
    # windows (shattered)
    c.poly([(13, 10), (16, 10), (16, 15), (10, 15)], "W")
    c.poly([(18, 10), (22, 10), (26, 15), (18, 15)], "W")
    for (x, y) in ((11, 14), (13, 12), (14, 14), (19, 11), (21, 13), (23, 14), (20, 14)):
        c.px(x, y, "w")
    c.line(19, 10, 24, 15, "T")
    c.rect(17, 10, 17, 22, "r")
    # door line + handle
    c.line(17, 16, 17, 24, "r"); c.rect(13, 18, 15, 18, "m")
    # lights
    c.rect(29, 18, 31, 19, "L"); c.rect(1, 18, 2, 19, "l")
    # bumpers
    c.rect(0, 23, 3, 24, "S"); c.rect(28, 23, 31, 24, "S")
    # wheels (front one flat)
    c.ellipse(8, 26.5, 3.6, 3.6, "T"); c.ellipse(8, 26.5, 1.6, 1.6, "m")
    c.ellipse(24, 27.5, 3.8, 2.8, "T"); c.ellipse(24, 27.5, 1.5, 1.2, "m")
    c.rect(4, 30, 28, 31, "T")
    # curse crack across the bodywork
    for x, y in ((4, 20), (5, 21), (6, 21), (7, 22), (8, 21), (9, 22), (10, 23)):
        c.px(x, y, "P")
    c.save("prop_b", "Wrecked compact car", outline=True)

    # prop_c: overturned bus-stop shelter behind a row of concrete barriers.
    pal = {"C": "#6a6872", "c": "#4a4852", "h": "#86848e", "Y": "#c8a030", "y": "#2a2420",
           "F": "#3a3a46", "f": "#24242c", "G": "#4a6a8a", "g": "#8ab0d0", "B": "#2a5aa0", "W": "#d8e0f0",
           "P": "#a050ff"}
    c = Canvas(48, 32, pal)
    # shelter lying on its side (frame + broken glass panel)
    c.rect(4, 6, 40, 7, "F"); c.rect(4, 6, 5, 20, "F"); c.rect(39, 6, 40, 20, "F")
    c.rect(6, 8, 38, 19, "G")
    for k in range(18):
        c.px(8 + k * 2 - (k % 3), 9 + (k * 5) % 10, "g")
    c.line(20, 8, 26, 19, "f"); c.line(26, 8, 22, 14, "f"); c.line(10, 19, 16, 11, "f")
    c.rect(6, 4, 38, 5, "f")  # roof edge
    # bus-stop sign leaning
    c.line(43, 4, 45, 21, "F"); c.rect(41, 1, 46, 5, "B"); c.rect(42, 2, 45, 4, "W"); c.rect(43, 3, 44, 3, "B")
    # three jersey barriers in front
    for i, x0 in enumerate((1, 16, 31)):
        top = 17 + (i == 1)
        c.poly([(x0 + 3, top), (x0 + 12, top), (x0 + 13, 26), (x0 + 15, 31), (x0, 31), (x0 + 2, 26)], "C")
        c.rect(x0 + 3, top, x0 + 12, top, "h")
        c.rect(x0 + 1, 28, x0 + 14, 31, "c")
        for x in range(x0 + 3, x0 + 13):
            if (x - x0) // 2 % 2 == 0:
                c.rect(x, top + 3, x, top + 4, "Y")
            else:
                c.rect(x, top + 3, x, top + 4, "y")
    # chip and curse crack on middle barrier
    c.rect(28, 18, 30, 19, ".")
    c.line(22, 19, 25, 30, "P"); c.line(23, 19, 26, 30, "c")
    c.save("prop_c", "Overturned bus shelter and concrete barriers", outline=True)

    # prop_d: rubble heap with a glowing cursed talisman.
    pal = {"C": "#5a5864", "c": "#3e3c48", "h": "#7a7884", "R": "#5a3a3a", "S": "#8a4a3a",
           "T": "#e8dcb8", "t": "#b0a080", "P": "#c070ff", "p": "#7a30c0", "I": "#2a1a3a"}
    c = Canvas(24, 24, pal)
    c.poly([(2, 23), (5, 15), (9, 12), (14, 11), (19, 14), (22, 23)], "c")
    rng = random.Random(55)
    for _ in range(14):
        x, y = rng.randint(4, 19), rng.randint(13, 21)
        w, h = rng.randint(2, 4), rng.randint(1, 3)
        c.rect(x, y, x + w, y + h, rng.choice("CCcRS"))
        c.px(x, y, "h")
    c.rect(3, 22, 21, 23, "c")
    # rebar
    c.line(6, 14, 3, 9, "R"); c.line(17, 13, 20, 8, "R")
    # talisman tilted, stuck in the top
    c.poly([(10, 2), (14, 1), (16, 12), (12, 13)], "T")
    c.poly([(14, 1), (15, 1), (17, 12), (16, 12)], "t")
    c.line(12, 4, 14, 11, "I"); c.line(11, 6, 15, 6, "I"); c.px(12, 9, "I"); c.px(15, 9, "I")
    c.px(13, 3, "P"); c.px(13, 8, "P"); c.px(14, 10, "P")
    c.save("prop_d", "Rubble heap with cursed talisman", outline=True)
    # purple glow pixels around the seal (after outline, so they float)
    c.px(9, 1, "p"); c.px(17, 3, "p"); c.px(8, 7, "P"); c.px(18, 9, "p")
    c.save("prop_d", "Rubble heap with cursed talisman")


# ───────────────────────── Enemies ─────────────────────────
CURSE = {"F": "#7a3aa8", "f": "#4e2470", "h": "#a868d8", "E": "#ffe85a", "W": "#fff8e0", "I": "#1a0a20",
         "M": "#ff5aa0"}


def eye(c, x, y, big=False, white="W", pupil="I"):
    c.px(x, y, white); c.px(x + 1, y, white)
    if big:
        c.px(x, y + 1, white); c.px(x + 1, y + 1, white)
        c.px(x + 1, y + 1, pupil)
    else:
        c.px(x + 1, y, pupil)


def fodder():
    # Cursed Spawn: a lumpy curse blob with too many eyes.
    c = Canvas(16, 16, CURSE)
    c.ellipse(8, 10, 7, 5.5, "F")
    c.ellipse(6, 6.5, 3, 3, "F"); c.ellipse(11, 6, 2.5, 2.5, "F")
    c.ellipse(9, 12.5, 6, 2.5, "f")
    c.rect(2, 14, 13, 14, "f")
    c.px(3, 15, "f"); c.px(7, 15, "f"); c.px(12, 15, "f")
    c.px(4, 6, "h"); c.px(5, 5, "h"); c.px(10, 4, "h"); c.px(3, 9, "h")
    for x, y in ((5, 6), (10, 5), (12, 9), (3, 10)):
        eye(c, x, y)
    eye(c, 7, 8, big=True, white="E")
    c.px(14, 11, "W"); c.px(13, 7, "W")
    c.rect(8, 12, 11, 12, "I"); c.px(9, 12, "M")
    c.save("fodder_a", "Cursed Spawn (many-eyed curse blob)", outline=True)

    # Wailing Commuter: cursed office-worker husk, gaping mouth, briefcase.
    pal = {"S": "#9a9ab0", "s": "#686880", "J": "#4c4c66", "j": "#34344a", "T": "#d8d8e0", "R": "#c0303a",
           "O": "#1a0a20", "P": "#c070ff", "E": "#e8c0ff", "B": "#8a5a30", "b": "#5a3a1a", "L": "#26263a"}
    c = Canvas(16, 16, pal)
    # legs
    c.rect(5, 12, 6, 15, "L"); c.rect(9, 12, 10, 14, "L"); c.px(10, 15, "L"); c.px(11, 15, "L")
    # suit body, hunched forward
    c.rect(4, 7, 11, 12, "J"); c.rect(9, 7, 11, 12, "j")
    c.rect(7, 7, 8, 9, "T"); c.px(7, 8, "R"); c.px(7, 9, "R"); c.px(7, 10, "R")
    # head (tilted forward)
    c.ellipse(8.5, 3.5, 3.5, 3.5, "S")
    c.rect(5, 1, 10, 1, "s"); c.px(5, 2, "s")
    c.px(9, 2, "O"); c.px(11, 2, "O"); c.px(9, 3, "P"); c.px(11, 3, "P")
    c.rect(9, 4, 11, 6, "O"); c.px(10, 5, "P")  # gaping mouth
    # arms: one dangling, one holding briefcase
    c.rect(3, 8, 3, 11, "J"); c.px(3, 12, "S")
    c.rect(12, 8, 12, 10, "j"); c.px(12, 11, "S")
    c.rect(11, 12, 15, 14, "B"); c.rect(11, 14, 15, 14, "b"); c.rect(12, 11, 14, 11, "b")
    c.save("fodder_b", "Wailing Commuter (cursed office husk)", outline=True)

    # Curse Eye: a floating giant eyeball wreathed in purple flame.
    pal = {"W": "#ece4f2", "w": "#b8a8c8", "I": "#d03aff", "i": "#7a10b0", "O": "#120818", "V": "#c03050",
           "F": "#8a3ad0", "f": "#5a1a9a", "Q": "#e0a8ff", "H": "#ffffff"}
    c = Canvas(16, 16, pal)
    # flame trailing back (left) and up
    c.poly([(1, 4), (4, 6), (3, 1), (6, 4), (8, 0), (9, 4), (12, 1), (12, 6), (15, 7), (13, 13), (6, 15), (2, 12), (0, 8), (3, 9)], "f")
    c.poly([(2, 6), (5, 6), (5, 3), (7, 5), (8, 2), (10, 5), (11, 4), (12, 8), (11, 12), (6, 13), (3, 11)], "F")
    c.ellipse(9, 8.5, 5, 5, "W")
    c.ellipse(9, 9.5, 5, 4, "W")
    for y in range(4, 14):
        for x in range(4, 15):
            if c.g[y][x] == "W" and ((x - 9) ** 2 + (y - 9) ** 2 > 14) and x < 9:
                c.g[y][x] = "w"
    c.ellipse(11, 8.5, 2.6, 2.6, "I"); c.px(13, 9, "i"); c.px(12, 10, "i")
    c.rect(11, 8, 12, 9, "O")
    c.px(10, 7, "H")
    c.line(5, 7, 7, 8, "V"); c.line(6, 11, 8, 10, "V")
    c.px(4, 1, "Q"); c.px(9, 1, "Q"); c.px(14, 5, "Q")
    c.save("ranged", "Curse Eye (flaming floating eyeball)", outline=True)

    # Glitch Shade: shadow figure with a flickering, glitched outline.
    pal = {"D": "#4a3c66", "d": "#33294a", "h": "#6a5a94", "E": "#ffffff", "C": "#3af0ff", "M": "#ff3aa0"}
    c = Canvas(16, 16, pal)
    c.ellipse(8, 3.5, 3, 3, "D")
    c.poly([(4, 6), (12, 6), (13, 12), (11, 15), (9, 13), (7, 15), (5, 13), (3, 15)], "D")
    c.rect(9, 6, 12, 12, "d"); c.rect(3, 7, 3, 11, "D"); c.rect(13, 7, 13, 10, "d")
    c.px(6, 5, "h"); c.px(5, 7, "h"); c.px(5, 8, "h")
    c.px(8, 3, "E"); c.px(10, 3, "E")
    c.outline()
    # glitch: shifted slices with chromatic edges
    rng = random.Random(56)
    for y in (5, 9, 10, 13):
        row = c.g[y][:]
        s = rng.choice((-1, 1, 2))
        for x in range(16):
            src = x - s
            c.g[y][x] = row[src] if 0 <= src < 16 else "."
        xs = [x for x in range(16) if c.g[y][x] != "."]
        if xs:
            c.px(min(xs) - 1, y, "C"); c.px(max(xs) + 1, y, "M")
    for x, y, col in ((1, 3, "C"), (14, 7, "M"), (0, 11, "C")):
        c.px(x, y, col)
    c.save("blinker", "Glitch Shade (glitched shadow figure)")


def elite():
    # Grade-One Brute: hulking four-armed curse with a stitched face.
    pal = {"F": "#7a5a8a", "f": "#523a62", "g": "#38264a", "h": "#a084b0", "T": "#e8dcc8", "O": "#1a0a1a",
           "E": "#ff5a3a", "e": "#ffd27a", "P": "#b050ff", "N": "#c8c0b0", "Z": "#5a1a2a"}
    c = Canvas(32, 32, pal)
    # legs
    c.rect(8, 25, 13, 31, "f"); c.rect(18, 25, 23, 31, "f")
    c.rect(6, 30, 14, 31, "g"); c.rect(17, 30, 25, 31, "g")
    # massive torso
    c.ellipse(16, 18, 11, 9, "F")
    c.ellipse(18, 21, 9, 6, "f")
    # hump/shoulders
    c.ellipse(9, 11, 6, 5, "F"); c.ellipse(22, 11, 6, 5, "F")
    # small head pushed forward
    c.ellipse(19, 8, 5, 5, "F")
    c.ellipse(20, 10, 4, 3, "f")
    c.px(17, 4, "h"); c.px(18, 4, "h"); c.px(7, 8, "h"); c.px(8, 7, "h"); c.px(21, 8, "h")
    # stitched face: seam down the face + crosses
    c.line(18, 4, 21, 12, "T")
    for (x, y) in ((17, 6), (19, 6), (18, 8), (20, 8), (19, 10), (21, 10)):
        c.px(x, y, "N")
    c.px(16, 7, "E"); c.px(17, 7, "e")      # left eye
    c.px(22, 6, "E"); c.px(23, 6, "E"); c.px(22, 7, "E")  # big right eye
    c.rect(18, 11, 23, 12, "O")
    for x in (18, 20, 22):
        c.px(x, 11, "T")
    # upper arms: huge, raised claws
    c.poly([(3, 9), (6, 9), (5, 18), (2, 21), (0, 19)], "F")
    c.poly([(26, 9), (29, 10), (31, 16), (30, 21), (27, 19)], "F")
    c.rect(0, 20, 3, 22, "f"); c.rect(28, 20, 31, 22, "f")
    for x in (0, 2):
        c.px(x, 23, "T")
    for x in (28, 30):
        c.px(x, 23, "T")
    # lower arms: hanging from the belly
    c.poly([(6, 18), (9, 19), (8, 26), (5, 27)], "g")
    c.poly([(24, 19), (27, 18), (28, 26), (25, 26)], "g")
    c.px(5, 27, "T"); c.px(28, 26, "T")
    # body stitches and a curse sigil
    c.line(10, 15, 14, 24, "T")
    for k in range(4):
        c.px(10 + k, 16 + k * 2, "N"); c.px(12 + k, 16 + k * 2, "N")
    c.ellipse(19, 18, 2, 2, "P"); c.px(19, 18, "O")
    c.px(14, 27, "Z")
    c.save("elite", "Grade-One Brute (four-armed stitched curse)", outline=True)


def mini():
    # The Patchwork Sorcerer: patchwork coat, mismatched eyes, puppet strings.
    pal = {"S": "#e0d4cc", "s": "#b4a49c", "H": "#4a5a7a", "h": "#2e3a54", "A": "#3a6a6a", "a": "#2a4a4a",
           "R": "#8a4a3a", "r": "#5a2e24", "U": "#8a7a3a", "u": "#5a4e22", "V": "#5a3a6a", "v": "#3a2448",
           "T": "#e8e0c8", "Y": "#ffd040", "C": "#40e0ff", "O": "#1a0a1a", "Q": "#d0b8ff", "q": "#8a70d0",
           "B": "#1e1a24", "G": "#c8a040"}
    c = Canvas(32, 32, pal)
    # legs and boots
    c.rect(11, 25, 13, 29, "h"); c.rect(17, 25, 19, 29, "h")
    c.rect(10, 30, 14, 31, "B"); c.rect(16, 30, 21, 31, "B")
    # long patchwork coat (flared)
    c.poly([(9, 12), (21, 12), (24, 27), (6, 27)], "A")
    patches = [(9, 12, 14, 17, "R"), (15, 12, 21, 16, "V"), (7, 18, 12, 22, "U"), (13, 17, 18, 22, "A"),
               (19, 17, 23, 22, "R"), (6, 23, 11, 27, "V"), (12, 23, 17, 27, "U"), (18, 23, 24, 27, "A")]
    for x0, y0, x1, y1, col in patches:
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                if c.g[y][x] != ".":
                    c.g[y][x] = col
    # shade right side of each patch colour
    for base, sh in (("A", "a"), ("R", "r"), ("U", "u"), ("V", "v")):
        for y in range(12, 28):
            for x in range(19, 25):
                if c.g[y][x] == base:
                    c.g[y][x] = sh
    # stitches along the seams
    for y in range(12, 28, 2):
        c.px(14 if y < 17 else 12 if y < 23 else 17, y, "T")
    for x in range(8, 24, 2):
        c.px(x, 17, "T"); c.px(x, 22, "T")
    # collar and buttons
    c.rect(11, 11, 19, 12, "v"); c.px(15, 14, "G"); c.px(15, 19, "G"); c.px(15, 24, "G")
    # left arm hanging
    c.poly([(8, 13), (10, 13), (8, 22), (6, 22)], "V"); c.rect(6, 23, 7, 23, "S")
    # right arm raised forward, fingers spread
    c.poly([(19, 13), (22, 12), (27, 11), (27, 13), (22, 15)], "U")
    c.line(23, 12, 23, 14, "T")
    c.rect(27, 10, 29, 12, "S")
    for x in (27, 29, 31):
        c.px(x, 9, "S")
    c.px(30, 10, "S")
    # puppet strings dropping from the fingertips, with tiny glowing knots
    for x, end in ((27, 22), (29, 26), (31, 19)):
        for y in range(13, end):
            c.px(x, y, "Q" if y % 4 else "q")
        c.px(x, end, "Q")
    # marionette cross-bar dangling
    c.rect(26, 22, 31, 22, "s")
    # head
    c.ellipse(15, 6.5, 5, 5.2, "S")
    c.rect(11, 9, 18, 10, "s")
    # hair: swept ash-blue, uneven
    c.ellipse(14.5, 3.5, 5.6, 3.6, "H")
    c.rect(9, 3, 11, 8, "H"); c.px(12, 7, "H")
    c.poly([(15, 0), (21, 2), (19, 5)], "H")
    c.px(10, 1, "h"); c.px(13, 1, "h"); c.px(17, 2, "h")
    # stitch scar across face
    c.line(13, 5, 19, 9, "s")
    c.px(14, 6, "T"); c.px(16, 7, "T"); c.px(18, 8, "T")
    # mismatched eyes (gold / cyan)
    c.px(14, 7, "Y"); c.px(15, 7, "O")
    c.px(18, 6, "C"); c.px(19, 6, "C"); c.px(18, 7, "O")
    c.rect(16, 10, 19, 10, "O"); c.px(19, 9, "O")  # crooked grin
    c.save("mini", "The Patchwork Sorcerer", outline=True)


def boss():
    # Calamity of the Crossing: giant curse fused with wrecked cars and traffic lights.
    pal = {"F": "#5a2e7a", "f": "#3e1c58", "g": "#2a1240", "h": "#8a52b0",
           "R": "#8a3036", "r": "#5a1e24", "M": "#6a6a78", "m": "#44444e", "D": "#22222a",
           "L": "#fff0a0", "l": "#ffc040", "X": "#ff3a3a", "Y": "#ffb030", "G": "#40e070",
           "O": "#ff7a2a", "o": "#ffe08a", "Z": "#1a0610", "T": "#ece0d0", "P": "#c070ff", "W": "#3a4a6a"}
    c = Canvas(48, 48, pal)
    # stumpy legs / base mass
    c.ellipse(13, 42, 7, 6, "f"); c.ellipse(34, 42, 7, 6, "f")
    c.rect(5, 45, 42, 47, "g")
    # main body mass
    c.ellipse(24, 28, 21, 16, "F")
    c.ellipse(26, 33, 17, 10, "f")
    # left: a car fused into the flank (hood + windshield + wheel)
    c.poly([(0, 20), (12, 17), (15, 24), (13, 34), (1, 36)], "R")
    c.poly([(1, 30), (13, 28), (13, 34), (1, 36)], "r")
    c.poly([(3, 19), (10, 17), (11, 22), (4, 24)], "W")
    c.line(4, 20, 10, 22, "M")
    c.ellipse(5, 37, 4, 4, "D"); c.ellipse(5, 37, 1.6, 1.6, "m")
    c.rect(0, 27, 2, 28, "L")
    # right: a car door slab jammed into the shoulder
    c.poly([(36, 14), (47, 18), (46, 30), (37, 28)], "M")
    c.poly([(38, 16), (45, 19), (44, 23), (38, 21)], "W")
    c.line(37, 26, 46, 28, "m"); c.rect(41, 25, 43, 25, "D")
    # head mass
    c.ellipse(26, 15, 12, 10, "F")
    c.ellipse(28, 18, 9, 6, "f")
    for x, y in ((18, 9), (20, 7), (23, 6), (12, 20), (15, 16), (31, 7)):
        c.px(x, y, "h")
    # traffic-light horns
    def signal(x0, y0, x1, y1):
        c.rect(x0, y0, x1, y1, "D")
        n = 3
        hgt = (y1 - y0 + 1) // n
        for i, col in enumerate("XYG"):
            cy = y0 + i * hgt + hgt // 2
            c.rect(x0 + 1, cy, x1 - 1, cy, col)
            if hgt > 2:
                c.rect(x0 + 1, cy - 1 + (hgt > 3), x1 - 1, cy - 1 + (hgt > 3), col if i == 0 else "D")
    c.line(19, 8, 15, 3, "M"); c.line(20, 8, 16, 3, "m")
    signal(13, 0, 17, 8); c.rect(15, 9, 16, 10, "m")
    c.line(33, 7, 37, 2, "M"); c.line(34, 8, 38, 3, "m")
    signal(35, 0, 39, 7); c.rect(36, 8, 37, 9, "m")
    # headlight eyes
    c.ellipse(24, 13, 2.6, 2.2, "L"); c.px(24, 13, "l")
    c.ellipse(32, 12, 2.6, 2.2, "L"); c.px(32, 12, "l")
    c.px(28, 10, "P"); c.px(29, 9, "P")
    # gaping glowing mouth with teeth
    c.ellipse(28, 24, 9, 5, "Z")
    c.ellipse(28, 25, 7, 3.4, "O")
    c.ellipse(28, 25.5, 4, 2, "o")
    for x in range(20, 37, 3):
        c.rect(x, 20, x, 21, "T")
        c.rect(x + 1, 28, x + 1, 29, "T")
    # bumper strap across the belly, curse cracks
    c.poly([(14, 34), (40, 31), (40, 33), (14, 36)], "M")
    c.line(14, 36, 40, 33, "m")
    for x, y in ((18, 38), (19, 39), (21, 39), (22, 40), (30, 37), (31, 38), (33, 38)):
        c.px(x, y, "P")
    c.save("boss", "Calamity of the Crossing (car-fused curse)", outline=True)


def final():
    # The Shatterline Sovereign: four-armed curse monarch, glass crown, skull shrine behind.
    pal = {"N": "#4a4458", "n": "#2e2a38",                          # shrine stone
           "T": "#d8ccb8", "t": "#a0947e", "O": "#140812",          # bone / skulls, void
           "S": "#b8a8c8", "s": "#8a789c", "k": "#2a1030",           # ash-violet skin, dark markings
           "R": "#3a2a6a", "r": "#241a46", "G": "#d0a840", "g": "#8a6a24",  # robe + gold trim
           "C": "#bff4ff", "c": "#5ac0e0", "Q": "#8a5ae0",           # shattered glass crown
           "E": "#ff3ac0", "e": "#ffd0f0", "P": "#b050ff", "p": "#2a1440", "q": "#4a2270"}
    c = Canvas(48, 48, pal)
    # shrine of skulls behind: two pillars + lintel
    c.rect(2, 8, 7, 47, "N"); c.rect(40, 8, 45, 47, "N")
    c.rect(6, 8, 7, 47, "n"); c.rect(44, 8, 45, 47, "n")
    c.rect(0, 4, 47, 8, "N"); c.rect(0, 8, 47, 9, "n")
    c.poly([(0, 4), (4, 1), (43, 1), (47, 4)], "n")
    for x in range(3, 46, 6):  # skulls on the lintel
        c.ellipse(x + 1.5, 5, 2, 2, "T")
        c.px(x + 1, 5, "O"); c.px(x + 2, 5, "O")
    for y in range(12, 46, 6):  # skulls on the pillars
        for x in (3, 41):
            c.ellipse(x + 1.5, y + 1, 2, 2, "T" if y % 12 else "t")
            c.px(x + 1, y + 1, "O"); c.px(x + 2, y + 1, "O")
    # domain glow inside the shrine
    c.rect(8, 10, 39, 47, "p")
    for y in range(10, 48):
        for x in range(8, 40):
            d = ((x - 23.5) ** 2 + (y - 22) ** 2) ** 0.5
            if 12 < d < 15:
                c.g[y][x] = "q"
            elif (x * 7 + y * 3) % 23 == 0:
                c.g[y][x] = "P"
    # robe (seated/standing, wide)
    c.poly([(16, 20), (32, 20), (38, 46), (10, 46)], "R")
    c.poly([(25, 20), (32, 20), (38, 46), (27, 46)], "r")
    c.rect(23, 20, 24, 46, "G"); c.rect(10, 45, 38, 46, "G")
    c.rect(9, 47, 39, 47, "g")
    # sash with a shattered-glass ornament
    c.rect(16, 30, 32, 31, "g"); c.px(24, 30, "C"); c.px(24, 31, "c")
    # shoulders / mantle
    c.ellipse(24, 21, 10, 3, "r")
    c.rect(14, 20, 34, 20, "G")
    # upper arms: raised outward, open palms
    c.poly([(15, 20), (17, 22), (11, 15), (9, 16)], "S")
    c.poly([(11, 15), (9, 16), (7, 10), (9, 9)], "S")
    c.poly([(33, 20), (31, 22), (37, 15), (39, 16)], "s")
    c.poly([(37, 15), (39, 16), (41, 10), (39, 9)], "s")
    c.rect(7, 7, 10, 9, "S"); c.rect(38, 7, 41, 9, "s")
    for x in (7, 9):
        c.px(x, 6, "S")
    for x in (38, 40):
        c.px(x, 6, "s")
    # dark bands on the upper arms
    c.line(10, 13, 12, 14, "k"); c.line(8, 11, 10, 11, "k")
    c.line(36, 14, 38, 13, "k"); c.line(38, 11, 40, 11, "k")
    # lower arms: forward, one holding a curse orb
    c.poly([(16, 23), (18, 25), (14, 31), (12, 30)], "S")
    c.rect(11, 30, 14, 32, "S")
    c.poly([(32, 23), (30, 25), (34, 30), (36, 29)], "s")
    c.ellipse(36, 31, 3, 3, "P"); c.ellipse(36, 31, 1.5, 1.5, "e")
    c.line(14, 26, 16, 25, "k"); c.line(33, 26, 34, 27, "k")
    # neck + head: cracked porcelain mask face
    c.rect(22, 17, 26, 20, "s")
    c.ellipse(24, 13, 5, 5.5, "S")
    c.rect(25, 9, 29, 18, "s")
    c.ellipse(24, 13, 4, 4.5, "S")
    c.line(21, 9, 23, 17, "k")          # crack/marking down the mask
    c.line(26, 15, 28, 18, "k")
    c.rect(21, 12, 22, 12, "E"); c.rect(26, 12, 27, 12, "E")   # glowing eyes
    c.px(21, 11, "e"); c.px(27, 11, "e")
    c.rect(22, 16, 26, 16, "O")
    # crown of shattered glass shards
    c.rect(19, 7, 29, 8, "c")
    for x0, hgt in ((19, 3), (21, 6), (23, 4), (24, 7), (26, 5), (28, 3)):
        c.line(x0, 7, x0 + (1 if x0 > 24 else -1 if x0 < 23 else 0), 7 - hgt, "C")
    c.px(22, 4, "Q"); c.px(25, 3, "Q"); c.px(27, 5, "c"); c.px(20, 6, "c")
    c.save("final", "The Shatterline Sovereign (four-armed curse monarch)", outline=True)


def relic():
    from gen_icons import Icon
    i = Icon({"T": "#ece0c0", "t": "#b8a882", "R": "#7a2ac0", "P": "#c070ff", "p": "#5a1a9a", "O": "#2a1030"})
    i.circle(8, 8, 7.5, "p")
    i.circle(8, 8, 6.5, "T")
    i.circle(8.6, 8.6, 6.5, "T")
    i.circle(8, 8, 4.5, "R")
    i.circle(8, 8, 3.5, "T")
    i.rect(7, 5, 8, 11, "R"); i.rect(5, 8, 11, 8, "R")
    i.px(5, 5, "P"); i.px(11, 11, "P"); i.px(11, 5, "P"); i.px(5, 11, "P")
    # crack through the seal
    i.line(10, 1, 7, 7, "O"); i.line(7, 7, 9, 10, "O"); i.line(9, 10, 6, 15, "O")
    i.px(11, 2, "P"); i.px(5, 14, "P")
    for y in range(9, 15):
        for x in range(10, 15):
            if i.g[y][x] == "T":
                i.g[y][x] = "t"
    i.save("relic_w5", "Shattered Seal (World 5 relic)")


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    ground()
    props()
    fodder()
    elite()
    mini()
    boss()
    final()
    relic()
    print("wrote world 5 art to", OUT)


if __name__ == "__main__":
    main()
