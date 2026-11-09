"""HD redraw of the phase-6 newcomers: two Uncommons and a Leader for each world.

    python3 tools/hd/enemies_p6.py [name ...]

Writes art/hd/<path>.png at 4x each .px source from tools/gen_p6.py (Uncommon 24x24, Leader 32x32):
the Shotengai trio at art/hd/<key>.png, the worlds at art/hd/w<n>/{uncommon_a,uncommon_b,leader}.png.
Leaders read as crew captains: a coat plus a hat or banner. Deterministic.
"""
import math
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from hdkit import Art, _catmull  # noqa: E402
from enemies_w12 import halo, ribbon, spark  # noqa: E402

GOLD = ("lin", "#ffe08a", "#b08a30", 90)


def shadow(a, x, y, rx, alpha=0.55):
    a.ellipse(x, y, rx, rx * 0.18, "#140f1e", alpha=alpha)


def flame(a, x, y, h, w, outer="#ff6a2a", inner="#ffe08a"):
    """A small upright flame with its base at (x, y)."""
    a.blob([(x - w, y), (x - w * 0.8, y - h * 0.45), (x - w * 0.15, y - h * 0.7), (x, y - h),
            (x + w * 0.45, y - h * 0.55), (x + w, y - h * 0.25), (x + w * 0.8, y)], ("lin", outer, "#ffb040", 90))
    a.blob([(x - w * 0.45, y - 0.1), (x - w * 0.2, y - h * 0.4), (x + w * 0.1, y - h * 0.62),
            (x + w * 0.45, y - h * 0.3), (x + w * 0.4, y - 0.1)], inner)


# ═════════════════════════ SHOTENGAI ═════════════════════════
def rokurokubi_clerk():
    # Rokurokubi Clerk: a shop clerk in an indigo apron, abacus in hand, neck looping up over the aisle.
    a = Art(24, 24)
    kimono = ("lin", "#d49a4a", "#8a5624", 90)
    skin = ("lin", "#fbefe4", "#dcc2b0", 0)
    shadow(a, 9.0, 23.0, 6.0)
    # striped kimono, pink obi, indigo shop apron with its crest
    a.blob([(3.6, 15.4), (13.2, 15.4), (14.0, 22.8), (3.0, 22.8)], kimono, smooth=False)
    a.blob([(3.0, 15.2), (3.2, 13.0), (5.0, 11.4), (12.2, 11.4), (14.0, 13.0), (14.2, 15.2)], kimono)
    for x in (4.4, 6.2, 8.0, 9.8, 11.6, 13.4):
        a.stroke([(x, 11.6), (x + (x - 8.6) * 0.06, 22.8)], 0.4, "#7a4a1e", clip=True, alpha=0.75)
    a.blob([(6.6, 11.4), (10.6, 11.4), (8.6, 14.6)], "#f4ecdc", smooth=False)
    a.blob([(7.4, 11.4), (9.8, 11.4), (8.6, 13.4)], "#c8b8a0", smooth=False)
    a.blob([(5.0, 16.6), (12.4, 16.6), (13.0, 22.6), (4.4, 22.6)], ("lin", "#3a4a8a", "#1e2650", 90), smooth=False)
    a.rect(3.4, 14.8, 13.8, 16.6, ("lin", "#ff5a9a", "#a8204a", 90), radius=0.4)
    a.ellipse(8.7, 19.6, 1.7, 1.7, "#f4ecdc")
    a.ellipse(8.7, 19.6, 1.05, 1.05, "#2a3470")
    a.stroke([(7.9, 19.6), (9.5, 19.6)], 0.4, "#f4ecdc")
    a.stroke([(8.7, 18.8), (8.7, 20.4)], 0.4, "#f4ecdc")
    # sleeves; the forward hand holds out an abacus
    a.blob([(2.2, 12.6), (5.0, 12.2), (5.4, 18.4), (2.0, 18.8)], ("lin", "#c48a3e", "#7a4a1e", 90))
    a.blob([(12.0, 12.2), (14.8, 12.6), (16.0, 16.6), (13.0, 17.6)], ("lin", "#c48a3e", "#7a4a1e", 90))
    a.rect(15.4, 14.0, 22.4, 18.6, ("lin", "#7a4a2a", "#3a2014", 90), radius=0.4)
    a.rect(16.0, 14.6, 21.8, 18.0, "#2a140c", radius=0.2)
    a.stroke([(16.0, 15.6), (21.8, 15.6)], 0.4, "#7a4a2a")
    for i in range(5):
        x = 16.7 + i * 1.12
        a.stroke([(x, 14.6), (x, 18.0)], 0.2, "#8a6a4a")
        a.ellipse(x, 15.0, 0.45, 0.32, "#ffcf6a")
        for y in ((16.5, 17.3) if i % 2 else (16.9, 17.6)):
            a.ellipse(x, y, 0.45, 0.32, "#e0503a")
    a.ellipse(15.6, 16.6, 1.0, 0.9, skin)
    # the long neck
    neck = [(8.6, 12.8), (6.4, 9.6), (7.2, 5.8), (10.4, 3.6), (13.8, 4.6), (16.2, 7.4)]
    ribbon(a, neck, lambda t: 2.4 - t * 0.5, skin)
    ribbon(a, neck, lambda t: 0.6, "#c8a898", alpha=0.7)
    # head facing right
    a.ellipse(18.2, 6.8, 3.6, 3.5, ("lin", "#3a2240", "#1e1224", 90))
    a.ellipse(19.2, 7.6, 2.9, 2.9, skin)
    a.blob([(15.0, 6.2), (16.6, 3.6), (19.6, 3.2), (22.2, 4.6), (22.0, 6.0), (19.6, 5.2), (17.6, 6.8)],
           ("lin", "#4a2c50", "#24142a", 90))
    a.ellipse(16.8, 2.8, 2.0, 1.8, ("lin", "#4a2c50", "#24142a", 90))
    a.stroke([(14.4, 3.8), (19.4, 1.4)], 0.45, "#e8c060")
    a.ellipse(19.6, 1.3, 0.7, 0.7, "#ff5aa8")
    # slit eye and too-wide smile
    a.stroke([(19.8, 7.0), (21.4, 6.6)], 0.75, "#ffb03a")
    a.stroke([(19.6, 6.3), (21.6, 5.9)], 0.35, "#1e1224")
    a.stroke([(18.6, 9.0), (20.2, 9.6), (21.6, 9.0)], 0.4, "#c0304a")
    a.ellipse(21.2, 8.4, 0.5, 0.3, "#ff8aa0", alpha=0.6)
    a.rim("#ffb0e0", 0.35)
    a.outline()
    halo(a, 20.6, 6.8, 1.4, "#ffb03a", 0.45)
    a.save("rokurokubi_clerk")


def tsurube_drop():
    # Tsurube Drop: a well-bucket on a rope, a grinning head of fire peering over its rim.
    a = Art(24, 24)
    rope = "#d8b47a"
    shadow(a, 12.0, 23.2, 4.4, 0.45)
    a.stroke([(12.0, -0.5), (12.0, 4.6)], 0.9, rope)
    a.stroke([(5.6, 13.4), (6.8, 7.8), (12.0, 4.4), (17.2, 7.8), (18.4, 13.4)], 0.7, rope)
    a.ellipse(12.0, 13.4, 6.8, 1.4, "#2a160c")
    # flaming hair
    a.blob([(6.4, 12.0), (5.4, 7.4), (7.0, 8.4), (7.2, 4.0), (9.2, 6.4), (10.4, 1.6), (12.2, 5.0), (14.0, 1.0),
            (15.0, 5.6), (17.2, 3.0), (17.0, 7.6), (18.8, 6.8), (17.8, 12.0)], ("lin", "#ff4a1a", "#ffb03a", 90))
    a.blob([(8.0, 12.0), (7.8, 8.6), (9.4, 9.0), (10.4, 5.2), (12.0, 7.6), (13.8, 4.2), (14.6, 8.4), (16.2, 7.2),
            (16.2, 12.0)], ("lin", "#ffb040", "#fff0a0", 90))
    # the head
    a.ellipse(12.0, 10.6, 4.4, 3.8, ("lin", "#f8b080", "#d0603a", 90))
    a.ellipse(12.0, 8.6, 3.6, 1.2, "#ffd27a", clip=True, alpha=0.5)
    for ex in (10.0, 14.0):
        a.ellipse(ex, 9.8, 1.2, 1.25, "#fffaf0")
        a.ellipse(ex + 0.25, 10.0, 0.45, 0.5, "#2a1010")
    a.stroke([(8.6, 8.2), (10.8, 8.9)], 0.6, "#5a1a10")
    a.stroke([(15.4, 8.2), (13.2, 8.9)], 0.6, "#5a1a10")
    a.blob([(9.4, 11.8), (14.6, 11.8), (13.6, 13.6), (10.4, 13.6)], "#4a1010")
    for x in (10.4, 11.6, 12.8, 14.0):
        a.poly([(x - 0.45, 11.8), (x + 0.45, 11.8), (x, 12.8)], "#fffaf0")
    # bucket
    a.poly([(5.0, 13.4), (19.0, 13.4), (17.4, 21.2), (6.6, 21.2)], ("lin", "#b8804a", "#6a4220", 90))
    for x in (8.4, 10.6, 12.8, 15.0):
        a.stroke([(x + (12 - x) * 0.04, 13.8), (x + (12 - x) * 0.12, 21.0)], 0.3, "#5a3418", clip=True, alpha=0.8)
    a.poly([(5.0, 13.4), (8.0, 13.4), (8.6, 21.2), (6.6, 21.2)], "#4a2a14", clip=True, alpha=0.4)
    for y, x0, x1 in ((15.0, 5.4, 18.6), (19.6, 6.3, 17.7)):
        a.stroke([(x0, y), (x1, y)], 0.9, ("lin", "#5a5a6a", "#2a2a36", 90))
        a.stroke([(x0 + 0.3, y - 0.25), (x1 - 0.3, y - 0.25)], 0.25, "#a8a8ba", alpha=0.8)
    a.ellipse(12.0, 13.4, 7.0, 0.7, "#d8a464")
    a.ellipse(5.4, 13.6, 0.6, 0.6, "#5a5a6a")
    a.ellipse(18.6, 13.6, 0.6, 0.6, "#5a5a6a")
    a.rim("#fff0c0", 0.3)
    a.outline()
    halo(a, 12.0, 6.0, 5.0, "#ff8a3a", 0.35)
    for x, y in ((4.4, 4.6), (19.8, 3.6), (16.6, 0.8)):
        spark(a, x, y, 0.7, "#ffcf6a")
    a.save("tsurube_drop")


