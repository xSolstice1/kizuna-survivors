"""HD redraw of the World 3 (Pale Citadel of Souls) and World 4 (Rainward 13) enemies and bosses.

    python3 tools/hd/enemies_w34.py

Writes art/hd/w3/{fodder_a,fodder_b,ranged,blinker,elite,mini,boss,final}.png and the same for w4.
Geometry follows tools/gen_w3.py / tools/gen_w4.py (same coordinate space) so footprints match.
Deterministic.
"""
import math
import pathlib
import random
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np  # noqa: E402
from PIL import Image, ImageChops, ImageDraw, ImageFilter  # noqa: E402
from hdkit import Art, SS, _catmull, rgb  # noqa: E402


class A(Art):
    """hdkit Art plus: local gradients ("g", c1, c2, x0, y0, x1, y1), layers, pixel helpers, cel shading."""

    def _fill_layer(self, fill):
        if isinstance(fill, tuple) and fill and fill[0] == "g":
            _, c1, c2, x0, y0, x1, y1 = fill
            c1, c2 = np.array(rgb(c1), float), np.array(rgb(c2), float)
            ys, xs = np.mgrid[0:self.h * 4, 0:self.w * 4]
            X, Y = (xs + 0.5) / 4, (ys + 0.5) / 4
            dx, dy = x1 - x0, y1 - y0
            t = np.clip(((X - x0) * dx + (Y - y0) * dy) / (dx * dx + dy * dy + 1e-9), 0, 1)[..., None]
            arr = (c1 * (1 - t) + c2 * t).astype(np.uint8)
            return Image.fromarray(arr, "RGBA").resize((self.W, self.H), Image.BILINEAR)
        return super()._fill_layer(fill)

    # layers
    def layer(self):
        return A(self.w, self.h)

    def put(self, other):
        self.img.alpha_composite(other.img)

    # pixel-grid helpers (inclusive pixel indices, like the .px generators)
    def R(self, x0, y0, x1, y1, fill, radius=0.0, **k):
        self.rect(x0, y0, x1 + 1, y1 + 1, fill, radius=radius, **k)

    def px(self, x, y, fill, r=0.5, **k):
        self.ellipse(x + 0.5, y + 0.5, r, r, fill, **k)

    def line(self, pts, w, fill, **k):
        self.stroke([(x + 0.5, y + 0.5) for x, y in pts], w, fill, **k)

    def thick(self, pts, r0, r1, fill, smooth=True, clip=False, alpha=1.0):
        """Variable-radius stroke (continuous coordinates), radius r0 -> r1."""
        path = _catmull(pts, False, 16) if smooth and len(pts) > 2 else list(pts)
        if len(path) == 2:
            (x0, y0), (x1, y1) = path
            path = [(x0 + (x1 - x0) * t / 20, y0 + (y1 - y0) * t / 20) for t in range(21)]
        dense = [path[0]]
        for p in path[1:]:
            q = dense[-1]
            d = math.hypot(p[0] - q[0], p[1] - q[1])
            n = max(1, int(d / 0.1))
            for i in range(1, n + 1):
                dense.append((q[0] + (p[0] - q[0]) * i / n, q[1] + (p[1] - q[1]) * i / n))
        m = self._mask()
        d = ImageDraw.Draw(m)
        N = len(dense)
        for i, (x, y) in enumerate(dense):
            r = (r0 + (r1 - r0) * i / max(1, N - 1)) * SS
            d.ellipse([x * SS - r, y * SS - r, x * SS + r, y * SS + r], fill=255)
        self._paint(m, fill, clip, alpha)

    def soft(self, cx, cy, rx, ry, fill, alpha=0.6, blur=1.0):
        """A blurred ellipse painted on top (light, glow, mist). Not clipped."""
        m = self._mask()
        ImageDraw.Draw(m).ellipse([(cx - rx) * SS, (cy - ry) * SS, (cx + rx) * SS, (cy + ry) * SS], fill=255)
        m = m.filter(ImageFilter.GaussianBlur(blur * SS))
        self._paint(m, fill, alpha=alpha)

    def cel(self, color, alpha=0.3, dx=1.0, dy=1.0, blur=0.2):
        """Shadow band on the far (bottom-right by default) side of this layer's shapes."""
        a = self.img.getchannel("A")
        sh = Image.new("L", a.size, 0)
        sh.paste(a, (int(-dx * SS), int(-dy * SS)))
        m = ImageChops.subtract(a, sh)
        if blur:
            m = m.filter(ImageFilter.GaussianBlur(blur * SS))
        self._paint(m, color, clip=True, alpha=alpha)

    def lit(self, color="#ffffff", alpha=0.3, dx=0.8, dy=0.8, blur=0.2):
        """Light band on the near (top-left) side of this layer's shapes."""
        a = self.img.getchannel("A")
        sh = Image.new("L", a.size, 0)
        sh.paste(a, (int(dx * SS), int(dy * SS)))
        m = ImageChops.subtract(a, sh)
        if blur:
            m = m.filter(ImageFilter.GaussianBlur(blur * SS))
        self._paint(m, color, clip=True, alpha=alpha)

    def spec(self, x, y, r=0.35, color="#ffffff", alpha=0.95):
        self.ellipse(x, y, r, r, color, alpha=alpha)

    def eye(self, x, y, color, core="#ffffff", r=0.55, glow=1.4, galpha=0.55):
        """Glowing eye: soft halo + coloured dot + bright core."""
        self.soft(x, y, glow, glow, color, alpha=galpha, blur=glow * 0.6)
        self.ellipse(x, y, r, r * 0.9, color)
        self.ellipse(x - r * 0.15, y - r * 0.15, r * 0.45, r * 0.4, core)

    def drop(self, x, y, s=1.0, color="#8ac0e0", alpha=0.85):
        """Rain drop / drip: small teardrop."""
        self.blob([(x, y - 1.1 * s), (x + 0.45 * s, y + 0.1 * s), (x, y + 0.55 * s), (x - 0.45 * s, y + 0.1 * s)],
                  color, alpha=alpha)
        self.spec(x - 0.1 * s, y - 0.05 * s, 0.13 * s, "#ffffff", 0.8)

    def streak(self, x, y, ln=2.0, color="#8ac0e0", alpha=0.55, w=0.35):
        """Slanted rain streak."""
        self.stroke([(x, y), (x - ln * 0.25, y + ln)], w, color, alpha=alpha, smooth=False)


# ═══════════════════════════════ WORLD 3 ═══════════════════════════════

def w3_fodder_a():
    # Lost Soul: blue teardrop soul flame, tail streaming up-left, sad face.
    a = A(16, 16)
    L = a.layer()
    F, f, W = "#3a7fd6", "#7cc4ff", "#dff3ff"
    L.thick([(9.6, 7.0), (9.6, 3.8), (9.0, 1.3)], 1.7, 0.25, F)
    L.thick([(9.0, 10.0), (6.0, 6.8), (3.4, 3.6), (1.3, 1.3)], 3.6, 0.3, F)
    L.ellipse(9, 10.5, 5.6, 5, F)
    L.cel("#1d3f8c", 0.55, 1.1, 1.1, 0.3)
    L.thick([(9.0, 10.5), (6.0, 7.3), (3.0, 3.8)], 2.3, 0.35, ("g", f, "#5aa6f0", 8, 9, 2, 3))
    L.thick([(9.6, 7.5), (9.5, 4.8)], 0.8, 0.2, f)
    L.ellipse(9.3, 10.8, 4.4, 3.9, ("g", "#a6dbff", f, 6, 8, 11, 14))
    L.ellipse(6.6, 8.4, 1.3, 1.1, W, alpha=0.9)
    L.spec(5.6, 7.4, 0.4)
    # face
    for ex in (8.5, 11.5):
        L.ellipse(ex, 11.0, 0.62, 0.95, "#101c36")
        L.spec(ex - 0.15, 10.6, 0.26)
    L.stroke([(7.2, 9.75), (8.9, 9.15)], 0.42, "#101c36")
    L.stroke([(11.1, 9.15), (12.8, 9.75)], 0.42, "#101c36")
    L.stroke([(9.7, 14.2), (10.5, 13.65), (11.3, 14.2)], 0.42, "#101c36")
    L.blob([(8.4, 12.0), (8.75, 12.9), (8.4, 13.3), (8.05, 12.9)], "#ffffff")
    L.soft(7.3, 12.6, 0.8, 0.45, "#ff9ad0", 0.35, 0.3)
    L.soft(12.7, 12.6, 0.8, 0.45, "#ff9ad0", 0.35, 0.3)
    a.put(L)
    a.outline()
    a.glow("#5ab0ff", 0.7, 0.4)
    a.save("w3/fodder_a")


def w3_fodder_b():
    # Masked Hound: four-legged shadow beast with a cracked bone mask, facing right.
    a = A(16, 16)
    N, n, Lc = "#2e2742", "#1f1a2e", "#463c62"
    B, b, E, X = "#e2dccb", "#aaa390", "#7fe8ff", "#14121c"
    body = a.layer()
    # far legs (darker), then near legs
    for x in (2, 9):
        body.thick([(x + 1.0, 10.5), (x + 0.9, 13.0), (x + 1.2, 14.6)], 0.95, 0.8, n)
    body.thick([(1.8, 8.0), (0.6, 6.2), (0.5, 4.2), (1.2, 3.2)], 1.5, 0.35, N)   # smoky tail
    body.thick([(2.2, 8.6), (0.7, 9.2)], 0.9, 0.3, N)
    for sx in (5.0, 8.0):
        body.thick([(sx, 6.4), (sx + 0.1, 4.6), (sx + 0.5, 3.0)], 0.9, 0.2, N)    # smoke spines
    body.ellipse(7, 9, 5.5, 3.4, ("g", Lc, N, 7, 5.6, 7, 9.5))
    for x in (4, 11):
        body.thick([(x + 1.0, 10.5), (x + 0.9, 13.0), (x + 1.2, 14.6)], 1.05, 0.9, N)
        body.ellipse(x + 1.5, 14.7, 1.15, 0.6, N)
    body.cel(n, 0.7, 0.0, 1.2, 0.3)
    body.lit(Lc, 0.7, 0.0, 0.6)
    body.ellipse(12, 7, 3.5, 3, N)
    a.put(body)
    # bone mask
    m = a.layer()
    m.blob([(10.2, 4.3), (12.0, 3.8), (14.2, 4.1), (15.4, 6.0), (15.6, 8.4), (14.0, 9.7), (12.0, 9.9), (10.3, 8.3)],
           ("g", "#f2eee2", B, 11, 4, 14, 9))
    m.thick([(10.6, 4.4), (10.9, 2.6), (11.2, 1.0)], 0.95, 0.2, B)               # ear horn
    m.cel(b, 1.0, 0.0, 1.1, 0.15)
    m.cel(b, 0.6, 1.0, 0.0, 0.2)
    m.stroke([(11.0, 3.4), (11.2, 1.6)], 0.3, b, clip=True)
    m.ellipse(12.4, 6.6, 0.7, 0.75, X)                                            # back eye hole
    m.ellipse(13.9, 6.5, 1.15, 0.75, X)
    m.ellipse(14.1, 6.5, 0.7, 0.45, E)
    m.spec(13.8, 6.3, 0.2)
    m.stroke([(14.4, 4.2), (14.9, 5.3), (14.6, 6.0)], 0.3, X)                     # crack
    m.stroke([(14.6, 7.2), (15.2, 8.3)], 0.25, X)
    m.stroke([(13.2, 9.1), (15.2, 9.0)], 0.3, "#6e6858")                          # jaw line
    a.put(m)
    a.outline()
    a.soft(14.1, 6.5, 1.2, 0.9, E, 0.45, 0.6)
    a.ellipse(14.1, 6.5, 0.45, 0.32, "#e8fdff")
    a.save("w3/fodder_b")


