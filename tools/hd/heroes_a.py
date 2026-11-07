"""HD redraws of the playable heroes Ren, Riku and Shun (and their transformed forms), plus
Shun's nodachi sprites. Writes art/hd/<name>.png via hdkit.

    python3 tools/hd/heroes_a.py

All heroes are 24x32 art pixels, face right, and keep their legs in the walk-lift regions
(columns 5..10 and 14..19, rows 27..32) used by the client.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from hdkit import Art, _catmull  # noqa: E402

INK = "#2a1a2e"          # interior line art
LEG_L, LEG_R = 7.6, 16.4  # leg centres (inside the lift columns)


# ───────────────────────── shared pieces ─────────────────────────

def legs(a, sock, sock_dark, shoe, shoe_dark, top=26.4, sock_end=29.0, cuff=None):
    """Two legs: sock / wrap from `top` down, then a sandal with a sole, toes pointing right."""
    for cx in (LEG_L, LEG_R):
        a.rect(cx - 1.25, top, cx + 1.25, sock_end + 0.4, ("lin", sock, sock_dark, 0), radius=0.5)
        if cuff:
            a.rect(cx - 1.3, top, cx + 1.3, top + 0.7, cuff, radius=0.3)
        a.blob([(cx - 1.3, sock_end - 0.1), (cx + 1.0, sock_end - 0.2), (cx + 1.75, sock_end + 0.9),
                (cx + 1.6, 31.0), (cx - 1.35, 31.0), (cx - 1.45, sock_end + 0.8)],
               ("lin", shoe, shoe_dark, 90))
        a.stroke([(cx - 1.25, 30.55), (cx + 1.55, 30.55)], 0.45, shoe_dark)
        # Sock shading on the inner/right side.
        a.rect(cx + 0.45, top, cx + 1.25, sock_end, sock_dark, radius=0.4, alpha=0.55)


def eye(a, cx, cy, iris, iris_dark, w=1.25, h=1.55, lash=INK, half=False, glow=None):
    """A big anime eye: white, gradient iris, pupil, lash line and highlight dots."""
    a.ellipse(cx, cy, w, h, "#ffffff")
    a.ellipse(cx + 0.1, cy + 0.12, w * 0.86, h * 0.92, ("lin", iris_dark, iris, 90))
    a.ellipse(cx + 0.12, cy + 0.05, w * 0.42, h * 0.45, iris_dark)
    a.ellipse(cx + 0.12, cy + 0.75, w * 0.5, h * 0.22, iris, alpha=0.75)
    top = cy - h + (0.75 if half else 0.15)
    if half:
        # Heavy upper lid (cool / sleepy look): skin covers the top of the eye.
        a.blob([(cx - w - 0.2, cy - h - 0.4), (cx + w + 0.2, cy - h - 0.4),
                (cx + w + 0.2, top + 0.1), (cx, top - 0.1), (cx - w - 0.2, top + 0.2)], "#ffe2cc")
    a.stroke([(cx - w - 0.15, top + 0.45), (cx - 0.2, top - 0.05), (cx + w + 0.25, top + 0.25)], 0.6, lash)
    a.stroke([(cx + w + 0.1, top + 0.25), (cx + w + 0.6, top - 0.1)], 0.35, lash)   # flick
    a.ellipse(cx - 0.4, cy - 0.35 + (0.4 if half else 0), 0.42, 0.48, "#ffffff")
    a.ellipse(cx + 0.45, cy + 0.6, 0.2, 0.2, "#ffffff", alpha=0.85)
    if glow:
        a.ellipse(cx, cy, w * 1.6, h * 1.4, glow, alpha=0.22)


def face(a, pts, skin="#ffe2cc", skin_dark="#f2b89a"):
    a.blob(pts, ("lin", skin, "#ffd8c0", 90))


def blush(a, xs, y, col="#ff9aa8", alpha=0.55):
    for x in xs:
        a.ellipse(x, y, 0.9, 0.42, col, alpha=alpha)


def mouth(a, x, y, col="#b0505a", w=0.5):
    a.stroke([(x - w, y - 0.05), (x, y + 0.15), (x + w, y - 0.05)], 0.35, col)


def finish(a, shade=0.13, rim=0.35, rim_col="#ffffff"):
    a.shade_side("#1a1030", shade, side="right", amount=0.32)
    a.rim(rim_col, rim)
    a.outline()


def ink(a, pts, w=0.32, col=INK, alpha=0.85, smooth=True):
    """Thin interior line along a closed shape's edge (separates overlapping parts)."""
    path = _catmull(pts, True) if smooth else list(pts)
    a.stroke(path + [path[0]], w, col, smooth=False, alpha=alpha)


def arm(a, path, col, cuff, w):
    """A sleeved arm drawn over the body, with its own interior outline."""
    a.stroke(path, w + 0.6, INK, alpha=0.75)
    a.stroke(path, w, ("lin", col, cuff, 90))
    a.stroke([path[0], path[1]], w * 0.35, "#ffffff", alpha=0.12)


def part(a, pts, fill, w=0.32, alpha=0.85, smooth=True):
    a.blob(pts, fill, smooth=smooth)
    ink(a, pts, w, alpha=alpha, smooth=smooth)


def _q(p0, c, p1, n=8):
    return [((1 - t) ** 2 * p0[0] + 2 * (1 - t) * t * c[0] + t * t * p1[0],
             (1 - t) ** 2 * p0[1] + 2 * (1 - t) * t * c[1] + t * t * p1[1]) for t in (i / n for i in range(n + 1))]


