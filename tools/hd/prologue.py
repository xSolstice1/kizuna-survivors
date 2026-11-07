"""HD redraws for the prologue group: yokai enemies, the Night Lord, chest, shrine, pickups.

    python3 tools/hd/prologue.py

Writes art/hd/<name>.png for each sprite (4x the .px art size). Deterministic.
"""
import sys
import pathlib

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from PIL import ImageDraw, ImageFilter  # noqa: E402
from hdkit import Art, SS, preview  # noqa: E402

K = "#1a1423"


# ── helpers ──
def mx(pts, w):
    """Mirror points across the vertical centre line of a canvas of width w."""
    return [(w - x, y) for x, y in pts]


def tr(pts, dx, dy):
    return [(x + dx, y + dy) for x, y in pts]


def sc(pts, cx, cy, s, sy=None):
    sy = s if sy is None else sy
    return [(cx + (x - cx) * s, cy + (y - cy) * sy) for x, y in pts]


def spot(a, cx, cy, r, color, alpha=0.6, blur=0.6, clip=False, ry=None):
    """A soft blurred ellipse (light spots, glows, soft shadows)."""
    ry = r if ry is None else ry
    m = a._mask()
    d = ImageDraw.Draw(m)
    d.ellipse([(cx - r) * SS, (cy - ry) * SS, (cx + r) * SS, (cy + ry) * SS], fill=255)
    m = m.filter(ImageFilter.GaussianBlur(blur * SS))
    a._paint(m, color, clip=clip, alpha=alpha)


def glow(a, color, radius=1.4, alpha=0.4):
    """hdkit-style soft glow behind the art, faded out before the canvas edge (no boxy cut-off)."""
    from PIL import Image
    from hdkit import rgb
    g = a.img.getchannel("A").filter(ImageFilter.GaussianBlur(radius * SS))
    edge = Image.new("L", (a.W, a.H), 0)
    m = int(radius * SS * 0.9)
    ImageDraw.Draw(edge).rectangle([m, m, a.W - m, a.H - m], fill=255)
    edge = edge.filter(ImageFilter.GaussianBlur(m * 0.6))
    g = Image.composite(g, Image.new("L", g.size, 0), edge).point(lambda v: int(v * alpha))
    under = Image.new("RGBA", (a.W, a.H), rgb(color))
    under.putalpha(g)
    under.alpha_composite(a.img)
    a.img = under


def shine(a, x, y, r=0.45, alpha=0.9):
    a.ellipse(x, y, r, r, "#ffffff", clip=True, alpha=alpha)


def eye_anime(a, cx, cy, rx, ry, iris, pupil="#1a1423", white="#ffffff"):
    a.ellipse(cx, cy, rx, ry, white)
    a.ellipse(cx, cy + ry * 0.1, rx * 0.72, ry * 0.85, iris)
    a.ellipse(cx, cy + ry * 0.15, rx * 0.36, ry * 0.5, pupil)
    shine(a, cx - rx * 0.3, cy - ry * 0.35, min(rx, ry) * 0.28)


# ── wisp 16x16 ──
def wisp():
    a = Art(16, 16)
    body = [(8, 0.9), (10.6, 2.6), (13.1, 5.2), (14.1, 8.2), (13.6, 11.0), (12.4, 12.9),
            (10.9, 12.5), (9.9, 14.9), (8.0, 13.2), (6.1, 14.9), (5.1, 12.5), (3.6, 12.9),
            (2.4, 11.0), (1.9, 8.2), (2.9, 5.2), (5.4, 2.6)]
    a.blob(body, ("lin", "#7a52b0", "#3a2463", 80))
    # lit core, cel style
    a.blob(sc(tr(body, -0.6, -0.8), 8, 7, 0.78), ("rad", "#a57de0", "#5b3b8c", (0.4, 0.3)), clip=True)
    # flame swirl highlight
    a.stroke([(6.0, 3.2), (4.6, 5.0), (4.0, 7.2)], 0.9, "#c9a8f2", clip=True, alpha=0.8, taper=True)
    a.stroke([(9.2, 2.2), (8.4, 3.6)], 0.6, "#c9a8f2", clip=True, alpha=0.6)
    # tail shadow
    a.blob([(2, 10), (14, 10), (14, 16), (2, 16)], "#2a1850", clip=True, alpha=0.45)
    # angry glowing eyes
    for side in (0, 1):
        e = [(4.7, 6.1), (7.1, 6.9), (6.8, 8.3), (5.1, 7.9)]
        e = mx(e, 16) if side else e
        cx = 10.0 if side else 6.0
        spot(a, cx, 7.3, 1.6, "#ffe066", alpha=0.45, blur=0.7, clip=True)
        a.blob(e, "#1a1423", smooth=False)
        a.blob(sc(e, cx, 7.3, 0.72), "#ffe066")
        shine(a, cx - 0.4, 7.0, 0.3)
    a.rim("#e6d4ff", 0.35)
    a.outline()
    glow(a, "#9b6fd6", radius=1.4, alpha=0.4)
    a.save("wisp")


