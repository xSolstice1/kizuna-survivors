"""Cut a hero into rig layers from aligned "same image, part removed" edits.

    python3 -I tools/ai/rig_cut.py art/rig/ren/cut.json

The image model keeps an edited image aligned with its source to well under a pixel, so a
part is simply where two edits differ:
  - hair_back = the hair-only edit (holes left by the body filled with the back-of-hair colour)
  - arms      = hairless body minus the armless edit, split into near/far at `arm_split_x`
  - legs      = hairless body minus the legless edit, split at `leg_split_x`
  - torso     = armless and legless together (the skirt hem from the legless edit)
  - head      = the torso above the neck line, so it can tilt on its own

cut.json names the source images and the cut points (1024px source coordinates). Writes
<dir>/<part>.png (cropped) and <dir>/rig.json: each part's offset in the source canvas, its
pivot, and the draw order.
"""
import json, pathlib, sys
import cv2
import numpy as np
from PIL import Image
from scipy import ndimage

from process import key_green, bleed

ROOT = pathlib.Path(__file__).resolve().parents[2]


def keyed(path):
    rgb, alpha = key_green(Image.open(ROOT / path))
    return rgb, alpha


def clean(mask, min_area=1500, grow=0):
    mask = ndimage.binary_opening(mask, iterations=2)
    mask = ndimage.binary_closing(mask, iterations=3)
    lab, n = ndimage.label(mask)
    if n:
        areas = ndimage.sum(mask, lab, range(1, n + 1))
        mask = np.isin(lab, [i + 1 for i, a in enumerate(areas) if a >= min_area])
    mask = ndimage.binary_fill_holes(mask)
    if grow:
        mask = ndimage.binary_dilation(mask, iterations=grow)
    return mask


def diff(a, b, thresh=40.0):
    return np.linalg.norm(a - b, axis=-1) > thresh


def hull(mask, extra=()):
    """The convex hull of a mask's pixels plus extra points (an arm is close to convex)."""
    pts = np.argwhere(mask)[:, ::-1].astype(np.int32)
    if len(extra):
        pts = np.vstack([pts, np.array(extra, np.int32)])
    out = np.zeros(mask.shape, np.uint8)
    if len(pts) >= 3:
        cv2.fillConvexPoly(out, cv2.convexHull(pts), 1)
    return out.astype(bool)


def row_fill(mask):
    """Fill every row between its leftmost and rightmost set pixel."""
    out = mask.copy()
    for y in np.nonzero(mask.any(1))[0]:
        xs = np.nonzero(mask[y])[0]
        out[y, xs[0]:xs[-1] + 1] = True
    return out


def part(rgb, alpha, mask, feather=1.5):
    a = alpha * ndimage.gaussian_filter(mask.astype(np.float32), feather) if feather else alpha * mask
    im = Image.fromarray(np.dstack([rgb, a * 255]).clip(0, 255).astype(np.uint8), "RGBA")
    box = im.getchannel("A").point(lambda v: 255 if v > 8 else 0).getbbox()
    return bleed(im.crop(box)), box


def main():
    cfg_path = pathlib.Path(sys.argv[1])
    cfg = json.loads(cfg_path.read_text())
    out = cfg_path.parent
    src = {k: keyed(v) for k, v in cfg["sources"].items()}
    H, W = src["full"][1].shape
    yy, xx = np.mgrid[0:H, 0:W]

    body_rgb, body_a = src["nohair"]
    solid = body_a > 0.5

    # Arms: where the armless edit changed the hairless body (covers the hands over the skirt too).
    # Navy sleeves over a navy torso barely differ, so each arm is the hull of what was found.
    found = clean(diff(body_rgb, src["noarms"][0], 25) & solid, min_area=300)
    near_x = xx < cfg["arm_split_x"]
    arms = (hull(found & near_x, cfg["arm_hull"]["near"]) | hull(found & ~near_x, cfg["arm_hull"]["far"])) & (body_a > 0.05)
    arms = ndimage.binary_dilation(arms, iterations=3) & (body_a > 0.05)
    # Legs: below the skirt, gone in the legless edit.
    legs = clean((src["nolegs"][1] < 0.5) & solid & (yy > cfg["hem_y"] - 40), grow=2) & (body_a > 0.05)

    # Torso: the armless image, minus legs; below the hem, the legless edit's redrawn skirt edge.
    t_rgb = src["noarms"][0].copy()
    hem = yy > cfg["hem_y"]
    t_rgb[hem] = src["nolegs"][0][hem]
    t_a = np.minimum(src["noarms"][1], np.where(hem, src["nolegs"][1], 1.0))

    # Head: torso pixels above the neck line; hair may hang a little lower.
    nx, ny = cfg["neck"]
    hairish = (t_rgb[..., 0] > 170) & (t_rgb[..., 0] > t_rgb[..., 2])   # pink / skin, not navy
    head = (t_a > 0.05) & ((yy < ny - 10) | ((yy < ny + 40) & hairish))
    head = clean(head, grow=1)
    # The torso keeps a neck band under the head, so tilting never opens a gap.
    torso = (t_a > 0.05) & ~(head & (yy < ny - 30))

    # Back hair: the hair-only edit, with the hole the body left filled by the darkest hair tone.
    h_rgb, h_a = src["hair"]
    hm = h_a > 0.5
    filled = row_fill(hm)
    hole = filled & ~hm
    if hole.any() and hm.any():
        pink = hm & (h_rgb[..., 0] > 150)
        back = np.median(h_rgb[pink], axis=0) * 0.72   # the hair's shadow side
        h_rgb = h_rgb.copy()
        h_rgb[hole] = back
        h_a = np.where(hole, 1.0, h_a)

    # Each arm splits at the elbow; the upper arm runs a little past it so a bent elbow
    # shows sleeve under the forearm, not a gap.
    near, far = arms & (xx < cfg["arm_split_x"]), arms & (xx >= cfg["arm_split_x"])
    ey_n, ey_f = cfg["pivots"]["arm_near_fore"][1], cfg["pivots"]["arm_far_fore"][1]
    parts = {
        "hair_back": (h_rgb, h_a, filled | hm),
        "arm_far_upper": (body_rgb, body_a, far & (yy < ey_f + 14)),
        "arm_far_fore": (body_rgb, body_a, far & (yy >= ey_f - 6)),
        "leg_far": (body_rgb, body_a, legs & (xx >= cfg["leg_split_x"])),
        "leg_near": (body_rgb, body_a, legs & (xx < cfg["leg_split_x"])),
        "torso": (t_rgb, t_a, torso),
        "head": (t_rgb, t_a, head),
        "arm_near_upper": (body_rgb, body_a, near & (yy < ey_n + 14)),
        "arm_near_fore": (body_rgb, body_a, near & (yy >= ey_n - 6)),
    }
    rig = {"canvas": [W, H], "order": list(parts), "parts": {}}
    for name, (rgb, a, m) in parts.items():
        im, box = part(rgb, a, m)
        im.save(out / f"{name}.png")
        rig["parts"][name] = {"offset": box[:2], "size": im.size, "pivot": cfg["pivots"][name]}
        print(name, box)
    (out / "rig.json").write_text(json.dumps(rig, indent=1))


if __name__ == "__main__":
    main()
