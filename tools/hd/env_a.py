"""HD redraw of the environment set A: prologue street (stage/*) and worlds 1-2 ground tiles + props.

    python3 tools/hd/env_a.py

Ground tiles are built from a base layer that is IDENTICAL for every tile of a set (so any tile
sits seamlessly next to any other), plus per-tile detail that fades out before the tile edges.
Path tiles keep their vertical wrap (a path column only ever touches itself above/below).
"""
import math
import pathlib
import random
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np  # noqa: E402
from PIL import Image, ImageChops, ImageDraw, ImageFilter  # noqa: E402

import hdkit  # noqa: E402
from hdkit import Art, SS, UP, OUT, ROOT, rgb  # noqa: E402

SRC = ROOT / "art" / "src"


# ───────────────────────── helpers ─────────────────────────

def grid(rel):
    """Rows of chars + palette of an original .px."""
    mirror, pal, rows, body = False, {}, [], False
    for line in (SRC / f"{rel}.px").read_text().splitlines():
        line = line.rstrip()
        if not line or line.startswith("#"):
            continue
        if body:
            rows.append(line + line[::-1] if mirror else line)
        elif line == "---":
            body = True
        elif line == "mirror":
            mirror = True
        else:
            c, h = line.split(" ", 1)
            pal[c] = h.strip()
    return rows, pal


def comps(rows, chars):
    """4-connected components of the given chars -> list of [(x, y), ...]."""
    h, w = len(rows), len(rows[0])
    seen, out = set(), []
    for y in range(h):
        for x in range(w):
            if rows[y][x] in chars and (x, y) not in seen:
                st, c = [(x, y)], []
                seen.add((x, y))
                while st:
                    px, py = st.pop()
                    c.append((px, py))
                    for nx, ny in ((px + 1, py), (px - 1, py), (px, py + 1), (px, py - 1)):
                        if 0 <= nx < w and 0 <= ny < h and (nx, ny) not in seen and rows[ny][nx] in chars:
                            seen.add((nx, ny))
                            st.append((nx, ny))
                out.append(c)
    return out


def blur(mask, radius, tile):
    if radius <= 0:
        return mask
    if not tile:
        return mask.filter(ImageFilter.GaussianBlur(radius))
    W, H = mask.size
    big = Image.new("L", (W * 3, H * 3))
    for i in range(3):
        for j in range(3):
            big.paste(mask, (i * W, j * H))
    return big.filter(ImageFilter.GaussianBlur(radius)).crop((W, H, 2 * W, 2 * H))


def fade_mask(a, margin=3.0, axes="xy"):
    """1 inside, 0 at the tile edges (on the given axes), smooth ramp of `margin` art px."""
    ys, xs = np.mgrid[0:a.H, 0:a.W].astype(np.float32) / SS + 0.5 / SS
    f = np.ones((a.H, a.W), np.float32)
    if "x" in axes:
        f = np.minimum(f, np.minimum(xs, a.w - xs) / margin)
    if "y" in axes:
        f = np.minimum(f, np.minimum(ys, a.h - ys) / margin)
    f = np.clip(f, 0, 1)
    f = f * f * (3 - 2 * f)
    return Image.fromarray((f * 255).astype(np.uint8), "L")


def soft(a, draw, color, radius=1.0, alpha=1.0, clip=False, fade=None):
    """Paint `color` through a blurred mask made by draw(tmp_art)."""
    t = Art(a.w, a.h, tile=a.tile)
    draw(t)
    m = blur(t.img.getchannel("A"), radius * SS, a.tile)
    if fade:
        m = ImageChops.multiply(m, fade_mask(a, *fade))
    a._paint(m, color, clip, alpha)


def soft_pixels(a, rows, chars, color, radius=1.0, alpha=1.0, fade=(3.0, "xy"), grow=0.0):
    """Organic soft patch following the original's pixels of `chars`."""
    def d(t):
        for y, r in enumerate(rows):
            for x, ch in enumerate(r):
                if ch in chars:
                    t.rect(x - grow, y - grow, x + 1 + grow, y + 1 + grow, "#ffffff")
    soft(a, d, color, radius, alpha, fade=fade)


def halo(a, cx, cy, r, color, alpha=0.4):
    """Light glow over everything (also into transparent space)."""
    m = Image.new("L", (a.W, a.H), 0)
    ImageDraw.Draw(m).ellipse([(cx - r * 0.45) * SS, (cy - r * 0.45) * SS, (cx + r * 0.45) * SS, (cy + r * 0.45) * SS], fill=255)
    m = m.filter(ImageFilter.GaussianBlur(r * 0.45 * SS))
    if not a.tile:
        m = ImageChops.multiply(m, fade_mask(a, 2.0, "xy"))
    lay = Image.new("RGBA", (a.W, a.H), rgb(color))
    lay.putalpha(m.point(lambda v: int(v * alpha)))
    a.img = Image.alpha_composite(a.img, lay)


def glow2(a, color, radius=2.0, alpha=0.6):
    """hdkit glow that fades out before the canvas edge (no visible box)."""
    m = a.img.getchannel("A").filter(ImageFilter.GaussianBlur(radius * SS))
    m = ImageChops.multiply(m, fade_mask(a, 1.5, "xy"))
    under = Image.new("RGBA", (a.W, a.H), rgb(color))
    under.putalpha(m.point(lambda v: int(v * alpha)))
    under.alpha_composite(a.img)
    a.img = under


def save_tile(a, path):
    """Seamless downscale (wrap-around resample) and fully opaque save."""
    img = a.img.convert("RGB")
    W, H = img.size
    big = Image.new("RGB", (W * 3, H * 3))
    for i in range(3):
        for j in range(3):
            big.paste(img, (i * W, j * H))
    w, h = a.w * UP, a.h * UP
    big = big.resize((w * 3, h * 3), Image.LANCZOS).crop((w, h, 2 * w, 2 * h))
    out = OUT / f"{path}.png"
    out.parent.mkdir(parents=True, exist_ok=True)
    big.convert("RGBA").save(out)
    return out


def rot(pts, cx, cy, ang):
    c, s = math.cos(ang), math.sin(ang)
    return [(cx + x * c - y * s, cy + x * s + y * c) for x, y in pts]


def petal(a, x, y, ang, size, face, shade):
    """A sakura petal (teardrop with a notch)."""
    p = [(-1.0, 0), (-0.45, -0.55), (0.45, -0.5), (1.0, -0.22), (0.78, 0), (1.0, 0.22), (0.45, 0.5), (-0.45, 0.55)]
    a.blob(rot([(px * size, py * size) for px, py in p], x, y, ang), shade)
    p2 = [(-0.8, -0.05), (-0.3, -0.42), (0.4, -0.38), (0.7, -0.12), (0.2, 0.05)]
    a.blob(rot([(px * size, py * size) for px, py in p2], x, y, ang), face)


def leaf(a, x, y, ang, size, face, shade):
    """A small maple leaf (5 lobes) with a stem."""
    pts = []
    for k in range(10):
        t = -math.pi / 2 + k * math.pi / 5
        r = size if k % 2 == 0 else size * 0.45
        if k in (4, 6):
            r *= 0.8
        pts.append((r * math.cos(t), r * math.sin(t)))
    a.blob(rot(pts, x, y, ang), shade, smooth=False)
    a.blob(rot([(px * 0.72, py * 0.72 - 0.08 * size) for px, py in pts], x, y, ang), face, smooth=False)
    a.stroke(rot([(0, 0), (0, size * 1.05)], x, y, ang), 0.22, shade)


def zigzag(a, x, y0, n, w, color="#eeeadc", shade="#b8b2a2", width=0.9):
    """Shide paper streamer hanging from (x, y0)."""
    pts = [(x, y0)]
    for i in range(n):
        pts.append((x + (w if i % 2 == 0 else 0), y0 + 1.0 + i * 1.15))
    a.stroke(pts, width + 0.25, shade, smooth=False)
    a.stroke(pts, width, color, smooth=False)


def ember_dots(a, rows, chars_col, ymax=99, r=0.5):
    for y, row in enumerate(rows):
        if y >= ymax:
            break
        for x, ch in enumerate(row):
            if ch in chars_col:
                a.ellipse(x + 0.5, y + 0.5, r, r, chars_col[ch])


# ───────────────────────── stage (prologue street) ─────────────────────────

PAVE = dict(A="#2b2840", B="#312d49", C="#27243a", M="#1c1a2b", H="#3a3654")
PAVE_SHADOW = {"A": "#242137", "B": "#29263e", "C": "#211e32"}
# Bands: (y0, y1 = grout row, stone x-ranges); x-ranges that pass 32 wrap around.
BANDS = [(0, 7, [(0, 7), (8, 15), (16, 23), (24, 31)]),
         (8, 15, [(28, 35), (4, 11), (12, 19), (20, 27)]),
         (16, 23, [(0, 7), (8, 15), (16, 23), (24, 31)]),
         (24, 31, [(28, 35), (4, 11), (12, 19), (20, 27)])]


def paving(a, rows):
    a.rect(-1, -1, 33, 33, PAVE["M"])
    for y0, y1, xs in BANDS:
        for x0, x1 in xs:
            tone = "A" if x1 > 32 else rows[y0 + 3][(x0 + 3) % 32]
            if tone not in PAVE_SHADOW:
                tone = "A"
            a.rect(x0 + 0.12, y0 + 0.1, x1 - 0.06, y1 - 0.04, PAVE_SHADOW[tone], radius=1.0)
            a.rect(x0 + 0.12, y0 + 0.1, x1 - 0.42, y1 - 0.4, PAVE[tone], radius=0.9)
            a.stroke([(x0 + 1.0, y0 + 0.5), (x1 - 1.3, y0 + 0.5)], 0.32, PAVE["H"], alpha=0.55)
            a.stroke([(x0 + 0.5, y0 + 1.1), (x0 + 0.5, y1 - 1.4)], 0.28, PAVE["H"], alpha=0.3)
    # Specular chips and cracks from the original.
    for y, r in enumerate(rows):
        for x, ch in enumerate(r):
            if ch == "H":
                a.ellipse(x + 0.6, y + 0.6, 0.55, 0.45, "#45405f", alpha=0.7)
    def grout(x, y):
        return any(y == y1 or (y0 <= y < y1 and x in [gx1 % 32 for _, gx1 in xs]) for y0, y1, xs in BANDS)
    inner_rows = ["".join("." if grout(x, y) else ch for x, ch in enumerate(r)) for y, r in enumerate(rows)]
    for inner in comps(inner_rows, "M"):
        if len(inner) > 6:
            continue
        inner.sort(key=lambda p: p[1])
        pts = [(x + 0.5, y + 0.5) for x, y in inner]
        if len(pts) == 1:
            pts = [(pts[0][0] - 0.2, pts[0][1] - 0.4), (pts[0][0] + 0.2, pts[0][1] + 0.4)]
        a.stroke(pts, 0.3, PAVE["M"], smooth=False)
        a.stroke([(px + 0.25, py) for px, py in pts], 0.18, PAVE["H"], alpha=0.35, smooth=False)