# ── kasa-obake 16x16 ──
def kasa():
    a = Art(16, 16)
    # handle (behind canopy)
    a.stroke([(8.0, 9.0), (8.1, 12.0), (7.8, 13.8), (6.9, 14.6)], 1.3, ("lin", "#e0c088", "#9a7a44", 0))
    # tongue
    a.blob([(7.1, 8.8), (8.9, 8.8), (8.9, 10.6), (8.3, 11.8), (7.6, 11.4), (7.1, 10.4)], ("lin", "#ff9ab8", "#e0507a", 90))
    a.stroke([(8.0, 9.3), (8.0, 10.8)], 0.3, "#c03a64", alpha=0.7)
    canopy = [(8.0, 0.6), (10.6, 1.6), (12.8, 3.6), (14.3, 6.2), (15.1, 8.9),
              (13.4, 9.3), (11.9, 8.8), (10.2, 9.5), (8.0, 9.0), (5.8, 9.5), (4.1, 8.8),
              (2.6, 9.3), (0.9, 8.9), (1.7, 6.2), (3.2, 3.6), (5.4, 1.6)]
    a.blob(canopy, ("lin", "#e0544c", "#a82c34", 90))
    # cel shadow panels (alternate ribs darker)
    a.blob([(8, 0.6), (15.4, 9.4), (12.0, 9.4)], "#8a2430", clip=True, alpha=0.45)
    a.blob([(8, 0.6), (4.0, 9.4), (1.0, 9.4)], "#8a2430", clip=True, alpha=0.3)
    # ribs
    for end in ((2.6, 9.0), (5.8, 9.3), (10.2, 9.3), (13.4, 9.0)):
        a.stroke([(8.0, 0.8), ((8.0 + end[0]) / 2 + (0.4 if end[0] < 8 else -0.4), 5.0), end], 0.35, "#7a1c28", clip=True, alpha=0.7)
    # dark hem
    a.blob([(0, 8.0), (16, 8.0), (16, 10), (0, 10)], "#8a2430", clip=True, alpha=0.7)
    # gloss highlight
    a.stroke([(5.6, 2.2), (3.8, 4.2), (2.8, 6.4)], 0.8, "#ff9a8a", clip=True, alpha=0.7, taper=True)
    # yellow paper patches
    a.ellipse(6.2, 3.6, 0.65, 0.6, "#f2d16b")
    a.ellipse(9.8, 3.6, 0.65, 0.6, "#f2d16b")
    # the single big eye
    a.ellipse(7.9, 6.4, 2.6, 1.7, "#1a1423")
    eye_anime(a, 7.9, 6.4, 2.1, 1.3, "#c23b3b")
    a.rim("#ffc0b0", 0.3)
    a.outline()
    a.save("kasa")


# ── chochin-obake 16x20 ──
def chochin():
    a = Art(16, 20)
    # top cap and hanger
    a.rect(5.2, 0.4, 10.8, 2.4, ("lin", "#4a3030", "#2a1a1a", 90), radius=0.5)
    a.rect(2.6, 1.8, 13.4, 4.6, ("lin", "#4a3030", "#1f1414", 90), radius=0.9)
    # bottom cap
    a.rect(3.8, 16.2, 12.2, 19.2, ("lin", "#3a2424", "#1f1414", 90), radius=0.9)
    body = [(8, 4.0), (12.6, 4.3), (14.9, 6.8), (15.3, 10.5), (14.8, 14.0), (12.4, 16.4),
            (8, 16.8), (3.6, 16.4), (1.2, 14.0), (0.7, 10.5), (1.1, 6.8), (3.4, 4.3)]
    a.blob(body, ("rad", "#ffb27a", "#d8532e", (0.48, 0.5)))
    # inner light
    spot(a, 8, 10.5, 5.0, "#ffd27a", alpha=0.35, blur=1.6, clip=True)
    # ribs
    for y in (5.2, 8.2, 11.2, 14.1):
        bow = 0.5
        a.stroke([(0, y - bow), (8, y + bow * 0.6), (16, y - bow)], 0.55, "#a8402a", clip=True, alpha=0.75)
    a.shade_side("#5a1a10", 0.3, side="right", amount=0.3)
    # gloss
    a.stroke([(3.2, 6.4), (2.4, 9.0), (2.6, 11.6)], 0.8, "#ffd8b0", clip=True, alpha=0.7, taper=True)
    # big eye
    a.ellipse(8, 9.0, 3.4, 2.9, "#1a1423")
    a.ellipse(8, 9.0, 2.9, 2.45, ("lin", "#ffffff", "#f0e6e0", 90))
    a.ellipse(8, 9.3, 0.95, 1.7, "#2a1a1a")
    shine(a, 6.9, 8.0, 0.5)
    # mouth + tongue
    a.blob([(5.8, 12.6), (10.2, 12.6), (9.6, 13.7), (6.4, 13.7)], "#2a1a1a")
    a.blob([(7.0, 13.2), (9.0, 13.2), (9.0, 15.0), (8.5, 16.6), (7.6, 16.4), (7.0, 15.0)], ("lin", "#ff9ab8", "#e0507a", 90))
    a.stroke([(8.0, 13.6), (8.0, 15.4)], 0.3, "#c03a64", alpha=0.7)
    a.rim("#ffe0c0", 0.3)
    a.outline()
    glow(a, "#ffa060", radius=1.6, alpha=0.35)
    a.save("chochin")


