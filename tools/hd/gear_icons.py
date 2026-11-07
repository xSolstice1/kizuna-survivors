"""HD redraw of the 16x16 equipment slot icons (art/src/gear) and Exotic / Arsenal icons (art/src/exotic).

    python3 tools/hd/gear_icons.py

Slot icons are clean and neutral; Exotics and Arsenal attacks get gold trim, glow and sparkles.
"""
import math
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from hdkit import Art  # noqa: E402

OL = 0.8  # outline width for 16px icons

# shared palette
K = "#1a1423"
RED, RED_D = "#c23b3b", "#7c1f2a"
G, g, H = "#f2c94c", "#a8761e", "#fff4b8"
J, j, Q = "#6cc995", "#2f8a5a", "#d4f5e0"
WOOD, WOOD_D, WOOD_L = "#c99a62", "#8e6a40", "#e6c48e"


# ── helpers ──
def ring(a, cx, cy, rx, ry, w, fill, a0=0, a1=360, n=72, **kw):
    pts = [(cx + rx * math.cos(math.radians(a0 + (a1 - a0) * k / n)),
            cy + ry * math.sin(math.radians(a0 + (a1 - a0) * k / n))) for k in range(n + 1)]
    a.stroke(pts, w, fill, smooth=False, **kw)


def star(a, x, y, r, fill, thin=0.26, alpha=1.0):
    """4-point twinkle."""
    pts = []
    for k in range(8):
        ang = math.radians(k * 45 - 90)
        rr = r if k % 2 == 0 else r * thin
        pts.append((x + rr * math.cos(ang), y + rr * math.sin(ang)))
    a.poly(pts, fill, alpha=alpha)


def halo(a, x, y, r, color, alpha=0.5, steps=6):
    for k in range(steps):
        t = (k + 1) / steps
        a.ellipse(x, y, r * (1.05 - 0.8 * t), r * (1.05 - 0.8 * t), color, alpha=alpha / steps * 1.6)


def light(a, side="right", alpha=0.16, rim=0.35):
    a.shade_side("#1a1030", alpha, side=side)
    a.rim("#ffffff", rim)


def gem(a, x, y, rx, ry, light_c, mid, dark, hl="#ffffff"):
    a.ellipse(x, y, rx, ry, ("rad", light_c, dark, ((x - rx * 0.3) / 16, (y - ry * 0.35) / 16)))
    a.ellipse(x + rx * 0.15, y + ry * 0.2, rx * 0.72, ry * 0.68, mid, alpha=0.55)
    a.ellipse(x - rx * 0.38, y - ry * 0.4, rx * 0.28, ry * 0.25, hl)


def glow(a, color, radius=1.1, alpha=0.6):
    a.glow(color, radius, alpha)


# ═════════════════ equipment slots ═════════════════
def hat():
    a = Art(16, 16)
    a.stroke([(5.2, 10.8), (8, 14.4), (10.8, 10.8)], 0.8, RED, smooth=False)
    a.poly([(8, 1.7), (13.8, 7.2), (15.2, 8.8), (15.2, 11), (0.8, 11), (0.8, 8.8), (2.2, 7.2)],
           ("lin", "#f2deaa", "#c9a463", 90))
    for x0 in (2.2, 4.6, 7.0, 9.4, 11.6, 13.8):
        a.stroke([(8, 2.2), (x0, 9)], 0.32, "#a8854a", alpha=0.75)
    a.rect(0.8, 9, 15.2, 11, ("lin", "#b8955a", "#8a6a3a", 90), radius=0.6)
    a.stroke([(1.2, 9.1), (14.8, 9.1)], 0.35, "#f2deaa", alpha=0.6)
    a.stroke([(7.8, 2.5), (4.6, 6.6)], 0.55, "#fff2cc", alpha=0.85)
    light(a)
    a.outline(width=OL)
    a.save("gear/hat")


def top():
    a = Art(16, 16)
    body = ("lin", "#94b09f", "#6a8576", 90)
    a.rect(0.9, 2.4, 5, 12, body, radius=0.7)
    a.rect(11, 2.4, 15.1, 12, body, radius=0.7)
    a.rect(0.9, 10, 5, 12, "#566e62", radius=0.6)
    a.rect(11, 10, 15.1, 12, "#566e62", radius=0.6)
    a.poly([(4, 2.6), (6, 1.5), (10, 1.5), (12, 2.6), (12, 15), (4, 15)], body)
    a.poly([(5.9, 1.4), (10.1, 1.4), (8.6, 12), (7.4, 12)], "#3e4a58")
    a.stroke([(5.6, 1.8), (7.3, 12.2), (7.4, 15)], 1.0, "#efe6d2", smooth=False)
    a.stroke([(10.4, 1.8), (8.7, 12.2), (8.6, 15)], 1.0, "#efe6d2", smooth=False)
    a.rect(7.2, 11.6, 8.8, 15, "#efe6d2")
    for x in (4.6, 11.4):
        a.stroke([(x, 3), (x, 14.6)], 0.4, "#566e62")
    a.stroke([(6.0, 8.6), (7.4, 8.6)], 0.8, RED)
    a.stroke([(8.6, 8.6), (10.0, 8.6)], 0.8, RED)
    light(a)
    a.outline(width=OL)
    a.save("gear/top")


def bottom():
    a = Art(16, 16)
    fab = ("lin", "#8d88a6", "#6a6582", 90)
    a.poly([(3.6, 3.5), (8.2, 3.5), (7.2, 11), (7.2, 15), (1.2, 15)], fab)
    a.poly([(12.4, 3.5), (7.8, 3.5), (8.8, 11), (8.8, 15), (14.8, 15)], fab)
    for x0, x1 in ((5.6, 4.0), (10.4, 12.0), (4.6, 2.4), (11.4, 13.6)):
        a.stroke([(x0, 4.5), (x1, 14.8)], 0.45, "#524d66")
    a.rect(3.4, 0.9, 12.6, 2.4, "#7d7894", radius=0.4)
    a.rect(3.4, 2.0, 12.6, 4.0, ("lin", "#f2e8cf", "#cdbf9c", 90), radius=0.3)
    a.ellipse(10.6, 4.6, 0.7, 0.6, "#b8a882")
    light(a)
    a.outline(width=OL)
    a.save("gear/bottom")