def captain_kitsunebi():
    # Captain Kitsunebi: a fox captain in a crimson coat and tricorn, flame banner on a pole, lantern in hand.
    a = Art(32, 32)
    fur = ("lin", "#f8a050", "#c8541e", 90)
    coat = ("lin", "#b8283e", "#5a1024", 90)
    shadow(a, 17.0, 30.8, 9.0)
    # tails
    for pts, w in (([(12.0, 22.0), (7.0, 21.0), (3.8, 17.6), (3.4, 13.4)], 3.4),
                   ([(12.0, 25.0), (7.2, 26.6), (3.4, 25.8), (1.6, 22.6)], 3.0)):
        path, _ = ribbon(a, pts, lambda t, w=w: w * math.sin(math.pi * min(0.94, 0.2 + t)) + 0.6, fur)
        ex, ey = path[-1]
        a.ellipse(ex + 0.3, ey + 0.6, 1.5, 1.7, "#fff4e0")
        flame(a, ex + 0.2, ey + 0.2, 2.6, 1.0)
    # banner pole and flame flag
    a.stroke([(5.0, 1.4), (5.0, 30.0)], 0.9, ("lin", "#7a5034", "#3a2418", 0))
    a.ellipse(5.0, 1.2, 0.9, 0.9, GOLD)
    a.blob([(5.2, 2.0), (8.6, 1.4), (11.4, 2.4), (13.6, 1.8), (12.6, 4.6), (14.4, 7.2), (11.0, 7.6),
            (8.4, 9.2), (5.2, 9.0)], ("lin", "#ff8a2a", "#c8201e", 0))
    a.blob([(5.2, 7.4), (8.4, 7.6), (11.0, 7.6), (8.4, 9.2), (5.2, 9.0)], "#8a1418", clip=True, alpha=0.5)
    a.ellipse(8.8, 5.2, 1.9, 1.9, "#fff4e0")
    flame(a, 8.8, 6.4, 2.6, 1.0, "#ff4a1a", "#ffb03a")
    # boots
    a.rect(13.0, 26.4, 16.2, 30.6, ("lin", "#2a2030", "#141018", 90), radius=0.7)
    a.rect(18.6, 26.4, 22.0, 30.6, ("lin", "#2a2030", "#141018", 90), radius=0.7)
    a.rect(12.4, 29.2, 16.8, 30.8, "#141018", radius=0.6)
    a.rect(18.2, 29.2, 23.2, 30.8, "#141018", radius=0.6)
    # long coat
    a.blob([(11.4, 13.0), (23.0, 13.0), (25.0, 21.0), (26.6, 28.0), (21.0, 28.4), (17.6, 26.6), (14.0, 28.4),
            (9.4, 28.0), (10.4, 20.0)], coat)
    a.blob([(9.0, 13.0), (13.6, 13.0), (13.6, 28.6), (9.0, 28.6)], "#3a0818", clip=True, alpha=0.35)
    a.stroke([(9.8, 27.6), (13.8, 28.0), (17.6, 26.4), (21.2, 28.0), (26.2, 27.6)], 0.6, GOLD)
    a.blob([(16.6, 13.2), (19.6, 13.2), (19.2, 22.0), (17.0, 22.0)], ("lin", "#fff4e0", "#d8c8b0", 90))
    a.blob([(16.4, 13.0), (19.8, 13.0), (18.1, 16.0)], "#fff8ec")
    for y in (15.6, 17.6, 19.6):
        a.ellipse(20.4, y, 0.5, 0.5, GOLD)
        a.ellipse(15.8, y, 0.5, 0.5, GOLD)
    a.stroke([(16.4, 13.4), (16.0, 27.0)], 0.5, GOLD)
    a.stroke([(19.8, 13.4), (20.4, 26.6)], 0.5, GOLD)
    a.rect(10.6, 20.6, 24.6, 21.8, ("lin", "#3a2418", "#1e120c", 90), radius=0.3)
    a.rect(17.0, 20.3, 19.4, 22.1, GOLD, radius=0.3)
    # back arm grips the pole
    a.stroke([(12.4, 14.4), (9.4, 16.4), (6.4, 17.2)], 1.9, ("lin", "#a01c34", "#5a1024", 90))
    a.ellipse(5.4, 17.2, 1.2, 1.1, "#fff4e0")
    # front arm holds a paper lantern
    a.stroke([(23.0, 14.4), (25.0, 17.6), (25.6, 19.6)], 1.9, coat)
    a.ellipse(25.8, 20.2, 1.1, 1.0, "#fff4e0")
    a.stroke([(25.8, 20.6), (25.8, 21.8)], 0.3, "#3a2418")
    a.rect(24.2, 21.6, 27.4, 22.2, "#2a1a1a", radius=0.2)
    a.ellipse(25.8, 23.8, 1.8, 2.0, ("rad", "#fff0a0", "#ff7a2a"))
    for y in (23.0, 24.6):
        a.stroke([(24.2, y), (27.4, y)], 0.25, "#c8401e", clip=True, alpha=0.7)
    a.rect(24.4, 25.6, 27.2, 26.2, "#2a1a1a", radius=0.2)
    # epaulettes
    for x in (12.0, 22.8):
        a.ellipse(x, 13.6, 2.0, 1.1, GOLD)
        for k in range(4):
            a.stroke([(x - 1.4 + k * 0.95, 14.2), (x - 1.5 + k * 0.95, 15.6)], 0.35, "#e0b040")
    # fox head facing right
    a.ellipse(18.6, 9.0, 4.6, 4.4, fur)
    a.blob([(14.6, 9.6), (17.0, 13.6), (21.0, 13.0), (19.6, 10.6)], "#fff4e6")
    a.blob([(19.6, 7.8), (24.4, 8.6), (27.4, 10.0), (26.8, 11.2), (24.0, 12.0), (20.6, 12.6)], fur)
    a.blob([(20.4, 10.6), (26.8, 10.8), (24.6, 12.4), (20.6, 12.8)], "#fff4e6")
    a.ellipse(27.2, 10.2, 0.8, 0.65, "#1a1018")
    a.stroke([(23.0, 11.4), (25.6, 11.6)], 0.3, "#8a4a3a")
    a.blob([(20.8, 7.9), (23.2, 7.5), (22.6, 8.7), (21.2, 8.7)], "#ffd23a")
    a.ellipse(22.2, 8.2, 0.25, 0.45, "#1a1018")
    a.stroke([(20.4, 7.9), (23.6, 7.3)], 0.5, "#1a1018")
    a.stroke([(20.4, 9.6), (22.0, 10.0), (23.0, 9.6)], 0.45, "#e02a2a", taper=True)
    a.stroke([(17.0, 6.4), (18.6, 7.6)], 0.5, "#e02a2a", taper=True)
    # tricorn with ears through it
    for pts in ([(15.0, 4.4), (15.4, 0.4), (17.8, 3.4)], [(20.0, 3.4), (22.4, 0.3), (23.0, 4.4)]):
        a.poly(pts, fur)
        cx = sum(p[0] for p in pts) / 3
        a.poly([(cx - 0.6, 3.6), (pts[1][0], pts[1][1] + 0.9), (cx + 0.6, 3.6)], "#3a1a1a", alpha=0.85)
    a.blob([(12.6, 5.8), (15.6, 3.6), (19.2, 3.0), (23.4, 3.6), (26.0, 5.4), (23.6, 6.2), (19.2, 5.4),
            (15.0, 6.8)], ("lin", "#3a2e58", "#1a1428", 90))
    a.stroke([(13.0, 5.6), (15.6, 3.8), (19.2, 3.2), (23.4, 3.8), (25.6, 5.4)], 0.5, GOLD)
    flame(a, 18.4, 4.6, 2.0, 0.8)
    a.rim("#ffd0a0", 0.35)
    a.outline()
    halo(a, 25.8, 23.8, 2.8, "#ffb040", 0.5)
    halo(a, 9.0, 5.0, 3.0, "#ff7a2a", 0.3)
    halo(a, 22.0, 8.2, 1.2, "#ffd23a", 0.4)
    for x, y in ((29.6, 3.6), (30.4, 14.6)):
        spark(a, x, y, 0.8, "#ffb040")
    a.save("captain_kitsunebi")


# ═════════════════════════ SHRINE VALLEY ═════════════════════════
def w1_uncommon_a():
    # Shard Moth: wings of cut moonlight in hard facets, a stolen jewel shard glowing on its thorax.
    a = Art(24, 24)
    fore = [((11.4, 9.2), (0.8, 2.2), (5.4, 8.4)), ((5.4, 8.4), (0.8, 2.2), (0.4, 9.0)),
            ((5.4, 8.4), (0.4, 9.0), (3.0, 12.8)), ((11.4, 9.2), (5.4, 8.4), (3.0, 12.8), (10.8, 12.0))]
    hind = [((11.0, 12.0), (3.6, 13.4), (6.0, 16.0)), ((6.0, 16.0), (3.6, 13.4), (3.4, 18.8)),
            ((11.0, 12.0), (6.0, 16.0), (3.4, 18.8), (7.4, 21.0), (11.0, 15.8))]
    tones = ["#f4f8ff", "#b8c4f0", "#7a84c8", "#d8c4f8"]
    for side in (1, -1):
        def m(p):
            return [(12 + (x - 12) * side, y) for x, y in p]
        for k, f in enumerate(hind):
            a.poly(m(f), ("lin", tones[(k + 1) % 4], "#8a90c8", 90))
        for k, f in enumerate(fore):
            a.poly(m(f), ("lin", tones[k], "#a4b0e4", 60))
        a.stroke(m([(11.4, 9.2), (0.8, 2.2)]), 0.4, "#ffffff", alpha=0.9)
        a.stroke(m([(5.4, 8.4), (0.8, 2.2)]), 0.25, "#ffffff", alpha=0.6)
        a.stroke(m([(5.4, 8.4), (3.0, 12.8)]), 0.25, "#5a5aa0", alpha=0.7)
        a.stroke(m([(11.0, 12.0), (3.4, 18.8)]), 0.25, "#ffffff", alpha=0.6)
        a.ellipse(12 + (4.4 - 12) * side, 6.4, 0.9, 0.9, "#9affd8", alpha=0.85)
        a.stroke(m([(11.4, 8.0), (9.6, 4.8), (8.4, 2.6)]), 0.4, "#c8c0e8")
        for t in range(3):
            x, y = 11.0 - t * 1.0, 7.0 - t * 1.6
            a.stroke(m([(x, y), (x - 0.9, y - 0.2)]), 0.25, "#c8c0e8")
    # body
    a.ellipse(12.0, 15.6, 1.4, 3.6, ("lin", "#5a5490", "#2a2450", 90))
    for y in (14.0, 15.6, 17.2):
        a.stroke([(10.8, y), (13.2, y)], 0.25, "#8a84c0", clip=True)
    a.ellipse(12.0, 11.0, 1.9, 2.6, ("lin", "#d8d4f4", "#6a64a0", 90))
    a.ellipse(12.0, 8.2, 1.4, 1.2, "#4a4480")
    a.ellipse(11.3, 7.9, 0.4, 0.4, "#9affd8")
    a.ellipse(12.7, 7.9, 0.4, 0.4, "#9affd8")
    a.poly([(12.0, 9.6), (13.1, 11.2), (12.0, 13.0), (10.9, 11.2)], ("lin", "#ffd0ea", "#e0408a", 90))
    a.poly([(12.0, 9.6), (13.1, 11.2), (12.0, 11.0)], "#ffffff", alpha=0.7)
    a.rim("#ffffff", 0.3)
    a.outline()
    halo(a, 12.0, 11.2, 2.0, "#ff6ab0", 0.5)
    spark(a, 1.4, 2.6, 1.0, "#d8f4ff")
    spark(a, 21.6, 19.6, 0.8, "#9affd8")
    a.save("w1/uncommon_a")


