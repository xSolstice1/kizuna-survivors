"""HD redraw of the World 1 (Hidden Shrine Valley) and World 2 (Ember Leaf) enemies and bosses.

    python3 tools/hd/enemies_w12.py

Writes art/hd/w1/{fodder_a,fodder_b,ranged,blinker,elite,mini,boss,final}.png and the same for w2.
Deterministic. Designs follow tools/gen_w1.py and tools/gen_w2.py.
"""
import math
import pathlib
import random
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from PIL import Image, ImageDraw, ImageFilter  # noqa: E402
from hdkit import Art, rgb, SS, _catmull  # noqa: E402

NAMES = ["fodder_a", "fodder_b", "ranged", "blinker", "elite", "mini", "boss", "final"]


# ───────────────────────── helpers ─────────────────────────
def ribbon(a, pts, wfun, fill, clip=False, alpha=1.0, steps=12):
    """Smooth open band whose width follows wfun(t), t in [0,1] along the path."""
    path = _catmull(pts, False, steps) if len(pts) > 2 else list(pts)
    seg = [math.dist(path[i], path[i + 1]) for i in range(len(path) - 1)]
    tot = sum(seg) or 1.0
    m = Image.new("L", (a.W, a.H), 0)
    d = ImageDraw.Draw(m)
    acc = 0.0
    ws = [wfun(0.0)]
    for s in seg:
        acc += s
        ws.append(wfun(acc / tot))
    for i in range(len(path) - 1):
        (x0, y0), (x1, y1) = path[i], path[i + 1]
        dx, dy = x1 - x0, y1 - y0
        L = math.hypot(dx, dy) or 1e-6
        nx, ny = -dy / L, dx / L
        w0, w1 = ws[i] / 2, ws[i + 1] / 2
        quad = [(x0 + nx * w0, y0 + ny * w0), (x1 + nx * w1, y1 + ny * w1),
                (x1 - nx * w1, y1 - ny * w1), (x0 - nx * w0, y0 - ny * w0)]
        d.polygon([(x * SS, y * SS) for x, y in quad], fill=255)
        r = w1 * SS
        d.ellipse([x1 * SS - r, y1 * SS - r, x1 * SS + r, y1 * SS + r], fill=255)
    r = ws[0] / 2 * SS
    x0, y0 = path[0]
    d.ellipse([x0 * SS - r, y0 * SS - r, x0 * SS + r, y0 * SS + r], fill=255)
    a._paint(m, fill, clip, alpha)
    return path, ws


def halo(a, cx, cy, r, color, alpha=0.6, behind=False):
    """Soft light blob composited properly (no dark fringe) - for glows after outline()."""
    m = Image.new("L", (a.W, a.H), 0)
    ImageDraw.Draw(m).ellipse([(cx - r) * SS, (cy - r) * SS, (cx + r) * SS, (cy + r) * SS], fill=int(255 * alpha))
    m = m.filter(ImageFilter.GaussianBlur(r * SS * 0.45))
    layer = Image.new("RGBA", (a.W, a.H), rgb(color))
    layer.putalpha(m)
    a.img = Image.alpha_composite(layer, a.img) if behind else Image.alpha_composite(a.img, layer)


def spark(a, x, y, r, color, core="#ffffff"):
    """Four-point twinkle with a halo (drawn after outline)."""
    halo(a, x, y, r * 1.6, color, 0.55)
    a.poly([(x, y - r), (x + r * 0.28, y - r * 0.28), (x + r, y), (x + r * 0.28, y + r * 0.28),
            (x, y + r), (x - r * 0.28, y + r * 0.28), (x - r, y), (x - r * 0.28, y - r * 0.28)], color)
    a.ellipse(x, y, r * 0.3, r * 0.3, core)


def eye(a, x, y, rx, ry, iris, dark="#1a1020", hl="#ffffff"):
    """Anime eye: dark lash cap, gradient iris, highlight dot."""
    a.ellipse(x, y, rx, ry, ("lin", dark, iris, 90))
    a.ellipse(x - rx * 0.3, y - ry * 0.35, rx * 0.38, ry * 0.3, hl)
    a.stroke([(x - rx * 1.1, y - ry * 0.95), (x + rx * 1.05, y - ry * 1.05)], max(0.35, ry * 0.35), dark)


def M(pts, w=48):
    return [(w - x, y) for x, y in pts]


def _mix(c1, c2, t):
    a, b = rgb(c1), rgb(c2)
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(4))


def lrad(a, x, y, rx, ry, inner, outer, steps=7, focus=0.3, clip=False):
    """Local radial gradient ellipse (light focus toward the top-left)."""
    for i in range(steps):
        t = i / (steps - 1)
        sc = 1 - t * 0.78
        a.ellipse(x - rx * focus * t, y - ry * focus * t, rx * sc, ry * sc, _mix(outer, inner, t), clip=clip)


def orb(a, x, y, r, inner, outer, hl="#ffffff"):
    lrad(a, x, y, r, r, inner, outer)
    a.ellipse(x - r * 0.35, y - r * 0.38, r * 0.32, r * 0.24, hl, alpha=0.85)


# ═════════════════════════ WORLD 1 ═════════════════════════
def w1_fodder_a():
    # Kodama Sprite: pale tree spirit, big round head, hollow dot eyes.
    a = Art(16, 16)
    a.ellipse(6.2, 14.5, 1.3, 0.95, "#9cbca8")
    a.ellipse(9.8, 14.5, 1.3, 0.95, "#9cbca8")
    a.blob([(5.2, 10.0), (10.8, 10.0), (11.3, 13.2), (9.6, 14.8), (6.4, 14.8), (4.7, 13.2)],
           ("lin", "#cfe2d4", "#98b8a4", 90))
    a.ellipse(8.0, 12.9, 1.9, 1.4, "#7a9a8c", alpha=0.7)
    a.ellipse(7.6, 12.4, 0.8, 0.5, "#a8c8b4", alpha=0.8)
    # head
    a.blob([(2.3, 6.2), (3.6, 2.6), (7.0, 1.2), (11.0, 1.3), (13.8, 3.2), (14.5, 6.4), (13.2, 9.6),
            (9.5, 10.8), (5.0, 10.6), (2.6, 9.0)], ("lin", "#f6fcf3", "#d7e9d9", 80))
    a.blob([(1.8, 7.0), (6.0, 8.6), (10.0, 10.2), (8.0, 12.0), (2.0, 11.0)], "#a8c8b4", clip=True, alpha=0.9)
    a.blob([(2.4, 6.6), (4.5, 8.0), (3.0, 9.4)], "#bcd6c2", clip=True, alpha=0.6)
    # little cheek glow, hollow eyes and mouth
    a.ellipse(13.0, 7.8, 0.9, 0.5, "#9affd8", alpha=0.35)
    for ex in (7.5, 11.5):
        a.ellipse(ex, 5.9, 0.95, 1.15, ("lin", "#0e1616", "#2c3c3c", 90))
        a.ellipse(ex - 0.25, 5.4, 0.3, 0.3, "#9affd8", alpha=0.9)
    a.ellipse(10.0, 9.0, 0.5, 0.6, "#1e2a2a")
    a.rim("#ffffff", 0.45)
    a.outline()
    spark(a, 3.0, 3.0, 1.3, "#9affd8")
    a.save("w1/fodder_a")


def w1_fodder_b():
    # Paper Crane Shikigami: folded white crane, ink seal on the wing.
    a = Art(16, 16)
    a.ellipse(8.0, 14.6, 3.2, 0.65, "#8a84a6", alpha=0.85)
    a.poly([(7.0, 9.0), (3.0, 1.6), (9.2, 8.0)], ("lin", "#d6d0e2", "#aaa4c2", 90))     # far wing
    a.poly([(4.5, 10.2), (1.1, 4.0), (2.2, 4.0), (5.8, 9.2)], "#e6e2d6")                 # tail
    a.poly([(3.0, 10.0), (8.0, 7.5), (13.0, 10.0), (8.0, 12.6)], "#f4f1e6")               # body
    a.poly([(8.0, 12.6), (13.0, 10.0), (10.2, 13.0)], "#c4bed2")
    a.poly([(3.0, 10.0), (8.0, 10.4), (8.0, 12.6)], "#dcd8cc")
    a.poly([(11.2, 10.4), (14.0, 4.2), (15.0, 4.6), (13.2, 10.8)], "#f4f1e6")             # neck
    a.poly([(13.7, 3.6), (15.3, 4.0), (15.6, 5.6), (14.6, 4.9)], "#f4f1e6")               # head
    a.poly([(14.6, 4.9), (15.6, 5.6), (14.2, 5.4)], "#ffd27a")                            # beak tint
    a.poly([(5.0, 9.6), (10.0, 0.6), (11.8, 9.0)], ("lin", "#fffdf6", "#e4e0d2", 70))      # near wing
    a.poly([(10.0, 0.6), (11.8, 9.0), (10.6, 9.2)], "#cdc8d8", alpha=0.9)
    a.stroke([(9.9, 1.3), (9.1, 8.8)], 0.35, "#b4aec6")
    a.ellipse(9.3, 6.0, 0.85, 0.95, "#e0403a")
    a.ellipse(9.3, 6.0, 0.45, 0.5, "#ff7a6a", alpha=0.7)
    a.ellipse(14.6, 4.3, 0.22, 0.22, "#2a1c2a")
    a.outline()
    halo(a, 9.3, 6.0, 1.3, "#ff6a5a", 0.25)
    a.save("w1/fodder_b")


def w1_ranged():
    # Lantern Fairy: teal-haired fairy in lilac, holding out a glowing light orb.
    a = Art(16, 16)
    a.blob([(1.2, 2.6), (3.8, 3.6), (5.6, 6.6), (4.2, 7.6), (1.6, 7.2)], ("lin", "#e6fbff", "#9ad8ee", 90), alpha=0.95)
    a.blob([(1.6, 8.2), (5.0, 8.0), (4.2, 10.4), (1.6, 10.5)], ("lin", "#a8e2f6", "#6ab0d8", 90), alpha=0.95)
    a.stroke([(5.2, 6.5), (2.0, 3.4)], 0.3, "#ffffff", alpha=0.6)
    # dress & feet
    a.ellipse(6.4, 14.9, 0.75, 0.7, "#f4e2d8")
    a.ellipse(9.2, 14.9, 0.75, 0.7, "#f4e2d8")
    a.blob([(5.4, 8.0), (9.8, 8.0), (10.7, 12.5), (11.0, 14.3), (4.0, 14.3), (4.4, 12.0)],
           ("lin", "#c6a6f0", "#7a5aaa", 90))
    a.blob([(4.2, 12.6), (11.0, 12.6), (11.0, 14.3), (4.0, 14.3)], "#7a5aaa", clip=True, alpha=0.6)
    a.stroke([(4.4, 13.9), (10.8, 13.9)], 0.45, "#e6d8ff", alpha=0.7)
    a.stroke([(7.6, 8.4), (7.6, 12.0)], 0.35, "#8a6ac0", alpha=0.6)
    # arm out to the orb
    a.stroke([(9.4, 9.0), (11.6, 8.9)], 1.0, "#f4e2d8")
    # head
    a.ellipse(7.3, 4.8, 3.5, 3.4, "#357a9a")
    a.blob([(4.6, 3.4), (7.0, 1.4), (10.2, 1.8), (11.2, 4.4), (10.8, 6.6), (9.0, 3.6), (6.0, 4.0)],
           ("lin", "#7ad4e2", "#4aa8c0", 90))
    a.ellipse(8.9, 5.5, 1.9, 1.9, "#f4e2d8")
    a.blob([(6.8, 3.4), (10.4, 3.0), (10.0, 4.4), (8.0, 4.6)], "#5ab8c8")
    a.stroke([(10.6, 4.2), (10.9, 6.8)], 0.6, "#5ab8c8")
    a.stroke([(5.0, 3.6), (4.4, 7.6)], 0.5, "#2a6280", alpha=0.7)
    eye(a, 9.5, 5.6, 0.5, 0.75, "#3a6a8a", dark="#203040")
    a.ellipse(10.2, 6.6, 0.4, 0.25, "#ff9ab0", alpha=0.6)
    # orb
    orb(a, 13.3, 8.5, 2.0, "#fff6c0", "#ffb83a")
    a.rim("#ffffff", 0.3)
    a.outline()
    halo(a, 13.3, 8.5, 3.0, "#ffd060", 0.45)
    a.ellipse(13.0, 8.1, 0.7, 0.6, "#fffbe6")
    a.save("w1/ranged")


