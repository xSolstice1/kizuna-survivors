"""HD redraw of the 16x16 weapon/passive upgrade icons (art/src/icons/*.px, minus relic_w*).

    python3 tools/hd/icons.py
"""
import math
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from PIL import Image, ImageChops, ImageDraw, ImageFilter  # noqa: E402
from hdkit import SS, Art, _catmull, rgb  # noqa: E402

N = 16
WHITE = "#ffffff"
DONE = []


# ── helpers ──
def new():
    return Art(N, N)


def merge(a, b):
    a.img.alpha_composite(b.img)


def erase_ellipse(a, cx, cy, rx, ry=None):
    ry = rx if ry is None else ry
    m = Image.new("L", (a.W, a.H), 0)
    ImageDraw.Draw(m).ellipse([(cx - rx) * SS, (cy - ry) * SS, (cx + rx) * SS, (cy + ry) * SS], fill=255)
    a.img.putalpha(ImageChops.subtract(a.img.getchannel("A"), m))


def erase_poly(a, pts):
    m = Image.new("L", (a.W, a.H), 0)
    ImageDraw.Draw(m).polygon([(x * SS, y * SS) for x, y in pts], fill=255)
    a.img.putalpha(ImageChops.subtract(a.img.getchannel("A"), m))


def rot_pts(pts, ang, cx=0.0, cy=0.0, ox=0.0, oy=0.0):
    c, s = math.cos(ang), math.sin(ang)
    return [(ox + (x - cx) * c - (y - cy) * s, oy + (x - cx) * s + (y - cy) * c) for x, y in pts]


def ell_pts(cx, cy, rx, ry, ang=0.0, n=40):
    pts = [(rx * math.cos(2 * math.pi * k / n), ry * math.sin(2 * math.pi * k / n)) for k in range(n)]
    return rot_pts(pts, ang, ox=cx, oy=cy)


def inset(pts, k):
    cx = sum(p[0] for p in pts) / len(pts)
    cy = sum(p[1] for p in pts) / len(pts)
    return [(cx + (x - cx) * k, cy + (y - cy) * k) for x, y in pts]


def star_points(cx, cy, r_out, r_in, n=5, rot=-math.pi / 2):
    pts = []
    for i in range(n * 2):
        r = r_out if i % 2 == 0 else r_in
        a = rot + i * math.pi / n
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def sparkle(a, cx, cy, r, fill, clip=False, alpha=1.0):
    a.poly(star_points(cx, cy, r, r * 0.28, n=4), fill, clip=clip, alpha=alpha)


def shine(a, cx, cy, rx, ry, alpha=0.85):
    """Small specular highlight painted only on existing art."""
    a.ellipse(cx, cy, rx, ry, WHITE, clip=True, alpha=alpha)


def glow_layer(draw, color, radius=1.0, alpha=0.7):
    """Draw something on its own layer with a glow, for merging onto an icon."""
    g = new()
    draw(g)
    g.glow(color, radius, alpha)
    return g


_FADE = {}


def _edge_fade(a):
    """Mask that falls to 0 at the canvas border, so glows don't end in a hard square."""
    if (a.W, a.H) not in _FADE:
        small = Image.new("L", (64, 64))
        px = small.load()
        for y in range(64):
            for x in range(64):
                d = min(x, y, 63 - x, 63 - y) / 63 * N  # distance to edge in art px
                px[x, y] = int(255 * max(0.0, min(1.0, (d - 0.15) / 1.6)))
        _FADE[(a.W, a.H)] = small.resize((a.W, a.H), Image.BILINEAR)
    return _FADE[(a.W, a.H)]


def soft_glow(a, color, radius=1.0, alpha=0.6):
    """Like Art.glow, but faded out towards the canvas edges."""
    g = a.img.getchannel("A").filter(ImageFilter.GaussianBlur(radius * SS))
    g = ImageChops.multiply(g.point(lambda v: int(v * alpha)), _edge_fade(a))
    under = Image.new("RGBA", (a.W, a.H), rgb(color))
    under.putalpha(g)
    under.alpha_composite(a.img)
    a.img = under


def fx(a, draw, color, radius=0.7, alpha=0.85, ol=0.45):
    """A glowing extra (sparkle, wisp, droplet) on its own layer, merged after the main outline."""
    g = new()
    draw(g)
    if ol:
        g.outline(width=ol)
    soft_glow(g, color, radius, alpha)
    merge(a, g)


def finish(a, name, glow=None, gr=1.0, ga=0.55, shade=0.16, rim=0.32, fxs=()):
    if shade:
        a.shade_side("#1a0a30", shade, side="right", amount=0.45)
        a.shade_side("#1a0a30", shade * 0.7, side="bottom", amount=0.35)
    if rim:
        a.rim(WHITE, rim, width=0.5)
    a.outline(width=0.8)
    if glow:
        soft_glow(a, glow, gr, ga)
    for f in fxs:
        fx(a, *f)
    a.save(f"icons/{name}")
    DONE.append(f"icons/{name}")


# ── weapons ──
def ofuda():
    a = new()
    a.rect(4, 1.2, 12, 14.8, ("lin", "#fffbf0", "#ead9b6", 0), radius=0.6)
    a.rect(4, 1.2, 12, 2.2, "#e8c070", clip=True, alpha=0.7)
    a.rect(4, 13.8, 12, 14.8, "#e8c070", clip=True, alpha=0.7)
    red = ("lin", "#ff5a4a", "#c01e2e", 90)
    a.stroke([(6, 4), (10, 3.8)], 1.1, red)
    a.stroke([(5.8, 6.6), (10.2, 6.4)], 1.1, red)
    a.stroke([(8, 6.6), (8.1, 9.2)], 1.1, red)
    a.rect(6.9, 8.2, 9.1, 9.6, red, radius=0.3)
    a.stroke([(6, 10.9), (10, 10.7)], 1.0, red)
    a.stroke([(6.2, 12.6), (9.8, 12.5)], 1.0, red)
    a.stroke([(11.2, 2.6), (11.2, 13.4)], 0.5, "#c8b48c", clip=True, alpha=0.6)
    a.stroke([(4.8, 2.6), (4.8, 13.4)], 0.5, WHITE, clip=True, alpha=0.8)
    finish(a, "ofuda", glow="#ffb070", ga=0.45)


def moon_orbs():
    a = new()
    a.ellipse(8, 8, 6.8, 6.8, ("rad", "#fffbe8", "#e3bf62", (0.38, 0.36)))
    a.ellipse(10.6, 10.6, 5.2, 5.2, "#c99e45", clip=True, alpha=0.35)
    a.ellipse(9.6, 9.6, 5.4, 5.4, "#fff2c0", clip=True, alpha=0.2)
    for cx, cy, r in ((9.8, 9.4, 1.3), (6.0, 10.8, 0.9), (10.8, 5.6, 0.8), (7.6, 12.6, 0.6)):
        a.ellipse(cx, cy, r, r * 0.9, "#d9b866", alpha=0.6)
        a.ellipse(cx - 0.25, cy - 0.25, r * 0.7, r * 0.6, "#fff2c8", clip=True, alpha=0.4)
    shine(a, 5.4, 5.2, 1.7, 1.3, 0.9)
    shine(a, 3.9, 7.6, 0.5, 0.7, 0.6)
    finish(a, "moon_orbs", glow="#fff0a0", gr=1.3, ga=0.6, shade=0.08)