def w1_uncommon_b():
    # Bone-Eater Crow: a hunched black crow in a bird-skull mask, shrine rope at its neck, a bone underfoot.
    a = Art(24, 24)
    black = ("lin", "#3a3a52", "#14141e", 90)
    bone = ("lin", "#f6f0dc", "#b8ae90", 90)
    shadow(a, 11.6, 22.8, 7.0)
    # raised back wing
    a.blob([(9.6, 10.4), (6.2, 5.0), (2.2, 1.4), (1.2, 4.6), (1.6, 9.0), (3.4, 13.4), (9.4, 15.0)], black)
    for k in range(5):
        x, y = 1.4 + k * 0.6, 5.0 + k * 2.2
        a.poly([(x + 0.6, y - 1.2), (x - 1.2, y + 1.4), (x + 1.6, y + 0.6)], "#1e1e2c")
    for p in ([(8.6, 10.6), (3.2, 3.2)], [(8.6, 12.0), (2.4, 7.4)], [(8.6, 13.4), (2.8, 11.4)]):
        a.stroke(p, 0.3, "#5a6a9a", alpha=0.8)
    # tail
    a.poly([(6.0, 15.4), (0.4, 18.6), (0.8, 20.4), (7.0, 18.2)], "#1e1e2c")
    a.poly([(6.4, 16.6), (1.4, 21.0), (2.6, 21.6), (7.6, 18.6)], "#2a2a3e")
    # legs and the bone
    for x in (10.6, 14.0):
        a.stroke([(x, 18.4), (x - 0.2, 21.2)], 0.6, "#8a8270")
        for dx in (-1.2, 0.0, 1.2):
            a.stroke([(x - 0.2, 21.2), (x - 0.2 + dx, 21.9)], 0.35, "#8a8270")
    a.stroke([(8.6, 21.8), (16.4, 21.4)], 1.0, bone)
    for x, y in ((8.4, 21.8), (16.6, 21.4)):
        a.ellipse(x, y - 0.4, 0.6, 0.55, "#f6f0dc")
        a.ellipse(x, y + 0.4, 0.6, 0.55, "#d8d0b8")
    # body and folded near wing
    a.blob([(5.4, 13.0), (10.4, 9.4), (15.6, 9.4), (18.0, 12.6), (17.0, 17.4), (12.4, 19.2), (7.4, 18.6),
            (3.6, 16.6)], black)
    a.blob([(8.0, 12.0), (14.6, 11.4), (15.6, 14.6), (10.4, 17.4), (5.0, 17.4)], ("lin", "#2e2e44", "#18182a", 90))
    for k in range(3):
        a.stroke([(13.6 - k * 2.2, 12.2 + k * 0.6), (9.8 - k * 2.0, 16.6 + k * 0.2)], 0.3, "#5a6a9a", alpha=0.8)
    a.stroke([(10.6, 10.2), (16.2, 10.4)], 0.4, "#6a7ab0", alpha=0.6)
    # head, skull mask and beak
    a.ellipse(17.6, 9.0, 3.0, 2.8, black)
    a.blob([(15.4, 6.6), (18.8, 5.6), (21.0, 7.0), (23.8, 10.0), (21.0, 10.8), (16.6, 11.2), (15.0, 9.2)], bone)
    a.blob([(18.6, 9.6), (23.8, 10.0), (21.0, 10.8), (18.0, 10.8)], "#a89e80", alpha=0.8)
    a.stroke([(19.0, 9.7), (23.4, 10.0)], 0.25, "#6a6050")
    a.ellipse(18.6, 8.1, 1.1, 1.0, "#14141e")
    a.ellipse(18.8, 8.2, 0.5, 0.5, "#7affd0")
    a.stroke([(16.4, 7.2), (17.2, 8.4), (16.6, 9.6)], 0.25, "#8a8270")
    a.ellipse(16.4, 8.0, 0.5, 0.6, "#6a6050", alpha=0.6)
    a.ellipse(17.0, 6.6, 0.5, 0.3, "#5a8a4a", alpha=0.9)
    # shrine rope and paper streamer
    a.stroke([(14.6, 9.6), (15.4, 11.4), (16.4, 12.4)], 0.9, ("lin", "#f0dca0", "#b89a5a", 90))
    a.blob([(15.6, 12.2), (16.8, 12.2), (16.2, 13.6), (17.2, 13.8), (16.0, 15.8), (16.0, 14.4), (15.2, 14.2)],
           "#fffaf0", smooth=False)
    a.rim("#a0b0e8", 0.35)
    a.outline()
    halo(a, 18.8, 8.2, 1.4, "#7affd0", 0.5)
    a.save("w1/uncommon_b")


def w1_leader():
    # Admiral Tengu: red long-nosed tengu in a bicorne and gold-braided coat, crow wings, a leaf fan, one-tooth geta.
    a = Art(32, 32)
    red = ("lin", "#f0644a", "#b0302a", 90)
    coat = ("lin", "#7a5a34", "#3a2616", 90)
    wing = ("lin", "#3e3a52", "#16141e", 90)
    shadow(a, 16.0, 31.0, 8.0)
    # wings
    a.blob([(12.0, 12.6), (8.0, 7.6), (3.4, 3.4), (0.6, 4.0), (0.8, 9.6), (1.8, 15.6), (4.4, 20.4), (10.6, 18.0)], wing)
    a.blob([(20.0, 12.6), (24.0, 7.2), (28.6, 3.0), (31.4, 4.0), (31.2, 10.0), (30.0, 16.0), (27.6, 20.2),
            (21.4, 18.0)], wing)
    for k in range(4):
        a.stroke([(10.6, 13.0 + k * 1.4), (1.6 + k * 0.6, 5.0 + k * 4.0)], 0.35, "#5a5a7a", alpha=0.8)
        a.stroke([(21.4, 13.0 + k * 1.4), (30.4 - k * 0.6, 5.0 + k * 4.0)], 0.35, "#5a5a7a", alpha=0.8)
    for k in range(4):
        y = 12.0 + k * 2.6
        a.poly([(1.2 + k * 0.6, y - 1.2), (0.4 + k * 1.0, y + 2.2), (2.8 + k * 0.6, y + 0.8)], "#16141e")
        a.poly([(30.8 - k * 0.6, y - 1.2), (31.6 - k * 1.0, y + 2.2), (29.2 - k * 0.6, y + 0.8)], "#16141e")
    # one-tooth geta and legs
    for x0, x1 in ((11.4, 15.4), (17.6, 21.6)):
        a.rect(x0 + 0.6, 26.0, x1 - 0.6, 29.0, ("lin", "#f4f0e8", "#b8b0a8", 0), radius=0.6)
        a.rect(x0, 28.8, x1, 29.8, ("lin", "#a8743a", "#6a4220", 90), radius=0.3)
        a.rect((x0 + x1) / 2 - 0.5, 29.6, (x0 + x1) / 2 + 0.5, 31.4, "#5a3418")
    # coat with gold buttons and yamabushi pom-poms
    a.blob([(10.4, 12.8), (21.6, 12.8), (23.4, 20.0), (23.6, 26.6), (16.0, 27.4), (8.4, 26.6), (8.8, 20.0)], coat)
    a.blob([(8.0, 12.8), (12.2, 12.8), (12.2, 27.4), (8.0, 27.4)], "#24160c", clip=True, alpha=0.35)
    a.stroke([(8.6, 26.4), (16.0, 27.2), (23.4, 26.4)], 0.6, GOLD)
    a.stroke([(16.0, 13.2), (16.0, 27.0)], 0.4, "#24160c")
    for y in (15.2, 18.2, 21.2, 24.2):
        a.ellipse(13.8, y, 0.45, 0.45, GOLD)
        a.ellipse(18.2, y, 0.45, 0.45, GOLD)
    for y in (15.4, 18.4, 21.4):
        a.ellipse(16.0, y, 1.2, 1.2, ("rad", "#ff9ab0", "#c82a44"))
    a.rect(9.2, 22.4, 22.8, 23.4, ("lin", "#2a1c10", "#140c08", 90), radius=0.3)
    # back arm and front arm with the leaf fan
    a.stroke([(10.8, 14.0), (8.8, 18.0), (8.8, 21.0)], 1.9, coat)
    a.ellipse(8.8, 21.6, 1.0, 1.0, "#e0503a")
    a.stroke([(21.2, 14.0), (24.0, 17.0), (25.6, 17.8)], 1.9, coat)
    a.ellipse(26.0, 18.0, 1.0, 1.0, "#e0503a")
    a.stroke([(26.0, 18.0), (27.2, 13.6)], 0.6, "#5a3418")
    for deg in (-170, -140, -114, -90, -66, -40, -10):
        t = math.radians(deg)
        ux, uy = math.cos(t), math.sin(t)
        cx, cy = 27.2, 12.4
        a.blob([(cx, cy), (cx + ux * 2.4 - uy * 1.2, cy + uy * 2.4 + ux * 1.2), (cx + ux * 4.6, cy + uy * 4.6),
                (cx + ux * 2.4 + uy * 1.2, cy + uy * 2.4 - ux * 1.2)], ("lin", "#9ad070", "#3a7a2a", 90))
        a.stroke([(cx, cy), (cx + ux * 3.8, cy + uy * 3.8)], 0.25, "#2a5a1e", alpha=0.8)
    a.ellipse(27.2, 12.4, 1.2, 1.0, "#3a7a2a")
    # epaulettes
    for x in (10.8, 21.2):
        a.ellipse(x, 13.4, 2.0, 1.1, GOLD)
        for k in range(4):
            a.stroke([(x - 1.4 + k * 0.95, 14.0), (x - 1.5 + k * 0.95, 15.4)], 0.35, "#e0b040")
    # wild white mane and beard
    hair = ("lin", "#fffaf0", "#c8c0b8", 90)
    a.poly([(14.4, 4.6), (10.0, 4.0), (11.8, 6.2), (8.2, 7.0), (11.0, 8.8), (8.0, 10.8), (11.0, 11.4),
            (9.4, 14.0), (13.2, 13.0), (14.4, 14.4), (15.4, 10.0)], hair)
    a.poly([(14.0, 10.8), (20.6, 10.8), (19.6, 13.0), (20.0, 15.2), (17.8, 13.8), (16.4, 15.8), (15.6, 13.6),
            (13.4, 14.6)], hair)
    for p in ([(13.4, 6.0), (10.4, 7.4)], [(13.0, 9.0), (9.6, 10.6)], [(16.6, 12.0), (16.4, 14.6)]):
        a.stroke(p, 0.3, "#a8a098", alpha=0.8)
    # red face, angry brow and the long nose
    a.ellipse(16.6, 8.8, 3.6, 3.4, red)
    a.blob([(15.0, 10.4), (19.4, 10.2), (18.6, 12.0), (15.6, 12.0)], "#a8302a", clip=True, alpha=0.5)
    a.stroke([(19.0, 9.4), (23.0, 9.6), (26.6, 9.8)], 1.9, red)
    a.ellipse(26.6, 9.8, 1.0, 0.95, "#e0503a")
    a.stroke([(19.6, 8.8), (26.2, 9.2)], 0.4, "#ffa080", alpha=0.85)
    a.ellipse(18.8, 8.0, 1.2, 0.85, "#3a0e0a")
    a.ellipse(19.0, 8.1, 0.8, 0.55, "#ffd23a")
    a.ellipse(19.2, 8.1, 0.3, 0.42, "#1a1018")
    a.poly([(15.6, 5.4), (20.8, 6.6), (20.4, 7.4), (16.2, 6.8)], "#fffaf0")
    # bicorne worn athwart, gold edged, with a cockade
    a.blob([(9.6, 6.2), (11.4, 2.4), (15.8, 0.6), (20.6, 1.6), (23.4, 5.4), (20.0, 4.8), (16.2, 5.0), (12.6, 5.8)],
           ("lin", "#2e2a40", "#141020", 90))
    a.stroke([(10.2, 5.8), (11.6, 2.6), (15.8, 1.0), (20.4, 2.0), (23.0, 5.2)], 0.55, GOLD)
    a.ellipse(17.6, 2.8, 1.1, 1.1, ("rad", "#ffe08a", "#c8401e"))
    a.rim("#ffd8b0", 0.3)
    a.outline()
    halo(a, 19.0, 8.1, 1.2, "#ffd23a", 0.4)
    a.save("w1/leader")