def w1_blinker():
    # Fox-Mask Phantom: white kitsune mask over a trailing violet wisp.
    a = Art(16, 16)
    a.blob([(2.6, 9.8), (7.0, 8.8), (11.0, 9.6), (10.5, 11.8), (8.0, 13.2), (6.0, 15.0), (4.0, 14.2),
            (2.4, 15.2), (1.4, 13.0)], ("lin", "#7a6ac8", "#3a3080", 100))
    a.stroke([(5.5, 11.5), (4.5, 13.5), (3.0, 15.0)], 0.9, "#2e2a62", alpha=0.85)
    a.stroke([(8.0, 11.5), (6.5, 13.2), (6.2, 14.6)], 0.6, "#2e2a62", alpha=0.7)
    a.stroke([(3.0, 11.4), (7.0, 10.4), (10.0, 10.8)], 0.5, "#a69aff", alpha=0.6)
    # mask
    a.poly([(5.2, 4.4), (5.6, 0.9), (8.4, 3.2)], "#f4f0ea")
    a.poly([(10.0, 3.2), (12.2, 0.6), (13.0, 4.6)], "#f4f0ea")
    a.poly([(5.9, 3.8), (6.0, 1.9), (7.4, 3.2)], "#e83a4a")
    a.poly([(10.8, 3.3), (11.9, 1.8), (12.3, 4.0)], "#e83a4a")
    a.blob([(5.2, 4.6), (8.0, 3.0), (11.6, 3.2), (13.2, 5.2), (15.2, 7.6), (15.0, 9.4), (12.0, 9.8),
            (8.0, 10.2), (5.4, 8.6)], ("lin", "#fffcf6", "#e0dad8", 90))
    a.blob([(4.6, 4.0), (6.6, 4.4), (6.4, 9.6), (4.6, 9.4)], "#c8c2cc", clip=True, alpha=0.9)
    a.blob([(8.0, 9.0), (15.4, 8.8), (15.4, 10.4), (8.0, 10.6)], "#d6d0d8", clip=True, alpha=0.8)
    # markings
    a.stroke([(8.0, 5.2), (9.2, 5.6), (10.4, 6.4)], 0.75, "#e83a4a", taper=True)
    a.stroke([(8.3, 6.3), (9.4, 6.8)], 0.5, "#e83a4a", taper=True)
    a.stroke([(10.1, 7.4), (11.9, 7.2)], 0.55, "#1a1020")
    a.stroke([(11.6, 9.2), (14.6, 9.2)], 0.65, "#e83a4a")
    a.ellipse(15.0, 8.2, 0.45, 0.4, "#1a1020")
    a.ellipse(7.2, 8.2, 0.55, 0.45, "#ff9ac0")
    a.rim("#ffffff", 0.35)
    a.outline()
    halo(a, 11.0, 7.3, 1.1, "#ff9ac0", 0.35)
    a.save("w1/blinker")


def w1_elite():
    # Komainu Guardian: sitting mossy stone lion-dog, open mouth, glowing teal eye.
    a = Art(32, 32)
    S, s, d, H = "#808a9c", "#5c6478", "#3c4256", "#a8b0c0"
    stone = ("lin", "#a2abbb", "#687084", 70)
    a.rect(1.6, 29.6, 27.0, 31.0, "#3c4256", radius=0.5)                      # plinth
    # tail flame curls
    a.blob([(3.4, 19.0), (0.9, 13.0), (2.4, 12.6), (1.6, 8.0), (4.4, 10.6), (5.0, 7.6), (6.8, 12.0),
            (7.0, 17.0)], stone)
    for p in ([(2.2, 12.4), (3.6, 13.6), (4.6, 16.4)], [(2.6, 9.0), (4.4, 11.4)], [(5.2, 8.8), (5.8, 12.6)]):
        a.stroke(p, 0.45, d, alpha=0.6)
    # haunch, hind paw, chest, front legs
    a.ellipse(10.0, 21.0, 8.0, 8.0, stone)
    a.rect(2.6, 26.6, 14.4, 30.0, ("lin", "#8c96a8", "#5c6478", 90), radius=1.6)
    for x in (5.0, 7.6, 10.2):
        a.stroke([(x, 28.2), (x, 29.8)], 0.4, d, alpha=0.7)
    a.blob([(12.0, 13.0), (24.0, 12.0), (25.6, 18.0), (25.0, 24.0), (14.0, 26.0)], stone)
    a.rect(18.8, 17.0, 23.6, 30.0, ("lin", "#98a2b4", "#5c6478", 0), radius=1.2)
    a.rect(17.6, 27.6, 25.8, 30.0, ("lin", "#8c96a8", "#5c6478", 90), radius=1.2)
    for x in (20.2, 22.4, 24.4):
        a.stroke([(x, 28.4), (x, 29.8)], 0.4, d, alpha=0.7)
    a.stroke([(18.6, 18.0), (18.4, 27.6)], 0.6, d, alpha=0.6)
    a.blob([(12.0, 20.0), (18.0, 24.0), (17.0, 29.0), (6.0, 29.5), (2.6, 24.0)], s, clip=True, alpha=0.45)
    # carved curls on the body
    for cx, cy, r in ((14.0, 16.4, 1.5), (8.0, 15.4, 1.6), (16.8, 22.0, 1.4), (5.4, 23.6, 1.6), (11.0, 20.0, 1.2)):
        pts = [(cx + r * math.cos(t) * (1 - t / 12), cy + r * math.sin(t) * (1 - t / 12)) for t in
               [i * 0.6 for i in range(12)]]
        a.stroke(pts, 0.45, d, alpha=0.75)
        a.stroke([(px - 0.2, py - 0.3) for px, py in pts[:6]], 0.3, H, alpha=0.5)
    # mane: ring of curls around the head
    for cx, cy, r in ((14.6, 3.2, 2.6), (12.6, 7.4, 2.8), (13.4, 12.0, 2.8), (17.6, 1.6, 2.2),
                      (17.0, 14.0, 2.4), (21.0, 14.2, 2.0)):
        a.ellipse(cx, cy, r, r, ("lin", "#949eb0", "#5c6478", 60))
        a.stroke([(cx - r * 0.5, cy - r * 0.2), (cx, cy - r * 0.5), (cx + r * 0.4, cy)], 0.4, d, alpha=0.6)
    # head & muzzle
    a.ellipse(21.0, 7.6, 6.0, 5.6, ("lin", "#b0b8c8", "#7a8496", 80))
    a.blob([(22.6, 5.6), (27.0, 5.8), (30.6, 7.0), (30.8, 11.0), (30.2, 13.6), (24.2, 14.2), (22.6, 10.0)],
           ("lin", "#b4bccc", "#7a8496", 90))
    a.rect(24.0, 10.8, 30.4, 13.4, "#2a2232", radius=0.8)                    # open mouth (a-gyo)
    a.blob([(25.0, 12.2), (29.4, 12.0), (29.0, 13.2), (25.4, 13.3)], "#6a2a3a")
    for x in (25.0, 29.4):
        a.poly([(x - 0.5, 10.8), (x + 0.5, 10.8), (x, 12.2)], "#ece6d6")
        a.poly([(x - 0.5, 13.4), (x + 0.5, 13.4), (x, 12.3)], "#ece6d6")
    a.ellipse(29.0, 7.6, 1.4, 1.0, H)
    a.ellipse(29.6, 8.0, 0.4, 0.3, d)
    a.ellipse(17.0, 5.0, 1.6, 1.6, s)
    a.stroke([(21.8, 4.8), (24.2, 4.6), (26.0, 5.4)], 0.8, d)                # heavy brow
    a.ellipse(24.4, 6.8, 1.3, 0.85, ("lin", "#d8fff2", "#2ad8a0", 90))
    a.ellipse(24.8, 6.8, 0.45, 0.55, "#1a5a4a")
    a.ellipse(24.0, 6.4, 0.3, 0.25, "#ffffff")
    a.stroke([(23.0, 9.4), (25.6, 9.0)], 0.4, d, alpha=0.6)
    # moss
    rng = random.Random(11)
    for cx, cy in ((19.0, 2.2), (14.0, 4.0), (6.2, 13.6), (8.0, 14.6), (12.6, 13.0), (15.0, 15.0),
                   (3.6, 12.4), (20.0, 21.0), (7.0, 23.0), (13.0, 24.4), (16.5, 25.0), (26.4, 11.0), (21.6, 1.2)):
        for _ in range(3):
            x, y = cx + rng.uniform(-1.0, 1.0), cy + rng.uniform(-0.4, 0.4)
            rx, ry = 0.4 + rng.random() * 0.6, 0.3 + rng.random() * 0.3
            a.ellipse(x, y, rx, ry, "#4c7a4c", clip=True, alpha=0.95)
            a.ellipse(x - 0.1, y - 0.15, rx * 0.6, ry * 0.5, "#6e9e5a", clip=True)
    a.shade_side("#1a1e30", 0.2, side="bottom", amount=0.25)
    a.rim("#dfe6f2", 0.45)
    a.outline()
    halo(a, 24.4, 6.8, 2.4, "#7affd0", 0.5)
    a.save("w1/elite")