def sakura_field():
    a = new()
    petal = [(0, -1.0), (-2.4, -2.6), (-3.0, -4.6), (-2.2, -6.2), (-0.9, -6.9), (0, -6.1),
             (0.9, -6.9), (2.2, -6.2), (3.0, -4.6), (2.4, -2.6)]
    for k in range(5):
        ang = k * 2 * math.pi / 5
        pts = rot_pts(petal, ang, ox=8, oy=8.4)
        a.blob(pts, ("rad", "#fff0f6", "#ff86b8", (0.5, 0.52)))
    for k in range(5):
        ang = -math.pi / 2 + k * 2 * math.pi / 5 + math.pi / 5
        a.stroke([(8, 8.4), (8 + 4.3 * math.cos(ang), 8.4 + 4.3 * math.sin(ang))], 0.45, "#c94d86", clip=True, alpha=0.9)
    for k in range(5):
        ang = -math.pi / 2 + k * 2 * math.pi / 5
        a.stroke([(8, 8.4), (8 + 3.4 * math.cos(ang), 8.4 + 3.4 * math.sin(ang))], 0.5, "#e0609a", clip=True, alpha=0.7)
        sx, sy = 8 + 2.3 * math.cos(ang + 0.35), 8.4 + 2.3 * math.sin(ang + 0.35)
        a.ellipse(sx, sy, 0.4, 0.4, "#ffe066")
    a.ellipse(8, 8.4, 1.6, 1.6, ("rad", "#fff7c0", "#f0b020"))
    shine(a, 5.6, 4.4, 0.8, 0.5, 0.7)
    finish(a, "sakura_field", glow="#ffa0c8", ga=0.45, shade=0.12)


def thunder_chain():
    a = new()
    bolt = inset([(12.5, 0.5), (2, 9.5), (7, 9.5), (3.5, 15.5), (14.5, 6), (9, 6), (14, 0.5)], 0.9)
    a.poly(bolt, ("lin", "#fffbd0", "#ffb000", 90))
    a.poly(inset(bolt, 0.6), "#fffbe8", clip=True, alpha=0.55)
    a.stroke([(11.6, 1.6), (6.2, 6.8)], 0.7, WHITE, clip=True, alpha=0.9)
    a.stroke([(7.0, 10.0), (5.2, 13.0)], 0.5, WHITE, clip=True, alpha=0.7)
    a.shade_side("#c05000", 0.3, side="right", amount=0.3)
    sp = lambda g: (sparkle(g, 2.6, 3.4, 1.3, "#fff6a0"), sparkle(g, 13.4, 12.2, 1.1, "#fff6a0"))
    finish(a, "thunder_chain", glow="#ffd21a", gr=1.4, ga=0.75, shade=0.06, fxs=[(sp, "#ffe14d")])


def taiko():
    a = new()
    stick = ("lin", "#d89a60", "#7a4a2a", 0)
    a.stroke([(1.4, 1.4), (6.2, 6.2)], 1.1, stick)
    a.stroke([(14.6, 1.4), (9.8, 6.2)], 1.1, stick)
    a.ellipse(1.6, 1.6, 0.9, 0.9, "#f0d0a0")
    a.ellipse(14.4, 1.6, 0.9, 0.9, "#f0d0a0")
    d = new()
    d.ellipse(8, 9.4, 6.6, 5.8, ("lin", "#f05050", "#8a1a28", 90))
    d.ellipse(8, 8.7, 5.4, 4.8, "#5a1018")
    d.ellipse(8, 8.6, 5.0, 4.4, ("rad", "#fff6e2", "#d8c08e", (0.42, 0.4)))
    d.ellipse(8, 8.6, 2.6, 2.3, "#c9ae7c", alpha=0.5)
    d.ellipse(7.8, 8.3, 2.2, 1.9, "#fff1d6", alpha=0.7)
    for k in range(10):
        ang = math.pi * 2 * k / 10
        x, y = 8 + 6.0 * math.cos(ang), 9.0 + 5.2 * math.sin(ang)
        if y > 6.0:
            d.ellipse(x, y, 0.42, 0.42, "#ffd27a")
    shine(d, 5.2, 6.6, 1.3, 0.7, 0.7)
    merge(a, d)
    finish(a, "taiko")


def star_fall():
    a = new()
    trail = ("lin", "#e8f8ff", "#5ab8ff", 135)
    a.stroke([(9.4, 6.6), (15.0, 1.0)], 2.2, trail, taper=True)
    a.stroke([(10.8, 8.8), (15.2, 4.6)], 1.3, trail, taper=True)
    a.stroke([(7.2, 5.4), (11.2, 1.2)], 1.3, trail, taper=True)
    s = new()
    pts = star_points(6.8, 9.4, 6.0, 2.8, rot=-math.pi / 2 - 0.15)
    s.poly(pts, ("rad", "#fffbe0", "#ffbe1a", (0.42, 0.55)))
    s.poly(inset(pts, 0.55), "#fff7c8", clip=True, alpha=0.5)
    s.shade_side("#d06a00", 0.25, side="bottom", amount=0.35)
    shine(s, 5.6, 7.6, 1.0, 0.7, 0.95)
    merge(a, s)
    finish(a, "star_fall", glow="#ffe066", gr=1.2, ga=0.6, shade=0.08)


def crescent():
    a = new()
    a.ellipse(7.6, 8.4, 6.9, 6.9, ("lin", "#ffffff", "#9cc8ee", 50))
    erase_ellipse(a, 11.0, 6.0, 6.1)
    a.ellipse(4.6, 10.4, 1.3, 1.1, "#8fb8d9", clip=True, alpha=0.6)
    a.ellipse(6.6, 12.6, 0.8, 0.7, "#8fb8d9", clip=True, alpha=0.5)
    a.ellipse(3.6, 7.0, 0.6, 0.6, "#8fb8d9", clip=True, alpha=0.5)
    a.stroke([(2.2, 6.2), (2.6, 10.4), (5.0, 13.4), (9.0, 14.2)], 0.5, WHITE, clip=True, alpha=0.9)
    finish(a, "crescent", glow="#a8d8ff", gr=1.2, ga=0.6, shade=0.1)