def w3_ranged():
    # Pale Archer: ghostly archer in tattered white robes with a glowing spirit bow.
    a = A(16, 16)
    R, r, q, X, E, G, g = "#cfd7e8", "#98a4bf", "#6c7690", "#161a28", "#8fe0ff", "#5aa8ff", "#cdeaff"
    robe = a.layer()
    robe.blob([(5.0, 4.6), (9.4, 4.8), (10.8, 8.5), (11.3, 12.6), (10.6, 15.2), (9.6, 14.0), (8.6, 15.7),
               (7.5, 14.1), (6.3, 15.8), (5.0, 14.1), (3.6, 15.7), (2.5, 14.0), (2.3, 11.8), (3.3, 7.6)],
              ("g", "#e6ecf7", R, 4, 5, 8, 14))
    robe.ellipse(7, 4, 3.3, 3.5, ("g", "#eef2fa", R, 5, 1, 9, 7))
    robe.cel(r, 0.9, 1.6, 0.0, 0.3)
    robe.cel(q, 0.6, 0.0, 1.4, 0.3)
    robe.blob([(3.1, 8.0), (4.8, 8.2), (5.0, 13.6), (3.8, 15.0), (2.9, 13.4)], q, clip=True, alpha=0.75)  # back fold
    robe.stroke([(6.5, 9.5), (6.0, 14.5)], 0.35, r, clip=True)
    robe.stroke([(8.8, 9.0), (9.2, 14.2)], 0.35, r, clip=True)
    robe.ellipse(8.1, 4.6, 2.0, 2.2, ("g", "#06080f", X, 8, 3, 8, 6))           # shadowed face
    robe.stroke([(4.7, 2.2), (6.4, 1.2), (8.6, 1.4)], 0.4, "#ffffff", alpha=0.6)
    a.put(robe)
    arm = a.layer()
    arm.R(6, 7, 11, 8, r, radius=0.9)
    arm.R(10, 7.6, 11.6, 8.8, R, radius=0.6)
    arm.lit("#ffffff", 0.35, 0, 0.5)
    a.put(arm)
    bow = a.layer()
    pts = [(13.5 + 1.6 * math.sin((y - 1) / 13 * math.pi), y + 0.5) for y in range(1, 15)]
    bow.stroke([(12.5, 1.2)] + pts[1:-1] + [(12.5, 14.8)], 0.95, ("g", "#9ed2ff", G, 13, 1, 13, 15))
    bow.stroke([(12.6, 1.4), (12.6, 14.6)], 0.32, g, smooth=False)
    a.put(bow)
    a.outline()
    # spirit arrow + glows drawn after outline (it is light)
    a.soft(13.5, 8.0, 2.2, 6.5, G, 0.35, 1.2)
    a.stroke([(6.0, 7.5), (15.2, 7.5)], 0.5, g, smooth=False)
    a.poly([(14.2, 6.4), (15.9, 7.5), (14.2, 8.6), (14.7, 7.5)], "#ffffff")
    a.soft(15.0, 7.5, 1.0, 0.8, E, 0.6, 0.5)
    a.eye(8.4, 4.6, E, r=0.38, glow=0.9, galpha=0.6)
    a.eye(9.5, 4.6, E, r=0.38, glow=0.9, galpha=0.6)
    a.eye(5.6, 7.6, E, r=0.3, glow=0.7, galpha=0.5)
    a.save("w3/ranged")


def w3_blinker():
    # Shade Dancer: thin black silhouette on tiptoe, white mask, two glowing ribbons trailing back.
    a = A(16, 16)
    N, Lc, M, m, X, E = "#28213a", "#5a4c82", "#ece8f4", "#b6b0c6", "#14121c", "#5ad0ff"
    s = a.layer()
    s.thick([(10.0, 10.5), (7.5, 11.6), (4.8, 12.6)], 0.75, 0.4, N)              # leg kicked back
    s.thick([(10.5, 5.2), (8.6, 4.4), (6.4, 4.5)], 0.6, 0.38, N)                 # arm swept back
    s.thick([(11.3, 4.0), (11.0, 7.0), (10.6, 10.5)], 1.05, 0.85, N)             # torso
    s.blob([(10.0, 9.0), (11.4, 9.4), (10.8, 11.2), (8.4, 11.8)], N)             # skirt flare
    s.thick([(10.8, 10.5), (11.2, 13.0), (11.7, 15.3)], 0.65, 0.35, N)           # standing leg
    s.ellipse(12.4, 15.5, 0.75, 0.38, N)
    s.thick([(11.8, 5.5), (13.4, 3.4), (14.6, 1.4)], 0.55, 0.32, N)              # arm raised forward
    s.ellipse(11, 2.5, 2, 2.4, N)
    s.ellipse(14.7, 1.2, 0.45, 0.45, N)
    s.lit(Lc, 0.9, -0.7, 0.4)                                                    # rim on the front edge
    a.put(s)
    k = a.layer()
    k.blob([(11.2, 1.0), (12.5, 0.2), (13.7, 1.2), (13.9, 3.0), (12.6, 4.4), (11.3, 3.6)], ("g", "#ffffff", M, 11, 0, 13, 4))
    k.cel(m, 0.9, 0.6, 0.6)
    k.ellipse(12.6, 2.4, 0.5, 0.35, X)
    k.stroke([(13.3, 2.2), (13.9, 2.4)], 0.3, X)
    k.stroke([(12.2, 3.5), (13.0, 3.7)], 0.25, "#d46aa0")                        # painted smile
    a.put(k)
    a.outline()
    a.eye(12.6, 2.4, E, r=0.32, glow=0.9, galpha=0.6)
    # ribbons
    for (xe, ye, amp, drift, col, w) in ((6.0, 4.5, 1.2, -0.25, "#d2dcf6", 0.75), (8.6, 9.6, 1.2, 0.3, "#6f8fe6", 0.85)):
        pts = []
        for i in range(13):
            x = xe - i * xe / 12
            y = ye + amp * math.sin((xe - x) * 0.6) + drift * (xe - x)
            pts.append((max(0.3, x), y))
        a.thick(pts, w * 0.95, w * 0.5, col, alpha=0.3)
        a.thick(pts, w * 0.55, w * 0.3, col, alpha=1.0)
        a.thick(pts[:7], w * 0.2, w * 0.1, "#ffffff", alpha=0.6)
    a.save("w3/blinker")


def w3_elite():
    # Bone Sentinel: towering skeleton guard in pale armour with a halberd.
    a = A(32, 32)
    B, b, n = "#ddd8c8", "#b0aa98", "#7e7968"
    Ar, Lt, ad, q = "#9aa6bc", "#c6cfe0", "#6a7488", "#4a5266"
    C, E, X, H, S, s = "#2a2d40", "#6fd8ff", "#14121c", "#4a3a3a", "#d8e2f0", "#8e9ab0"
    # halberd
    h = a.layer()
    h.R(25, 3, 25, 31, ("g", "#6a5450", H, 25, 0, 26, 0), radius=0.4)
    for y in (9, 13):
        h.R(24.8, y, 26.2, y + 0.8, "#9a8a70")
    h.blob([(25.5, -0.2), (26.3, 2.0), (26.1, 3.6), (24.9, 3.6), (24.7, 2.0)], S)  # spike
    h.blob([(26.0, 3.0), (28.4, 2.0), (30.4, 1.9), (31.6, 4.5), (31.5, 7.5), (30.4, 10.4), (28.0, 9.3), (26.0, 8.4)],
           ("g", "#f2f6fc", S, 27, 3, 30, 8))
    h.blob([(28.2, 4.0), (30.2, 3.2), (30.9, 6.0), (30.2, 9.0), (28.2, 7.6)], s, alpha=0.7)
    h.stroke([(30.8, 2.6), (31.4, 5.5), (30.8, 9.4)], 0.35, "#ffffff", alpha=0.8)
    h.poly([(24.5, 5.0), (20.8, 6.1), (24.5, 7.2)], s)
    h.lit("#ffffff", 0.3, 0.5, 0.5)
    a.put(h)
    # legs
    lg = a.layer()
    lg.R(11, 25.5, 12, 29.5, ("g", B, b, 11, 0, 13, 0), radius=0.6)
    lg.R(18, 25.5, 19, 29.5, b, radius=0.6)
    lg.ellipse(12, 26.5, 1.3, 0.7, B)
    lg.ellipse(19, 26.5, 1.3, 0.7, b)
    lg.R(10, 22, 13, 25, ("g", Lt, Ar, 10, 0, 14, 0), radius=1.0)
    lg.R(17, 22, 20, 25, ("g", Ar, ad, 17, 0, 21, 0), radius=1.0)
    lg.blob([(8.8, 31.9), (9.2, 30.0), (12.0, 29.6), (14.0, 31.0), (14.0, 31.9)], ad)
    lg.blob([(16.8, 31.9), (17.2, 30.0), (20.0, 29.6), (22.0, 31.0), (22.0, 31.9)], q)
    lg.cel(q, 0.5, 0.7, 0.7)
    a.put(lg)
    # tabard
    t = a.layer()
    t.poly([(11, 17), (20, 17), (21, 25.5), (18.5, 23.3), (15.5, 26.5), (12.8, 23.3), (10, 25.5)], ("g", "#3a3e58", C, 15, 17, 15, 25))
    t.stroke([(15.5, 18), (15.5, 25.5)], 0.4, "#1c1e2c", clip=True)
    t.stroke([(11.2, 18.2), (20.0, 18.2)], 0.5, "#c9b27a", clip=True)
    a.put(t)
    # left arm (hangs)
    la = a.layer()
    la.thick([(7.0, 12.0), (6.9, 16.5), (7.0, 20.5)], 1.0, 0.85, ("g", B, b, 6, 0, 8, 0))
    la.ellipse(7.0, 16.5, 1.1, 0.7, B)
    la.blob([(5.8, 20.3), (8.2, 20.3), (8.4, 21.8), (7.6, 22.6), (6.0, 22.4)], b)
    la.stroke([(6.4, 21.3), (6.3, 22.5)], 0.3, n)
    la.stroke([(7.4, 21.3), (7.4, 22.6)], 0.3, n)
    a.put(la)
    # torso: breastplate halves with ribcage between
    tr = a.layer()
    tr.R(10, 9, 21, 17, Ar, radius=1.2)
    tr.R(10, 9, 12, 17, ("g", "#e0e6f0", Lt, 10, 9, 12, 17), radius=1.0)
    tr.R(19, 9, 21, 17, ad, radius=1.0)
    tr.R(13, 10, 18, 16, "#0c0d16", radius=0.6)
    for i, y in enumerate((10.2, 12.2, 14.2)):
        tr.thick([(13.2, y + 0.4), (15.5, y + 0.1), (17.8, y + 0.4)], 0.42, 0.38, B if i == 0 else b)
    tr.R(15, 10, 16, 16.5, ("g", B, b, 0, 10, 0, 17), radius=0.5)
    for y in (11.5, 13.5, 15.5):
        tr.stroke([(15.2, y), (16.8, y)], 0.25, n)
    tr.R(10, 16.6, 21, 17.6, ad, radius=0.4)
    tr.lit("#ffffff", 0.25, 0.5, 0.5)
    a.put(tr)
    # pauldrons
    p = a.layer()
    p.ellipse(8, 10, 3.5, 2.6, ("g", "#eef2f8", Lt, 6, 8, 9, 12))
    p.ellipse(23, 10, 3.5, 2.6, ("g", Lt, Ar, 21, 8, 25, 12))
    p.cel(ad, 0.8, 0.0, 0.9)
    p.stroke([(5.2, 11.4), (10.8, 11.4)], 0.5, ad)
    p.stroke([(20.2, 11.4), (26.2, 11.4)], 0.5, q)
    p.stroke([(6.0, 8.6), (8.0, 7.8), (10.0, 8.4)], 0.4, "#ffffff", alpha=0.7)
    p.spec(21.7, 8.6, 0.35)
    a.put(p)
    # right arm gripping the halberd
    ra = a.layer()
    ra.thick([(22.8, 12.0), (22.8, 14.5), (23.6, 16.5)], 0.95, 0.85, b)
    ra.ellipse(25.2, 16.6, 1.6, 1.4, ("g", B, b, 24, 15, 26, 18))
    for y in (15.9, 16.9, 17.9):
        ra.stroke([(24.4, y), (26.2, y)], 0.25, n)
    a.put(ra)
    # skull + crested helm
    k = a.layer()
    k.ellipse(16, 5, 4.2, 4.2, ("g", "#f0ece0", B, 14, 2, 18, 9))
    k.R(13, 7, 19, 9, B, radius=1.2)
    k.cel(b, 0.9, 1.0, 0.6)
    k.stroke([(13.9, 8.6), (18.6, 8.6)], 0.4, n)
    for x in (14.5, 15.7, 16.9, 18.1):
        k.stroke([(x, 8.4), (x, 9.6)], 0.25, n)
    k.blob([(13.8, 4.0), (15.6, 3.9), (15.8, 6.5), (14.0, 6.8)], X)
    k.blob([(16.8, 3.9), (18.8, 4.0), (18.8, 6.8), (17.0, 6.5)], X)
    k.poly([(16.5, 6.9), (17.0, 7.9), (16.0, 7.9)], X)
    k.poly([(11.5, 4.2), (16.0, -0.3), (20.5, 4.2)], ("g", Lt, Ar, 13, 1, 19, 4))
    k.R(11.6, 3, 20.4, 3.9, ("g", "#eef2f8", Lt, 12, 0, 20, 0), radius=0.4)
    k.R(15, -0.5, 16, 2.5, ("g", "#ffffff", Lt, 0, 0, 0, 3), radius=0.5)
    k.R(12, 4, 12.9, 6.9, ad, radius=0.3)
    k.R(20, 4, 20.9, 6.9, q, radius=0.3)
    k.stroke([(12.4, 2.6), (15.0, 0.6)], 0.3, "#ffffff", alpha=0.7)
    a.put(k)
    a.outline()
    for x, y in ((14.8, 5.4), (17.9, 5.4)):
        a.eye(x, y, E, r=0.5, glow=1.6, galpha=0.6)
    a.save("w3/elite")