# ── nopperabo 16x24 ──
def nopperabo():
    a = Art(16, 24)
    # kimono body
    body = [(5.0, 10.8), (11.0, 10.8), (14.0, 11.8), (15.1, 13.2), (15.1, 19.0), (13.8, 20.2),
            (13.2, 23.1), (2.8, 23.1), (2.2, 20.2), (0.9, 19.0), (0.9, 13.2), (2.0, 11.8)]
    a.blob(body, ("lin", "#4a3a72", "#2e2250", 90), smooth=False)
    a.blob(body, ("lin", "#4a3a72", "#2e2250", 90))
    # sleeve folds
    a.stroke([(3.8, 13.0), (3.6, 17.0), (2.6, 19.4)], 0.4, "#221a3c", clip=True, alpha=0.8)
    a.stroke([(12.2, 13.0), (12.4, 17.0), (13.4, 19.4)], 0.4, "#221a3c", clip=True, alpha=0.8)
    a.shade_side("#120c24", 0.35, side="right", amount=0.3)
    a.blob([(0, 21), (16, 21), (16, 24), (0, 24)], "#1c1434", clip=True, alpha=0.5)
    # centre fold and collar
    a.stroke([(8.0, 12.0), (8.0, 23.0)], 0.7, "#2a1d44", clip=True)
    a.blob([(5.0, 10.6), (8.0, 14.6), (11.0, 10.6), (10.0, 10.6), (8.0, 13.0), (6.0, 10.6)], "#d8d0e6", smooth=False)
    a.stroke([(5.4, 10.8), (8.0, 14.6), (10.6, 10.8)], 0.5, "#6a5a9a", alpha=0.9, smooth=False)
    # obi
    a.rect(3.8, 15.0, 12.2, 17.1, ("lin", "#e0c088", "#a8864e", 90), radius=0.3)
    a.stroke([(3.8, 16.05), (12.2, 16.05)], 0.25, "#8a6a34", alpha=0.6)
    a.rect(7.2, 15.2, 8.8, 16.9, "#8a3a4a", radius=0.3)
    # neck
    a.rect(6.2, 8.8, 9.8, 11.4, "#d4b8a0", radius=0.6)
    # hair back
    a.ellipse(8, 4.6, 4.9, 4.1, ("lin", "#3e2a44", "#1e1222", 90))
    # blank face
    a.ellipse(8, 6.4, 3.9, 3.9, ("rad", "#fbeadc", "#e6c8b0", (0.42, 0.24)))
    spot(a, 8, 10.0, 3.0, "#c4a088", alpha=0.45, blur=0.8, clip=True, ry=1.1)
    # fringe over the top of the face
    a.blob([(3.4, 5.6), (4.0, 2.6), (8.0, 1.0), (12.0, 2.6), (12.6, 5.6), (11.6, 4.6),
            (10.0, 3.4), (8.6, 3.0), (8.0, 3.6), (7.4, 3.0), (6.0, 3.4), (4.4, 4.6)], ("lin", "#4a3450", "#2a1a2e", 90))
    a.stroke([(5.0, 2.4), (6.6, 1.6)], 0.5, "#8a6a90", clip=True, alpha=0.6)
    # eerie sheen where a face should be
    spot(a, 6.6, 6.0, 1.2, "#ffffff", alpha=0.35, blur=0.6, clip=True)
    a.rim("#c8b8ff", 0.25)
    a.outline()
    a.save("nopperabo")