def w1_mini():
    # Kagura Bell Maiden: ghostly lilac/teal spirit dancer raising a bell cluster.
    a = Art(32, 32)
    L, l, j, T, t = "#c0a6e6", "#8a6cc0", "#5a448e", "#5ad0c4", "#2a8a92"
    # long flowing hair behind (to the left)
    a.blob([(12.0, 3.0), (15.0, 3.0), (14.0, 10.0), (11.0, 17.0), (9.0, 22.0), (6.0, 25.0), (3.6, 24.0),
            (5.0, 18.0), (7.6, 9.0), (9.6, 4.6)], ("lin", "#ece6fa", "#b4a8d8", 100))
    for p in ([(10.6, 5.0), (8.6, 12.0), (6.0, 20.0), (4.6, 23.6)], [(12.4, 6.0), (10.6, 14.0), (8.2, 21.0)]):
        a.stroke(p, 0.4, "#9a8ec8", alpha=0.7)
    # trailing spirit hem (no feet)
    a.blob([(7.0, 23.0), (14.0, 22.6), (22.6, 23.6), (24.4, 26.4), (22.0, 29.0), (19.4, 30.6), (17.0, 29.6),
            (14.6, 31.0), (12.0, 29.8), (9.6, 30.4), (7.6, 28.6), (5.0, 29.0), (6.2, 26.0)],
           ("lin", "#9a7cd0", "#4a3880", 90))
    # outer robe
    a.blob([(11.4, 10.6), (20.4, 10.8), (23.0, 15.0), (24.4, 22.0), (22.6, 25.4), (13.0, 25.6), (7.6, 23.4),
            (8.2, 16.4)], ("lin", "#d6c4f4", "#9a7cd0", 90))
    a.blob([(13.6, 10.8), (18.6, 10.8), (19.0, 25.0), (13.4, 25.0)], T)              # teal mid layer
    a.blob([(14.6, 10.8), (17.4, 10.8), (17.6, 25.0), (14.4, 25.0)], ("lin", "#ffffff", "#d8d0f0", 90))
    a.blob([(18.6, 11.0), (19.4, 25.0), (18.4, 25.0)], t, alpha=0.6)
    a.rect(11.0, 16.8, 19.8, 18.8, ("lin", "#3aa6aa", t, 90), radius=0.4)            # sash
    a.ellipse(16.4, 17.8, 1.1, 1.0, "#ffd86a")
    a.ellipse(16.2, 17.5, 0.4, 0.3, "#fff6c8")
    a.stroke([(12.0, 18.8), (10.6, 22.0), (9.4, 24.0), (8.0, 26.6)], 0.9, "#e8607a", taper=True)
    a.stroke([(13.0, 18.8), (12.6, 22.4), (11.0, 25.4)], 0.7, "#c84060", taper=True)
    a.stroke([(13.4, 25.0), (23.0, 25.0)], 0.7, t)
    # left (back) sleeve hanging
    a.blob([(8.6, 13.0), (12.4, 12.6), (13.0, 19.0), (12.0, 23.4), (8.0, 23.6), (4.6, 22.6), (6.0, 17.0)],
           ("lin", "#e2d6f8", "#a68ad6", 80))
    a.blob([(4.4, 21.2), (9.0, 21.6), (8.6, 23.6), (4.4, 23.2)], T, clip=True)
    a.stroke([(7.4, 15.0), (6.2, 21.0)], 0.4, l, alpha=0.6)
    # right arm raised toward the bells
    a.blob([(19.0, 11.4), (22.6, 10.0), (25.6, 8.6), (27.2, 10.6), (26.0, 14.0), (24.4, 17.0), (21.4, 16.4)],
           ("lin", "#e2d6f8", "#a68ad6", 90))
    a.blob([(23.8, 9.4), (26.4, 8.2), (27.6, 11.2), (25.2, 11.6)], T, clip=True)
    a.stroke([(21.0, 13.0), (24.6, 14.6)], 0.4, l, alpha=0.6)
    a.ellipse(25.8, 9.0, 0.9, 0.8, "#f0e4ea")
    # head
    a.ellipse(16.4, 7.0, 3.6, 3.8, "#d8d0f0")
    a.ellipse(17.6, 8.2, 2.5, 2.6, "#f6ecf2")
    a.blob([(11.4, 3.0), (19.0, 2.8), (20.0, 5.2), (18.4, 6.2), (16.0, 5.6), (13.6, 7.2), (12.2, 6.0)],
           ("lin", "#f0ecfc", "#c8bee6", 90))
    a.stroke([(18.4, 5.0), (19.6, 8.0)], 0.6, "#d8d0f0")
    a.stroke([(16.8, 7.8), (17.6, 7.3), (18.6, 7.7)], 0.45, "#2aa8a0")           # closed eye
    a.stroke([(16.6, 7.4), (16.2, 7.0)], 0.3, "#2aa8a0")
    a.ellipse(18.2, 9.0, 0.7, 0.35, "#ff9ab0", alpha=0.55)
    a.stroke([(19.0, 10.0), (19.6, 9.9)], 0.3, "#c86a80")
    # headdress: crescent ornament with teal flowers
    a.blob([(11.4, 1.6), (15.6, 0.6), (19.6, 1.2), (19.2, 2.8), (15.6, 2.2), (11.8, 3.2)], ("lin", "#fff0a0", "#c09030", 90))
    for fx in (13.2, 17.2):
        a.ellipse(fx, 1.2, 0.85, 0.85, T)
        a.ellipse(fx, 1.2, 0.3, 0.3, "#ffffff")
    a.ellipse(15.4, 0.9, 0.6, 0.6, "#ffd86a")
    a.stroke([(11.6, 3.2), (10.4, 5.6), (9.4, 8.2)], 0.6, T)
    # kagura bell cluster on a short handle
    a.stroke([(26.2, 10.6), (27.2, 4.6)], 0.8, "#c09030")
    a.stroke([(27.2, 9.0), (28.6, 12.0), (30.0, 15.2)], 0.55, T)
    a.stroke([(27.8, 9.0), (29.4, 11.0), (30.0, 13.0)], 0.55, "#e8607a")
    for bx, by, r in ((25.4, 3.4, 1.2), (29.4, 3.4, 1.2), (27.4, 2.0, 1.25), (26.0, 5.8, 1.1), (28.8, 5.8, 1.1),
                      (27.4, 4.2, 1.15)):
        lrad(a, bx, by, r, r, "#fff4b0", "#d09a30", steps=5)
        a.stroke([(bx - r * 0.5, by + r * 0.35), (bx + r * 0.5, by + r * 0.35)], 0.25, "#8a6020")
        a.ellipse(bx - r * 0.35, by - r * 0.35, r * 0.25, r * 0.22, "#ffffff")
    a.ellipse(27.4, 1.0, 0.5, 0.5, "#bffff4")
    a.shade_side("#3a2a6a", 0.18, side="right", amount=0.25)
    a.rim("#ffffff", 0.4)
    a.outline()
    halo(a, 17.6, 7.6, 1.6, "#4ae0d0", 0.35)
    halo(a, 27.4, 4.0, 3.4, "#ffd86a", 0.3)
    for x, y, r in ((3.0, 6.0, 0.9), (29.2, 22.0, 0.9), (4.0, 27.4, 0.8), (8.0, 1.6, 0.6)):
        spark(a, x, y, r, "#bffff4")
    a.save("w1/mini")


def w1_boss():
    # Thousand-Torii Serpent: a vermilion serpent coiled through black torii, stone dragon head.
    a = Art(48, 48)
    R, r, q = "#e0473a", "#a02c30", "#6a1a26"
    path = [(33.0, 13.4), (24.0, 14.6), (15.0, 14.8), (8.6, 16.6), (5.6, 21.8), (7.2, 26.8), (13.0, 28.4),
            (24.0, 28.4), (35.0, 28.2), (41.0, 30.0), (42.4, 35.4), (39.0, 40.0), (31.0, 42.6), (19.0, 43.0),
            (9.4, 42.8), (4.8, 40.4), (3.0, 35.4)]

    def width(t):
        return 9.8 - 2.2 * t if t < 0.86 else (9.8 - 2.2 * 0.86) * (1 - (t - 0.86) / 0.14) + 1.4 * ((t - 0.86) / 0.14)

    a.stroke([(2.6, 36.0), (2.4, 31.0)], 1.6, "#efe8da")                         # paper streamer at the tail
    a.poly([(1.6, 31.0), (3.4, 31.0), (2.8, 33.2), (3.6, 34.4), (1.8, 34.0)], "#efe8da")
    shift = lambda pts, dx, dy: [(x + dx, y + dy) for x, y in pts]  # noqa: E731
    ribbon(a, path, width, q)
    ribbon(a, shift(path, -0.25, -0.55), lambda t: width(t) * 0.86, r)
    ribbon(a, shift(path, -0.45, -1.15), lambda t: width(t) * 0.56, ("lin", "#f4644a", R, 90))
    ribbon(a, shift(path, -0.6, -2.2), lambda t: width(t) * 0.16, "#ff9a7a", alpha=0.55)
    # scales: small arcs along the body
    dense = _catmull(path, False, 40)
    acc, seg_len, pts_at = 0.0, [], []
    for i in range(len(dense) - 1):
        seg_len.append(math.dist(dense[i], dense[i + 1]))
    tot = sum(seg_len)
    marks = []
    for i in range(len(dense) - 1):
        acc += seg_len[i]
        marks.append((acc / tot, dense[i], dense[i + 1]))

    def at(u):
        for t, p0, p1 in marks:
            if t >= u:
                dx, dy = p1[0] - p0[0], p1[1] - p0[1]
                L = math.hypot(dx, dy) or 1
                return p0, (dx / L, dy / L)
        return marks[-1][1], (0, 1)

    u = 0.02
    k = 0
    while u < 0.9:
        (x, y), (tx, ty) = at(u)
        nx, ny = -ty, tx
        hw = width(u) / 2
        for off in (-0.45, 0.0, 0.45):
            ox, oy = x + nx * hw * off + (tx * 0.6 if k % 2 else 0), y + ny * hw * off + (ty * 0.6 if k % 2 else 0)
            a.stroke([(ox - nx * 0.9 - tx * 0.2, oy - ny * 0.9 - ty * 0.2), (ox + tx * 0.5, oy + ty * 0.5),
                      (ox + nx * 0.9 - tx * 0.2, oy + ny * 0.9 - ty * 0.2)], 0.28, q, alpha=0.28, clip=True)
        u += 0.016
        k += 1
    # torii rings the serpent coils through
    for u in (0.07, 0.17, 0.37, 0.46, 0.55, 0.76, 0.86):
        (x, y), (tx, ty) = at(u)
        nx, ny = -ty, tx
        hw = width(u) / 2 + 1.0
        e = hw + 0.6
        a.stroke([(x - nx * e - tx * 0.7, y - ny * e - ty * 0.7), (x - nx * hw * 0.5, y - ny * hw * 0.5),
                  (x + nx * hw * 0.5, y + ny * hw * 0.5), (x + nx * e - tx * 0.7, y + ny * e - ty * 0.7)], 2.2,
                 ("lin", "#4a4458", "#16121e", 90))
        a.stroke([(x - nx * hw * 0.9 - tx * 0.5, y - ny * hw * 0.9 - ty * 0.5), (x - tx * 0.5, y - ty * 0.5)], 0.45,
                 "#7a7090", alpha=0.8)
        bx, by = x + tx * 2.4, y + ty * 2.4
        a.stroke([(bx - nx * (hw - 0.4), by - ny * (hw - 0.4)), (bx + nx * (hw - 0.4), by + ny * (hw - 0.4))],
                 1.0, "#2e2a3a")
        px_, py_ = x + tx * 1.0 - nx * (hw - 1.4), y + ty * 1.0 - ny * (hw - 1.4)
        a.rect(px_ - 0.55, py_ - 0.55, px_ + 0.55, py_ + 0.55, "#c8a860", radius=0.2)
    # head: stone dragon facing right, jaw open
    a.blob([(30.0, 7.4), (34.0, 5.4), (37.0, 5.6), (36.4, 9.0)], "#5e667a")
    a.poly([(31.4, 7.6), (25.4, 0.9), (29.0, 1.8), (34.2, 6.0)], ("lin", "#b4bccc", "#5e667a", 30))  # horn
    a.poly([(34.0, 6.4), (33.6, 0.8), (36.2, 2.0), (36.8, 6.6)], ("lin", "#c4ccdc", "#6e768a", 30))  # horn
    a.blob([(29.0, 10.0), (31.4, 6.6), (36.0, 5.6), (41.0, 6.4), (46.4, 8.8), (46.6, 12.0), (39.0, 13.4),
            (35.0, 17.4), (31.4, 18.6), (29.4, 15.0)], ("lin", "#b4bccc", "#6e768a", 80))
    a.blob([(35.0, 14.0), (45.2, 15.0), (44.0, 18.2), (36.6, 18.4), (34.0, 17.0)], ("lin", "#8a92a4", "#4c5468", 90))
    a.blob([(36.8, 12.6), (46.2, 12.0), (45.4, 15.2), (36.4, 15.0)], "#24202e")
    a.blob([(38.0, 14.0), (44.0, 13.6), (43.6, 14.8), (38.4, 14.8)], "#7a2a3a")
    for x in (38.6, 41.4, 44.2):
        a.poly([(x - 0.5, 12.5), (x + 0.5, 12.4), (x, 13.8)], "#efe8da")
    for x in (40.0, 42.8):
        a.poly([(x - 0.5, 15.1), (x + 0.5, 15.1), (x, 14.0)], "#efe8da")
    a.stroke([(33.0, 11.4), (40.0, 11.0), (45.6, 10.6)], 0.5, "#dfe4ee", alpha=0.7)      # snout ridge
    a.stroke([(36.6, 7.6), (41.6, 7.2)], 0.8, "#3c4256")                                   # brow
    a.ellipse(39.4, 8.8, 1.6, 0.85, ("lin", "#fff6a0", "#ffa030", 90))
    a.ellipse(39.8, 8.8, 0.35, 0.65, "#3a1a10")
    a.ellipse(44.8, 9.4, 0.5, 0.35, "#3c4256")
    for p in ([(33.4, 13.0), (31.4, 16.4), (30.6, 19.4)], [(31.6, 11.6), (29.4, 14.0), (28.0, 16.6)]):
        a.stroke(p, 0.6, "#9aa2b4", taper=True)
    for cx, cy, rx in ((34.0, 15.2, 1.6), (30.6, 10.0, 0.8), (38.4, 17.2, 0.8)):
        a.ellipse(cx, cy, rx, rx * 0.6, "#4c7a4c", clip=True)
        a.ellipse(cx - 0.2, cy - 0.2, rx * 0.5, rx * 0.3, "#6e9e5a", clip=True)
    a.rim("#ffd8c8", 0.35)
    a.outline()
    halo(a, 39.4, 8.8, 2.6, "#ffe066", 0.55)
    a.save("w1/boss")