def stage_tiles():
    for name in ("paving_a", "paving_b"):
        rows, _ = grid(f"stage/{name}")
        a = Art(32, 32, tile=True)
        paving(a, rows)
        save_tile(a, f"stage/{name}")

    # Tactile strip (x 12..20) over paving.
    rows, _ = grid("stage/tactile")
    a = Art(32, 32, tile=True)
    paving(a, rows)
    a.rect(11.7, -1, 20.3, 33, "#1c1a2b")
    a.rect(12.0, -1, 20.0, 33, "#6e6028")
    a.rect(12.0, -1, 12.45, 33, "#5a4e20")
    a.rect(19.55, -1, 20.0, 33, "#5a4e20")
    for k in range(8):
        y = 1 + 4 * k
        for bx0, bx1 in ((13.0, 15.6), (16.4, 19.0)):
            a.rect(bx0, y + 0.25, bx1, y + 1.35, "#4e441c", radius=0.5)
            a.rect(bx0, y, bx1, y + 1.0, "#867636", radius=0.5)
            a.stroke([(bx0 + 0.5, y + 0.25), (bx1 - 0.5, y + 0.25)], 0.22, "#a08e48", alpha=0.7)
    save_tile(a, "stage/tactile")

    # Manhole.
    rows, _ = grid("stage/manhole")
    a = Art(32, 32, tile=True)
    paving(a, rows)
    cx, cy = 16.5, 16.0
    a.ellipse(cx, cy + 0.3, 11.2, 11.2, "#16141f", alpha=0.7)
    a.ellipse(cx, cy, 10.7, 10.7, "#1c1a2b")
    a.ellipse(cx, cy, 10.2, 10.2, "#3a3b52")
    a.ellipse(cx, cy, 9.1, 9.1, "#272838")
    a.ellipse(cx, cy, 8.7, 8.7, "#1f2030")
    for gx in (8.5, 12.5, 16.5, 20.5, 24.5):
        hc = math.sqrt(max(0.0, 8.2 ** 2 - (gx - cx) ** 2))
        if hc > 0.5:
            a.stroke([(gx, cy - hc), (gx, cy + hc)], 0.85, "#2a2b3d")
            a.stroke([(gx - 0.2, cy - hc + 0.3), (gx - 0.2, cy + hc - 0.3)], 0.25, "#353648", alpha=0.8)
    a.stroke([(cx - 8.2, 15.5), (cx + 8.2, 15.5)], 0.85, "#2a2b3d")
    a.stroke([(cx - 8.0, 15.3), (cx + 8.0, 15.3)], 0.25, "#353648", alpha=0.8)
    arc = [(cx + 9.65 * math.cos(t), cy + 9.65 * math.sin(t)) for t in np.linspace(math.pi * 0.95, math.pi * 1.6, 12)]
    a.stroke(arc, 0.4, "#4a4b66", alpha=0.8)
    arc = [(cx + 9.65 * math.cos(t), cy + 9.65 * math.sin(t)) for t in np.linspace(0.05, math.pi * 0.6, 12)]
    a.stroke(arc, 0.4, "#2c2c40", alpha=0.9)
    for bx, by in ((cx - 6.9, cy - 6.0), (cx + 6.9, cy + 6.0)):
        a.ellipse(bx, by, 0.55, 0.55, "#2f3045")
        a.ellipse(bx - 0.15, by - 0.15, 0.25, 0.25, "#4a4b66")
    save_tile(a, "stage/manhole")

    # Puddle reflecting neon.
    rows, _ = grid("stage/puddle")
    a = Art(32, 32, tile=True)
    paving(a, rows)
    soft(a, lambda t: t.ellipse(16, 18.2, 12.2, 6.7, "#ffffff"), "#191628", 0.6, 0.6)
    a.blob([(4.4, 17.5), (6.5, 13.8), (11.5, 11.6), (18, 11.3), (24.5, 12.6), (27.8, 16.2), (27.4, 20.4), (23, 23.6), (15.5, 24.4), (8.5, 23.4), (5, 21)], "#1d2440")
    soft(a, lambda t: t.ellipse(16, 15.3, 9.0, 2.6, "#ffffff"), "#232c4e", 1.0, 0.7)
    a.stroke([(6.8, 14.4), (11.5, 12.3), (18, 12.0)], 0.3, "#3a4468", alpha=0.6)
    for (x0, x1, y, col, core) in ((9.0, 14.2, 16.5, "#ff5aa8", "#ffc4e0"), (17.2, 21.2, 20.5, "#5ad8ff", "#c8f4ff")):
        soft(a, lambda t, x0=x0, x1=x1, y=y: t.stroke([(x0, y), (x1, y)], 1.0, "#ffffff"), col, 0.7, 0.5)
        a.stroke([(x0 + 0.3, y), (x1 - 0.3, y)], 0.55, col, alpha=0.85)
        a.stroke([(x0 + 1.0, y - 0.05), (x1 - 1.4, y - 0.05)], 0.22, core, alpha=0.8)
        a.stroke([(x0 + 1.5, y + 1.1), (x1 - 1.0, y + 1.1)], 0.25, col, alpha=0.3)
    save_tile(a, "stage/puddle")