def w3_mini():
    # Captain of the Ninth Gate: gate-shaped tower shield, long spear, white cloak, crested kabuto.
    a = A(32, 32)
    W, w, v = "#d6dbe6", "#a8afc2", "#7a8298"
    N, n, P, Y, y = "#2c3350", "#1f2438", "#46507a", "#c9b27a", "#8a7648"
    S, s, T, t, D, G, g, E, Hs, Bl = ("#b4b9c9", "#828799", "#1e2030", "#3a3f58", "#141826",
                                       "#5ab4ff", "#2f5f90", "#9fe8ff", "#5a4636", "#e4ecf8")
    sp = a.layer()
    sp.R(5, 4, 5, 30.6, ("g", "#7a6250", Hs, 5, 0, 6, 0), radius=0.4)
    sp.blob([(5.5, -0.2), (6.9, 3.4), (5.5, 6.2), (4.1, 3.4)], ("g", "#ffffff", Bl, 5, 0, 6, 0))
    sp.stroke([(5.5, 0.6), (5.5, 5.4)], 0.3, "#9aa6c0")
    sp.R(4, 6, 6, 6.9, ("g", "#f0dca0", Y, 0, 6, 0, 7), radius=0.4)
    sp.thick([(4.4, 7.0), (3.9, 8.8)], 0.5, 0.25, W)
    sp.thick([(6.6, 7.0), (7.0, 9.3)], 0.5, 0.25, W)
    a.put(sp)
    # cloak
    c = a.layer()
    c.blob([(8, 8.4), (15, 8.2), (15.8, 18), (16, 29.2), (13.5, 29.8), (11, 30.2), (9.2, 28.6), (7, 28.4),
            (4.4, 30.4), (2.0, 30.6), (3.0, 25), (4.6, 18)], ("g", "#eef1f8", W, 8, 9, 12, 28))
    c.blob([(4.6, 19.0), (7.8, 12.5), (9.0, 28.3), (6.8, 28.2), (2.2, 30.4), (3.2, 25.0)], w, clip=True)
    for x in (5.4, 8.2):
        c.stroke([(x + 0.4, 23.5), (x, 30.0)], 0.5, v, clip=True)
    c.cel(v, 0.5, 0.0, 0.8)
    a.put(c)
    # legs / greaves
    l = a.layer()
    l.R(11, 25, 13, 29, ("g", P, N, 11, 0, 14, 0), radius=0.5)
    l.R(14, 25, 16, 29, n, radius=0.5)
    l.blob([(9.8, 31.9), (10.2, 30.0), (13.6, 29.8), (14.0, 31.9)], n)
    l.blob([(13.8, 31.9), (14.2, 30.0), (17.4, 29.8), (18.0, 31.9)], D)
    a.put(l)
    # armoured torso
    tr = a.layer()
    tr.R(9, 11, 17, 24, N, radius=1.0)
    tr.R(9, 11, 11, 24, ("g", "#5a6698", P, 9, 0, 12, 0), radius=1.0)
    tr.R(9, 18, 17, 19.9, ("g", "#e8d49c", y, 0, 18, 0, 20))
    tr.ellipse(13, 19, 0.9, 0.9, "#f6e8b8")
    for yy in (21.3, 23.3):
        tr.stroke([(9.3, yy), (16.7, yy)], 0.45, n, clip=True)
    for yy in (13.3, 15.3):
        tr.stroke([(9.3, yy), (16.7, yy)], 0.35, n, clip=True, alpha=0.7)
    tr.cel(n, 0.6, 1.0, 0.0)
    tr.ellipse(9, 11, 3, 2.2, ("g", "#6a76a8", P, 7, 9, 10, 13))
    tr.R(7, 12, 11, 12.9, Y, radius=0.4)
    tr.R(6, 12, 7.4, 17.5, N, radius=0.6)
    tr.R(5, 16, 7.6, 17.6, ("g", "#e8d49c", Y, 0, 16, 0, 18), radius=0.6)
    a.put(tr)
    # head: kabuto with tall golden crest and face guard
    hd = a.layer()
    hd.poly([(12.2, 4.5), (13.0, -0.3), (14.2, 4.5)], ("g", "#fff0b8", Y, 12, 0, 14, 4))
    hd.blob([(13.0, 3.6), (15.0, 1.4), (17.4, -0.2), (16.0, 2.6), (15.2, 4.4)], ("g", "#fff0b8", Y, 13, 0, 17, 4))
    hd.ellipse(13, 7, 3.6, 3.4, ("g", P, N, 11, 4, 15, 9))
    hd.R(11, 6, 16, 9, ("g", "#262c44", D, 0, 6, 0, 10), radius=0.6)
    hd.R(9, 5, 17, 5.9, ("g", "#f0dca0", Y, 0, 5, 0, 6), radius=0.45)
    hd.stroke([(10.6, 4.4), (12.4, 3.7)], 0.35, "#8890b0", alpha=0.8)
    hd.blob([(10, 9.1), (16.6, 9.0), (16.4, 10.8), (13, 11.4), (10, 10.9)], ("g", "#ffffff", W, 0, 9, 0, 11))
    a.put(hd)
    # gate shield
    sh = a.layer()
    sh.R(17, 14, 30, 30, D)
    sh.R(19, 15, 28, 29, ("g", "#1c2236", "#10131e", 19, 15, 28, 29))
    sh.R(17, 14, 18, 29.9, ("g", "#d6dae6", S, 17, 0, 19, 0), radius=0.3)
    sh.R(29, 14, 30, 29.9, ("g", S, s, 29, 0, 31, 0), radius=0.3)
    sh.R(19, 15, 28, 15.6, s)
    sh.stroke([(23.5, 16), (23.5, 29.5)], 0.4, n)
    for yy in (18.5, 25.5):
        for xx in (20.5, 22.5, 24.5, 26.5):
            sh.ellipse(xx, yy, 0.38, 0.38, t)
            sh.spec(xx - 0.1, yy - 0.1, 0.12, "#8a90b0", 0.8)
    sh.ellipse(23.5, 21.5, 3.2, 3.2, g)
    sh.ellipse(23.5, 21.5, 2.3, 2.3, D)
    sh.R(17, 29, 30, 29.8, s)
    sh.R(16.6, 30, 30.4, 31.0, T, radius=0.3)
    # lintel: black tile roof with upswept ends
    sh.blob([(15.4, 10.6), (17.5, 11.4), (23.5, 11.2), (29.5, 11.4), (31.6, 10.6), (30.4, 13.2), (29.8, 14.4),
             (17.2, 14.4), (16.6, 13.2)], ("g", t, T, 0, 11, 0, 14))
    sh.stroke([(15.6, 10.8), (17.6, 11.4), (29.4, 11.4), (31.4, 10.8)], 0.4, "#5a6080")
    for xx in range(18, 30, 2):
        sh.stroke([(xx + 0.5, 12.0), (xx + 0.5, 14.2)], 0.25, "#0e1018")
    a.put(sh)
    a.outline()
    # glowing sigil + visor (light, after outline)
    a.soft(23.5, 21.5, 4.0, 4.0, G, 0.4, 1.4)
    a.stroke([(23.5, 19.0), (23.5, 24.0)], 0.9, G, smooth=False)
    a.stroke([(21.0, 21.5), (26.0, 21.5)], 0.9, G, smooth=False)
    a.poly([(23.5, 19.6), (25.0, 21.5), (23.5, 23.4), (22.0, 21.5)], "#7ac6ff", alpha=0.6)
    a.ellipse(23.5, 21.5, 0.9, 0.9, E)
    a.spec(23.3, 21.3, 0.4)
    a.soft(15.0, 7.5, 2.6, 0.9, E, 0.45, 0.6)
    a.R(13, 7.1, 16.6, 7.9, E, radius=0.4)
    a.R(14.2, 7.3, 16.3, 7.65, "#e8fdff", radius=0.2)
    a.save("w3/mini")