def spike(a, b1, b2, tip, fill, bend=0.2, clip=False, alpha=1.0):
    """A sharp hair spike with curved sides sweeping the same way (anime hair lock)."""
    def ctrl(p):
        mx, my = (p[0] + tip[0]) / 2, (p[1] + tip[1]) / 2
        dx, dy = tip[0] - p[0], tip[1] - p[1]
        return (mx - dy * bend, my + dx * bend)
    pts = _q(b1, ctrl(b1), tip)[:-1] + _q(tip, ctrl(b2), b2)
    a.poly(pts, fill, clip=clip, alpha=alpha)


def spikes(a, specs, fill):
    for b1, b2, tip, bend in specs:
        spike(a, b1, b2, tip, fill, bend)


# ───────────────────────── Ren ─────────────────────────

def ren(divine):
    a = Art(24, 32)
    H, Hd, L = "#3a2622", "#22150f", ("#a07a5a" if divine else "#6e4a3a")
    O, k = "#f6f2ec", ("#ffe8a8" if divine else "#c8c0b8")
    R, r = ("#ff3a4a", "#c42a3a") if divine else ("#c42a3a", "#8a1a26")
    Y = "#ffd23a"
    iris, iris_d = ("#ffd23a", "#a0400a") if divine else ("#6a7cff", "#24163f")

    # Long hair behind the body.
    a.blob([(4.2, 8), (19.6, 8), (20.1, 16), (19.9, 21.6), (18.6, 22.2), (17.4, 21.4),
            (6.4, 21.4), (5.3, 22.2), (3.9, 21.6), (3.7, 15)], ("lin", H, Hd, 90))

    # Gohei stick (behind the hand, in front of the hair).
    a.stroke([(20.7, 8.0), (20.7, 22.6)], 0.95, ("lin", "#b07a4a", "#6a4228", 0))

    # Hakama: flared, pleated.
    a.poly([(6.3, 19.4), (17.7, 19.4), (18.4, 26.7), (5.5, 26.7)], ("lin", R, r, 90))
    for x0, x1 in ((9.0, 8.7), (12.0, 12.0), (15.0, 15.3)):
        a.stroke([(x0, 21.0), (x1, 26.4)], 0.45, r)
        a.stroke([(x0 + 0.45, 21.2), (x1 + 0.5, 26.4)], 0.25, "#ffffff", alpha=0.18)
    a.poly([(14.8, 19.4), (17.7, 19.4), (18.4, 26.7), (15.6, 26.7)], "#000000", alpha=0.13)
    if divine:
        a.rect(5.5, 25.7, 18.4, 26.8, ("lin", "#fff0a0", Y, 90))
    # Hakama himo (sash) across the waist.
    a.rect(6.2, 19.4, 17.8, 20.6, r if not divine else Y, radius=0.3)
    a.stroke([(6.4, 20.6), (17.6, 20.6)], 0.3, INK, alpha=0.6)
    a.stroke([(11.2, 20.0), (12.6, 20.0)], 0.5, "#ffffff", alpha=0.35)

    # Wide sleeves.
    part(a, [(6.2, 14.6), (3.1, 15.6), (2.4, 19.0), (2.7, 21.4), (5.0, 21.9), (6.8, 21.0), (7.0, 17.0)],
         ("lin", O, k, 0), alpha=0.5)
    part(a, [(17.4, 14.6), (20.0, 15.3), (21.0, 17.6), (20.9, 19.9), (18.8, 20.6), (17.2, 19.6)],
         ("lin", O, k, 20), alpha=0.5)
    # Kimono body.
    a.blob([(7.2, 14.2), (16.8, 14.2), (17.6, 16.0), (17.6, 20.0), (6.4, 20.0), (6.4, 16.0)],
           ("lin", O, "#e8e0d8", 0))
    a.poly([(14.6, 14.2), (17.0, 14.2), (17.7, 16), (17.7, 20.0), (15.0, 20.0)], k, alpha=0.75)
    # Sleeve folds.
    a.stroke([(4.4, 16.4), (4.0, 19.6)], 0.3, k, alpha=0.9)
    a.stroke([(19.2, 16.0), (19.5, 18.8)], 0.3, "#a8a098" if not divine else "#e0b860", alpha=0.6)
    # Crossed collar: red under-collar V.
    a.poly([(9.3, 14.1), (10.4, 14.1), (12.0, 17.2), (13.6, 14.1), (14.7, 14.1), (12.0, 18.6)], R)
    a.poly([(10.4, 14.1), (13.6, 14.1), (12.0, 17.2)], "#ffe2cc")
    a.stroke([(10.0, 14.2), (12.0, 18.4)], 0.25, r, alpha=0.6)
    if divine:
        a.stroke([(6.6, 14.6), (6.5, 19.6)], 0.45, Y)
        a.stroke([(17.3, 14.6), (17.4, 19.6)], 0.45, Y)
        a.stroke([(2.6, 21.2), (5.0, 21.8), (6.8, 21.0)], 0.5, Y)
        a.stroke([(18.8, 20.5), (20.9, 19.8)], 0.5, Y)

    # Hands.
    a.ellipse(4.6, 21.6, 0.95, 0.8, "#ffe2cc")
    a.ellipse(20.3, 20.0, 1.0, 0.85, "#ffe2cc")
    a.stroke([(19.9, 19.8), (20.9, 19.8)], 0.25, "#f2b89a")

    # Gohei paper streamers (shide), zig-zagging off the top of the stick.
    shide = ("lin", "#ffffff", "#e8e4f0", 90) if not divine else ("lin", "#ffffff", "#fff2b0", 90)
    a.stroke([(21.0, 8.7), (23.1, 9.5), (21.4, 11.0), (23.3, 12.2), (21.7, 13.6), (22.9, 14.6)], 1.15,
             shide, smooth=False)
    a.stroke([(20.4, 8.9), (19.3, 10.0), (20.3, 11.2), (19.3, 12.4)], 0.95, shide, smooth=False)
    a.stroke([(21.6, 11.0), (23.0, 12.0)], 0.22, "#b8b0c8", alpha=0.8)
    a.stroke([(21.8, 13.6), (22.6, 14.3)], 0.22, "#b8b0c8", alpha=0.8)
    a.stroke([(21.1, 8.7), (22.6, 9.3)], 0.22, "#b8b0c8", alpha=0.6)
    a.ellipse(20.7, 8.0, 0.6, 0.6, "#c9a040" if not divine else Y)
    if divine:
        a.ellipse(21.3, 11.0, 2.2, 3.4, Y, alpha=0.12)

    # Face.
    face(a, [(6.6, 9.0), (17.4, 9.0), (17.4, 12.0), (15.8, 14.4), (12.0, 15.3), (8.2, 14.4), (6.6, 12.0)])

    # Hair: rounded crown with a straight hime fringe.
    a.blob([(3.9, 11.4), (4.1, 6.4), (7.0, 3.2), (12.0, 2.2), (17.0, 3.2), (19.9, 6.4), (20.1, 11.4),
            (18.2, 10.6), (17.3, 9.9), (15.7, 9.6), (14.4, 10.0), (12.9, 9.5), (11.2, 10.0), (9.6, 9.5),
            (8.2, 10.0), (6.8, 9.9), (5.8, 10.6)], ("lin", "#4e342c", H, 90))
    # Hime sidelocks.
    for pts in ([(3.9, 8.6), (6.6, 9.2), (7.1, 12.6), (7.1, 16.3), (5.5, 16.0), (3.9, 16.3), (3.7, 12.4)],
                [(17.4, 9.2), (20.1, 8.6), (20.3, 12.4), (20.1, 16.3), (18.5, 16.0), (16.9, 16.3), (16.9, 12.6)]):
        a.blob(pts, ("lin", "#4a302a", Hd, 90), smooth=False)
        ink(a, pts, 0.25, alpha=0.5, smooth=False)
    # Strand lines and shine.
    for x0, x1 in ((8.2, 8.9), (11.2, 11.6), (14.4, 14.4), (16.6, 16.0)):
        a.stroke([(x0 + 0.3, 6.6), (x1, 9.4)], 0.3, Hd, alpha=0.8)
    a.stroke([(5.6, 6.6), (7.8, 4.8), (10.8, 4.1)], 0.75, L, alpha=0.9)
    a.stroke([(12.4, 4.0), (13.4, 4.1)], 0.6, L, alpha=0.7)
    a.stroke([(6.4, 6.0), (8.2, 4.9)], 0.3, "#ffffff", alpha=0.35)
    a.stroke([(5.4, 10.6), (5.5, 15.6)], 0.3, Hd, alpha=0.8)
    a.stroke([(18.6, 10.6), (18.5, 15.6)], 0.3, Hd, alpha=0.8)
    a.stroke([(4.4, 10.4), (4.4, 14.4)], 0.35, L, alpha=0.5)

    # White folded-paper hair tie on top, red cord.
    tie = "#ffffff"
    a.blob([(8.9, 0.4), (11.8, 1.1), (15.1, 0.4), (15.0, 2.4), (11.8, 2.0), (9.0, 2.4)],
           ("lin", tie, "#e4e0ec", 90))
    if divine:
        a.blob([(8.9, 0.4), (11.8, 1.1), (15.1, 0.4), (15.1, 1.1), (11.8, 1.7), (8.9, 1.1)], Y)
    a.rect(11.0, 1.2, 13.0, 3.0, R, radius=0.4)
    a.stroke([(11.4, 1.6), (12.4, 1.6)], 0.25, "#ffffff", alpha=0.5)

    # Eyes, blush, mouth.
    eye(a, 9.4, 11.6, iris, iris_d, glow=(Y if divine else None))
    eye(a, 14.6, 11.6, iris, iris_d, glow=(Y if divine else None))
    blush(a, (8.3, 15.7), 13.6)
    mouth(a, 12.0, 13.9)

    legs(a, "#ffffff", "#d8d4e0", "#5a3a32", "#3a2420", top=26.2, sock_end=28.7)

    finish(a)
    if divine:
        a.glow("#ffd23a", radius=0.7, alpha=0.35)
    a.save("ren_divine" if divine else "ren")