# ═════════════════════════ EMBER LEAF ═════════════════════════
def w2_uncommon_a():
    # Rogue Puppeteer: a marionette that cut its own strings, now working a knife-doll on a control bar.
    a = Art(24, 24)
    cloak = ("lin", "#4a6050", "#1e2a24", 90)
    wood = ("lin", "#e8c494", "#a87a4a", 90)
    shadow(a, 11.0, 23.0, 7.4)
    # cut strings drifting up from its own joints
    for p in ([(7.4, 3.6), (6.6, 1.6), (7.4, 0.2)], [(4.2, 10.0), (2.8, 6.6), (3.4, 4.0)],
              [(12.6, 10.6), (13.0, 8.6)]):
        a.stroke(p, 0.3, "#e8e4f0", alpha=0.85)
    # cloak and hood
    a.blob([(4.4, 9.0), (11.6, 8.6), (13.2, 15.0), (14.0, 21.4), (12.0, 22.6), (10.4, 21.6), (8.4, 22.8),
            (6.2, 21.6), (4.0, 22.6), (2.6, 21.0), (3.2, 15.0)], cloak)
    a.blob([(2.4, 9.0), (6.0, 9.0), (6.0, 23.0), (2.4, 23.0)], "#141c18", clip=True, alpha=0.4)
    a.stroke([(8.0, 10.4), (8.6, 21.4)], 0.35, "#141c18", alpha=0.7)
    a.stroke([(3.4, 15.6), (13.0, 15.0)], 0.7, "#a02a2a")
    a.ellipse(8.4, 6.6, 4.0, 3.9, cloak)
    # wooden marionette face with a hinged jaw
    a.ellipse(10.0, 7.4, 2.4, 2.7, wood)
    a.stroke([(9.0, 8.6), (9.0, 10.0)], 0.3, "#5a3a20")
    a.stroke([(11.0, 8.6), (11.0, 10.0)], 0.3, "#5a3a20")
    a.ellipse(9.4, 6.6, 0.55, 0.55, "#2a1a10")
    a.ellipse(11.4, 6.6, 0.6, 0.6, "#ff4a2a")
    a.stroke([(11.0, 5.4), (12.2, 6.0)], 0.35, "#a02a2a")
    a.blob([(5.0, 4.4), (8.6, 2.4), (12.0, 3.6), (12.4, 5.0), (9.0, 4.4), (6.4, 6.6)], ("lin", "#5a7460", "#2a3a30", 90))
    # raised arm with the control bar
    a.stroke([(10.8, 10.4), (13.6, 8.6), (15.0, 7.0)], 1.5, cloak)
    a.ellipse(15.4, 6.4, 0.85, 0.8, wood)
    a.stroke([(13.4, 5.0), (19.6, 5.6)], 0.8, ("lin", "#c89a5a", "#7a5030", 90))
    a.stroke([(16.6, 3.4), (16.4, 8.0)], 0.8, ("lin", "#c89a5a", "#7a5030", 0))
    for p in ([(13.6, 5.2), (18.2, 14.0)], [(19.4, 5.8), (21.6, 15.4)], [(16.4, 7.8), (19.6, 12.8)]):
        a.stroke(p, 0.22, "#e8e4f0", alpha=0.8)
    # knife-doll
    a.blob([(18.0, 15.2), (21.4, 15.2), (22.2, 20.4), (17.4, 20.4)], ("lin", "#d8403a", "#8a1c1c", 90))
    a.rect(17.8, 17.2, 21.8, 17.9, "#2a1a2a")
    a.stroke([(18.6, 20.4), (18.4, 21.8)], 0.6, wood)
    a.stroke([(20.8, 20.4), (21.0, 21.8)], 0.6, wood)
    a.ellipse(19.6, 13.8, 1.7, 1.7, "#fff4ea")
    a.blob([(17.8, 13.6), (18.4, 11.8), (20.8, 11.8), (21.4, 13.6), (20.8, 13.0), (18.4, 13.0)], "#1a1418")
    a.ellipse(20.4, 14.0, 0.3, 0.3, "#1a1418")
    a.stroke([(21.2, 16.2), (22.6, 16.4)], 0.6, wood)
    for ang in (-0.5, 0.0, 0.5):
        dx, dy = math.cos(ang), math.sin(ang)
        x0, y0 = 22.6, 16.4
        a.poly([(x0 - dy * 0.35, y0 + dx * 0.35), (x0 + dx * 1.8, y0 + dy * 1.8),
                (x0 + dy * 0.35, y0 - dx * 0.35)], ("lin", "#ffffff", "#9aa0ac", 90))
    a.rim("#c8e0b0", 0.3)
    a.outline()
    halo(a, 11.4, 6.6, 1.2, "#ff4a2a", 0.45)
    a.save("w2/uncommon_a")


def w2_uncommon_b():
    # Exploding Clay Bird: a fat terracotta bird with celadon glaze, cracks glowing with powder, fuse lit.
    a = Art(24, 24)
    clay = ("lin", "#d8844a", "#8a4220", 90)
    shadow(a, 12.0, 22.8, 7.0)
    for x in (9.4, 13.4):
        a.stroke([(x, 17.6), (x - 0.4, 21.0)], 1.2, "#8a4220")
        a.blob([(x - 1.8, 20.6), (x + 1.0, 20.6), (x + 1.2, 21.8), (x - 1.8, 21.8)], "#a8582a")
    # tail fan
    a.poly([(4.6, 11.0), (0.6, 7.8), (0.4, 10.6), (0.8, 13.4), (4.4, 14.2)], clay)
    a.stroke([(4.2, 12.0), (0.8, 9.2)], 0.3, "#6a3018")
    a.stroke([(4.2, 13.0), (0.8, 12.2)], 0.3, "#6a3018")
    # body
    body = [(3.4, 12.0), (6.4, 7.6), (11.0, 6.0), (16.0, 6.6), (19.4, 9.6), (19.0, 14.6), (14.6, 18.6),
            (8.0, 19.0), (4.0, 16.4)]
    a.blob(body, clay)
    a.blob([(2.0, 3.0), (22.0, 3.0), (20.6, 10.6), (18.6, 10.2), (17.4, 12.8), (15.8, 10.8), (13.6, 12.4),
            (11.6, 10.4), (9.4, 12.6), (7.4, 10.4), (5.6, 12.0), (2.0, 10.4)], ("lin", "#c4ecd0", "#5aa088", 90),
           clip=True, smooth=False)
    for x, y in ((6.6, 13.6), (8.6, 15.6), (15.2, 15.0), (12.4, 17.0), (17.0, 13.0)):
        a.ellipse(x, y, 0.35, 0.35, "#6a3018", alpha=0.8)
    a.blob([(3.0, 15.0), (20.0, 13.0), (20.0, 20.0), (3.0, 20.0)], "#5a2410", clip=True, alpha=0.35)
    # stubby wing
    a.blob([(7.4, 11.6), (13.2, 10.8), (14.6, 13.4), (11.2, 15.8), (6.8, 14.8)], ("lin", "#e8945a", "#a8582a", 90))
    for k in range(3):
        a.stroke([(8.4 + k * 1.8, 12.0), (8.0 + k * 1.6, 14.8)], 0.3, "#6a3018", alpha=0.9)
    # beak and hollow glowing eye
    a.poly([(18.6, 8.0), (23.6, 9.6), (18.8, 11.0)], ("lin", "#f0c060", "#b07a2a", 90))
    a.stroke([(19.0, 9.6), (23.0, 9.6)], 0.25, "#6a3018")
    a.ellipse(16.8, 8.4, 1.1, 1.1, "#2a1008")
    a.ellipse(16.9, 8.5, 0.6, 0.6, "#ffb030")
    # glowing powder cracks
    for p in ([(9.6, 17.6), (10.8, 16.2), (11.6, 17.4), (13.0, 16.0)], [(17.6, 12.6), (18.6, 14.2), (17.4, 15.0)],
              [(5.0, 14.0), (6.0, 15.4)]):
        a.stroke(p, 0.5, "#ffcf4a", smooth=False)
    # fuse
    a.stroke([(9.0, 6.6), (8.0, 4.4), (6.0, 3.2), (4.6, 2.0)], 0.6, "#3a2a20")
    a.ellipse(9.2, 6.6, 1.0, 0.6, "#5a3a28")
    a.rim("#fff0d0", 0.3)
    a.outline()
    for x, y in ((11.2, 16.8), (18.0, 13.8), (16.9, 8.5)):
        halo(a, x, y, 1.6, "#ff9a30", 0.45)
    spark(a, 4.4, 1.8, 1.4, "#ffcf4a")
    a.save("w2/uncommon_b")


def w2_leader():
    # Mist Swordsman Captain: lacquered jingasa, scarred open face, navy captain's jinbaori over a mist-grey
    # kimono, crew banner on his back, plain katana held low for the charge, mist curling at his feet.
    a = Art(32, 32)
    grey = ("lin", "#d4dce6", "#7a8898", 90)
    coat = ("lin", "#2e5288", "#142448", 90)
    skin = ("lin", "#ecd0b0", "#c49a78", 90)
    steel = ("lin", "#f4f8ff", "#9aa6b8", 90)
    shadow(a, 16.0, 31.0, 8.6)
    # crew banner
    a.stroke([(10.4, 0.8), (11.4, 15.0)], 0.7, "#3a2a20")
    a.blob([(10.2, 1.0), (6.4, 1.0), (3.2, 1.8), (2.4, 4.4), (3.6, 6.4), (2.2, 9.0), (3.4, 11.6), (10.8, 11.6)],
           ("lin", "#3a5a9a", "#1a2a5a", 90))
    for r, c in ((2.6, "#e8f4ff"), (1.7, "#8ad0ff"), (0.8, "#e8f4ff")):
        for cx in (4.6, 8.4):
            a.stroke([(cx - r * math.cos(t * math.pi / 8), 8.4 - r * math.sin(t * math.pi / 8)) for t in range(9)],
                     0.45, c, smooth=False)
    for r, c in ((2.6, "#e8f4ff"), (1.7, "#8ad0ff"), (0.8, "#e8f4ff")):
        a.stroke([(6.5 - r * math.cos(t * math.pi / 8), 5.6 - r * math.sin(t * math.pi / 8)) for t in range(9)],
                 0.45, c, smooth=False)
    # hakama and sandals
    a.blob([(11.6, 21.6), (21.6, 21.6), (22.8, 29.6), (17.4, 29.6), (16.6, 25.0), (15.8, 29.6), (10.4, 29.6)],
           ("lin", "#4e586c", "#262c3a", 90), smooth=False)
    a.stroke([(16.6, 22.4), (16.6, 25.0)], 0.35, "#1a1e28")
    a.rect(10.6, 29.2, 15.6, 30.8, "#2a1e16", radius=0.5)
    a.rect(17.4, 29.2, 22.6, 30.8, "#2a1e16", radius=0.5)
    # grey kimono, sash, and the captain's jinbaori over it
    a.blob([(11.4, 12.2), (21.4, 12.2), (22.0, 22.4), (11.0, 22.4)], grey)
    a.rect(12.6, 19.2, 20.4, 20.8, ("lin", "#3a4e7a", "#1a2440", 90), radius=0.3)
    a.blob([(9.6, 12.0), (14.6, 12.0), (14.8, 25.2), (8.8, 25.6), (9.2, 18.0)], coat, smooth=False)
    a.blob([(18.4, 12.0), (23.0, 12.0), (24.0, 18.0), (24.8, 25.6), (18.6, 25.2)], coat, smooth=False)
    a.stroke([(14.6, 12.2), (14.8, 25.2), (8.8, 25.6)], 0.5, GOLD, smooth=False)
    a.stroke([(18.4, 12.2), (18.6, 25.2), (24.8, 25.6)], 0.5, GOLD, smooth=False)
    a.ellipse(21.6, 15.4, 1.2, 1.2, GOLD)
    a.stroke([(20.8, 15.6), (21.6, 15.0), (22.4, 15.6)], 0.3, "#142448")
    # back arm in a wide grey sleeve
    a.blob([(10.0, 12.8), (12.4, 13.2), (11.6, 20.0), (8.6, 20.4)], grey)
    a.ellipse(10.0, 21.0, 0.95, 0.95, skin)
    # plain katana held low and forward
    a.poly([(23.2, 19.0), (24.4, 17.8), (28.8, 22.2), (31.6, 28.0), (30.4, 28.8), (27.4, 23.4)], steel)
    a.stroke([(24.0, 19.0), (27.8, 23.0), (30.6, 28.4)], 0.3, "#8ae0ff", alpha=0.9)
    a.stroke([(21.2, 15.4), (23.4, 18.0)], 1.0, "#1a1e28")
    for t in (0.25, 0.55, 0.85):
        a.ellipse(21.2 + 2.2 * t, 15.4 + 2.6 * t, 0.3, 0.3, "#c8d0dc")
    a.ellipse(23.6, 18.3, 1.2, 0.8, GOLD)
    a.blob([(18.8, 12.8), (22.4, 12.6), (23.4, 16.6), (21.6, 17.6)], grey)
    a.ellipse(22.4, 17.2, 1.0, 1.0, skin)
    # collar, head, tied-back hair
    a.blob([(14.6, 11.6), (18.6, 11.6), (16.6, 14.4)], "#f0f4f8", smooth=False)
    a.stroke([(14.6, 8.4), (13.2, 9.8), (12.0, 11.4)], 1.2, ("lin", "#3a3440", "#18141e", 90), taper=True)
    a.ellipse(18.0, 9.8, 3.3, 3.0, skin)
    a.blob([(14.6, 7.8), (15.6, 7.6), (15.4, 10.6), (14.6, 10.2)], "#2a2430")
    a.blob([(15.0, 11.2), (21.2, 10.8), (20.4, 12.8), (16.4, 12.8)], "#8a6a58", clip=True, alpha=0.3)
    # stern eye, cheek scar, mouth
    a.stroke([(18.8, 8.7), (21.0, 8.5)], 0.45, "#1a1a24")
    a.ellipse(19.9, 9.3, 0.75, 0.5, "#fffaf0")
    a.ellipse(20.2, 9.3, 0.35, 0.42, "#1a2a3a")
    a.stroke([(17.2, 9.6), (18.6, 11.8)], 0.3, "#d07a7a")
    a.stroke([(17.4, 10.6), (18.2, 10.3)], 0.2, "#d07a7a")
    a.stroke([(19.6, 11.9), (20.8, 11.7)], 0.3, "#6a3a2a")
    # lacquered jingasa with a gold rim, chin cord and wave mon
    a.stroke([(15.4, 7.6), (16.0, 11.4), (18.0, 12.2)], 0.25, "#e8e0d0")
    a.blob([(10.4, 7.2), (14.0, 4.4), (17.4, 2.6), (21.0, 4.2), (24.8, 7.0), (17.6, 7.8)],
           ("lin", "#34405e", "#10162a", 90))
    a.stroke([(10.6, 7.2), (17.6, 7.9), (24.6, 7.0)], 0.5, GOLD)
    a.ellipse(17.6, 4.9, 1.1, 1.0, GOLD)
    a.stroke([(16.9, 5.1), (17.6, 4.5), (18.3, 5.1)], 0.3, "#10162a")
    a.stroke([(13.4, 5.6), (17.2, 3.0)], 0.35, "#6a7aa0", alpha=0.7)
    a.rim("#e8f4ff", 0.35)
    a.outline()
    halo(a, 20.0, 9.3, 0.9, "#bff0ff", 0.35)
    for x, y, r in ((9.0, 29.8, 3.0), (23.4, 30.2, 3.2), (16.0, 30.8, 2.4), (5.6, 27.0, 2.0)):
        halo(a, x, y, r, "#d4e4f4", 0.35)
    a.save("w2/leader")