def gloves():
    a = Art(16, 16)
    skin = ("lin", "#f8dcc2", "#dcae8c", 90)
    for x, t in ((5, 3), (7, 2), (9, 2), (11, 3)):
        a.rect(x, t, x + 1.0, 6, skin, radius=0.5)
    a.rect(2.2, 6, 5.5, 10, skin, radius=0.9)
    a.rect(5, 5, 13.1, 10.2, "#8a8494", radius=0.6)
    a.rect(5.8, 5.8, 12.3, 8.7, ("lin", "#e4e8ef", "#9aa2b0", 90), radius=0.4)
    a.ellipse(7.5, 7.2, 0.4, 0.4, "#5f5a6a")
    a.ellipse(10.5, 7.2, 0.4, 0.4, "#5f5a6a")
    a.rect(5, 10, 13.1, 15, ("lin", "#8a8494", "#5f5a6a", 0), radius=0.5)
    for y in (11.0, 13.0):
        a.stroke([(5.8, y), (8.8, y + 0.6), (12.2, y)], 0.55, RED, smooth=False)
        a.stroke([(5.8, y + 1.2), (8.8, y + 0.6), (12.2, y + 1.2)], 0.55, "#a32f35", smooth=False)
    light(a)
    a.outline(width=OL)
    a.save("gear/gloves")


def shoes():
    a = Art(16, 16)
    for x0 in (1, 9):
        a.rect(x0, 2, x0 + 6, 15, ("lin", "#d8ad78", "#a87c4c", 0), radius=1.0)
        a.rect(x0, 11.6, x0 + 6, 12.6, WOOD_D)
        a.stroke([(x0 + 1.5, 9.2), (x0 + 1.5, 11)], 0.5, WOOD_L, alpha=0.9)
        a.stroke([(x0 + 1.5, 13.2), (x0 + 1.5, 14)], 0.5, WOOD_L, alpha=0.9)
        a.stroke([(x0 + 0.7, 8.6), (x0 + 3.5, 4.6), (x0 + 6.0, 8.0)], 0.85, RED, smooth=False)
        a.ellipse(x0 + 3.5, 4.6, 0.55, 0.55, RED_D)
    light(a)
    a.outline(width=OL)
    a.save("gear/shoes")


def necklace():
    a = Art(16, 16)
    cord = [(1.4, 0.6), (2.2, 4.2), (4.6, 6.6), (8, 7.6), (11.4, 6.6), (13.8, 4.2), (14.6, 0.6)]
    a.stroke(cord, 1.0, "#c8a878")
    a.stroke(cord, 0.35, "#8a6a44", alpha=0.6)
    a.stroke([(7.2, 7.4), (7.0, 9.0)], 0.7, "#c8a878")
    # magatama: round head (top-left) with a tapering tail curling down-right
    jade = ("rad", "#9fe8bd", "#3f9e6a", (0.36, 0.6))
    a.stroke([(8.0, 9.0), (10.4, 10.6), (11.0, 12.8), (9.8, 14.4), (7.6, 15.0)], 2.6, jade, taper=True)
    a.ellipse(6.9, 11.0, 3.0, 2.9, jade)
    a.stroke([(10.0, 11.4), (10.2, 13.2), (9.0, 14.4)], 0.6, j, alpha=0.6)
    a.ellipse(7.0, 10.0, 0.75, 0.75, K)
    a.ellipse(5.3, 11.6, 0.45, 0.85, Q)
    light(a, rim=0.25)
    a.outline(width=OL)
    a.save("gear/necklace")


def ring_icon():
    a = Art(16, 16)
    ring(a, 8, 10.6, 4.1, 3.6, 2.2, ("lin", "#f4dc98", "#b8963e", 45))
    ring(a, 8, 10.6, 4.1, 3.6, 0.6, "#a8883a", a0=-30, a1=110, alpha=0.7)
    ring(a, 8, 10.6, 4.4, 3.9, 0.45, "#fff2cc", a0=150, a1=250, alpha=0.8)
    a.ellipse(8, 4.6, 2.8, 2.6, ("lin", "#c8a248", "#8a6a2a", 90))
    gem(a, 8, 4.5, 2.0, 1.85, "#b8f0cf", J, j, Q)
    light(a, rim=0.25)
    a.outline(width=OL)
    a.save("gear/ring")


def earring():
    a = Art(16, 16)
    a.stroke([(8.3, 3.0), (8.4, 1.6), (9.4, 1.2), (10.5, 2.0), (10.5, 3.6), (9.8, 4.2)], 0.7, "#cdd2dc")
    gem(a, 7.6, 4.6, 1.6, 1.4, "#b8f0cf", J, j, Q)
    a.rect(5.8, 6, 10.2, 7.4, ("lin", RED, RED_D, 90), radius=0.4)
    a.poly([(6, 7.2), (10, 7.2), (11.2, 15), (4.8, 15)], ("lin", "#f5ead4", "#cdb994", 0))
    for x0, x1 in ((7, 6.0), (8, 8.6), (9, 10.2)):
        a.stroke([(x0, 8), (x1, 14.6)], 0.38, "#b09c78", alpha=0.85)
    light(a)
    a.outline(width=OL)
    a.save("gear/earring")


def charm():
    a = Art(16, 16)
    a.stroke([(7.0, 4.0), (7.2, 2.2), (8.0, 1.6), (8.8, 2.2), (9.0, 4.0)], 0.75, "#c8a878")
    a.ellipse(8, 9.6, 5.6, 5.6, ("rad", "#e05050", "#8a2430", (0.42, 0.48)))
    a.ellipse(8, 8.3, 3.0, 2.5, ("lin", "#fffaf0", "#e8dcc6", 90))
    # brows
    a.stroke([(5.1, 6.2), (6.6, 5.8)], 0.55, K)
    a.stroke([(9.4, 5.8), (10.9, 6.2)], 0.55, K)
    a.ellipse(6.6, 8.4, 0.75, 0.75, K)       # painted eye
    ring(a, 9.6, 8.4, 0.75, 0.75, 0.35, K)    # unpainted eye
    a.stroke([(7.6, 10.0), (8.4, 10.0)], 0.4, RED_D)
    a.blob([(5, 12.0), (8, 11.6), (11, 12.0), (10.8, 13.6), (8, 14.0), (5.2, 13.6)], ("lin", "#f4d27a", "#c89a3a", 90))
    a.ellipse(5.2, 6.6, 1.0, 0.7, "#ffffff", alpha=0.35)
    light(a, rim=0.25)
    a.outline(width=OL)
    a.save("gear/charm")


def omamori():
    a = Art(16, 16)
    a.blob([(5.4, 4.4), (10.6, 4.4), (12.8, 6.4), (12.9, 14.0), (12.4, 15), (3.6, 15), (3.1, 14.0), (3.2, 6.4)],
           ("lin", "#f6eedc", "#d2c4a2", 70))
    a.rect(3.2, 13.8, 12.9, 15, "#b8a886", radius=0.6, clip=True, alpha=0.8)
    gold = ("lin", "#f0d070", "#c8962e", 90)
    a.stroke([(6.6, 7.4), (9.4, 7.4)], 0.8, gold)
    a.stroke([(8, 7.4), (8, 12.8)], 0.8, gold)
    a.stroke([(6.0, 10.4), (10.0, 10.4)], 0.8, gold)
    a.ellipse(5.9, 2.4, 1.6, 1.3, ("lin", "#e05050", RED_D, 90))
    a.ellipse(10.1, 2.4, 1.6, 1.3, ("lin", "#e05050", RED_D, 90))
    a.ellipse(5.9, 2.4, 0.5, 0.4, K)
    a.ellipse(10.1, 2.4, 0.5, 0.4, K)
    a.rect(7, 2, 9, 4.6, RED, radius=0.5)
    a.stroke([(7.4, 4.2), (6.0, 5.8)], 0.6, RED)
    a.stroke([(8.6, 4.2), (10.0, 5.8)], 0.6, RED)
    light(a)
    a.outline(width=OL)
    a.save("gear/omamori")