# ───────────────────────── Riku ─────────────────────────

def riku(fox):
    a = Art(24, 32)
    if fox:
        H, Hd, L = "#ffb43a", "#e8641a", "#fff2c0"
        N, n, J = "#ff9a2a", "#d4560e", "#ffd36a"
        C, c = "#ffcf4a", "#ff7a1a"
        G, g = "#ffffff", "#ffd36a"
        D, B = "#c4500e", "#7a2a0a"
        Fc, Fm, Fe = "#ffffff", "#ffd36a", "#ff8a1a"
        iris, iris_d = "#ff7a1a", "#ff3a00"
    else:
        H, Hd, L = "#d8343a", "#8e1c2a", "#ff8a6a"
        N, n, J = "#2b2f5e", "#1b1d3e", "#454a8a"
        C, c = "#c42a3a", "#7e1828"
        G, g = "#f2b632", "#b07e1c"
        D, B = "#3a3348", "#4a2e2a"
        Fc, Fm, Fe = "#bff0ff", "#3a9cff", "#1a52c8"
        iris, iris_d = "#ffb02e", "#24163f"

    # Flaring cloak tails in fox form (behind the body).
    if fox:
        a.blob([(4.0, 19.0), (2.0, 22.4), (3.0, 23.6), (5.5, 22.5)], ("lin", J, c, 90))
        a.blob([(17.5, 21.5), (20.6, 22.0), (20.0, 24.2), (17.8, 24.5)], ("lin", J, c, 90))

    # Body: indigo shinobi gear.
    a.blob([(4.4, 16.6), (17.8, 16.6), (18.3, 18.5), (18.1, 25.3), (6.0, 25.6), (3.9, 24.0), (3.7, 18.6)],
           ("lin", N, N, 0))
    a.poly([(14.6, 16.6), (17.9, 16.6), (18.3, 18.5), (18.1, 25.3), (14.8, 25.4)], n)
    # Front opening.
    a.rect(10.7, 16.8, 12.2, 21.0, J, radius=0.3)
    a.stroke([(11.45, 17.2), (11.45, 20.6)], 0.25, n, alpha=0.6)
    if fox:
        # Flame crests on the chest.
        for x in (8.3, 13.6):
            a.stroke([(x - 0.9, 19.8), (x, 17.6), (x + 0.9, 19.8)], 0.55, "#3a1004")
        a.rect(4.0, 19.8, 18.2, 20.8, J, radius=0.3)
    # Gold sash with a hanging tail.
    a.rect(5.6, 20.7, 18.2, 22.2, ("lin", G, g, 90), radius=0.4)
    a.stroke([(6.2, 21.0), (16.8, 21.0)], 0.3, "#ffffff", alpha=0.45)
    a.blob([(9.0, 21.8), (10.6, 21.8), (10.9, 24.6), (9.8, 25.2), (8.8, 24.4)], ("lin", G, g, 90))
    # Lower tunic hem.
    a.rect(5.8, 24.4, 18.1, 25.6, n, radius=0.4)

    arm(a, [(5.8, 17.4), (4.7, 20.0), (4.7, 22.9)], N, n, 2.1)
    arm(a, [(17.4, 17.4), (18.8, 19.2), (19.3, 20.7)], n, n, 2.0)
    # Left hand (low, by the hip).
    a.ellipse(4.8, 23.6, 1.0, 0.85, "#ffe2cc")

    # Crimson scarf: wraps the neck and trails off to the right.
    part(a, [(4.4, 15.6), (17.8, 15.6), (18.2, 16.6), (17.6, 17.4), (4.3, 17.4), (3.9, 16.5)], ("lin", C, c, 90),
         alpha=0.6)
    part(a, [(15.4, 16.6), (17.6, 16.8), (18.0, 19.6), (17.4, 21.0), (16.2, 19.6), (15.2, 18.0)],
         ("lin", C, c, 90), alpha=0.6)
    a.stroke([(16.4, 17.4), (16.7, 19.6)], 0.3, c, alpha=0.8)
    a.stroke([(6.0, 16.1), (14.0, 16.1)], 0.3, "#ffffff", alpha=0.3)
    # High collar.
    a.rect(8.0, 14.7, 16.0, 15.9, ("lin", J, n, 90), radius=0.5)

    # Foxfire in the right hand.
    a.blob([(20.6, 15.6), (21.8, 17.0), (23.3, 17.6), (23.2, 20.0), (21.6, 21.6), (19.6, 21.2),
            (18.8, 19.4), (19.6, 17.4)], ("rad", Fm, Fe, (0.4, 0.5)))
    a.blob([(20.9, 17.4), (21.9, 18.6), (22.3, 20.0), (21.4, 21.0), (20.1, 20.6), (20.0, 19.0)], Fm)
    a.blob([(21.1, 19.0), (21.7, 19.8), (21.3, 20.6), (20.6, 20.2)], Fc)
    a.ellipse(19.4, 21.4, 1.0, 0.85, "#ffe2cc")
    a.stroke([(18.9, 21.1), (19.9, 21.0)], 0.25, "#f2b89a")

    # Face.
    face(a, [(6.6, 8.6), (17.4, 8.6), (17.4, 11.8), (15.8, 14.0), (12.0, 15.0), (8.2, 14.0), (6.6, 11.8)])

    # Spiky crimson hair: rounded mass plus sharp swept locks.
    hair = ("lin", "#e8484a" if not fox else "#ffd060", Hd, 90)
    a.blob([(3.4, 8.4), (4.0, 3.8), (8.0, 1.9), (12.0, 1.5), (16.5, 1.9), (20.0, 3.8), (20.6, 8.4),
            (19.6, 10.2), (4.4, 10.2)], hair)
    spikes(a, [((7.6, 3.0), (12.0, 2.4), (10.6, -0.2), 0.18),
               ((14.2, 2.6), (18.8, 4.2), (18.6, 0.2), -0.12),
               ((4.0, 4.6), (8.0, 2.6), (4.8, 0.6), 0.1),
               ((3.8, 4.6), (3.6, 9.2), (0.2, 7.2), 0.16),
               ((3.6, 8.6), (4.8, 12.0), (2.0, 12.4), 0.12),
               ((20.0, 4.2), (20.4, 9.0), (23.7, 6.7), -0.16),
               ((20.4, 8.4), (19.4, 12.0), (21.9, 11.8), -0.12),
               ((4.0, 8.2), (7.6, 8.4), (5.2, 14.4), 0.08),
               ((16.4, 8.4), (20.0, 8.2), (18.8, 14.4), -0.08),
               ((5.6, 7.4), (8.8, 7.4), (6.6, 10.6), 0.12),
               ((7.8, 7.2), (11.0, 7.2), (9.0, 10.0), 0.12),
               ((10.2, 7.2), (13.6, 7.2), (12.0, 10.9), 0.05),
               ((12.8, 7.2), (16.2, 7.2), (15.0, 10.0), -0.12),
               ((15.2, 7.4), (18.4, 7.4), (17.4, 10.6), -0.12)], hair)
    a.poly([(3.6, 8.6), (4.8, 12.0), (2.0, 12.4)], Hd, alpha=0.6)
    a.poly([(20.4, 8.4), (19.4, 12.0), (21.9, 11.8)], Hd, alpha=0.6)
    a.stroke([(6.4, 6.6), (8.8, 4.4), (11.6, 3.8)], 0.8, L, alpha=0.9)
    a.stroke([(14.0, 4.6), (15.6, 4.0)], 0.6, L, alpha=0.7)
    a.stroke([(10.6, 6.6), (10.9, 9.2)], 0.3, Hd, alpha=0.8)
    a.stroke([(14.0, 6.8), (14.4, 9.2)], 0.3, Hd, alpha=0.8)
    a.stroke([(5.0, 9.4), (5.3, 13.4)], 0.3, Hd, alpha=0.8)
    a.stroke([(19.0, 9.4), (18.7, 13.4)], 0.3, Hd, alpha=0.8)

    # Fox mask tilted on the upper left of his head.
    a.blob([(2.4, 0.4), (3.9, 1.6), (5.6, 1.4), (6.8, 0.6), (7.6, 2.6), (7.4, 5.4), (6.2, 7.0),
            (3.4, 7.0), (1.4, 5.6), (1.0, 3.2)], ("lin", "#ffffff", "#d8d0e4", 90))
    a.blob([(2.6, 1.2), (3.6, 2.0), (2.7, 2.7)], "#e0263c", alpha=0.85)     # ear insides
    a.blob([(6.6, 1.3), (6.9, 2.6), (5.9, 2.1)], "#e0263c", alpha=0.85)
    a.stroke([(2.3, 3.8), (3.3, 3.6)], 0.45, "#e0263c")                    # eye markings
    a.stroke([(5.0, 3.6), (6.0, 3.8)], 0.45, "#e0263c")
    a.ellipse(4.2, 5.4, 0.45, 0.35, "#e0263c")                              # nose
    a.stroke([(2.2, 6.0), (4.2, 6.6), (6.2, 6.0)], 0.3, "#b8b0c8")
    a.stroke([(3.4, 2.4), (4.6, 2.3)], 0.25, "#ffffff", alpha=0.7)

    # Eyes (determined, sharp brows), mouth.
    eye(a, 9.4, 11.4, iris, iris_d, w=1.2, h=1.45, glow=("#ff9a2a" if fox else None))
    eye(a, 14.6, 11.4, iris, iris_d, w=1.2, h=1.45, glow=("#ff9a2a" if fox else None))
    a.stroke([(8.0, 9.4), (10.6, 9.9)], 0.4, Hd)
    a.stroke([(13.4, 9.9), (16.0, 9.4)], 0.4, Hd)
    blush(a, (8.2, 15.8), 13.4, alpha=0.35)
    a.stroke([(11.4, 13.7), (12.3, 13.95), (12.9, 13.6)], 0.35, "#b0505a")   # little smirk

    legs(a, D, "#262030" if not fox else "#9a3a08", B, "#2e1c1a" if not fox else "#4a1804",
         top=25.4, sock_end=29.0)
    for cx in (LEG_L, LEG_R):  # leg-wrap bands
        for y in (26.6, 27.8):
            a.stroke([(cx - 1.2, y), (cx + 1.2, y + 0.35)], 0.3, "#000000", alpha=0.3)

    finish(a)
    if fox:
        a.glow("#ffb43a", radius=0.55, alpha=0.28)
    a.save("riku_fox" if fox else "riku")