# ═════════════════════════ PALE CITADEL ═════════════════════════
def w3_uncommon_a():
    # Masked Retainer: a citadel retainer in pale livery, crouched to lunge, hand on a sheathed blade,
    # face hidden behind a porcelain mask mended with gold and painted with a blue tear.
    a = Art(24, 24)
    navy = ("lin", "#34406a", "#161c34", 90)
    pale = ("lin", "#f4f8ff", "#a8b4c8", 90)
    shadow(a, 11.0, 23.0, 8.0)
    a.blob([(4.4, 15.6), (9.4, 15.0), (9.6, 19.0), (5.4, 21.6), (1.6, 22.4), (1.8, 21.0), (4.2, 19.4)], navy)
    a.rect(0.8, 21.2, 4.0, 22.8, "#10142a", radius=0.6)
    a.blob([(11.0, 15.4), (15.6, 15.2), (16.4, 19.4), (17.8, 21.2), (13.2, 21.8), (12.8, 19.4)], navy)
    a.rect(13.0, 21.0, 18.6, 22.8, "#10142a", radius=0.6)
    a.stroke([(6.4, 15.0), (1.6, 11.6)], 1.0, ("lin", "#2a2232", "#100c16", 90))
    a.stroke([(1.8, 11.8), (1.0, 11.2)], 1.0, GOLD)
    a.blob([(5.8, 10.4), (11.0, 8.2), (15.6, 9.4), (16.4, 13.6), (15.4, 16.6), (7.0, 16.6), (5.4, 13.6)], pale)
    a.blob([(5.0, 14.6), (17.0, 14.6), (17.0, 17.0), (5.0, 17.0)], "#34406a", clip=True)
    a.stroke([(11.6, 8.6), (13.0, 12.6)], 0.6, "#34406a")
    a.ellipse(9.0, 11.6, 1.1, 1.1, "#34406a")
    a.ellipse(9.0, 11.6, 0.55, 0.55, "#f4f8ff")
    a.stroke([(6.6, 15.4), (9.0, 16.4)], 1.0, ("lin", "#5a4a3a", "#2a2018", 0))
    a.ellipse(9.4, 16.6, 0.8, 0.6, GOLD)
    a.stroke([(14.0, 10.4), (11.6, 13.8), (9.6, 15.6)], 1.6, pale)
    a.ellipse(9.4, 15.8, 0.85, 0.8, "#e8d4c4")
    a.ellipse(15.2, 6.8, 3.1, 3.0, ("lin", "#2a3048", "#10142a", 90))
    a.ellipse(13.8, 3.6, 1.2, 1.0, "#1a2038")
    a.stroke([(13.2, 4.6), (12.4, 6.6)], 0.5, "#f4f8ff")
    a.ellipse(17.0, 7.4, 2.5, 2.9, ("lin", "#ffffff", "#c4d0e2", 0))
    for ex in (16.4, 18.6):
        a.stroke([(ex - 0.6, 6.6), (ex + 0.5, 6.4)], 0.45, "#0a0e1e")
    a.stroke([(18.6, 7.0), (18.8, 9.6)], 0.35, "#4aa8ff", taper=True)
    a.stroke([(15.4, 5.2), (16.4, 6.8), (16.0, 8.4), (17.2, 10.0)], 0.22, "#e8b840", smooth=False)
    a.rim("#ffffff", 0.35)
    a.outline()
    halo(a, 18.6, 6.4, 1.0, "#8ae8ff", 0.45)
    spark(a, 21.4, 3.0, 0.8, "#c8f4ff")
    a.save("w3/uncommon_a")


def w3_uncommon_b():
    # Hollow Husk: a towering suit of frosted lamellar armour with nobody inside;
    # spirit fire leaks from the empty helmet and the arms hang limp.
    a = Art(24, 24)
    plate = ("lin", "#d8e4f4", "#6a7c98", 90)
    dark = "#2a3450"
    shadow(a, 12.0, 23.2, 9.6)
    for x0 in (7.2, 13.2):
        a.rect(x0, 18.0, x0 + 3.6, 22.4, plate, radius=0.6)
        a.stroke([(x0 + 0.4, 19.6), (x0 + 3.2, 19.6)], 0.3, dark)
        a.rect(x0 - 0.4, 21.6, x0 + 4.0, 23.0, "#1a2034", radius=0.6)
    a.blob([(5.0, 14.4), (19.0, 14.4), (19.8, 19.6), (4.2, 19.6)], plate, smooth=False)
    a.blob([(4.6, 7.6), (19.4, 7.6), (20.0, 14.8), (4.0, 14.8)], plate)
    for y in (9.6, 11.6, 13.6, 16.2, 18.0):
        a.stroke([(4.2, y), (19.8, y)], 0.35, dark, clip=True)
    for y in (10.6, 12.6, 17.0):
        for x in range(6, 19, 3):
            a.ellipse(x, y, 0.3, 0.3, "#5ac8ff", clip=True)
    a.blob([(4.6, 7.6), (8.0, 7.6), (8.0, 19.6), (4.0, 19.6)], "#3a4660", clip=True, alpha=0.35)
    for x0, sgn in ((1.0, -1), (17.8, 1)):
        a.stroke([(x0 + 2.6, 12.0), (x0 + 2.6 + sgn * 0.6, 16.0), (x0 + 2.6 + sgn * 0.8, 18.6)], 2.0, plate)
        hx = x0 + 2.6 + sgn * 0.8
        for dx in (-0.8, 0.0, 0.8):
            a.stroke([(hx + dx, 19.0), (hx + dx * 1.4, 20.6)], 0.45, "#5a6a84")
        a.rect(x0, 5.4, x0 + 5.2, 11.8, plate, radius=0.6)
        for y in (7.2, 9.0, 10.8):
            a.stroke([(x0 + 0.2, y), (x0 + 5.0, y)], 0.3, dark)
        for x in (x0 + 1.2, x0 + 2.6, x0 + 4.0):
            a.poly([(x - 0.4, 11.6), (x + 0.4, 11.6), (x, 13.2)], "#e8f8ff")
    for hx, sgn in ((9.2, -1), (14.8, 1)):
        a.poly([(hx - 0.6, 4.6), (hx + 0.6, 4.6), (hx + sgn * 1.2, 1.6), (hx + sgn * 0.4, 2.8)],
               ("lin", "#e8d8a0", "#8a7a4a", 90))
    a.blob([(6.8, 7.2), (17.2, 7.2), (18.2, 9.8), (5.8, 9.8)], ("lin", "#b8c8dc", "#5a6a84", 90))
    a.ellipse(12.0, 6.0, 4.0, 3.2, plate)
    a.ellipse(12.4, 7.2, 2.5, 1.9, "#060a14")
    for ex in (11.4, 13.4):
        a.ellipse(ex, 7.2, 0.4, 0.3, "#8af0ff")
    a.stroke([(8.8, 4.8), (11.2, 3.4)], 0.4, "#ffffff", alpha=0.7)
    for p in ([(9.0, 9.0), (10.0, 10.6), (9.4, 12.0)], [(15.4, 15.0), (14.6, 16.4)]):
        a.stroke(p, 0.3, "#1a2234", smooth=False)
    for x, y in ((6.0, 9.0), (15.0, 13.0), (9.0, 16.0)):
        a.stroke([(x, y), (x + 1.6, y - 0.4)], 0.4, "#ffffff", alpha=0.6)
    a.rim("#ffffff", 0.3)
    a.outline()
    flame(a, 12.0, 3.4, 3.0, 1.3, "#4aa8ff", "#d8f8ff")
    halo(a, 12.4, 7.2, 2.0, "#5ad8ff", 0.4)
    halo(a, 12.0, 2.0, 1.8, "#8ae0ff", 0.35)
    a.save("w3/uncommon_b")