def flame_pool():
    a = new()
    outer = [(8, 0.9), (10.6, 4.2), (13.2, 7.2), (13.8, 10.6), (12.0, 14.0), (8, 15.2), (4.0, 14.0),
             (2.2, 10.6), (3.0, 7.0), (5.0, 4.8), (5.8, 7.6), (6.9, 4.0)]
    a.blob(outer, ("lin", "#ff6a3a", "#c22a2a", 90))
    mid = [(8.4, 4.4), (11.2, 8.4), (11.6, 11.6), (10.0, 14.0), (8, 14.6), (6.0, 14.0), (4.6, 11.6), (5.0, 9.4), (6.6, 10.2)]
    a.blob(mid, ("lin", "#ffb04a", "#ff7a1a", 90))
    inner = [(8.2, 8.0), (10.0, 11.0), (9.6, 13.4), (8, 14.0), (6.4, 13.4), (6.2, 11.4)]
    a.blob(inner, ("rad", "#fffbe0", "#ffd23a", (0.5, 0.75)))
    shine(a, 7.0, 3.4, 0.4, 0.9, 0.5)
    finish(a, "flame_pool", glow="#ff7a2a", gr=1.3, ga=0.7, shade=0.08)


def crimson_tendrils():
    a = new()
    tend = ("lin", "#ff6070", "#7a1020", 90)
    paths = [
        [(6.0, 9.0), (3.4, 6.6), (1.6, 3.6), (2.6, 1.4)],
        [(7.2, 8.6), (5.6, 5.6), (5.6, 2.8), (6.6, 1.2)],
        [(9.2, 8.6), (10.2, 5.6), (10.0, 3.0), (11.4, 1.2)],
        [(10.4, 9.0), (12.8, 6.6), (14.2, 3.6), (13.8, 1.4)],
    ]
    for p in paths:
        a.stroke(p, 1.7, tend, taper=True)
    base = [(4.2, 9.0), (8, 8.2), (11.8, 9.0), (10.8, 11.6), (9.6, 13.4), (8.8, 15.2), (7.4, 14.0), (6.0, 11.8)]
    a.blob(base, ("lin", "#ff5a6a", "#80121f", 90))
    for y, w in ((10.2, 3.2), (12.2, 2.2), (13.9, 1.1)):
        a.stroke([(8 - w, y - 0.5), (8, y + 0.3), (8 + w, y - 0.5)], 0.45, "#5a0814", clip=True, alpha=0.8)
    for p in paths:
        a.stroke(p[1:], 0.35, "#ffb0b8", clip=True, alpha=0.55)
    shine(a, 6.0, 9.4, 0.9, 0.5, 0.6)
    finish(a, "crimson_tendrils", glow="#ff3048", gr=1.0, ga=0.5)


def devour():
    a = new()
    a.ellipse(8, 8, 7.0, 7.0, ("rad", "#c8283a", "#5a0812", (0.4, 0.35)))
    a.ellipse(8, 8.2, 4.8, 4.4, "#2a0306")
    a.ellipse(8, 8.6, 4.2, 3.6, ("rad", "#8a1020", "#2a0306", (0.5, 0.7)))
    tooth = ("lin", "#ffffff", "#d8ccb6", 90)
    for x in (5.0, 8.0, 11.0):
        a.poly([(x - 1.3, 3.9), (x + 1.3, 3.9), (x, 7.0)], tooth, clip=True)
    for x in (6.5, 9.5):
        a.poly([(x - 1.2, 12.5), (x + 1.2, 12.5), (x, 9.5)], ("lin", "#d8ccb6", "#ffffff", 90), clip=True)
    shine(a, 4.6, 3.8, 1.4, 0.8, 0.6)
    finish(a, "devour", glow="#ff2040", gr=0.9, ga=0.35)


def familiars():
    a = new()
    fur = ("lin", "#ffb850", "#e8661a", 90)
    a.poly([(1.6, 8.0), (2.2, 1.0), (6.6, 3.8)], fur)
    a.poly([(14.4, 8.0), (13.8, 1.0), (9.4, 3.8)], fur)
    a.poly([(2.6, 6.0), (2.9, 2.6), (5.2, 4.2)], "#c04a10")
    a.poly([(13.4, 6.0), (13.1, 2.6), (10.8, 4.2)], "#c04a10")
    head = [(1.6, 5.4), (8, 3.4), (14.4, 5.4), (14.2, 9.0), (11.8, 12.4), (8, 15.0), (4.2, 12.4), (1.8, 9.0)]
    a.blob(head, fur)
    a.blob([(2.6, 9.6), (5.4, 9.8), (8, 10.6), (10.6, 9.8), (13.4, 9.6), (11.0, 12.6), (8, 14.8), (5.0, 12.6)],
           ("lin", "#fffaf0", "#f2dcb4", 90), clip=True)
    for x in (5.4, 10.6):
        a.ellipse(x, 7.9, 0.8, 1.0, "#2a0a00")
        a.ellipse(x - 0.25, 7.55, 0.3, 0.3, WHITE)
        a.ellipse(x + (-1.2 if x < 8 else 1.2), 9.8, 0.9, 0.5, "#ff7a8a", alpha=0.5)
    a.ellipse(8, 11.0, 0.7, 0.5, "#2a0a00")
    a.stroke([(7.0, 12.0), (8, 12.5), (9.0, 12.0)], 0.35, "#5a2a10")
    shine(a, 5.0, 4.8, 1.2, 0.5, 0.5)
    finish(a, "familiars")


def _fox_head(a, dx, fill, alpha=1.0):
    a.poly([(6.0 + dx, 6.6), (6.6 + dx, 1.8), (9.8 + dx, 4.6)], fill, alpha=alpha)
    a.poly([(14.6 + dx, 6.6), (14.0 + dx, 1.8), (10.8 + dx, 4.6)], fill, alpha=alpha)
    a.blob([(5.8 + dx, 6.0), (10.3 + dx, 4.2), (14.8 + dx, 6.0), (14.6 + dx, 9.4), (12.6 + dx, 12.4),
            (10.3 + dx, 13.4), (8.0 + dx, 12.4), (6.0 + dx, 9.4)], fill, alpha=alpha)


def fox_mirage():
    a = new()
    ghost = new()
    _fox_head(ghost, -4.6, ("lin", "#5a7ad0", "#3a5ab8", 0))
    _fox_head(ghost, -2.4, ("lin", "#8aa8f8", "#b8ccff", 0))
    for y in (5.0, 8.2, 11.0):
        ghost.stroke([(0.6, y), (3.4, y - 0.3)], 0.6, "#e8f0ff", alpha=0.6, clip=True)
    merge(a, ghost)
    sep = new()
    _fox_head(sep, 0, "#1a1423")
    sep.outline(width=0.5)
    merge(a, sep)
    _fox_head(a, 0, ("lin", "#ffffff", "#e2dcef", 90))
    a.blob([(7.4, 11.0), (10.3, 10.6), (13.2, 11.0), (10.3, 13.4)], "#b8b0c8", clip=True, alpha=0.7)
    a.poly([(7.0, 5.6), (7.2, 3.0), (8.8, 4.6)], "#e0263c", alpha=0.85)
    a.poly([(13.6, 5.6), (13.4, 3.0), (11.8, 4.6)], "#e0263c", alpha=0.85)
    a.stroke([(7.6, 8.4), (8.6, 8.9), (9.2, 8.6)], 0.55, "#e0263c")
    a.stroke([(13.0, 8.4), (12.0, 8.9), (11.4, 8.6)], 0.55, "#e0263c")
    a.stroke([(10.3, 5.4), (10.3, 6.8)], 0.5, "#e0263c")
    a.ellipse(10.3, 11.4, 0.5, 0.35, "#2a1423")
    shine(a, 9.0, 5.6, 0.9, 0.4, 0.8)
    finish(a, "fox_mirage", glow="#7a9aff", gr=1.1, ga=0.55, shade=0.1)