def arsenal():
    a = Art(16, 16)
    px, py = 12.5, 12.5
    pts = []
    for k in range(17):
        ang = math.radians(182 + k * 5.4)
        pts.append((px + 11.3 * math.cos(ang), py + 11.3 * math.sin(ang)))
    for k in range(16, -1, -1):
        ang = math.radians(182 + k * 5.4)
        pts.append((px + 3.6 * math.cos(ang), py + 3.6 * math.sin(ang)))
    a.poly(pts, ("lin", "#fbf0d6", "#d8c096", 45))
    for k in range(6):
        ang = math.radians(186 + k * 16.5)
        a.stroke([(px + 3.6 * math.cos(ang), py + 3.6 * math.sin(ang)),
                  (px + 10.8 * math.cos(ang), py + 10.8 * math.sin(ang))], 0.3, "#b89a6a", alpha=0.85)
    ring(a, px, py, 10.9, 10.9, 0.5, "#b89a6a", a0=182, a1=268, alpha=0.6)
    a.ellipse(6.4, 6.6, 1.7, 1.7, ("rad", "#e85a50", "#b02c30"))
    # katana
    a.stroke([(4.4, 11.6), (14.7, 1.2)], 1.3, ("lin", "#ffffff", "#aab2c0", 45))
    a.stroke([(5.0, 12.1), (14.6, 2.1)], 0.45, "#8a92a2", smooth=False)
    a.stroke([(4.6, 11.0), (14.2, 1.3)], 0.3, "#ffffff", smooth=False)
    a.stroke([(2.6, 10.6), (5.6, 13.6)], 1.0, ("lin", "#f4d27a", "#b88a2a", 0), smooth=False)
    a.stroke([(3.8, 12.4), (0.9, 15.2)], 1.5, "#3a2a2a")
    for t in (0.25, 0.55, 0.85):
        x, y = 3.8 - 2.9 * t, 12.4 + 2.8 * t
        a.ellipse(x, y, 0.35, 0.35, "#8a6a5a")
    light(a)
    a.outline(width=OL)
    a.save("gear/arsenal")


def dust():
    a = Art(16, 16)
    a.blob([(1.2, 15), (3.6, 11.6), (6.6, 9.8), (9.8, 10.0), (12.6, 11.8), (14.9, 15)],
           ("lin", "#d2f0ff", "#5f9fd0", 90), smooth=True)
    a.rect(0.9, 14.2, 15.1, 15.2, "#5f9fd0")
    for x, y, r in ((4.6, 13.2, 0.4), (7.4, 11.4, 0.45), (9.6, 13.4, 0.35), (11.6, 12.6, 0.4), (6.2, 14.2, 0.3)):
        a.ellipse(x, y, r, r, "#f0faff")
    star(a, 5.4, 4.4, 2.9, ("rad", "#ffffff", "#9fd8ff"))
    star(a, 11.3, 3.6, 1.8, ("rad", "#ffffff", "#9fd8ff"))
    star(a, 12.2, 7.9, 1.5, ("rad", "#ffffff", "#9fd8ff"))
    a.ellipse(2.0, 8.6, 0.6, 0.6, "#9fd8ff")
    a.ellipse(6.4, 8.2, 0.5, 0.5, "#cfefff")
    a.rim("#ffffff", 0.3)
    a.outline(width=OL)
    glow(a, "#9fd8ff", 0.9, 0.35)
    a.save("gear/dust")


def draw():
    a = Art(16, 16)
    a.stroke([(10.6, 4.8), (12.6, 1.6)], 1.2, ("lin", "#fff2d2", "#d8c08e", 0))
    a.stroke([(12.4, 1.9), (13.4, 0.7)], 1.3, RED)
    a.poly([(3.4, 3.8), (12.6, 3.8), (13.6, 5.8), (2.4, 5.8)], ("lin", "#f2d8a8", "#d8b47c", 90))
    a.ellipse(10.3, 4.6, 1.2, 0.5, "#5a4028")
    a.rect(2.4, 5.8, 13.6, 15, ("lin", "#d8ad78", "#b88a54", 90), radius=0.4)
    a.rect(2.4, 5.8, 4.4, 15, WOOD_D, alpha=0.75)
    a.rect(11.6, 5.8, 13.6, 15, WOOD_D, alpha=0.75)
    a.stroke([(4.4, 6), (4.4, 15)], 0.3, "#e6c48e", alpha=0.6)
    a.rect(5.9, 7.9, 10.1, 14.1, ("lin", "#fffaec", "#efe0bf", 90), radius=0.3)
    a.rect(7.0, 9.0, 9.0, 9.9, RED, radius=0.2)
    a.rect(7.0, 10.9, 9.0, 13.0, RED, radius=0.2)
    light(a)
    a.outline(width=OL)
    a.save("gear/draw")


# ═════════════════ exotics ═════════════════
def lantern_heart():
    a = Art(16, 16)
    a.rect(6.2, 0.9, 9.8, 3.4, ("lin", H, g, 90), radius=0.8)
    a.blob([(8, 4.6), (9.6, 3.1), (12.4, 2.9), (14.6, 5.0), (14.6, 7.4), (12.2, 9.9),
            (9.6, 12.2), (8, 13.6), (6.4, 12.2), (3.8, 9.9), (1.4, 7.4), (1.4, 5.0), (3.6, 2.9), (6.4, 3.1)],
           ("rad", "#ff9a3e", "#a82230", (0.5, 0.48)))
    for pts in ([(4.5, 3.6), (3.6, 7.2), (6.2, 11.6)], [(11.5, 3.6), (12.4, 7.2), (9.8, 11.6)],
                [(8, 4.8), (8, 13)]):
        a.stroke(pts, 0.35, "#8e1f2e", alpha=0.7)
    a.ellipse(8, 7.7, 2.6, 2.4, ("rad", "#fffbe6", "#ff9a2e00", (0.5, 0.48)))
    a.ellipse(8, 7.6, 1.3, 1.2, "#fffbe6")
    a.ellipse(4.0, 5.0, 1.0, 0.6, "#ffffff", alpha=0.45)
    a.rect(7, 13.6, 9, 15.2, ("lin", H, g, 90), radius=0.5)
    a.rect(5.4, 3.0, 10.6, 3.8, G, radius=0.4)
    light(a, alpha=0.12)
    a.outline(width=OL)
    glow(a, "#ff7a2e", 1.2, 0.65)
    halo(a, 8, 7.7, 3.0, "#ffe27a", 0.25)
    a.save("exotic/lantern_heart")