def w3_leader():
    # Numbered Quartermaster: a citadel quartermaster in a fur-trimmed white greatcoat and tall fur hat,
    # monocle on, ledger under his arm, gate keys and numbered tags at his belt, a tally banner on a pole.
    a = Art(32, 32)
    coat = ("lin", "#f4f8ff", "#9aaac4", 90)
    navy = ("lin", "#34406a", "#141a32", 90)
    fur = ("lin", "#ffffff", "#c4ccd8", 90)
    silver = ("lin", "#f4f8ff", "#8a96ac", 90)
    shadow(a, 17.0, 31.0, 9.0)
    a.stroke([(5.0, 1.0), (5.0, 30.2)], 0.9, ("lin", "#6a5a4a", "#2a2018", 0))
    a.stroke([(4.4, 1.6), (11.6, 1.6)], 0.7, silver)
    a.blob([(5.4, 1.8), (11.2, 1.8), (11.2, 12.6), (8.4, 11.2), (5.4, 12.6)], navy, smooth=False)
    for k in range(4):
        a.stroke([(6.6 + k * 1.1, 4.0), (6.6 + k * 1.1, 9.2)], 0.45, "#e8f4ff")
    a.stroke([(5.8, 8.6), (10.8, 4.6)], 0.5, "#8ad0ff")
    a.rect(13.0, 26.4, 16.2, 30.6, navy, radius=0.7)
    a.rect(18.6, 26.4, 21.8, 30.6, navy, radius=0.7)
    a.blob([(11.0, 13.0), (23.4, 13.0), (25.0, 21.0), (26.2, 28.4), (9.6, 28.4), (10.2, 21.0)], coat)
    a.blob([(8.6, 13.0), (13.0, 13.0), (13.0, 29.0), (8.6, 29.0)], "#5a6a8a", clip=True, alpha=0.3)
    a.blob([(15.8, 13.2), (19.8, 13.2), (20.2, 27.8), (15.6, 27.8)], navy)
    for y in (15.4, 17.6, 19.8, 22.0, 24.2):
        a.ellipse(15.0, y, 0.45, 0.45, silver)
        a.ellipse(20.6, y, 0.45, 0.45, silver)
    for x in range(10, 27, 2):
        a.ellipse(x + 0.2, 28.4, 1.2, 0.9, fur)
    a.rect(10.6, 21.2, 24.6, 22.4, ("lin", "#3a2a24", "#1a1210", 90), radius=0.3)
    a.rect(16.8, 20.9, 18.8, 22.7, silver, radius=0.3)
    a.stroke([(12.4, 22.4), (12.2, 23.4)], 0.3, "#3a2a24")
    a.ellipse(12.2, 24.0, 1.0, 1.0, silver)
    a.ellipse(12.2, 24.0, 0.5, 0.5, "#9aaac4")
    for x in (11.0, 13.2):
        a.stroke([(x + 0.4, 24.6), (x + 0.2, 26.4)], 0.3, silver)
    for k, x in enumerate((21.6, 23.4)):
        a.stroke([(x, 22.4), (x, 23.4)], 0.25, "#3a2a24")
        a.rect(x - 0.8, 23.4, x + 0.8, 25.4, ("lin", "#d8b47a", "#9a7444", 90), radius=0.2)
        for d in range(k + 1):
            a.ellipse(x, 23.9 + d * 0.7, 0.22, 0.22, "#3a2010")
    a.stroke([(12.6, 14.4), (9.4, 16.6), (6.0, 17.4)], 1.9, coat)
    a.ellipse(5.2, 17.4, 1.0, 1.0, "#e8d4c4")
    a.stroke([(22.4, 14.4), (25.0, 17.4), (25.4, 19.0)], 1.9, coat)
    a.rect(22.4, 16.6, 28.2, 21.6, navy, radius=0.4)
    a.rect(22.8, 16.9, 27.8, 17.6, "#f4f0e4", radius=0.2)
    for x, y in ((22.6, 16.8), (27.8, 16.8), (22.6, 21.2), (27.8, 21.2)):
        a.ellipse(x, y, 0.45, 0.45, silver)
    a.ellipse(25.2, 21.6, 1.0, 0.9, "#e8d4c4")
    for x in (12.4, 22.0):
        a.ellipse(x, 13.6, 2.0, 1.1, silver)
        for k in range(4):
            a.stroke([(x - 1.4 + k * 0.95, 14.2), (x - 1.5 + k * 0.95, 15.4)], 0.35, "#c8d4e4")
    for x in range(12, 24, 2):
        a.ellipse(x + 0.6, 12.8, 1.2, 1.0, fur)
    a.ellipse(18.4, 9.6, 3.3, 3.2, ("lin", "#f4e4dc", "#c8a898", 90))
    a.stroke([(19.4, 8.8), (21.4, 8.6)], 0.45, "#1a1a24")
    a.ellipse(20.4, 9.4, 1.15, 1.15, silver)
    a.ellipse(20.4, 9.4, 0.85, 0.85, "#e4f4ff")
    a.ellipse(20.4, 9.4, 0.4, 0.4, "#3a8ac8")
    a.stroke([(21.4, 10.0), (21.8, 12.4)], 0.2, silver)
    a.stroke([(19.2, 11.8), (21.0, 11.6)], 0.35, "#8a6a64")
    a.blob([(14.6, 7.6), (14.8, 3.8), (16.6, 3.0), (18.6, 3.4), (20.6, 2.8), (22.4, 3.6), (22.6, 7.6)],
           ("lin", "#5a6480", "#1e2440", 90))
    for x in (15.8, 17.6, 19.6, 21.4):
        a.stroke([(x, 3.8), (x - 0.2, 6.6)], 0.3, "#8a94b0", alpha=0.6)
    a.rect(14.4, 6.4, 22.8, 7.8, fur, radius=0.6)
    a.rect(17.6, 3.8, 20.0, 6.0, silver, radius=0.3)
    for k in range(3):
        a.ellipse(18.1 + k * 0.7, 4.9, 0.22, 0.22, "#34406a")
    a.rim("#ffffff", 0.35)
    a.outline()
    halo(a, 20.4, 9.4, 1.1, "#8ae8ff", 0.45)
    for x, y in ((29.4, 4.0), (2.0, 27.0)):
        spark(a, x, y, 0.8, "#c8f4ff")
    a.save("w3/leader")


# ═════════════════════════ RAINWARD ═════════════════════════
def w4_uncommon_a():
    # Bladetail Runner: a hooded runner in a soaked raincoat, sprinting low, a chained steel blade-tail
    # whipping behind from a belt mount.
    a = Art(24, 24)
    coat = ("lin", "#6a8296", "#2a3a46", 90)
    steel = ("lin", "#f0f6ff", "#8a98aa", 90)
    shadow(a, 12.0, 23.0, 8.0)
    tail = [(8.6, 13.4), (5.2, 15.0), (2.4, 13.4), (1.4, 9.8), (2.8, 6.4), (5.6, 4.6)]
    a.stroke(tail, 0.5, "#2a3038")
    path = _catmull(tail, False, 8)
    for i in range(4, len(path) - 1, 5):
        (x0, y0), (x1, y1) = path[i], path[min(i + 3, len(path) - 1)]
        dx, dy = x1 - x0, y1 - y0
        L = math.hypot(dx, dy) or 1.0
        nx, ny = -dy / L, dx / L
        a.poly([(x0 + nx * 0.9, y0 + ny * 0.9), (x1 + dx / L * 0.8, y1 + dy / L * 0.8), (x0 - nx * 0.9, y0 - ny * 0.9)],
               steel)
    ex, ey = path[-1]
    a.poly([(ex - 0.8, ey + 0.8), (ex + 2.6, ey - 2.2), (ex + 0.6, ey + 1.2)], steel)
    a.ellipse(8.6, 13.4, 1.1, 1.1, ("lin", "#8a98aa", "#2a3038", 90))
    a.stroke([(10.4, 14.0), (7.6, 17.6), (4.4, 19.6)], 1.6, coat)
    a.rect(2.6, 19.0, 5.4, 20.6, "#14181e", radius=0.5)
    a.stroke([(13.6, 14.0), (17.6, 16.6), (19.4, 21.0)], 1.6, coat)
    a.rect(18.4, 20.6, 21.8, 22.2, "#14181e", radius=0.5)
    a.blob([(7.2, 9.6), (13.6, 7.4), (17.6, 9.4), (16.2, 13.8), (12.6, 15.0), (8.4, 14.8)], coat)
    a.blob([(7.0, 12.6), (16.6, 12.0), (16.0, 14.2), (8.0, 15.2)], "#141a20", clip=True, alpha=0.5)
    a.stroke([(8.2, 10.0), (15.6, 8.4)], 0.35, "#8aa0b4", alpha=0.7)
    a.stroke([(15.0, 10.4), (18.4, 12.0), (20.0, 12.6)], 1.3, coat)
    a.ellipse(20.4, 12.8, 0.8, 0.7, "#c8b8a8")
    a.stroke([(9.0, 10.6), (6.4, 12.4), (5.6, 13.8)], 1.2, coat)
    a.ellipse(17.6, 7.6, 3.1, 2.9, coat)
    a.blob([(15.0, 6.8), (17.4, 4.2), (20.4, 5.0), (19.0, 6.4)], "#5a6a7a")
    a.ellipse(19.2, 8.2, 1.7, 1.6, "#0a0e12")
    for ex2 in (18.8, 20.0):
        a.ellipse(ex2, 8.0, 0.32, 0.26, "#ff4a4a")
    a.rim("#a8d0f0", 0.4)
    a.outline()
    halo(a, 19.4, 8.0, 1.2, "#ff4a4a", 0.35)
    for x, y in ((20.6, 22.8), (22.4, 22.0), (17.4, 22.6)):
        a.ellipse(x, y, 0.5, 0.3, "#8ad0ff", alpha=0.8)
    for x, y in ((22.0, 3.0), (11.0, 2.0), (2.0, 20.0), (14.0, 4.6)):
        a.stroke([(x, y), (x - 0.4, y + 1.6)], 0.2, "#a8d0f0", alpha=0.6)
    a.save("w4/uncommon_a")


def w4_uncommon_b():
    # Mask Collector: a gaunt figure in a long rain-dark coat hung with other people's masks,
    # holding out one more; static crackles between them.
    a = Art(24, 24)
    coat = ("lin", "#4a4660", "#1c1a28", 90)
    shadow(a, 11.6, 23.0, 7.0)
    a.blob([(7.0, 8.0), (16.0, 8.0), (17.6, 15.6), (18.6, 22.6), (14.4, 22.0), (12.0, 22.8), (9.2, 22.0),
            (5.0, 22.6), (6.0, 15.6)], coat)
    a.blob([(5.0, 8.0), (9.0, 8.0), (9.0, 23.0), (5.0, 23.0)], "#100e18", clip=True, alpha=0.4)
    a.stroke([(11.6, 9.0), (11.8, 22.0)], 0.35, "#14121c")
    a.stroke([(6.6, 10.2), (11.8, 12.6), (17.0, 10.2)], 0.25, "#c8c0a8", smooth=True)

    def mask(x, y, base, mark):
        a.ellipse(x, y, 1.35, 1.6, base)
        if mark == "smile":
            a.ellipse(x - 0.5, y - 0.4, 0.25, 0.25, "#1a1418")
            a.ellipse(x + 0.5, y - 0.4, 0.25, 0.25, "#1a1418")
            a.stroke([(x - 0.7, y + 0.4), (x, y + 0.8), (x + 0.7, y + 0.4)], 0.25, "#1a1418")
        elif mark == "cross":
            for sx in (-0.5, 0.5):
                a.stroke([(x + sx - 0.3, y - 0.7), (x + sx + 0.3, y - 0.1)], 0.2, "#1a1418")
                a.stroke([(x + sx + 0.3, y - 0.7), (x + sx - 0.3, y - 0.1)], 0.2, "#1a1418")
            a.stroke([(x - 0.4, y + 0.7), (x + 0.4, y + 0.7)], 0.2, "#1a1418")
        elif mark == "dots":
            for dx, dy in ((-0.5, -0.4), (0.5, -0.4), (0.0, 0.5)):
                a.ellipse(x + dx, y + dy, 0.28, 0.28, "#1a1418")
        else:
            for dy in (-0.7, 0.0, 0.7):
                a.stroke([(x - 1.1, y + dy), (x + 1.1, y + dy)], 0.3, "#2a6a3a", clip=True)
            a.ellipse(x, y - 0.3, 0.3, 0.3, "#1a1418")

    mask(8.4, 13.0, ("lin", "#ff8a7a", "#c03a3a", 90), "smile")
    mask(11.8, 14.4, ("lin", "#a8d0ff", "#4a7ac8", 90), "cross")
    mask(15.2, 13.0, ("lin", "#fff0a0", "#d0a030", 90), "dots")
    mask(9.8, 18.4, ("lin", "#d8f0d8", "#8ac08a", 90), "stripe")
    mask(13.8, 18.6, ("lin", "#f0d8ff", "#a070c8", 90), "dots")
    a.stroke([(15.8, 9.4), (19.2, 11.2), (20.6, 11.6)], 1.2, coat)
    for k in range(3):
        a.stroke([(20.6, 11.6), (21.6 + k * 0.2, 10.8 + k * 0.6)], 0.25, "#d8d0e0")
    mask(21.8, 13.6, ("lin", "#ffffff", "#b8b4c8", 90), "smile")
    a.ellipse(11.6, 4.8, 3.6, 3.6, ("lin", "#3a3650", "#16141e", 90))
    a.blob([(9.2, 4.2), (11.6, 1.4), (14.4, 3.0), (14.8, 6.2), (12.4, 7.4), (9.6, 6.6)], "#24202e")
    a.ellipse(12.6, 5.4, 2.1, 2.5, ("lin", "#ffffff", "#c8c4d4", 0))
    a.blob([(11.4, 3.6), (12.6, 3.0), (13.0, 7.8), (12.0, 7.6)], "#f8d84a", alpha=0.85)
    a.ellipse(11.8, 5.0, 0.35, 0.5, "#14121c")
    a.ellipse(13.6, 5.0, 0.35, 0.5, "#14121c")
    a.rim("#c8c0ff", 0.3)
    a.outline()
    for x, y in ((11.8, 5.0), (13.6, 5.0)):
        halo(a, x, y, 0.8, "#ffe84a", 0.5)
    for p in ([(9.6, 13.0), (10.4, 13.6), (10.6, 14.4)], [(13.2, 14.4), (13.6, 13.6), (14.0, 13.2)],
              [(19.6, 13.0), (20.0, 12.2), (20.6, 12.6)]):
        a.stroke(p, 0.3, "#ffe84a", smooth=False)
    spark(a, 4.0, 6.0, 0.8, "#ffe84a")
    a.save("w4/uncommon_b")