# ── oni 32x32 ──
def oni():
    W = 32
    a = Art(32, 32)
    red = ("lin", "#e6544a", "#b8302c", 90)

    def both(fn, pts, *args, **kw):
        fn(pts, *args, **kw)
        fn(mx(pts, W), *args, **kw)

    # horns
    horn = [(4.4, 6.6), (4.2, 3.4), (4.8, 0.6), (5.9, 2.6), (7.3, 5.6)]
    both(a.blob, horn, ("lin", "#fff4d0", "#c9b37a", 0))
    both(a.blob, [(5.6, 4.2), (7.6, 4.2), (7.6, 7), (5.6, 7)], "#a8925a", clip=True, alpha=0.5)
    # legs and feet
    for flip in (False, True):
        leg = [(6.0, 26.0), (10.4, 26.0), (10.4, 29.6), (6.0, 29.6)]
        foot = [(4.6, 28.6), (11.0, 28.6), (11.2, 30.4), (4.6, 30.4)]
        if flip:
            leg, foot = mx(leg, W), mx(foot, W)
        xs = [p[0] for p in leg]
        a.rect(min(xs), 25.5, max(xs), 29.8, ("lin", "#c23a32", "#8a2424", 0 if not flip else 180), radius=0.8)
        fx = [p[0] for p in foot]
        a.rect(min(fx), 28.6, max(fx), 30.5, "#9c2a2a", radius=0.9)
    # torso
    torso = [(7.0, 15.5), (16, 15.0), (25.0, 15.5), (26.0, 20.0), (25.6, 26.5), (16, 27.4),
             (6.4, 26.5), (6.0, 20.0)]
    a.blob(torso, red)
    # pecs / belly
    a.stroke([(10.0, 19.6), (13.0, 20.6), (15.6, 19.8)], 0.45, "#9c2a2a", clip=True, alpha=0.7)
    a.stroke([(16.4, 19.8), (19.0, 20.6), (22.0, 19.6)], 0.45, "#9c2a2a", clip=True, alpha=0.7)
    a.stroke([(16, 20.8), (16, 23.0)], 0.4, "#9c2a2a", clip=True, alpha=0.5)
    # arms
    arm = [(7.6, 16.0), (4.4, 16.4), (2.2, 18.6), (1.2, 22.0), (1.4, 24.4), (2.4, 26.0),
           (4.4, 26.0), (5.6, 24.4), (5.8, 21.6), (7.2, 19.2)]
    both(a.blob, arm, red)
    # fists
    for flip in (False, True):
        cx = 3.4 if not flip else W - 3.4
        a.ellipse(cx, 24.6, 2.3, 1.9, ("lin", "#d84a40", "#9c2a2a", 90))
        a.stroke([(cx - 1.2, 24.4), (cx + 1.2, 24.4)], 0.3, "#7a1c1c", clip=True, alpha=0.6)
    # arm highlights (O)
    a.stroke([(4.4, 17.6), (3.0, 19.8), (2.6, 22.4)], 0.9, "#f07a5a", clip=True, alpha=0.9, taper=True)
    a.stroke([(27.6, 17.6), (29.0, 19.8), (29.4, 22.4)], 0.6, "#f07a5a", clip=True, alpha=0.5, taper=True)
    # tiger-skin loincloth
    cloth = [(6.6, 22.8), (25.4, 22.8), (25.6, 26.4), (24.0, 27.0), (22.6, 26.2), (20.6, 27.0),
             (18.6, 26.2), (16.0, 27.1), (13.4, 26.2), (11.4, 27.0), (9.4, 26.2), (8.0, 27.0), (6.4, 26.4)]
    a.blob(cloth, ("lin", "#ffc848", "#e09a20", 90), smooth=False)
    for x in (8.4, 12.6, 16.8, 21.0, 24.4):
        a.blob([(x, 23.0), (x + 1.0, 23.0), (x + 0.2, 25.6)], "#3a2014")
    a.stroke([(6.6, 23.1), (25.4, 23.1)], 0.6, "#fff0a0", clip=True, alpha=0.6)
    # head
    head = [(16, 3.4), (21.6, 3.6), (25.4, 5.6), (26.9, 9.2), (26.6, 13.0), (24.0, 15.8),
            (16, 16.6), (8.0, 15.8), (5.4, 13.0), (5.1, 9.2), (6.6, 5.6), (10.4, 3.6)]
    a.blob(head, ("lin", "#ee5c4c", "#c43a32", 90))
    spot(a, 24.6, 11.6, 2.4, "#9c2a2a", alpha=0.5, blur=1.0, clip=True)
    spot(a, 7.4, 12.0, 1.4, "#ff8a7a", alpha=0.45, blur=0.6, clip=True)
    spot(a, 24.6, 12.0, 1.4, "#ff8a7a", alpha=0.35, blur=0.6, clip=True)
    # hair
    hair = [(6.0, 7.4), (6.6, 4.6), (10.4, 3.0), (16, 2.6), (21.6, 3.0), (25.4, 4.6), (26.0, 7.4),
            (24.6, 6.8), (23.4, 8.2), (21.4, 6.6), (19.4, 8.0), (16.0, 6.4), (12.6, 8.0),
            (10.6, 6.6), (8.6, 8.2), (7.4, 6.8)]
    a.blob(hair, ("lin", "#4a3044", "#2a1a2e", 90), smooth=False)
    a.blob(hair, ("lin", "#4a3044", "#2a1a2e", 90))
    a.stroke([(10.0, 4.4), (13.4, 3.6)], 0.6, "#7a5a74", clip=True, alpha=0.7)
    # eyes: brows + eyes
    for flip in (False, True):
        brow = [(7.0, 8.6), (11.4, 9.4), (11.2, 10.2), (7.2, 9.6)]
        if flip:
            brow = mx(brow, W)
        a.blob(brow, "#1a1423", smooth=False)
        cx = 9.4 if not flip else W - 9.4
        a.ellipse(cx, 11.0, 1.5, 1.1, "#ffffff")
        a.ellipse(cx + (0.25 if not flip else -0.25), 11.1, 0.85, 0.85, "#ffd23a")
        a.ellipse(cx + (0.25 if not flip else -0.25), 11.1, 0.38, 0.55, "#1a1423")
        shine(a, cx - 0.2, 10.7, 0.25)
    # grinning mouth with lower fangs
    mouth = [(9.4, 13.0), (16, 13.4), (22.6, 13.0), (21.6, 14.9), (16, 15.4), (10.4, 14.9)]
    a.blob(mouth, "#3a1020")
    a.blob(tr(mouth, 0, 1.2), "#c03a50", clip=True, alpha=0.7)
    for x in (10.8, W - 10.8):
        a.blob([(x - 0.8, 15.0), (x + 0.8, 15.0), (x, 13.4)], "#fffaf0", smooth=False)
    a.stroke([(13, 16.1), (19, 16.1)], 0.3, "#9c2a2a", clip=True, alpha=0.5)
    a.rim("#ffb0a0", 0.35)
    a.outline()
    a.save("oni")