def jeweled_branch():
    a = Art(16, 16)
    gold = ("lin", H, g, 45)
    a.stroke([(1.2, 14.4), (5, 10.6), (8.6, 6.6), (11.4, 3.6)], 1.5, gold)
    a.stroke([(8.4, 7.0), (11.4, 9.0)], 1.0, gold)
    a.stroke([(4.6, 7.4), (3.8, 5.6)], 0.9, gold)
    a.stroke([(5.8, 10.0), (9.4, 12.2)], 0.8, gold)
    a.stroke([(1.6, 13.6), (8.4, 6.4)], 0.4, H, alpha=0.8)
    gem(a, 12.0, 3.0, 2.3, 2.1, "#bfe4ff", "#4aa8ff", "#1f5ec0")
    gem(a, 13.0, 9.0, 2.1, 2.0, "#ffd0ea", "#ff7ac0", "#b8307a")
    gem(a, 2.6, 3.9, 2.0, 1.6, "#ffffff", "#e8eefc", "#a8b4d8")
    # silver crescent charm
    a.blob([(9.0, 11.0), (11.2, 12.0), (12.4, 14.2), (10.6, 15.0), (8.8, 14.4), (10.6, 13.6), (10.4, 12.2)],
           ("lin", "#ffffff", "#9aa6c8", 45))
    light(a, alpha=0.12)
    a.outline(width=OL)
    glow(a, "#f2c94c", 1.0, 0.5)
    star(a, 7.2, 2.2, 1.4, "#ffffff")
    star(a, 15.0, 6.0, 0.9, "#ffe9a0")
    a.save("exotic/jeweled_branch")


def unbroken_yinyang():
    a = Art(16, 16)
    a.ellipse(8, 8, 7.4, 7.4, ("lin", H, g, 45))
    a.ellipse(8, 8, 6.0, 6.0, "#f6f2ff")
    R = 6.0
    half = [(8 + R * math.cos(math.radians(t)), 8 + R * math.sin(math.radians(t))) for t in range(-90, 91, 4)]
    a.poly(half, ("lin", "#5a4a96", "#2c2254", 45))
    a.ellipse(8, 11, 3.0, 3.0, ("lin", "#4a3f82", "#2c2254", 45))
    a.ellipse(8, 5, 3.0, 3.0, "#f6f2ff")
    a.ellipse(8, 5, 1.0, 1.0, "#3b2f6a")
    a.ellipse(8, 11, 1.0, 1.0, "#f6f2ff")
    ring(a, 8, 8, 6.7, 6.7, 0.4, "#ffffff", a0=170, a1=280, alpha=0.8)
    ring(a, 8, 8, 6.0, 6.0, 0.35, g, alpha=0.8)
    a.ellipse(5.0, 5.0, 1.0, 0.7, "#ffffff", alpha=0.8)
    light(a, alpha=0.1, rim=0.3)
    a.outline(width=OL)
    glow(a, "#b8a8ff", 1.1, 0.55)
    star(a, 14.0, 2.0, 1.3, "#fff4b8")
    a.save("exotic/unbroken_yinyang")


def nine_seal():
    a = Art(16, 16)
    a.blob([(10.4, 7.4), (9.4, 5.2), (10.0, 3.4), (11.6, 1.2), (12.4, 3.0), (13.6, 1.8), (13.4, 4.2),
            (13.0, 6.4), (11.6, 7.6)], ("lin", "#ffd85a", "#ff7a1e", 90))
    a.ellipse(11.6, 5.0, 1.1, 1.5, "#fffbe0")
    a.rect(1.0, 7.4, 15.0, 15.2, ("lin", "#fbf3df", "#d8c49c", 90), radius=0.4)
    a.rect(1.0, 7.0, 2.4, 15.4, ("lin", H, g, 0), radius=0.6)
    a.rect(13.6, 7.0, 15.0, 15.4, ("lin", H, g, 0), radius=0.6)
    a.rect(2.4, 13.8, 13.6, 15.0, "#c8b48c", alpha=0.7)
    for cx in (3.6, 5.6, 7.6):
        for cy in (9.5, 11.5, 13.3):
            a.rect(cx - 0.45, cy - 0.45, cx + 0.45, cy + 0.45, "#d8303a", radius=0.15)
    a.rect(10.0, 7.4, 11.8, 15.2, ("lin", "#c040b0", "#7a1a6a", 0))
    a.stroke([(11.8, 9.0), (12.6, 9.6), (12.4, 12.2)], 0.6, "#9a2a8a")
    light(a, alpha=0.12)
    a.outline(width=OL)
    glow(a, "#ff8a1e", 1.0, 0.5)
    halo(a, 11.6, 4.6, 3.0, "#ffd85a", 0.3)
    star(a, 9.2, 1.4, 1.0, "#ffd85a")
    star(a, 15.2, 3.6, 0.9, "#ffd85a")
    a.save("exotic/nine_seal")


def clone_scroll():
    a = Art(16, 16)
    side = ("lin", "#8a9ae0", "#4a58a8", 90)
    for cx in (3.0, 13.0):
        a.ellipse(cx, 4.4, 1.6, 1.6, side)
        a.blob([(cx - 2.1, 11.2), (cx - 2.0, 7.4), (cx - 1.0, 6.2), (cx + 1.0, 6.2), (cx + 2.0, 7.4), (cx + 2.1, 11.2)], side)
    a.ellipse(8, 2.9, 1.9, 1.9, ("lin", "#ffffff", "#9fd4f4", 90))
    a.blob([(4.9, 11.2), (5.2, 7.0), (6.6, 5.3), (9.4, 5.3), (10.8, 7.0), (11.1, 11.2)],
           ("lin", "#e4f6ff", "#8ac4ec", 90))
    a.stroke([(10.3, 7.0), (10.4, 11)], 0.5, "#8ac4ec", alpha=0.8)
    a.ellipse(7.4, 2.4, 0.6, 0.5, "#ffffff")
    a.rect(1.0, 10.8, 15.0, 14.0, ("lin", "#fbf3df", "#c8b48c", 90), radius=0.3)
    for x in (4.2, 6.2, 9.8, 11.8):
        a.rect(x, 11.6, x + 0.6, 12.4, "#b8a47c")
    a.rect(0.6, 10.4, 1.9, 14.4, ("lin", H, g, 0), radius=0.5)
    a.rect(14.1, 10.4, 15.4, 14.4, ("lin", H, g, 0), radius=0.5)
    light(a, alpha=0.12)
    a.outline(width=OL)
    glow(a, "#8ac4ec", 1.2, 0.6)
    star(a, 1.0, 1.4, 0.9, "#bfeaff")
    star(a, 15.0, 1.6, 0.9, "#bfeaff")
    a.save("exotic/clone_scroll")