def w4_leader():
    # Drowned Pirate Captain: a flood-drowned captain in a sodden plumed hat and greatcoat, barnacled and weeded,
    # cutlass raised and a ship's lantern burning drowned-blue.
    a = Art(32, 32)
    skin = ("lin", "#a8c8c0", "#5a7a78", 90)
    coat = ("lin", "#2e5060", "#122028", 90)
    tarn = ("lin", "#d8c070", "#7a6430", 90)
    steel = ("lin", "#f0f6ff", "#8a98aa", 90)
    shadow(a, 17.0, 31.0, 9.4)
    a.ellipse(17.0, 30.6, 11.0, 1.4, "#3a6a8a", alpha=0.5)
    a.rect(12.8, 25.0, 16.2, 30.6, ("lin", "#4a3428", "#1e140e", 90), radius=0.7)
    a.rect(18.0, 25.0, 21.4, 30.6, ("lin", "#4a3428", "#1e140e", 90), radius=0.7)
    a.rect(12.4, 25.0, 16.6, 26.4, "#5a4232", radius=0.5)
    a.rect(17.6, 25.0, 21.8, 26.4, "#5a4232", radius=0.5)
    a.blob([(10.6, 12.6), (23.0, 12.6), (25.0, 21.0), (26.2, 28.8), (23.4, 27.6), (21.4, 29.2), (18.6, 27.4),
            (16.4, 29.2), (13.8, 27.4), (11.6, 29.0), (9.6, 28.0), (9.8, 20.0)], coat, smooth=False)
    a.blob([(8.6, 12.6), (13.0, 12.6), (13.0, 29.4), (8.6, 29.4)], "#0a141a", clip=True, alpha=0.35)
    a.blob([(15.6, 13.0), (19.2, 13.0), (19.0, 22.0), (15.8, 22.0)], ("lin", "#c8d4c8", "#7a8a7a", 90))
    a.stroke([(16.4, 15.0), (17.0, 17.4), (16.2, 19.6)], 0.3, "#5a6a5a")
    a.stroke([(15.4, 13.2), (15.0, 27.0)], 0.5, tarn)
    a.stroke([(19.4, 13.2), (20.4, 26.8)], 0.5, tarn)
    a.rect(10.6, 21.0, 24.6, 22.2, ("lin", "#3a2818", "#1a1008", 90), radius=0.3)
    a.rect(16.4, 20.7, 18.6, 22.5, tarn, radius=0.3)
    for p in ([(11.6, 13.4), (11.0, 17.0), (12.0, 20.0)], [(23.2, 13.6), (24.0, 18.0)], [(14.0, 27.6), (13.4, 29.6)],
              [(22.0, 27.6), (22.6, 29.6)]):
        a.stroke(p, 0.6, ("lin", "#5aa060", "#2a5a34", 90), taper=True)
    for x, y in ((21.6, 16.4), (22.4, 17.2), (12.6, 24.0), (13.4, 24.8), (24.0, 25.0)):
        a.ellipse(x, y, 0.55, 0.5, ("lin", "#f0e4c8", "#a89878", 90))
        a.ellipse(x, y, 0.2, 0.2, "#4a4030")
    a.stroke([(11.4, 14.2), (9.8, 17.6), (9.6, 19.8)], 1.9, coat)
    a.ellipse(9.6, 20.2, 0.9, 0.9, skin)
    a.stroke([(9.6, 20.6), (9.6, 21.4)], 0.3, "#3a2a1a")
    a.rect(7.8, 21.2, 11.4, 21.8, tarn, radius=0.2)
    a.rect(8.0, 21.8, 11.2, 25.4, ("rad", "#e0fff8", "#2a9a9a"), radius=0.5)
    for x in (8.9, 10.3):
        a.stroke([(x, 21.9), (x, 25.3)], 0.3, "#2a2018")
    a.rect(7.8, 25.2, 11.4, 25.8, tarn, radius=0.2)
    a.poly([(25.0, 17.6), (26.0, 17.0), (29.8, 11.8), (31.2, 7.6), (30.6, 7.4), (28.6, 11.2)], steel)
    a.stroke([(25.6, 17.4), (29.4, 12.0), (30.8, 8.0)], 0.25, "#8af0ff", alpha=0.8)
    a.stroke([(23.6, 18.2), (25.0, 19.0), (26.2, 17.8)], 0.6, tarn)
    a.stroke([(22.6, 14.0), (24.4, 16.6), (25.0, 18.0)], 1.9, coat)
    a.ellipse(25.2, 18.0, 0.95, 0.95, skin)
    for x in (12.0, 22.4):
        a.ellipse(x, 13.4, 2.0, 1.1, tarn)
        for k in range(4):
            a.stroke([(x - 1.4 + k * 0.95, 14.0), (x - 1.5 + k * 0.95, 15.4)], 0.35, "#a8904a")
    a.ellipse(18.4, 9.8, 3.2, 3.1, skin)
    a.blob([(15.4, 10.6), (21.6, 10.4), (21.0, 13.4), (19.4, 14.4), (17.6, 13.0), (16.0, 13.6)],
           ("lin", "#4a6a5a", "#22362c", 90))
    a.stroke([(19.6, 11.6), (21.0, 11.4)], 0.3, "#14201a")
    a.stroke([(19.2, 8.8), (21.4, 8.6)], 0.45, "#14201a")
    a.ellipse(20.4, 9.4, 0.5, 0.4, "#7affe8")
    a.blob([(10.6, 7.6), (13.6, 5.4), (19.0, 4.8), (24.8, 5.4), (27.0, 7.6), (23.4, 8.0), (18.6, 7.2), (13.6, 8.4)],
           ("lin", "#2a3038", "#0e1216", 90))
    a.blob([(14.6, 5.8), (15.2, 2.4), (18.6, 1.4), (22.2, 2.4), (22.6, 5.8)], ("lin", "#2a3038", "#0e1216", 90))
    a.stroke([(14.8, 5.2), (22.4, 5.2)], 0.6, tarn)
    a.stroke([(15.4, 3.0), (12.0, 2.6), (9.4, 4.4), (8.6, 7.6), (9.4, 10.0)], 1.5, ("lin", "#e8ecf0", "#8a96a0", 90),
             taper=True)
    a.stroke([(12.4, 3.0), (10.4, 4.8), (10.0, 7.6)], 0.25, "#6a7680", alpha=0.8)
    a.rim("#a8e8e0", 0.3)
    a.outline()
    halo(a, 9.6, 23.6, 2.8, "#5affe0", 0.5)
    halo(a, 20.4, 9.4, 1.0, "#7affe8", 0.45)
    for x, y in ((11.0, 9.4), (26.6, 9.0), (13.0, 30.2), (24.8, 29.8)):
        a.ellipse(x, y, 0.35, 0.55, "#8ad8ff", alpha=0.85)
    a.save("w4/leader")


# ═════════════════════════ SHATTERLINE ═════════════════════════
def w5_uncommon_a():
    # Grade-Three Swarm Mother: a squat curse-toad of road-brown flesh with a slab of asphalt on her back
    # and a glowing egg-sac of spawn; one has already dropped off.
    a = Art(24, 24)
    flesh = ("lin", "#b08850", "#5a3a20", 90)
    shadow(a, 12.0, 23.0, 9.6)
    a.ellipse(6.4, 18.8, 3.8, 3.0, flesh)
    a.blob([(2.4, 21.0), (7.0, 20.6), (8.0, 22.4), (2.0, 22.6)], "#5a3a20")
    a.blob([(3.0, 15.0), (5.0, 10.0), (10.0, 7.6), (16.0, 8.0), (20.4, 11.0), (21.6, 15.6), (18.8, 19.8),
            (10.0, 20.8), (4.0, 19.0)], flesh)
    a.blob([(15.6, 17.0), (18.4, 17.0), (18.8, 22.0), (15.2, 22.0)], flesh)
    for dx in (-1.0, 0.0, 1.0):
        a.stroke([(17.0 + dx * 0.6, 21.6), (17.0 + dx * 1.4, 22.6)], 0.5, "#4a2e18")
    a.blob([(3.0, 15.6), (21.6, 15.6), (21.6, 21.0), (3.0, 21.0)], "#3a2210", clip=True, alpha=0.3)
    for p in ([(6.0, 12.6), (8.6, 14.6), (8.0, 17.4)], [(12.0, 9.4), (13.0, 12.6), (11.6, 15.4)],
              [(17.6, 10.6), (16.6, 13.6)]):
        a.stroke(p, 0.45, "#b050f0", clip=True, alpha=0.9)
    a.poly([(5.4, 10.8), (8.0, 8.4), (12.4, 7.6), (15.2, 9.4), (13.6, 11.2), (10.8, 10.8), (8.8, 12.6)],
           ("lin", "#74747e", "#3a3a44", 90))
    for x0 in (7.4, 10.4, 13.0):
        a.stroke([(x0, 10.6 - (x0 - 7.4) * 0.22), (x0 + 1.4, 10.3 - (x0 - 7.4) * 0.22)], 0.45, "#f0c030")
    a.stroke([(9.6, 8.2), (10.4, 9.6), (9.6, 11.4)], 0.25, "#24242c", smooth=False)
    a.blob([(4.2, 10.4), (5.2, 5.4), (9.6, 2.4), (14.4, 3.8), (15.2, 7.8), (11.8, 8.4), (8.0, 9.6)],
           ("lin", "#e0b0ff", "#8040c8", 90), alpha=0.85)
    for x, y in ((7.0, 7.2), (9.8, 5.0), (12.6, 5.6), (9.4, 8.0), (6.4, 9.4)):
        a.ellipse(x, y, 1.0, 0.9, "#6a2aa0")
        a.ellipse(x + 0.3, y - 0.1, 0.25, 0.25, "#ffe04a")
    a.blob([(15.4, 15.2), (21.4, 14.0), (20.6, 15.6), (16.4, 16.4)], "#2a1008")
    for x in (17.0, 18.4, 19.8):
        a.poly([(x - 0.3, 15.0), (x + 0.3, 14.9), (x, 15.7)], "#f0e8d0")
    for x, y, r in ((18.4, 11.0, 0.8), (20.2, 11.6, 0.7), (19.0, 12.8, 0.55)):
        a.ellipse(x, y, r, r, "#ffe04a")
        a.ellipse(x + 0.15, y, r * 0.4, r * 0.5, "#2a1008")
    a.rim("#e0c8a0", 0.3)
    a.outline()
    a.ellipse(1.6, 20.4, 1.1, 1.0, "#6a2aa0")
    a.ellipse(1.8, 20.2, 0.3, 0.3, "#ffe04a")
    halo(a, 9.6, 6.2, 3.6, "#c060ff", 0.3)
    halo(a, 19.0, 11.6, 1.6, "#ffe04a", 0.3)
    a.save("w5/uncommon_a")