# ───────────────────────── Shun ─────────────────────────

def shun(hannya):
    a = Art(24, 32)
    H, Hd, L = "#c8ccd8", "#8a8fa8", "#ffffff"
    tip = "#c42a3a" if hannya else Hd
    if hannya:
        N, n, V = "#1a1418", "#0e0a0e", "#6a1028"
    else:
        N, n, V = "#3a3a46", "#24242e", "#5a3a8a"
    C, c = "#c42a3a", "#7e1828"
    D, B = "#2a2433", "#4a2e2a"

    # Haori body.
    a.blob([(3.6, 16.4), (20.4, 16.4), (20.6, 18.0), (18.6, 24.8), (6.0, 24.8), (3.4, 23.6), (3.2, 18.0)],
           N)
    a.poly([(15.0, 16.4), (20.4, 16.4), (20.6, 18.0), (18.6, 24.8), (15.0, 24.8)], n)
    # Violet kimono V in front, skin at the neck.
    a.poly([(8.8, 16.4), (15.2, 16.4), (15.0, 21.2), (9.0, 21.2)], ("lin", V, "#3e2862" if not hannya else "#4a0a1a", 90))
    a.poly([(10.6, 16.4), (13.4, 16.4), (12.0, 19.6)], "#ffe2cc")
    a.stroke([(10.4, 16.4), (12.0, 19.8), (13.6, 16.4)], 0.35, "#2a1a40" if not hannya else "#2a0812")
    # Haori lapels.
    a.stroke([(8.8, 16.6), (9.0, 21.0)], 0.5, "#4c4c5c" if not hannya else "#2c2028")
    a.stroke([(15.1, 16.6), (15.0, 21.0)], 0.5, "#18181f" if not hannya else "#050305")
    if hannya:
        # Dark markings on the haori.
        for x, y in ((4.6, 18.4), (5.0, 20.4), (19.4, 18.4), (19.0, 20.4)):
            a.stroke([(x - 0.6, y), (x + 0.6, y + 0.8)], 0.5, "#4a0a1a")
    # Rope belt.
    a.rect(5.8, 21.0, 18.0, 22.6, ("lin", C, c, 90), radius=0.5)
    for x in (7.4, 9.4, 11.4, 13.4, 15.4):
        a.stroke([(x, 21.1), (x + 0.9, 22.5)], 0.3, c, alpha=0.8)
    a.stroke([(6.4, 21.4), (17.2, 21.4)], 0.25, "#ff8a8a", alpha=0.5)
    # Kimono below the belt.
    a.rect(9.0, 22.4, 15.0, 24.8, V)
    a.rect(13.0, 22.4, 15.0, 24.8, "#000000", alpha=0.2)
    # Soul-lantern charm hanging from the belt.
    a.stroke([(7.6, 22.4), (7.6, 23.0)], 0.25, "#ffd27a")
    a.ellipse(7.6, 23.9, 0.85, 1.05, ("rad", "#fff4c8", "#e8613a"))
    a.rect(6.9, 22.8, 8.3, 23.1, "#4a2e2a")
    a.rect(6.9, 24.8, 8.3, 25.1, "#4a2e2a")
    # Dark hakama.
    a.poly([(6.2, 24.6), (17.8, 24.6), (18.2, 26.9), (5.8, 26.9)], ("lin", D, "#1c1824", 90))
    a.stroke([(12.0, 25.0), (12.0, 26.8)], 0.3, "#15111c")
    arm(a, [(5.2, 17.2), (4.3, 20.2), (4.5, 22.8)], N, n, 2.6)
    arm(a, [(18.8, 17.2), (19.8, 19.4), (19.7, 20.9)], n, n, 2.4)
    # Hands: left low by the hip, right by the waist.
    a.ellipse(4.6, 23.5, 1.0, 0.85, "#ffe2cc")
    a.ellipse(19.6, 21.6, 1.0, 0.85, "#ffe2cc")

    if hannya:
        # Bone armour on the shoulders and chest.
        for pts in ([(3.0, 16.2), (8.6, 16.0), (8.8, 17.2), (8.0, 18.4), (3.0, 18.4), (2.6, 17.2)],
                    [(15.4, 16.0), (21.0, 16.2), (21.4, 17.2), (21.0, 18.4), (16.0, 18.4), (15.2, 17.2)]):
            a.blob(pts, ("lin", "#e8e0d0", "#a8a090", 90))
        a.stroke([(3.4, 17.3), (8.2, 17.3)], 0.25, "#a8a090")
        a.stroke([(15.8, 17.3), (20.8, 17.3)], 0.25, "#a8a090")
        a.blob([(10.0, 18.6), (14.0, 18.6), (13.8, 20.2), (12.0, 20.8), (10.2, 20.2)], ("lin", "#e8e0d0", "#a8a090", 90))
        a.stroke([(10.6, 19.5), (13.4, 19.5)], 0.25, "#a8a090")
    # High black collar of the haori.
    a.rect(5.8, 14.8, 18.2, 16.8, ("lin", "#26222c", "#141018", 90), radius=0.6)
    a.stroke([(6.6, 15.3), (17.0, 15.3)], 0.25, "#5a5668", alpha=0.6)

    # Face.
    face(a, [(6.8, 8.6), (17.2, 8.6), (17.2, 12.0), (15.6, 14.4), (12.0, 15.3), (8.4, 14.4), (6.8, 12.0)])

    # Wild ash-silver hair: big spiky mane that flares out at the sides.
    hair = ("lin", "#eceef4", H, 90)
    a.blob([(3.0, 7.6), (4.0, 3.2), (8.0, 2.0), (12.0, 1.8), (16.0, 2.0), (20.0, 3.2), (21.0, 7.6),
            (20.6, 11.0), (3.4, 11.0)], hair)
    spikes(a, [((3.6, 4.8), (6.8, 2.4), (1.6, 0.3), 0.12),
               ((6.6, 2.6), (10.8, 2.0), (8.6, 0.0), 0.1),
               ((13.2, 2.0), (17.4, 2.6), (15.4, 0.0), -0.1),
               ((17.2, 2.4), (20.4, 4.8), (21.8, 0.4), -0.12),
               ((3.4, 4.6), (3.2, 8.4), (0.3, 7.2), 0.12),
               ((3.2, 7.8), (3.2, 11.4), (0.1, 10.2), 0.1),
               ((3.4, 10.4), (5.0, 12.6), (1.2, 12.8), 0.08),
               ((20.6, 4.6), (20.8, 8.4), (23.7, 7.2), -0.12),
               ((20.8, 7.8), (20.8, 11.4), (23.9, 10.4), -0.1),
               ((20.6, 10.4), (19.0, 12.6), (22.8, 12.8), -0.08),
               ((3.8, 9.0), (6.8, 9.4), (3.8, 14.6), 0.05),
               ((17.2, 9.4), (20.2, 9.0), (20.2, 14.6), -0.05),
               ((5.6, 7.2), (9.0, 7.0), (7.2, 10.8), 0.1),
               ((8.2, 7.0), (11.4, 7.0), (9.6, 10.2), 0.1),
               ((10.4, 7.0), (13.6, 7.0), (12.0, 10.4), 0.0),
               ((12.6, 7.0), (15.8, 7.0), (14.4, 10.2), -0.1),
               ((15.0, 7.0), (18.4, 7.2), (16.8, 10.8), -0.1)], hair)
    # Shadowed under-layer at the sides.
    for b1, b2, tp, bend in (((3.2, 7.8), (3.2, 11.4), (0.1, 10.2), 0.1),
                              ((3.4, 10.4), (5.0, 12.6), (1.2, 12.8), 0.08),
                              ((20.8, 7.8), (20.8, 11.4), (23.9, 10.4), -0.1),
                              ((20.6, 10.4), (19.0, 12.6), (22.8, 12.8), -0.08)):
        spike(a, b1, b2, tp, Hd, bend, alpha=0.65)
    a.poly([(16.0, 1.0), (22.0, 0.0), (24.0, 9.0), (19.0, 9.0), (18.0, 5.0)], Hd, alpha=0.35, clip=True)
    if hannya:
        for x, y in ((1.6, 0.3), (8.6, 0.3), (15.2, 0.3), (21.6, 0.6), (0.4, 7.4), (23.6, 7.4),
                     (0.2, 10.0), (23.8, 10.4), (1.2, 12.6), (22.8, 12.8)):
            a.ellipse(x, y, 1.3, 1.3, tip, clip=True, alpha=0.9)
    a.stroke([(5.6, 6.4), (8.0, 4.4), (10.8, 3.8)], 0.8, L, alpha=0.9)
    a.stroke([(12.4, 3.8), (13.8, 3.6)], 0.55, L, alpha=0.7)
    for x0, x1 in ((9.0, 9.6), (12.0, 12.0), (15.0, 14.6)):
        a.stroke([(x0, 6.2), (x1, 9.0)], 0.3, Hd, alpha=0.9)

    if hannya:
        # Horned hannya mask over the face.
        horn = ("lin", "#f2c060", "#7a4a10", 0)
        a.blob([(7.6, 7.6), (9.2, 7.0), (7.8, 3.6), (6.4, 0.4), (5.6, 0.8), (6.2, 4.0)], horn)
        a.blob([(16.4, 7.6), (14.8, 7.0), (16.2, 3.6), (17.6, 0.4), (18.4, 0.8), (17.8, 4.0)], horn)
        a.stroke([(7.0, 5.6), (8.2, 5.2)], 0.3, "#7a4a10")
        a.stroke([(16.8, 5.6), (15.8, 5.2)], 0.3, "#7a4a10")
        a.stroke([(6.6, 3.2), (7.4, 2.8)], 0.3, "#7a4a10")
        a.stroke([(17.2, 3.2), (16.6, 2.8)], 0.3, "#7a4a10")
        a.ellipse(6.0, 0.7, 0.7, 0.7, "#ffd23a")
        a.ellipse(18.0, 0.7, 0.7, 0.7, "#ffd23a")
        a.blob([(7.0, 7.4), (17.0, 7.4), (17.4, 10.6), (16.6, 13.8), (14.6, 15.8), (12.0, 16.4),
                (9.4, 15.8), (7.4, 13.8), (6.6, 10.6)], ("lin", "#fbf8f0", "#c8c0ae", 90))
        # Furrowed brows and eye sockets with glowing gold eyes.
        a.blob([(7.8, 9.0), (11.2, 10.2), (11.0, 11.8), (8.6, 11.6), (7.6, 10.4)], "#120808")
        a.blob([(16.2, 9.0), (12.8, 10.2), (13.0, 11.8), (15.4, 11.6), (16.4, 10.4)], "#120808")
        a.ellipse(9.6, 10.9, 0.75, 0.5, "#ffd23a")
        a.ellipse(14.4, 10.9, 0.75, 0.5, "#ffd23a")
        a.ellipse(9.6, 10.9, 1.4, 1.0, "#ffd23a", alpha=0.25)
        a.ellipse(14.4, 10.9, 1.4, 1.0, "#ffd23a", alpha=0.25)
        a.stroke([(7.6, 8.4), (11.6, 9.8)], 0.5, "#b8b0a0")
        a.stroke([(16.4, 8.4), (12.4, 9.8)], 0.5, "#b8b0a0")
        # Grinning mouth with fangs.
        a.blob([(8.6, 13.0), (15.4, 13.0), (14.4, 15.0), (12.0, 15.5), (9.6, 15.0)], "#120808")
        for x in (9.4, 10.8, 13.2, 14.6):
            a.poly([(x - 0.5, 13.0), (x + 0.5, 13.0), (x, 14.4)], "#fbf8f0")
        a.poly([(11.4, 15.3), (12.6, 15.3), (12.0, 14.0)], "#fbf8f0")
        a.stroke([(11.6, 12.0), (12.0, 12.6), (12.4, 12.0)], 0.3, "#b8b0a0")
        a.stroke([(7.4, 11.6), (8.0, 13.6)], 0.3, "#c42a3a", alpha=0.8)
        a.stroke([(16.6, 11.6), (16.0, 13.6)], 0.3, "#c42a3a", alpha=0.8)
    else:
        # Half-lidded cool violet eyes.
        eye(a, 9.4, 11.5, "#9a6ae0", "#1c1426", w=1.2, h=1.4, half=True)
        eye(a, 14.6, 11.5, "#9a6ae0", "#1c1426", w=1.2, h=1.4, half=True)
        a.stroke([(11.6, 13.9), (12.5, 13.9)], 0.3, "#a05050")
        blush(a, (8.3, 15.7), 13.3, alpha=0.25)

    legs(a, D, "#1a1622", B, "#2e1c1a", top=26.4, sock_end=29.0)

    finish(a)
    if hannya:
        a.glow("#c42a3a", radius=1.0, alpha=0.35)
    a.save("shun_hannya" if hannya else "shun")