def w1_final():
    # Mitsuhime of the Veiled Moon: veiled moon priestess before a moon halo, vast robes, floating orbs.
    a = Art(48, 48)
    Y, y = "#f0d080", "#b8943c"
    # moon halo: pale ring around a deep night disc
    a.ellipse(24.0, 12.0, 14.6, 12.6, ("rad", "#ffffff", "#c6d2f0", (0.5, 0.15)))
    a.ellipse(24.0, 12.6, 12.4, 11.0, ("lin", "#2e3a72", "#455a98", 90))
    for cx, cy, rr in ((15.8, 7.0, 1.4), (32.4, 9.0, 1.8), (17.4, 16.0, 1.0), (31.0, 16.6, 1.2)):
        a.ellipse(cx, cy, rr, rr, "#6a7ab8", alpha=0.6)
    # long silver hair spreading below the shoulders
    for sgn in (1, -1):
        pts = [(18.6, 8.0), (16.4, 11.0), (14.0, 16.0), (12.4, 21.0), (12.8, 25.0), (16.0, 24.0), (18.4, 18.0),
               (20.0, 12.0)]
        a.blob(pts if sgn == 1 else M(pts), ("lin", "#f2f4fc", "#9aa4cc", 90))
        st = [(17.6, 10.0), (15.4, 16.0), (14.0, 22.4)]
        a.stroke(st if sgn == 1 else M(st), 0.4, "#8a94c0", alpha=0.7)
    # vast robes flaring to the floor
    robe = [(19.0, 20.0), (29.0, 20.0), (34.0, 24.0), (38.4, 32.0), (43.4, 40.0), (46.0, 46.4), (2.0, 46.4),
            (4.6, 40.0), (9.6, 32.0), (14.0, 24.0)]
    a.poly(_catmull(robe, True, 8), ("lin", "#b4c8ee", "#6a86c0", 90))
    for x0 in (12.0, 17.0, 31.0, 36.0):                                       # folds
        dx = (x0 - 24.0) * 0.45
        a.stroke([(x0, 26.0), (x0 + dx * 0.6, 36.0), (x0 + dx, 45.6)], 0.9, "#6a86c0", alpha=0.75, clip=True)
        a.stroke([(x0 + 1.0, 27.0), (x0 + 1.0 + dx * 0.6, 36.0), (x0 + 1.0 + dx, 45.6)], 0.5, "#dfe8fa",
                 alpha=0.6, clip=True)
    a.poly([(20.6, 21.0), (27.4, 21.0), (32.4, 46.0), (15.6, 46.0)], ("lin", "#ffffff", "#dfe4f4", 90))
    a.poly([(21.6, 29.0), (26.4, 29.0), (27.6, 46.0), (20.4, 46.0)], ("lin", "#d6dcf0", "#aab4d8", 90))
    a.stroke([(21.0, 29.0), (19.6, 46.0)], 0.6, "#455a98")
    a.stroke([(27.0, 29.0), (28.4, 46.0)], 0.6, "#455a98")
    # huge sleeves hanging on both sides
    for sgn in (1, -1):
        sl = [(15.0, 21.0), (19.0, 22.0), (17.6, 30.0), (14.0, 40.6), (10.0, 43.0), (4.0, 43.4), (3.4, 40.0),
              (8.6, 30.0), (11.6, 23.6)]
        sl = sl if sgn == 1 else M(sl)
        a.blob(sl, ("lin", "#c4d4f2", "#6a86c0", 90))
        trim = [(4.0, 41.2), (9.0, 40.6), (13.6, 40.2)]
        a.stroke(trim if sgn == 1 else M(trim), 0.9, Y)
        cuff = [(4.0, 41.8), (13.4, 41.0), (12.0, 43.4), (4.0, 43.6)]
        a.blob(cuff if sgn == 1 else M(cuff), "#455a98", clip=True)
        fold = [(15.0, 23.0), (12.4, 31.0), (9.0, 39.6)]
        a.stroke(fold if sgn == 1 else M(fold), 0.6, "#6a86c0", alpha=0.8)
        hi = [(16.0, 23.4), (14.6, 30.0), (12.6, 38.0)]
        a.stroke(hi if sgn == 1 else M(hi), 0.4, "#ffffff", alpha=0.6)
    # hem pooling with gold trim
    a.rect(2.0, 44.8, 46.0, 47.0, ("lin", "#455a98", "#2e3a72", 90), radius=0.8)
    a.stroke([(2.6, 45.0), (45.4, 45.0)], 0.7, y)
    for xx in range(5, 44, 6):
        a.ellipse(xx + 0.5, 45.0, 0.6, 0.6, Y)
    # obi with crescent clasp
    a.rect(18.0, 27.0, 30.0, 29.6, ("lin", "#455a98", "#2e3a72", 90), radius=0.5)
    a.stroke([(18.2, 27.2), (29.8, 27.2)], 0.45, Y)
    a.ellipse(24.0, 28.0, 1.8, 1.8, Y)
    a.ellipse(24.8, 27.6, 1.5, 1.5, "#2e3a72")
    a.stroke([(24.0, 29.4), (23.4, 32.0)], 0.5, Y)
    a.stroke([(24.4, 29.4), (25.4, 31.6)], 0.5, Y)
    # collar layers
    a.poly([(19.0, 20.6), (24.0, 26.6), (21.0, 26.0)], "#f2f4fc")
    a.poly(M([(19.0, 20.6), (24.0, 26.6), (21.0, 26.0)]), "#f2f4fc")
    a.stroke([(19.6, 20.8), (24.0, 26.2), (28.4, 20.8)], 0.5, "#9ab6e2")
    # head and veil
    a.ellipse(24.0, 13.0, 5.0, 5.6, "#d4dcf2")
    a.blob([(19.0, 8.2), (24.0, 6.8), (29.0, 8.2), (28.6, 11.2), (19.4, 11.2)], ("lin", "#f2f4fc", "#c4cce6", 90))
    a.blob([(19.4, 11.0), (28.6, 11.0), (29.2, 19.0), (24.0, 20.8), (18.8, 19.0)], ("lin", "#c4cceff0", "#8a94d0e8", 90))
    a.stroke([(21.0, 12.0), (20.6, 19.0)], 0.4, "#7a84c0", alpha=0.7)
    a.stroke([(27.0, 12.0), (27.4, 19.0)], 0.4, "#7a84c0", alpha=0.7)
    a.stroke([(24.0, 12.0), (24.0, 20.2)], 0.4, "#7a84c0", alpha=0.6)
    a.stroke([(19.6, 11.2), (28.4, 11.2)], 0.7, Y)
    for ex in (22.4, 26.0):
        a.rect(ex - 0.6, 13.4, ex + 1.0, 14.2, "#7af0ff", radius=0.3)
    a.stroke([(18.8, 8.4), (18.4, 16.6)], 0.6, Y)                               # hair ornaments
    a.stroke([(29.2, 8.4), (29.6, 16.6)], 0.6, Y)
    for xx in (18.4, 29.6):
        a.ellipse(xx, 17.4, 0.5, 0.7, "#bfe8ff")
    # moon crown: crescent with jewels
    a.ellipse(24.0, 4.6, 3.6, 3.6, ("lin", "#fff0b0", y, 90))
    a.ellipse(24.0, 3.0, 3.2, 2.8, "#e6ecfa")
    a.ellipse(24.0, 3.2, 1.0, 1.0, "#bfe8ff")
    for xx in (21.0, 27.0):
        a.ellipse(xx, 7.4, 0.6, 0.6, Y)
    a.rim("#ffffff", 0.35)
    a.outline()
    halo(a, 24.0, 13.8, 3.4, "#7af0ff", 0.4)
    halo(a, 24.0, 12.0, 11.0, "#bfd8ff", 0.22, behind=True)
    # floating orbs
    for x, yy, rr in ((8.4, 9.0, 1.9), (39.6, 9.0, 1.9), (5.0, 22.0, 2.6), (43.0, 22.0, 2.6), (3.2, 33.0, 1.9),
                      (44.8, 33.0, 1.9)):
        halo(a, x, yy, rr * 2.0, "#7ac8ff", 0.45)
        b = Art(48, 48)
        orb(b, x, yy, rr, "#f0fbff", "#4aa0e8")
        b.outline(width=0.6)
        a.img = Image.alpha_composite(a.img, b.img)
    a.save("w1/final")