def hollow_mask():
    a = Art(16, 16)
    a.blob([(5.2, 0.8), (10.4, 0.8), (13.2, 2.2), (14.2, 5.2), (13.8, 8.6), (12.0, 10.4), (10.6, 12.2),
            (9.6, 14.2), (7.4, 15.1), (5.6, 14.0), (4.0, 12.2), (2.2, 10.0), (1.4, 6.6), (2.2, 3.2)],
           ("lin", "#fffaf0", "#d8cfb8", 30))
    a.blob([(11.4, 1.6), (13.2, 2.2), (14.2, 5.2), (13.8, 8.6), (12.0, 10.4), (11.4, 6.0)], "#c8bfa8", alpha=0.8)
    a.rect(5.0, 12.8, 10.0, 15.2, "#c8bfa8", clip=True, alpha=0.85)
    # red stripes
    a.stroke([(10.6, 1.0), (10.9, 4.5)], 0.7, "#d02a3a", taper=True)
    a.stroke([(12.2, 1.8), (12.8, 5.4)], 0.6, "#d02a3a", taper=True)
    a.stroke([(2.4, 8.8), (4.2, 11.6)], 0.6, "#d02a3a")
    a.stroke([(10.4, 9.0), (10.4, 11.8)], 0.6, "#d02a3a")
    # eyes
    a.blob([(4.0, 6.0), (6.6, 5.6), (7.2, 7.6), (5.4, 8.8), (3.8, 7.8)], "#2a1a2a")
    a.ellipse(5.5, 7.1, 1.0, 0.9, ("rad", "#fff8b0", "#ff8a1e", (0.34, 0.44)))
    a.blob([(9.0, 5.4), (11.6, 5.6), (11.8, 7.8), (10.0, 8.4), (8.8, 7.2)], "#2a1a2a")
    # cracks + teeth
    a.stroke([(5.0, 1.0), (6.0, 2.8), (6.8, 3.6), (7.4, 5.0)], 0.35, "#2a1a2a", smooth=False)
    a.stroke([(6.0, 2.8), (5.0, 3.8)], 0.3, "#2a1a2a")
    for x in (5.6, 7.6, 9.4):
        a.rect(x - 0.35, 11.8, x + 0.35, 12.9, "#2a1a2a", radius=0.2)
    a.ellipse(3.6, 3.4, 1.0, 0.7, "#ffffff", alpha=0.6)
    light(a, alpha=0.12)
    a.outline(width=OL)
    halo(a, 5.4, 7.1, 3.4, "#ff9a2e", 0.55)
    a.ellipse(5.5, 7.1, 0.5, 0.45, "#ffffff")
    a.save("exotic/hollow_mask")


def white_haori():
    a = Art(16, 16)
    white = ("lin", "#ffffff", "#d4d8ea", 90)
    a.rect(0.0, 1.8, 16.0, 11.0, white, radius=0.3)
    a.rect(2.4, 1.8, 13.6, 15.2, white, radius=0.4)
    a.rect(0.0, 9.0, 2.8, 11.0, "#2a2a4a")
    a.rect(13.2, 9.0, 16.0, 11.0, "#2a2a4a")
    a.stroke([(2.8, 3), (2.8, 11)], 0.4, "#c8cce0")
    a.stroke([(13.2, 3), (13.2, 11)], 0.4, "#c8cce0")
    a.stroke([(3.4, 11), (3.4, 14.6)], 0.4, "#c8cce0")
    a.stroke([(12.6, 11), (12.6, 14.6)], 0.4, "#c8cce0")
    a.rect(5.0, 0.8, 11.0, 2.4, ("lin", "#4a4a7a", "#2a2a4a", 90), radius=0.6)
    a.rect(2.4, 12.8, 13.6, 13.8, ("lin", H, g, 0))
    a.rect(2.4, 13.8, 13.6, 15.2, ("lin", "#4a4a7a", "#2a2a4a", 90))
    # crest
    ring(a, 8, 6.6, 1.7, 1.7, 0.7, ("lin", H, g, 45))
    a.ellipse(8, 6.6, 0.55, 0.55, G)
    for k in range(4):
        ang = math.radians(45 + 90 * k)
        a.ellipse(8 + 1.7 * math.cos(ang), 6.6 + 1.7 * math.sin(ang), 0.45, 0.45, G)
    light(a, alpha=0.12)
    a.outline(width=OL)
    glow(a, "#fff4b8", 0.8, 0.4)
    a.save("exotic/white_haori")


def hunger_organ():
    a = Art(16, 16)
    T = "#b02a4a"
    for pts in ([(11.4, 5.6), (13.4, 4.4), (14.8, 4.0)], [(4.6, 6.2), (2.6, 7.0), (1.0, 6.4)],
                [(6.2, 11.6), (4.2, 13.2), (1.8, 14.0)], [(8.0, 12.4), (8.2, 15.2)],
                [(10.0, 11.6), (12.0, 13.0), (14.4, 15.0)], [(12.0, 13.0), (13.6, 12.0)]):
        a.stroke(pts, 1.25, ("lin", "#e8405e", "#a02444", 90), taper=True)
    a.poly([(8, 0.8), (11.8, 4.0), (12.2, 9.4), (10.6, 11.8), (8, 12.8), (5.4, 11.8), (3.8, 9.4), (4.2, 4.0)],
           ("lin", "#a82648", "#3e0a20", 0))
    a.poly([(8, 0.8), (4.2, 4.0), (5.6, 7.0)], "#c03050")
    a.poly([(8, 0.8), (11.8, 4.0), (9.2, 6.0)], "#6a1230")
    a.poly([(3.8, 9.4), (5.4, 11.8), (8, 12.8), (8, 9.6)], "#7a1838")
    a.stroke([(8, 1.2), (8.2, 6.0), (7.6, 12.2)], 0.35, "#e8304a", alpha=0.8)
    a.stroke([(5.2, 4.4), (6.4, 9.6)], 0.3, "#e8304a", alpha=0.7)
    a.ellipse(8.3, 7.4, 1.9, 1.8, ("rad", "#ffe0e8", "#e8304a00", (0.52, 0.46)))
    a.ellipse(8.3, 7.3, 0.8, 0.8, "#ffe0e8")
    light(a, alpha=0.1, rim=0.3)
    a.outline(width=OL)
    glow(a, "#e8304a", 1.1, 0.55)
    halo(a, 8.3, 7.3, 2.6, "#ff8aa0", 0.35)
    a.save("exotic/hunger_organ")