def w3_boss():
    # The Hollow Monarch: hunched beast-king of black smoke and bone, horn crown, hollow chest with a blue core.
    a = A(48, 48)
    N, n, Lc, l = "#221c33", "#16121f", "#3b3258", "#2e2745"
    B, b, o, X, E, C, c, W, wl = ("#e4dece", "#b8b19e", "#857f6e", "#0c0a12", "#6fd8ff", "#4ab0ff", "#2a5f9a",
                                  "#ffffff", "#a8e4ff")
    rng = random.Random(48)
    # smoke wisps rising from the back (soft, translucent)
    wsp = a.layer()
    for (x, y, r) in ((6, 18, 3), (4, 25, 2.5), (9, 12, 2.5), (3, 33, 2.5), (12, 9, 2)):
        wsp.ellipse(x, y, r, r, ("g", Lc, l, x - r, y - r, x + r, y + r))
    wsp.thick([(6, 16), (4, 11), (5.5, 7), (4.5, 4)], 1.6, 0.3, l)
    wsp.thick([(12, 8), (11, 5), (12.5, 2.5)], 1.2, 0.25, l)
    wsp.lit("#5a4c82", 0.6, 0.6, 0.6, 0.3)
    a.put(wsp)
    # smoke body + tattered hem
    bd = a.layer()
    bd.ellipse(22, 27, 15, 13, N)
    bd.ellipse(17, 22, 11, 9, N)
    bd.blob([(8, 33), (36, 33), (37.8, 40), (40, 47.4), (36, 44.5), (33, 43), (30, 46), (27, 47.4), (24, 44.5),
             (21, 43), (18, 46), (15, 47.4), (12.5, 44.5), (10, 43), (7.5, 46), (5, 47.4), (6, 40)], N)
    bd.ellipse(16, 18, 9, 6, ("g", Lc, N, 12, 13, 18, 24), alpha=0.9)
    bd.ellipse(14, 16, 6, 3, "#4a3f6c", alpha=0.6)
    for _ in range(22):
        x, y = rng.uniform(8, 36), rng.uniform(20, 44)
        bd.thick([(x, y), (x + rng.uniform(-1.5, 1.5), y + rng.uniform(1.5, 3.5))], 0.5, 0.15,
                 rng.choice([l, n]), clip=True, alpha=0.8)
    bd.cel(n, 0.85, 5.0, 2.0, 2.0)
    bd.lit("#5a4c82", 0.7, 0.7, 0.7, 0.3)
    a.put(bd)
    # bone spines along the back
    sp = a.layer()
    for (x, y) in ((9, 15), (13, 12), (18, 11)):
        sp.thick([(x + 0.6, y + 2.6), (x - 0.4, y - 0.4), (x - 1.8, y - 3.0)], 1.5, 0.2, ("g", B, b, x - 2, y - 3, x + 1, y + 3))
    sp.cel(o, 0.7, 0.5, 0.5)
    a.put(sp)
    # left (back) arm, smoky, bone claws
    la = a.layer()
    la.blob([(10, 26), (12.2, 31), (9.6, 40), (5, 40.5), (5.6, 33)], ("g", Lc, l, 6, 28, 10, 40))
    for k in range(3):
        la.thick([(5.5 + k * 2, 40), (5.1 + k * 2, 42.4), (4.0 + k * 2, 44.6)], 0.6, 0.15, b)
    a.put(la)
    # shoulder plate
    sh = a.layer()
    sh.ellipse(29, 21, 6, 3.5, ("g", "#f2eee2", B, 26, 18, 31, 24))
    sh.R(24, 22, 34, 23.6, b, radius=0.8)
    sh.cel(o, 0.8, 0.0, 0.9)
    sh.stroke([(24.5, 20.0), (27.5, 18.4), (31, 18.2)], 0.4, W, alpha=0.6)
    a.put(sh)
    # hollow chest
    ch = a.layer()
    ch.ellipse(24, 30, 7, 6.5, o)
    ch.ellipse(24, 30, 6, 5.5, ("rad", "#1a2440", X, (0.5, 0.62)))
    ch.ellipse(24, 30, 5.6, 5.1, ("g", "#1d2a4a", X, 24, 30, 24, 35.5), alpha=0.6)
    a.put(ch)
    a.soft(24, 30, 4.2, 4.2, C, 0.55, 1.6)
    a.ellipse(24, 30, 3, 3, ("g", C, c, 22, 28, 26, 32))
    a.ellipse(24, 30, 2.2, 2.2, ("g", wl, C, 23, 29, 25, 32))
    a.ellipse(24, 30, 1.2, 1.2, W)
    for ang in range(0, 360, 45):
        r = math.radians(ang)
        a.stroke([(24 + 1.5 * math.cos(r), 30 + 1.5 * math.sin(r)), (24 + 3.4 * math.cos(r), 30 + 3.4 * math.sin(r))],
                 0.3, wl, alpha=0.7, smooth=False)
    rb = a.layer()
    for i, y in enumerate((26.4, 30.4, 34.4)):
        bow = -0.6 if i == 0 else 0.6 if i == 2 else 0.0
        rb.thick([(18.0, y + 0.3), (24, y + bow), (30.0, y + 0.3)], 0.55, 0.5, ("g", B, b, 0, y - 0.6, 0, y + 0.6))
    rb.cel(o, 0.6, 0.0, 0.4, 0.1)
    a.put(rb)
    # right arm reaching forward with long bone claws
    ra = a.layer()
    ra.blob([(31, 23), (36.5, 24.8), (40.5, 29), (42.5, 34), (38.5, 36.5), (34.5, 31)], ("g", Lc, N, 33, 24, 40, 35))
    ra.ellipse(41, 35, 3, 2.5, n)
    ra.lit("#5a4c82", 0.7, 0.6, 0.6)
    a.put(ra)
    cl = a.layer()
    for k in range(4):
        cl.thick([(39.5 + k, 37), (40 + k * 1.3, 39.6), (40.5 + k * 1.5, 42.6 + (k % 2))], 0.62, 0.12,
                 ("g", B, b, 39, 37, 44, 43))
    cl.thick([(43.2, 36.0), (45.0, 37.6), (46.6, 40.4)], 0.62, 0.12, B)
    cl.cel(o, 0.6, 0.4, 0.4, 0.1)
    a.put(cl)
    # crown of horns (behind the mask)
    cr = a.layer()
    for (bx, tx, ty) in ((25, 19, 0), (29, 27, 1), (33, 34, 0), (37, 42, 1), (40, 47, 6)):
        mx, my = (bx + tx) / 2 + (0.8 if tx < bx else -0.8 if tx > bx else 0), (9 + ty) / 2
        cr.thick([(bx, 9.0), (mx, my), (tx, ty + 0.4)], 2.0, 0.2, ("g", "#f6f2e6", B, bx, 9, tx, ty))
    cr.cel(b, 0.9, 0.8, 0.0, 0.2)
    cr.R(25, 7, 40, 8.8, ("g", "#a8a08a", o, 0, 7, 0, 9), radius=0.6)
    a.put(cr)
    # head: big bone mask facing right
    hd = a.layer()
    hd.ellipse(32, 13, 8, 7, ("g", "#f6f2e6", B, 28, 8, 36, 19))
    hd.blob([(33, 15), (39, 14.0), (44, 13.8), (46.2, 16.6), (45.6, 19), (42, 21.2), (32, 21.2)], B)
    hd.cel(b, 1.0, 1.6, 1.4, 0.4)
    hd.cel(o, 0.6, 0.6, 0.6, 0.15)
    hd.stroke([(28.2, 18.5), (36, 18.6), (44.6, 18.4)], 0.55, o)
    hd.R(32.8, 20.6, 43.2, 21.6, o, radius=0.4)
    a.put(hd)
    th = a.layer()
    for x in range(33, 45, 2):
        th.poly([(x - 0.1, 18.7), (x + 1.1, 18.7), (x + 0.5, 21.0)], "#fbf8f0")
        th.poly([(x + 1.0, 18.8), (x + 2.0, 18.8), (x + 1.5, 20.0)], X)
    a.put(th)
    hd2 = a.layer()
    hd2.blob([(29, 10), (32, 9.1), (34.4, 9.2), (36, 11.8), (33.4, 13.2), (31, 13)], X)
    hd2.blob([(38, 11), (41.2, 11), (41.4, 13.2), (38.2, 13.3)], X)
    a.put(hd2)
    a.stroke([(28.4, 7.3), (29.6, 10.0), (31.3, 13.4), (30.2, 15.5), (29.4, 17.4)], 0.42, o)
    a.stroke([(31.3, 13.4), (32.6, 14.6)], 0.3, o)
    for yy in range(14, 18):
        a.stroke([(26.4 + (yy % 2), yy + 0.1), (27.2 + (yy % 2), yy + 0.9)], 0.3, o)
    a.stroke([(29, 6.8), (33, 6.2), (36.5, 6.8)], 0.5, W, alpha=0.5)
    a.outline()
    # glowing eyes + crack glow
    a.soft(33.2, 10.9, 2.8, 1.9, E, 0.6, 1.0)
    a.blob([(31.8, 10.2), (34.2, 10.0), (34.8, 11.0), (32.4, 11.6)], E)
    a.ellipse(33.2, 10.6, 0.6, 0.45, W)
    a.soft(40.2, 12.3, 1.6, 1.3, E, 0.5, 0.7)
    a.ellipse(40.3, 12.3, 0.6, 0.55, E)
    a.spec(40.1, 12.1, 0.25)
    a.soft(30.4, 14.3, 0.9, 0.9, C, 0.7, 0.4)
    a.spec(30.4, 14.3, 0.3, "#d8f6ff")
    a.save("w3/boss")