# ── Night Lord 48x48 ──
def night_lord():
    W = 48
    a = Art(48, 48)
    gold = ("lin", "#ffe08a", "#b08a30", 90)
    # kuwagata horns
    for flip in (False, True):
        h = [(5.6, 2.0), (7.4, 1.6), (10.6, 3.2), (14.2, 5.6), (13.4, 7.2), (9.6, 5.4), (6.0, 3.4)]
        if flip:
            h = mx(h, W)
        a.blob(h, ("lin", "#f2cf5a", "#a07a28", 90))
    a.stroke([(7.0, 2.2), (12.6, 5.4)], 0.5, "#fff2b0", clip=True, alpha=0.8)
    a.stroke([(41.0, 2.2), (35.4, 5.4)], 0.5, "#fff2b0", clip=True, alpha=0.5)
    # base (purple dais) + black cap
    a.rect(8.0, 41.6, 40.0, 47.0, ("lin", "#6e3880", "#4a2058", 90), radius=0.8)
    for i in range(8):
        x = 9.2 + i * 4
        a.rect(x, 43.0, x + 1.8, 45.8, "#3a1a4a", radius=0.4)
    a.stroke([(8.4, 42.1), (39.6, 42.1)], 0.5, "#9a5aaa", clip=True, alpha=0.7)
    a.rect(9.4, 38.8, 38.6, 42.2, ("lin", "#3a2828", "#1e1414", 90), radius=1.0)
    a.stroke([(11, 39.6), (37, 39.6)], 0.4, "#c9a040", alpha=0.8)
    # lantern body
    body = [(24, 12.6), (31.6, 13.4), (36.6, 15.8), (39.4, 19.6), (40.2, 24.5), (39.4, 30.0),
            (37.0, 34.6), (32.2, 37.8), (24, 39.2), (15.8, 37.8), (11.0, 34.6), (8.6, 30.0),
            (7.8, 24.5), (8.6, 19.6), (11.4, 15.8), (16.4, 13.4)]
    a.blob(body, ("rad", "#ffa36a", "#d04e2c", (0.5, 0.52)))
    spot(a, 24, 25, 11, "#ffd27a", alpha=0.3, blur=3.0, clip=True)
    for y in (17.2, 21.2, 25.2, 29.2, 33.2, 36.6):
        bow = 0.9
        a.stroke([(4, y - bow), (24, y + bow * 0.5), (44, y - bow)], 0.75, "#a8402a", clip=True, alpha=0.8)
    a.shade_side("#4a1008", 0.35, side="right", amount=0.28)
    a.shade_side("#4a1008", 0.15, side="left", amount=0.12)
    a.stroke([(12.4, 18.0), (10.6, 23.0), (10.8, 28.6)], 1.4, "#ffd2a8", clip=True, alpha=0.6, taper=True)
    # helmet (kabuto) dome + brim
    dome = [(12.6, 10.4), (13.6, 6.6), (18.0, 4.0), (24, 3.2), (30.0, 4.0), (34.4, 6.6), (35.4, 10.4)]
    a.blob(dome, ("lin", "#3e2a2a", "#1a1010", 90))
    for x in (16.0, 19.6, 28.4, 32.0):
        a.stroke([(x + (x - 24) * 0.05, 4.4), (x + (x - 24) * 0.12, 10.0)], 0.35, "#5a4040", clip=True, alpha=0.8)
    a.stroke([(16.4, 5.0), (19.6, 3.9)], 0.6, "#7a6060", clip=True, alpha=0.8)
    brim = [(10.8, 9.0), (24, 8.4), (37.2, 9.0), (37.0, 12.0), (32, 12.8), (24, 13.2), (16, 12.8), (11.0, 12.0)]
    a.blob(brim, ("lin", "#b08a30", "#6e5018", 90))
    a.stroke([(11.6, 9.4), (24, 8.9), (36.4, 9.4)], 0.5, "#f2cf5a", clip=True, alpha=0.8)
    # gold crest (maedate)
    a.rect(22.0, 3.6, 26.0, 9.6, gold, radius=0.5)
    a.blob([(24, 2.4), (26.4, 4.6), (24, 6.4), (21.6, 4.6)], ("lin", "#fff0a8", "#d0a840", 90))
    a.ellipse(24, 4.5, 0.8, 0.8, "#ff3a3a")
    shine(a, 23.6, 4.1, 0.3)
    # eye
    a.ellipse(24, 24.6, 8.6, 6.6, "#1a1423")
    a.ellipse(24, 24.6, 7.9, 5.9, ("rad", "#fff6c0", "#ffd060", (0.5, 0.45)))
    spot(a, 24, 24.8, 5.0, "#ff6a3a", alpha=0.6, blur=0.8, clip=True, ry=4.2)
    a.ellipse(24, 24.6, 4.6, 4.0, ("rad", "#ff6a5a", "#c41a24", (0.5, 0.5)))
    a.ellipse(24, 24.8, 1.0, 3.0, "#1a0a0a")
    shine(a, 22.0, 22.6, 0.8)
    shine(a, 26.0, 26.8, 0.35, 0.6)
    # toothy mouth
    a.rect(15.4, 31.6, 32.6, 35.2, "#1a0a0a", radius=1.0)
    for i in range(6):
        x = 16.6 + i * 2.9
        a.blob([(x, 31.7), (x + 1.4, 31.7), (x + 0.7, 33.6)], "#fffaf0", smooth=False)
    a.ellipse(24, 34.6, 4.0, 0.9, "#b0303a", clip=True, alpha=0.6)
    a.rim("#ffe0b0", 0.3, width=0.8)
    a.outline(width=1.0)
    glow(a, "#ff7a3a", radius=2.2, alpha=0.3)
    a.save("night_lord")


# ── chests ──
def _chest_body(a, y0):
    """Red lacquer box with gold bands; y0 = top of the body (latch band)."""
    a.rect(1.0, y0, 13.0, y0 + 7.0, ("lin", "#cc3c42", "#8a2230", 90), radius=0.5)
    a.rect(1.0, y0 + 5.0, 13.0, y0 + 7.0, ("lin", "#f2c14e", "#b58a2e", 90), radius=0.4)
    a.stroke([(1.4, y0 + 5.3), (12.6, y0 + 5.3)], 0.4, "#fff0a0", clip=True, alpha=0.8)
    a.rect(1.0, y0 + 4.0, 13.0, y0 + 5.0, "#7c1f2a", clip=True, alpha=0.7)
    # corner fittings
    for x in (1.0, 12.1):
        a.rect(x, y0, x + 0.9, y0 + 7.0, ("lin", "#f2c14e", "#b58a2e", 90), clip=True, alpha=0.8)
    # latch plate
    a.rect(5.4, y0 + 0.6, 8.6, y0 + 3.6, "#1a1423", radius=0.8)
    a.rect(5.9, y0 + 1.0, 8.1, y0 + 3.2, ("lin", "#ffd860", "#b58a2e", 90), radius=0.6)
    a.ellipse(7.0, y0 + 2.2, 0.4, 0.55, "#1a1423")
    shine(a, 6.3, y0 + 1.4, 0.25)
    a.stroke([(2.6, y0 + 1.0), (2.6, y0 + 3.6)], 0.4, "#e8606a", clip=True, alpha=0.6)