# ═════════════════════════ WORLD 2 ═════════════════════════
def w2_fodder_a():
    # Straw Puppet Soldier: animated straw dummy, sack head with glowing eyes, ragged tabard, spear.
    a = Art(16, 16)
    Yl = ("lin", "#e8c46a", "#a8803c", 90)
    a.stroke([(13.2, 2.4), (13.2, 14.4)], 0.9, ("lin", "#8a603a", "#5a3a20", 0))      # spear shaft
    a.poly([(13.2, 0.4), (14.0, 2.2), (13.2, 3.0), (12.4, 2.2)], ("lin", "#f0f2f6", "#9aa0ac", 0))
    # legs
    a.blob([(5.8, 12.0), (7.6, 12.0), (7.6, 14.8), (5.4, 15.0)], Yl)
    a.blob([(8.8, 12.0), (10.6, 12.0), (11.0, 15.0), (8.8, 14.8)], Yl)
    for x0 in (5.6, 8.9):
        a.stroke([(x0, 14.8), (x0 - 0.6, 15.2)], 0.4, "#ecd08a")
        a.stroke([(x0 + 1.8, 14.8), (x0 + 2.3, 15.3)], 0.4, "#ecd08a")
    # arms
    a.stroke([(2.2, 8.4), (12.4, 8.2)], 1.3, Yl)
    a.stroke([(1.6, 8.0), (2.6, 8.4)], 0.5, "#ecd08a")
    a.stroke([(1.8, 9.0), (2.8, 8.6)], 0.4, "#9a7434")
    # body + ragged tabard
    a.rect(5.0, 7.0, 10.8, 12.4, Yl, radius=0.6)
    a.blob([(6.0, 7.2), (10.4, 7.2), (10.6, 12.2), (9.6, 12.8), (8.4, 12.2), (7.2, 12.8), (6.0, 12.2)],
           ("lin", "#a83a30", "#5c1c1a", 90))
    a.blob([(9.0, 7.2), (10.4, 7.2), (10.6, 12.2), (9.0, 12.4)], "#5c1c1a", alpha=0.55)
    a.stroke([(5.0, 9.2), (10.8, 9.2)], 0.7, "#9a7434")
    a.stroke([(5.0, 9.0), (10.8, 9.0)], 0.3, "#ecd08a", alpha=0.7)
    a.stroke([(5.2, 9.6), (5.0, 11.0)], 0.4, "#9a7434")
    # sack head
    a.blob([(4.6, 3.0), (6.0, 1.4), (9.0, 1.0), (11.6, 2.0), (12.0, 4.8), (11.0, 6.8), (8.0, 7.2), (5.0, 6.6)],
           ("lin", "#d0ae74", "#9a7a4a", 90))
    a.blob([(4.6, 5.4), (12.0, 5.4), (11.0, 7.0), (5.0, 6.8)], "#86683c", clip=True, alpha=0.7)
    for p in ([(6.0, 1.6), (5.6, 0.4)], [(8.2, 1.0), (8.2, -0.2)], [(10.2, 1.4), (10.8, 0.4)],
              [(4.8, 2.6), (3.8, 2.0)]):
        a.stroke(p, 0.5, "#ecd08a", taper=True)
    a.stroke([(7.0, 2.6), (7.4, 4.4)], 0.3, "#86683c")
    a.stroke([(6.6, 3.0), (7.6, 3.0)], 0.25, "#86683c")
    a.stroke([(6.6, 3.8), (7.6, 3.8)], 0.25, "#86683c")
    for ex in (9.2, 11.2):
        a.ellipse(ex, 4.4, 0.55, 0.55, "#ff4a2a")
        a.ellipse(ex - 0.15, 4.25, 0.18, 0.18, "#ffe0a0")
    a.stroke([(9.0, 6.2), (9.8, 6.0), (10.6, 6.3), (11.2, 6.0)], 0.3, "#3a2a1a")
    a.stroke([(6.2, 7.2), (10.8, 7.1)], 0.6, "#3a2a1a")
    a.rim("#fff0c0", 0.35)
    a.outline()
    halo(a, 10.2, 4.4, 1.6, "#ff6a3a", 0.35)
    a.save("w2/fodder_a")


def w2_fodder_b():
    # Leaf Shinobi: lean masked ninja in green-brown wrappings, crouched to dash.
    a = Art(16, 16)
    G, g, h, k = "#5a6a34", "#3c4824", "#7e8c46", "#1e2016"
    a.ellipse(7.0, 15.0, 6.0, 0.6, "#1e2016", alpha=0.7)
    a.stroke([(6.4, 3.6), (4.4, 3.6), (2.6, 4.4), (1.2, 5.6)], 0.9, ("lin", "#c86a34", "#7a3a1c", 0), taper=True)
    a.stroke([(6.0, 4.6), (4.2, 5.0), (2.8, 6.4)], 0.6, "#7a3a1c", taper=True)
    # back leg extended, front leg bent
    a.blob([(4.0, 11.2), (6.6, 11.4), (5.6, 13.6), (3.2, 14.2), (1.4, 14.6), (1.4, 13.6), (2.8, 12.6)],
           ("lin", g, "#2a3418", 90))
    a.blob([(0.8, 13.8), (3.6, 13.8), (3.8, 15.0), (0.8, 15.0)], k)
    a.blob([(7.8, 10.6), (10.6, 10.4), (10.8, 13.2), (11.4, 14.0), (8.0, 14.2), (8.4, 12.6)], ("lin", h, G, 0))
    a.blob([(7.6, 13.6), (11.6, 13.6), (11.6, 14.8), (7.6, 14.8)], k)
    # crouched torso
    a.blob([(4.0, 7.6), (8.0, 6.6), (11.0, 6.6), (12.0, 9.0), (11.6, 11.2), (5.0, 11.6), (3.8, 9.6)],
           ("lin", h, G, 90))
    a.blob([(3.4, 7.6), (7.2, 7.4), (7.2, 11.6), (3.6, 11.6)], g, clip=True, alpha=0.8)
    a.stroke([(5.6, 10.2), (11.4, 9.0)], 0.75, "#3a2a1c")
    a.stroke([(6.4, 7.4), (10.6, 7.2)], 0.4, "#9aaa5a", alpha=0.7)
    # forward arm with a short blade held low
    a.stroke([(10.6, 8.0), (12.6, 9.6), (13.4, 10.2)], 1.0, G)
    a.ellipse(13.8, 10.4, 0.6, 0.6, "#3a2a1c")
    a.poly([(13.9, 10.6), (14.6, 10.8), (15.6, 12.8), (15.2, 13.0)], ("lin", "#ffffff", "#9aa0ac", 0))
    # head
    a.ellipse(9.5, 4.5, 3.2, 3.2, ("lin", h, G, 90))
    a.blob([(6.6, 2.6), (9.4, 1.2), (12.2, 2.2), (12.4, 3.2), (7.0, 3.4)], h, clip=True)
    a.rect(7.6, 3.6, 12.8, 5.2, k, radius=0.6)
    a.ellipse(10.4, 4.4, 0.5, 0.45, "#f8f080")
    a.ellipse(12.2, 4.4, 0.45, 0.42, "#f8f080")
    a.stroke([(9.0, 6.2), (12.4, 6.0)], 0.5, g)
    a.ellipse(6.4, 4.4, 0.6, 0.8, "#b05a2a")
    a.rim("#c8d880", 0.35)
    a.outline()
    halo(a, 11.3, 4.4, 1.4, "#f8f080", 0.3)
    a.save("w2/fodder_b")


def w2_ranged():
    # Kunai Thrower: navy ninja, red headband, raised throwing hand, bandolier of blades.
    a = Art(16, 16)
    N, n, h, S = "#4a4a6e", "#30304a", "#6a6a92", "#e0b894"
    a.stroke([(5.2, 2.6), (3.4, 3.4), (2.0, 4.6), (1.2, 6.0)], 0.9, "#c0302a", taper=True)
    a.stroke([(5.0, 3.2), (3.2, 4.8), (2.6, 6.4)], 0.6, "#801c1c", taper=True)
    # legs
    a.blob([(5.0, 12.0), (7.6, 12.0), (7.4, 15.0), (4.6, 15.0)], n)
    a.blob([(8.4, 12.0), (11.0, 12.0), (11.4, 15.0), (8.6, 15.0)], ("lin", N, n, 90))
    a.stroke([(4.8, 14.6), (7.6, 14.6)], 0.8, "#18141e")
    a.stroke([(8.6, 14.6), (11.6, 14.6)], 0.8, "#18141e")
    # back arm
    a.stroke([(5.0, 8.4), (4.0, 10.6), (3.6, 12.0)], 1.2, n)
    a.ellipse(3.6, 12.4, 0.6, 0.6, S)
    # body
    a.blob([(5.0, 7.4), (10.6, 7.2), (11.2, 10.4), (10.8, 12.8), (5.0, 12.8), (4.6, 10.0)], ("lin", h, N, 90))
    a.blob([(4.4, 7.0), (6.8, 7.0), (6.6, 13.0), (4.4, 13.0)], n, clip=True, alpha=0.7)
    a.stroke([(6.0, 7.6), (10.8, 12.8)], 0.9, "#7a5030")                          # bandolier
    for i in range(3):
        bx, by = 7.0 + i * 1.4, 8.6 + i * 1.4
        a.poly([(bx - 0.3, by + 0.3), (bx + 0.3, by - 0.3), (bx + 1.2, by - 1.2)], "#d4dae4")
    a.stroke([(5.0, 11.8), (10.8, 11.6)], 0.5, "#c0302a")
    # raised throwing arm
    a.stroke([(9.8, 7.6), (11.4, 5.4), (11.8, 3.4)], 1.1, N)
    a.ellipse(11.9, 2.9, 0.75, 0.75, S)
    a.poly([(12.2, 2.4), (14.6, 0.4), (15.0, 0.8), (12.6, 2.9)], ("lin", "#ffffff", "#9aa0ac", 0))
    a.poly([(12.4, 3.2), (14.8, 3.0), (14.8, 3.6), (12.4, 3.6)], ("lin", "#ffffff", "#9aa0ac", 0))
    # head
    a.ellipse(8.0, 4.6, 3.2, 3.1, ("lin", h, N, 90))
    a.rect(4.8, 2.0, 11.0, 3.3, ("lin", "#e0403a", "#a0201c", 90), radius=0.5)
    a.ellipse(11.0, 2.6, 0.6, 0.6, "#d4dae4")
    a.rect(7.2, 3.8, 10.6, 5.2, S, radius=0.5)
    eye(a, 9.4, 4.5, 0.45, 0.55, "#5a3a20", dark="#18141e")
    a.ellipse(7.8, 4.5, 0.35, 0.45, "#18141e")
    a.stroke([(7.0, 5.6), (10.6, 5.6)], 0.5, n)
    a.rim("#b0b0e0", 0.35)
    a.outline()
    halo(a, 14.0, 1.4, 1.0, "#ffffff", 0.3)
    a.save("w2/ranged")


def w2_blinker():
    # Smoke Ninja: a ninja half-dissolved into grey smoke, glowing cyan eyes and fingertip seal.
    a = Art(16, 16)
    D, d, h = "#3a3646", "#26222e", "#56506a"
    smoke = ("lin", "#a4a2b4", "#5e5c6e", 90)
    for cx, cy, rx, ry in ((6.0, 11.0, 3.5, 2.5), (10.0, 12.0, 3.2, 2.2), (7.0, 13.4, 4.6, 1.6),
                           (3.8, 9.0, 2.4, 2.0), (11.4, 14.0, 2.5, 1.0), (2.6, 12.6, 1.6, 1.4)):
        a.ellipse(cx, cy, rx, ry, smoke)
    for cx, cy, rx, ry in ((5.0, 10.0, 1.8, 1.0), (9.0, 11.4, 1.6, 0.9), (4.0, 13.2, 1.6, 0.6)):
        a.ellipse(cx, cy, rx, ry, "#bebccc", alpha=0.6)
    # torso (front still solid)
    a.blob([(6.6, 7.0), (12.0, 7.0), (12.4, 10.8), (7.4, 11.0)], ("lin", h, D, 90))
    a.stroke([(11.8, 8.0), (13.4, 9.6), (14.0, 10.8)], 1.0, D)
    # head
    a.ellipse(9.0, 4.5, 3.2, 3.2, ("lin", h, D, 90))
    a.blob([(6.6, 2.4), (9.0, 1.2), (11.4, 2.0), (11.6, 2.8), (7.0, 3.0)], h, clip=True)
    a.rect(7.8, 3.6, 12.6, 5.0, d, radius=0.6)
    # back half dissolves into wisps
    a.blob([(5.0, 1.0), (7.4, 2.0), (8.0, 9.0), (6.6, 12.0), (4.4, 10.0)], "#8a8898", clip=True, alpha=0.75)
    a.blob([(5.0, 1.0), (6.8, 2.0), (7.0, 9.0), (4.4, 10.0)], "#bebccc", clip=True, alpha=0.45)
    for p in ([(6.0, 4.4), (4.0, 5.0), (2.2, 4.6)], [(6.0, 6.6), (3.6, 7.4), (1.4, 7.2)]):
        a.stroke(p, 0.8, "#bebccc", taper=True, alpha=0.9)
    a.ellipse(10.4, 4.3, 0.5, 0.45, "#8af0ff")
    a.ellipse(12.2, 4.3, 0.45, 0.42, "#d8fcff")
    a.rim("#d8d4ea", 0.3)
    a.outline()
    halo(a, 11.3, 4.3, 1.6, "#8af0ff", 0.4)
    spark(a, 14.2, 11.2, 0.9, "#8af0ff")
    for x, y in ((2.2, 5.0), (13.0, 13.4), (1.6, 12.0)):
        halo(a, x, y, 0.6, "#d8d4ea", 0.45)
    a.save("w2/blinker")


