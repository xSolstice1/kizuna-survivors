"""Worn gear fitted to any rigged hero from another hero's pieces: no image generation.

    python3 -I tools/ai/gear_fit.py hero <name>           # a new rigged hero: every worn piece, $0
    python3 -I tools/ai/gear_fit.py fit <src> <dst> [--only top_haori_1 ...] [--out DIR]
    python3 -I tools/ai/gear_fit.py preview <dst> <piece ...> [--out img.png]

Every rigged hero has the same nine parts (gear_art.PARTS) on the same 1024 canvas, and a worn
piece is a patch over some of them (see gear_art.py). Fitting maps each of the target's parts
onto the source's, so the source's patch can be sampled for every target pixel:

- One smooth warp (a thin-plate spline) from the target's canvas to the source's, through the
  same body points on both (`Hero.landmarks`: neck, waist, shoulders, elbows, hands, hips,
  ankles, soles, the torso's sides): a top's hem lands on the target's waist, a sleeve follows
  its arm, and the gear keeps its own shape beyond the body (a hakama's flare, a wide sleeve).
- Inside the target's own outline the gear must cover everything (or the old clothes show):
  any gap the warp leaves takes the nearest gear colour.

- Hats and gloves are laid over the hero: only what the source's edit changed (the item, not
  the hair or skin it was drawn on) is taken, placed by the head's outline or along the arm.
- What isn't clothes on the target (`Hero.keep`: a flame in the hand, a weapon, hair over the
  shoulders) stays its own. A new hero has no pieces to learn that from: design it with arms
  hanging clear, nothing held over its clothes and hair off the shoulders (Toya's pose).

Toya is the default source: his arms hang separate from the body, so his sleeves are drawn on
his arms (Kuon's are on his torso).

The result is written exactly as gear_art.slice writes it (art/hd/wear/<dst>/<piece>.<part>.png
and the rects in art/rig/<dst>/wear.json), so the game reads it unchanged.
"""
import argparse, json, pathlib, sys

import numpy as np
from PIL import Image
from scipy import ndimage

sys.path.insert(0, str(pathlib.Path(__file__).parent))
import gear_art as GA  # noqa: E402
from process import bleed, key_green  # noqa: E402
from rig_export import TEXELS_PER_UNIT  # noqa: E402

ROOT = GA.ROOT
PARTS = GA.PARTS
FILL = 14  # canvas px: gaps nearer the gear than this are filled with it
MARGIN = 28  # canvas px past the target's outline the gear may reach (flare, wide sleeves)


