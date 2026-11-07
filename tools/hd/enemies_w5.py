"""HD redraw of World 5 (Shatterline Crossing) enemies + bosses, and the five relic icons.

    python3 tools/hd/enemies_w5.py

Mirrors the designs in tools/gen_w5.py (and the relic() functions in gen_w1..gen_w5.py),
drawn smooth and shaded with hdkit. Deterministic.
"""
import math
import pathlib
import random
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from PIL import Image, ImageDraw, ImageFilter  # noqa: E402
from hdkit import Art, SS, rgb  # noqa: E402


# ── helpers (pixel-grid coordinates of the .px generators -> continuous coords) ──
def R(a, x0, y0, x1, y1, fill, radius=0.0, **k):
    """Inclusive pixel rect, like Canvas.rect."""
    a.rect(x0, y0, x1 + 1, y1 + 1, fill, radius=radius, **k)


def D(a, x, y, fill, r=0.5, **k):
    """Round dot centred on pixel (x, y)."""
    a.ellipse(x + 0.5, y + 0.5, r, r, fill, **k)


def L(a, pts, w, fill, **k):
    """Stroke through pixel centres."""
    a.stroke([(x + 0.5, y + 0.5) for x, y in pts], w, fill, **k)


def halo(a, x, y, r, color, alpha=0.6):
    """Local soft light blob composited on top (for glowing eyes, lights, magic)."""
    m = Image.new("L", (a.W, a.H), 0)
    ImageDraw.Draw(m).ellipse([(x - r) * SS, (y - r) * SS, (x + r) * SS, (y + r) * SS], fill=255)
    m = m.filter(ImageFilter.GaussianBlur(r * SS * 0.5))
    layer = Image.new("RGBA", (a.W, a.H), rgb(color))
    layer.putalpha(m.point(lambda v: int(v * alpha)))
    a.img.alpha_composite(layer)


def spec(a, x, y, r=0.35, alpha=0.95):
    a.ellipse(x, y, r, r, "#ffffff", alpha=alpha)


def eye_small(a, x, y, white="#fff8e0", pupil="#1a0a20"):
    """A 2-px curse eyelet at pixel (x, y), pupil looking right."""
    a.ellipse(x + 1.0, y + 0.5, 1.0, 0.78, white)
    a.ellipse(x + 1.45, y + 0.55, 0.45, 0.55, pupil)
    spec(a, x + 1.25, y + 0.3, 0.18)


# ───────────────────────── fodder ─────────────────────────
def fodder_a():
    # Cursed Spawn: lumpy many-eyed curse blob.
    a = Art(16, 16)
    body = ("lin", "#a060d0", "#4a2268", 90)
    a.ellipse(8, 10, 7, 5.5, body)
    a.ellipse(6, 6.6, 3, 3, body)
    a.ellipse(11, 6.1, 2.6, 2.6, body)
    # drippy underside
    a.blob([(2.2, 12.5), (6, 11.6), (11, 11.4), (14.2, 12.2), (13.8, 14.4), (12.6, 15.8), (11.6, 14.9),
            (8.2, 15.0), (7.4, 15.9), (6.6, 15.0), (4.0, 15.0), (3.4, 15.9), (2.6, 14.6)], "#4e2470")
    a.ellipse(9, 12.4, 5.6, 2.2, "#5a2a80", clip=True, alpha=0.8)
    a.shade_side("#1a0a30", 0.30, side="right", amount=0.3)
    a.shade_side("#1a0a30", 0.25, side="bottom", amount=0.3)
    a.rim("#e0b0ff", 0.45)
    # glossy highlights
    a.ellipse(4.8, 5.6, 1.2, 0.7, "#c890f0", alpha=0.8)
    a.ellipse(10.4, 4.3, 0.9, 0.5, "#c890f0", alpha=0.8)
    a.ellipse(3.4, 9.0, 0.6, 0.9, "#b878e0", alpha=0.6)
    # many eyes
    for x, y in ((5, 6), (10, 5), (12, 9), (3, 10)):
        eye_small(a, x, y)
    a.ellipse(14.2, 11.4, 0.55, 0.5, "#fff8e0"); a.ellipse(14.4, 11.45, 0.25, 0.3, "#1a0a20")
    a.ellipse(13.4, 7.4, 0.55, 0.5, "#fff8e0"); a.ellipse(13.6, 7.45, 0.25, 0.3, "#1a0a20")
    # big yellow central eye
    a.ellipse(8.0, 9.0, 1.35, 1.3, "#ffe85a")
    a.ellipse(8.0, 9.6, 1.2, 0.6, "#e0a020", clip=True, alpha=0.6)
    a.ellipse(8.45, 9.15, 0.55, 0.85, "#1a0a20")
    spec(a, 7.5, 8.4, 0.35)
    # grinning mouth with pink tongue
    a.blob([(7.9, 12.0), (12.2, 11.9), (11.6, 13.0), (8.6, 13.1)], "#1a0a20")
    a.ellipse(9.8, 12.75, 0.9, 0.4, "#ff5aa0")
    a.outline()
    halo(a, 8.0, 9.0, 1.8, "#ffe85a", 0.35)
    a.save("w5/fodder_a")