def stage_props():
    # Crates.
    a = Art(14, 12)
    a.rect(0.9, 4.0, 13.1, 11.6, ("lin", "#a8743f", "#865a30", 90), radius=0.6)
    for y in (5.6, 9.7):
        a.stroke([(1.2, y), (12.8, y)], 0.22, "#6a4424", alpha=0.6)
    a.rect(0.9, 7.0, 13.1, 8.1, "#6a4424")
    a.stroke([(1.2, 7.1), (12.8, 7.1)], 0.22, "#8a5c32")
    a.rect(6.0, 4.0, 8.0, 11.6, "#6a4424")
    a.stroke([(6.2, 4.6), (6.2, 11.2)], 0.25, "#8a5c32")
    a.rect(0.9, 4.0, 1.9, 11.6, "#7e522c")
    a.rect(12.1, 4.0, 13.1, 11.6, "#74492a")
    for x, y in ((1.4, 4.6), (12.6, 4.6), (1.4, 11.0), (12.6, 11.0), (7.0, 7.55)):
        a.ellipse(x, y, 0.28, 0.28, "#3a2414")
        a.ellipse(x - 0.07, y - 0.07, 0.12, 0.12, "#d8a870")
    a.stroke([(1.0, 4.25), (13.0, 4.25)], 0.3, "#c08a52", alpha=0.7)
    a.rect(3.0, 3.6, 11.0, 4.9, "#3a2414", alpha=0.45, radius=0.4)
    a.rect(3.0, 0.9, 11.0, 4.5, ("lin", "#d8a066", "#b47a42", 90), radius=0.5)
    a.rect(3.0, 3.1, 11.0, 4.5, "#9a6a3a", radius=0.4)
    a.rect(6.0, 0.9, 8.0, 4.5, "#7a4e2a")
    a.stroke([(3.4, 1.3), (10.6, 1.3)], 0.3, "#f0c088", alpha=0.7)
    a.stroke([(3.4, 2.2), (5.6, 2.2)], 0.18, "#a8743f", alpha=0.6)
    a.stroke([(8.4, 2.2), (10.6, 2.2)], 0.18, "#a8743f", alpha=0.6)
    a.shade_side("#2a1408", 0.18, "right", 0.3)
    a.rim("#ffe0b0", 0.3)
    a.outline()
    a.save("stage/crates")

    # Red paper lantern (chochin).
    a = Art(12, 18)
    a.rect(5.0, 0.6, 7.0, 3.4, "#2a1a1a", radius=0.3)
    a.ellipse(6.0, 9.8, 5.15, 5.6, ("rad", "#ff7d5c", "#b8302c", (0.5, 0.55)))
    a.rect(0.2, 9.1, 1.2, 10.1, "#d8433a", radius=0.3)
    a.rect(10.8, 9.1, 11.8, 10.1, "#d8433a", radius=0.3)
    soft(a, lambda t: t.ellipse(6.0, 10.0, 3.6, 4.0, "#ffffff"), "#ffb070", 1.2, 0.55, clip=True)
    for y, bow in ((6.6, 0.25), (9.7, 0.35), (12.8, 0.25)):
        a.stroke([(0.6, y - bow), (3.5, y + bow * 0.6), (6, y + bow), (8.5, y + bow * 0.6), (11.4, y - bow)], 0.42, "#8c2424", clip=True, alpha=0.85)
    for y in (5.3, 8.1, 11.2, 14.1):
        a.stroke([(1.5, y), (6, y + 0.3), (10.5, y)], 0.18, "#ff9a80", clip=True, alpha=0.35)
    a.rect(3.9, 7.0, 8.1, 13.0, ("rad", "#fff6d0", "#ffc860", (0.5, 0.55)), radius=0.6)
    a.stroke([(4.9, 8.4), (7.1, 8.4)], 0.32, "#b84a2a", alpha=0.75)
    a.stroke([(6.0, 7.8), (6.0, 12.2)], 0.32, "#b84a2a", alpha=0.75)
    a.stroke([(6.0, 9.8), (4.6, 11.8)], 0.32, "#b84a2a", alpha=0.75)
    a.stroke([(6.0, 9.8), (7.4, 11.8)], 0.32, "#b84a2a", alpha=0.75)
    a.rect(2.1, 3.0, 9.9, 5.3, ("lin", "#3a2626", "#1e1212", 90), radius=0.8)
    a.stroke([(2.6, 5.0), (9.4, 5.0)], 0.3, "#c8963a")
    a.rect(2.1, 14.9, 9.9, 17.3, ("lin", "#3a2626", "#1e1212", 90), radius=0.8)
    a.stroke([(2.6, 15.2), (9.4, 15.2)], 0.3, "#c8963a")
    a.stroke([(2.8, 3.5), (9.2, 3.5)], 0.22, "#5a4040", alpha=0.8)
    a.ellipse(3.3, 7.0, 0.9, 0.55, "#ffd0c0", alpha=0.55)
    a.shade_side("#3a0010", 0.16, "right", 0.3)
    a.rim("#ffd0b0", 0.35)
    a.outline()
    glow2(a, "#ff7a40", 1.0, 0.3)
    halo(a, 6.0, 10.0, 7, "#ffb060", 0.18)
    a.save("stage/lantern")

    # Neon sign (katakana-like glyphs).
    a = Art(26, 10)
    a.rect(0.6, 0.6, 25.4, 9.4, ("lin", "#33204a", "#221632", 90), radius=1.0)
    a.rect(1.7, 2.4, 24.3, 7.6, "#1c1128", radius=0.6)
    a.stroke([(1.4, 0.95), (24.6, 0.95)], 0.25, "#4a3466", alpha=0.8)

    def neon(pts, col, core, w=0.85):
        soft(a, lambda t: t.stroke(pts, w * 2.0, "#ffffff", smooth=False), col, 0.7, 0.55, clip=True)
        a.stroke(pts, w, col, smooth=False)
        a.stroke(pts, w * 0.38, core, smooth=False, alpha=0.9)

    neon([(1.8, 1.55), (24.2, 1.55)], "#c02a78", "#ff8ac4", 0.7)
    neon([(1.8, 8.45), (24.2, 8.45)], "#c02a78", "#ff8ac4", 0.7)
    for x0, col, core in ((3.4, "#ff5aa8", "#ffd0e8"), (9.4, "#5ad8ff", "#d8f8ff"), (15.4, "#ff5aa8", "#ffd0e8"), (20.4, "#5ad8ff", "#d8f8ff")):
        xc = x0 + 1.6
        neon([(x0, 3.5), (x0 + 3.2, 3.5)], col, core)
        neon([(xc, 3.6), (xc, 6.4)], col, core)
        neon([(x0 - 0.1, 6.5), (x0 + 3.3, 6.5)], col, core)
    for x, y in ((1.2, 1.2), (24.8, 1.2), (1.2, 8.8), (24.8, 8.8)):
        a.ellipse(x, y, 0.3, 0.3, "#5a4478")
    a.outline()
    a.save("stage/neon_sign")

    # Vending machine.
    a = Art(16, 28)
    a.rect(0.9, 0.9, 15.1, 27.4, ("lin", "#3a6ee0", "#2850b8", 0), radius=1.0)
    a.rect(0.9, 0.9, 15.1, 3.4, "#1d3a8a", radius=1.0)
    a.rect(0.9, 2.6, 15.1, 3.4, "#1d3a8a")
    a.rect(3.0, 1.5, 13.0, 2.6, "#4a86f0", radius=0.5)
    a.stroke([(3.6, 1.9), (12.4, 1.9)], 0.25, "#a8d0ff", alpha=0.8)
    a.rect(1.9, 3.9, 14.1, 15.3, "#1d3a8a", radius=0.6)
    a.rect(2.2, 4.2, 13.8, 15.0, ("lin", "#ffffff", "#cfe4ff", 90), radius=0.4)
    cans = ["LYYR", "LLRL", "RRRY"]
    colors = {"L": ("#7fd8f8", "#4aa8d8"), "Y": ("#ffe066", "#d8a830"), "R": ("#ff5a6a", "#c8323e")}
    for j, rowc in enumerate(cans):
        y = 5.0 + 3 * j
        a.rect(2.4, y + 2.0, 13.6, y + 2.45, "#a8bcd8")
        for i, ch in enumerate(rowc):
            x = 3.0 + 3 * i
            face, sh = colors[ch]
            a.rect(x + 0.15, y - 0.1, x + 1.95, y + 2.0, sh, radius=0.55)
            a.rect(x + 0.15, y - 0.1, x + 1.55, y + 2.0, face, radius=0.55)
            a.rect(x + 0.35, y - 0.25, x + 1.75, y + 0.25, "#d8dce8", radius=0.25)
            a.stroke([(x + 0.55, y + 0.5), (x + 0.55, y + 1.6)], 0.25, "#ffffff", alpha=0.75)
            a.ellipse(x + 1.05, y + 1.1, 0.3, 0.3, "#ffffff", alpha=0.35)
    a.poly([(9.0, 4.2), (11.0, 4.2), (5.5, 15.0), (3.5, 15.0)], "#ffffff", clip=True, alpha=0.18)
    a.rect(1.9, 15.8, 10.2, 19.3, "#1d3a8a", radius=0.4)
    a.rect(2.2, 16.1, 9.9, 19.0, ("lin", "#c8f4ff", "#7fd4f4", 90), radius=0.3)
    for i in range(4):
        a.ellipse(3.3 + i * 1.8, 18.2, 0.45, 0.35, "#ffffff", alpha=0.8)
    a.stroke([(3.0, 16.8), (8.5, 16.8)], 0.35, "#ffffff", alpha=0.7)
    a.rect(10.9, 15.9, 14.1, 22.2, "#3a3a4a", radius=0.4)
    a.rect(12.1, 16.7, 12.9, 18.9, "#16161e", radius=0.3)
    a.rect(11.6, 20.0, 13.4, 21.4, "#22222e", radius=0.3)
    a.stroke([(11.2, 16.2), (13.8, 16.2)], 0.22, "#5a5a70")
    a.rect(1.9, 22.7, 14.1, 26.1, "#3a3a4a", radius=0.6)
    a.rect(2.6, 23.4, 13.4, 25.4, ("lin", "#14141c", "#2a2a38", 90), radius=0.4)
    a.stroke([(2.9, 23.6), (13.1, 23.6)], 0.3, "#5a5a70")
    a.rect(0.9, 26.5, 15.1, 27.4, "#1d3a8a")
    a.shade_side("#081040", 0.22, "right", 0.18)
    a.rim("#c8e0ff", 0.35)
    a.outline()
    soft(a, lambda t: t.rect(2.2, 4.2, 13.8, 15.0, "#ffffff"), "#e8f4ff", 1.2, 0.18)
    a.save("stage/vending")

    # Festival food stall.
    a = Art(32, 24)
    a.rect(1.0, 9.0, 3.0, 23.6, ("lin", "#9a6a46", "#6a4430", 0))
    a.rect(29.0, 9.0, 31.0, 23.6, ("lin", "#9a6a46", "#6a4430", 0))
    a.rect(1.0, 15.0, 31.0, 18.4, ("lin", "#9a6a46", "#7e5034", 90))
    a.rect(1.0, 18.2, 31.0, 20.6, "#5a3a24")
    a.stroke([(1.4, 15.3), (30.6, 15.3)], 0.35, "#c8946a")
    for y in (16.5, 17.6):
        a.stroke([(2.0, y), (12.0, y + 0.1), (22.0, y - 0.1), (30.0, y)], 0.15, "#6a4430", alpha=0.6)
    a.stroke([(1.4, 19.4), (30.6, 19.4)], 0.18, "#3e2616", alpha=0.6)
    a.rect(1.0, 20.4, 31.0, 21.0, "#2e1e14")
    # goods
    for i in range(5):
        x0 = 4.0 + 5 * i
        if i % 2 == 0:  # golden buns
            a.rect(x0, 14.4, x0 + 4.0, 15.3, "#7a4a2a", radius=0.3)
            for k, (bx, by) in enumerate(((x0 + 1.0, 13.6), (x0 + 3.0, 13.6), (x0 + 2.0, 12.8))):
                a.ellipse(bx, by, 1.05, 0.95, ("rad", "#fff0a0", "#e8a820", (0.4, 0.4)))
                a.ellipse(bx - 0.35, by - 0.35, 0.3, 0.22, "#ffffff", alpha=0.7)
        else:  # ramune bottles
            a.rect(x0, 14.5, x0 + 4.0, 15.3, "#3a6a5a", radius=0.3)
            for bx in (x0 + 0.95, x0 + 2.0, x0 + 3.05):
                a.rect(bx - 0.5, 12.6, bx + 0.5, 15.0, "#6be0a0", radius=0.45)
                a.rect(bx - 0.25, 11.9, bx + 0.25, 12.9, "#9af0c4", radius=0.2)
                a.stroke([(bx - 0.2, 13.0), (bx - 0.2, 14.5)], 0.18, "#e0fff0", alpha=0.8)
    # awning
    stripes = ["#d8433a", "#f2e6d0"]
    shade = ["#a8302c", "#c8b8a0"]
    for i in range(8):
        x0 = 4 * i
        c, s = stripes[i % 2], shade[i % 2]
        a.rect(max(x0, 0.6), 0.6, min(x0 + 4, 31.4), 7.8, c)
        a.ellipse(x0 + 2.0, 7.7, 1.75, 1.6, s)
        a.ellipse(x0 + 2.0, 7.5, 1.7, 1.45, c)
        a.rect(max(x0, 0.6), 6.4, min(x0 + 4, 31.4), 7.4, s, alpha=0.5)
    a.rect(0.6, 0.6, 31.4, 1.6, "#ffffff", alpha=0.12)
    for i in range(1, 8):
        a.stroke([(4 * i, 0.8), (4 * i, 7.2)], 0.18, "#5a1e1e", alpha=0.25)
    soft(a, lambda t: t.rect(3, 10, 29, 16, "#ffffff"), "#ffd27a", 1.5, 0.2, clip=True)
    a.shade_side("#20080a", 0.15, "right", 0.25)
    a.rim("#fff0d0", 0.3)
    a.outline()
    a.save("stage/stall")


# ───────────────────────── world 1: shrine valley ─────────────────────────