# ───────────────────────── Shun's nodachi ─────────────────────────

def blade_colors(hannya):
    if hannya:
        return "#1a1418", "#2e2630", "#e0203c"
    return "#f2ece0", "#b8b0a0", "#ffffff"


def cleaver(hannya):
    a = Art(9, 38)
    W, w, e = blade_colors(hannya)
    # Blade: long, slightly curved, edge on the left.
    a.blob([(4.4, 0.2), (6.4, 1.4), (6.5, 8.0), (5.9, 18.0), (5.9, 27.0), (3.0, 27.0), (3.1, 18.0),
            (3.6, 8.0), (3.8, 1.4)], ("lin", W, w, 0))
    a.blob([(4.4, 0.2), (6.4, 1.4), (6.5, 8.0), (5.9, 18.0), (5.9, 27.0), (5.0, 27.0), (5.0, 18.0),
            (5.4, 8.0), (5.3, 1.6)], w, alpha=0.8)
    # Cutting edge.
    a.stroke([(4.3, 0.6), (3.8, 2.0), (3.6, 8.0), (3.2, 18.0), (3.2, 26.8)], 0.55, e)
    # Hamon (temper line).
    a.stroke([(4.6, 2.4), (4.4, 6.0), (4.2, 10.0), (4.0, 14.0), (3.9, 18.0), (3.9, 22.0), (3.9, 26.4)], 0.25,
             "#ffffff" if not hannya else "#6a1028", alpha=0.6)
    if hannya:
        # Jagged crimson edge teeth.
        for y in range(3, 26, 3):
            x = 3.7 - (0.4 if y > 8 else 0.0)
            a.poly([(x, y - 0.6), (x - 1.8, y + 0.5), (x, y + 1.6)], ("lin", "#ff5060", "#a01020", 0))
        a.stroke([(3.3, 1.0), (2.9, 26.8)], 0.3, "#ff6070", alpha=0.6)
    # Guard (tsuba).
    a.rect(0.4, 26.8, 8.6, 29.0, ("lin", "#f0d070", "#a07820", 90), radius=0.7)
    a.stroke([(1.2, 27.4), (7.6, 27.4)], 0.3, "#fff4c0", alpha=0.7)
    # Wrapped crimson handle.
    a.rect(2.6, 28.8, 6.4, 37.2, ("lin", "#d8384a", "#9a1e2e", 0), radius=0.8)
    for y in (30.0, 31.8, 33.6, 35.4):
        a.poly([(4.5, y - 0.7), (5.6, y + 0.15), (4.5, y + 1.0), (3.4, y + 0.15)], "#7e1828")
        a.ellipse(4.5, y + 0.15, 0.3, 0.3, "#f2b632" if hannya else "#e8d0b0")
    a.rect(2.6, 36.6, 6.4, 37.3, "#c9a040", radius=0.3)
    a.rim("#ffffff", 0.3)
    a.outline()
    if hannya:
        a.glow("#e0203c", radius=0.9, alpha=0.4)
    a.save("cleaver_hannya" if hannya else "cleaver")