def w3_final():
    # Seraph of the Pale Throne: six white wings, halo of blades, glowing blue eyes, seated on a broken throne.
    a = A(48, 48)
    W, w, v, u = "#eef1f8", "#c2cadb", "#8e98b2", "#5f6884"
    Rr, r, S, s, d, Lc = "#f6f8fc", "#b9c2d6", "#80869c", "#5c6276", "#3c4052", "#a9aec0"
    Y, y, H, h, E, X, F, f, G = ("#ffe9a8", "#c8a860", "#dfe8ff", "#6f98d8", "#5ad8ff", "#1a1f33", "#f0e4d8",
                                 "#c8b8b0", "#3d8ee8")
    cx = 24
    # throne
    th = a.layer()
    th.poly([(13, 22), (16, 17), (19, 20), (22, 15), (26, 18), (29, 14), (33, 19), (35, 22), (35, 40), (13, 40)],
            ("g", "#969cb2", S, 24, 14, 24, 40))
    th.R(13, 22, 15, 40, s)
    th.R(33, 22, 35, 40, s)
    th.R(8, 38, 40, 42, ("g", Lc, S, 0, 38, 0, 42), radius=0.6)
    th.R(6, 43, 42, 47, ("g", S, s, 0, 43, 0, 47), radius=0.6)
    th.R(6, 46, 42, 47.9, d, radius=0.5)
    th.stroke([(8.4, 38.4), (40.6, 38.4)], 0.5, "#c8ccda")
    th.stroke([(6.4, 43.4), (42.6, 43.4)], 0.5, Lc)
    th.stroke([(30.5, 43.5), (32.0, 45.5), (33.5, 47.5)], 0.4, d)
    th.stroke([(14.5, 39.5), (12.5, 42.5)], 0.4, d)
    th.stroke([(38.5, 39.5), (40.5, 42.5)], 0.4, d)
    th.stroke([(20, 41), (22, 41.6)], 0.3, s)
    a.put(th)

    def wing(L, root, tip, flen, cols, lead, nf):
        (rx, ry), (tx, ty) = root, tip
        dx, dy = tx - rx, ty - ry
        ln = math.hypot(dx, dy)
        ax, ay = dx / ln, dy / ln
        nx, ny = -ay, ax
        if ny < 0:
            nx, ny = -nx, -ny
        for k in range(1, nf + 1):
            t = k / nf
            bx, by = rx + dx * t, ry + dy * t
            fl = flen * (0.55 + 0.6 * t)
            fx, fy = nx + ax * 0.55 * t, ny + ay * 0.55 * t
            mm = math.hypot(fx, fy)
            fx, fy = fx / mm, fy / mm
            ex, ey = bx + fx * fl, by + fy * fl
            p1 = (bx - ax * 2.2, by - ay * 2.2)
            p2 = (bx + ax * 1.6, by + ay * 1.6)
            p3 = (ex + ax * 0.6, ey + ay * 0.6)
            p4 = (ex - ax * 1.0, ey - ay * 1.0)
            tipp = (ex + fx * 0.6, ey + fy * 0.6)
            col = cols[k % 2]
            L.blob([p1, p2, ((p2[0] + p3[0]) / 2, (p2[1] + p3[1]) / 2), p3, tipp, p4], col)
            L.stroke([(bx, by), (ex - fx * 0.8, ey - fy * 0.8)], 0.35, cols[2], alpha=0.8)
        L.thick([(rx, ry), (rx + dx * 0.5 - nx * 0.6, ry + dy * 0.5 - ny * 0.6), (tx, ty)], 1.2, 0.5, lead)
        L.ellipse(tx, ty, 0.6, 0.6, Rr)

    lw = a.layer()
    for side in (-1, 1):
        wing(lw, (cx + side * 3, 26), (cx + side * 20, 33), 6, ("#9aa4bc", v, u), w, 5)
    lw.cel(u, 0.6, 0.0, 0.8)
    a.put(lw)
    uw = a.layer()
    for side in (-1, 1):
        wing(uw, (cx + side * 3, 18), (cx + side * 21, 2), 8, (w, "#b0b9cc", v), W, 6)
    uw.cel(v, 0.6, 0.0, 0.8)
    a.put(uw)
    mw = a.layer()
    for side in (-1, 1):
        wing(mw, (cx + side * 3, 21), (cx + side * 23, 17), 9, ("#f4f6fb", "#d6dce8", v), Rr, 6)
    mw.cel(v, 0.6, 0.0, 0.8)
    mw.lit("#ffffff", 0.5, 0.0, 0.6)
    a.put(mw)
    # halo of blades
    hl = a.layer()
    arc = [(cx + 8 * math.cos(math.radians(t)), 15.5 + 8 * math.sin(math.radians(t))) for t in range(146, 395, 6)]
    hl.stroke(arc, 1.1, ("g", "#fff4c8", y, 24, 7, 24, 20))
    for i in range(9):
        ang = math.pi + i * math.pi / 8
        bx, by = cx + 8.5 * math.cos(ang), 15.5 + 8.5 * math.sin(ang)
        tx_, ty_ = cx + 13.8 * math.cos(ang), 15.5 + 13.8 * math.sin(ang)
        qx, qy = -math.sin(ang) * 1.1, math.cos(ang) * 1.1
        hl.poly([(bx + qx, by + qy), (tx_, ty_), (bx - qx, by - qy)], ("g", "#ffffff", H, bx, by, tx_, ty_))
        hl.poly([(bx, by), (tx_, ty_), (bx - qx, by - qy)], "#a8bce6", alpha=0.7)
        hl.stroke([(bx, by), ((bx + tx_) / 2, (by + ty_) / 2)], 0.3, h)
    a.put(hl)
    # robed body
    bd = a.layer()
    bd.blob([(19, 21), (29, 21), (30.4, 30), (31.4, 39), (24, 39.6), (16.8, 39)], ("g", "#ffffff", Rr, 24, 21, 24, 39))
    bd.blob([(25, 22), (29, 21), (30.4, 30), (31.4, 39), (26, 39.4)], r, clip=True, alpha=0.9)
    bd.R(16.8, 35, 32.2, 39.6, Rr, radius=1.2)
    bd.R(27, 35, 32.2, 39.6, r, radius=1.0)
    bd.R(16.8, 39, 32.2, 39.9, h, radius=0.4)
    bd.stroke([(24.5, 28), (24.5, 39.4)], 0.6, h)
    for x in (20.5, 22.5, 28.5, 30.5):
        bd.stroke([(x, 31.2), (x + (0.2 if x > 24 else -0.2), 38.8)], 0.4, w)
    bd.thick([(17.9, 22.2), (17.6, 27.0), (18.0, 32.6)], 1.0, 1.1, ("g", W, w, 17, 0, 19, 0))
    bd.thick([(30.9, 22.2), (31.2, 27.0), (30.8, 32.6)], 1.0, 1.1, v)
    bd.ellipse(18.4, 33.9, 1.4, 1.0, F)
    bd.ellipse(30.4, 33.9, 1.4, 1.0, f)
    bd.R(19, 25, 29, 25.8, ("g", Y, y, 19, 0, 29, 0), radius=0.4)
    bd.blob([(24.5, 23.6), (25.9, 25.4), (24.5, 27.9), (23.1, 25.4)], ("g", "#fff6d0", Y, 24, 24, 25, 28))
    for x in (19.6, 22.0, 26.8, 29.0):
        bd.stroke([(x, 22.5), (x + (0.5 if x > 24 else -0.5), 34.5)], 0.45, v, clip=True, alpha=0.45)
    bd.ellipse(24.5, 34.6, 7.5, 1.0, v, clip=True, alpha=0.35)
    bd.cel(v, 0.35, 0.8, 0.0)
    a.put(bd)
    # head with veil
    hdl = a.layer()
    hdl.blob([(19.0, 21.8), (19.0, 15.0), (20.4, 11.6), (24.5, 10.3), (28.6, 11.6), (30.0, 15.0), (30.0, 21.8),
              (27.6, 21.8), (24.5, 20.0), (21.4, 21.8)], ("g", "#ffffff", W, 21, 11, 27, 21))
    hdl.cel(w, 0.9, 1.4, 0.0, 0.4)
    hdl.stroke([(20.0, 14.0), (21.2, 11.8), (24.5, 10.9)], 0.4, "#ffffff", alpha=0.9)
    a.put(hdl)
    fc = a.layer()
    fc.ellipse(cx + 0.5, 16.3, 3.3, 3.9, ("g", "#fdf6ee", F, 23, 13, 25, 20))
    fc.cel(f, 0.9, 0.9, 0.0, 0.3)
    fc.R(21, 12, 27.9, 12.8, ("g", "#a8c4f0", h, 21, 0, 28, 0), radius=0.4)
    fc.stroke([(23.9, 19.0), (25.1, 19.0)], 0.3, "#b89898")
    fc.blob([(20, 20), (24.5, 20.6), (29, 20), (28.8, 21.9), (24.5, 22.2), (20.2, 21.9)], Rr)
    a.put(fc)
    a.outline()
    # light: eyes, circlet gem, clasp gem, throne sigils
    for ex in (23.0, 26.0):
        a.soft(ex, 16.4, 1.4, 0.9, E, 0.55, 0.6)
        a.ellipse(ex, 16.5, 0.85, 0.5, E)
        a.ellipse(ex - 0.2, 16.35, 0.35, 0.25, "#ffffff")
        a.stroke([(ex - 0.9, 15.5), (ex + 0.8, 15.4)], 0.25, H)
    a.soft(24.5, 12.4, 1.0, 1.0, E, 0.6, 0.5)
    a.ellipse(24.5, 12.4, 0.5, 0.5, E)
    a.ellipse(24.5, 25.6, 0.55, 0.55, E)
    a.spec(24.4, 25.4, 0.22)
    for gx, gy, gr in ((11.5, 41.0, 1.4), (36.5, 45.5, 0.7), (20.5, 44.5, 0.7)):
        a.soft(gx, gy, gr + 0.8, gr * 0.7 + 0.5, G, 0.5, 0.7)
        a.ellipse(gx, gy, gr * 0.8, gr * 0.4, "#7ac0ff")
    a.save("w3/final")


# ═══════════════════════════════ WORLD 4 ═══════════════════════════════
RAIN = "#8ac0e0"


def w4_fodder_a():
    # Drowned Rat: soaked grey rat with matted fur spikes and a glowing red eye, facing right.
    a = A(16, 16)
    B, b, Hc, T, t, E, Yc, P = "#4e4a5c", "#36323f", "#7a7a94", "#b07484", "#7a4c5a", "#ff3a3a", "#ffd0a0", "#e090a0"
    tl = a.layer()
    tl.thick([(3.4, 12.2), (1.6, 12.2), (0.6, 11.0), (0.6, 9.2)], 0.55, 0.22, ("g", T, t, 3, 0, 0, 0))
    a.put(tl)
    bd = a.layer()
    bd.ellipse(4.8, 14.6, 1.1, 0.55, b)
    bd.ellipse(9.8, 14.6, 1.1, 0.55, b)
    bd.ellipse(7, 11, 5.2, 3.4, ("g", "#5e5a70", B, 6, 8, 6, 13))
    for x in (4, 6, 8):                     # matted wet fur spikes
        bd.thick([(x + 0.2, 9.0), (x + 1.0, 8.0), (x + 1.8, 7.2)], 0.65, 0.12, B)
    bd.ellipse(7, 12.6, 4.4, 1.6, b, alpha=0.9)
    bd.ellipse(12, 10.3, 3.2, 2.4, ("g", "#5e5a70", B, 11, 8, 12, 12))
    bd.blob([(13, 8.9), (15.0, 9.9), (15.9, 11.0), (14.6, 11.9), (13, 12.1)], B)
    bd.blob([(10.0, 7.0), (11.6, 6.9), (12.0, 8.6), (10.2, 9.0)], B)          # ear
    bd.ellipse(10.9, 8.1, 0.55, 0.65, T)
    bd.cel(b, 0.8, 0.0, 1.0, 0.3)
    for x, y in ((4.5, 8.8), (6.5, 8.8), (8.5, 8.8), (5.5, 9.6), (9.5, 9.6)):
        bd.stroke([(x - 0.3, y + 0.4), (x + 0.5, y - 0.3)], 0.35, Hc, clip=True)
    bd.stroke([(3.0, 9.6), (5.5, 8.4), (9.0, 8.3), (12.0, 8.4)], 0.35, "#b8c8e0", clip=True, alpha=0.7)  # wet sheen
    bd.ellipse(15.4, 11.0, 0.55, 0.45, P)
    bd.ellipse(11.3, 13.4, 0.8, 0.4, b)
    bd.stroke([(14.0, 11.6), (15.6, 12.2)], 0.2, "#22202a")
    a.put(bd)
    a.outline()
    a.ellipse(12.6, 9.6, 0.45, 0.45, Yc)
    a.eye(13.4, 9.6, E, r=0.42, glow=1.0, galpha=0.6)
    a.stroke([(15.9, 10.8), (17.0, 10.2)], 0.12, "#c8c8d8", alpha=0.6)
    a.drop(14.4, 13.6, 0.75, RAIN)
    a.drop(3.4, 6.5, 0.8, RAIN)
    a.save("w4/fodder_a")


def w4_fodder_b():
    # Rain Ghoul: hunched hooded figure with a pale face, red eyes and a long arm dragging to the ground.
    a = A(16, 16)
    J, j, h, S, s, D, E = "#3e4a66", "#29324a", "#5a6a94", "#cfd3da", "#8c94a2", "#121420", "#ff4a4a"
    g = a.layer()
    g.thick([(4.3, 9.0), (3.4, 11.2), (2.6, 13.4)], 0.55, 0.45, s)              # back arm
    g.ellipse(2.6, 14.3, 0.7, 0.6, S)
    g.R(5, 13, 6, 15, j, radius=0.5)
    g.R(9, 13, 10, 15, j, radius=0.5)
    g.ellipse(6.4, 15.5, 0.8, 0.45, s)
    g.ellipse(10.4, 15.5, 0.8, 0.45, s)
    g.blob([(3, 14.2), (3.3, 10), (4.3, 7.0), (6, 4.2), (10, 3.0), (12.5, 6), (12.2, 9.5), (12, 11), (10.6, 14.0),
            (8.0, 13.2), (5.0, 14.4)], ("g", h, J, 6, 4, 9, 12))
    g.blob([(3, 14.2), (3.3, 10), (4.3, 7.0), (6, 4.2), (6.5, 14.0)], j, clip=True)
    g.ellipse(10.5, 5.5, 3.6, 3.2, ("g", h, J, 9, 3, 11, 8))
    g.stroke([(7.0, 4.6), (8.6, 3.2), (10.4, 2.6)], 0.4, "#8a9ac0", alpha=0.8)
    g.ellipse(11.7, 6.3, 2.3, 2.3, D)
    g.ellipse(12.4, 6.6, 1.6, 1.9, ("g", "#e8ebf0", S, 12, 5, 12, 8.5))
    g.cel(s, 0.6, 0.5, 0.5)
    g.stroke([(11.6, 8.0), (12.8, 8.2)], 0.25, "#4a4450")
    g.stroke([(12.4, 8.6), (13.5, 8.4)], 0.2, s)
    for (x, y) in ((7, 5.5), (5.6, 8), (9.0, 9.5)):
        g.stroke([(x, y), (x - 0.3, y + 3.0)], 0.3, j, clip=True, alpha=0.7)   # wet drape folds
    a.put(g)
    ar = a.layer()
    ar.thick([(11.4, 8.4), (12.6, 10.0), (13.6, 11.6)], 0.75, 0.6, J)
    ar.thick([(13.5, 11.5), (14.0, 12.8), (14.6, 13.8)], 0.55, 0.45, S)
    ar.thick([(14.4, 13.8), (15.4, 14.8)], 0.3, 0.12, S)
    ar.thick([(14.4, 13.8), (14.6, 15.1)], 0.3, 0.12, S)
    ar.thick([(14.7, 13.6), (15.7, 13.6)], 0.28, 0.12, s)
    a.put(ar)
    a.outline()
    a.eye(12.2, 6.4, E, r=0.32, glow=0.8, galpha=0.55)
    a.eye(13.4, 6.4, E, r=0.36, glow=0.9, galpha=0.6)
    a.drop(8.5, 1.4, 0.75, RAIN)
    a.drop(4.4, 3.2, 0.7, RAIN)
    a.save("w4/fodder_b")