def w2_elite():
    # Iron Mask Ronin: hulking armoured ronin, iron demon mask, wild mane, a huge blade.
    a = Art(32, 32)
    A, aa, q = "#2e2a38", "#4a4458", "#625a74"
    I, i_, J, B, R, G = "#767684", "#aeaebe", "#4a4a56", "#e0d4b4", "#a02a2a", "#c89a3a"
    armor = ("lin", "#58526a", "#2e2a38", 90)
    # wild dark mane
    a.blob([(9.0, 3.2), (12.0, 2.0), (12.0, 10.0), (11.0, 16.0), (9.0, 22.0), (5.0, 23.0), (3.0, 18.0),
            (2.8, 12.0), (5.2, 6.0)], ("lin", "#2a2434", "#14101a", 90))
    for p in ([(7.4, 5.0), (5.0, 12.0), (4.4, 19.0)], [(9.6, 5.4), (7.6, 13.0), (7.0, 20.0)]):
        a.stroke(p, 0.5, "#3e3650", alpha=0.8)
    # hakama + feet
    a.blob([(7.0, 22.8), (24.6, 22.8), (25.6, 28.8), (18.4, 29.0), (16.0, 26.0), (13.4, 29.0), (6.4, 29.0)],
           ("lin", "#4a3e3a", "#2c2422", 90))
    a.stroke([(11.0, 23.6), (10.4, 28.4)], 0.5, "#2a2020")
    a.stroke([(20.6, 23.6), (21.4, 28.4)], 0.5, "#2a2020")
    a.rect(4.6, 28.2, 13.0, 31.0, ("lin", "#3a3444", "#14101a", 90), radius=0.8)
    a.rect(18.0, 28.2, 26.0, 31.0, ("lin", "#3a3444", "#14101a", 90), radius=0.8)
    # torso armour (do) with lacing
    a.blob([(6.0, 11.4), (24.0, 11.4), (25.0, 16.0), (24.4, 22.6), (7.0, 22.6), (5.6, 16.0)], armor)
    for row, yy in enumerate((13.4, 16.0, 18.6, 21.0)):
        a.stroke([(6.2, yy), (24.4, yy)], 0.35, q, clip=True)
        for xx in range(8 + (row % 2) * 2, 24, 4):
            a.rect(xx - 0.45, yy - 1.0, xx + 0.45, yy - 0.1, R, radius=0.2, clip=True)
    a.rect(8.0, 21.4, 23.6, 22.8, ("lin", "#e8c060", "#8a6424", 90), radius=0.4)
    # shoulder guard (sode) and left arm with fist
    a.blob([(3.6, 11.4), (11.6, 11.0), (12.0, 18.0), (4.0, 18.6)], ("lin", "#625a74", "#2e2a38", 90))
    for yy in (13.4, 15.6, 17.6):
        a.stroke([(3.8, yy), (11.8, yy)], 0.4, "#1e1a24", clip=True)
        a.rect(5.4, yy - 0.9, 6.2, yy - 0.2, R, radius=0.2, clip=True)
        a.rect(9.0, yy - 0.9, 9.8, yy - 0.2, R, radius=0.2, clip=True)
    a.stroke([(5.6, 18.0), (5.6, 21.4)], 2.2, A)
    a.blob([(4.4, 21.0), (8.4, 21.0), (8.6, 23.6), (4.8, 23.8)], ("lin", "#d8ac88", "#a07454", 90))
    # iron demon mask / kabuto
    a.blob([(11.2, 4.0), (16.0, 2.2), (21.6, 3.2), (22.6, 7.0), (22.0, 11.6), (16.6, 12.6), (11.4, 11.6),
            (10.6, 7.0)], ("lin", "#a4a4b4", "#5a5a68", 90))
    a.blob([(12.2, 1.8), (21.0, 1.8), (22.2, 4.6), (11.6, 4.6)], ("lin", "#c4c4d2", "#767684", 90))
    a.stroke([(11.4, 4.8), (22.4, 4.8)], 0.7, J)
    a.blob([(12.0, 5.0), (21.4, 5.0), (20.6, 7.4), (16.8, 6.6), (12.6, 7.4)], "#3a3a46")
    for ex in (14.6, 19.0):
        a.ellipse(ex, 6.4, 1.0, 0.55, ("lin", "#ffd08a", "#ff4a2a", 90))
    a.stroke([(16.8, 7.0), (16.8, 9.6)], 0.6, J)
    a.blob([(13.0, 9.6), (20.6, 9.6), (19.8, 11.4), (13.8, 11.4)], "#24202c")
    for xx in (15.0, 18.6):
        a.poly([(xx - 0.5, 11.4), (xx + 0.5, 11.4), (xx, 9.0)], B)
    a.stroke([(12.0, 8.0), (13.0, 9.6)], 0.4, J)
    a.stroke([(21.6, 8.0), (20.6, 9.6)], 0.4, J)
    for hx, sgn in ((12.0, -1), (21.6, 1)):                                      # horns
        a.poly([(hx - 0.7, 2.6), (hx + 0.7, 2.6), (hx + sgn * 0.4, 0.3)], ("lin", "#fff4dc", "#c0b090", 90))
    # right arm + huge blade
    a.poly([(23.6, 21.2), (30.0, 1.4), (31.0, 1.6), (31.0, 3.0), (26.4, 20.4)], ("lin", "#f4f8ff", "#a8b0c0", 0))
    a.stroke([(24.6, 20.8), (30.4, 2.4)], 0.4, "#8a92a4", alpha=0.8)
    a.stroke([(26.0, 19.4), (30.8, 3.2)], 0.3, "#ffffff", alpha=0.8)
    a.rect(23.0, 17.6, 28.0, 19.8, ("lin", "#e8c060", "#8a6424", 90), radius=0.6)   # tsuba
    a.stroke([(21.6, 22.0), (23.8, 20.2)], 1.6, R)
    a.blob([(22.0, 15.0), (25.6, 15.4), (26.0, 18.6), (22.4, 19.4)], ("lin", "#d8ac88", "#a07454", 90))
    a.stroke([(22.2, 12.0), (23.4, 15.4)], 2.0, aa)
    a.rim("#c8c4dc", 0.35)
    a.outline()
    for ex in (14.6, 19.0):
        halo(a, ex, 6.4, 1.6, "#ff4a2a", 0.45)
    a.save("w2/elite")


def w2_mini():
    # Kagerou, Twin-Blade Captain: elite ninja captain, two short blades, flowing scarf, red visor.
    a = Art(32, 32)
    N, n, h, P, p = "#2a2e48", "#1a1c2e", "#424a6c", "#5a6280", "#8a94b4"
    Rc, r, G, S = "#d02a3a", "#801424", "#c89a3a", "#d8b090"
    blade = ("lin", "#ffffff", "#8a92a4", 0)
    # scarf trailing to the left
    ribbon(a, [(15.4, 10.0), (11.0, 9.8), (7.0, 8.6), (3.6, 9.4), (0.8, 8.0)], lambda t: 3.0 - 1.6 * t,
           ("lin", "#e84050", r, 90))
    ribbon(a, [(14.0, 11.0), (10.0, 13.0), (6.6, 12.6), (3.6, 14.6), (2.2, 15.6)], lambda t: 2.4 - 1.6 * t,
           ("lin", Rc, r, 90))
    a.stroke([(13.6, 9.0), (8.0, 8.0), (3.6, 8.6)], 0.4, "#ff8a90", alpha=0.6, clip=True)
    # legs in a dash stance
    a.blob([(13.0, 19.4), (17.0, 19.4), (15.6, 25.0), (13.6, 29.0), (10.0, 29.0), (11.6, 24.0)], ("lin", h, n, 0))
    a.blob([(11.6, 24.2), (14.6, 24.6), (13.8, 28.6), (11.0, 28.6)], ("lin", p, P, 0))
    a.blob([(7.0, 28.4), (14.0, 28.4), (14.0, 31.0), (6.6, 31.0)], ("lin", "#2a2e48", "#10101a", 90))
    a.blob([(18.4, 19.4), (22.4, 19.4), (25.0, 23.4), (27.6, 28.8), (24.0, 29.0), (21.6, 24.2)], ("lin", h, N, 0))
    a.blob([(23.0, 24.0), (26.0, 24.0), (27.6, 28.6), (24.4, 28.6)], ("lin", p, P, 0))
    a.blob([(22.0, 28.4), (28.6, 28.4), (28.8, 31.0), (21.6, 31.0)], ("lin", "#2a2e48", "#10101a", 90))
    # back arm with reverse-grip blade
    a.stroke([(14.0, 13.4), (11.4, 16.8), (9.6, 18.4)], 2.0, N)
    a.ellipse(9.4, 18.6, 1.0, 1.0, S)
    a.rect(9.6, 17.4, 10.8, 18.8, G, radius=0.3)
    a.poly([(9.0, 19.0), (3.0, 22.6), (2.6, 23.6), (4.0, 23.6), (9.8, 19.8)], blade)
    # torso
    a.blob([(13.6, 10.2), (22.8, 10.2), (24.4, 14.0), (23.4, 19.8), (13.0, 19.8), (12.6, 14.0)], ("lin", h, N, 90))
    a.rect(15.2, 11.8, 20.8, 16.6, ("lin", p, P, 90), radius=1.0)
    a.stroke([(15.4, 14.2), (20.6, 14.2)], 0.35, P)
    a.rect(13.0, 17.6, 23.6, 19.6, ("lin", "#e84050", r, 90), radius=0.4)
    a.rect(17.2, 17.4, 18.6, 19.8, G, radius=0.3)
    a.blob([(20.8, 10.4), (25.0, 10.8), (25.6, 13.4), (21.4, 13.6)], ("lin", p, P, 90))      # shoulder plate
    # head: hood, metal brow plate, glowing visor
    a.ellipse(17.2, 6.0, 4.8, 5.0, ("lin", h, N, 90))
    a.rect(13.2, 2.6, 21.6, 4.4, ("lin", "#e0e6f0", "#8a92a4", 90), radius=0.6)
    a.ellipse(15.6, 3.5, 0.4, 0.4, "#5a6280")
    a.rect(14.0, 5.0, 22.6, 6.8, ("lin", "#ff6a70", "#c0202c", 90), radius=0.8)
    a.rect(19.6, 5.2, 22.4, 5.9, "#ffc0c0", radius=0.3)
    a.ellipse(21.6, 5.5, 0.45, 0.35, "#ffffff")
    a.blob([(12.6, 6.0), (14.0, 6.0), (14.4, 11.0), (12.8, 10.6)], n, clip=True, alpha=0.7)
    # scarf wrap at the neck
    a.rect(13.6, 8.8, 22.4, 11.2, ("lin", "#e84050", r, 90), radius=1.0)
    a.stroke([(14.0, 9.4), (22.0, 9.4)], 0.35, "#ff9aa0", alpha=0.6)
    # forward arm, blade thrust forward
    a.stroke([(22.6, 12.4), (25.6, 15.4), (26.6, 15.6)], 2.0, N)
    a.ellipse(26.8, 15.6, 1.0, 1.0, S)
    a.rect(26.4, 14.0, 28.0, 15.2, G, radius=0.3)
    a.poly([(27.0, 14.4), (30.4, 9.4), (31.2, 9.0), (31.0, 10.0), (28.2, 15.0)], blade)
    a.rim("#a8b4e0", 0.35)
    a.outline()
    halo(a, 18.6, 5.8, 2.6, "#ff3a4a", 0.45)
    halo(a, 30.6, 9.4, 1.0, "#ffffff", 0.4)
    a.save("w2/mini")