def cleaver_rest(hannya):
    a = Art(30, 30)
    W, w, e = blade_colors(hannya)
    # Blade along the diagonal from the guard (~9,20) to the tip (~28.6,0.6); edge on the upper right.
    a.poly([(28.8, 0.4), (27.6, 3.2), (11.4, 20.0), (9.4, 18.0), (26.2, 1.8)], ("lin", W, w, 135))
    a.poly([(28.8, 0.4), (26.2, 1.8), (9.4, 18.0), (10.2, 18.8), (27.0, 2.6)], w, alpha=0.85)
    a.stroke([(28.6, 0.6), (27.8, 2.8), (11.0, 19.6)], 0.55, e)
    a.stroke([(27.0, 2.6), (10.8, 18.9)], 0.22, "#ffffff" if not hannya else "#6a1028", alpha=0.55)
    if hannya:
        for t in range(2, 16, 2):
            bx, by = 27.4 - t * 1.08, 3.6 + t * 1.1
            a.poly([(bx, by - 0.9), (bx + 1.6, by + 0.9), (bx - 0.6, by + 0.5)], ("lin", "#ff5060", "#a01020", 0))
    # Guard.
    a.poly([(5.6, 17.6), (6.8, 16.8), (11.6, 21.6), (10.8, 22.8)], ("lin", "#f0d070", "#a07820", 90))
    a.stroke([(6.4, 17.6), (10.8, 21.8)], 0.25, "#fff4c0", alpha=0.7)
    # Handle down to the bottom-left.
    a.stroke([(8.6, 21.4), (0.9, 27.2)], 1.9, ("lin", "#d8384a", "#9a1e2e", 90))
    for t in (0.18, 0.4, 0.62, 0.84):
        x, y = 8.6 + (0.9 - 8.6) * t, 21.4 + (27.2 - 21.4) * t
        a.poly([(x - 0.5, y - 0.6), (x + 0.6, y - 0.1), (x + 0.5, y + 0.6), (x - 0.6, y + 0.1)], "#7e1828")
    a.ellipse(0.9, 27.2, 0.9, 0.9, "#c9a040")
    a.rim("#ffffff", 0.3)
    a.outline()
    if hannya:
        a.glow("#e0203c", radius=0.9, alpha=0.4)
    a.save("cleaver_rest_hannya" if hannya else "cleaver_rest")


if __name__ == "__main__":
    for flag in (False, True):
        ren(flag)
        riku(flag)
        shun(flag)
        cleaver(flag)
        cleaver_rest(flag)