def w4_ranged():
    # Gutter Spitter: bloated toxic green blob with a rusty pipe in its back, spitting acid.
    a = A(16, 16)
    G, g, Hc, w, M, Ac, ac, Yc, I = ("#5a6c3c", "#3a4828", "#8aa058", "#2e3a20", "#22140f", "#c8f03a", "#f4ff9a",
                                     "#ffe040", "#4a4e58")
    p = a.layer()
    p.R(0.8, 5.6, 3.6, 7.2, ("g", "#7a6a58", I, 0, 5, 0, 7), radius=0.4)
    p.R(1.6, 4.6, 2.9, 6.2, I, radius=0.3)
    p.stroke([(1.2, 6.0), (3.2, 6.0)], 0.25, "#a06a3a", alpha=0.8)
    a.put(p)
    b = a.layer()
    b.ellipse(3.9, 14.6, 1.2, 0.6, g)
    b.ellipse(9.9, 14.6, 1.2, 0.6, g)
    b.ellipse(7, 9.5, 6.4, 5.2, ("g", "#6e8248", G, 5, 5, 8, 13))
    b.ellipse(6, 11.5, 5.2, 3.0, g, clip=True, alpha=0.75)
    b.cel(w, 0.5, 0.8, 1.0, 0.3)
    b.ellipse(5.5, 7, 2.6, 1.8, Hc, alpha=0.85)
    b.ellipse(4.8, 6.5, 1.0, 0.6, "#c0d890", alpha=0.9)
    for x, y, r in ((3.5, 10.5, 0.6), (8.5, 12.5, 0.55), (5.5, 13.2, 0.45), (10.5, 8.6, 0.5), (2.6, 8.4, 0.4)):
        b.ellipse(x, y, r, r * 0.85, w)
        b.spec(x - r * 0.3, y - r * 0.3, r * 0.3, "#8aa058", 0.8)
    b.blob([(10.8, 9.0), (13.2, 8.6), (14.0, 10.2), (13.4, 11.9), (11.2, 11.8)], M)
    b.blob([(11.6, 11.0), (13.0, 10.6), (13.4, 11.7), (11.8, 11.9)], "#5a2a20")
    b.thick([(12.2, 11.6), (12.4, 12.8), (12.5, 13.8)], 0.42, 0.25, Ac)        # drool
    a.put(b)
    a.outline()
    for x, y, r in ((9.5, 6.4, 0.6), (11.4, 7.4, 0.55), (7.4, 5.4, 0.5)):
        a.ellipse(x, y, r, r, Yc)
        a.ellipse(x + 0.1, y, r * 0.35, r * 0.7, "#2a1a08")
        a.spec(x - r * 0.35, y - r * 0.35, r * 0.25)
    a.soft(14.8, 8.8, 1.6, 1.4, Ac, 0.55, 0.7)
    a.blob([(13.6, 9.2), (14.4, 8.0), (15.6, 7.9), (15.9, 9.0), (15.0, 9.9)], Ac)
    a.ellipse(15.1, 8.5, 0.45, 0.35, ac)
    a.ellipse(12.8, 10.2, 0.5, 0.4, Ac)
    a.soft(13.5, 15.4, 0.8, 0.5, Ac, 0.5, 0.4)
    a.ellipse(13.5, 15.4, 0.55, 0.4, Ac)
    a.save("w4/ranged")


def w4_blinker():
    # Umbrella Stalker: tall dark figure hidden under a black umbrella, only red eyes visible beneath.
    a = A(16, 16)
    U, u, D, C, c, M, E = "#2e2c40", "#5a5880", "#0f0d16", "#2c2640", "#4c4070", "#8a8a9a", "#ff3a50"
    body = a.layer()
    body.R(7, 14, 7, 15.6, C, radius=0.4)
    body.R(9, 14, 9, 15.6, C, radius=0.4)
    body.blob([(6.0, 8.0), (11.0, 8.0), (11.3, 11), (11.2, 14.2), (8.5, 14.6), (5.8, 14.2), (5.8, 11)],
              ("g", c, C, 6, 0, 11, 0))
    body.stroke([(6.5, 8.6), (6.3, 13.8)], 0.4, c, clip=True)
    body.stroke([(10.6, 9.4), (10.8, 13.8)], 0.4, c, clip=True, alpha=0.8)
    body.stroke([(8.5, 9.6), (8.5, 14.2)], 0.3, D, clip=True, alpha=0.6)
    body.R(5, 5, 11, 7.5, D, radius=1.0)
    body.R(8.2, 5, 8.8, 9.6, M, radius=0.3)                                      # umbrella shaft
    body.ellipse(9.1, 9.6, 0.9, 0.7, "#6a6478")                                    # hand
    a.put(body)
    um = a.layer()
    um.blob([(0.2, 5.2), (1.0, 3.2), (3.4, 1.4), (8.0, 0.8), (12.6, 1.4), (15.0, 3.2), (15.8, 5.2), (14.4, 4.7),
             (12.8, 5.4), (11.2, 4.7), (9.6, 5.4), (8.0, 4.7), (6.4, 5.4), (4.8, 4.7), (3.2, 5.4), (1.6, 4.7)],
            ("g", "#46446a", U, 6, 1, 9, 5))
    for x0, x1 in ((8.0, 8.0), (5.0, 3.2), (11.0, 12.8), (2.6, 0.6), (13.4, 15.4)):
        um.stroke([(8.0, 1.0), ((8 + x0) / 2, 2.0), (x1, 5.0)], 0.35, u, alpha=0.9)
    um.cel(D, 0.4, 0.0, 0.8)
    um.stroke([(3.2, 2.0), (6.0, 1.1), (8.5, 1.0)], 0.35, "#a0a6d0", alpha=0.6)
    um.R(7.6, -0.4, 8.4, 1.2, M, radius=0.35)
    a.put(um)
    a.outline()
    a.eye(9.5, 6.4, E, r=0.38, glow=0.9, galpha=0.6)
    a.eye(11.0, 6.4, E, r=0.38, glow=0.9, galpha=0.6)
    a.stroke([(9.9, 7.3), (10.7, 7.4)], 0.2, "#ffb0b8", alpha=0.6)
    a.drop(1.4, 7.6, 0.7, RAIN)
    a.drop(14.6, 8.4, 0.7, RAIN)
    a.drop(15.3, 10.8, 0.6, RAIN)
    a.save("w4/blinker")


def w4_elite():
    # Ward Butcher: hulking figure in a yellow-olive raincoat, bloodied apron, meat hook.
    a = A(32, 32)
    R, r, Lc, D, E = "#8a7d3c", "#5e5428", "#b8a858", "#100e14", "#ffd040"
    X, x_, P, p, S, M, m, B = "#6a1818", "#9a2626", "#c4bcaa", "#8e8676", "#c09a84", "#b0b8c2", "#6a707a", "#24242a"
    ba = a.layer()
    ba.R(2, 14, 5, 23.5, ("g", "#6e6430", r, 2, 0, 6, 0), radius=1.4)
    ba.blob([(2, 23.4), (5.2, 23.4), (5.4, 25.6), (3.6, 26.8), (2.0, 26.0)], S)
    ba.stroke([(2.2, 24.6), (5.2, 24.6)], 0.35, "#8e6e5e", clip=True)
    a.put(ba)
    bt = a.layer()
    bt.blob([(7, 31.9), (7.2, 29.2), (13.4, 29.0), (14.0, 31.9)], B)
    bt.blob([(18, 31.9), (18.2, 29.2), (24.4, 29.0), (25.0, 31.9)], B)
    bt.lit("#5a5a68", 0.8, 0.0, 0.5)
    a.put(bt)
    c = a.layer()
    c.blob([(5.2, 30.0), (5.6, 20), (6.4, 14), (10, 10.2), (16, 9.6), (22, 10.2), (26.6, 14), (27.2, 22), (27, 30.0),
            (21, 30.6), (16, 30.0), (11, 30.6)], ("g", Lc, R, 14, 10, 18, 22))
    c.blob([(5.2, 30.0), (5.6, 20), (6.4, 14), (10, 10.2), (11.4, 30.4)], r, clip=True)
    c.cel(r, 0.7, 1.2, 0.0, 0.4)
    for yy in (17.5, 22.5, 27.5):
        c.stroke([(7, yy), (11.6, yy - 0.3)], 0.4, "#4a4220", clip=True)          # quilted seams
    c.stroke([(26.4, 15), (26.6, 29)], 0.6, Lc, clip=True, alpha=0.9)
    c.stroke([(26.6, 16), (26.8, 24)], 0.3, "#e0d488", clip=True, alpha=0.8)
    a.put(c)
    ap = a.layer()
    ap.blob([(16, 13), (21.8, 13), (22, 15), (23.8, 15.2), (24, 29.6), (14, 29.6), (14, 15.2), (16, 15)],
            ("g", "#dcd6c6", P, 20, 14, 18, 29))
    ap.R(14, 15, 15.4, 29.6, p, clip=True)
    for (ex, ey, rx, ry) in ((19, 20, 2.2, 1.6), (16.5, 25, 1.8, 2.2), (22, 27, 1.5, 1.4)):
        ap.ellipse(ex, ey, rx, ry, X, clip=True, alpha=0.9)
        ap.ellipse(ex - 0.4, ey - 0.4, rx * 0.5, ry * 0.5, x_, clip=True)
    ap.thick([(19.4, 21), (19.5, 23.5), (19.3, 24.4)], 0.45, 0.3, X, clip=True)
    ap.thick([(16.6, 27), (16.7, 29.2)], 0.4, 0.3, X, clip=True)
    ap.stroke([(14.4, 28.8), (23.8, 28.8)], 0.4, p, clip=True)
    a.put(ap)
    hd = a.layer()
    hd.ellipse(16, 7, 6.5, 6, ("g", Lc, R, 13, 2, 17, 11))
    hd.ellipse(13.5, 6.5, 3, 4.5, r, alpha=0.9)
    hd.cel(r, 0.7, 1.0, 0.0, 0.4)
    hd.ellipse(18.3, 8, 3.6, 3.6, ("g", "#000000", D, 18, 5, 18, 11))
    hd.R(17, 10, 20.4, 10.9, S, radius=0.4)
    hd.stroke([(17.6, 10.5), (20.0, 10.5)], 0.25, "#4a2a26")
    hd.stroke([(16.6, 1.6), (18.6, 2.6), (20.6, 4.0)], 0.5, "#dccc78", alpha=0.85)
    hd.stroke([(22.4, 5.2), (22.6, 7.8)], 0.5, Lc)
    a.put(hd)
    fa = a.layer()
    fa.blob([(22, 11), (24.8, 11.6), (26.6, 13.6), (29.2, 19.0), (26.4, 21.4), (23, 16)], ("g", Lc, R, 23, 12, 28, 20))
    fa.cel(r, 0.6, 0.8, 0.8)
    fa.R(26, 20, 29, 22, S, radius=0.8)
    fa.stroke([(26.5, 21.4), (28.6, 21.4)], 0.3, "#8e6e5e")
    a.put(fa)
    hk = a.layer()
    hk.R(24.6, 21, 30.6, 21.9, ("g", M, m, 0, 21, 0, 22), radius=0.4)
    hk.thick([(28.5, 22.6), (28.5, 25.4), (28.2, 27.4), (27.0, 28.9), (25.4, 28.8), (24.5, 27.6), (24.6, 26.6)],
             0.48, 0.2, ("g", "#e0e8f0", M, 27, 22, 29, 22))
    hk.stroke([(28.9, 23.0), (28.9, 26.0)], 0.25, m)
    a.put(hk)
    a.outline()
    a.ellipse(24.6, 26.4, 0.45, 0.45, x_)
    a.eye(18.4, 8.4, E, r=0.48, glow=1.2, galpha=0.6)
    a.eye(20.4, 8.4, E, r=0.48, glow=1.2, galpha=0.6)
    a.ellipse(24.5, 30.6, 0.9, 0.35, X, alpha=0.9)
    a.drop(24.6, 29.4, 0.5, x_, 0.95)
    for xx, yy in ((9.5, 1.5), (3.5, 9.5), (29.5, 9.5), (12.5, 13.5)):
        a.drop(xx, yy, 0.8, RAIN)
    for xx, yy in ((1.5, 3), (30.5, 1.5), (0.8, 18)):
        a.streak(xx, yy, 2.0, RAIN, 0.45)
    a.save("w4/elite")