class Hero:
    def __init__(self, name):
        self.name = name
        self.parts, self.game = GA.rig_parts(name)
        self.cut = json.loads((ROOT / "art" / "rig" / name / "cut.json").read_text())
        self.mask = {n: self.parts[n][0][..., 3] > 0.5 for n in PARTS}
        self.waist = self.find_waist()
        self.k = TEXELS_PER_UNIT * self.game["units_per_px"]
        wp = ROOT / "art" / "rig" / name / "wear.json"
        self.wear = json.loads(wp.read_text()) if wp.exists() else {"anchors": {}, "pieces": {}}
        # The hero's own pieces before fitting adds any (what `keep` learns from).
        self.own = list(self.wear["pieces"])
        self._keep = {}

    def find_waist(self):
        if self.cut.get("waist_y"):
            return float(self.cut["waist_y"])
        try:
            rgb, a = key_green(Image.open(GA.hero_src(self.name)))
            w = GA.find_waist(self.name, rgb, a, self.parts["torso"][0][..., 3] > 0.02)
        except Exception:
            w = None
        return w or (self.cut["neck"][1] + self.cut["hem_y"]) / 2 + 40

    def landmarks(self):
        """Body points on the canvas, the same list for every hero: neck, waist, shoulders,
        elbows, hands, hips, ankles, soles, the torso's sides at the chest and the waist."""
        P = {n: self.parts[n] for n in PARTS}
        pts = [self.cut["neck"], [P["torso"][2][0], self.waist]]
        for side in ("far", "near"):
            pts.append(P[f"arm_{side}_upper"][2])
            pts.append(P[f"arm_{side}_fore"][2])
            m = self.mask[f"arm_{side}_fore"]
            ys, xs = np.where(m)
            o = P[f"arm_{side}_fore"][2]
            d = np.hypot(xs - o[0], ys - o[1])
            far = d >= np.percentile(d, 92)
            pts.append([xs[far].mean(), ys[far].mean()])
        for side in ("far", "near"):
            _, r, pv = P[f"leg_{side}"]
            m = self.mask[f"leg_{side}"]
            ys, xs = np.where(m)
            pts.append(pv)
            ay = r[1] + GA.ANKLE * r[3]
            row = xs[np.abs(ys - ay) < 3]
            pts.append([row.mean() if len(row) else xs.mean(), ay])
            pts.append([xs[ys > ys.max() - 6].mean(), ys.max()])
        m = self.mask["torso"]
        for y in (self.cut["neck"][1] + 0.35 * (self.waist - self.cut["neck"][1]), self.waist - 4):
            xs = np.where(m[int(y)])[0]
            if len(xs):
                pts += [[xs.min(), y], [xs.max(), y]]
            else:
                pts += [[P["torso"][2][0] - 60, y], [P["torso"][2][0] + 60, y]]
        return np.array(pts, float)

    def keep(self, slot, part):
        """This part's pixels that are not clothes for `slot`: what every one of the hero's
        own pieces for the slot (as they were before any fitting) left as it was: hair over the
        shoulders, a flame in the hand, a weapon, a scarf. A hero with no pieces of its own
        keeps nothing (design it with clear arms and shoulders)."""
        key = (slot, part)
        if key in self._keep:
            return self._keep[key]
        base = self.parts[part][0]
        same, n = np.zeros((1024, 1024), np.int32), 0
        for piece in self.own:
            if not piece.startswith(slot + "_") or not piece[-1].isdigit():
                continue
            got = self.patch_canvas(piece, part)
            if got is None:
                continue
            pc, st = got
            d = np.linalg.norm(pc[..., :3] - base[..., :3], axis=-1)
            same += (st == 0) | ((st == 2) & (d < 0.07) & (base[..., 3] > 0.5))
            n += 1
        if n == 0:
            self._keep[key] = np.zeros((1024, 1024), bool)
            return self._keep[key]
        k = (same >= 0.85 * n) & self.mask[part]
        # Specks are noise (a fold that happened to match); keep only solid areas.
        k = ndimage.binary_opening(k, iterations=2)
        lab, m = ndimage.label(k)
        if m:
            areas = ndimage.sum(np.ones_like(lab), lab, range(1, m + 1))
            k = np.isin(lab, [i + 1 for i, a in enumerate(areas) if a >= 150])
        k = ndimage.binary_dilation(k, iterations=1) & self.mask[part]
        self._keep[key] = k
        return k

    def patch_canvas(self, piece, part):
        """This hero's patch for `piece` on `part`, at canvas scale: (rgba float canvas, state)
        with state 0 outside, 1 cleared, 2 drawn."""
        r = self.wear["pieces"].get(piece, {}).get(part)
        if not r:
            return None
        im = Image.open(ROOT / "art" / "hd" / "wear" / self.name / f"{piece}.{part}.png").convert("RGBA")
        a8 = np.asarray(im)[..., 3]
        st = np.where(a8 >= 2, 2, np.where(a8 == 1, 1, 0)).astype(np.uint8)
        x, y, w, h = r
        rgba = np.asarray(im.resize((w, h), Image.BILINEAR)).astype(np.float32) / 255
        st = np.asarray(Image.fromarray(st).resize((w, h), Image.NEAREST))
        c = np.zeros((1024, 1024, 4), np.float32)
        s = np.zeros((1024, 1024), np.uint8)
        y1, x1 = min(1024, y + h), min(1024, x + w)
        c[y:y1, x:x1] = rgba[: y1 - y, : x1 - x]
        s[y:y1, x:x1] = st[: y1 - y, : x1 - x]
        c[..., 3] = np.where(s == 2, c[..., 3], 0)
        return c, s


def warp(src, dst, step=8):
    """Thin-plate spline from the target's canvas to the source's through their landmarks:
    (map y, map x) on the full 1024 grid."""
    from scipy.interpolate import RBFInterpolator
    d, s = dst.landmarks(), src.landmarks()
    # The canvas corners stay put, so the warp stays tame away from the body.
    c = np.array([[0, 0], [1023, 0], [0, 1023], [1023, 1023]], float)
    f = RBFInterpolator(np.vstack([d, c]), np.vstack([s, c]), kernel="thin_plate_spline", smoothing=2.0)
    g = np.arange(0, 1024 + step, step)
    gy, gx = np.meshgrid(g, g, indexing="ij")
    m = f(np.column_stack([gx.ravel(), gy.ravel()])).reshape(len(g), len(g), 2)
    full = np.arange(1024) / step
    yy, xx = np.meshgrid(full, full, indexing="ij")
    mx = ndimage.map_coordinates(m[..., 0], [yy, xx], order=1)
    my = ndimage.map_coordinates(m[..., 1], [yy, xx], order=1)
    return my, mx