def foxfire_orb():
    a = new()
    drop = [(8, 0.9), (9.8, 3.6), (12.8, 6.6), (14.0, 10.0), (12.6, 13.4), (8, 15.0), (3.4, 13.4), (2.0, 10.0),
            (3.2, 6.6), (6.2, 3.6)]
    a.blob(drop, ("rad", "#c8f4ff", "#1a4ec0", (0.45, 0.62)))
    a.blob([(8, 4.4), (11.0, 8.4), (11.4, 11.4), (8, 13.6), (4.6, 11.4), (5.0, 8.4)], "#3a9cff", clip=True, alpha=0.35)
    a.ellipse(7.2, 9.6, 3.0, 3.0, ("rad", "#ffffff", "#8fe0ff"), clip=True, alpha=0.75)
    shine(a, 6.0, 8.4, 1.0, 1.0, 0.95)
    shine(a, 7.2, 3.8, 0.35, 0.9, 0.6)
    sp = lambda g: (g.ellipse(14.6, 13.2, 0.6, 0.6, "#bff0ff"), g.ellipse(1.6, 13.6, 0.5, 0.5, "#bff0ff"),
                    sparkle(g, 13.4, 2.8, 1.1, "#e0f8ff"))
    finish(a, "foxfire_orb", glow="#3a9cff", gr=1.4, ga=0.75, shade=0.12, fxs=[(sp, "#3a9cff")])


def gale_shuriken():
    a = new()
    pts = star_points(8, 8, 7.0, 2.1, n=4)
    a.poly(pts, ("lin", "#f4f8ff", "#646c84", 135))
    for k in range(4):
        ang = -math.pi / 2 + k * math.pi / 2
        a.stroke([(8, 8), (8 + 6.4 * math.cos(ang), 8 + 6.4 * math.sin(ang))], 0.35,
                 WHITE if k in (0, 1) else "#4a5068", clip=True, alpha=0.7)
    a.ellipse(8, 8, 2.0, 2.0, ("rad", "#ff6070", "#a01828"))
    a.ellipse(8, 8, 0.75, 0.75, "#1a1423")
    shine(a, 7.3, 7.3, 0.4, 0.3, 0.9)
    shine(a, 8.4, 2.6, 0.3, 0.8, 0.9)
    w = new()
    w.stroke([(2.2, 4.4), (3.0, 3.0), (4.4, 2.2)], 0.45, "#dff4ff", taper=True, alpha=0.6)
    w.stroke([(13.8, 11.6), (13.0, 13.0), (11.6, 13.8)], 0.45, "#dff4ff", taper=True, alpha=0.6)
    merge(a, w)
    finish(a, "gale_shuriken", glow="#b8e0ff", gr=0.9, ga=0.4)


def grave_cleaver():
    a = new()
    u = (1 / math.sqrt(2), -1 / math.sqrt(2))
    n = (1 / math.sqrt(2), 1 / math.sqrt(2))
    B, T = (5.2, 10.8), (14.6, 1.4)

    def P(p, du, dn):
        return (p[0] + u[0] * du + n[0] * dn, p[1] + u[1] * du + n[1] * dn)

    blade = [P(B, 0, 1.5), P(T, -2.6, 1.7), P(T, 0, 0), P(T, -1.0, -1.0), P(B, 0, -1.2)]
    a.poly(blade, ("lin", "#ffffff", "#b4ac9c", 45))
    a.poly([P(B, 0, 1.5), P(T, -2.6, 1.7), P(T, -1.6, 0.5), P(B, 0.4, 0.4)], "#a8a090", clip=True, alpha=0.55)
    a.stroke([P(B, 0.6, -0.4), P(T, -2.4, -0.4)], 0.35, WHITE, clip=True, alpha=0.9)
    a.stroke([P(B, -1.9, 0), P(B, -4.6, 0)], 1.25, ("lin", "#e04050", "#8a1a28", 45))
    for t in (-2.5, -3.5):
        a.stroke([P(B, t, -0.6), P(B, t + 0.4, 0.6)], 0.3, "#ffb0a0", clip=True, alpha=0.6)
    a.stroke([P(B, -0.6, 2.6), P(B, -0.6, -2.6)], 1.15, ("lin", "#ffe08a", "#a87a20", 90))
    a.ellipse(*P(B, -5.0, 0), 0.85, 0.85, ("rad", "#ffe08a", "#a87a20"))
    sp = lambda g: (sparkle(g, 2.8, 3.6, 1.6, "#d8c0ff"), sparkle(g, 6.2, 1.8, 1.0, "#d8c0ff"),
                    sparkle(g, 1.8, 7.2, 1.0, "#d8c0ff"))
    finish(a, "grave_cleaver", glow="#b08aff", gr=0.8, ga=0.35, shade=0.14, fxs=[(sp, "#9a6aff")])


def gravity_well():
    a = new()
    a.ellipse(8, 8, 7.0, 7.0, ("rad", "#5a7ae8", "#1e3488", (0.38, 0.35)))
    a.ellipse(8, 8, 5.3, 5.3, ("rad", "#1a2450", "#070b1c", (0.5, 0.5)))
    for k in range(3):
        base = k * 2 * math.pi / 3
        pts = []
        for s in range(14):
            t = s / 13
            r = 5.2 - 4.4 * t
            ang = base + t * 1.6 * math.pi
            pts.append((8 + r * math.cos(ang), 8 + r * math.sin(ang)))
        a.stroke(pts, 1.0, "#5a8aff", clip=True, taper=True, alpha=0.9)
    a.ellipse(8, 8, 1.6, 1.6, ("rad", "#ffffff", "#7aa8ff"))
    shine(a, 4.6, 3.6, 1.4, 0.6, 0.55)
    finish(a, "gravity_well", glow="#4a6aff", gr=1.2, ga=0.6, shade=0.1)


def iai_flash():
    a = new()
    a.blob([(0.6, 6.2), (8, 5.1), (15.4, 6.2), (8, 7.3)], ("lin", "#fff0a0", "#ffffff", 0))
    a.stroke([(1.3, 14.7), (6.0, 10.0)], 1.6, ("lin", "#4a3e5c", "#2a2433", 90))
    for t in (0.2, 0.45, 0.7):
        x, y = 1.3 + 4.7 * t, 14.7 - 4.7 * t
        a.poly([(x, y - 0.6), (x + 0.6, y), (x, y + 0.6), (x - 0.6, y)], "#8a7aa0", alpha=0.9)
    a.ellipse(6.6, 9.4, 1.35, 1.35, ("rad", "#ff6070", "#9a1828"))
    a.poly([(7.0, 9.8), (6.2, 9.0), (13.6, 1.2), (15.0, 1.0), (14.8, 2.4)], ("lin", "#ffffff", "#c8c0b0", 45))
    a.stroke([(7.4, 8.4), (13.8, 1.8)], 0.3, "#9a9488", clip=True, alpha=0.7)
    finish(a, "iai_flash", glow="#ffe880", gr=1.0, ga=0.6, shade=0.1)


