"""Redraw existing game art in the new style, as drop-in replacements.

    python3 tools/ai/restyle.py plan            # list the batches and the cost
    python3 tools/ai/restyle.py run [--only w1] # generate, then write art/hd/*.png in place
    python3 tools/ai/restyle.py apply           # re-run only the write-back from saved raws

The image model keeps an edit aligned with its input, so each old sprite is scaled up onto a
flat key colour (2x2 per call for sprites, one per call for ground tiles) with chibi Ren as the
style reference, and asked for the same subject, pose and size, only redrawn. The result is
keyed, re-aligned to the old sprite (phase correlation on the masks), clipped to a margin
around the old silhouette, and resized back to the old canvas, so layouts, anchors and
hitboxes stay exactly as they were.

Green subjects go on magenta so the key doesn't eat them. Ground tiles are made seamless after.
Raw outputs: art/ai/restyle/; the originals stay in git history.
"""
import argparse, concurrent.futures as cf, csv, json, pathlib, sys, time
import cv2
import numpy as np
from PIL import Image
from scipy import ndimage

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from gen import call, key  # noqa: E402
from process import bleed  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[2]
HD = ROOT / "art" / "hd"
RAW = ROOT / "art" / "ai" / "restyle"
SPEND = ROOT / "art" / "ai" / "spend.csv"
STYLE = ROOT / "art" / "ai" / "raw" / "ren_chibi_gemini-3-pro-image_0.png"
NANO, PRO = "google/gemini-nano-banana-2.1", "google/gemini-3-pro-image"
MODEL = NANO  # set by --model
CANVAS, CELL = 1024, 512
GREEN, MAGENTA = (0, 255, 0), (255, 0, 255)

# Not restyled here: the rig and Ren's new art, the old Ren sprite, and heroes (done with the rig).
SKIP = {"ren.png", "ren_front.png", "ren_face.png", "ren_divine.png", "riku.png", "shun.png", "toya.png", "kuon.png"}
BIG = {"boss", "final", "night_lord"}

STYLE_TEXT = (
    "polished modern anime game art: clean confident dark navy outlines with varied line weight, "
    "soft cel shading with 2-3 tones, subtle highlights and rim light, vibrant but tasteful colours, crisp edges"
)
SPRITES = (
    "Image 1 shows rough placeholder sprites from a 2D top-down anime game, each on the flat {bg} background. "
    "Repaint EVERY one of them as finished, high-quality game art in exactly the art style and quality of image 2 "
    "(the chibi girl): the same confident dark navy line art with varied weight, faces drawn like hers (large detailed anime eyes with highlights, small features), hair and fur "
    "drawn as clean locks with shine, cloth with simple folds, rich but tasteful colour. Characters and creatures use chibi "
    "proportions like image 2. "
    "Keep the glossy, modern finish of image 1: smooth glossy shading with soft gradients and bright specular highlights, like polished vinyl figures; combine it with the line art, faces and detail quality of image 2. "
    "Keep each sprite's identity from image 1: what it is, its design and main colours, its pose and which way it faces, "
    "and keep it in the same place and at about the same size. Do not add, remove, merge or move sprites. "
    "No text, no labels, no borders, no ground shadows. The background stays perfectly flat solid {bg_name} ({bg_hex}) everywhere."
)
ICONS = (
    "Image 1 shows rough placeholder UI icons from a 2D anime game, each on the flat {bg} background. "
    "Repaint EVERY icon as a finished, high-quality game icon in the art style and quality of image 2 ({style}). "
    "Keep the glossy, modern finish of image 1: smooth glossy shading with soft gradients and bright specular highlights, like polished vinyl figures; combine it with the line art, faces and detail quality of image 2. "
    "Keep each icon's subject, design and main colours, and keep it in the same place and at about the same size; "
    "make it read clearly at small size: bold simple shapes, a dark outline, clean shading. "
    "Do not add, remove or move icons. No text, no letters, no frames, no borders. "
    "The background stays perfectly flat solid {bg_name} ({bg_hex}) everywhere."
)
TILE = (
    "Image 1 is a ground texture tile seen from directly above, from a 2D top-down anime game. "
    "Redraw it in the art style of image 2 ({style}), as a flat top-down texture that fills the whole image edge to edge: "
    "same material, layout, colours and brightness as image 1, with cleaner shapes and soft painted detail. "
    "Keep it low-contrast so characters stand out on it. No characters, no objects, no text, no border, no perspective, no vignette."
)
BOSS = (
    "Image 1 shows a rough placeholder boss sprite from a 2D top-down anime game, on a flat {bg} background. "
    "Repaint it as an impressive, detailed, finished boss in exactly the art style and quality of image 2 (the chibi girl): "
    "the same dark navy line art with varied weight, detailed anime eyes and faces, "
    "clean locks of hair or fur, cloth folds, rich tasteful colour. "
    "Keep the glossy, modern finish of image 1: smooth glossy shading with soft gradients and bright specular highlights, like polished vinyl figures; combine it with the line art, faces and detail quality of image 2. "
    "Keep its identity from image 1: what it is, its design and main colours, its pose and facing, and keep it in the same place "
    "and at about the same size. "
    "No text, no border, no ground shadow. The background stays perfectly flat solid {bg_name} ({bg_hex}) everywhere."
)