def segment(h, part):
    """A limb's bone on the canvas: (root, end, mean half-width)."""
    o = np.array(h.parts[part][2], float)
    if part.endswith("_upper"):
        e = np.array(h.parts[part.replace("_upper", "_fore")][2], float)
    else:
        ys, xs = np.where(h.mask[part])
        d = np.hypot(xs - o[0], ys - o[1])
        far = d >= np.percentile(d, 92)
        e = np.array([xs[far].mean(), ys[far].mean()])
    L = np.linalg.norm(e - o)
    return o, e, h.mask[part].sum() / max(L, 1.0) / 2


def head_warp(src, dst):
    """Head to head by their outlines: the same width, the crown and the neck aligned (a hat
    sits on the hair, whatever its height)."""
    def box(h):
        ys, xs = np.where(h.mask["head"])
        return xs.min(), xs.max(), ys.min(), h.cut["neck"][1]
    dx0, dx1, dy0, dy1 = box(dst)
    sx0, sx1, sy0, sy1 = box(src)
    yy, xx = np.mgrid[0:1024, 0:1024].astype(np.float32)
    return sy0 + (yy - dy0) * (sy1 - sy0) / (dy1 - dy0), sx0 + (xx - dx0) * (sx1 - sx0) / (dx1 - dx0)


def limb_warp(src, dst, part):
    """A limb maps bone to bone: along it by length, across by its width (so a sleeve follows
    the arm whatever its angle)."""
    (do, de, dw), (so, se, sw) = segment(dst, part), segment(src, part)
    dax, sax = (de - do) / np.linalg.norm(de - do), (se - so) / np.linalg.norm(se - so)
    dL, sL = np.linalg.norm(de - do), np.linalg.norm(se - so)
    dp, sp = np.array([-dax[1], dax[0]]), np.array([-sax[1], sax[0]])
    yy, xx = np.mgrid[0:1024, 0:1024].astype(np.float32)
    t = ((xx - do[0]) * dax[0] + (yy - do[1]) * dax[1]) / dL
    n = ((xx - do[0]) * dp[0] + (yy - do[1]) * dp[1]) * (sw / max(dw, 1.0))
    return so[1] + sax[1] * t * sL + sp[1] * n, so[0] + sax[0] * t * sL + sp[0] * n


def fit_part(src, dst, piece, part, region, wp, slot):
    got = src.patch_canvas(piece, part)
    if got is None:
        return None
    sc, ss = got
    worn_on = slot in ("hat", "gloves")
    if worn_on:
        # Only the item itself, not the source hero's hair, face or skin it was drawn over:
        # what the edit changed from the source's own art (or added past its outline).
        base = src.parts[part][0]
        d = np.linalg.norm(sc[..., :3] - base[..., :3], axis=-1)
        item = (ss == 2) & ((base[..., 3] < 0.5) | (d > 0.16))
        item = ndimage.binary_opening(item, iterations=2)
        # The item is solid; the faint re-render elsewhere (specks on the face) is not.
        lab, n = ndimage.label(item)
        if n:
            areas = ndimage.sum(np.ones_like(lab), lab, range(1, n + 1))
            big = max(areas)
            keep = [i + 1 for i, a in enumerate(areas) if a >= max(120, 0.08 * big)]
            item = ndimage.binary_closing(np.isin(lab, keep), iterations=2)
        sc = sc.copy()
        sc[..., 3] *= ndimage.gaussian_filter(item.astype(np.float32), 0.7)
    if part == "head":
        my, mx = head_warp(src, dst)
    elif part.startswith("arm_"):
        my, mx = limb_warp(src, dst, part)
    else:
        my, mx = wp
    # Where the gear may land: the target's part, and a margin round it (flare, wide sleeves).
    near = ndimage.binary_dilation(dst.mask[part], iterations=MARGIN)
    want = region & near
    sel = np.where(want)
    coords = np.vstack([my[sel], mx[sel]])
    out = np.zeros((1024, 1024, 4), np.float32)
    pre = sc.copy()
    pre[..., :3] *= pre[..., 3:4]
    for ch in range(4):
        out[..., ch][sel] = ndimage.map_coordinates(pre[..., ch], coords, order=1, mode="constant")
    a = out[..., 3]
    out[..., :3] = np.where(a[..., None] > 1e-3, out[..., :3] / np.maximum(a[..., None], 1e-3), 0)
    drawn = a > 2.5 / 255
    if not drawn.any():
        return None
    if worn_on:
        # Laid over the hero: nothing cleared, nothing filled.
        return out, drawn, want & drawn
    # Inside the target's own outline the new clothes must cover everything (or its old
    # clothes show): gaps take the nearest gear colour, at the part's own opacity.
    _, (iy, ix) = ndimage.distance_transform_edt(~(a > 0.6), return_indices=True)
    # Only gaps close to the gear: anything further is not clothes (a flame in the hand, a
    # weapon) and stays the hero's own.
    gap = ndimage.distance_transform_edt(~(a > 0.6))
    hole = region & dst.mask[part] & (a < 0.6) & (gap < FILL)
    out[..., :3] = np.where((hole | ~drawn)[..., None], out[iy, ix, :3], out[..., :3])
    out[..., 3] = np.where(hole, np.maximum(a, dst.parts[part][0][..., 3]), a)
    drawn = out[..., 3] > 2.5 / 255
    inside = want & (drawn | (dst.mask[part] & (gap < FILL)))
    # What isn't clothes stays the hero's own (and the gear goes under it).
    inside &= ~dst.keep(slot, part)
    return out, drawn, inside


