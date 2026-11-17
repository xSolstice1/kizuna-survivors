"""Foliage that moves: grass tufts per world.

    python3 tools/ai/foliage_art.py          # generate what's missing (pro, 4 per call), then cut
    python3 tools/ai/foliage_art.py cut      # re-cut the saved sheets

Grass: art/hd/<world>/grass_0..3.png, tufts seen from slightly above, roots at the bottom
centre (the game sways their tops and bends them away from the hero). The world's own ground
texture is the style and colour reference, so tufts sit in the painted ground.
Raw sheets: art/ai/foliage/.
"""
import json, pathlib, sys

import numpy as np
from PIL import Image
from scipy import ndimage

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from gear_art import call, PRO  # noqa: E402
from icons_hd import cell  # noqa: E402
from process import bleed  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[2]
HD = ROOT / "art" / "hd"
RAW = ROOT / "art" / "ai" / "foliage"

SHEETS = {
    "w1/grass": ("art/hd/w1/terrain/ground_a.jpg", "magenta", [
        "a lush clump of tall wild grass blades, deep green with lighter tips, a few blades bending",
        "a low wide tuft of soft grass mixed with clover leaves",
        "a clump of tall grass with two small white wildflowers",
        "a tuft of thin feathery grass with seed heads",
    ]),
    "w3/grass": ("art/hd/w3/terrain/ground_a.jpg", "green", [
        "a clump of dry autumn grass blades, golden and rust coloured, a few bending",
        "a low tuft of dry grass with two fallen red maple leaves caught in it",
        "a tall clump of pampas-like silver plume grass",
        "a tuft of brown and orange autumn grass",
    ]),
}

PROMPT = (
    "A 2x2 sheet of four separate game sprites for a polished modern 2D anime action game seen from above at a slant, "
    "one centred in each quarter, each filling about 80% of its quarter, on a perfectly flat solid {bg} ({hx}) background. "
    "Top left: {0}. Top right: {1}. Bottom left: {2}. Bottom right: {3}. "
    "{style} Each one stands alone with its base at the bottom of its quarter, no ground, no shadow, no frame, no text."
)
GRASS_STYLE = ("Use the reference image ONLY for colours and painting style: do NOT draw its ground, stones or any texture behind "
               "the grass. Only the grass itself, isolated on the flat key colour, soft painterly shading with gentle highlights, no outlines.")


def generate():
    RAW.mkdir(parents=True, exist_ok=True)
    for name, (ref, bg, subjects) in SHEETS.items():
        out = RAW / f"{name.replace('/', '_')}.png"
        if out.exists():
            continue
        text = PROMPT.format(*subjects, bg="pure " + bg, hx="#FF00FF" if bg == "magenta" else "#00FF00",
                             style=GRASS_STYLE)
        refs = [ROOT / ref] if ref else []
        if ref:
            tmp = RAW / f"ref_{name.replace('/', '_')}.png"
            Image.open(ROOT / ref).convert("RGB").resize((512, 512)).save(tmp)
            refs = [tmp]
        data, msg = call(PRO, text, refs, "foliage_" + name.replace("/", "_"), "2K")
        print(msg)
        if data:
            out.write_bytes(data)
            out.with_suffix(".json").write_text(json.dumps({"bg": bg, "name": name}))


def cut():
    for js in sorted(RAW.glob("*.json")):
        meta = json.loads(js.read_text())
        for i in range(4):
            rgb, alpha = cell(js.with_suffix(".png"), i, meta["bg"], False)
            if meta["bg"] == "magenta":
                # Key-colour fringe (pink-purple where red and blue both beat green): drop it.
                r, g, bl = rgb[..., 0], rgb[..., 1], rgb[..., 2]
                alpha = alpha * ~((r > g + 30) & (bl > g + 30))
                # And dark purple scraps at the roots (only low down: seed heads may be pink).
                low = np.zeros_like(alpha, bool)
                ys = np.where(alpha.max(1) > 0.05)[0]
                if len(ys):
                    low[int(ys.max() - 0.18 * (ys.max() - ys.min())):] = True
                alpha = alpha * ~(low & (r > g + 6) & (bl > g + 6))
            b = alpha.shape[0] // 20
            alpha = alpha.copy()
            alpha[:b] = alpha[-b:] = 0
            alpha[:, :b] = alpha[:, -b:] = 0
            lab, n = ndimage.label(alpha > 0.3)
            if n:
                areas = ndimage.sum(alpha > 0.3, lab, range(1, n + 1))
                alpha = alpha * ndimage.binary_dilation(np.isin(lab, [k + 1 for k, ar in enumerate(areas) if ar >= 0.02 * areas.max()]), iterations=2)
            ys, xs = np.where(alpha > 0.05)
            im = Image.fromarray(np.dstack([rgb, alpha * 255]).clip(0, 255).astype(np.uint8), "RGBA").crop((xs.min(), ys.min(), xs.max() + 1, ys.max() + 1))
            # 128 px tall is plenty (grass is drawn ~12 units tall).
            h = 128
            im = im.resize((max(1, round(im.width * h / im.height)), h), Image.LANCZOS)
            out = HD / f"{meta['name']}_{i}.png"
            out.parent.mkdir(parents=True, exist_ok=True)
            bleed(im).save(out, optimize=True)
            print(out.relative_to(ROOT), im.size)


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "cut":
        cut()
    else:
        generate()
        cut()
