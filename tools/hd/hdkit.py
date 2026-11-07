"""hdkit: smooth, anti-aliased sprite drawing for Kizuna Survivors' high-resolution art.

Every sprite is drawn in its ART-PIXEL coordinate space (the same size as its .px source,
e.g. a hero is 24x32) with float coordinates, supersampled 16x, then saved as a PNG at
4x the art size to art/hd/<path>.png. The game picks the PNG up automatically.

    from hdkit import Art, rgb
    a = Art(24, 32)                       # same size as art/src/ren.px
    a.blob([(6,10),(12,4),(18,10),(17,18),(7,18)], "#3a2a2a")        # smooth closed shape
    a.ellipse(12, 12, 5, 5.5, ("lin", "#ffe9dc", "#f2c9b8", 90))      # gradient fill
    a.stroke([(8,20),(10,26),(9,31)], 1.6, "#2a1c2a")                # smooth line
    a.shade_side("#000000", 0.18, side="right")                      # cel shadow on one side
    a.rim("#ffffff", 0.35)                                           # rim light on the top-left
    a.outline()                                                      # dark outline around everything
    a.save("ren")

Fills: "#rrggbb", "#rrggbbaa", (r,g,b[,a]), ("lin", c1, c2, angle_deg), ("rad", inner, outer).
Draw order matters (later shapes on top). Call outline() once, last (before save).
For seamless ground tiles use Art(32, 32, tile=True): every shape is drawn wrapped.
"""
import math
import pathlib
from PIL import Image, ImageChops, ImageDraw, ImageFilter

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "art" / "hd"
SS = 16   # supersample per art pixel
UP = 4    # saved resolution per art pixel
OUTLINE = "#1a1423"


def rgb(c, alpha=None):
    if isinstance(c, str):
        h = c.lstrip("#")
        v = [int(h[i:i + 2], 16) for i in range(0, len(h), 2)]
        if len(v) == 3:
            v.append(255)
    else:
        v = list(c) + ([255] if len(c) == 3 else [])
    if alpha is not None:
        v[3] = int(255 * alpha)
    return tuple(v)


def _catmull(points, closed, steps=10):
    pts = list(points)
    if len(pts) < 3:
        return pts
    out = []
    n = len(pts)
    rng = range(n) if closed else range(n - 1)
    for i in rng:
        p0 = pts[(i - 1) % n] if closed else pts[max(i - 1, 0)]
        p1 = pts[i]
        p2 = pts[(i + 1) % n]
        p3 = pts[(i + 2) % n] if closed else pts[min(i + 2, n - 1)]
        for s in range(steps):
            t = s / steps
            t2, t3 = t * t, t * t * t
            x = 0.5 * ((2 * p1[0]) + (-p0[0] + p2[0]) * t + (2 * p0[0] - 5 * p1[0] + 4 * p2[0] - p3[0]) * t2 + (-p0[0] + 3 * p1[0] - 3 * p2[0] + p3[0]) * t3)
            y = 0.5 * ((2 * p1[1]) + (-p0[1] + p2[1]) * t + (2 * p0[1] - 5 * p1[1] + 4 * p2[1] - p3[1]) * t2 + (-p0[1] + 3 * p1[1] - 3 * p2[1] + p3[1]) * t3)
            out.append((x, y))
    if not closed:
        out.append(pts[-1])
    return out