def lantern_vigil():
    a = new()
    cap = ("lin", "#4a4058", "#221c2c", 90)
    a.rect(5.0, 0.9, 11.0, 3.0, cap, radius=0.4)
    a.rect(5.0, 13.0, 11.0, 15.1, cap, radius=0.4)
    body = new()
    body.ellipse(8, 8, 6.2, 5.6, ("rad", "#fffaf0", "#c8bea6", (0.5, 0.5)))
    for y, c in ((4.6, 1.0), (11.4, 1.0)):
        body.stroke([(2.4, y + c), (8, y), (13.6, y + c)] if y < 8 else [(2.4, y - c), (8, y), (13.6, y - c)],
                    0.45, "#a89e88", clip=True)
    body.stroke([(1.6, 8.0), (14.4, 8.0)], 0.35, "#b8b0a0", clip=True, alpha=0.6)
    core = glow_layer(lambda g: g.ellipse(8, 8, 2.5, 2.5, ("rad", "#e8f8ff", "#2a88ff")), "#3aa0ff", 1.8, 1.0)
    merge(body, core)
    merge(a, body)
    a.ellipse(7.4, 7.4, 0.55, 0.55, WHITE)
    a.rect(5.4, 1.2, 10.6, 1.6, "#8a7aa0", clip=True, alpha=0.6)
    a.rect(5.4, 13.3, 10.6, 13.7, "#8a7aa0", clip=True, alpha=0.6)
    shine(a, 4.4, 5.6, 0.8, 1.2, 0.5)
    finish(a, "lantern_vigil", glow="#8fd0ff", gr=1.2, ga=0.45, shade=0.12)


def mukade_spines():
    a = new()
    a.rect(0, 13.7, 16, 16, ("lin", "#4a3e52", "#2a2433", 90))
    a.ellipse(3.0, 14.4, 2.4, 0.6, "#5a4e62", clip=True, alpha=0.8)
    a.ellipse(12.0, 14.4, 2.0, 0.5, "#5a4e62", clip=True, alpha=0.8)
    ctrl = [(2.6, 13.8), (3.0, 9.6), (4.6, 5.8), (7.4, 3.6), (10.2, 4.6), (11.6, 7.6), (12.0, 10.8)]
    path = _catmull(ctrl, False, steps=6)
    idx = [int(i * (len(path) - 1) / 8) for i in range(9)]
    seg = [path[i] for i in idx]
    body = new()
    for i, (x, y) in enumerate(seg):
        r = 1.75 - 0.03 * i
        j = min(i + 1, len(seg) - 1)
        k = max(i - 1, 0)
        tx, ty = seg[j][0] - seg[k][0], seg[j][1] - seg[k][1]
        L = math.hypot(tx, ty) or 1
        nx, ny = -ty / L, tx / L
        if ny > 0:
            nx, ny = -nx, -ny
        body.stroke([(x, y), (x + nx * 2.6, y + ny * 2.6)], 0.7, "#f0e6d6", taper=True)
        body.stroke([(x, y), (x - nx * 2.4, y - ny * 2.4)], 0.55, "#c8bca8", taper=True)
        body.ellipse(x, y, r + 0.35, r + 0.35, "#2a0a12")
        body.ellipse(x, y, r, r, ("rad", "#ff5a66", "#6a0e1a", (0.4, 0.35)))
    hx, hy = seg[-1]
    body.stroke([(hx - 0.6, hy + 0.8), (hx - 1.4, hy + 2.2)], 0.5, "#f0e6d6", taper=True)
    body.stroke([(hx + 0.6, hy + 0.8), (hx + 1.4, hy + 2.2)], 0.5, "#f0e6d6", taper=True)
    body.ellipse(hx + 0.5, hy - 0.3, 0.45, 0.45, "#ffe066")
    erase_poly(body, [(0, 13.7), (16, 13.7), (16, 16), (0, 16)])
    merge(a, body)
    a.stroke([(12.6, 3.4), (14.9, 1.1)], 1.0, ("lin", "#fff8ec", "#c8bca8", 0), taper=False)
    a.poly([(14.1, 1.0), (15.2, 0.8), (15.0, 1.9)], "#fff8ec")
    shine(a, 6.6, 3.0, 0.7, 0.4, 0.6)
    finish(a, "mukade_spines")


def purification_wave():
    a = new()
    a.ellipse(8, 8, 7.0, 7.0, ("rad", "#ffe2ee", "#ff96bc", (0.5, 0.5)))
    a.ellipse(8, 8, 5.2, 5.2, "#3a1c32")
    a.ellipse(8, 8, 4.3, 4.3, ("rad", "#ffffff", "#ffe0ec", (0.4, 0.4)))
    a.ellipse(8, 8, 3.3, 3.3, "#3a1c32")
    a.ellipse(8, 8, 2.6, 2.6, "#2a1428", alpha=1.0)
    rc = glow_layer(lambda g: g.ellipse(8, 8, 1.35, 1.35, ("rad", "#ff9a8a", "#d81e2e")), "#ff3a4a", 0.8, 0.9)
    merge(a, rc)
    for k in range(4):
        ang = k * math.pi / 2
        x, y = 8 + 6.1 * math.cos(ang), 8 + 6.1 * math.sin(ang)
        a.ellipse(x, y, 0.55, 0.55, "#c94d86")
        a.ellipse(x - 0.12, y - 0.12, 0.25, 0.25, WHITE)
    shine(a, 4.2, 3.6, 1.2, 0.6, 0.6)
    finish(a, "purification_wave", glow="#ffa0c8", gr=1.2, ga=0.55, shade=0.1)


def repulsion_star():
    a = new()
    a.poly(star_points(8, 8, 4.6, 1.6, n=4, rot=-math.pi / 4), ("lin", "#d02a3a", "#7a1020", 90))
    a.poly(star_points(8, 8, 7.0, 1.9, n=4), ("lin", "#ff8090", "#d01e30", 90))
    for k in range(4):
        ang = -math.pi / 2 + k * math.pi / 2
        a.stroke([(8, 8), (8 + 6.2 * math.cos(ang), 8 + 6.2 * math.sin(ang))], 0.35, "#ffd0d6", clip=True, alpha=0.7)
    c = glow_layer(lambda g: g.ellipse(8, 8, 1.9, 1.9, ("rad", "#ffffff", "#ffc0c8")), "#ffffff", 0.8, 0.8)
    merge(a, c)
    finish(a, "repulsion_star", glow="#ff3a4a", gr=1.2, ga=0.65, shade=0.1)


