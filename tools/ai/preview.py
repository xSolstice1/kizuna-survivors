"""Show candidate sprites the way the game draws them, so art is judged at real size.

    python3 -I tools/ai/preview.py <sprite.png ...> --name ren_chibi [--height 32] [--stage w1]

Each candidate is a transparent PNG, or a figure on flat green (keyed out with process.py's
key). It is trimmed and scaled to `--height` world units, like art/hd sprites (4 texels per
unit, drawn at 2 screen pixels per unit on desktop and 1 on small phones).

Writes art/ai/preview/<name>.png:
  - a lineup: the candidates beside the current heroes and the stage's enemies, at desktop
    size (1x) and on a retina screen (2x);
  - per candidate, the same busy horde scene cropped around the hero at three screens:
    desktop 1x, desktop retina, and a phone (1 px per unit).
"""
import argparse, pathlib, random
import numpy as np
from PIL import Image, ImageDraw, ImageFont

from process import key_green

ROOT = pathlib.Path(__file__).resolve().parents[2]
HD = ROOT / "art" / "hd"
OUT = ROOT / "art" / "ai" / "preview"
UP = 4                      # texels per world unit (sprites::UP)
HEROES = ["ren", "riku", "shun", "toya", "kuon"]
CROP = (440, 300)           # screen pixels shown per scene panel


def load(path):
    im = Image.open(path)
    if im.mode != "RGBA" or np.asarray(im)[..., 3].min() == 255:
        rgb, alpha = key_green(im)
        im = Image.fromarray(np.dstack([rgb, alpha * 255]).clip(0, 255).astype(np.uint8), "RGBA")
    return im.convert("RGBA")


def trim(im):
    box = im.getchannel("A").point(lambda a: 255 if a > 8 else 0).getbbox()
    return im.crop(box) if box else im


def fit(im, height):
    """Scale a trimmed figure so it is `height` world units tall, in texels."""
    im = trim(im)
    h = round(height * UP)
    return im.resize((max(1, round(im.width * h / im.height)), h), Image.LANCZOS)


def shadow(w):
    sh = Image.new("RGBA", (w, max(2, w // 3)), (0, 0, 0, 0))
    ImageDraw.Draw(sh).ellipse((0, 0, sh.width - 1, sh.height - 1), fill=(0, 0, 0, 100))
    return sh


def place(canvas, im, foot_x, foot_y):
    """Paste with the figure's feet at (foot_x, foot_y), plus a ground shadow."""
    sh = shadow(int(im.width * 0.8))
    canvas.alpha_composite(sh, (int(foot_x - sh.width / 2), int(foot_y - sh.height / 2)))
    canvas.alpha_composite(im, (int(foot_x - im.width / 2), int(foot_y - im.height)))


def ground(w, h, stage):
    tiles = [Image.open(p).convert("RGBA") for p in sorted((HD / stage).glob("ground_*.png"))]
    if not tiles:
        return Image.new("RGBA", (w, h), (38, 46, 40, 255))
    rnd = random.Random(1)
    g = Image.new("RGBA", (w, h))
    t = tiles[0].width
    for y in range(0, h, t):
        for x in range(0, w, t):
            g.paste(rnd.choice(tiles), (x, y))
    return g


def enemies(stage):
    names = ["fodder_a", "fodder_b", "ranged", "blinker"]
    return [Image.open(HD / stage / f"{n}.png").convert("RGBA") for n in names if (HD / stage / f"{n}.png").exists()]


def scene(hero, stage, seed=7):
    """A horde around the hero, in texel space (4 per world unit); hero at the centre."""
    W, H = 900 * UP, 520 * UP   # world view large enough for the phone panel
    c = ground(W, H, stage)
    rnd = random.Random(seed)
    foes = enemies(stage)
    cx, cy = W // 2, H // 2
    spots = []
    for _ in range(420):
        # a ring of pressure around the hero, thinning outwards
        ang = rnd.uniform(0, 6.283)
        r = (34 + abs(rnd.gauss(0, 120))) * UP
        spots.append((cx + r * np.cos(ang), cy + r * np.sin(ang) * 0.8))
    spots.append((cx, cy))
    for x, y in sorted(spots, key=lambda s: s[1]):
        if (x, y) == (cx, cy):
            place(c, hero, x, y)
        else:
            f = rnd.choice(foes)
            place(c, f.transpose(Image.FLIP_LEFT_RIGHT) if x > cx else f, x, y)
    return c


def view(c, px_per_unit, label, font):
    """Crop around the centre at a screen's scale (screen px per world unit)."""
    k = px_per_unit / UP
    w, h = round(CROP[0] / k), round(CROP[1] / k)
    cx, cy = c.width // 2, c.height // 2 - 16 * UP
    v = c.crop((cx - w // 2, cy - h // 2, cx - w // 2 + w, cy - h // 2 + h)).resize(CROP, Image.LANCZOS)
    ImageDraw.Draw(v).text((6, 4), label, fill="white", font=font, stroke_width=2, stroke_fill="black")
    return v


def lineup(cands, stage, font):
    heroes = [Image.open(HD / f"{h}.png").convert("RGBA") for h in HEROES if (HD / f"{h}.png").exists()]
    figs = [im for _, im in cands] + heroes + enemies(stage)
    gap = 6 * UP
    W = sum(f.width for f in figs) + gap * (len(figs) + 1)
    H = max(f.height for f in figs) + 14 * UP
    c = ground(W, H, stage)
    x = gap
    for f in figs:
        place(c, f, x + f.width / 2, H - 6 * UP)
        x += f.width + gap
    rows = []
    for ppu, label in [(2, "desktop 1x"), (4, "retina 2x")]:
        k = ppu / UP
        r = c.resize((round(W * k), round(H * k)), Image.LANCZOS)
        ImageDraw.Draw(r).text((6, 4), f"lineup, {label}  (new: left {len(cands)})", fill="white", font=font,
                               stroke_width=2, stroke_fill="black")
        rows.append(r)
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("sprites", nargs="+")
    ap.add_argument("--name", required=True)
    ap.add_argument("--height", type=float, default=32.0, help="world units (current heroes are 32)")
    ap.add_argument("--stage", default="w1")
    a = ap.parse_args()
    font = ImageFont.load_default(size=14)
    cands = [(pathlib.Path(p).stem, fit(load(p), a.height)) for p in a.sprites]

    rows = lineup(cands, a.stage, font)
    for name, im in cands:
        c = scene(im, a.stage)
        panels = [view(c, 2, f"{name}: desktop 1x", font), view(c, 4, "retina 2x (zoomed detail)", font), view(c, 1, "phone", font)]
        row = Image.new("RGBA", (sum(p.width for p in panels) + 8 * (len(panels) - 1), CROP[1]), (20, 20, 20, 255))
        for i, p in enumerate(panels):
            row.paste(p, (i * (CROP[0] + 8), 0))
        rows.append(row)

    W = max(r.width for r in rows)
    sheet = Image.new("RGBA", (W, sum(r.height for r in rows) + 8 * len(rows)), (20, 20, 20, 255))
    y = 0
    for r in rows:
        sheet.paste(r, (0, y))
        y += r.height + 8
    OUT.mkdir(parents=True, exist_ok=True)
    out = OUT / f"{a.name}.png"
    sheet.convert("RGB").save(out)
    print("wrote", out.relative_to(ROOT))


if __name__ == "__main__":
    main()