def thousand_counter():
    a = Art(16, 16)
    # band with red tick segments
    ring(a, 8, 11.2, 5.0, 3.4, 2.0, ("lin", "#5e5470", "#2a2236", 90))
    for k in range(10):
        a0 = 200 + k * 36
        ring(a, 8, 11.2, 5.0, 3.4, 0.9, ("lin", "#ff4a50", "#8e1a2a", 90), a0=a0, a1=a0 + 14, n=6)
    a.ellipse(8, 3.6, 3.4, 3.3, ("lin", H, g, 45))
    a.ellipse(8, 3.6, 2.4, 2.3, ("rad", "#ffffff", "#f2e2c8"))
    for k in range(4):
        ang = math.radians(90 * k)
        a.ellipse(8 + 1.8 * math.cos(ang), 3.6 + 1.8 * math.sin(ang), 0.28, 0.28, "#8e1a2a")
    a.stroke([(8, 3.6), (8, 2.0)], 0.4, "#3c3448")
    a.stroke([(8, 3.6), (9.4, 4.2)], 0.4, "#e02a3a")
    a.rect(6.4, 6.4, 9.6, 7.6, ("lin", G, g, 90), radius=0.4)
    light(a, alpha=0.1)
    a.outline(width=OL)
    glow(a, "#e02a3a", 1.0, 0.5)
    star(a, 13.6, 1.6, 1.1, "#fff4b8")
    a.save("exotic/thousand_counter")


def sovereign_finger():
    a = Art(16, 16)
    a.stroke([(2.2, 12.6), (5.4, 9.4), (8.6, 6.0), (12.4, 2.6)], 3.0, ("lin", "#6a3a84", "#2e1640", 90))
    a.ellipse(5.4, 9.4, 1.8, 1.7, "#5a3070")
    a.ellipse(9.0, 5.6, 1.7, 1.6, "#5a3070")
    a.stroke([(2.0, 11.6), (11.6, 2.0)], 0.5, "#8a5aa8", alpha=0.7)
    for x, y in ((5.4, 9.4), (9.0, 5.6)):
        for o in (-0.45, 0.45):
            a.stroke([(x - 1.0 + o, y - 1.0 - o), (x + 1.0 + o, y + 1.0 - o)], 0.28, "#2a1238", alpha=0.85)
    a.blob([(11.0, 2.4), (12.4, 1.2), (14.0, 1.6), (14.2, 3.0), (13.0, 4.2)], ("lin", "#f6e6ff", "#a888c0", 45))
    a.ellipse(12.6, 2.2, 0.5, 0.35, "#ffffff", alpha=0.8)
    # talisman strip
    a.stroke([(4.4, 7.0), (7.4, 10.8), (10.6, 15.0)], 1.7, ("lin", "#fbf2dc", "#c8b48c", 0), smooth=False)
    for t in (0.18, 0.42, 0.66, 0.88):
        x, y = 4.4 + 6.2 * t, 7.0 + 8.0 * t
        a.ellipse(x, y, 0.4, 0.4, "#d07aff")
    light(a, alpha=0.1)
    a.outline(width=OL)
    glow(a, "#d07aff", 1.1, 0.6)
    star(a, 14.8, 0.9, 1.3, "#f2d0ff")
    a.save("exotic/sovereign_finger")


def closed_hand_vow():
    a = Art(16, 16)
    ring(a, 8.6, 11.0, 3.6, 3.6, 1.6, ("lin", G, "#8a5e14", 30), a0=-60, a1=200)
    a.rect(2.6, 0.8, 13.4, 9.6, ("lin", H, g, 45), radius=2.4)
    a.rect(3.9, 2.0, 12.1, 8.6, ("lin", "#a8243a", "#5a1020", 90), radius=1.6)
    # fist sigil: four curled fingers with the thumb folded across
    a.rect(4.8, 2.6, 11.2, 8.0, "#7a1a2a", radius=1.2)
    for x in (5.0, 6.6, 8.2, 9.8):
        a.rect(x, 2.8, x + 1.4, 6.4, ("lin", "#fff0c8", "#e0b880", 90), radius=0.6)
    a.rect(5.0, 5.7, 9.6, 7.6, ("lin", "#fff6dc", "#e8c890", 90), radius=0.8)
    a.ellipse(9.4, 6.6, 0.5, 0.6, "#b8844a")
    # binding cord
    a.stroke([(0.8, 15.0), (2.8, 12.6), (5.4, 11.0), (8.0, 10.2)], 1.3, ("lin", "#ff4a50", "#8e1a2a", 90))
    for t in (0.25, 0.55, 0.85):
        x, y = 0.8 + 7.2 * t, 15.0 - 4.8 * t
        a.stroke([(x - 0.4, y - 0.5), (x + 0.3, y + 0.5)], 0.3, "#8e1a2a")
    a.ellipse(4.6, 1.8, 0.9, 0.5, "#ffffff", alpha=0.7)
    light(a, alpha=0.1)
    a.outline(width=OL)
    glow(a, "#f2c94c", 1.0, 0.55)
    star(a, 14.8, 1.2, 1.0, "#fff4b8")
    a.save("exotic/closed_hand_vow")


def kusanagi():
    a = Art(16, 16)
    a.poly([(5.2, 11.6), (4.2, 10.4), (13.4, 1.6), (15.4, 0.6), (14.6, 2.6)], ("lin", "#f0c070", "#a86a30", 45))
    a.poly([(4.7, 11.0), (14.4, 1.4), (15.4, 0.6), (5.2, 11.6)], "#9a6430", alpha=0.6)
    a.stroke([(4.9, 10.8), (14.6, 1.4)], 0.4, "#fff0c8", smooth=False)
    a.stroke([(3.0, 8.6), (6.8, 12.6)], 1.1, ("lin", H, g, 0), smooth=False)
    a.stroke([(4.6, 11.4), (1.6, 14.2)], 1.3, "#4a2a2a")
    for t in (0.3, 0.7):
        x, y = 4.6 - 3.0 * t, 11.4 + 2.8 * t
        a.stroke([(x - 0.5, y - 0.5), (x + 0.5, y + 0.5)], 0.3, "#a8761e")
    a.ellipse(1.2, 14.8, 1.2, 1.1, ("lin", H, g, 45))
    light(a, alpha=0.1)
    a.outline(width=OL)
    glow(a, "#f2c94c", 1.1, 0.65)
    star(a, 13.2, 7.4, 1.6, "#fff4b8")
    star(a, 2.6, 5.6, 0.9, "#f2c94c")
    a.save("exotic/kusanagi")