def w2_boss():
    # Great Ember Toad: enormous war toad, saddle-shrine on its back, glowing warts, pipe smoke.
    a = Art(48, 48)
    T, t, q = "#7a3424", "#5a2418", "#9a4a2e"
    U, u, G, g = "#d0aa76", "#9a7850", "#c89a3a", "#8a6424"
    hide = ("lin", "#9a4a2e", "#5a2418", 90)
    # pipe smoke puffs rising
    for x, y, r in ((41.6, 21.0, 1.8), (44.4, 18.2, 1.9), (42.6, 14.6, 2.2), (45.0, 11.0, 2.0), (42.4, 7.2, 2.0)):
        a.ellipse(x, y, r, r * 0.9, ("lin", "#c4c2ce", "#6e6c7c", 90))
        a.ellipse(x - r * 0.3, y - r * 0.3, r * 0.4, r * 0.3, "#e0dee8", alpha=0.8)
    # back haunch, body, head
    a.ellipse(12.0, 37.0, 10.6, 9.8, hide)
    a.blob([(4.0, 32.0), (10.0, 23.0), (24.0, 20.6), (34.0, 20.0), (42.0, 22.0), (45.4, 26.0), (45.0, 34.0),
            (42.0, 40.0), (40.0, 44.6), (20.0, 45.4), (3.4, 44.0), (2.0, 38.0)], hide)
    a.blob([(32.0, 18.0), (36.0, 15.6), (41.0, 16.0), (45.2, 21.0), (45.6, 28.4), (40.0, 31.4), (32.0, 30.0)],
           ("lin", "#a8583a", "#6a2c1c", 90))
    # belly / throat
    a.blob([(22.6, 34.6), (30.0, 31.4), (38.6, 31.6), (43.4, 33.6), (42.4, 41.0), (34.0, 42.6), (26.0, 43.0),
            (22.0, 40.4)], ("lin", "#e0bc86", "#a8865a", 90))
    a.blob([(25.0, 38.0), (33.0, 37.6), (36.0, 40.0), (32.0, 43.0), (25.4, 43.0)], u, clip=True, alpha=0.6)
    for yy in (35.0, 37.4, 39.8):
        a.stroke([(24.0, yy), (31.0, yy - 0.6), (38.0, yy - 0.4)], 0.3, u, alpha=0.6, clip=True)
    # mouth, nostril
    a.stroke([(33.0, 30.4), (38.0, 31.0), (42.6, 29.8), (44.6, 28.2)], 0.8, "#160e10")
    a.stroke([(33.4, 29.4), (39.0, 29.8), (43.6, 28.6)], 0.4, q, alpha=0.7)
    a.ellipse(43.4, 23.2, 0.6, 0.4, "#160e10")
    # eye bulge
    a.ellipse(37.8, 19.0, 4.0, 3.8, ("lin", "#a8583a", "#7a3424", 90))
    lrad(a, 37.8, 19.4, 2.8, 2.6, "#fff4a0", "#ffb020")
    a.rect(35.0, 18.9, 40.6, 20.0, "#160e10", radius=0.5)
    a.ellipse(36.8, 18.0, 0.6, 0.45, "#ffffff")
    a.stroke([(34.4, 16.6), (37.8, 15.6), (41.2, 16.8)], 0.6, t)
    # front leg, back foot
    a.blob([(32.0, 37.0), (39.0, 38.0), (42.0, 42.0), (42.0, 45.4), (32.0, 45.4)], ("lin", "#8a3e28", "#5a2418", 0))
    a.stroke([(34.0, 39.0), (33.4, 44.0)], 0.5, t)
    for x0 in (32.0, 35.6, 39.4):
        a.ellipse(x0 + 1.6, 46.2, 2.0, 1.1, ("lin", U, u, 90))
    for x0 in (4.6, 9.6, 14.6):
        a.ellipse(x0 + 1.0, 46.4, 2.0, 0.9, ("lin", U, u, 90))
    a.blob([(4.0, 38.0), (12.0, 40.0), (16.0, 45.0), (3.0, 45.0)], t, clip=True, alpha=0.6)
    # pipe (kiseru) from the mouth corner
    a.stroke([(41.0, 30.0), (44.0, 28.4), (45.4, 27.0)], 0.8, "#3a2418")
    a.blob([(43.6, 24.4), (46.8, 24.4), (46.4, 28.0), (44.0, 28.0)], ("lin", "#f0c860", g, 90))
    a.ellipse(45.2, 24.4, 1.6, 0.5, "#3a2418")
    a.ellipse(45.2, 24.3, 1.0, 0.3, "#ff8a2a")
    # saddle blanket with gold trim
    a.blob([(8.0, 22.6), (29.0, 21.6), (30.6, 29.2), (7.4, 29.6), (6.6, 25.0)], ("lin", "#9a2434", "#541220", 90))
    a.blob([(6.0, 22.0), (10.0, 22.0), (9.4, 30.0), (6.0, 29.4)], "#541220", clip=True, alpha=0.7)
    a.rect(7.0, 28.6, 31.0, 30.8, ("lin", "#f0c860", g, 90), radius=0.6)
    for xx in range(9, 31, 3):
        a.ellipse(xx, 31.4, 0.5, 0.7, G)
    a.rect(8.0, 20.0, 28.8, 22.8, ("lin", "#6a4428", "#4a2c18", 90), radius=0.6)
    a.stroke([(28.4, 25.0), (30.4, 29.0), (32.0, 34.0)], 0.6, "#d8c08a")
    # shrine on the saddle
    a.rect(10.4, 18.6, 27.6, 20.6, ("lin", "#f0dcaa", "#c0a070", 90), radius=0.4)
    a.rect(12.0, 11.0, 26.0, 19.0, "#160e10")
    a.rect(11.4, 11.0, 13.6, 19.6, ("lin", "#d03a2a", "#901c18", 0), radius=0.3)
    a.rect(24.4, 11.0, 26.6, 19.6, ("lin", "#d03a2a", "#901c18", 0), radius=0.3)
    a.rect(13.6, 11.6, 24.4, 12.8, "#d8c08a")
    for xx in (15.0, 17.6, 20.4, 23.0):
        a.poly([(xx - 0.4, 12.8), (xx + 0.4, 12.8), (xx + 0.6, 14.2), (xx - 0.2, 14.0)], "#efe6c8")
    lrad(a, 19.0, 15.6, 2.6, 2.4, "#fff4a0", "#ff6a1a", focus=0.1)
    a.poly([(19.0, 12.8), (20.4, 15.4), (19.0, 17.4), (17.6, 15.4)], "#ffd060", alpha=0.9)
    # roof
    a.blob([(4.4, 10.0), (9.0, 7.6), (19.0, 6.6), (29.0, 7.6), (33.6, 10.0), (28.0, 11.6), (10.0, 11.6)],
           ("lin", "#5a5260", "#2e2634", 90))
    a.stroke([(9.0, 8.4), (29.0, 8.4)], 0.4, "#6e6478", alpha=0.8)
    a.rect(10.6, 5.0, 27.4, 7.0, ("lin", "#f0c860", g, 90), radius=0.5)
    for ex, sgn in ((4.6, -1), (33.4, 1)):
        a.poly([(ex - sgn * 1.6, 9.4), (ex + sgn * 0.8, 8.0), (ex + sgn * 0.2, 10.6)], G)
    a.rect(17.0, 2.8, 21.0, 5.4, ("lin", "#f0c860", g, 90), radius=0.8)
    a.blob([(18.0, 3.2), (18.6, 1.4), (19.4, 0.6), (20.0, 2.0), (20.2, 3.2)], ("lin", "#ffd060", "#ff6a1a", 90))
    # glowing warts
    for x, y, rr in ((8.0, 35.0, 0.8), (5.6, 39.6, 1.0), (9.6, 41.6, 0.8), (17.0, 38.8, 0.7), (19.0, 34.4, 0.7),
                     (30.4, 24.0, 0.8), (33.0, 25.6, 0.6), (36.4, 23.6, 0.6), (38.0, 39.8, 0.7), (35.0, 42.4, 0.6),
                     (40.0, 33.6, 0.5), (13.0, 32.4, 0.6)):
        lrad(a, x, y, rr, rr, "#ffe080", "#c8501a", steps=4)
    a.rim("#ffb070", 0.3)
    a.outline()
    halo(a, 19.0, 15.6, 4.0, "#ff8a2a", 0.4)
    halo(a, 37.8, 19.4, 3.0, "#ffd040", 0.35)
    halo(a, 19.0, 1.8, 1.6, "#ff8a2a", 0.5)
    for x, y, rr in ((8.0, 35.0, 0.8), (5.6, 39.6, 1.0), (30.4, 24.0, 0.8), (38.0, 39.8, 0.7), (17.0, 38.8, 0.7)):
        halo(a, x, y, rr * 2.2, "#ff8a2a", 0.4)
    a.save("w2/boss")