def fodder_b():
    # Wailing Commuter: hunched cursed office husk, gaping mouth, briefcase.
    a = Art(16, 16)
    # legs (rear leg straight, front leg stepping)
    a.rect(5, 11.8, 7, 15.8, "#26263a", radius=0.4)
    a.blob([(9, 11.8), (11, 11.8), (11, 14.6), (12, 15.0), (12, 15.9), (9.2, 15.9)], "#2e2e44")
    # dangling far arm
    a.rect(2.8, 7.8, 4.2, 12, "#3c3c56", radius=0.6)
    a.ellipse(3.5, 12.4, 0.75, 0.8, "#9a9ab0")
    # suit torso, hunched
    a.blob([(4.2, 7.6), (7, 6.8), (11.2, 7.1), (12.1, 9.5), (11.8, 12.9), (4.0, 12.9), (3.9, 10)],
           ("lin", "#5a5a78", "#34344a", 0))
    # shirt + tie
    a.poly([(7, 6.9), (9, 6.9), (8.2, 9.6), (7.5, 9.6)], "#d8d8e0")
    a.poly([(7.3, 7.6), (8.0, 7.6), (8.2, 10.6), (7.6, 11.1), (7.1, 10.6)], "#c0303a")
    a.stroke([(5.2, 10.5), (5.4, 12.6)], 0.25, "#26263a", alpha=0.6)
    # near arm holding the briefcase
    a.rect(11.6, 7.8, 13.0, 11.0, "#34344a", radius=0.6)
    a.ellipse(12.4, 11.5, 0.7, 0.7, "#9a9ab0")
    # briefcase
    a.stroke([(12.4, 12.0), (12.6, 11.1), (14.4, 11.1), (14.6, 12.0)], 0.45, "#5a3a1a")
    a.rect(11, 12, 16, 15, ("lin", "#a06a38", "#6a4220", 90), radius=0.5)
    a.rect(11, 14.2, 16, 15, "#5a3a1a", radius=0.4, clip=True)
    a.rect(13.2, 12.6, 13.8, 13.3, "#d0b060")
    # head, drooping forward
    a.ellipse(8.6, 3.6, 3.5, 3.4, ("lin", "#b0b0c4", "#7a7a92", 30))
    a.blob([(5.2, 2.6), (6.0, 0.6), (9, 0.2), (11.2, 1.0), (10.6, 1.9), (7.0, 1.9), (6.2, 3.4)], "#686880")
    a.shade_side("#14142a", 0.25, side="right", amount=0.25)
    a.rim("#ffffff", 0.35)
    # hollow eyes with purple pinprick glow, gaping mouth
    a.ellipse(9.5, 2.6, 0.6, 0.7, "#1a0a20"); a.ellipse(11.4, 2.6, 0.55, 0.7, "#1a0a20")
    a.ellipse(9.55, 2.8, 0.25, 0.25, "#e8c0ff"); a.ellipse(11.45, 2.8, 0.22, 0.25, "#e8c0ff")
    a.blob([(9.1, 4.2), (11.8, 4.0), (12.0, 5.6), (11.0, 7.0), (9.4, 6.4)], "#1a0a20")
    a.ellipse(10.6, 5.4, 0.7, 0.8, "#c070ff", alpha=0.85)
    a.outline()
    halo(a, 10.6, 5.4, 1.2, "#c070ff", 0.45)
    halo(a, 10.5, 2.8, 1.2, "#c070ff", 0.25)
    a.save("w5/fodder_b")


def ranged():
    # Curse Eye: giant floating eyeball wreathed in purple flame (flame streams back / left).
    a = Art(16, 16)
    a.blob([(1, 4), (4, 6), (3, 1), (6, 4), (8, 0), (9, 4), (12, 1), (12, 6), (15, 7), (13, 13),
            (6, 15), (2, 12), (0, 8), (3, 9)], ("lin", "#7a2ac0", "#3e1070", 90))
    a.blob([(2, 6), (5, 6), (5, 3), (7, 5), (8, 2), (10, 5), (11, 4), (12, 8), (11, 12), (6, 13), (3, 11)],
           ("lin", "#c080ff", "#8a3ad0", 90))
    # eyeball
    a.ellipse(9, 9, 5, 4.8, ("rad", "#ffffff", "#c8b8d8", (0.62, 0.5)))
    a.ellipse(7.2, 9.6, 3.4, 4.0, "#a090b8", clip=True, alpha=0.45)
    # veins
    a.stroke([(4.6, 7.4), (6.0, 7.9), (7.6, 8.4)], 0.32, "#c03050", taper=True)
    a.stroke([(5.0, 11.6), (6.6, 11.0), (8.4, 10.6)], 0.32, "#c03050", taper=True)
    a.stroke([(9.6, 13.4), (10.2, 12.2)], 0.25, "#c03050", taper=True)
    # iris + pupil (looking right)
    a.ellipse(11.4, 9.0, 2.7, 2.7, ("rad", "#f080ff", "#7a10b0", (0.72, 0.56)))
    a.ellipse(11.9, 9.1, 1.25, 1.35, "#120818")
    spec(a, 10.5, 7.8, 0.6)
    spec(a, 12.6, 10.3, 0.25, 0.7)
    a.outline()
    halo(a, 11.4, 9.0, 2.8, "#d03aff", 0.25)
    for x, y, r in ((4.5, 1.5, 0.45), (9.5, 1.3, 0.4), (14.5, 5.5, 0.45)):
        halo(a, x, y, 1.0, "#c070ff", 0.6)
        a.ellipse(x, y, r, r, "#f0d0ff")
    a.save("w5/ranged")