def yata_mirror():
    a = Art(16, 16)
    a.ellipse(8, 8, 7.3, 7.3, ("lin", "#e0a858", "#7a4a20", 45))
    for k in range(12):
        ang = math.radians(k * 30 + 15)
        a.ellipse(8 + 6.4 * math.cos(ang), 8 + 6.4 * math.sin(ang), 0.55, 0.55, ("lin", H, G, 45))
    a.ellipse(8, 8, 5.3, 5.3, "#7a4a20")
    a.ellipse(8, 8, 4.8, 4.8, ("lin", "#ffffff", "#a8c0e0", 45))
    for off, w in ((-1.6, 0.7), (0.6, 0.45), (2.4, 0.3)):
        a.stroke([(5.4 + off, 11.0 + off), (11.0 + off, 5.4 + off)], w, "#ffffff", clip=False, alpha=0.9)
    ring(a, 8, 8, 4.8, 4.8, 0.5, "#a8c0e0", a0=-10, a1=100, alpha=0.6)
    light(a, alpha=0.12, rim=0.3)
    a.outline(width=OL)
    glow(a, "#ffe9a8", 1.1, 0.55)
    star(a, 1.4, 1.4, 1.5, ("rad", "#ffffff", G))
    a.save("exotic/yata_mirror")


def yasakani_jewel():
    a = Art(16, 16)
    gold = ("lin", H, g, 0)
    a.stroke([(8.4, 0.2), (8.6, 3.8)], 0.9, gold)
    a.stroke([(12.6, 0.2), (11.4, 2.4), (9.8, 4.0)], 0.9, gold)
    a.blob([(4.0, 4.6), (7.2, 3.0), (10.6, 3.8), (11.8, 6.4), (10.4, 8.6), (7.4, 9.2), (5.6, 10.4),
            (5.4, 13.0), (5.8, 15.2), (3.6, 13.6), (2.4, 10.8), (2.6, 7.2)],
           ("rad", "#7af0d8", "#0e5a72", (0.42, 0.36)))
    a.blob([(10.4, 4.6), (11.8, 6.4), (10.4, 8.6), (7.4, 9.2), (9.6, 7.2)], "#0e4a6a", alpha=0.55)
    a.ellipse(6.0, 5.8, 1.2, 1.0, "#e0fff6")
    a.ellipse(3.8, 11.4, 0.4, 0.9, "#9af0dc", alpha=0.8)
    a.ellipse(8.0, 4.0, 0.9, 0.7, G)
    light(a, alpha=0.1)
    a.outline(width=OL)
    glow(a, "#5ae0c8", 1.1, 0.6)
    star(a, 13.0, 13.0, 1.8, ("rad", "#ffffff", "#5ae0c8"))
    a.save("exotic/yasakani_jewel")


# ── Arsenal attacks ──
def raijin_drums():
    a = Art(16, 16)
    ring(a, 8, 8, 5.2, 5.2, 0.9, ("lin", G, g, 45))
    for k in range(6):
        ang = math.radians(-90 + 60 * k)
        x, y = 8 + 5.3 * math.cos(ang), 8 + 5.3 * math.sin(ang)
        a.ellipse(x, y, 2.1, 2.1, ("lin", "#f05050", "#8e1f2e", 90))
        a.ellipse(x, y, 1.3, 1.3, ("rad", "#fffaf0", "#e8d8b8"))
        a.ellipse(x, y, 0.45, 0.45, "#c03030", alpha=0.7)
    a.poly([(8.8, 3.6), (6.0, 8.4), (8.0, 8.4), (6.8, 12.6), (10.2, 7.2), (8.2, 7.2), (9.6, 3.6)],
           ("lin", "#fffbe0", "#ffd03a", 90))
    light(a, alpha=0.1)
    a.outline(width=OL)
    glow(a, "#ffe14d", 1.1, 0.55)
    halo(a, 8, 8, 2.6, "#ffe14d", 0.4)
    star(a, 1.0, 1.4, 1.2, "#ffe14d")
    star(a, 15.0, 14.2, 1.2, "#ffe14d")
    a.save("exotic/raijin_drums")


def fox(a, cx, cy):
    a.blob([(cx - 1.8, cy + 0.2), (cx - 2.6, cy - 1.6), (cx - 3.0, cy - 3.4), (cx - 1.6, cy - 2.4), (cx - 0.8, cy - 0.6)],
           ("lin", "#bfeaff", "#3a7aff", 90))
    a.ellipse(cx, cy, 2.1, 1.15, ("lin", "#ffffff", "#c8c8e0", 90))
    a.stroke([(cx - 1.2, cy + 0.6), (cx - 1.5, cy + 2.2)], 0.6, "#e8e8f4")
    a.stroke([(cx + 1.2, cy + 0.6), (cx + 1.4, cy + 2.2)], 0.6, "#e8e8f4")
    a.ellipse(cx + 2.0, cy - 1.0, 1.25, 1.05, "#fbf8ff")
    a.poly([(cx + 1.3, cy - 1.6), (cx + 1.8, cy - 3.1), (cx + 2.6, cy - 1.8)], "#fbf8ff")
    a.poly([(cx + 2.6, cy - 1.6), (cx + 3.7, cy - 0.7), (cx + 2.6, cy - 0.2)], "#fbf8ff")
    a.ellipse(cx + 2.3, cy - 1.2, 0.35, 0.3, "#e0303a")


def fox_procession():
    a = Art(16, 16)
    for cx, cy in ((12.0, 3.4), (7.6, 8.3), (3.2, 13.2)):
        fox(a, cx, cy)
    light(a, alpha=0.1)
    a.outline(width=OL)
    glow(a, "#3a7aff", 1.1, 0.6)
    a.save("exotic/fox_procession")


def tengu_fan():
    a = Art(16, 16)
    a.stroke([(8.4, 12.2), (5.6, 15.4)], 1.3, ("lin", "#e05050", RED_D, 0))
    for ang in (-172, -150, -128, -106, -84, -62, -40, -18, 4):
        r = math.radians(ang)
        L = 7.4 if abs(ang + 90) < 60 else 7.6
        dx, dy = math.cos(r), math.sin(r)
        nx, ny = -dy, dx
        bx, by = 8 + dx * 1.0, 11.4 + dy * 1.0
        tx, ty = 8 + dx * L, 11.4 + dy * L
        mx, my = 8 + dx * L * 0.62, 11.4 + dy * L * 0.62
        w = 1.35
        a.blob([(bx, by), (mx + nx * w, my + ny * w), (tx, ty), (mx - nx * w, my - ny * w)],
               ("lin", "#ffffff", "#9ab8a0", 90))
        a.stroke([(bx, by), (tx - dx * 0.6, ty - dy * 0.6)], 0.28, "#4a6a5a", alpha=0.85)
        a.blob([(mx + dx * 1.2 + nx * w * 0.8, my + dy * 1.2 + ny * w * 0.8), (tx, ty),
                (mx + dx * 1.2 - nx * w * 0.8, my + dy * 1.2 - ny * w * 0.8)], "#8aa890", alpha=0.6)
    a.ellipse(8, 11.4, 2.1, 1.3, ("lin", H, g, 90))
    a.ellipse(7.5, 11.0, 0.6, 0.35, "#ffffff", alpha=0.8)
    light(a, alpha=0.12)
    a.outline(width=OL)
    glow(a, "#8ad8ff", 1.0, 0.45)
    a.stroke([(12.4, 11.2), (14.0, 12.4), (14.8, 14.6)], 0.5, "#bfeaff", alpha=0.9)
    a.stroke([(1.4, 11.6), (1.0, 13.2), (2.2, 14.4)], 0.5, "#bfeaff", alpha=0.9)
    a.stroke([(12.0, 13.4), (13.0, 15.2)], 0.4, "#d8f4ff", alpha=0.8)
    a.save("exotic/tengu_fan")


