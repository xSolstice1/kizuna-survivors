"""Generates Riku Kazamine (original fox-spirit shinobi) as .px sources in art/src/.

    python3 tools/gen_riku.py

riku.px      base form: crimson hair, fox mask tilted on his head, indigo shinobi gear with a
             high collar, crimson scarf, gold sash, blue foxfire in his right hand.
riku_fox.px  Ninefold Foxfire form: the same pose wreathed in golden foxfire, glowing hair and
             eyes, flame crests on the chest and a flaring cloak. (The nine tails, aura and
             particles are drawn in code so they can move.)

Same 24x32 frame and leg rows as heroine.px, so the client's walk-cycle leg lift works.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_stage  # noqa: E402
from gen_stage import Canvas  # noqa: E402

gen_stage.OUT = pathlib.Path(__file__).resolve().parent.parent / "art/src"

BASE = {
    "H": "#d8343a", "h": "#8e1c2a", "L": "#ff8a6a",   # hair
    "S": "#ffe2cc", "s": "#f2b89a",                   # skin
    "E": "#24163f", "A": "#ffb02e", "W": "#ffffff",   # eyes
    "M": "#f4f0fa", "m": "#b8b0c8", "r": "#e0263c",   # fox mask
    "N": "#2b2f5e", "n": "#1b1d3e", "J": "#454a8a",   # indigo gear
    "G": "#f2b632", "g": "#b07e1c",                   # gold sash
    "C": "#c42a3a", "c": "#7e1828",                   # crimson scarf
    "D": "#3a3348", "B": "#4a2e2a",                   # leg wraps, sandals
    "F": "#bff0ff", "f": "#3a9cff", "q": "#1a52c8",   # foxfire (core, mid, edge)
}

# Ninefold Foxfire: same letters, new colours (gear and hair become flame).
FOX = {
    **BASE,
    "H": "#ffb43a", "h": "#e8641a", "L": "#fff2c0",
    "A": "#ff7a1a", "E": "#ff3a00",
    "N": "#ff9a2a", "n": "#d4560e", "J": "#ffd36a",
    "C": "#ffcf4a", "c": "#ff7a1a",
    "G": "#ffffff", "g": "#ffd36a",
    "D": "#c4500e", "B": "#7a2a0a",
    "F": "#ffffff", "f": "#ffd36a", "q": "#ff8a1a",
    "X": "#3a1004",   # flame-crest markings
}


def tri_up(c, x0, x1, tip_x, tip_y, base_y, col):
    """Filled triangle from a base row up to a tip (hair spikes)."""
    for y in range(tip_y, base_y + 1):
        t = (y - tip_y) / max(1, base_y - tip_y)
        lx = round(tip_x + (x0 - tip_x) * t)
        rx = round(tip_x + (x1 - tip_x) * t)
        c.rect(min(lx, rx), y, max(lx, rx), y, col)


def draw(c, fox):
    # ── Legs (rows 26..30; same columns as the heroine so the walk lift lines up) ──
    for x0 in (7, 14):
        c.rect(x0, 26, x0 + 2, 28, "D")
        c.px(x0, 27, "n")
        c.rect(x0, 29, x0 + 2, 30, "B")
    # ── Body ──
    c.rect(6, 17, 17, 25, "N")
    c.rect(15, 17, 17, 25, "n")          # shaded side
    c.rect(11, 17, 12, 20, "J")          # front fold
    c.rect(6, 21, 17, 22, "G")           # sash
    c.rect(6, 22, 17, 22, "g")
    c.rect(9, 23, 10, 24, "G")           # sash knot tails
    c.rect(7, 25, 16, 25, "n")           # hem
    # Arms: left hangs, right holds the foxfire out.
    c.rect(4, 17, 5, 22, "N")
    c.rect(4, 23, 5, 23, "S")
    c.rect(18, 17, 19, 20, "n")
    c.rect(18, 21, 19, 21, "S")
    # ── High collar + scarf ──
    c.rect(7, 15, 16, 16, "n")
    c.rect(8, 15, 15, 15, "N")
    c.rect(6, 16, 17, 16, "C")
    c.rect(16, 17, 17, 18, "C")          # scarf end trailing down the right shoulder
    c.rect(17, 19, 17, 20, "c")
    # ── Head ──
    c.ellipse(12, 8.5, 7.6, 6.8, "H")
    c.rect(7, 8, 16, 14, "S")            # face
    c.px(7, 14, "s"); c.px(16, 14, "s"); c.rect(8, 14, 15, 14, "s")
    # Shaggy spikes (crown, sides).
    tri_up(c, 4, 9, 4, 0, 4, "H")
    tri_up(c, 9, 13, 11, 0, 3, "H")
    tri_up(c, 13, 18, 18, 1, 4, "H")
    tri_up(c, 2, 5, 1, 7, 10, "H")
    tri_up(c, 18, 21, 22, 7, 10, "H")
    c.rect(4, 9, 6, 13, "H")             # side locks over the ears
    c.rect(17, 9, 19, 13, "H")
    c.rect(4, 12, 5, 13, "h"); c.rect(18, 12, 19, 13, "h")
    # Bangs: jagged fringe over the forehead.
    for x, d in zip(range(7, 17), [9, 8, 10, 8, 9, 10, 8, 9, 8, 9]):
        c.rect(x, 7, x, d, "H")
    for x in (8, 11, 14):
        c.px(x, 8, "h")
    c.rect(9, 3, 11, 3, "L"); c.rect(8, 4, 9, 4, "L")     # shine
    # Eyes: determined, amber.
    for ex in (8, 13):
        c.rect(ex, 10, ex + 2, 10, "E")
        c.px(ex, 11, "E"); c.px(ex + 1, 11, "A"); c.px(ex + 2, 11, "A")
        c.px(ex + 1, 12, "A"); c.px(ex + 2, 12, "E")
        c.px(ex + 2, 11, "W")
    c.rect(11, 13, 12, 13, "s")          # mouth (small confident grin)
    c.px(12, 13, "h")
    # ── Fox mask tilted on the left of his head ──
    c.ellipse(4.5, 4.5, 3.2, 2.8, "M")
    c.px(2, 1, "M"); c.px(2, 2, "M"); c.px(6, 1, "M"); c.px(6, 2, "M")   # ears
    c.px(2, 2, "r"); c.px(6, 2, "r")
    c.px(3, 4, "r"); c.px(5, 4, "r")                                      # eye marks
    c.rect(4, 6, 4, 6, "r")                                               # nose
    c.rect(2, 6, 3, 6, "m"); c.rect(5, 6, 6, 6, "m")
    # ── Foxfire in the right hand ──
    c.ellipse(20.5, 19.5, 2.4, 2.6, "q")
    c.ellipse(20.5, 19.8, 1.6, 1.8, "f")
    c.px(20, 20, "F"); c.px(21, 19, "F")
    c.px(21, 16, "f"); c.px(20, 17, "f"); c.px(22, 17, "q")               # flame tip
    if fox:
        # Flame crests (upward chevrons) across the upper chest, and a flaring cloak hem.
        for x0 in (7, 13):
            c.px(x0, 19, "X"); c.px(x0 + 1, 18, "X"); c.px(x0 + 2, 19, "X")
            c.px(x0 + 1, 17, "X")
        c.rect(6, 20, 17, 20, "J")
        c.px(5, 24, "N"); c.px(18, 24, "N"); c.px(4, 24, "n"); c.px(19, 23, "n")
        c.px(3, 22, "J"); c.px(20, 22, "J")


def weapon_art():
    """Icons for Riku's Fox-tagged weapons, plus the familiar and shuriken sprites."""
    import math
    import gen_icons
    from gen_icons import Icon

    # Foxfire Orb: blue-white flame ball with a gathering swirl.
    i = Icon({"q": "#1a52c8", "f": "#3a9cff", "F": "#bff0ff", "W": "#ffffff"})
    i.circle(8, 9.5, 5.6, "q")
    i.circle(8, 9.5, 4.2, "f")
    i.poly([(8, 0.5), (11.5, 6), (4.5, 6)], "f")
    i.circle(7, 9, 2.4, "F")
    i.px(6, 8, "W")
    for k in range(6):
        a = k * math.pi / 3
        i.px(round(8 + 6.8 * math.cos(a)), round(9.5 + 6.8 * math.sin(a)), "F")
    i.save("foxfire_orb", "Foxfire Orb")

    # Kitsune Familiars: a small flame fox head.
    i = Icon({"O": "#ff9a2a", "o": "#d4560e", "W": "#fff2d0", "E": "#2a0a00"})
    i.poly([(2, 1), (6.5, 6), (2.5, 8)], "O")
    i.poly([(14, 1), (9.5, 6), (13.5, 8)], "O")
    i.circle(8, 9, 5.2, "O")
    i.poly([(3.5, 10), (12.5, 10), (8, 15.5)], "W")
    i.px(5, 8, "E"); i.px(10, 8, "E")
    i.px(8, 14, "E")
    i.px(3, 3, "o"); i.px(12, 3, "o")
    i.save("familiars", "Kitsune Familiars")

    # Gale Shuriken: four-bladed star with a ring.
    i = Icon({"S": "#c8d0e0", "s": "#7a8298", "W": "#ffffff", "C": "#c42a3a"})
    i.poly([(8, 0.5), (10, 6), (8, 8), (6, 6)], "S")
    i.poly([(15.5, 8), (10, 10), (8, 8), (10, 6)], "S")
    i.poly([(8, 15.5), (6, 10), (8, 8), (10, 10)], "s")
    i.poly([(0.5, 8), (6, 6), (8, 8), (6, 10)], "s")
    i.circle(8, 8, 2.0, "C")
    i.px(8, 8, "K")
    i.px(8, 2, "W"); i.px(14, 8, "W")
    i.save("gale_shuriken", "Gale Shuriken")

    # Fox Mirage: a fox mask with two ghostly copies behind it.
    i = Icon({"M": "#f4f0fa", "m": "#b8b0c8", "r": "#e0263c", "g": "#6a8ad8", "G": "#9ab8ff"})
    for ox, col in ((-4, "g"), (-2, "G")):
        i.circle(9 + ox, 9, 4.6, col)
        i.poly([(5 + ox, 2), (8 + ox, 6), (5 + ox, 7)], col)
        i.poly([(13 + ox, 2), (10 + ox, 6), (13 + ox, 7)], col)
    i.circle(10, 9, 4.6, "M")
    i.poly([(6, 2), (9, 6), (6, 7)], "M")
    i.poly([(14, 2), (11, 6), (14, 7)], "M")
    i.px(7, 4, "r"); i.px(13, 4, "r")
    i.px(8, 9, "r"); i.px(12, 9, "r")
    i.px(10, 11, "r")
    i.rect(8, 13, 12, 13, "m")
    i.save("fox_mirage", "Fox Mirage")

    # In-world sprites.
    c = Canvas(12, 10, {"O": "#ff9a2a", "o": "#d4560e", "W": "#fff2d0", "E": "#2a0a00", "Y": "#ffd36a"})
    c.rect(1, 2, 2, 4, "O"); c.rect(6, 2, 7, 4, "O")     # ears
    c.ellipse(4.5, 5.5, 3.6, 2.8, "O")                     # head
    c.rect(3, 6, 6, 7, "W")                                # muzzle
    c.px(3, 5, "E"); c.px(6, 5, "E")
    c.rect(8, 4, 10, 5, "Y"); c.px(10, 3, "Y"); c.px(9, 6, "o")   # flame tail
    c.save("familiar", "Kitsune familiar", outline=True)

    c = Canvas(16, 16, {"S": "#c8d0e0", "s": "#7a8298", "W": "#ffffff", "C": "#c42a3a"})
    for y in range(16):
        for x in range(16):
            dx, dy = x - 7.5, y - 7.5
            r = math.hypot(dx, dy)
            a = math.atan2(dy, dx)
            for k in range(4):
                # Each blade sweeps back as it reaches out (pinwheel), and tapers to a point.
                centre = k * math.pi / 2 + r * 0.12
                d = (a - centre + math.pi) % (2 * math.pi) - math.pi
                if r <= 7.6 and -0.15 <= d <= 0.75 * (1 - r / 8.0) + 0.05:
                    c.px(x, y, "S" if d < 0.25 else "s")
    c.ellipse(8, 8, 2.2, 2.2, "C")
    c.px(7, 7, "W"); c.px(8, 8, "K")
    c.save("shuriken", "Gale Shuriken", outline=True)


def main():
    weapon_art()
    for name, pal, fox in (("riku", BASE, False), ("riku_fox", FOX, True)):
        c = Canvas(24, 32, pal)
        draw(c, fox)
        label = "Riku Kazamine, Ninefold Foxfire form" if fox else "Riku Kazamine (fox-spirit shinobi)"
        c.save(name, label, outline=True)
    print("wrote riku.px, riku_fox.px")


if __name__ == "__main__":
    main()