W1 = dict(A="#18232e", B="#1b2a33", D="#141c27", M="#1d3133", m="#223a38", P="#7a4a66", p="#a06684")


def w1_base(a):
    a.rect(-1, -1, 33, 33, W1["A"])
    soft(a, lambda t: (t.ellipse(8.5, 0.8, 5.0, 2.3, "#ffffff"), t.ellipse(8.0, 30.6, 5.5, 1.8, "#ffffff")), W1["D"], 1.0, 0.8)
    soft(a, lambda t: (t.ellipse(29.0, 26.5, 3.0, 2.2, "#ffffff"), t.ellipse(2.0, 23.5, 1.6, 1.3, "#ffffff")), W1["D"], 1.0, 0.5)
    soft(a, lambda t: (t.ellipse(0.5, 16.0, 1.4, 2.0, "#ffffff"), t.ellipse(24.0, 0.5, 1.8, 1.0, "#ffffff")), W1["B"], 1.0, 0.6)


def w1_petals(a, rows, margin=1.6, axes="xy", seed=1):
    rnd = random.Random(seed)
    for c in comps(rows, "Pp"):
        x = sum(p[0] for p in c) / len(c) + 0.5
        y = sum(p[1] for p in c) / len(c) + 0.5
        if ("x" in axes and not margin < x < 32 - margin) or ("y" in axes and not margin < y < 32 - margin):
            continue
        petal(a, x, y, rnd.uniform(0, math.tau), 0.6 + 0.07 * min(len(c), 3), "#94607e", "#64405a")


def w1_tiles():
    rows, _ = grid("w1/ground_a")
    a = Art(32, 32, tile=True)
    w1_base(a)
    soft_pixels(a, rows, "BM m", W1["B"], 1.2, 0.9)
    soft_pixels(a, rows, "Mm", W1["M"], 0.9, 1.0)
    soft_pixels(a, rows, "m", W1["m"], 0.7, 1.0)
    soft(a, lambda t: (t.ellipse(14, 7.5, 3, 1, "#ffffff"), t.ellipse(10, 20.5, 2.5, 1, "#ffffff")), "#2a463e", 0.8, 0.5, fade=(3, "xy"))
    w1_petals(a, rows)
    save_tile(a, "w1/ground_a")

    rows, _ = grid("w1/ground_b")
    a = Art(32, 32, tile=True)
    w1_base(a)
    soft_pixels(a, rows, "D", W1["D"], 1.0, 0.8)
    soft_pixels(a, rows, "B", W1["B"], 1.0, 0.8)
    stones = [(6.2, 6.1, 15.0, 12.9), (17.1, 8.1, 26.0, 15.9), (9.1, 17.1, 19.0, 24.9), (21.1, 19.1, 27.0, 24.9)]
    rnd = random.Random(4)
    for x0, y0, x1, y1 in stones:
        j = lambda: rnd.uniform(-0.35, 0.35)  # noqa: E731
        pts = [(x0 + 0.8 + j(), y0 + j()), ((x0 + x1) / 2 + j(), y0 - 0.2 + j()), (x1 - 0.6 + j(), y0 + 0.2 + j()),
               (x1 + j() * 0.5, (y0 + y1) / 2), (x1 - 0.5 + j(), y1 + j()), ((x0 + x1) / 2 + j(), y1 + 0.2),
               (x0 + 0.4 + j(), y1 - 0.2 + j()), (x0 - 0.1 + j() * 0.5, (y0 + y1) / 2)]
        a.blob([(x + 0.25, y + 0.45) for x, y in pts], "#10161f", alpha=0.85)
        a.blob(pts, "#1a2130")
        a.blob([(x0 + (x - x0) * 0.96, y0 + (y - y0) * 0.9) for x, y in pts], "#262f40")
        a.blob([(x0 + 1.5 + (x - x0) * 0.55, y0 + 0.8 + (y - y0) * 0.35) for x, y in pts], "#2a3446", alpha=0.9)
        a.stroke([(x0 + 1.2, y0 + 0.7), ((x0 + x1) / 2, y0 + 0.4), (x1 - 1.2, y0 + 0.6)], 0.32, "#34405a", alpha=0.85)
    for x, y, rx in ((11.0, 12.6, 1.2), (25.6, 8.6, 1.0), (11.2, 17.3, 1.1), (19.8, 15.5, 1.1), (26.4, 19.8, 0.9), (11.0, 24.5, 0.9)):
        a.ellipse(x, y, rx, 0.55, W1["M"])
        a.ellipse(x - 0.2, y - 0.15, rx * 0.6, 0.3, "#2a463e")
    a.stroke([(20.5, 10.5), (21.3, 12.0), (20.8, 13.2)], 0.22, "#151b26")
    w1_petals(a, rows, seed=2)
    save_tile(a, "w1/ground_b")

    rows, _ = grid("w1/decal")
    a = Art(32, 32, tile=True)
    w1_base(a)
    soft_pixels(a, rows, "D", W1["D"], 1.0, 0.7)
    soft_pixels(a, rows, "B", W1["B"], 1.0, 0.8)
    soft_pixels(a, rows, "M", W1["M"], 0.9, 1.0)
    cx, cy, R = 15.6, 15.2, 9.6
    soft(a, lambda t: t.ellipse(cx, cy, R - 1.4, R - 1.6, "#ffffff"), "#1d2a36", 1.6, 0.8)
    soft(a, lambda t: t.ellipse(cx, cy, 4.5, 4.2, "#ffffff"), "#3a2a40", 1.6, 0.35)
    ring = [(cx + R * math.cos(t), cy + R * 0.98 * math.sin(t)) for t in np.linspace(0, math.tau, 40)]
    a.stroke(ring, 1.15, "#4c4438", smooth=False)
    a.stroke(ring, 0.75, "#6e6250", smooth=False)
    for k in range(48):
        t = k / 48 * math.tau
        x, y = cx + R * math.cos(t), cy + R * 0.98 * math.sin(t)
        tx, ty = -math.sin(t), math.cos(t)
        a.stroke([(x - tx * 0.3 - math.cos(t) * 0.3, y - ty * 0.3 - math.sin(t) * 0.3), (x + tx * 0.3 + math.cos(t) * 0.3, y + ty * 0.3 + math.sin(t) * 0.3)], 0.16, "#3a3428", alpha=0.8)
    for c in comps(rows, "HS"):
        if len(c) < 3:
            continue
        x = sum(p[0] for p in c) / len(c) + 0.5
        y = sum(p[1] for p in c) / len(c) + 0.6
        a.ellipse(x + 0.2, y + 0.5, 1.75, 1.2, "#0e141c", alpha=0.8)
        a.blob([(x - 1.6, y + 0.3), (x - 0.9, y - 1.0), (x + 0.6, y - 1.25), (x + 1.6, y - 0.2), (x + 1.3, y + 0.9), (x - 0.8, y + 1.05)], "#262f40")
        a.blob([(x - 1.2, y - 0.1), (x - 0.6, y - 0.9), (x + 0.6, y - 1.0), (x + 0.9, y - 0.4), (x - 0.2, y + 0.1)], "#34405a")
        a.ellipse(x - 0.5, y - 0.6, 0.35, 0.22, "#4a5874")
    for x, y in ((11.4, 9.3), (23.5, 11.4), (8.6, 23.4), (22.6, 24.4)):
        zigzag(a, x, y, 3, 0.5, "#74747c", "#4a4a54", 0.45)
    w1_petals(a, rows, seed=3)
    save_tile(a, "w1/decal")

    rows, _ = grid("w1/path")
    a = Art(32, 32, tile=True)
    w1_base(a)
    soft_pixels(a, rows, "B", W1["B"], 1.0, 0.9)
    soft_pixels(a, rows, "D", W1["D"], 1.0, 0.8)
    for y0, seam in ((0, 13.5), (16, 18.5)):
        a.rect(4.9, y0 + 0.4, 27.1, y0 + 16.4, "#10161f", radius=0.8, alpha=0.8)
        a.rect(5.3, y0 + 12.6, 26.7, y0 + 15.6, "#202734", radius=0.6)
        a.rect(5.3, y0 + 14.9, 26.7, y0 + 15.8, "#191f2c", radius=0.4)
        a.rect(5.3, y0 + 0.05, 26.7, y0 + 12.9, "#28303f", radius=0.8)
        a.stroke([(5.9, y0 + 0.45), (26.1, y0 + 0.45)], 0.5, "#36415a")
        a.stroke([(5.7, y0 + 0.9), (5.7, y0 + 12.0)], 0.3, "#36415a", alpha=0.5)
        a.stroke([(5.6, y0 + 12.75), (26.4, y0 + 12.75)], 0.3, "#3a4560", alpha=0.6)
        a.stroke([(seam, y0 + 1.0), (seam + 0.1, y0 + 6.5), (seam - 0.1, y0 + 12.4)], 0.4, "#191f2c")
        a.stroke([(seam + 0.4, y0 + 1.0), (seam + 0.5, y0 + 12.2)], 0.2, "#36415a", alpha=0.45)
    # mottling of the slab faces
    def mott(t):
        for y, r in enumerate(rows):
            for x, ch in enumerate(r):
                if ch == "T":
                    t.rect(x, y, x + 1, y + 1, "#ffffff")
    soft(a, mott, "#2c3546", 0.8, 0.8, clip=True, fade=(2.5, "x"))
    for x, y, rx in ((27.0, 4.5, 0.9), (5.4, 21.0, 0.8), (26.6, 26.0, 0.8), (24.3, 9.2, 0.7)):
        a.ellipse(x, y, rx, 0.5, W1["M"])
        a.ellipse(x - 0.15, y - 0.1, rx * 0.6, 0.28, "#2a463e")
    rnd = random.Random(5)
    for c in comps(rows, "Pp"):
        x = sum(p[0] for p in c) / len(c) + 0.5
        y = sum(p[1] for p in c) / len(c) + 0.5
        if 2 < x < 30:
            petal(a, x, y, rnd.uniform(0, math.tau), 0.7, "#94607e", "#64405a")
    save_tile(a, "w1/path")