def dragon_coil():
    a = Art(16, 16)
    a.stroke([(8.4, 2.6), (8.0, 0.4)], 0.8, ("lin", H, g, 0), taper=True)
    a.stroke([(11.6, 2.6), (12.6, 0.4)], 0.8, ("lin", H, g, 0), taper=True)
    ring(a, 7.6, 9.4, 4.6, 4.5, 3.4, ("lin", "#5ae0a0", "#1e7a5a", 60), a0=-80, a1=275, n=90)
    ring(a, 7.6, 9.4, 2.9, 2.8, 0.9, ("lin", H, g, 45), a0=-80, a1=275)
    for k in range(10):
        ang = math.radians(-60 + k * 33)
        a.ellipse(7.6 + 5.4 * math.cos(ang), 9.4 + 5.3 * math.sin(ang), 0.45, 0.4, "#1e7a5a", alpha=0.8)
    # head
    a.blob([(8.6, 3.4), (11.0, 2.2), (13.6, 3.0), (15.6, 4.8), (15.4, 6.2), (13.2, 6.6), (10.6, 6.0)],
           ("lin", "#6af0b0", "#2a9a6a", 90))
    a.ellipse(12.2, 3.8, 0.6, 0.5, "#e0303a")
    a.stroke([(15.0, 6.2), (15.6, 7.4)], 0.4, "#ffffff")
    a.stroke([(13.6, 6.4), (14.0, 7.8)], 0.35, "#ffffff", alpha=0.8)
    light(a, alpha=0.12)
    a.outline(width=OL)
    glow(a, "#3ac48a", 1.0, 0.55)
    a.save("exotic/dragon_coil")


def moonfall():
    a = Art(16, 16)
    for (x0, y0, x1, y1, w) in ((10.6, 3.0, 14.6, -0.2, 1.0), (12.0, 4.6, 15.8, 1.8, 0.8), (9.0, 1.6, 10.6, 0.2, 0.6)):
        a.stroke([(x0, y0), (x1, y1)], w, ("lin", "#bcd0ff", "#7a8ad8", 0), taper=False)
    a.blob([(1.4, 15.0), (3.0, 12.8), (5.6, 12.4), (8.0, 12.6), (10.6, 12.6), (12.8, 13.0), (14.6, 15.0)],
           ("lin", "#ffe27a", "#ff7a1e", 90))
    a.ellipse(7.2, 7.4, 5.9, 5.9, ("rad", "#fffcec", "#d8d0b0", (0.36, 0.34)))
    a.blob([(12.4, 5.0), (13.0, 8.4), (11.2, 11.8), (7.6, 13.2), (10.4, 10.4), (11.8, 7.2)], "#c8c0a0", alpha=0.8)
    for x, y, r in ((5.0, 6.4, 1.0), (8.6, 4.4, 0.7), (9.4, 9.8, 0.9), (5.8, 10.2, 0.5)):
        a.ellipse(x, y, r, r * 0.85, "#b8ae8e", alpha=0.75)
        a.ellipse(x - 0.2, y - 0.2, r * 0.7, r * 0.6, "#e8e2c8", alpha=0.6)
    a.rect(0, 14.8, 16, 16, ("lin", "#6a4a40", "#3a2626", 90))
    light(a, alpha=0.1)
    a.outline(width=OL)
    glow(a, "#ffe9b0", 1.0, 0.5)
    star(a, 2.2, 12.6, 1.0, "#ffe27a")
    star(a, 13.0, 12.4, 1.0, "#ffe27a")
    a.save("exotic/moonfall")


def skeleton_hand():
    a = Art(16, 16)
    bone = ("lin", "#fffaea", "#c8bd9e", 0)
    a.rect(0, 14.2, 16, 16, ("lin", "#8a6a58", "#4a3a3a", 90))
    a.stroke([(3.0, 0), (3.0, 2.6)], 1.7, bone)
    a.stroke([(8.2, 0), (8.2, 2.6)], 1.7, bone)
    a.rect(1.4, 2.0, 10.6, 4.4, bone, radius=1.0)
    for x in (2.6, 5.6, 8.6):
        a.stroke([(x, 3.4), (x, 12.6)], 1.5, bone)
    a.stroke([(10.0, 3.4), (12.4, 4.6), (14.6, 6.4)], 1.4, bone)
    for x in (2.6, 5.6, 8.6):
        for y in (6.8, 9.8):
            a.stroke([(x - 0.9, y), (x + 0.9, y)], 0.35, "#1a1423")
    a.stroke([(12.0, 3.8), (12.6, 5.4)], 0.35, "#1a1423")
    for x in (3.0, 6.0, 9.0):
        a.stroke([(x, 4.6), (x, 6.2)], 0.4, "#b8ad90", alpha=0.8)
    # cracks
    a.stroke([(2.6, 14.2), (1.6, 15.2), (0.4, 15.6)], 0.4, "#1a1423", smooth=False)
    a.stroke([(13.4, 14.2), (14.2, 15.0), (15.8, 15.4)], 0.4, "#1a1423", smooth=False)
    a.stroke([(5.6, 14.2), (6.0, 15.6)], 0.35, "#1a1423")
    light(a, alpha=0.1)
    a.outline(width=OL)
    glow(a, "#c8a8ff", 1.0, 0.55)
    for x, y, r in ((0.8, 9.4, 0.9), (0.8, 11.6, 0.8), (15.2, 9.4, 0.9), (15.2, 11.6, 0.8), (1.8, 12.8, 0.6), (14.2, 12.8, 0.6)):
        star(a, x, y, r, "#c8a8ff")
    a.save("exotic/skeleton_hand")


ALL = [hat, top, bottom, gloves, shoes, necklace, ring_icon, earring, charm, omamori, arsenal, dust, draw,
       lantern_heart, jeweled_branch, unbroken_yinyang, nine_seal, clone_scroll, hollow_mask, white_haori,
       hunger_organ, thousand_counter, sovereign_finger, closed_hand_vow, kusanagi, yata_mirror, yasakani_jewel,
       raijin_drums, fox_procession, tengu_fan, dragon_coil, moonfall, skeleton_hand]

if __name__ == "__main__":
    for fn in ALL:
        fn()
    print(f"wrote {len(ALL)} icons")