_ORIG = {}


def load(rel):
    """The ORIGINAL art (as committed at HEAD), so re-runs and fixes never restyle a restyle."""
    if rel not in _ORIG:
        import io, subprocess
        r = subprocess.run(["git", "show", f"HEAD:art/hd/{rel}"], cwd=ROOT, capture_output=True)
        src = io.BytesIO(r.stdout) if r.returncode == 0 and r.stdout else HD / rel
        _ORIG[rel] = Image.open(src).convert("RGBA")
    return _ORIG[rel].copy()


def greenish(im):
    a = np.asarray(im).astype(float) / 255
    op = a[..., 3] > 0.5
    if not op.any():
        return 0.0
    r, g, b = (a[..., i][op] for i in range(3))
    return float(((g - np.maximum(r, b)) > 0.12).mean())


def pick_bg(im):
    """The key colour that clashes least with the sprite: green unless it has green-ish areas."""
    a = np.asarray(im).astype(float) / 255
    op = a[..., 3] > 0.5
    if not op.any():
        return "green"
    r, g, b = (a[..., i][op] for i in range(3))
    green = ((g - np.maximum(r, b)) > 0.03).mean()
    magenta = ((np.minimum(r, b) - g) > 0.15).mean()
    return "magenta" if green > magenta else "green"


def opaque(im):
    return (np.asarray(im)[..., 3] > 128).mean() > 0.98


def plan():
    files = sorted(p.relative_to(HD).as_posix() for p in HD.rglob("*.png") if not p.relative_to(HD).as_posix().startswith("rig/"))
    files = [f for f in files if f not in SKIP]
    tiles, bosses, sheets = [], [], {}
    for f in files:
        im = load(f)
        stem = pathlib.Path(f).stem
        folder = f.split("/")[0] if "/" in f else "misc"
        if opaque(im) and im.size == (128, 128):
            tiles.append(f)
        elif stem in BIG:
            bosses.append(f)
        else:
            kind = "icons" if folder in ("icons", "gear", "exotic", "pickups") else "sprites"
            bg = pick_bg(im)
            sheets.setdefault((folder, kind, bg), []).append(f)
    batches = []
    for (folder, kind, bg), fs in sheets.items():
        for i in range(0, len(fs), 4):
            batches.append({"id": f"{folder}_{kind}_{bg}_{i // 4}", "kind": kind, "bg": bg, "files": fs[i:i + 4], "model": MODEL})
    for f in bosses:
        batches.append({"id": f.replace("/", "_")[:-4], "kind": "boss", "bg": pick_bg(load(f)), "files": [f], "model": PRO})
    for f in tiles:
        batches.append({"id": "tile_" + f.replace("/", "_")[:-4], "kind": "tile", "bg": "none", "files": [f], "model": NANO})
    return batches