def w4_mini():
    # The Night Investigator: trench coat and fedora, glowing visor, briefcase that unfolds into a blade.
    a = A(32, 32)
    Hh, i, b, S, s = "#2a2a3a", "#474a62", "#6a2030", "#b89a88", "#8a6e62"
    V, v, wv, C, c, Lc, G = "#5af0ff", "#2a9cc0", "#e8ffff", "#7a6448", "#52432f", "#a48a64", "#d0a840"
    B, M, m, Z, z = "#20202a", "#3a3c4a", "#6e7282", "#c8d2dc", "#f0faff"
    lg = a.layer()
    lg.R(11, 29, 13, 31.9, B, radius=0.4)
    lg.blob([(18, 31.9), (18, 29), (20.4, 29), (21.9, 30.6), (21.9, 31.9)], B)
    a.put(lg)
    bk = a.layer()
    bk.R(6, 14, 8, 24.8, ("g", "#5e4c36", c, 6, 0, 9, 0), radius=1.1)
    bk.blob([(6, 24.6), (8.8, 24.6), (8.8, 26.6), (7.2, 27.0), (6, 26.4)], S)
    a.put(bk)
    co = a.layer()
    co.blob([(7, 30.4), (8.0, 21), (9, 13.2), (15.5, 12.6), (22, 13.2), (24, 21), (25, 30.4), (16, 30.8)],
            ("g", "#8e7656", C, 14, 13, 18, 29))
    co.blob([(7, 30.4), (8.0, 21), (9, 13.2), (12, 13.2), (11.2, 30.6)], c, clip=True)
    co.R(22, 16, 23.9, 29.5, Lc, clip=True, alpha=0.9)
    co.stroke([(13.4, 13.4), (16.4, 20.4)], 0.55, c)
    co.stroke([(19.4, 13.4), (16.6, 20.4)], 0.55, Lc)
    co.stroke([(16.5, 21.5), (16.5, 30.6)], 0.5, c)
    co.R(8, 20, 24, 21.9, c, clip=True)
    co.R(15, 20, 18, 21.9, ("g", "#f0d070", G, 0, 20, 0, 22), radius=0.3)
    co.R(15.8, 20.5, 17.2, 21.4, c, radius=0.2)
    for yy in (15.5, 17.5):
        co.ellipse(18.5, yy, 0.42, 0.42, G)
    co.stroke([(9.5, 24.5), (11.5, 24.5)], 0.35, "#3e3222", clip=True)
    co.cel(c, 0.5, 0.0, 1.0)
    a.put(co)
    cl = a.layer()
    cl.blob([(10, 14.2), (10.8, 9.0), (14.2, 12.0), (12, 14.2)], ("g", Lc, C, 10, 9, 13, 14))
    cl.blob([(21.2, 14.2), (21, 9.0), (17.8, 12.0), (19.8, 14.2)], ("g", "#c8ac80", Lc, 21, 9, 18, 14))
    a.put(cl)
    hd = a.layer()
    hd.R(12, 7, 19, 12, ("g", "#d0b4a0", S, 0, 7, 0, 12), radius=0.8)
    hd.R(12, 7, 13.4, 12, s, radius=0.6)
    hd.R(15, 11, 18, 11.6, s, radius=0.3)
    hd.stroke([(13.2, 12.4), (18.6, 12.4)], 0.4, "#6e5448", alpha=0.6)
    hd.R(12.8, 7.6, 21.0, 10.0, v, radius=0.8)
    hd.R(13.6, 7.9, 20.6, 9.2, ("g", "#9afcff", V, 13, 0, 21, 0), radius=0.6)
    a.put(hd)
    ht = a.layer()
    ht.blob([(6.6, 6.6), (8, 5.0), (15.5, 4.6), (23, 5.0), (24.4, 6.6), (22.6, 7.3), (15.5, 7.0), (8.4, 7.3)],
            ("g", i, Hh, 0, 5, 0, 7.5))
    ht.blob([(11, 5.6), (11.2, 2.0), (13.0, 1.0), (15.5, 0.2), (18.0, 1.0), (20.0, 1.6), (20.4, 5.6)],
            ("g", "#3e3e54", Hh, 13, 1, 18, 5))
    ht.stroke([(13.6, 1.4), (15.5, 0.8), (17.4, 1.4)], 0.4, "#2a2a3a")
    ht.R(11, 4, 20.6, 5.2, ("g", "#8a2a40", b, 0, 4, 0, 5.2))
    ht.stroke([(11.8, 1.8), (13.0, 1.2)], 0.35, "#6a6e8e", alpha=0.8)
    ht.stroke([(19.5, 2.2), (19.8, 3.6)], 0.5, i)
    ht.stroke([(8.0, 6.6), (23.0, 6.6)], 0.3, "#5a5e7a", alpha=0.7)
    a.put(ht)
    fa = a.layer()
    fa.blob([(21, 13), (24, 14), (26, 20), (23, 21), (21, 16)], ("g", "#8e7656", C, 22, 14, 25, 20))
    fa.R(23, 20, 25.8, 21.9, S, radius=0.7)
    a.put(fa)
    bl = a.layer()
    bl.poly([(27.0, 20.6), (30.8, 3.6), (31.9, 2.4), (31.6, 4.4), (29.6, 20.6)], ("g", Z, "#9aa6b4", 27, 0, 31, 0))
    bl.poly([(28.6, 20.6), (31.4, 4.0), (31.9, 2.4), (30.0, 20.6)], z)
    for k in range(4, 17, 4):
        yy = 20 - k
        xx = 27 + k * 0.25
        bl.stroke([(xx + 0.6, yy + 0.5), (xx + 1.6, yy + 0.5)], 0.3, V)
    a.put(bl)
    bc = a.layer()
    bc.R(22, 21, 29, 26, ("g", "#4a4c5c", M, 0, 21, 0, 26), radius=0.5)
    bc.R(22, 21, 29, 21.6, m)
    bc.R(29, 22, 29.9, 26.9, m)
    bc.R(26, 20, 29.9, 20.9, m, radius=0.3)
    bc.stroke([(22.4, 25.6), (29.4, 25.6)], 0.3, "#24242e")
    a.put(bc)
    a.outline()
    a.soft(17, 8.6, 4.6, 1.6, V, 0.4, 0.8)
    a.stroke([(14.4, 8.3), (20.2, 8.3)], 0.3, wv, alpha=0.9, smooth=False)
    a.spec(19.8, 8.2, 0.3)
    a.soft(25.9, 24.5, 3.2, 0.9, V, 0.5, 0.5)
    a.R(22.2, 24, 29.7, 24.8, v)
    a.R(25, 24, 26.9, 24.8, V)
    a.soft(30.6, 10, 1.4, 8, V, 0.25, 1.0)
    a.spec(31.2, 3.4, 0.35, z)
    for xx, yy in ((4.5, 4.5), (25.5, 8.5), (3.5, 18.5), (5.5, 1.5)):
        a.drop(xx, yy, 0.8, RAIN)
    for xx, yy in ((1.4, 9), (27.5, 12.0), (2.0, 26)):
        a.streak(xx, yy, 2.0, RAIN, 0.45)
    a.save("w4/mini")


def w4_boss():
    # Mother of Gutters: huge many-eyed sewer slime rising out of a manhole, tentacles, toothy maw.
    a = A(48, 48)
    I, i, D, G, g, Hc, h = "#3a3e48", "#5c626e", "#0b0c10", "#2e5a48", "#1d3c31", "#4c8a68", "#9ad8a8"
    Yc, y, O, pp, M, T, Rr = "#f0e050", "#b0a030", "#e06020", "#200c10", "#120a10", "#d8d0b8", "#6a3a24"
    mh = a.layer()
    mh.ellipse(24, 43.5, 22, 4.5, ("g", i, I, 24, 39, 24, 48))
    mh.ellipse(24, 42.5, 19, 3.2, D)
    for k in range(16):
        ang = math.pi * k / 15
        xx, yy = 24 + 20.6 * math.cos(ang), 43.5 + 4.0 * math.sin(ang)
        mh.ellipse(xx, yy, 0.55, 0.3, "#2a2e36")
    mh.stroke([(3.0, 43.0), (10, 46.4), (24, 47.6), (38, 46.4), (45.0, 43.0)], 0.4, "#7a808e", alpha=0.7)
    a.put(mh)
    # tentacles
    tn = a.layer()
    tn.thick([(12, 30), (5, 26), (3, 18), (6, 11), (8.0, 9.0)], 3.2, 0.7, ("g", Hc, g, 3, 12, 10, 30))
    tn.thick([(36, 30), (43, 26), (45, 18), (42, 10), (39.8, 8.4)], 3.2, 0.7, ("g", Hc, G, 46, 12, 38, 30))
    tn.cel(g, 0.7, 1.0, 1.0, 0.4)
    tn.thick([(42.0, 27), (44.6, 19.0), (42.6, 12.0)], 0.6, 0.3, h, alpha=0.6)
    tn.thick([(4.4, 24), (3.2, 18), (5.4, 12.6)], 0.5, 0.25, h, alpha=0.4)
    for (tx, ty) in ((5.0, 22), (4.3, 17.5), (44.0, 23), (44.6, 17.6)):
        tn.ellipse(tx, ty, 0.6, 0.5, "#6aa880", clip=True)
    a.put(tn)
    # slime mass
    sm = a.layer()
    sm.ellipse(24, 28, 15, 15, G)
    sm.ellipse(24, 15, 12, 11, G)
    sm.blob([(9.5, 34), (38.5, 34), (38, 40.0), (37, 41.5), (30, 42.0), (24, 42.6), (18, 42.0), (11, 41.5), (10, 40.0)], G)
    sm.ellipse(17, 30, 7, 13, g, alpha=0.85)
    sm.ellipse(15, 14, 4, 8, g, alpha=0.85)
    sm.cel(g, 0.6, 3.0, 2.0, 1.2)
    sm.ellipse(30, 10, 4, 3, Hc)
    sm.ellipse(33, 22, 3, 6, Hc, alpha=0.9)
    sm.ellipse(29.5, 8.8, 1.8, 1.0, h, alpha=0.8)
    sm.ellipse(34, 19.5, 1.0, 2.0, h, alpha=0.6)
    sm.lit(h, 0.5, 0.8, 0.8, 0.4)
    # bubbles in the slime
    rng = random.Random(4)
    for _ in range(14):
        bx, by = rng.uniform(12, 36), rng.uniform(6, 40)
        rr = rng.uniform(0.35, 0.8)
        sm.ellipse(bx, by, rr, rr, Hc, clip=True, alpha=0.6)
        sm.spec(bx - rr * 0.35, by - rr * 0.35, rr * 0.3, h, 0.8)
    a.put(sm)
    # grate + rusty debris stuck in it
    gr = a.layer()
    gr.R(9, 33, 15, 37, ("g", i, I, 0, 33, 0, 38), radius=0.4)
    for xx in (10, 12, 14):
        gr.R(xx, 34, xx, 36, D)
    gr.R(36, 34, 39, 36, ("g", "#8a5030", Rr, 0, 34, 0, 37), radius=0.3)
    gr.stroke([(36.4, 35.0), (38.6, 35.6)], 0.3, "#3a1e14")
    a.put(gr)
    # mouth
    mo = a.layer()
    mo.ellipse(24, 32, 9, 3.6, M)
    mo.ellipse(24, 33.2, 7.0, 2.0, "#3a0e18")
    mo.ellipse(25, 33.6, 3.0, 1.2, "#7a2030")                                      # tongue
    for xx in range(17, 31, 3):
        mo.blob([(xx - 0.1, 28.8), (xx + 2.1, 28.8), (xx + 1.0, 31.6)], ("g", "#f2ecd8", T, 0, 29, 0, 32))
    for xx in range(18, 30, 3):
        mo.blob([(xx - 0.1, 35.2), (xx + 2.1, 35.2), (xx + 1.0, 32.6)], ("g", T, "#a89e84", 0, 33, 0, 35))
    mo.stroke([(15.4, 31.0), (16.0, 32.0)], 0.3, g)
    a.put(mo)
    # drips over the rim
    dr = a.layer()
    for xx, y0, y1 in ((11, 38, 42), (18, 41, 44), (30, 41, 45), (37, 37, 41)):
        dr.thick([(xx + 0.5, y0), (xx + 0.5, y1)], 0.6, 0.45, G)
        dr.ellipse(xx + 0.5, y1 + 0.4, 0.75, 0.75, Hc)
    a.put(dr)
    a.outline()
    # eyes (after outline so they read as bulging and glowing)
    for (ex, ey, rr) in ((14, 10, 1.8), (33, 15, 1.9), (17, 22, 1.6), (31, 25, 2.2), (12, 25, 1.4),
                         (36, 9, 1.3), (20, 6, 1.3), (29, 5, 1.2), (38, 20, 1.3), (8, 15, 1.2)):
        a.ellipse(ex, ey, rr + 0.45, rr + 0.45, g)
        a.ellipse(ex, ey, rr, rr, ("g", "#fff8a0", Yc, ex - rr, ey - rr, ex + rr, ey + rr))
        a.ellipse(ex + 0.1, ey, rr * 0.28, rr * 0.7, pp)
        a.spec(ex - rr * 0.4, ey - rr * 0.4, rr * 0.25)
    a.soft(24, 15, 6.4, 5.2, Yc, 0.35, 1.2)
    a.ellipse(24, 15, 5.6, 4.6, g)
    a.ellipse(24, 15, 5, 4, ("g", "#fff8a0", Yc, 24, 11, 24, 19))
    a.ellipse(24.5, 15, 2.8, 2.8, ("g", "#ff9040", O, 23, 13, 26, 18))
    a.ellipse(24.5, 15, 0.55, 2.3, pp)
    a.spec(22.4, 13.2, 0.6)
    a.spec(26.0, 16.6, 0.3, "#ffe0a0")
    a.stroke([(19.2, 15.0), (24, 10.9), (28.8, 15.0)], 0.5, g, alpha=0.8)        # heavy upper lid line
    for xx, yy in ((3, 33), (44, 34), (6, 39), (41, 4), (2, 8), (26, 3)):
        a.drop(xx + 0.5, yy + 0.5, 0.9, h, 0.75)
    for xx, yy in ((1.5, 2), (46, 12), (45.4, 38), (2.4, 22), (22, 0.5)):
        a.streak(xx, yy, 2.6, RAIN, 0.4)
    a.save("w4/boss")