def blinker():
    # Glitch Shade: shadow figure with flickering, chromatic, glitched slices.
    a = Art(16, 16)
    body = ("lin", "#5a4a80", "#2a2240", 90)
    a.blob([(4.2, 6.4), (8, 5.8), (12, 6.4), (13.4, 9.5), (13, 12.5), (11.5, 15.5), (10.2, 13.4),
            (9, 13.4), (7.2, 15.6), (6, 13.4), (5, 13.6), (3.2, 15.6), (2.8, 11.5), (3.0, 8)], body)
    a.ellipse(8.5, 3.8, 3.1, 3.1, body)
    a.shade_side("#100a20", 0.4, side="right", amount=0.35)
    a.rim("#a090e0", 0.5)
    a.ellipse(6.2, 5.0, 0.8, 0.5, "#6a5a94", alpha=0.7)
    a.stroke([(5.5, 7.3), (5.2, 9.0)], 0.6, "#6a5a94", alpha=0.6)
    # white eyes
    a.ellipse(8.6, 3.6, 0.5, 0.65, "#ffffff")
    a.ellipse(10.6, 3.6, 0.5, 0.65, "#ffffff")
    a.outline()
    halo(a, 9.6, 3.6, 1.6, "#ffffff", 0.3)
    # glitch: shift horizontal slices, then add chromatic fringes
    rng = random.Random(56)
    img = a.img
    for y in (5, 9, 10, 13):
        s = rng.choice((-1, 1, 2))
        band = img.crop((0, y * SS, a.W, (y + 1) * SS))
        clear = Image.new("RGBA", band.size, (0, 0, 0, 0))
        img.paste(clear, (0, y * SS))
        img.paste(band, (s * SS, y * SS))
        al = band.getchannel("A")
        xs = [x for x in range(band.width) if al.getpixel((x, SS // 2)) > 60]
        a.img = img
        if xs:
            x0 = xs[0] / SS + s
            x1 = xs[-1] / SS + s
            a.rect(x0 - 1.1, y + 0.15, x0 - 0.1, y + 0.85, "#3af0ff")
            a.rect(x1 + 0.1, y + 0.15, x1 + 1.1, y + 0.85, "#ff3aa0")
        img = a.img
    for x, y, col in ((1, 3, "#3af0ff"), (14, 7, "#ff3aa0"), (0, 11, "#3af0ff")):
        a.rect(x, y + 0.2, x + 1, y + 0.8, col)
    a.rect(12.5, 1.4, 14.5, 1.8, "#3af0ff", alpha=0.6)
    a.rect(1.5, 14.2, 3.0, 14.6, "#ff3aa0", alpha=0.6)
    a.save("w5/blinker")


# ───────────────────────── elite / mini ─────────────────────────
def elite():
    # Grade-One Brute: hulking four-armed curse with a stitched face.
    a = Art(32, 32)
    skin = ("lin", "#9a7aaa", "#4a3258", 90)
    # legs
    a.blob([(8, 24.5), (14, 24.5), (14.2, 29.6), (15, 31.9), (6, 31.9), (7.2, 29.6)], "#523a62")
    a.blob([(18, 24.5), (24, 24.5), (24, 29.6), (25.8, 31.9), (17, 31.9), (17.8, 29.6)], "#523a62")
    a.rect(6, 30.2, 15, 32, "#38264a", radius=0.8)
    a.rect(17, 30.2, 26, 32, "#38264a", radius=0.8)
    for x in (6.8, 8.6, 10.4): a.ellipse(x, 31.6, 0.5, 0.4, "#e8dcc8")
    for x in (21.6, 23.4, 25.2): a.ellipse(x, 31.6, 0.5, 0.4, "#e8dcc8")
    # torso + hulking shoulders
    a.ellipse(16, 18, 11, 9, skin)
    a.ellipse(9, 11, 6, 5, skin)
    a.ellipse(22, 11, 6, 5, skin)
    a.ellipse(18, 21.5, 9, 5.6, "#523a62", alpha=0.8)
    # upper arms: huge raised claws
    a.blob([(3, 9), (6.5, 9.2), (5.6, 17.5), (3.6, 21.5), (0.6, 21.5), (0.4, 18)], skin)
    a.blob([(26, 9), (29.2, 10), (31.4, 16), (31.4, 21.5), (27.6, 21.5), (26.8, 18)], skin)
    a.rect(0, 19.8, 4, 23, "#523a62", radius=1.0)
    a.rect(28, 19.8, 32, 23, "#523a62", radius=1.0)
    for x in (0.6, 2.6):
        a.poly([(x - 0.2, 22.6), (x + 0.9, 22.6), (x + 0.3, 24.6)], "#e8dcc8")
    for x in (28.6, 30.6):
        a.poly([(x - 0.2, 22.6), (x + 0.9, 22.6), (x + 0.5, 24.6)], "#e8dcc8")
    # head pushed forward
    a.ellipse(19.5, 8.2, 5, 5, skin)
    a.ellipse(20.5, 10.8, 3.6, 2.4, "#523a62", alpha=0.45)
    a.shade_side("#1a0a24", 0.30, side="right", amount=0.3)
    a.shade_side("#1a0a24", 0.25, side="bottom", amount=0.3)
    a.rim("#e0c8f0", 0.45, width=0.8)
    # lower arms hanging from the belly (darker, behind-ish)
    a.blob([(6, 18), (9.2, 19), (8.6, 25.6), (5.4, 27.4), (4.8, 24)], "#38264a")
    a.blob([(24, 19), (27, 18), (28.4, 24), (28.4, 26.8), (25.2, 26.2)], "#38264a")
    a.poly([(5, 26.8), (6.2, 26.8), (5.2, 28.6)], "#e8dcc8")
    a.poly([(27.6, 26.2), (28.8, 26.2), (28.6, 28.0)], "#e8dcc8")
    # highlights
    a.ellipse(8, 8, 2.4, 1.2, "#b89cc8", alpha=0.7)
    a.ellipse(18, 4.6, 1.5, 0.7, "#c0a4d0", alpha=0.7)
    # face seam + stitches
    a.stroke([(18.5, 4.2), (19.8, 8), (21.5, 12.5)], 0.5, "#e8dcc8")
    for t in (5.5, 7.6, 9.8):
        cx = 18.5 + (t - 4.2) * 0.36
        a.stroke([(cx - 0.9, t - 0.3), (cx + 0.9, t + 0.3)], 0.32, "#c8c0b0")
    # eyes: small left eye, big glaring right eye
    a.ellipse(17.2, 7.5, 0.8, 0.6, "#ff5a3a"); a.ellipse(17.5, 7.5, 0.35, 0.35, "#ffd27a")
    a.ellipse(23.1, 6.8, 1.3, 1.1, "#ff5a3a"); a.ellipse(23.4, 6.8, 0.6, 0.6, "#ffd27a")
    spec(a, 22.6, 6.3, 0.3)
    # mouth with teeth
    a.blob([(17.6, 10.8), (24.4, 10.6), (23.8, 13.4), (18.4, 13.5)], "#1a0a1a")
    a.ellipse(21, 12.9, 2.2, 0.5, "#7a1a2a", alpha=0.8)
    for x in (18.2, 20.2, 22.2):
        a.poly([(x, 10.8), (x + 1.4, 10.8), (x + 0.7, 12.3)], "#e8dcc8")
    for x in (19.3, 21.3):
        a.poly([(x, 13.4), (x + 1.2, 13.4), (x + 0.6, 12.4)], "#c8c0b0")
    # body stitches + curse sigil
    a.stroke([(10.5, 15.5), (12.4, 20), (14.5, 24.5)], 0.5, "#e8dcc8")
    for k in range(4):
        y = 16.5 + k * 2
        cx = 10.5 + (y - 15.5) * 0.44
        a.stroke([(cx - 1.0, y - 0.2), (cx + 1.0, y + 0.2)], 0.32, "#c8c0b0")
    a.ellipse(19.5, 18.5, 2, 2, ("rad", "#e0a0ff", "#8a30d0"))
    a.ellipse(19.5, 18.5, 0.7, 0.7, "#1a0a1a")
    a.outline()
    halo(a, 19.5, 18.5, 2.8, "#b050ff", 0.45)
    halo(a, 23.1, 6.8, 2.0, "#ff5a3a", 0.45)
    halo(a, 17.2, 7.5, 1.3, "#ff5a3a", 0.35)
    a.save("w5/elite")


def mini():
    # The Patchwork Sorcerer: patchwork coat, mismatched eyes, puppet strings.
    a = Art(32, 32)
    # coat (flared) + patches
    coat = [(9.2, 12), (20.8, 12), (22.6, 18), (24.2, 27.2), (15, 27.8), (5.8, 27.2), (7.4, 18)]
    a.blob(coat, "#3a6a6a")
    patches = [(9, 12, 14, 17, "#8a4a3a"), (15, 12, 21, 16, "#5a3a6a"), (7, 18, 12, 22, "#8a7a3a"),
               (13, 17, 18, 22, "#3a6a6a"), (19, 17, 23, 22, "#8a4a3a"), (6, 23, 11, 27, "#5a3a6a"),
               (12, 23, 17, 27, "#8a7a3a"), (18, 23, 24, 27, "#3a6a6a")]
    for x0, y0, x1, y1, col in patches:
        a.rect(x0, y0, x1 + 1, y1 + 1.4, col, clip=True)
    a.shade_side("#101020", 0.35, side="right", amount=0.28)
    a.shade_side("#101020", 0.2, side="bottom", amount=0.25)
    # seams with stitches
    for x in range(8, 24, 2):
        a.stroke([(x + 0.2, 17.5), (x + 0.9, 17.5)], 0.35, "#e8e0c8")
        a.stroke([(x + 0.2, 22.5), (x + 0.9, 22.5)], 0.35, "#e8e0c8")
    for y in range(12, 28, 2):
        x = 14 if y < 17 else 12 if y < 23 else 17
        a.stroke([(x + 0.5, y + 0.2), (x + 0.5, y + 0.9)], 0.35, "#e8e0c8")
    # legs / boots below the hem
    a.rect(11, 27, 14, 30, "#2e3a54")
    a.rect(17, 27, 20, 30, "#2e3a54")
    a.rect(10, 29.6, 15, 32, "#1e1a24", radius=0.8)
    a.rect(16, 29.6, 22, 32, "#1e1a24", radius=0.8)
    a.ellipse(20.5, 30.2, 0.8, 0.4, "#4a4458", alpha=0.7)
    # collar + buttons
    a.blob([(10.8, 11.2), (15, 12.6), (19.4, 11.2), (19.6, 13), (15, 14.2), (10.6, 13)], "#3a2448")
    for y in (14.5, 19.5, 24.5):
        a.ellipse(15.5, y, 0.55, 0.55, "#c8a040")
    # left arm hanging
    a.blob([(8, 13), (10.4, 13.2), (8.4, 22.6), (6.0, 22.6)], "#5a3a6a")
    a.ellipse(7.0, 23.4, 1.1, 0.8, "#e0d4cc")
    # right arm raised forward, fingers spread
    a.blob([(19, 13.2), (22, 12.2), (27.2, 11), (27.4, 13.6), (22.4, 15.4)], "#8a7a3a")
    a.rect(26.2, 10.6, 26.6, 14, "#3a2448", alpha=0.5)
    a.stroke([(23.5, 12.4), (23.5, 14.6)], 0.35, "#e8e0c8")
    a.ellipse(28.4, 11.4, 1.6, 1.3, "#e0d4cc")
    for x, y in ((27.4, 8.6), (29.0, 8.4), (30.8, 9.0), (30.8, 10.6)):
        a.stroke([(28.4, 11), (x, y)], 0.55, "#e0d4cc")
    # head
    a.ellipse(15.2, 6.6, 5, 5.1, ("lin", "#f0e6de", "#c8b8ae", 0))
    a.ellipse(15.0, 9.8, 3.6, 1.2, "#b4a49c", clip=True, alpha=0.6)
    # hair: swept ash-blue, uneven (fringe covers the back of the head)
    hair = ("lin", "#5a6c90", "#2e3a54", 90)
    a.blob([(9.0, 4.0), (10.4, 0.9), (14.5, -0.1), (18.6, 0.6), (21.2, 2.0), (19.4, 3.4), (17.6, 3.0),
            (16.6, 4.6), (14.4, 3.6), (12.6, 5.2), (12.4, 8.2), (11.0, 9.0), (9.6, 8.6)], hair)
    a.blob([(15, 0), (21.4, 2.0), (19.2, 5.2), (17.2, 3.0)], hair)
    a.stroke([(11, 1.6), (13.4, 1.0), (16.4, 1.2)], 0.4, "#8a9cc0", alpha=0.7)
    a.stroke([(10.2, 3.6), (10.2, 7.6)], 0.35, "#1e2638", alpha=0.6)
    a.rim("#ffffff", 0.35)
    # stitch scar across the face
    a.stroke([(13.4, 5.4), (16.4, 7.2), (19.6, 9.4)], 0.35, "#a0908a")
    for x, y in ((14.5, 6.0), (16.5, 7.3), (18.5, 8.6)):
        a.stroke([(x - 0.3, y + 0.6), (x + 0.3, y - 0.6)], 0.25, "#f0e8d8")
    # mismatched eyes: gold (left) / cyan (right)
    a.ellipse(14.7, 7.3, 0.95, 1.15, "#ffd040"); a.ellipse(15.1, 7.5, 0.5, 0.7, "#1a0a1a")
    spec(a, 14.4, 6.8, 0.3)
    a.ellipse(18.9, 6.7, 1.0, 1.15, "#40e0ff"); a.ellipse(19.1, 7.0, 0.5, 0.7, "#1a0a1a")
    spec(a, 18.6, 6.2, 0.3)
    # crooked grin
    a.stroke([(16.3, 10.6), (18.2, 10.9), (19.8, 9.8)], 0.45, "#1a0a1a")
    # marionette cross-bar
    a.rect(26, 22, 32, 23, ("lin", "#c8b8ae", "#8a7a70", 90), radius=0.4)
    a.outline()
    # puppet strings (glowing, un-outlined)
    for x, end in ((27.4, 22.5), (29.3, 26.5), (31.1, 19.5)):
        a.stroke([(x, 10.6), (x, end)], 0.28, "#d0b8ff")
        halo(a, x, end, 0.9, "#b090ff", 0.7)
        a.ellipse(x, end, 0.45, 0.45, "#efe4ff")
    halo(a, 19.0, 6.7, 1.4, "#40e0ff", 0.35)
    halo(a, 14.8, 7.3, 1.3, "#ffd040", 0.3)
    a.save("w5/mini")


# ───────────────────────── bosses ─────────────────────────
def boss():
    # Calamity of the Crossing: giant curse fused with wrecked cars and traffic lights.
    a = Art(48, 48)
    curse = ("lin", "#7a44a0", "#2e1448", 90)
    # stumpy legs / base mass
    a.ellipse(13, 42, 7, 5.8, "#3e1c58")
    a.ellipse(34, 42, 7, 5.8, "#3e1c58")
    a.rect(5, 44.6, 43, 48, "#2a1240", radius=1.2)
    # main mass
    a.ellipse(24, 28, 21, 16, curse)
    a.ellipse(26, 33.4, 17, 9.8, "#3e1c58", alpha=0.75)
    # drips on the base
    for x, y in ((9, 45.6), (20, 46.4), (29, 46.2), (40, 45.6)):
        a.ellipse(x, y, 1.4, 1.6, "#2a1240")
    # head mass
    a.ellipse(26, 15, 12, 10, curse)
    a.ellipse(28, 18.4, 9, 6, "#3e1c58", alpha=0.55)
    a.shade_side("#100620", 0.30, side="right", amount=0.25)
    a.rim("#d8a8ff", 0.4, width=1.0)
    for x, y, rx, ry in ((19, 8.5, 2.4, 1.0), (14, 18, 1.6, 0.9), (8, 24, 1.4, 1.0), (31.5, 7.2, 1.4, 0.6)):
        a.ellipse(x, y, rx, ry, "#9a62c8", alpha=0.7)
    # left: a car fused into the flank (hood + windshield + wheel)
    car = ("lin", "#a83a44", "#5a1e24", 90)
    a.blob([(0, 20), (12, 17), (15, 24), (13, 34), (1, 36)], car, smooth=False)
    a.poly([(1, 30), (13, 28), (13, 34), (1, 36)], "#5a1e24")
    a.stroke([(0.6, 23.6), (13.6, 21.6)], 0.5, "#c0505a", alpha=0.8)
    a.poly([(3, 19), (10, 17), (11, 22), (4, 24)], ("lin", "#5a7aa8", "#2a3a5a", 45))
    a.stroke([(4.4, 20.6), (10.4, 22.4)], 0.4, "#8a8a96")
    a.stroke([(6, 19), (8.6, 21.4), (7.0, 23.2)], 0.25, "#a8c0e0")
    a.rect(0, 27, 3, 29, "#fff0a0", radius=0.5)
    a.ellipse(5, 37, 4, 4, "#22222a")
    a.ellipse(5, 37, 1.8, 1.8, "#6a6a78")
    a.ellipse(4.6, 36.6, 0.6, 0.6, "#a0a0ac")
    # right: a car door slab jammed into the shoulder
    a.poly([(36, 14), (47, 18), (46, 30), (37, 28)], ("lin", "#8a8a98", "#44444e", 90))
    a.poly([(38, 16), (45, 19), (44, 23), (38, 21)], ("lin", "#5a7aa8", "#2a3a5a", 45))
    a.stroke([(39, 17.6), (42.6, 21.0)], 0.25, "#a8c0e0")
    a.stroke([(37.6, 26.4), (45.6, 28.2)], 0.45, "#44444e")
    a.rect(41, 24.6, 44, 25.6, "#22222a", radius=0.4)
    # traffic-light horns
    def signal(x0, y0, x1, y1, ax, ay, bx, by):
        a.stroke([(ax, ay), (bx, by)], 1.6, "#5a5a68")
        a.stroke([(ax - 0.3, ay - 0.3), (bx - 0.3, by - 0.3)], 0.5, "#8a8a98")
        a.rect(x0, y0, x1 + 1, y1 + 1, ("lin", "#34343e", "#18181e", 0), radius=0.8)
        hgt = (y1 - y0 + 1) / 3
        cx = (x0 + x1 + 1) / 2
        for i, col in enumerate(("#ff3a3a", "#ffb030", "#40e070")):
            cy = y0 + hgt * (i + 0.5)
            a.ellipse(cx, cy, 1.25, 1.1, col)
            spec(a, cx - 0.4, cy - 0.35, 0.3, 0.8)
        return cx, y0 + hgt * 0.5
    r1 = signal(13, 0, 17, 8, 19.5, 9, 16, 5)
    r2 = signal(35, 0, 39, 7, 33.5, 8, 37, 4)
    # headlight eyes
    for ex, ey in ((24.5, 13.5), (32.5, 12.5)):
        a.ellipse(ex, ey, 2.7, 2.3, ("rad", "#ffffff", "#ffe070"))
        a.ellipse(ex + 0.3, ey + 0.2, 1.0, 0.9, "#ffb030")
        spec(a, ex - 1.0, ey - 0.8, 0.5)
    a.ellipse(28.5, 10.0, 0.6, 0.6, "#c070ff"); a.ellipse(29.5, 9.0, 0.5, 0.5, "#c070ff")
    # gaping glowing mouth with teeth
    a.ellipse(28, 24, 9, 5, "#1a0610")
    a.ellipse(28, 25, 7, 3.4, ("rad", "#ffe08a", "#ff6a1a", (0.58, 0.53)))
    a.ellipse(28, 25.6, 3.6, 1.6, "#fff0b0", alpha=0.8)
    for x in range(20, 37, 3):
        a.poly([(x - 0.2, 19.4), (x + 1.3, 19.4), (x + 0.55, 22.4)], "#ece0d0")
        a.poly([(x + 0.8, 29.4), (x + 2.3, 29.4), (x + 1.55, 26.6)], "#ece0d0")
    # bumper strap across the belly
    a.poly([(14, 34), (40, 31), (40, 33.2), (14, 36.2)], ("lin", "#a0a0ac", "#55555f", 90))
    a.stroke([(14.4, 34.6), (39.6, 31.6)], 0.3, "#d0d0dc", alpha=0.8)
    a.rect(38.4, 30.6, 41, 33.6, "#44444e", radius=0.5)
    a.outline(width=1.0)
    # curse cracks + lights glowing over the top
    a.stroke([(17.5, 37.8), (19.4, 39.4), (21.4, 39.2), (23, 40.6)], 0.45, "#d090ff")
    a.stroke([(30, 37.4), (31.6, 38.6), (33.8, 38.4)], 0.45, "#d090ff")
    halo(a, 20, 39.2, 2.0, "#a050ff", 0.4)
    halo(a, 32, 38.2, 1.8, "#a050ff", 0.4)
    halo(a, 28, 25, 6.5, "#ff7a2a", 0.30)
    halo(a, 24.5, 13.5, 3.6, "#fff0a0", 0.45)
    halo(a, 32.5, 12.5, 3.6, "#fff0a0", 0.45)
    halo(a, 1.5, 28, 2.6, "#fff0a0", 0.5)
    halo(a, r1[0], r1[1], 2.2, "#ff3a3a", 0.45)
    halo(a, r2[0], r2[1], 2.2, "#ff3a3a", 0.45)
    a.save("w5/boss")


def final():
    # The Shatterline Sovereign: four-armed curse monarch, glass crown, skull shrine behind.
    a = Art(48, 48)
    stone = ("lin", "#5a546a", "#2e2a38", 0)
    bone, void = "#d8ccb8", "#140812"
    # domain glow inside the shrine
    a.rect(7.5, 9, 40.5, 48, ("rad", "#5a2a90", "#1e0e32", (0.5, 0.42)))
    a.ellipse(24, 22.5, 14.5, 14.5, "#4a2270", clip=True, alpha=0.6)
    a.ellipse(24, 22.5, 12.2, 12.2, ("rad", "#3a1a5a", "#2a1440"), clip=True)
    # pillars + lintel
    for x0 in (2, 40):
        a.rect(x0, 8, x0 + 6, 48, stone)
        a.rect(x0 + 4.4, 8, x0 + 6, 48, "#24202c", alpha=0.7)
        a.rect(x0 - 0.4, 45.6, x0 + 6.4, 48, "#2e2a38", radius=0.4)
    a.poly([(0, 4), (4, 0.8), (44, 0.8), (48, 4)], "#2e2a38")
    a.rect(0, 3.6, 48, 9, ("lin", "#5e586e", "#3a3646", 90))
    a.rect(0, 8.2, 48, 10, "#2a2634")
    a.stroke([(0.4, 4.2), (47.6, 4.2)], 0.35, "#7a748a", alpha=0.8)

    def skull(cx, cy, col=bone):
        a.ellipse(cx, cy, 2.0, 1.8, col)
        a.rect(cx - 1.1, cy + 0.8, cx + 1.1, cy + 2.0, col, radius=0.4)
        a.ellipse(cx - 0.75, cy + 0.2, 0.55, 0.6, void)
        a.ellipse(cx + 0.75, cy + 0.2, 0.55, 0.6, void)
        a.stroke([(cx - 0.4, cy + 1.6), (cx + 0.4, cy + 1.6)], 0.2, "#7a6e5a")
        spec(a, cx - 1.0, cy - 0.8, 0.3, 0.6)
    for x in range(3, 46, 6):
        skull(x + 1.5, 5.4)
    for y in range(12, 46, 6):
        for x in (3, 41):
            skull(x + 1.5, y + 1, bone if y % 12 else "#a0947e")
    # upper arms (behind the robe): raised outward, open palms
    skin = ("lin", "#c8b8d8", "#8a789c", 0)
    a.stroke([(16, 21), (10.5, 15.5), (8.4, 9.6)], 2.3, skin)
    a.stroke([(32, 21), (37.5, 15.5), (39.6, 9.6)], 2.3, "#8a789c")
    for hx, col in ((8.6, "#c8b8d8"), (39.4, "#8a789c")):
        a.ellipse(hx, 8.0, 1.8, 1.5, col)
        for dx in (-1.4, 0, 1.4):
            a.stroke([(hx + dx * 0.6, 7.6), (hx + dx, 5.6)], 0.6, col)
    a.stroke([(9.8, 13.2), (12.2, 14.4)], 0.5, "#2a1030"); a.stroke([(8.0, 11.0), (10.2, 11.0)], 0.5, "#2a1030")
    a.stroke([(35.8, 14.4), (38.2, 13.2)], 0.5, "#2a1030"); a.stroke([(37.8, 11.0), (40.0, 11.0)], 0.5, "#2a1030")
    # robe
    a.blob([(16, 20), (32, 20), (35.6, 33), (38.6, 46.4), (24, 46.8), (9.4, 46.4), (12.4, 33)],
           ("lin", "#4a3a80", "#2a1e52", 90), smooth=False)
    a.poly([(25, 20), (32, 20), (38.6, 46.4), (27, 46.8)], "#241a46", clip=True, alpha=0.75)
    a.rect(23, 20, 25, 47, ("lin", "#f0cc60", "#a07a28", 0))
    a.rect(9.6, 45, 38.4, 47, "#d0a840")
    a.rect(9, 47, 39, 48, "#8a6a24")
    # sash with a shattered-glass ornament
    a.rect(15.4, 30, 32.6, 32, ("lin", "#a88430", "#6a4e1c", 90))
    a.poly([(24, 29.2), (25.4, 31), (24, 32.8), (22.6, 31)], "#bff4ff")
    a.poly([(24, 29.2), (25.4, 31), (24, 31)], "#5ac0e0")
    # mantle
    a.ellipse(24, 21.4, 10.4, 3.2, ("lin", "#3a2a6a", "#1e1640", 90))
    a.stroke([(14.2, 20.4), (24, 19.8), (33.8, 20.4)], 0.8, "#d0a840")
    # lower arms: forward, one with a curse orb
    a.stroke([(17, 24), (14.4, 29.0), (12.8, 31.0)], 2.0, "#c8b8d8")
    a.ellipse(12.6, 31.4, 1.8, 1.5, "#c8b8d8")
    a.stroke([(31, 24), (33.6, 28.0), (35.0, 29.6)], 2.0, "#8a789c")
    a.stroke([(14.2, 26.6), (16.2, 25.6)], 0.5, "#2a1030"); a.stroke([(32.6, 26.2), (34.0, 27.4)], 0.5, "#2a1030")
    # neck + porcelain mask face
    a.rect(22, 17, 27, 21, "#8a789c")
    a.ellipse(24.2, 13, 5, 5.5, ("lin", "#d8cce4", "#9a88ac", 0))
    a.ellipse(26.6, 13.6, 2.6, 4.6, "#8a789c", clip=True, alpha=0.55)
    a.stroke([(21.5, 9.5), (22.4, 13), (23.4, 17.4)], 0.35, "#2a1030")
    a.stroke([(26.4, 15.4), (28.4, 18.2)], 0.35, "#2a1030")
    # glowing eyes
    for ex in (22.0, 27.0):
        a.ellipse(ex, 12.5, 1.1, 0.6, "#ff3ac0")
        a.ellipse(ex, 12.5, 0.45, 0.35, "#ffd0f0")
    a.stroke([(22.6, 16.4), (25.8, 16.6)], 0.45, void)
    # crown of shattered glass shards
    a.rect(18.8, 7, 30.2, 9, ("lin", "#8ae0f8", "#3a90b8", 90), radius=0.4)
    for x0, hgt in ((19, 3), (21, 6), (23, 4), (24, 7), (26, 5), (28, 3), (29.5, 2.4)):
        tip = x0 + 0.5 + (0.9 if x0 > 24 else -0.9 if x0 < 23 else 0)
        a.poly([(x0 - 0.2, 7.4), (x0 + 1.2, 7.4), (tip, 7.4 - hgt)], ("lin", "#ffffff", "#7ad0f0", 90))
    a.ellipse(24.2, 5.4, 0.45, 0.45, "#8a5ae0")
    a.outline()
    # orb + domain sparkles + glow (un-outlined)
    halo(a, 36.4, 31.4, 4.0, "#b050ff", 0.55)
    a.ellipse(36.4, 31.4, 3, 3, ("rad", "#ffd0f0", "#a040f0", (0.76, 0.66)))
    spec(a, 35.4, 30.4, 0.6)
    halo(a, 22.0, 12.5, 1.8, "#ff3ac0", 0.55)
    halo(a, 27.0, 12.5, 1.8, "#ff3ac0", 0.55)
    halo(a, 24.4, 4.6, 4.5, "#bff4ff", 0.25)
    rng = random.Random(57)
    for _ in range(14):
        x, y = rng.uniform(9, 39), rng.uniform(11, 44)
        if 13 < x < 35 and 18 < y < 46:
            continue
        a.ellipse(x, y, 0.35, 0.35, "#d090ff")
        halo(a, x, y, 0.8, "#b050ff", 0.5)
    a.save("w5/final")


# ───────────────────────── relic icons (16x16) ─────────────────────────
def relic_w1():
    # Moonveil Bell
    a = Art(16, 16)
    a.stroke([(6.6, 2.6), (7, 1.2), (8, 0.8), (9, 1.2), (9.4, 2.6)], 0.8, "#d8434a")
    a.blob([(5.2, 4.6), (8, 1.9), (10.8, 4.6), (11.6, 8), (13, 10.2), (13, 11.9), (3, 11.9), (3, 10.2),
            (4.4, 8)], ("lin", "#f4f6fa", "#8a94a8", 0))
    a.rect(2.6, 10.0, 13.4, 12.0, ("lin", "#b8c0d0", "#6a7488", 90), radius=0.9)
    a.ellipse(8, 12.6, 1.4, 1.3, "#626c84")
    a.ellipse(5.6, 6.0, 0.6, 1.4, "#ffffff", alpha=0.9)
    a.ellipse(4.5, 8.8, 0.4, 0.6, "#ffffff", alpha=0.7)
    a.stroke([(11.2, 11.4), (12.4, 12.6)], 0.5, "#d8434a")
    # crescent charm
    m = Art(16, 16)
    a.ellipse(13.0, 13.3, 2.3, 2.3, ("lin", "#fff0b0", "#d0a040", 90))
    a.img = _cut_circle(a.img, 14.1, 12.6, 1.9)
    a.outline()
    halo(a, 13.0, 13.6, 1.8, "#f0d080", 0.35)
    _ = m
    a.save("icons/relic_w1")


def _cut_circle(img, cx, cy, r):
    """Erase a circle (alpha) — used for crescents."""
    al = img.getchannel("A")
    m = Image.new("L", img.size, 0)
    ImageDraw.Draw(m).ellipse([(cx - r) * SS, (cy - r) * SS, (cx + r) * SS, (cy + r) * SS], fill=255)
    from PIL import ImageChops
    al = ImageChops.subtract(al, m)
    img = img.copy()
    img.putalpha(al)
    return img


def relic_w2():
    # Ember Leaf Headband
    a = Art(16, 16)
    cloth = ("lin", "#4a4258", "#241e2e", 90)
    a.poly([(1, 9), (0.5, 15.5), (3, 15), (3.5, 10)], "#241e2e")
    a.poly([(2.5, 9.5), (5, 14.5), (7, 13.5), (4.5, 9.5)], "#3a3448")
    a.blob([(0.5, 6.5), (8, 5.2), (15.5, 6.5), (15.5, 10.5), (8, 9.2), (0.5, 10.5)], cloth, smooth=False)
    a.stroke([(1, 6.6), (8, 5.6), (15, 6.6)], 0.4, "#6a5e80", alpha=0.8)
    # metal plate
    a.rect(5, 4, 12, 11, ("lin", "#e0e6ee", "#6e7684", 45), radius=0.7)
    a.rect(5.4, 4.3, 11.6, 5.0, "#ffffff", alpha=0.6, radius=0.3)
    for x, y in ((6, 5.2), (11, 5.2), (6, 9.8), (11, 9.8)):
        a.ellipse(x, y, 0.35, 0.35, "#4a4e58")
    # engraved flame-leaf
    a.blob([(8.5, 4.8), (10.4, 7.4), (9.6, 9.6), (7.4, 9.6), (6.6, 7.4)], ("lin", "#ffd060", "#ff5a1a", 90))
    a.blob([(8.5, 6.4), (9.3, 8.0), (8.5, 9.2), (7.7, 8.0)], "#fff0a0", alpha=0.9)
    a.stroke([(8.5, 6.2), (8.5, 9.6)], 0.2, "#c0400a", alpha=0.8)
    a.outline()
    halo(a, 8.5, 7.6, 2.0, "#ff7a2a", 0.3)
    a.save("icons/relic_w2")


def relic_w3():
    # Pale Mask Shard
    a = Art(16, 16)
    a.blob([(3, 2), (10, 1), (14, 4), (13, 9), (11, 14), (6, 15), (2, 10), (1, 5)],
           ("lin", "#f4f0e4", "#b3ad9a", 0))
    a.blob([(10, 1), (14, 4), (13, 9), (11, 14), (9.4, 13), (11, 8)], "#b3ad9a", alpha=0.8, clip=True)
    a.ellipse(4.2, 4.0, 1.4, 0.8, "#ffffff", alpha=0.7)
    a.blob([(4, 5), (8, 5), (8.4, 8.4), (5, 8.6)], "#14121c")
    a.ellipse(6.6, 6.8, 0.6, 0.6, "#5ab4ff")
    a.stroke([(3.4, 11.4), (6.4, 12.6)], 0.35, "#7e7968")
    # glowing crack
    a.stroke([(10.5, 1.2), (9.5, 5.5), (11.5, 8.5), (8.5, 11.5), (9.5, 15.2)], 0.7, "#5ab4ff", smooth=False)
    a.stroke([(10.5, 1.2), (9.5, 5.5), (11.5, 8.5), (8.5, 11.5), (9.5, 15.2)], 0.3, "#dff4ff", smooth=False)
    a.outline()
    halo(a, 10, 8, 3, "#5ab4ff", 0.2)
    halo(a, 6.6, 6.8, 1.2, "#5ab4ff", 0.5)
    a.save("icons/relic_w3")


def relic_w4():
    # Ward 13 Key
    a = Art(16, 16)
    gold = ("lin", "#ffe490", "#a07020", 45)
    a.stroke([(5.5, 6.5), (7.2, 5.0), (8.8, 4.2)], 0.4, "#5a4a3a")
    a.stroke([(7.2, 11.6), (13.4, 14.6)], 1.3, gold)
    a.rect(10.6, 14.2, 11.6, 16, "#d0a040", radius=0.3)
    a.rect(12.0, 14.6, 13.0, 16, "#d0a040", radius=0.3)
    a.ellipse(4.5, 9.5, 3.4, 3.4, gold)
    a.img = _cut_circle(a.img, 4.5, 9.5, 1.6)
    a.stroke([(2.2, 9.0), (3.0, 7.4), (4.4, 6.6)], 0.5, "#fff4c0", alpha=0.85)
    # paper tag "13"
    a.rect(8, 0, 15, 7, ("lin", "#ece2c8", "#b8ac90", 90), radius=0.5)
    a.rect(8, 6.2, 15, 7, "#a89c80", radius=0.4)
    a.stroke([(9.5, 2.0), (10.4, 1.3), (10.4, 5.4)], 0.6, "#8a2020", smooth=False)
    a.stroke([(11.9, 1.4), (13.4, 1.4), (12.6, 3.2), (13.6, 3.9), (13.2, 5.3), (11.9, 5.4)], 0.55, "#8a2020",
             smooth=False)
    a.ellipse(8.9, 0.9, 0.3, 0.3, "#5a4a3a")
    a.outline()
    a.save("icons/relic_w4")


def relic_w5():
    # Shattered Seal: cursed talisman disc with a cross sigil, split by a crack.
    a = Art(16, 16)
    a.ellipse(8, 8, 7.3, 7.3, ("lin", "#7a2ac0", "#3a0e70", 90))
    a.ellipse(8, 8, 6.6, 6.6, ("lin", "#fbf2d8", "#b0a080", 45))
    a.ellipse(8, 8, 4.4, 4.4, ("lin", "#9a40e0", "#5a1a9a", 90))
    a.ellipse(8, 8, 3.5, 3.5, ("lin", "#f4ead0", "#c8b890", 45))
    a.rect(7, 4.6, 9, 11.4, "#7a2ac0", radius=0.3)
    a.rect(4.6, 7.0, 11.4, 9.0, "#7a2ac0", radius=0.3)
    a.ellipse(8, 8, 0.8, 0.8, "#c070ff")
    a.ellipse(4.9, 3.6, 1.4, 0.7, "#ffffff", alpha=0.55)
    for x, y in ((5.5, 5.5), (10.5, 10.5), (10.5, 5.5), (5.5, 10.5)):
        a.ellipse(x, y, 0.55, 0.55, "#c070ff")
    a.outline()
    # crack through the seal (drawn over the outline so it splits the rim)
    crack = [(10.6, 0.6), (7.5, 7.5), (9.5, 10.5), (6.2, 15.4)]
    a.stroke(crack, 0.75, "#2a1030", smooth=False)
    a.stroke(crack, 0.25, "#d890ff", smooth=False)
    for x, y in ((11.5, 2.5), (5.5, 14.5)):
        halo(a, x, y, 1.1, "#c070ff", 0.7)
        a.ellipse(x, y, 0.35, 0.35, "#f0d8ff")
    halo(a, 8, 8, 2.2, "#c070ff", 0.3)
    a.save("icons/relic_w5")


def main():
    for f in (fodder_a, fodder_b, ranged, blinker, elite, mini, boss, final,
              relic_w1, relic_w2, relic_w3, relic_w4, relic_w5):
        f()
    print("wrote W5 enemies/bosses + relic icons to art/hd/")


if __name__ == "__main__":
    main()