def w5_uncommon_b():
    # Cursed Tool Thief: a hunched curse in a grey hoodie, a sack of stolen loot on its back and stolen coins
    # round its neck, reaching for one more prize; glitch-cyan and magenta eyes.
    a = Art(24, 24)
    hood = ("lin", "#6a6484", "#2e2a3e", 90)
    skin = ("lin", "#a070d0", "#5a3a8a", 90)
    shadow(a, 12.0, 23.0, 8.0)
    a.stroke([(9.4, 16.6), (8.4, 19.6), (9.0, 22.0)], 1.2, hood)
    a.stroke([(13.6, 16.6), (15.2, 19.4), (15.6, 22.0)], 1.2, hood)
    a.rect(7.6, 21.4, 10.4, 22.8, "#1a1622", radius=0.5)
    a.rect(14.6, 21.4, 17.6, 22.8, "#1a1622", radius=0.5)
    a.blob([(1.2, 9.0), (3.0, 4.6), (7.6, 3.4), (10.6, 6.0), (10.6, 12.0), (7.6, 15.0), (2.6, 14.0)],
           ("lin", "#a8845a", "#5a4228", 90))
    a.stroke([(3.6, 5.6), (5.6, 9.4), (4.6, 12.6)], 0.35, "#4a3420", alpha=0.8)
    a.stroke([(4.6, 3.6), (3.4, 1.2)], 0.6, ("lin", "#d8dce4", "#7a8090", 0))
    a.rect(2.6, 0.6, 4.4, 1.6, GOLD, radius=0.3)
    a.ellipse(7.6, 3.4, 1.2, 1.2, GOLD)
    a.ellipse(7.6, 3.6, 0.4, 0.4, "#5a4010")
    a.ellipse(9.6, 5.0, 1.0, 1.0, ("rad", "#e0fff8", "#2ad0c0"))
    a.blob([(7.0, 9.0), (14.0, 8.0), (16.6, 12.0), (15.6, 17.2), (8.0, 17.4), (6.6, 13.0)], hood)
    a.stroke([(7.6, 8.8), (4.4, 12.0), (3.6, 10.2)], 1.1, hood)
    a.ellipse(15.0, 7.2, 3.4, 3.2, hood)
    a.ellipse(16.4, 7.8, 2.2, 2.1, skin)
    a.rect(15.4, 6.6, 16.6, 7.4, "#5afff0")
    a.rect(17.4, 6.8, 18.4, 7.6, "#ff4ad8")
    a.blob([(15.6, 8.8), (18.4, 8.6), (18.0, 9.6), (16.0, 9.6)], "#1a0e24")
    for x in (16.2, 17.0, 17.8):
        a.poly([(x - 0.3, 8.7), (x + 0.3, 8.7), (x, 9.4)], "#f0e8f8")
    for k in range(5):
        a.ellipse(12.4 + k * 1.0, 10.8 + abs(k - 2) * -0.3 + 0.6, 0.45, 0.45, GOLD)
    a.stroke([(14.6, 11.2), (18.0, 12.6), (19.8, 12.0)], 1.1, hood)
    for k in range(3):
        a.stroke([(19.8, 12.0), (21.0, 11.0 + k * 0.8)], 0.3, skin)
    a.ellipse(21.8, 11.8, 1.3, 1.3, ("rad", "#fff8c0", "#ffb020"))
    a.rim("#c8c0ff", 0.3)
    a.outline()
    halo(a, 21.8, 11.8, 2.0, "#ffe04a", 0.45)
    for p in ([(20.0, 9.0), (20.6, 9.8), (20.2, 10.4)], [(23.2, 10.6), (22.8, 11.2), (23.4, 11.8)]):
        a.stroke(p, 0.3, "#ffe84a", smooth=False)
    for x, y, w, c in ((13.0, 6.4, 2.0, "#5afff0"), (18.6, 14.0, 1.6, "#ff4ad8"), (6.0, 18.0, 1.8, "#5afff0")):
        a.stroke([(x, y), (x + w, y)], 0.3, c, alpha=0.8)
    a.save("w5/uncommon_b")


def w5_leader():
    # Curse User Captain: a crew boss in a naval peaked cap with a talisman in its band and a black greatcoat
    # worn as a cape, gold chains across his chest, a cane topped with a caged, crackling curse-eye.
    a = Art(32, 32)
    black = ("lin", "#3a3444", "#100e16", 90)
    uni = ("lin", "#3a3a4a", "#1a1a24", 90)
    skin = ("lin", "#f0e0d8", "#c0a098", 90)
    shadow(a, 16.0, 31.0, 10.0)
    a.rect(12.6, 24.0, 15.8, 30.6, uni, radius=0.7)
    a.rect(17.6, 24.0, 20.8, 30.6, uni, radius=0.7)
    a.rect(12.2, 29.0, 16.2, 30.8, "#0a080e", radius=0.5)
    a.rect(17.2, 29.0, 21.6, 30.8, "#0a080e", radius=0.5)
    a.blob([(11.8, 12.4), (21.0, 12.4), (21.6, 24.6), (11.4, 24.6)], uni)
    a.blob([(8.4, 12.0), (13.4, 12.0), (12.8, 26.0), (11.6, 29.4), (8.8, 27.8), (5.0, 29.6), (6.8, 20.0)], black,
           smooth=False)
    a.blob([(19.4, 12.0), (24.4, 12.0), (26.4, 20.0), (28.4, 29.6), (24.6, 27.8), (21.6, 29.4), (20.2, 26.0)], black,
           smooth=False)
    a.stroke([(13.2, 12.4), (12.6, 26.0), (11.6, 29.2)], 0.5, "#8a3ad0", smooth=False)
    a.stroke([(19.6, 12.4), (20.4, 26.0), (21.6, 29.2)], 0.5, "#8a3ad0", smooth=False)
    a.rect(11.6, 21.0, 21.4, 22.6, ("lin", "#8a3ad0", "#4a1a80", 90), radius=0.3)
    for y0, y1 in ((14.0, 17.2), (15.6, 19.4)):
        pts = [(12.6, y0), (16.4, y1), (20.4, y0)]
        a.stroke(pts, 0.35, "#a08030")
        for t in range(7):
            x = 12.6 + t * 1.3
            y = y0 + (y1 - y0) * math.sin(math.pi * t / 6)
            a.ellipse(x, y, 0.4, 0.3, GOLD)
    a.blob([(14.4, 12.2), (18.4, 12.2), (16.4, 14.8)], "#d8d0e0", smooth=False)
    a.stroke([(21.4, 14.0), (24.0, 17.0), (24.6, 18.6)], 1.8, uni)
    a.ellipse(25.0, 18.8, 0.95, 0.95, skin)
    a.stroke([(25.2, 30.0), (25.6, 9.4)], 0.8, ("lin", "#3a3444", "#100e16", 0))
    a.rect(24.6, 9.2, 26.6, 10.0, GOLD, radius=0.2)
    a.ellipse(25.6, 7.0, 2.2, 2.2, ("rad", "#fffce0", "#e0a020"))
    a.ellipse(25.6, 7.0, 0.9, 1.1, "#3a0e40")
    a.ellipse(25.8, 7.0, 0.35, 0.6, "#ffe84a")
    for x in (24.2, 25.6, 27.0):
        a.stroke([(x, 4.8), (x + (25.6 - x) * 0.2, 9.2)], 0.3, "#2a2030")
    a.stroke([(23.6, 6.0), (27.6, 6.0)], 0.3, "#2a2030")
    for x in (9.6, 23.0):
        a.ellipse(x, 12.6, 2.2, 1.1, GOLD)
        for k in range(5):
            a.stroke([(x - 1.8 + k * 0.9, 13.2), (x - 1.9 + k * 0.9, 14.8)], 0.35, "#e0b040")
    a.ellipse(16.8, 9.4, 3.1, 3.0, skin)
    a.stroke([(13.8, 7.8), (13.6, 10.6)], 1.0, "#1a1420")
    a.stroke([(18.0, 8.4), (20.0, 8.2)], 0.45, "#1a1420")
    a.ellipse(19.0, 9.0, 0.45, 0.35, "#ffe84a")
    a.stroke([(16.8, 7.4), (17.8, 10.4)], 0.3, "#c06a6a")
    a.stroke([(17.6, 11.6), (19.0, 11.8), (19.8, 11.2)], 0.35, "#5a2a2a")
    a.blob([(12.6, 6.6), (12.8, 3.6), (15.6, 2.0), (20.0, 2.0), (22.2, 3.6), (21.4, 6.6)], black)
    a.rect(12.6, 5.4, 21.6, 6.8, ("lin", "#2a2430", "#0a080e", 90), radius=0.3)
    a.stroke([(12.8, 5.6), (21.4, 5.6)], 0.4, GOLD)
    a.blob([(16.4, 6.4), (21.0, 6.2), (23.2, 7.4), (20.4, 7.8), (16.4, 7.4)], ("lin", "#2a2430", "#000000", 90))
    a.ellipse(17.4, 3.8, 1.2, 1.1, GOLD)
    a.stroke([(16.8, 3.2), (17.4, 3.9), (17.0, 4.6)], 0.2, "#5a4010")
    a.poly([(19.6, 3.2), (20.8, 2.8), (21.6, 6.0), (20.4, 6.4)], ("lin", "#fff4d0", "#e0d0a0", 90))
    for t in (0.3, 0.6):
        a.stroke([(19.9 + t * 1.0, 3.4 + t * 3.0), (20.6 + t * 1.0, 3.2 + t * 3.0)], 0.22, "#c0302a")
    a.rim("#d0c0ff", 0.3)
    a.outline()
    halo(a, 25.6, 7.0, 3.2, "#ffe04a", 0.45)
    halo(a, 19.0, 9.0, 1.0, "#ffe84a", 0.45)
    for p in ([(23.0, 4.4), (22.0, 3.4), (22.6, 2.6), (21.6, 1.6)], [(28.0, 5.0), (29.4, 4.2), (29.0, 3.0), (30.4, 2.0)],
              [(27.8, 9.0), (29.4, 9.6), (29.0, 10.8)]):
        a.stroke(p, 0.35, "#fff080", smooth=False)
    a.save("w5/leader")


def main():
    fns = [rokurokubi_clerk, tsurube_drop, captain_kitsunebi,
           w1_uncommon_a, w1_uncommon_b, w1_leader,
           w2_uncommon_a, w2_uncommon_b, w2_leader,
           w3_uncommon_a, w3_uncommon_b, w3_leader,
           w4_uncommon_a, w4_uncommon_b, w4_leader,
           w5_uncommon_a, w5_uncommon_b, w5_leader]
    only = sys.argv[1:]
    for f in fns:
        if not only or any(o in f.__name__ for o in only):
            f()
            print("wrote", f.__name__)


if __name__ == "__main__":
    main()