def _chest_lid(a, y0):
    a.rect(1.0, y0 + 0.6, 13.0, y0 + 4.0, ("lin", "#d84a50", "#9c2834", 90), radius=1.4)
    a.stroke([(2.6, y0 + 1.5), (11.4, y0 + 1.5)], 0.6, "#ff8a8a", clip=True, alpha=0.6)
    a.rect(1.0, y0 + 2.6, 13.0, y0 + 4.0, ("lin", "#f2c14e", "#b58a2e", 90), clip=True)
    a.stroke([(1.4, y0 + 2.9), (12.6, y0 + 2.9)], 0.35, "#fff0a0", clip=True, alpha=0.8)
    for x in (1.0, 12.1):
        a.rect(x, y0, x + 0.9, y0 + 4.0, ("lin", "#f2c14e", "#b58a2e", 90), clip=True, alpha=0.8)


def chest():
    a = Art(14, 12)
    _chest_body(a, 3.9)
    _chest_lid(a, 0.4)
    a.ellipse(7.0, 4.2, 1.9, 1.0, "#1a1423")
    a.rect(5.9, 3.7, 8.1, 5.2, ("lin", "#ffd860", "#b58a2e", 90), radius=0.5)
    shine(a, 6.4, 4.1, 0.25)
    a.outline()
    a.save("chest")


def chest_open():
    a = Art(14, 16)
    # treasure light spilling between lid and body
    _chest_body(a, 8.0)
    # treasure inside (gold glow + coins)
    a.rect(1.2, 7.6, 12.8, 9.0, ("lin", "#fffadc", "#ffd860", 90), radius=0.5)
    for cx in (3.4, 5.6, 8.6, 10.8):
        a.ellipse(cx, 8.1, 0.9, 0.5, "#f2c14e")
        shine(a, cx - 0.3, 7.9, 0.2)
    a.ellipse(7.0, 7.8, 0.6, 0.6, "#ffffff", alpha=0.9)
    _chest_lid(a, 0.4)
    # underside of the lid catches the light
    a.rect(1.0, 4.4, 13.0, 5.2, "#fff3b0", clip=True, alpha=0.5)
    a.outline()
    # treasure light spilling up between body and lid (no outline on light)
    spot(a, 7.0, 6.6, 5.4, "#fff3b0", alpha=0.5, blur=0.9, ry=1.2)
    glow(a, "#ffe080", radius=1.0, alpha=0.3)
    a.save("chest_open")


# ── shrine 16x18 ──
def shrine():
    a = Art(16, 18)
    red = ("lin", "#e85a4a", "#a82c2c", 0)
    # pillars
    for x in (3.0, 12.0):
        a.blob([(x - 0.15, 2.0), (x + 1.15, 2.0), (x + 1.35, 17.0), (x - 0.35, 17.0)], red, smooth=False)
        a.rect(x, 13.5, x + 1.0, 17, "#9c2a2a", clip=True, alpha=0.8)
        a.rect(x - 0.7, 16.2, x + 1.7, 17.4, "#2a1a24", radius=0.3)
    # nuki (tie beam)
    a.rect(1.8, 4.6, 14.2, 5.8, red, radius=0.3)
    a.stroke([(2.2, 4.9), (13.8, 4.9)], 0.3, "#ff9a80", clip=True, alpha=0.6)
    # kasagi (top lintel) with upswept ends
    top = [(0.4, 0.3), (4, 0.9), (8, 1.05), (12, 0.9), (15.6, 0.3), (15.2, 2.0), (8, 2.2), (0.8, 2.0)]
    a.blob(top, ("lin", "#f06a58", "#b0302e", 90))
    a.rect(0, 0, 16, 0.9, "#2a1a24", clip=True, alpha=0.85)
    a.stroke([(1.6, 1.4), (14.4, 1.4)], 0.3, "#ffa088", clip=True, alpha=0.6)
    # little lantern box with its glowing light
    a.rect(5.6, 6.6, 10.4, 10.4, ("lin", "#3a2a30", "#1e1418", 90), radius=0.4)
    a.rect(5.2, 6.4, 10.8, 7.1, "#2a1a24", radius=0.3)
    spot(a, 8, 8.6, 1.6, "#ffd23a", alpha=0.7, blur=0.6)
    a.rect(6.9, 7.6, 9.1, 10.0, ("lin", "#fff0a0", "#ffc020", 90), radius=0.3)
    # offering box
    a.rect(4.2, 10.3, 11.8, 14.6, ("lin", "#f8eedc", "#c9b89a", 90), radius=0.4)
    a.rect(4.2, 10.3, 11.8, 11.1, "#5a3a2a", clip=True)
    for x in (5.9, 7.3, 8.7, 10.1):
        a.stroke([(x, 11.6), (x, 13.4)], 0.45, "#a8987a", clip=True, alpha=0.9)
    a.rect(4.2, 13.8, 11.8, 14.6, "#8a7a60", clip=True, alpha=0.7)
    a.outline()
    a.save("shrine")