def w1_props():
    S, s, d, H, M, m = "#7d8698", "#5a6276", "#3c4356", "#a3acbc", "#4c7a4c", "#6c9c5a"

    def moss(a, x, y, rx, ry):
        a.ellipse(x, y, rx, ry, M)
        a.ellipse(x - rx * 0.25, y - ry * 0.3, rx * 0.55, ry * 0.5, m)

    # A: stone toro lantern.
    a = Art(16, 32)
    a.ellipse(8.0, 3.9, 1.95, 2.2, S)
    a.ellipse(8.0, 1.6, 0.7, 0.8, S)
    a.rect(6.1, 5.0, 9.9, 7.4, S, radius=0.4)
    a.rect(6.6, 7.0, 9.4, 8.2, s)
    a.blob([(0.9, 10.9), (2.6, 9.4), (5.2, 8.1), (8.0, 7.4), (10.8, 8.1), (13.4, 9.4), (15.1, 10.9), (14.6, 11.5), (8.0, 11.3), (1.4, 11.5)], ("lin", H, S, 90))
    a.rect(1.4, 10.4, 14.6, 11.6, s, radius=0.5)
    a.rect(3.9, 11.4, 12.1, 12.4, d)
    a.rect(4.0, 12.2, 12.0, 17.2, s, radius=0.3)
    a.rect(5.0, 12.8, 11.0, 16.7, "#c46a2a", radius=0.2)
    for x0 in (5.3, 8.3):
        a.rect(x0, 13.0, x0 + 2.4, 16.4, "#ff9a36", radius=0.2)
        soft(a, lambda t, x0=x0: t.ellipse(x0 + 1.2, 14.8, 0.8, 1.2, "#ffffff"), "#ffe9a8", 0.5, 1.0)
    a.rect(7.7, 12.9, 8.3, 16.6, "#c46a2a")
    a.rect(3.9, 16.9, 12.1, 19.1, S, radius=0.3)
    a.stroke([(4.2, 17.25), (11.8, 17.25)], 0.3, H, alpha=0.8)
    a.rect(3.9, 18.2, 12.1, 19.1, s, radius=0.2)
    a.blob([(6.0, 19.2), (10.0, 19.2), (10.3, 21.8), (10.9, 22.5), (10.3, 23.4), (10.0, 28.0), (6.0, 28.0), (5.7, 23.4), (5.1, 22.5), (5.7, 21.8)], S, smooth=False)
    a.rect(8.4, 19.2, 10.2, 28.0, s, clip=True, alpha=0.75)
    a.stroke([(5.4, 22.5), (10.6, 22.5)], 0.35, s)
    a.rect(4.0, 27.8, 12.0, 30.2, S, radius=0.5)
    a.stroke([(4.4, 28.2), (11.6, 28.2)], 0.35, H)
    a.rect(4.0, 29.2, 12.0, 30.2, s)
    a.rect(2.9, 30.0, 13.1, 31.8, d, radius=0.4)
    moss(a, 7.0, 4.7, 0.9, 0.45)
    moss(a, 8.9, 20.8, 0.8, 0.5)
    moss(a, 6.8, 24.8, 0.8, 0.5)
    moss(a, 7.4, 28.7, 1.1, 0.45)
    moss(a, 3.2, 10.2, 1.1, 0.35)
    a.stroke([(7.2, 2.5), (7.0, 4.0)], 0.35, H, alpha=0.8)
    a.shade_side("#101828", 0.2, "right", 0.32)
    a.rim("#dce4f0", 0.3)
    a.outline()
    halo(a, 8.0, 14.8, 8, "#ffcf7a", 0.35)
    soft(a, lambda t: t.rect(5.0, 12.8, 11.0, 16.7, "#ffffff"), "#fff0c0", 0.5, 0.25, clip=True)
    a.save("w1/prop_a")

    # B: hokora shrine.
    a = Art(32, 32)
    a.poly([(7.0, 6.8), (25.0, 6.8), (29.6, 12.3), (30.3, 13.9), (1.7, 13.9), (2.4, 12.3)], "#2c2a3e")
    for i in range(9):
        xt = 8.5 + i * 1.9
        xb = 3.5 + i * 3.1
        a.stroke([(xt, 7.2), (xb, 12.8)], 0.55, "#3e3c56", clip=True)
        a.stroke([(xt + 0.35, 7.2), (xb + 0.5, 12.8)], 0.2, "#4e4c6a", clip=True, alpha=0.6)
    a.rect(1.7, 12.4, 30.3, 14.2, "#3e3c56", radius=0.4)
    a.stroke([(2.2, 12.6), (29.8, 12.6)], 0.3, "#5a5878")
    a.blob([(5.2, 4.3), (6.8, 5.0), (16.0, 5.3), (25.2, 5.0), (26.8, 4.3), (26.6, 6.6), (24.0, 7.3), (8.0, 7.3), (5.4, 6.6)], ("lin", "#8a6248", "#5a3e2e", 90))
    a.stroke([(6.6, 5.5), (25.4, 5.5)], 0.3, "#a87a58", alpha=0.8)
    for x in (2.5, 7.5, 14.5, 16.8, 21.5, 23.6, 27.6):
        moss(a, x, 13.0, 0.8, 0.45)
    a.rect(7.9, 15.0, 24.1, 26.3, "#4a3226")
    a.rect(9.9, 15.0, 11.1, 26.3, "#6a4a38")
    a.rect(20.9, 15.0, 22.1, 26.3, "#6a4a38")
    a.rect(11.9, 15.6, 20.1, 25.4, ("lin", "#d8504a", "#a83030", 90), radius=0.3)
    a.rect(15.5, 15.6, 16.5, 25.4, "#8a2a2a")
    a.rect(11.9, 19.0, 20.1, 20.0, "#8a2a2a")
    a.stroke([(12.4, 16.0), (12.4, 25.0)], 0.25, "#f07068", alpha=0.6)
    a.ellipse(13.6, 21.8, 0.55, 0.55, "#ffd27a")
    a.ellipse(18.4, 21.8, 0.55, 0.55, "#ffd27a")
    a.rect(15.0, 23.0, 17.0, 24.0, "#ffd27a", radius=0.3)
    a.rect(7.6, 14.0, 24.4, 15.3, ("lin", "#ffe6a0", "#d8a848", 90), radius=0.3)
    a.ellipse(16.0, 25.4, 2.9, 0.8, "#c8c0b0")
    a.ellipse(16.0, 24.6, 2.4, 1.1, "#eeeadc")
    a.ellipse(16.0, 23.5, 1.6, 0.9, "#f8f6ee")
    a.ellipse(16.0, 22.8, 0.5, 0.4, "#ff9a3a")
    for x in (10.3, 15.3, 21.0):
        zigzag(a, x, 15.1, 3, 0.9, "#eeeadc", "#b8b2a2", 0.7)
    a.rect(3.9, 26.1, 28.1, 31.1, S)
    a.rect(3.9, 26.1, 28.1, 27.1, H)
    a.rect(3.9, 29.3, 28.1, 31.1, s)
    a.rect(2.9, 30.9, 29.1, 31.9, d, radius=0.3)
    moss(a, 5.6, 29.4, 1.2, 0.5)
    moss(a, 14.6, 30.4, 1.0, 0.45)
    a.shade_side("#101020", 0.18, "right", 0.25)
    a.rim("#e0e4f0", 0.25)
    a.outline()
    halo(a, 16.0, 20.0, 12, "#ffd27a", 0.12)
    a.save("w1/prop_b")

    # C: torii fragment.
    a = Art(48, 32)
    R, r, q = "#d4433a", "#962c2c", "#6a1e24"
    a.rect(7.0, 8.0, 11.0, 28.8, ("lin", "#e0544a", "#a83430", 0))
    a.rect(37.0, 8.0, 41.0, 28.8, ("lin", "#e0544a", "#a83430", 0))
    a.rect(22.0, 8.5, 26.0, 13.4, R)
    a.rect(25.0, 8.5, 26.0, 13.4, r)
    a.rect(3.9, 13.2, 45.1, 16.5, R, radius=0.3)
    a.rect(3.9, 15.4, 45.1, 16.5, r)
    a.stroke([(4.3, 13.6), (44.7, 13.6)], 0.35, "#f07060", alpha=0.7)
    a.rect(3.5, 7.4, 42.6, 9.3, R)
    a.rect(3.5, 8.4, 42.6, 9.3, r)
    a.blob([(0.7, 1.4), (2.3, 3.2), (4.6, 4.1), (20.0, 4.5), (39.0, 4.4), (41.6, 3.4), (43.5, 4.4), (43.0, 6.1), (41.4, 7.6), (4.0, 7.6), (1.6, 5.8), (0.6, 3.3)], ("lin", "#3a3648", "#1c1a26", 90))
    a.stroke([(1.4, 2.4), (4.6, 4.5), (20, 4.95), (39, 4.85), (41.6, 3.9)], 0.35, "#55506a", alpha=0.8)
    a.stroke([(7.6, 9.5), (7.6, 28.2)], 0.3, "#f07060", alpha=0.55)
    a.stroke([(37.6, 9.5), (37.6, 28.2)], 0.3, "#f07060", alpha=0.55)
    for x, y, h in ((10.6, 10.2, 2.0), (15.0, 13.6, 0.6), (31.5, 13.7, 0.6), (37.6, 20.5, 1.2), (40.0, 13.0, 1.5)):
        a.stroke([(x, y), (x + 0.1, y + h)], 0.35, q, alpha=0.8)
    for x, y in ((7.6, 23.2), (8.2, 24.4), (39.6, 25.8), (40.2, 27.3), (26.6, 14.5), (38.6, 18.6)):
        moss(a, x, y, 0.75, 0.6)
    rope = [(10.6, 18.0), (14.0, 19.6), (18.5, 21.0), (24.0, 21.6), (29.5, 21.0), (34.0, 19.6), (37.4, 18.0)]
    a.stroke(rope, 2.2, "#8e7440")
    a.stroke([(x, y - 0.25) for x, y in rope], 1.6, ("lin", "#e0c27a", "#c8a860", 90))
    path = hdkit._catmull(rope, False, 12)
    for k in range(2, len(path) - 2, 4):
        x, y = path[k]
        a.stroke([(x - 0.45, y - 0.85), (x + 0.45, y + 0.6)], 0.28, "#8e7440", alpha=0.9)
    for x, y in ((16.3, 21.0), (23.5, 22.4), (30.5, 21.0)):
        zigzag(a, x, y, 6, 1.1, "#f4f0e4", "#2a2430", 0.85)
    a.rect(5.5, 28.4, 12.5, 31.8, "#22202c", radius=0.3)
    a.rect(5.5, 28.4, 12.5, 29.3, d)
    a.rect(35.5, 28.4, 42.5, 31.8, "#22202c", radius=0.3)
    a.rect(35.5, 28.4, 42.5, 29.3, d)
    a.shade_side("#1a0010", 0.12, "right", 0.2)
    a.rim("#ffd8c8", 0.25)
    a.outline()
    a.save("w1/prop_c")

    # D: moss boulder with rope and ofuda.
    a = Art(24, 24)
    a.rect(1.9, 20.6, 22.1, 23.8, d, radius=0.6)
    a.rect(1.9, 20.6, 22.1, 21.8, M, radius=0.5)
    a.blob([(7.0, 5.3), (15.0, 5.0), (18.5, 7.8), (21.2, 10.4), (22.6, 13.6), (22.4, 17.4), (20.6, 20.4), (16.0, 21.5), (8.0, 21.6), (3.2, 20.6), (1.5, 17.0), (1.6, 13.0), (3.2, 10.2), (4.6, 8.2)], ("lin", "#9ca6b8", "#5e6678", 60))
    a.blob([(15.0, 9.0), (21.5, 11.0), (22.8, 16.5), (20.0, 21.0), (14.0, 21.6), (17.0, 17.0)], s, clip=True, alpha=0.8)
    a.blob([(2.0, 17.0), (8.0, 19.0), (14.0, 19.5), (20.0, 19.0), (19.5, 21.6), (4.0, 21.5)], s, clip=True, alpha=0.6)
    a.blob([(5.5, 10.8), (11.5, 10.4), (12.0, 12.3), (7.0, 12.8), (4.5, 12.4)], H, clip=True, alpha=0.75)
    a.ellipse(7.0, 11.2, 0.8, 0.4, "#d8e0ec", clip=True, alpha=0.7)
    a.blob([(6.5, 5.4), (15.0, 5.1), (17.8, 7.4), (17.0, 9.6), (15.0, 10.0), (12.0, 10.6), (9.0, 10.2), (6.5, 10.4), (4.3, 9.2)], M)
    a.blob([(7.2, 6.0), (12.0, 5.8), (12.5, 7.4), (8.5, 8.2), (6.0, 7.6)], m)
    for x, y in ((5.6, 10.4), (9.5, 10.9), (16.4, 9.9), (19.6, 11.5), (3.6, 12.2)):
        a.ellipse(x, y, 0.6, 0.7, M)
    a.ellipse(8.4, 6.4, 1.1, 0.4, "#8cc070", alpha=0.7)
    rope = [(0.4, 15.7), (6.0, 16.1), (12.0, 16.2), (18.0, 16.1), (23.6, 15.7)]
    a.stroke(rope, 1.25, "#8e7440")
    a.stroke([(x, y - 0.15) for x, y in rope], 0.85, "#d4b670")
    for k in range(1, 24, 2):
        a.stroke([(k - 0.3, 15.5), (k + 0.3, 16.5)], 0.2, "#8e7440")
    a.rect(12.2, 10.8, 16.4, 21.2, "#c8c0b0", radius=0.3)
    a.rect(12.2, 10.8, 15.9, 20.8, "#f0ecdc", radius=0.3)
    a.rect(12.7, 11.4, 15.4, 12.6, "#d43a3a", radius=0.2)
    a.stroke([(13.1, 13.6), (15.0, 13.6)], 0.35, "#d43a3a")
    a.stroke([(14.05, 13.6), (14.05, 19.6)], 0.4, "#d43a3a")
    a.stroke([(13.9, 15.2), (13.0, 16.4)], 0.32, "#d43a3a")
    a.stroke([(14.2, 15.2), (15.1, 16.4)], 0.32, "#d43a3a")
    a.stroke([(13.1, 17.8), (15.0, 17.8)], 0.32, "#d43a3a")
    a.ellipse(14.05, 20.2, 0.35, 0.3, "#d43a3a")
    a.shade_side("#101828", 0.15, "right", 0.25)
    a.rim("#e0e8f4", 0.3)
    a.outline()
    a.save("w1/prop_d")