def w4_final():
    # The Crimson Rain Lord: masked lord under an umbrella-like blood halo, long coat dissolving into red rain.
    a = A(48, 48)
    R, r, Q, N, W, w, k, E = "#c02030", "#6a0c18", "#ff4a58", "#1e0a10", "#ece6dc", "#b4aca2", "#4a3a40", "#ff2a3a"
    C, c, Lc, Bc, Mg, Sb = "#5a0e1a", "#2c0810", "#8a1a2a", "#a01424", "#d8b060", "#8aa8c8"
    hx, hy = 24, 12
    # canopy halo
    ca = a.layer()
    angs = [math.pi * 0.92 + kk * (math.pi * 1.16) / 8 for kk in range(9)]
    outer, inner = [], []
    a0, a1 = math.pi * 0.86, math.pi * 2.14
    steps = 72
    for sidx in range(steps + 1):
        t = a0 + (a1 - a0) * sidx / steps
        frac = (sidx / steps) * 8
        scallop = 0.9 * abs(math.sin(frac * math.pi))
        outer.append((hx + (15.2 - scallop) * math.cos(t), hy + (15.2 - scallop) * math.sin(t)))
        inner.append((hx + 10.5 * math.cos(t), hy + 10.5 * math.sin(t)))
    ca.poly(outer + inner[::-1], ("g", "#e03040", R, 24, -3, 24, 18))
    for ang in angs:
        ca.thick([(hx + 6 * math.cos(ang), hy + 6 * math.sin(ang)), (hx + 15 * math.cos(ang), hy + 15 * math.sin(ang))],
                 0.5, 0.4, r)
    ca.stroke([(hx + 14.6 * math.cos(t), hy + 14.6 * math.sin(t)) for t in
               [a0 + (a1 - a0) * j / 40 for j in range(41)]], 0.5, Q, clip=True, alpha=0.9)
    ca.cel(r, 0.5, 0.0, 1.0)
    for ang in angs:
        ca.thick([(hx + 15 * math.cos(ang), hy + 15 * math.sin(ang)), (hx + 17.6 * math.cos(ang), hy + 17.6 * math.sin(ang))],
                 0.55, 0.15, Q)
    a.put(ca)
    # tendrils behind coat
    td = a.layer()
    td.thick([(9, 37), (5, 40), (2, 36), (3, 31)], 1.6, 0.4, ("g", Bc, R, 9, 37, 3, 31))
    td.thick([(39, 37), (43, 39), (46, 34), (44, 28), (46, 24)], 1.6, 0.4, ("g", Bc, R, 39, 37, 46, 24))
    td.thick([(38, 38), (40, 44)], 1.2, 0.4, Bc)
    td.thick([(42.8, 38.6), (45, 34.4)], 0.4, 0.2, Q)
    td.thick([(5, 39.4), (3, 36)], 0.4, 0.2, Q)
    a.put(td)
    # long coat
    co = a.layer()
    co.blob([(7, 45.2), (10, 34), (14, 21), (18, 18), (24, 17.6), (30, 18), (34, 21), (38, 34), (41, 45.2), (24, 45.8)],
            ("g", "#7a1424", C, 24, 18, 24, 45))
    co.poly([(7, 45.2), (14, 21), (18, 18), (20, 45.6)], c, clip=True, alpha=0.9)
    for x0, x1, col in ((22, 20, c), (27, 29, Lc), (32, 36, Lc)):
        co.stroke([(x0, 24), ((x0 + x1) / 2 + 0.3, 34), (x1, 45)], 0.5, col, clip=True)
    co.stroke([(24.5, 19), (24.5, 45.5)], 0.6, c, clip=True)
    co.lit(Lc, 0.8, -0.8, 0.0)
    for x, y1 in ((9, 47), (13, 46), (17, 47), (22, 46), (27, 47), (31, 46), (35, 47), (39, 46)):
        co.thick([(x + 1.0, 44.5), (x + 1.0, y1 + 0.2)], 0.9, 0.5, Bc)
        co.ellipse(x + 1.0, y1 + 0.4, 0.7, 0.6, Q)
    co.R(8, 45, 40, 45.8, r, radius=0.4)
    a.put(co)
    # sleeves
    sl = a.layer()
    sl.thick([(15, 22), (11, 30), (9, 36.4)], 2.6, 2.2, ("g", C, c, 9, 0, 15, 0))
    sl.thick([(33, 22), (37, 30), (39, 36.4)], 2.6, 2.2, ("g", Lc, C, 33, 0, 40, 0))
    sl.ellipse(9.0, 37.4, 2.0, 0.9, r)
    sl.ellipse(39.0, 37.4, 2.0, 0.9, r)
    sl.cel(c, 0.6, 1.0, 0.0)
    a.put(sl)
    # collar spikes
    cs = a.layer()
    cs.poly([(15, 21.6), (17, 12.6), (21.4, 19.4)], ("g", C, c, 17, 13, 18, 21))
    cs.poly([(33, 21.6), (31, 12.6), (26.6, 19.4)], ("g", Lc, C, 31, 13, 30, 21))
    cs.stroke([(31.2, 13.6), (30.2, 17.0)], 0.4, "#c03040", alpha=0.8)
    a.put(cs)
    # head: hood + cracked porcelain mask
    hd = a.layer()
    hd.ellipse(24, 11.5, 6, 7, ("g", "#3a1420", N, 22, 5, 26, 18))
    hd.ellipse(24.6, 12, 4.4, 5.4, ("g", "#fbf8f2", W, 24, 7, 25, 17))
    hd.ellipse(22.2, 12, 2, 4.6, w, clip=True, alpha=0.8)
    hd.cel(w, 0.6, 0.6, 0.6, 0.3)
    hd.stroke([(24.5, 6.6), (25.4, 9.2), (24.4, 12.4)], 0.32, k)
    hd.stroke([(25.4, 9.2), (26.6, 8.4)], 0.25, k)
    hd.stroke([(23.4, 15.7), (25.4, 15.4), (26.8, 15.8)], 0.32, k)
    hd.stroke([(22.4, 16.4), (23.4, 15.8)], 0.25, k)
    hd.blob([(22, 11.2), (23.8, 11.0), (23.6, 12.0), (22.2, 12.0)], "#2a0a10")
    hd.blob([(26, 11.0), (27.9, 11.2), (27.8, 12.0), (26.2, 12.0)], "#2a0a10")
    hd.R(22, 18, 26.9, 18.9, ("g", "#f0d488", Mg, 0, 18, 0, 19), radius=0.4)
    a.put(hd)
    a.outline()
    # light & blood
    a.soft(24, 26, 3.6, 4.0, Q, 0.5, 1.4)
    a.ellipse(24, 26, 2.6, 3, ("g", Q, Bc, 23, 23, 25, 29))
    a.ellipse(24, 25.5, 1.4, 1.6, ("g", "#ffb0b8", Q, 24, 24, 24, 27))
    a.spec(23.6, 24.9, 0.4)
    a.thick([(27.5, 12.2), (27.4, 14.4), (27.6, 16.6)], 0.42, 0.28, Bc)           # blood tear
    a.ellipse(27.6, 16.8, 0.45, 0.5, Bc)
    a.eye(23.0, 11.6, E, r=0.42, glow=1.1, galpha=0.6)
    a.eye(27.0, 11.6, E, r=0.42, glow=1.1, galpha=0.6)
    a.soft(27.5, 10.5, 0.5, 0.5, Q, 0.6, 0.3)
    a.spec(24.4, 18.3, 0.3, "#fff4c8")
    for xx, yy in ((3, 6), (4, 7), (44, 5), (45, 6), (6, 24), (42, 15), (1, 16), (47, 42), (2, 44)):
        col = Q if (xx + yy) % 2 else Sb
        a.streak(xx + 0.7, yy - 0.6, 1.8, col, 0.7, 0.4)
    a.save("w4/final")


def main():
    for fn in (w3_fodder_a, w3_fodder_b, w3_ranged, w3_blinker, w3_elite, w3_mini, w3_boss, w3_final,
               w4_fodder_a, w4_fodder_b, w4_ranged, w4_blinker, w4_elite, w4_mini, w4_boss, w4_final):
        fn()
    print("wrote 16 HD sprites to art/hd/w3, art/hd/w4")


if __name__ == "__main__":
    main()