def layout(b):
    """Where each file goes on the input canvas: (file, x, y, w, h) of its scaled rect."""
    if b["kind"] == "tile":
        return [(b["files"][0], 0, 0, CANVAS, CANVAS)]
    cells = [(0, 0, CANVAS)] if len(b["files"]) == 1 else [(x, y, CELL) for y in (0, CELL) for x in (0, CELL)]
    out = []
    for f, (cx, cy, size) in zip(b["files"], cells):
        im = load(f)
        k = size * 0.78 / max(im.size)
        w, h = round(im.width * k), round(im.height * k)
        out.append((f, cx + (size - w) // 2, cy + (size - h) // 2, w, h))
    return out


def compose(b):
    bg = MAGENTA if b["bg"] == "magenta" else GREEN
    c = Image.new("RGBA", (CANVAS, CANVAS), bg + (255,))
    for f, x, y, w, h in layout(b):
        im = load(f).resize((w, h), Image.LANCZOS)
        c.alpha_composite(im, (x, y))
    return c.convert("RGB")


# Identity notes for sprites the model kept turning into anime girls.
HINTS = {
    "w1/fodder_a.png": " It is a kodama: a tiny tree spirit with a big round pale mint-green head and body, two black dot eyes and a small round 'o' mouth. No hair, no clothes, not human, not a girl.",
    "w5/mini.png": " He is a grown man: a sinister puppet sorcerer in a patchwork cloak with puppet strings on his fingers. Keep him a man, not a girl.",
}

NOT_HER = (" Image 2 is ONLY a reference for the drawing style: do not draw the girl from image 2, "
           "or any pink-haired girl, anywhere in the picture.")


def prompt(b):
    bg = b["bg"]
    fill = dict(style=STYLE_TEXT, bg=bg, bg_name="pure " + bg, bg_hex="#FF00FF" if bg == "magenta" else "#00FF00")
    return {"tile": TILE, "icons": ICONS, "boss": BOSS}.get(b["kind"], SPRITES).format(**fill) + "".join(HINTS.get(f, "") for f in b["files"]) + NOT_HER


def run_one(k, b):
    out = RAW / f"{b['id']}.png"
    if out.exists():
        return b["id"], 0.0, "cached"
    src = RAW / f"{b['id']}_in.png"
    compose(b).save(src)
    r = call(k, b["model"], prompt(b), [str(src), str(STYLE)], "1:1")
    if "error" in r:
        return b["id"], 0.0, "error " + str(r["error"])[:120]
    cost = float(r.get("usage", {}).get("cost") or 0)
    imgs = r["choices"][0]["message"].get("images") or []
    if not imgs:
        return b["id"], cost, "no image"
    import base64
    out.write_bytes(base64.b64decode(imgs[0]["image_url"]["url"].split(",", 1)[1]))
    return b["id"], cost, "ok"


# --- write-back -------------------------------------------------------------------------------

def key_alpha(rgb, bg):
    """Alpha from the key colour; spill is only cleaned near the edges, so pale greens or pinks
    inside a sprite keep their colour."""
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    spill = np.minimum(r, b) - g if bg == "magenta" else g - np.maximum(r, b)
    alpha = np.clip(((110.0 if bg == "magenta" else 90.0) - spill) / 60.0, 0, 1)
    edge = ndimage.binary_dilation(alpha < 0.5, iterations=3) & (spill > 0)
    rgb = rgb.copy()
    if bg == "magenta":
        rgb[..., 0] = np.where(edge, np.minimum(r, g + 40), r)
        rgb[..., 2] = np.where(edge, np.minimum(b, g + 40), b)
    else:
        rgb[..., 1] = np.where(edge, np.minimum(g, (r + b) / 2), g)
    return rgb, alpha


def bbox(a, t=0.1):
    ys, xs = np.nonzero(a > t)
    return (xs.min(), ys.min(), xs.max() + 1, ys.max() + 1) if len(xs) else None


def seamless(im):
    """Blend a tile with its half-offset copy so opposite edges meet."""
    a = np.asarray(im).astype(np.float32)
    h, w = a.shape[:2]
    rolled = np.roll(a, (h // 2, w // 2), axis=(0, 1))
    yy, xx = np.mgrid[0:h, 0:w]
    d = np.minimum(np.minimum(xx, w - 1 - xx), np.minimum(yy, h - 1 - yy)) / (min(h, w) / 2)
    m = np.clip((d - 0.12) / 0.3, 0, 1)[..., None]
    return Image.fromarray((a * m + rolled * (1 - m)).clip(0, 255).astype(np.uint8), a.shape[2] == 4 and "RGBA" or "RGB")


def apply(b):
    raw = RAW / f"{b['id']}.png"
    if not raw.exists():
        return
    out = Image.open(raw).convert("RGB").resize((CANVAS, CANVAS), Image.LANCZOS)
    rgb_all = np.asarray(out).astype(np.float32)
    cells = [(0, 0, CANVAS)] if len(b["files"]) == 1 else [(x, y, CELL) for y in (0, CELL) for x in (0, CELL)]
    for (f, x, y, w, h), (cx, cy, size) in zip(layout(b), cells):
        old = load(f)
        if b["kind"] == "tile":
            new = Image.fromarray(rgb_all.astype(np.uint8)).resize(old.size, Image.LANCZOS)
            # Decals are scattered single overlays, not a repeating ground: leave their edges.
            (new if "decal" in f else seamless(new)).convert("RGBA").save(HD / f, optimize=True)
            continue
        if opaque(old):
            # Full-rect sprites (signs, paper): the rect itself, under the old outline.
            crop = rgb_all[y:y + h, x:x + w].astype(np.uint8)
            im = Image.fromarray(crop).resize(old.size, Image.LANCZOS).convert("RGBA")
            im.putalpha(old.getchannel("A"))
            bleed(im).save(HD / f, optimize=True)
            continue
        rgb, alpha = key_alpha(rgb_all[cy:cy + size, cx:cx + size], b["bg"])
        # Drop specks: keep pieces at least 1% of the biggest one.
        lab, n = ndimage.label(alpha > 0.3)
        if n:
            areas = ndimage.sum(alpha > 0.3, lab, range(1, n + 1))
            keep = np.isin(lab, [i + 1 for i, ar in enumerate(areas) if ar >= 0.01 * areas.max()])
            alpha = alpha * ndimage.binary_dilation(keep, iterations=2)
        nb = bbox(alpha)
        ob = bbox(np.asarray(old)[..., 3].astype(np.float32) / 255)
        if nb is None or ob is None:
            continue
        # Fit the new sprite to the old one's box: same height (or width, if it got wider than
        # the canvas allows), bottom-centred, so the feet and footprint don't move.
        piece = np.dstack([rgb, alpha * 255]).clip(0, 255).astype(np.uint8)[nb[1]:nb[3], nb[0]:nb[2]]
        pw, ph = nb[2] - nb[0], nb[3] - nb[1]
        ow, oh = ob[2] - ob[0], ob[3] - ob[1]
        k = oh / ph
        if pw * k > old.width:
            k = old.width / pw
        tw, th = max(1, round(pw * k)), max(1, round(ph * k))
        im = Image.fromarray(piece, "RGBA").resize((tw, th), Image.LANCZOS)
        px = int(round((ob[0] + ob[2]) / 2 - tw / 2))
        px = min(max(px, 0), old.width - tw)
        oa = np.asarray(old)[..., 3] > 25

        def placed(img):
            c = Image.new("RGBA", old.size)
            c.alpha_composite(img, (px, ob[3] - th))
            return c

        # The model sometimes mirrors a sprite; keep whichever way matches the old facing.
        best = max((placed(im), placed(im.transpose(Image.FLIP_LEFT_RIGHT))),
                   key=lambda c: ((np.asarray(c)[..., 3] > 25) & oa).sum() / max(1, ((np.asarray(c)[..., 3] > 25) | oa).sum()))
        bleed(best).save(HD / f, optimize=True)


def match_levels(img, ref):
    """Shift a tile's colour mean and spread to match another (per channel), so alternating
    tiles don't checkerboard."""
    a, r = (np.asarray(x.convert("RGB")).astype(np.float32) for x in (img, ref))
    out = (a - a.mean((0, 1))) / (a.std((0, 1)) + 1e-3) * r.std((0, 1)) + r.mean((0, 1))
    return Image.fromarray(out.clip(0, 255).astype(np.uint8)).convert("RGBA")


def harmonize():
    """Per stage: match ground_b / path / decal to the main ground tile, and feather each decal
    into it, so the ground reads as one surface."""
    for base, others, decals in [(f"{w}/ground_a.png", [f"{w}/ground_b.png", f"{w}/path.png"], [f"{w}/decal.png"]) for w in ("w1", "w2", "w3", "w4", "w5")] + \
            [("stage/paving_a.png", ["stage/paving_b.png", "stage/tactile.png"], ["stage/manhole.png", "stage/puddle.png"])]:
        ref = Image.open(HD / base)
        for f in others:
            if (HD / f).exists():
                seamless(match_levels(Image.open(HD / f), ref)).save(HD / f, optimize=True)
        for f in decals:
            if not (HD / f).exists():
                continue
            d = np.asarray(match_levels(Image.open(HD / f), ref)).astype(np.float32)
            g = np.asarray(ref.convert("RGBA")).astype(np.float32)
            h, w = d.shape[:2]
            yy, xx = np.mgrid[0:h, 0:w]
            rr = np.hypot((xx - w / 2) / (w / 2), (yy - h / 2) / (h / 2))
            m = np.clip((1.0 - rr) / 0.35, 0, 1)[..., None]       # solid centre, ground at the edges
            Image.fromarray((d * m + g * (1 - m)).clip(0, 255).astype(np.uint8)).save(HD / f, optimize=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["plan", "run", "apply", "harmonize"])
    ap.add_argument("--only", default="")
    ap.add_argument("--model", default="nano", choices=["nano", "pro"], help="model for sprite sheets and tiles (bosses always use pro)")
    ap.add_argument("--tag", default="", help="suffix for raw outputs, to compare runs")
    ap.add_argument("--files", nargs="*", default=[], help="redo just these art/hd files, one per call (pro)")
    a = ap.parse_args()
    if a.cmd == "harmonize":
        harmonize()
        return
    global MODEL
    MODEL = PRO if a.model == "pro" else NANO
    if a.tag:
        global RAW
        RAW = RAW / a.tag
    batches = [b for b in plan() if a.only in b["id"]]
    if a.files:
        def one(f):
            im = load(f)
            kind = "tile" if opaque(im) and im.size == (128, 128) else "boss" if pathlib.Path(f).stem in BIG else \
                "icons" if f.split("/")[0] in ("icons", "gear", "exotic", "pickups") else "sprites"
            return {"id": "fix_" + f.replace("/", "_")[:-4], "kind": kind, "bg": pick_bg(im), "files": [f], "model": PRO}
        batches = [one(f) for f in a.files]
    if a.cmd == "plan":
        cost = sum(0.14 if b["model"] == PRO else 0.04 for b in batches)
        for b in batches:
            print(f"{b['id']:40s} {b['model'].split('/')[-1]:22s} {' '.join(b['files'])}")
        print(f"{len(batches)} calls, {sum(len(b['files']) for b in batches)} files, ~${cost:.2f}")
        (RAW / "plan.json").parent.mkdir(parents=True, exist_ok=True)
        (RAW / "plan.json").write_text(json.dumps(batches, indent=1))
        return
    RAW.mkdir(parents=True, exist_ok=True)
    if a.cmd == "run":
        k = key()
        total = 0.0
        with cf.ThreadPoolExecutor(8) as ex:
            for bid, cost, status in ex.map(lambda b: run_one(k, b), batches):
                total += cost
                print(f"{bid:40s} {status} ${cost:.3f}", flush=True)
                if cost:
                    new = not SPEND.exists()
                    with SPEND.open("a", newline="") as fh:
                        w = csv.writer(fh)
                        if new:
                            w.writerow(["time", "name", "model", "cost"])
                        w.writerow([time.strftime("%Y-%m-%d %H:%M:%S"), "restyle_" + bid, "", f"{cost:.6f}"])
        print(f"spent ${total:.3f}")
    for b in batches:
        apply(b)
    print("applied", len(batches), "batches")


if __name__ == "__main__":
    main()