# ── pickups 16x16 ──
def exorcism_seal():
    a = Art(16, 16)
    a.rect(4.0, 0.9, 12.0, 15.1, ("lin", "#fffaff", "#e2d6f6", 90), radius=0.6)
    a.rect(5.0, 2.0, 11.0, 14.0, ("lin", "#a87cff", "#7a50e0", 90), radius=0.3)
    a.rect(5.4, 10.9, 10.6, 13.0, "#5a3ac8", radius=0.3)
    a.stroke([(6.2, 11.9), (9.8, 11.9)], 0.35, "#a87cff", alpha=0.8)
    star = []
    import math
    for i in range(10):
        r = 2.6 if i % 2 == 0 else 1.1
        ang = math.radians(-90 + i * 36)
        star.append((8 + r * math.cos(ang), 7.4 + r * math.sin(ang)))
    spot(a, 8, 7.4, 2.6, "#ffe066", alpha=0.5, blur=0.8, clip=True)
    a.poly(star, ("lin", "#fff4a0", "#ffc830", 90))
    shine(a, 7.5, 6.6, 0.3)
    a.stroke([(5.6, 2.6), (5.6, 10.0)], 0.4, "#d8c4ff", clip=True, alpha=0.5)
    a.outline()
    glow(a, "#9a6aff", radius=1.2, alpha=0.35)
    a.save("pickups/exorcism_seal")


def feast():
    a = Art(16, 16)
    # leaf plate
    a.blob([(0.8, 13.0), (8, 12.6), (15.2, 13.0), (14.4, 14.9), (8, 15.2), (1.6, 14.9)], ("lin", "#6ad86a", "#2a7a2a", 90))
    a.stroke([(2.0, 13.9), (14.0, 13.9)], 0.3, "#2a7a2a", clip=True, alpha=0.7)
    for cx in (4.5, 11.5):
        ri = [(cx, 3.4), (cx + 1.6, 4.6), (cx + 3.4, 8.4), (cx + 3.6, 11.2), (cx, 12.6), (cx - 3.6, 11.2),
              (cx - 3.4, 8.4), (cx - 1.6, 4.6)]
        a.blob(ri, ("lin", "#ffffff", "#dcdce6", 90))
        spot(a, cx + 2.0, 9.6, 1.4, "#b8b8c8", alpha=0.5, blur=0.6, clip=True)
        a.rect(cx - 1.9, 8.8, cx + 1.9, 13.0, ("lin", "#3a4a3a", "#1e2a1e", 90), radius=0.4, clip=True)
        a.stroke([(cx - 1.4, 9.4), (cx + 1.4, 9.4)], 0.3, "#5a6a5a", clip=True, alpha=0.7)
        shine(a, cx - 1.4, 5.8, 0.5, 0.8)
    # second rice ball drawn on top needs its own edge
    a.stroke([(8.6, 6.6), (8.0, 9.0), (7.9, 11.4)], 0.35, "#1a1423", clip=True, alpha=0.6)
    a.outline()
    a.save("pickups/feast")


def hourglass():
    a = Art(16, 16)
    glass = [(4.0, 3.0), (12.0, 3.0), (11.6, 4.8), (8.8, 7.4), (8.8, 8.6), (11.8, 11.2), (12.2, 12.6),
             (3.8, 12.6), (4.2, 11.2), (7.2, 8.6), (7.2, 7.4), (4.4, 4.8)]
    a.blob(glass, "#bff0ff", alpha=0.85, smooth=False)
    # remaining top sand (frozen) + bottom pile
    a.blob([(4.6, 3.4), (11.4, 3.4), (10.8, 4.8), (8.4, 6.6), (7.6, 6.6), (5.2, 4.8)], ("lin", "#e6faff", "#9adcff", 90), clip=True)
    a.blob([(8.0, 9.0), (10.6, 11.0), (11.8, 12.6), (4.2, 12.6), (5.4, 11.0)], ("lin", "#7ac8ff", "#3a8ae0", 90), clip=True)
    a.stroke([(8, 7.0), (8, 9.4)], 0.4, "#9adcff", clip=True)
    a.stroke([(5.2, 3.8), (6.0, 5.4)], 0.45, "#ffffff", clip=True, alpha=0.9)
    a.stroke([(5.0, 12.0), (5.8, 10.8)], 0.4, "#ffffff", clip=True, alpha=0.7)
    # caps
    for y0 in (0.9, 12.8):
        a.rect(2.2, y0, 13.8, y0 + 2.3, ("lin", "#5a6aa0", "#2e3a66", 90), radius=0.6)
        a.stroke([(3.0, y0 + 0.6), (13.0, y0 + 0.6)], 0.35, "#8a9ad0", clip=True, alpha=0.8)
    a.outline()
    glow(a, "#8ad8ff", radius=1.0, alpha=0.3)
    a.save("pickups/hourglass")