# ───────────────────────── world 2: ember leaf ─────────────────────────

W2 = dict(A="#271c1b", B="#2e211e", D="#211818", H="#352622", a="#3a3332", b="#2f2a2b")
LEAF = {"R": ("#5e2a1e", "#4e2019"), "O": ("#7a3c1e", "#62301a"), "r": ("#4a2420", "#3c1d18"),
        "Y": ("#80542a", "#6c4420"), "E": ("#c45a24", "#a8461c")}


def w2_base(a):
    a.rect(-1, -1, 33, 33, W2["A"])
    soft(a, lambda t: (t.ellipse(2.0, 18.0, 3.0, 5.0, "#ffffff"), t.ellipse(30.5, 9.0, 2.0, 3.0, "#ffffff"), t.ellipse(29.5, 31.5, 3.0, 1.5, "#ffffff")), W2["B"], 1.2, 0.8)
    soft(a, lambda t: (t.ellipse(28.0, 29.5, 2.0, 1.6, "#ffffff"), t.ellipse(0.5, 4.0, 1.5, 1.5, "#ffffff")), W2["D"], 1.0, 0.6)


def w2_scatter(a, rows, margin=1.6, axes="xy", seed=1):
    rnd = random.Random(seed)
    def ok(x, y):
        return not (("x" in axes and not margin < x < 32 - margin) or ("y" in axes and not margin < y < 32 - margin))
    for c in comps(rows, "ab"):
        x = sum(p[0] for p in c) / len(c) + 0.5
        y = sum(p[1] for p in c) / len(c) + 0.5
        if ok(x, y) and len(c) >= 2:
            rx = 0.4 + 0.15 * min(len(c), 4)
            a.ellipse(x + 0.15, y + 0.25, rx, rx * 0.7, "#1a1212", alpha=0.5)
            a.ellipse(x, y, rx, rx * 0.7, "#352d2c")
            a.ellipse(x - rx * 0.3, y - rx * 0.25, rx * 0.4, rx * 0.25, "#433a39")
    for c in comps(rows, "RrOYE"):
        x = sum(p[0] for p in c) / len(c) + 0.5
        y = sum(p[1] for p in c) / len(c) + 0.5
        if ok(x, y):
            ch = max(set(rows[py][px] for px, py in c), key=lambda k: sum(1 for px, py in c if rows[py][px] == k))
            face, sh = LEAF[ch]
            leaf(a, x, y, rnd.uniform(-1.2, 1.2) + (math.pi if rnd.random() < 0.3 else 0), 0.9 + 0.2 * min(len(c), 4), face, sh)