def soul_tether():
    a = new()
    centres = [(2.6, 13.4), (4.6, 11.4), (6.6, 9.4), (8.6, 7.4), (10.4, 5.6)]
    ang = -math.pi / 4
    for i, (cx, cy) in enumerate(centres):
        l = new()
        if i % 2 == 0:
            l.blob(ell_pts(cx, cy, 1.9, 1.25, ang), ("lin", "#d4b8ff", "#6a4ac8", 45))
            erase_poly(l, ell_pts(cx, cy, 0.95, 0.4, ang))
        else:
            l.blob(ell_pts(cx, cy, 1.9, 0.75, ang), ("lin", "#a07aff", "#4a2a9a", 45))
        l.outline(width=0.45)
        merge(a, l)
    a.stroke([(10.8, 5.2), (12.8, 3.6)], 0.6, "#e8e0ff")
    wisp = [(12.2, 4.2), (12.6, 2.4), (13.8, 1.0), (13.8, 2.2), (15.0, 1.6), (14.8, 3.6), (13.8, 4.8)]
    w = lambda g: g.blob(wisp, ("rad", "#ffffff", "#c8b0ff", (0.4, 0.6)))
    finish(a, "soul_tether", glow="#9a6aff", gr=1.0, ga=0.5, shade=0.08, fxs=[(w, "#c8a8ff", 0.9, 0.95)])


def spirit_seal():
    a = new()
    petal = [(0, -1.4), (-2.6, -2.4), (-3.1, -4.4), (-2.3, -6.2), (-1.0, -6.9), (0, -6.2),
             (1.0, -6.9), (2.3, -6.2), (3.1, -4.4), (2.6, -2.4)]
    for k in range(4):
        ang = k * math.pi / 2
        pts = rot_pts(petal, ang, ox=8, oy=8)
        a.blob(pts, ("lin", "#e0609a", "#ffb4d2", (ang * 180 / math.pi) % 360))
    for k in range(4):
        ang = k * math.pi / 2 + math.pi / 4
        a.stroke([(8, 8), (8 + 4.0 * math.cos(ang), 8 + 4.0 * math.sin(ang))], 0.5, "#9a2a5a", clip=True, alpha=0.8)
    a.rect(5.5, 5.5, 10.5, 10.5, "#3a1428", radius=0.8)
    c = new()
    c.rect(5.9, 5.9, 10.1, 10.1, ("rad", "#fff6c0", "#f0b020"), radius=0.6)
    c.stroke([(7.0, 7.2), (9.0, 7.2)], 0.4, "#c0601a", alpha=0.8)
    c.stroke([(8.0, 7.2), (8.0, 9.2)], 0.4, "#c0601a", alpha=0.8)
    c.stroke([(7.0, 9.0), (9.0, 9.0)], 0.4, "#c0601a", alpha=0.8)
    c.glow("#ffe066", 0.9, 0.9)
    merge(a, c)
    shine(a, 6.6, 6.6, 0.4, 0.4, 0.9)
    shine(a, 5.6, 2.6, 0.8, 0.5, 0.6)
    finish(a, "spirit_seal", glow="#ff9ec4", gr=1.1, ga=0.5, shade=0.1)


def venom_fang():
    a = new()
    outer = _catmull([(2.0, 1.2), (6.2, 1.3), (10.0, 2.8), (12.6, 5.8), (13.4, 9.0), (12.4, 12.0), (9.8, 14.0)], False)
    inner = _catmull([(9.8, 14.0), (9.8, 11.2), (8.8, 8.4), (7.0, 6.0), (4.6, 4.4), (2.6, 3.6)], False)
    fang = outer + inner
    a.poly(fang, ("lin", "#fffbf0", "#c8bea6", 0))
    a.stroke([(5.0, 3.0), (8.8, 4.4), (10.8, 7.0), (11.4, 9.8), (10.6, 12.4)], 0.5, "#a89e88", clip=True, alpha=0.7)
    a.stroke([(4.0, 2.0), (7.6, 2.6), (10.0, 4.4)], 0.45, WHITE, clip=True, alpha=0.9)
    a.ellipse(11.2, 12.6, 2.2, 2.0, ("lin", "#a8ff7a", "#3a8a2a", 90), clip=True, alpha=0.85)
    a.rect(1.6, 0.6, 3.4, 3.6, "#a8a090", clip=True, alpha=0.5)
    d = lambda g: g.blob([(8.6, 13.2), (9.3, 14.4), (8.6, 15.2), (7.9, 14.4)], ("lin", "#d8ffb0", "#3a8a2a", 90))
    finish(a, "venom_fang", glow="#7ae05a", gr=0.7, ga=0.3, shade=0.12, fxs=[(d, "#7ae05a")])


def void_lance():
    a = new()
    a.ellipse(8, 8, 6.9, 6.9, ("rad", "#c890ff", "#5a22a8", (0.4, 0.35)))
    a.ellipse(8, 8, 5.2, 5.2, "#1a0a30")
    a.ellipse(8.3, 8.3, 4.8, 4.8, ("rad", "#2a1050", "#05020f", (0.55, 0.55)))
    a.stroke([(5.0, 10.6), (6.8, 12.0), (9.6, 11.8)], 0.5, "#6a3ac0", clip=True, alpha=0.6)
    a.stroke([(10.6, 5.0), (11.8, 7.0), (11.6, 9.4)], 0.45, "#6a3ac0", clip=True, alpha=0.5)
    a.ellipse(6.4, 6.4, 1.5, 1.5, ("rad", "#ffffff", "#c8a8ff"))
    shine(a, 4.2, 3.6, 1.2, 0.6, 0.6)
    sp = lambda g: (sparkle(g, 14.0, 13.6, 1.4, "#e8d8ff"), sparkle(g, 1.9, 13.4, 1.0, "#e8d8ff"),
                    sparkle(g, 13.8, 2.1, 1.0, "#e8d8ff"))
    finish(a, "void_lance", glow="#9a4aff", gr=1.3, ga=0.7, shade=0.1, fxs=[(sp, "#9a4aff")])


def yinyang_orbs():
    a = new()
    a.ellipse(8, 8, 7.0, 7.0, ("rad", "#ffffff", "#e4dcea", (0.35, 0.4)))
    dark = ("rad", "#4a4266", "#1e1a2e", (0.7, 0.6))
    a.rect(8, 0, 16, 16, dark, clip=True)
    a.ellipse(8, 4.5, 3.5, 3.5, ("rad", "#ffffff", "#ece6f0", (0.4, 0.4)), clip=True)
    a.ellipse(8, 11.5, 3.5, 3.5, dark, clip=True)
    a.ellipse(8, 4.5, 1.15, 1.15, "#2a2440")
    a.ellipse(8, 11.5, 1.15, 1.15, "#ffffff")
    shine(a, 4.4, 4.0, 1.3, 0.7, 0.7)
    a.ellipse(12.2, 10.4, 0.9, 1.6, "#8a80aa", clip=True, alpha=0.35)
    finish(a, "yinyang_orbs", glow="#ffb0b8", gr=1.0, ga=0.45, shade=0.06)