def fit(src_name, dst_name, only=None, out_dir=None):
    src, dst = Hero(src_name), Hero(dst_name)
    reg, _ = GA.regions(dst.parts, dst.waist, dst.cut["neck"][1])
    out_dir = pathlib.Path(out_dir) if out_dir else ROOT / "art" / "hd" / "wear" / dst_name
    out_dir.mkdir(parents=True, exist_ok=True)
    meta = dst.wear["pieces"]
    wp = warp(src, dst)
    for piece in sorted(src.wear["pieces"]):
        if only and piece not in only:
            continue
        slot = piece.split("_")[0]
        if slot not in GA.G.WORN:
            continue
        pieces = {}
        for part, region in reg[slot].items():
            got = fit_part(src, dst, piece, part, region, wp, slot)
            if got is None:
                continue
            img, drawn, inside = got
            ys, xs = np.where(inside)
            x0, y0, x1, y1 = xs.min(), ys.min(), xs.max() + 1, ys.max() + 1
            patch = img[y0:y1, x0:x1]
            im = Image.fromarray((patch * 255).clip(0, 255).astype(np.uint8), "RGBA")
            tw, th = max(1, round(im.width * dst.k)), max(1, round(im.height * dst.k))
            im = bleed(im.resize((tw, th), Image.LANCZOS))
            rs = np.asarray(Image.fromarray((inside[y0:y1, x0:x1] * 255).astype(np.uint8)).resize((tw, th), Image.BILINEAR)) > 127
            arr = np.asarray(im).copy()
            al = arr[..., 3].astype(np.int32)
            arr[..., 3] = np.where(rs, np.maximum(al, 1), 0).astype(np.uint8)
            GA.save_piece(arr, out_dir / f"{piece}.{part}.png")
            pieces[part] = [int(x0), int(y0), int(x1 - x0), int(y1 - y0)]
        meta[piece] = pieces
        print(f"{src_name} -> {dst_name}: {piece} ({', '.join(pieces)})")
    if not dst.wear.get("anchors"):
        dst.wear["anchors"] = GA.anchors(dst_name, dst.parts, dst.cut, dst.waist)
    (ROOT / "art" / "rig" / dst_name / "wear.json").write_text(json.dumps(dst.wear, indent=0, sort_keys=True))


def dressed(hero, pieces):
    """The hero in its rest pose (the design layout) wearing `pieces`, as an RGBA canvas."""
    h = Hero(hero)
    canvas = np.zeros((1024, 1024, 4), np.float32)
    for part in PARTS:
        c = h.parts[part][0].copy()
        for p in pieces:
            got = h.patch_canvas(p, part)
            if got is None:
                continue
            pc, st = got
            c = np.where((st == 2)[..., None], pc, c)
            c[..., 3] = np.where(st == 1, 0, c[..., 3])
        a = c[..., 3:4]
        canvas[..., :3] = c[..., :3] * a + canvas[..., :3] * (1 - a)
        canvas[..., 3:4] = a + canvas[..., 3:4] * (1 - a)
    return canvas


def preview(dst, pieces, out):
    im = dressed(dst, pieces)
    bg = np.ones_like(im[..., :3]) * np.array([0.25, 0.3, 0.25])
    rgb = im[..., :3] * im[..., 3:4] + bg * (1 - im[..., 3:4])
    Image.fromarray((rgb * 255).astype(np.uint8)).save(out)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd")
    ap.add_argument("args", nargs="*")
    ap.add_argument("--only", nargs="*")
    ap.add_argument("--out")
    a = ap.parse_args()
    if a.cmd == "hero":
        fit("toya", a.args[0], a.only, a.out)
    elif a.cmd == "fit":
        fit(a.args[0], a.args[1], a.only, a.out)
    elif a.cmd == "preview":
        preview(a.args[0], a.args[1:], a.out or f"/tmp/{a.args[0]}_dressed.png")