def w2_tiles():
    rows, _ = grid("w2/ground_a")
    a = Art(32, 32, tile=True)
    w2_base(a)
    soft_pixels(a, rows, "B", W2["B"], 1.2, 0.8)
    soft_pixels(a, rows, "D", W2["D"], 1.0, 0.8)
    soft_pixels(a, rows, "H", W2["H"], 1.0, 0.9)
    w2_scatter(a, rows, seed=1)
    save_tile(a, "w2/ground_a")

    rows, _ = grid("w2/ground_b")
    a = Art(32, 32, tile=True)
    w2_base(a)
    soft_pixels(a, rows, "B", W2["B"], 1.2, 0.8)
    soft_pixels(a, rows, "D", W2["D"], 1.0, 0.8)
    soft_pixels(a, rows, "H", "#3a2c28", 1.1, 0.9)
    soft_pixels(a, rows, "H", "#433834", 1.6, 0.35)
    w2_scatter(a, rows, seed=2)
    save_tile(a, "w2/ground_b")

    rows, _ = grid("w2/decal")
    a = Art(32, 32, tile=True)
    w2_base(a)
    soft_pixels(a, rows, "B", W2["B"], 1.2, 0.8)
    soft_pixels(a, rows, "D", W2["D"], 1.0, 0.8)
    soft_pixels(a, rows, "H", W2["H"], 1.0, 0.9)
    cx, cy = 16.4, 16.4
    soft(a, lambda t: t.ellipse(cx, cy, 11.6, 9.6, "#ffffff"), "#1e1616", 1.2, 0.8)
    a.ellipse(cx, cy, 10.6, 8.8, "#3a3332")
    a.ellipse(cx, cy - 0.3, 9.9, 8.1, "#463c3a")
    a.ellipse(cx, cy + 0.3, 9.0, 7.5, "#2a2022")
    a.ellipse(cx, cy + 0.5, 8.6, 7.1, "#181213")
    soft(a, lambda t: t.ellipse(cx, cy + 0.8, 6.4, 5.0, "#ffffff"), "#110c0e", 0.8, 1.0)
    a.stroke([(cx - 8.4, cy - 1.8), (cx - 4, cy - 6.2), (cx + 4, cy - 6.4), (cx + 8.6, cy - 2.0)], 0.35, "#5a4e4a", alpha=0.7)
    cracks = [[(16.4, 17.0), (14.0, 15.4), (12.0, 13.2)], [(16.4, 17.0), (16.9, 14.2), (16.6, 12.6)],
              [(16.4, 17.0), (19.4, 15.0), (21.6, 13.4)], [(16.4, 17.0), (20.5, 18.4), (22.8, 19.6)],
              [(16.4, 17.0), (13.4, 19.0), (11.6, 20.6)], [(16.4, 17.0), (16.8, 20.2), (17.2, 22.0)],
              [(16.4, 17.0), (12.8, 16.4), (10.4, 16.0)]]
    for cr in cracks:
        soft(a, lambda t, cr=cr: t.stroke(cr, 1.4, "#ffffff", taper=True), "#d0541e", 0.8, 0.6)
    for cr in cracks:
        a.stroke(cr, 0.6, "#ff8a2a", taper=True)
        a.stroke(cr[:2], 0.28, "#ffc456", taper=True)
    soft(a, lambda t: t.ellipse(16.6, 17.2, 1.8, 1.4, "#ffffff"), "#ffc456", 0.8, 0.8)
    a.ellipse(16.5, 17.1, 0.8, 0.6, "#fff0c0")
    for x, y, c in ((12.4, 12.4, "#d0541e"), (14.9, 12.6, "#ff8a2a"), (24.5, 12.5, "#ff8a2a"), (25.2, 19.4, "#ff8a2a"), (13.8, 21.6, "#ff8a2a"), (12.0, 20.4, "#d0541e"), (20.0, 19.6, "#ffc456")):
        a.ellipse(x, y, 0.38, 0.38, c)
    w2_scatter(a, rows, seed=3)
    save_tile(a, "w2/decal")

    rows, _ = grid("w2/path")
    a = Art(32, 32, tile=True)
    w2_base(a)
    soft_pixels(a, rows, "B", W2["B"], 1.2, 0.8, fade=(3.0, "x"))
    soft_pixels(a, rows, "D", W2["D"], 1.0, 0.8, fade=(3.0, "x"))
    trail = "Ppqgvwn W"
    soft_pixels(a, rows, trail, "#33251e", 0.9, 1.0, fade=(1.5, "x"), grow=0.3)
    soft_pixels(a, rows, "Pqgvwn W", "#3b2b22", 0.9, 1.0, fade=(2.0, "x"))
    soft(a, lambda t: (t.ellipse(16, 8, 6, 5, "#ffffff"), t.ellipse(15, 30, 6, 2, "#ffffff")), "#43322a", 1.5, 0.45, fade=(3, "x"))
    rnd = random.Random(7)
    for c in comps(rows, "qn"):
        x = sum(p[0] for p in c) / len(c) + 0.5
        y = sum(p[1] for p in c) / len(c) + 0.5
        if (15.5 < y < 28.5 and 6 < x < 26) or not 3 < x < 29:
            continue
        rr = rnd.uniform(0.35, 0.55)
        a.ellipse(x, y, rr, rr * 0.7, "#4a3a30")
        a.ellipse(x - 0.1, y - 0.1, rr * 0.5, rr * 0.3, "#5a4a40")
    for (x0, x1, y0) in ((7.6, 24.4, 16.6), (7.6, 24.4, 20.6), (5.6, 25.4, 24.6)):
        a.rect(x0 - 0.2, y0 - 0.1, x1 + 0.2, y0 + 3.8, "#1c1414", radius=0.4)
        a.rect(x0, y0 + 0.25, x1, y0 + 3.35, ("lin", "#56402a", "#3e2a1a", 90), radius=0.4)
        a.rect(x0, y0 + 2.6, x1, y0 + 3.35, "#3a2618", radius=0.3)
        a.stroke([(x0 + 0.5, y0 + 0.65), (x1 - 0.5, y0 + 0.65)], 0.3, "#6a5034")
        for gx, gl in ((x0 + 3.5, 3.0), (x0 + 9.0, 4.0), (x1 - 4.0, 2.5)):
            a.stroke([(gx, y0 + 1.5), (gx + gl, y0 + 1.6)], 0.16, "#3a2618", alpha=0.8)
        for nx in (x0 + 0.9, x1 - 0.9):
            a.ellipse(nx, y0 + 1.5, 0.3, 0.3, "#2a2020")
            a.ellipse(nx - 0.08, y0 + 1.42, 0.15, 0.15, "#8a8280")
    w2_scatter(a, rows, margin=2.5, axes="x", seed=4)
    save_tile(a, "w2/path")