# ── passives ──
def might():
    a = new()
    arrow = [(8, 1.0), (14.8, 8.0), (11, 8.0), (11, 14.9), (5, 14.9), (5, 8.0), (1.2, 8.0)]
    a.poly(arrow, ("lin", "#e84a50", "#9a1e2e", 90))
    a.poly([(8, 2.6), (12.6, 7.2), (10.2, 7.2), (10.2, 14.0), (5.8, 14.0), (5.8, 7.2), (3.4, 7.2)],
           ("lin", "#ffa090", "#ff4a50", 90))
    a.rect(5.8, 8.0, 7.0, 14.0, "#b02e3a", clip=True, alpha=0.55)
    a.stroke([(3.6, 6.8), (8, 2.4)], 0.45, WHITE, clip=True, alpha=0.9)
    a.stroke([(7.4, 8.4), (7.4, 13.0)], 0.4, WHITE, clip=True, alpha=0.4)
    finish(a, "might", shade=0.12)


def haste():
    a = new()
    wood = ("lin", "#c08a5a", "#6a4628", 90)
    a.stroke([(3.8, 2.4), (3.8, 13.6)], 0.7, "#7a5434")
    a.stroke([(12.2, 2.4), (12.2, 13.6)], 0.7, "#7a5434")
    glass = [(4.4, 2.8), (11.6, 2.8), (11.2, 5.4), (8.7, 8.0), (11.2, 10.6), (11.6, 13.2), (4.4, 13.2),
             (4.8, 10.6), (7.3, 8.0), (4.8, 5.4)]
    a.blob(glass, ("lin", "#eaf6ff", "#a8cce8", 0))
    a.blob([(5.6, 4.2), (10.4, 4.2), (9.4, 5.8), (8, 7.4), (6.6, 5.8)], ("lin", "#ffe78a", "#e8a830", 90), clip=True)
    a.stroke([(8, 7.2), (8, 11.6)], 0.4, "#ffd96a")
    a.blob([(5.0, 13.2), (6.6, 11.6), (8, 10.8), (9.4, 11.6), (11.0, 13.2)], ("lin", "#ffe78a", "#e0a020", 90), clip=True)
    a.stroke([(5.6, 3.6), (5.4, 5.6), (7.0, 7.4)], 0.4, WHITE, clip=True, alpha=0.85)
    a.stroke([(7.0, 8.8), (5.6, 10.4), (5.4, 12.4)], 0.4, WHITE, clip=True, alpha=0.7)
    a.rect(2.6, 1.0, 13.4, 2.9, wood, radius=0.6)
    a.rect(2.6, 13.1, 13.4, 15.0, wood, radius=0.6)
    a.rect(2.9, 1.2, 13.1, 1.6, "#e8b884", clip=True, alpha=0.7)
    a.rect(2.9, 13.3, 13.1, 13.7, "#e8b884", clip=True, alpha=0.7)
    finish(a, "haste", shade=0.12)


def expanse():
    a = new()
    a.ellipse(8, 8, 7.0, 7.0, ("lin", "#b4f8d0", "#36b070", 90))
    erase_ellipse(a, 8, 8, 5.2)
    a.ellipse(8, 8, 3.7, 3.7, ("lin", "#5ad08e", "#1f7a50", 90))
    erase_ellipse(a, 8, 8, 1.8)
    for k in range(4):
        ang = math.pi / 4 + k * math.pi / 2
        x, y = 8 + 4.45 * math.cos(ang), 8 + 4.45 * math.sin(ang)
        a.ellipse(x, y, 0.42, 0.42, "#c8ffe0", alpha=0.9)
    a.stroke([(2.4, 6.4), (3.6, 3.6), (6.4, 2.0)], 0.45, WHITE, clip=True, alpha=0.9)
    a.stroke([(5.4, 7.0), (6.6, 5.2)], 0.35, WHITE, clip=True, alpha=0.6)
    finish(a, "expanse", glow="#6be0a0", gr=1.0, ga=0.4, shade=0.1)


def swiftness():
    a = new()
    back = [(1.0, 2.2), (4.8, 2.2), (9.4, 8.0), (4.8, 13.8), (1.0, 13.8), (5.4, 8.0)]
    a.poly(back, ("lin", "#5ac8ee", "#1e88b8", 90))
    sep = new()
    front = [(x + 5.4, y) for x, y in back]
    sep.poly(front, "#1a1423")
    sep.outline(width=0.55)
    merge(a, sep)
    a.poly(front, ("lin", "#c8f8ff", "#5ad8ff", 90))
    a.stroke([(7.0, 2.8), (10.6, 7.4)], 0.45, WHITE, clip=True, alpha=0.9)
    a.stroke([(1.6, 2.8), (5.2, 7.4)], 0.4, WHITE, clip=True, alpha=0.5)
    finish(a, "swiftness", glow="#7ee8ff", gr=0.9, ga=0.35, shade=0.12)


def magnetism():
    a = new()
    path = [(3.3, 14.6), (3.3, 7.6)]
    for s in range(1, 24):
        ang = math.pi + math.pi * s / 24
        path.append((8 + 4.7 * math.cos(ang), 7.6 + 4.7 * math.sin(ang)))
    path += [(12.7, 7.6), (12.7, 14.6)]
    a.stroke(path, 3.2, ("lin", "#ff7a70", "#b8283a", 90), smooth=False)
    a.rect(0, 11.2, 16, 16, ("lin", "#f4f6fc", "#a8aec0", 0), clip=True)
    a.rect(0, 11.0, 16, 11.4, "#5a2030", clip=True, alpha=0.6)
    arc = [(8 + 5.6 * math.cos(math.pi + math.pi * s / 12), 7.6 + 5.6 * math.sin(math.pi + math.pi * s / 12)) for s in range(1, 8)]
    a.stroke(arc, 0.5, WHITE, clip=True, alpha=0.8, smooth=False)
    a.stroke([(2.4, 8.0), (2.4, 10.6)], 0.45, WHITE, clip=True, alpha=0.6)
    a.stroke([(2.4, 12.0), (2.4, 14.0)], 0.45, WHITE, clip=True, alpha=0.9)
    sp = lambda g: sparkle(g, 8, 13.0, 1.3, "#d8f2ff")
    finish(a, "magnetism", shade=0.14, fxs=[(sp, "#8fd3ff")])