class Art:
    def __init__(self, w, h, tile=False):
        self.w, self.h, self.tile = w, h, tile
        self.W, self.H = w * SS, h * SS
        self.img = Image.new("RGBA", (self.W, self.H), (0, 0, 0, 0))

    # ── internals ──
    def _s(self, pts):
        return [(x * SS, y * SS) for x, y in pts]

    def _offsets(self):
        if not self.tile:
            return [(0, 0)]
        return [(dx * self.W, dy * self.H) for dx in (-1, 0, 1) for dy in (-1, 0, 1)]

    def _fill_layer(self, fill):
        """A full-size RGBA image of the fill (colour or gradient)."""
        if isinstance(fill, tuple) and fill and isinstance(fill[0], str) and fill[0] in ("lin", "rad"):
            kind = fill[0]
            if kind == "lin":
                _, c1, c2, ang = fill
                return self._linear(rgb(c1), rgb(c2), ang)
            _, c1, c2 = fill[:3]
            return self._radial(rgb(c1), rgb(c2), fill[3] if len(fill) > 3 else None)
        return Image.new("RGBA", (self.W, self.H), rgb(fill))

    def _linear(self, c1, c2, ang):
        # Build along the angle explicitly (0 = left->right, 90 = top->bottom).
        a = math.radians(ang)
        dx, dy = math.cos(a), math.sin(a)
        small = Image.new("L", (64, 64))
        px = small.load()
        for y in range(64):
            for x in range(64):
                t = ((x / 63 - 0.5) * dx + (y / 63 - 0.5) * dy) / (abs(dx) * 0.5 + abs(dy) * 0.5 + 1e-6) * 0.5 + 0.5
                px[x, y] = max(0, min(255, int(t * 255)))
        mask = small.resize((self.W, self.H), Image.BILINEAR)
        return Image.composite(Image.new("RGBA", (self.W, self.H), c2), Image.new("RGBA", (self.W, self.H), c1), mask)

    def _radial(self, c1, c2, centre):
        cx, cy = centre if centre else (0.5, 0.45)
        small = Image.new("L", (64, 64))
        px = small.load()
        for y in range(64):
            for x in range(64):
                d = math.hypot(x / 63 - cx, y / 63 - cy) / 0.6
                px[x, y] = max(0, min(255, int(d * 255)))
        mask = small.resize((self.W, self.H), Image.BILINEAR)
        return Image.composite(Image.new("RGBA", (self.W, self.H), c2), Image.new("RGBA", (self.W, self.H), c1), mask)

    def _paint(self, mask, fill, clip=False, alpha=1.0):
        layer = self._fill_layer(fill)
        if alpha < 1.0:
            mask = mask.point(lambda v: int(v * alpha))
        if clip:
            mask = ImageChops.multiply(mask, self.img.getchannel("A"))
        # Respect the fill's own alpha.
        fa = layer.getchannel("A")
        mask = ImageChops.multiply(mask, fa)
        solid = layer.copy()
        solid.putalpha(255)
        base = self.img
        comp = Image.composite(solid, base, mask)
        # Alpha: union.
        a = ImageChops.lighter(base.getchannel("A"), mask)
        comp.putalpha(a)
        self.img = comp

    def _mask(self):
        return Image.new("L", (self.W, self.H), 0)

    # ── shapes ──
    def poly(self, pts, fill, clip=False, alpha=1.0):
        m = self._mask()
        d = ImageDraw.Draw(m)
        for ox, oy in self._offsets():
            d.polygon([(x + ox, y + oy) for x, y in self._s(pts)], fill=255)
        self._paint(m, fill, clip, alpha)

    def blob(self, pts, fill, clip=False, alpha=1.0, smooth=True):
        """A smooth closed shape through the points (Catmull-Rom)."""
        self.poly(_catmull(pts, True) if smooth else pts, fill, clip, alpha)

    def ellipse(self, cx, cy, rx, ry, fill, clip=False, alpha=1.0):
        m = self._mask()
        d = ImageDraw.Draw(m)
        for ox, oy in self._offsets():
            d.ellipse([(cx - rx) * SS + ox, (cy - ry) * SS + oy, (cx + rx) * SS + ox, (cy + ry) * SS + oy], fill=255)
        self._paint(m, fill, clip, alpha)

    def rect(self, x0, y0, x1, y1, fill, radius=0.0, clip=False, alpha=1.0):
        m = self._mask()
        d = ImageDraw.Draw(m)
        for ox, oy in self._offsets():
            d.rounded_rectangle([x0 * SS + ox, y0 * SS + oy, x1 * SS + ox, y1 * SS + oy], radius=radius * SS, fill=255)
        self._paint(m, fill, clip, alpha)

    def stroke(self, pts, width, fill, clip=False, alpha=1.0, smooth=True, taper=False):
        """A smooth open line of `width` art pixels (rounded ends). `taper` thins it to a point."""
        path = _catmull(pts, False) if smooth and len(pts) > 2 else list(pts)
        m = self._mask()
        d = ImageDraw.Draw(m)
        n = len(path)
        for ox, oy in self._offsets():
            for i in range(n - 1):
                w = width * (1 - i / (n - 1)) if taper else width
                (x0, y0), (x1, y1) = path[i], path[i + 1]
                d.line([(x0 * SS + ox, y0 * SS + oy), (x1 * SS + ox, y1 * SS + oy)], fill=255, width=max(1, int(w * SS)))
                r = w * SS / 2
                d.ellipse([x1 * SS + ox - r, y1 * SS + oy - r, x1 * SS + ox + r, y1 * SS + oy + r], fill=255)
            r = width * SS / 2
            x0, y0 = path[0]
            d.ellipse([x0 * SS + ox - r, y0 * SS + oy - r, x0 * SS + ox + r, y0 * SS + oy + r], fill=255)
        self._paint(m, fill, clip, alpha)

    # ── lighting ──
    def shade_side(self, color="#000000", alpha=0.18, side="right", amount=0.4):
        """Cel shadow over the existing art on one side (right / left / bottom / top)."""
        m = self._mask()
        d = ImageDraw.Draw(m)
        if side == "right":
            d.rectangle([self.W * (1 - amount), 0, self.W, self.H], fill=255)
        elif side == "left":
            d.rectangle([0, 0, self.W * amount, self.H], fill=255)
        elif side == "bottom":
            d.rectangle([0, self.H * (1 - amount), self.W, self.H], fill=255)
        else:
            d.rectangle([0, 0, self.W, self.H * amount], fill=255)
        m = m.filter(ImageFilter.GaussianBlur(SS * 1.2))
        self._paint(m, color, clip=True, alpha=alpha)

    def rim(self, color="#ffffff", alpha=0.3, width=0.6, dx=-0.5, dy=-0.5):
        """Rim light: a thin bright edge on the light side (top-left by default)."""
        a = self.img.getchannel("A")
        shifted = ImageChops.offset(a, int(-dx * width * SS * 2), int(-dy * width * SS * 2))
        edge = ImageChops.subtract(a, shifted)
        self._paint(edge.filter(ImageFilter.GaussianBlur(SS * 0.3)), color, clip=True, alpha=alpha)

    def glow(self, color, radius=2.0, alpha=0.6):
        """Soft glow behind the art (for light sources and magic)."""
        a = self.img.getchannel("A").filter(ImageFilter.GaussianBlur(radius * SS))
        under = Image.new("RGBA", (self.W, self.H), rgb(color))
        under.putalpha(a.point(lambda v: int(v * alpha)))
        under.alpha_composite(self.img)
        self.img = under

    def outline(self, color=OUTLINE, width=0.9):
        """Dark outline around everything drawn so far (drawn behind it)."""
        a = self.img.getchannel("A").point(lambda v: 255 if v > 40 else 0)
        size = max(3, int(width * SS * 2) | 1)
        grown = a.filter(ImageFilter.MaxFilter(size)).filter(ImageFilter.GaussianBlur(SS * 0.15))
        under = Image.new("RGBA", (self.W, self.H), rgb(color))
        under.putalpha(grown)
        under.alpha_composite(self.img)
        self.img = under

    # ── output ──
    def save(self, path):
        out = OUT / f"{path}.png"
        out.parent.mkdir(parents=True, exist_ok=True)
        img = self.img.resize((self.w * UP, self.h * UP), Image.LANCZOS)
        if self.tile:
            # Tiles must be fully opaque.
            bg = Image.new("RGBA", img.size, img.getpixel((0, 0))[:3] + (255,))
            bg.alpha_composite(img)
            img = bg
        img.save(out)
        return out


def preview(paths, out, scale=1, bg="#16122a", cols=8):
    """A contact sheet of saved HD PNGs (by art path) for checking."""
    ims = [Image.open(OUT / f"{p}.png") for p in paths]
    cw = max(i.width for i in ims) * scale + 16
    ch = max(i.height for i in ims) * scale + 16
    rows = (len(ims) + cols - 1) // cols
    sheet = Image.new("RGBA", (cw * min(cols, len(ims)), ch * rows), rgb(bg))
    for k, im in enumerate(ims):
        im = im.resize((im.width * scale, im.height * scale), Image.LANCZOS)
        sheet.alpha_composite(im, ((k % cols) * cw + 8, (k // cols) * ch + 8))
    sheet.save(out)
    return out