def w2_props():
    W, w, v, u = "#6a4a2e", "#4a3220", "#8a6440", "#2e2016"

    def flame(a, pts_outer, pts_mid, pts_core):
        a.blob(pts_outer, ("lin", "#ffb040", "#e0501c", 90))
        a.blob(pts_mid, "#ffd060")
        a.blob(pts_core, "#fff4c0")

    # A: torch pole with hanging lantern.
    a = Art(16, 32)
    a.rect(3.9, 28.3, 12.1, 31.8, "#4a4448", radius=0.5)
    a.rect(3.9, 28.3, 12.1, 29.2, "#5a5660", radius=0.4)
    a.ellipse(6.2, 30.2, 0.6, 0.4, "#38333a")
    a.ellipse(9.5, 30.9, 0.7, 0.35, "#38333a")
    a.rect(7.0, 9.5, 9.0, 28.5, ("lin", v, w, 0))
    for y in (12.5, 18.4, 20.4, 22.4, 24.4):
        a.stroke([(7.1, y), (8.9, y + 0.15)], 0.3, w, alpha=0.8)
    a.rect(3.8, 8.1, 13.3, 10.2, W, radius=0.3)
    a.rect(3.8, 9.3, 13.3, 10.2, w)
    a.stroke([(4.1, 8.45), (13.0, 8.45)], 0.3, v)
    a.stroke([(12.1, 10.0), (12.1, 11.6)], 0.3, "#2a2020")
    a.poly([(3.7, 5.2), (12.3, 5.2), (11.7, 7.6), (4.3, 7.6)], "#5a5660")
    a.rect(4.1, 6.9, 11.9, 8.2, "#4a4448")
    a.stroke([(4.0, 5.5), (12.0, 5.5)], 0.35, "#7a7680")
    a.rect(10.4, 11.3, 14.6, 17.6, u, radius=0.5)
    a.rect(10.9, 12.2, 14.1, 16.8, "#e08a3a", radius=0.3)
    soft(a, lambda t: t.ellipse(12.5, 14.5, 1.2, 1.8, "#ffffff"), "#ffd060", 0.6, 1.0)
    a.rect(11.6, 14.0, 13.4, 15.0, "#a02a24", radius=0.2)
    a.stroke([(12.5, 12.2), (12.5, 16.8)], 0.2, "#c06a2a", alpha=0.6)
    flame(a, [(7.6, 0.4), (9.4, 1.9), (10.9, 1.7), (11.3, 3.8), (10.8, 5.5), (5.2, 5.5), (4.6, 3.6), (5.6, 2.6), (6.5, 1.4)],
          [(7.9, 1.8), (9.4, 3.0), (10.0, 4.2), (9.4, 5.3), (6.6, 5.3), (6.0, 4.2), (6.8, 3.0)],
          [(8.2, 3.1), (9.0, 4.2), (8.8, 5.2), (7.4, 5.2), (7.3, 4.2)])
    a.shade_side("#100808", 0.2, "right", 0.3)
    a.rim("#ffe0b0", 0.25)
    a.outline()
    for x, y, c in ((11.8, 1.3, "#ff8a2a"), (4.3, 2.0, "#c8461c"), (12.6, 3.0, "#ffd060")):
        a.ellipse(x, y, 0.35, 0.35, c)
    halo(a, 8.0, 3.4, 9, "#ff8a2a", 0.4)
    halo(a, 12.5, 14.5, 6, "#ffd060", 0.25)
    a.save("w2/prop_a")

    # B: barrel, sack and crate.
    a = Art(32, 32)
    T, t, Q = "#7a5434", "#5a3c24", "#94704a"
    a.blob([(3.6, 13.6), (9.4, 12.8), (15.8, 13.6), (16.4, 22.5), (15.8, 31.7), (3.2, 31.7), (2.7, 22.5)], ("lin", Q, t, 0))
    for x in (6.0, 9.4, 12.6):
        a.stroke([(x, 13.8), (x + (x - 9.4) * 0.06, 22.5), (x, 31.5)], 0.22, "#4a3020", clip=True, alpha=0.7)
    a.stroke([(5.5, 14.4), (5.1, 22.5), (5.5, 31.0)], 0.6, "#a8845a", clip=True, alpha=0.7)
    for y in (16.5, 22.5, 28.5):
        a.rect(2.0, y - 0.55, 17.0, y + 0.55, "#4a4650", clip=True)
        a.stroke([(3.0, y - 0.25), (8.0, y - 0.3)], 0.25, "#8e8a96", clip=True)
    a.ellipse(9.4, 13.6, 6.0, 1.1, "#2e2016")
    a.ellipse(9.4, 13.9, 5.2, 0.7, "#5a3c24")
    a.rect(14.2, 19.2, 30.8, 31.8, ("lin", "#7a5636", "#5a3e26", 90), radius=0.3)
    a.rect(14.6, 19.6, 30.4, 20.8, v)
    a.rect(14.6, 30.2, 30.4, 31.4, w)
    a.stroke([(15.4, 30.0), (29.6, 21.0)], 1.0, v)
    a.stroke([(15.6, 30.4), (29.8, 21.4)], 0.3, w)
    for y in (24.0, 27.2):
        a.stroke([(14.8, y), (30.2, y)], 0.18, "#3a2618", alpha=0.7)
    a.blob([(15.4, 10.4), (19.4, 9.0), (25.6, 9.0), (28.6, 10.6), (28.9, 15.0), (28.6, 19.6), (15.6, 19.6), (15.2, 15.0)], ("lin", Q, t, 30))
    a.stroke([(17.6, 11.0), (17.0, 18.5)], 0.35, "#a8845a", clip=True, alpha=0.7)
    a.stroke([(26.5, 11.5), (27.2, 18.5)], 0.25, "#4a3020", clip=True, alpha=0.7)
    a.rect(18.4, 15.4, 26.6, 16.7, u, radius=0.4)
    a.rect(20.2, 15.7, 23.8, 16.5, "#8a2a22", radius=0.2)
    a.ellipse(22.3, 8.4, 2.6, 1.3, "#c8a870")
    a.ellipse(22.3, 7.6, 1.2, 0.8, "#c8a870")
    a.stroke([(20.4, 8.4), (24.2, 8.4)], 0.2, "#8e7448")
    rope_v = [(22.4, 9.4), (22.6, 14.0), (22.4, 19.5), (22.6, 31.7)]
    a.stroke(rope_v, 0.75, "#8e7448")
    a.stroke([(x - 0.12, y) for x, y in rope_v], 0.45, "#c8a870")
    a.stroke([(14.4, 22.6), (30.6, 22.6)], 0.75, "#8e7448")
    a.stroke([(14.4, 22.45), (30.6, 22.45)], 0.45, "#c8a870")
    for k in range(10, 31, 2):
        a.stroke([(22.25, k), (22.75, k + 0.6)], 0.15, "#8e7448")
    a.stroke([(28.6, 15.4), (30.9, 13.1)], 0.75, "#6e6a74")
    a.stroke([(28.7, 15.1), (30.7, 13.1)], 0.3, "#d0ccd4")
    a.ellipse(3.3, 30.7, 2.8, 1.3, "#8e7448")
    a.ellipse(3.3, 30.5, 2.3, 0.95, "#c8a870")
    a.ellipse(3.3, 30.5, 1.1, 0.45, "#8e7448")
    a.shade_side("#100804", 0.18, "right", 0.25)
    a.rim("#ffe0b8", 0.25)
    a.outline()
    a.save("w2/prop_b")

    # C: palisade of sharpened stakes.
    rows, _ = grid("w2/prop_c")
    a = Art(48, 32)
    stakes = [4, 9, 14, 19, 24, 29, 34, 39]
    tops, burnt = [], []
    for x in stakes:
        top = min(y for y in range(32) if any(rows[y][xx] not in ".K" for xx in range(x, x + 4)))
        tops.append(top)
        burnt.append(any(rows[y][xx] in "Xx" for y in range(13) for xx in range(x, x + 4)))
    for x, top, b in zip(stakes, tops, burnt):
        pts = [(x + 0.1, 30.8), (x + 0.1, top + 2.6), (x + 1.6, top + 0.3), (x + 2.2, top + 0.1), (x + 3.9, top + 2.4), (x + 3.9, 30.8)]
        a.poly(pts, ("lin", v, w, 0))
        a.poly([(x + 2.9, top + 1.7), (x + 3.9, top + 2.4), (x + 3.9, 30.8), (x + 2.9, 30.8)], w, clip=True)
        a.stroke([(x + 0.6, top + 2.6), (x + 0.6, 30.5)], 0.35, "#a07a50", clip=True, alpha=0.7)
        a.stroke([(x + 1.8, top + 4), (x + 1.7, top + 9), (x + 1.9, 29)], 0.15, "#4a3220", clip=True, alpha=0.6)
    a.rect(1.0, 13.2, 47.0, 16.0, W, radius=0.4)
    a.rect(1.0, 15.0, 47.0, 16.0, w, radius=0.3)
    a.stroke([(1.4, 13.6), (46.6, 13.6)], 0.35, v)
    a.rect(1.0, 23.2, 47.0, 26.0, W, radius=0.4)
    a.rect(1.0, 25.0, 47.0, 26.0, w, radius=0.3)
    a.stroke([(1.4, 23.6), (46.6, 23.6)], 0.35, v)
    for x in stakes:
        for y0 in (12.6, 22.6):
            a.rect(x + 1.0, y0, x + 3.0, y0 + 3.8, "#7e6640", radius=0.4)
            a.rect(x + 1.0, y0, x + 2.6, y0 + 3.8, "#b89a64", radius=0.4)
            for k in range(3):
                a.stroke([(x + 1.0, y0 + 0.9 + k * 1.1), (x + 3.0, y0 + 0.4 + k * 1.1)], 0.18, "#7e6640")
    a.rect(2.0, 30.0, 46.0, 31.2, u, radius=0.3)
    a.rect(2.0, 31.0, 46.0, 32.0, "#3a3440")
    # charred tops
    for x, top, b in zip(stakes, tops, burnt):
        if not b:
            continue
        def char(t, x=x, top=top):
            t.rect(x - 0.5, top - 1, x + 4.5, 10.5, "#ffffff")
        soft(a, char, "#2a2224", 1.2, 1.0, clip=True)
        a.stroke([(x + 1.0, top + 2.5), (x + 1.3, 8.5)], 0.3, "#3e3436", clip=True)
        a.stroke([(x + 2.8, top + 3.5), (x + 2.6, 9.0)], 0.25, "#151012", clip=True)
    a.shade_side("#100804", 0.15, "right", 0.2)
    a.rim("#ffe0b8", 0.22)
    a.outline()
    def embers(t):
        ember_dots(t, rows, {"F": "#ffffff", "f": "#ffffff", "e": "#ffffff"}, 13, 0.9)
    soft(a, embers, "#ff6a1a", 0.8, 0.6, clip=True)
    ember_dots(a, rows, {"F": "#ff8a2a", "f": "#ffd060", "e": "#c8461c"}, 13, 0.42)
    ember_dots(a, rows, {"f": "#fff4c0"}, 13, 0.2)
    a.save("w2/prop_c")

    # D: burnt stump with embers.
    rows, _ = grid("w2/prop_d")
    a = Art(24, 24)
    a.blob([(6.2, 6.0), (7.2, 4.4), (8.4, 5.6), (10.0, 2.0), (11.9, 2.3), (12.5, 5.2), (14.2, 4.3), (15.3, 5.6), (17.2, 5.9), (18.3, 8.0),
            (18.5, 17.6), (20.4, 19.4), (21.9, 21.2), (22.8, 22.6), (20.5, 23.6), (3.5, 23.6), (1.2, 22.4), (2.8, 20.6), (4.8, 19.0), (6.0, 17.4)],
           ("lin", "#3e3436", "#1e181a", 0), smooth=False)
    for x in (7.3, 9.6, 12.4, 15.2):
        a.stroke([(x, 9.5), (x + 0.2, 14), (x - 0.1, 19.5)], 0.35, "#4c3e3a", clip=True, alpha=0.6)
    a.stroke([(8.0, 8.4), (16.2, 8.4)], 1.1, "#5a3e2a")
    a.stroke([(10.0, 8.35), (14.6, 8.35)], 0.55, "#d0541e")
    a.stroke([(10.6, 8.3), (13.8, 8.3)], 0.22, "#ffd060")
    for x, y in ((8.6, 18.6), (11.6, 18.6), (11.5, 21.5), (5.0, 21.6), (20.0, 21.6)):
        a.ellipse(x, y, 0.55, 0.4, "#6a6466", alpha=0.8)
    a.shade_side("#000000", 0.2, "right", 0.3)
    a.rim("#ffb080", 0.2)
    a.outline()
    cracks = [[(9.6, 10.6), (9.5, 12.4), (8.6, 13.5), (7.7, 14.6), (8.5, 16.3), (9.0, 17.6)],
              [(15.6, 10.6), (14.6, 11.9), (13.9, 13.4), (14.4, 14.7), (13.7, 15.8), (14.1, 17.2), (14.6, 18.4)],
              [(12.9, 20.4), (12.0, 21.5)]]
    for cr in cracks:
        soft(a, lambda t, cr=cr: t.stroke(cr, 1.6, "#ffffff"), "#ff5a10", 0.8, 0.55, clip=True)
    for cr in cracks:
        a.stroke(cr, 0.55, "#e0601e")
        if len(cr) > 2:
            a.stroke(cr[1:-1], 0.22, "#ffd060")
    for x, y, c in ((4.5, 2.5, "#ff8a2a"), (15.5, 0.6, "#c8461c"), (18.5, 1.6, "#ffd060")):
        a.ellipse(x, y, 0.45, 0.45, c)
        halo(a, x, y, 2.2, c, 0.35)
        a.save("w2/prop_d")


# ───────────────────────── previews ─────────────────────────

TMP = pathlib.Path("/home/jinweiang/.claude/jobs/08925af1/tmp")


def tiled_patch(names, path_name, out, cols=6, rows_=4, seed=0):
    rnd = random.Random(seed)
    ims = {n: Image.open(OUT / f"{n}.png") for n in names + [path_name]}
    s = 128
    sheet = Image.new("RGBA", (cols * s, rows_ * s))
    for ty in range(rows_):
        for tx in range(cols):
            n = path_name if tx == 2 else rnd.choice(names)
            sheet.paste(ims[n], (tx * s, ty * s))
    sheet.save(out)


def compare(paths, out, scale=4):
    sys.path.insert(0, str(ROOT / "tools"))
    tiles = []
    for p in paths:
        rows, pal = grid(p)
        o = Image.new("RGBA", (len(rows[0]), len(rows)))
        for y, r in enumerate(rows):
            for x, ch in enumerate(r):
                if ch != ".":
                    o.putpixel((x, y), rgb(pal[ch]))
        hd = Image.open(OUT / f"{p}.png")
        tiles.append((o.resize((o.width * scale, o.height * scale), Image.NEAREST), hd.resize((o.width * scale, o.height * scale), Image.LANCZOS)))
    W = sum(a.width * 2 + 24 for a, _ in tiles) + 8
    H = max(a.height for a, _ in tiles) + 16
    sheet = Image.new("RGBA", (W, H), rgb("#3a3550"))
    x = 8
    for o, h in tiles:
        sheet.alpha_composite(o, (x, 8))
        sheet.alpha_composite(h, (x + o.width + 4, 8))
        x += o.width * 2 + 24
    sheet.save(out)


if __name__ == "__main__":
    stage_tiles()
    stage_props()
    w1_tiles()
    w1_props()
    w2_tiles()
    w2_props()
    if "--preview" in sys.argv:
        tiled_patch(["stage/paving_a"] * 6 + ["stage/paving_b"] * 5 + ["stage/manhole", "stage/puddle"], "stage/tactile", TMP / "hd_env_a_tiles_stage.png", seed=3)
        tiled_patch(["w1/ground_a"] * 5 + ["w1/ground_b"] * 3 + ["w1/decal"], "w1/path", TMP / "hd_env_a_tiles_w1.png", seed=3)
        tiled_patch(["w2/ground_a"] * 5 + ["w2/ground_b"] * 3 + ["w2/decal"], "w2/path", TMP / "hd_env_a_tiles_w2.png", seed=3)
        props = ["stage/crates", "stage/lantern", "stage/neon_sign", "stage/vending", "stage/stall",
                 "w1/prop_a", "w1/prop_b", "w1/prop_c", "w1/prop_d", "w2/prop_a", "w2/prop_b", "w2/prop_c", "w2/prop_d"]
        hdkit.preview(props, str(TMP / "hd_env_a.png"), scale=2, cols=5)
        compare(props[:5], TMP / "hd_env_a_cmp_stage.png")
        compare(props[5:9], TMP / "hd_env_a_cmp_w1.png")
        compare(props[9:], TMP / "hd_env_a_cmp_w2.png")