def w2_final():
    # Shogun of the Burning Leaf: black-and-crimson lacquered armour, blazing maple crest, burning cape, war fan.
    a = Art(48, 48)
    R, G = "#c02030", "#d8a840"
    lac = ("lin", "#4e445e", "#1a1620", 90)
    gold = ("lin", "#ffe08a", "#a87a2a", 90)

    def piece(pts, y0, y1, step=2.4, cord=2.2, smooth=True, trim=True):
        """A lacquered armour piece: horizontal plates laced with vertical crimson cords."""
        p = Art(48, 48)
        if smooth:
            p.blob(pts, lac)
        else:
            p.poly(pts, lac)
        xs = [x for x, _ in pts]
        yy = y0 + step
        while yy < y1:
            p.stroke([(min(xs) - 1, yy), (max(xs) + 1, yy)], 0.45, "#0e0a10", clip=True)
            p.stroke([(min(xs) - 1, yy + 0.45), (max(xs) + 1, yy + 0.45)], 0.3, "#6a5e7c", clip=True, alpha=0.8)
            yy += step
        xx = min(xs) + cord * 0.7
        while xx < max(xs) - 0.4:
            p.stroke([(xx, y0 - 1), (xx, y1 + 1)], 0.5, ("lin", "#e03040", "#801424", 90), clip=True)
            xx += cord
        if trim:
            p.stroke([(min(xs) - 1, y1 - 0.5), (max(xs) + 1, y1 - 0.5)], 0.9, gold, clip=True)
        p.shade_side("#000000", 0.2, side="right", amount=0.3)
        p.outline(width=0.45)
        a.img = Image.alpha_composite(a.img, p.img)

    # burning cape behind, flaring down to the left
    a.blob([(15.0, 15.0), (12.0, 22.0), (7.0, 29.0), (4.4, 37.0), (6.0, 44.0), (12.0, 45.4), (18.0, 44.0),
            (19.0, 28.0)], ("lin", "#b01c2c", "#4a0a14", 90))
    for p_ in ([(13.0, 21.0), (9.6, 30.0), (8.4, 42.0)], [(16.0, 22.0), (14.4, 33.0), (14.0, 43.0)]):
        a.stroke(p_, 0.7, "#560c18", alpha=0.8)
    a.stroke([(14.0, 18.0), (10.6, 26.0), (7.6, 33.0)], 0.5, "#e04050", alpha=0.6)
    edge = [(13.0, 17.0), (9.0, 24.6), (5.2, 30.6), (3.2, 37.6), (4.8, 43.6), (10.0, 45.4), (17.0, 45.0)]
    ribbon(a, edge, lambda t: 1.6 + 1.4 * math.sin(t * math.pi), ("lin", "#ffb040", "#ff5a1a", 90))
    dense = _catmull(edge, False, 8)
    rng = random.Random(7)
    for i in range(1, len(dense) - 2, 3):
        (bx, by), (cx, cy) = dense[i], dense[i + 1]
        dx, dy = cx - bx, cy - by
        L = math.hypot(dx, dy) or 1
        ox, oy = -dy / L, dx / L                      # outward (away from the body)
        ux, uy = ox * 0.8, oy * 0.8 - 1.0
        U = math.hypot(ux, uy)
        ux, uy = ux / U, uy / U
        h = 3.4 + rng.random() * 3.0
        tip = (bx + ux * h, by + uy * h)
        px_, py_ = -uy, ux
        a.blob([(bx + px_ * 1.5, by + py_ * 1.5), (bx + ux * h * 0.5 + px_ * 0.7, by + uy * h * 0.5 + py_ * 0.5), tip,
                (bx + ux * h * 0.45 - px_ * 0.8, by + uy * h * 0.45 - py_ * 0.8), (bx - px_ * 1.5, by - py_ * 1.5)],
               ("lin", "#ffe080", "#ff5a1a", 90))
        a.blob([(bx + px_ * 0.5, by + py_ * 0.5), (bx + ux * h * 0.5, by + uy * h * 0.5), (bx - px_ * 0.5, by - py_ * 0.5)],
               "#fff4c0", alpha=0.85)
    ribbon(a, [(x + 0.3, y - 0.2) for x, y in edge], lambda t: 0.6 + 0.6 * math.sin(t * math.pi), "#fff4c0", alpha=0.8)
    # legs: haidate thighs, greaves, sabatons
    piece([(16.6, 37.0), (23.4, 37.0), (23.2, 43.6), (17.0, 43.6)], 37.0, 43.6, step=2.0, cord=2.0, smooth=False,
          trim=False)
    piece([(25.6, 37.0), (32.4, 37.0), (32.0, 43.6), (25.8, 43.6)], 37.0, 43.6, step=2.0, cord=2.0, smooth=False,
          trim=False)
    a.rect(17.4, 41.4, 22.8, 42.4, "#a8a4b0", radius=0.3)
    a.rect(26.0, 41.4, 31.6, 42.4, "#a8a4b0", radius=0.3)
    a.rect(15.4, 43.6, 33.6, 44.8, gold, radius=0.4)
    a.blob([(15.6, 44.6), (23.8, 44.6), (24.0, 47.0), (15.0, 47.0)], ("lin", "#3a3244", "#0e0a10", 90))
    a.blob([(24.8, 44.6), (33.0, 44.6), (33.8, 47.0), (24.6, 47.0)], ("lin", "#3a3244", "#0e0a10", 90))
    a.blob([(22.4, 37.0), (26.4, 37.0), (26.0, 43.2), (22.8, 43.2)], ("lin", "#d02838", "#6a1018", 90))
    # kusazuri: four flaring skirt panels
    for x0, x1, b0, b1 in ((12.2, 18.6, 10.8, 18.2), (18.0, 24.2, 17.8, 24.0), (23.8, 30.0, 24.0, 30.4),
                           (29.4, 35.8, 30.0, 37.4)):
        piece([(x0, 29.0), (x1, 29.0), (b1, 38.4), (b0, 38.4)], 29.0, 38.4, step=2.3, cord=2.0, smooth=False)
    # torso (do)
    piece([(15.0, 14.8), (33.0, 14.8), (33.4, 22.0), (32.0, 30.0), (16.0, 30.0), (14.6, 22.0)], 17.6, 30.0,
          step=2.5, cord=2.4)
    a.rect(15.6, 15.0, 32.4, 18.0, ("lin", "#e03040", "#901c24", 90), radius=0.6)        # muna-ita
    a.stroke([(16.0, 15.4), (32.0, 15.4)], 0.35, "#ff8a8a", alpha=0.6)
    a.rect(15.4, 28.6, 32.6, 30.4, ("lin", "#3a3244", "#14101a", 90), radius=0.3)          # obi
    a.stroke([(15.4, 29.5), (32.6, 29.5)], 0.5, G)
    # gold maple emblem on the chest
    mx, my = 24.0, 23.0
    leaf = []
    for kk in range(14):
        ang = -math.pi / 2 + kk * math.pi / 7
        rr = 3.0 if kk % 2 == 0 else 1.2
        if kk in (6, 8):
            rr = 1.9
        leaf.append((mx + rr * math.cos(ang), my + rr * math.sin(ang)))
    a.poly(leaf, gold)
    a.stroke([(mx, my), (mx, my + 3.6)], 0.5, "#a87a2a")
    a.ellipse(mx - 0.6, my - 1.0, 0.5, 0.5, "#fff4c0", alpha=0.8)
    # shoulder guards (sode), flared out and down
    piece([(6.0, 15.0), (15.6, 14.6), (15.4, 27.4), (4.4, 26.6)], 16.0, 27.0, step=2.2, cord=2.2, smooth=False)
    piece([(32.4, 14.6), (42.0, 15.0), (43.6, 26.6), (32.6, 27.4)], 16.0, 27.0, step=2.2, cord=2.2, smooth=False)
    a.stroke([(6.0, 15.2), (15.6, 14.8)], 0.8, gold)
    a.stroke([(32.4, 14.8), (42.0, 15.2)], 0.8, gold)
    # gauntlets
    a.ellipse(41.6, 28.4, 1.8, 1.6, ("lin", "#564c64", "#1e1a24", 90))
    a.ellipse(6.0, 28.4, 1.8, 1.6, ("lin", "#564c64", "#1e1a24", 90))
    # war fan (gunbai) held up on the right
    a.stroke([(41.6, 19.0), (41.6, 29.6)], 1.2, ("lin", "#3a3040", "#14101a", 0))
    a.ellipse(41.8, 14.8, 4.3, 5.4, gold)
    a.ellipse(41.8, 14.8, 3.4, 4.5, ("lin", "#e03040", "#901c24", 90))
    lrad(a, 41.8, 14.4, 1.8, 2.0, "#fff4c0", "#ffa030")
    a.stroke([(39.0, 10.8), (41.8, 10.2), (44.6, 10.8)], 0.4, "#ffe08a", alpha=0.6)
    a.ellipse(41.6, 20.4, 1.0, 0.8, gold)
    # helmet neck guard (shikoro), flaring out
    piece([(16.0, 9.6), (32.0, 9.6), (35.4, 14.8), (12.6, 14.8)], 9.6, 14.8, step=1.7, cord=2.2, smooth=False)
    # helmet bowl with gold band
    a.blob([(16.8, 10.0), (18.2, 6.8), (24.0, 5.4), (29.8, 6.8), (31.2, 10.0)], ("lin", "#5e5470", "#1e1a24", 90))
    for xx in (19.6, 22.0, 24.0, 26.0, 28.4):
        a.stroke([(24.0 + (xx - 24.0) * 0.6, 5.8), (xx, 9.4)], 0.3, "#0e0a10", alpha=0.8)
    a.rect(16.6, 8.8, 31.4, 10.2, gold, radius=0.4)
    # menpo mask with glowing eyes
    a.blob([(20.0, 10.2), (28.0, 10.2), (28.6, 14.0), (27.2, 16.4), (20.8, 16.4), (19.4, 14.0)],
           ("lin", "#5a1c24", "#2a0c10", 90))
    a.rect(19.8, 10.6, 28.2, 12.2, "#0e0a10", radius=0.4)
    a.ellipse(22.2, 11.5, 1.1, 0.5, "#ffe040")
    a.ellipse(25.8, 11.5, 1.1, 0.5, "#ffe040")
    a.rect(20.6, 13.0, 27.4, 14.2, ("lin", "#e8e4f0", "#8a8698", 90), radius=0.3)
    for xx in (22.3, 24.0, 25.7):
        a.stroke([(xx, 13.0), (xx, 14.2)], 0.25, "#3a1418")
    a.stroke([(21.0, 15.4), (27.0, 15.4)], 0.4, "#8a3a40", alpha=0.7)
    # gold horns (kuwagata) and the blazing maple-leaf crest
    horn = [(19.4, 9.0), (13.0, 1.0), (15.4, 0.6), (21.6, 7.6)]
    a.poly(horn, gold)
    a.poly(M(horn), gold)
    a.stroke([(14.4, 1.0), (20.4, 8.0)], 0.3, "#fff4c0", alpha=0.7)
    a.stroke(M([(14.4, 1.0), (20.4, 8.0)]), 0.3, "#fff4c0", alpha=0.7)
    crest = []
    for kk in range(14):
        ang = -math.pi / 2 + kk * math.pi / 7
        rr = 4.0 if kk % 2 == 0 else 1.6
        if kk in (6, 8):
            rr = 2.6
        crest.append((24.0 + rr * math.cos(ang) * 1.1, 4.6 + rr * math.sin(ang)))
    a.poly(crest, ("lin", "#ffd060", "#ff5a1a", 90))
    a.blob([(24.0, 2.2), (25.2, 4.6), (24.0, 6.6), (22.8, 4.6)], "#fff4c0")
    a.rim("#c8b0d8", 0.25)
    a.outline()
    halo(a, 24.0, 4.6, 4.6, "#ff7a2a", 0.5)
    halo(a, 24.0, 11.5, 3.0, "#ffe040", 0.35)
    halo(a, 6.0, 34.0, 6.0, "#ff5a1a", 0.3, behind=True)
    halo(a, 41.8, 14.4, 2.4, "#ffc040", 0.3)
    for x, y, rr in ((8.0, 16.0, 0.7), (4.0, 13.4, 0.6), (2.0, 21.0, 0.5), (10.4, 11.6, 0.5), (34.0, 2.0, 0.5),
                     (1.6, 30.0, 0.5)):
        spark(a, x, y, rr * 1.4, "#ffb040", core="#fff4c0")
    a.save("w2/final")


def main():
    fns = [w1_fodder_a, w1_fodder_b, w1_ranged, w1_blinker, w1_elite, w1_mini, w1_boss, w1_final,
           w2_fodder_a, w2_fodder_b, w2_ranged, w2_blinker, w2_elite, w2_mini, w2_boss, w2_final]
    only = sys.argv[1:]
    for f in fns:
        if not only or any(o in f.__name__ for o in only):
            f()
            print("wrote", f.__name__)


if __name__ == "__main__":
    main()