def oni_sake():
    a = Art(16, 16)
    a.rect(6.8, 0.8, 10.0, 3.4, ("lin", "#8a5a3a", "#4a2e1e", 0), radius=0.6)
    a.stroke([(7.2, 1.4), (9.6, 1.4)], 0.35, "#b07a50", clip=True, alpha=0.8)
    flask = [(7.0, 3.0), (9.8, 3.0), (10.4, 5.0), (11.0, 7.4), (12.8, 9.6), (13.0, 12.2), (11.4, 14.6),
             (8.6, 15.2), (5.8, 14.6), (3.6, 12.6), (3.8, 9.6), (5.8, 7.4), (6.4, 5.0)]
    a.blob(flask, ("lin", "#f05248", "#b02c28", 90))
    spot(a, 10.6, 11.8, 2.8, "#8a1e1e", alpha=0.55, blur=1.0, clip=True)
    # rope tie at the neck
    a.stroke([(6.3, 4.6), (10.4, 4.6)], 0.5, "#ffd27a", clip=True)
    # label
    a.rect(6.6, 7.0, 10.0, 9.6, ("lin", "#fff0b0", "#ffd27a", 90), radius=0.3)
    a.stroke([(8.3, 7.5), (8.3, 9.1)], 0.4, "#a02a24")
    a.stroke([(5.0, 9.4), (4.6, 11.6), (5.2, 13.0)], 0.7, "#ff9a90", clip=True, alpha=0.7, taper=True)
    shine(a, 7.6, 4.0, 0.3, 0.6)
    a.outline()
    a.save("pickups/oni_sake")


def sen_purse():
    a = Art(16, 16)
    bag = [(5.0, 4.6), (8, 4.2), (11.0, 4.6), (12.4, 6.6), (13.2, 10.2), (12.8, 13.6), (8, 15.0),
           (3.2, 13.6), (2.8, 10.2), (3.6, 6.6)]
    a.blob(bag, ("lin", "#b06a44", "#7a4428", 90))
    spot(a, 9.2, 11.2, 2.6, "#5a3420", alpha=0.7, blur=0.8, clip=True)
    a.stroke([(4.8, 7.6), (4.2, 10.6)], 0.7, "#d08a60", clip=True, alpha=0.7, taper=True)
    # drawstring band and bow
    a.stroke([(3.6, 6.4), (8, 6.9), (12.4, 6.4)], 0.9, "#e83a3a", clip=True)
    a.stroke([(3.6, 6.2), (8, 6.6), (12.4, 6.2)], 0.3, "#ff8a8a", clip=True, alpha=0.7)
    # sen coins (square holes)
    for cx, cy in ((3.2, 13.0), (12.6, 13.0)):
        a.ellipse(cx, cy, 2.2, 1.9, ("lin", "#ffe680", "#d8a02a", 90))
        a.rect(cx - 0.45, cy - 0.4, cx + 0.45, cy + 0.4, "#5a3a10")
        shine(a, cx - 0.9, cy - 0.8, 0.3)
    a.outline()
    a.save("pickups/sen_purse")


def shrine_bell():
    a = Art(16, 16)
    a.stroke([(8.0, 0.4), (8.0, 4.0)], 1.1, ("lin", "#ff5a5a", "#b82a2a", 0))
    a.ellipse(8.0, 3.8, 1.6, 0.9, "#c9962a")
    bell = [(8, 3.6), (11.6, 4.8), (13.6, 8.2), (12.8, 11.6), (10.2, 13.6), (8, 14.4), (5.8, 13.6),
            (3.2, 11.6), (2.4, 8.2), (4.4, 4.8)]
    a.blob(bell, ("rad", "#fff090", "#e0a020", (0.4, 0.4)))
    spot(a, 10.4, 10.0, 3.0, "#b07a18", alpha=0.6, blur=1.0, clip=True)
    a.stroke([(2.6, 9.0), (8, 9.8), (13.4, 9.0)], 0.35, "#a87818", clip=True, alpha=0.6)
    # sound slit
    a.stroke([(5.8, 12.0), (10.2, 12.0)], 0.7, "#2a1a10")
    a.ellipse(8.0, 12.0, 0.7, 0.7, "#2a1a10")
    shine(a, 5.8, 6.4, 0.9, 0.85)
    shine(a, 4.6, 8.0, 0.35, 0.6)
    a.outline()
    glow(a, "#ffd23a", radius=1.0, alpha=0.3)
    a.save("pickups/shrine_bell")


def swift_geta():
    a = Art(16, 16)
    # speed lines
    for y in (5.4, 8.4, 11.4):
        a.stroke([(2.6, y), (0.3, y)], 0.75, "#bff0ff", taper=True)
    # teeth
    for x0 in (4.0, 10.0):
        a.rect(x0, 9.4, x0 + 3.0, 14.0, ("lin", "#8a5428", "#5a3418", 0), radius=0.4)
    # platform
    a.rect(3.0, 5.6, 14.0, 10.0, ("lin", "#e0a060", "#a8682e", 90), radius=0.9)
    a.stroke([(4.0, 6.4), (13.0, 6.4)], 0.35, "#ffd0a0", clip=True, alpha=0.7)
    a.rect(3.0, 9.0, 14.0, 10.0, "#7a4a20", clip=True, alpha=0.6)
    # strap (hanao)
    a.stroke([(5.8, 6.0), (8.2, 3.0), (10.8, 6.0)], 0.8, ("lin", "#ff5a5a", "#c02a2a", 90), smooth=False)
    a.ellipse(8.2, 3.0, 0.7, 0.7, "#e83a3a")
    a.outline()
    a.save("pickups/swift_geta")


NAMES = ["wisp", "kasa", "chochin", "nopperabo", "oni", "night_lord", "chest", "chest_open", "shrine",
         "pickups/exorcism_seal", "pickups/feast", "pickups/hourglass", "pickups/oni_sake",
         "pickups/sen_purse", "pickups/shrine_bell", "pickups/swift_geta"]

if __name__ == "__main__":
    for fn in (wisp, kasa, chochin, nopperabo, oni, night_lord, chest, chest_open, shrine,
               exorcism_seal, feast, hourglass, oni_sake, sen_purse, shrine_bell, swift_geta):
        fn()
    if len(sys.argv) > 1:
        preview(NAMES, sys.argv[1], scale=2)