def vitality():
    a = new()
    raw = []
    for k in range(64):
        t = 2 * math.pi * k / 64
        raw.append((16 * math.sin(t) ** 3, -(13 * math.cos(t) - 5 * math.cos(2 * t) - 2 * math.cos(3 * t) - math.cos(4 * t))))
    xs, ys = [p[0] for p in raw], [p[1] for p in raw]
    x0, x1, y0, y1 = min(xs), max(xs), min(ys), max(ys)
    heart = [(1.2 + (x - x0) / (x1 - x0) * 13.6, 2.0 + (y - y0) / (y1 - y0) * 12.8) for x, y in raw]
    a.poly(heart, ("rad", "#ff9aac", "#d82848", (0.4, 0.35)))
    a.poly(inset(heart, 0.7), "#ff6a84", clip=True, alpha=0.25)
    shine(a, 4.6, 4.8, 1.5, 1.0, 0.85)
    shine(a, 3.0, 7.4, 0.4, 0.6, 0.5)
    finish(a, "vitality", glow="#ff4d6d", gr=0.9, ga=0.35, shade=0.12)


def abundance():
    a = new()
    for cx, cy in ((8, 4.6), (4.6, 11.4), (11.4, 11.4)):
        a.ellipse(cx, cy, 3.75, 3.75, "#5a3a10")
        a.ellipse(cx, cy, 3.3, 3.3, ("rad", "#fff4a8", "#e09a18", (0.4, 0.38)))
        a.ellipse(cx, cy, 2.2, 2.2, "#c88410", alpha=0.45)
        a.ellipse(cx - 0.15, cy - 0.15, 1.9, 1.9, ("rad", "#fff0a0", "#f0b830"))
        a.ellipse(cx - 1.3, cy - 1.4, 0.7, 0.5, WHITE, alpha=0.9)
    sp = lambda g: (sparkle(g, 13.4, 3.2, 1.5, "#fffbe0"), sparkle(g, 2.6, 3.8, 1.0, "#fffbe0"))
    finish(a, "abundance", glow="#ffd23a", gr=0.9, ga=0.4, shade=0.1, fxs=[(sp, "#ffe066")])


def persistence():
    a = new()
    a.ellipse(8, 8, 7.0, 7.0, ("lin", "#e0c8ff", "#8a5ad0", 90))
    a.ellipse(8, 8, 5.4, 5.4, "#4a2a8a")
    a.ellipse(8, 8, 5.0, 5.0, ("rad", "#ffffff", "#e6dcf8", (0.4, 0.4)))
    for k in range(12):
        ang = k * math.pi / 6
        r0 = 3.7 if k % 3 == 0 else 4.1
        a.stroke([(8 + r0 * math.cos(ang), 8 + r0 * math.sin(ang)), (8 + 4.5 * math.cos(ang), 8 + 4.5 * math.sin(ang))],
                 0.45 if k % 3 == 0 else 0.3, "#9a7ad0")
    a.stroke([(8, 8), (8, 4.0)], 0.9, "#6a3ab8")
    a.stroke([(8, 8), (10.9, 9.2)], 0.9, "#6a3ab8")
    a.ellipse(8, 8, 0.8, 0.8, "#c94d86")
    shine(a, 4.0, 3.4, 1.2, 0.6, 0.7)
    shine(a, 5.6, 5.4, 1.2, 0.8, 0.6)
    finish(a, "persistence", shade=0.12)


def recovery():
    a = new()
    a.rect(5.6, 1.4, 10.4, 14.6, "#1f8a40", radius=1.0)
    a.rect(1.4, 5.6, 14.6, 10.4, "#1f8a40", radius=1.0)
    lt = ("lin", "#b4ffc4", "#3ccf64", 90)
    a.rect(6.4, 2.2, 9.6, 13.8, lt, radius=0.7)
    a.rect(2.2, 6.4, 13.8, 9.6, lt, radius=0.7)
    a.stroke([(6.9, 3.0), (6.9, 6.0)], 0.45, WHITE, alpha=0.9)
    a.stroke([(3.0, 6.9), (6.2, 6.9)], 0.45, WHITE, alpha=0.9)
    finish(a, "recovery", glow="#5be07a", gr=1.0, ga=0.45, shade=0.12)


def guard():
    a = new()
    sh = [(2.0, 2.4), (5.0, 1.8), (8, 1.3), (11.0, 1.8), (14.0, 2.4), (14.0, 6.0), (13.6, 8.6), (12.2, 11.0),
          (10.2, 13.2), (8, 14.8), (5.8, 13.2), (3.8, 11.0), (2.4, 8.6), (2.0, 6.0)]
    a.poly(sh, ("lin", "#5a74a8", "#34487a", 90))
    inner = inset(sh, 0.8)
    a.poly(inner, ("lin", "#c4daf8", "#8eaee0", 90))
    r = new()
    r.poly(inner, "#000000")
    r.poly([(8, 0), (16, 0), (16, 16), (8, 16)], ("lin", "#6e8ecc", "#4a64a0", 90), clip=True)
    erase_poly(r, [(0, 0), (8, 0), (8, 16), (0, 16)])
    merge(a, r)
    a.stroke([(8, 2.2), (8, 13.6)], 0.35, "#e8f0ff", clip=True, alpha=0.6)
    a.stroke([(4.4, 3.6), (4.4, 8.2)], 0.6, "#f4f8ff", clip=True, alpha=0.95)
    a.stroke([(2.6, 2.6), (8, 1.6), (13.4, 2.6)], 0.35, "#d8e4ff", clip=True, alpha=0.7)
    finish(a, "guard", shade=0.08)


def onigiri():
    a = new()
    a.blob([(8, 1.3), (11.2, 6.0), (14.0, 11.4), (12.8, 14.7), (3.2, 14.7), (2.0, 11.4), (4.8, 6.0)],
           ("rad", "#ffffff", "#d8d6e2", (0.45, 0.4)))
    for x, y in ((6.0, 6.6), (9.4, 5.2), (10.6, 8.6), (4.6, 9.6), (8, 8.4), (11.8, 11.0)):
        a.ellipse(x, y, 0.5, 0.3, "#c8c6d4", clip=True, alpha=0.7)
        a.ellipse(x - 0.1, y - 0.12, 0.35, 0.18, WHITE, clip=True)
    a.rect(5.0, 10.2, 11.0, 16, ("lin", "#3a5040", "#1e2c22", 90), radius=0.4, clip=True)
    a.stroke([(5.8, 11.2), (5.8, 14.2)], 0.35, "#6a8a70", clip=True, alpha=0.7)
    a.stroke([(5.0, 10.3), (11.0, 10.3)], 0.25, "#000000", clip=True, alpha=0.4)
    shine(a, 6.4, 3.6, 0.8, 1.2, 0.8)
    finish(a, "onigiri", shade=0.12)


ALL = [ofuda, moon_orbs, sakura_field, thunder_chain, taiko, star_fall, crescent, flame_pool,
       crimson_tendrils, devour, familiars, fox_mirage, foxfire_orb, gale_shuriken, grave_cleaver,
       gravity_well, iai_flash, lantern_vigil, mukade_spines, purification_wave, repulsion_star,
       soul_tether, spirit_seal, venom_fang, void_lance, yinyang_orbs,
       might, haste, expanse, swiftness, magnetism, vitality, abundance, persistence, recovery,
       guard, onigiri]

if __name__ == "__main__":
    for f in ALL:
        f()
    print("wrote", len(ALL), "icons")
