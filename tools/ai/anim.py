"""Turn a hero's generated animation clips into one consistent set of game frames.

    python3 -I tools/ai/anim.py <hero> --first <first_frame.png> [--height 53.5] [--density 6]

Clips: art/ai/raw/<hero>_{walk,idle,attack,wattack}_m720.mp4, all generated from the same first
frame (character on flat green). Because the first frame is shared, every clip uses ONE crop box
and ONE scale taken from it, so switching animations never jumps.

Stabilising: the video camera is static, so frames keep their own vertical position (no per-frame
foot snapping); only the slow sideways drift is removed (a straight-line fit of the torso centre).

Segments:
  walk    best seamless loop (silhouette match)        idle   best seamless loop
  attack  the throw: from leaving the rest pose back to it
  wattack the walking throw: around the arm's furthest reach, ending on a frame that matches walk

Writes art/hd/<hero>_anim_<set>_<k>.png and art/hd/<hero>_anim.txt
(`<set> <frames>` lines, `fps`, `walk_start` = walk frame closest to idle,
`wattack_resume` = walk frame to continue from after the walking throw).
"""
import argparse, pathlib, subprocess, sys, tempfile
import cv2
import numpy as np
from PIL import Image
from scipy import ndimage

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from process import key_green, bleed, ROOT

RAW = ROOT / "art" / "ai" / "raw"
OUT = ROOT / "art" / "hd"


def keyed(img):
    rgb, alpha = key_green(img)
    lab, n = ndimage.label(ndimage.binary_closing(alpha > 0.5, iterations=2))
    if n == 0:
        return np.zeros(alpha.shape, bool), rgb, alpha
    sizes = ndimage.sum(np.ones_like(alpha), lab, range(1, n + 1))
    keep = lab == int(np.argmax(sizes)) + 1
    m = ndimage.binary_dilation(keep, iterations=3)
    return keep, rgb, np.where(m, alpha, 0)


def clip_frames(path):
    with tempfile.TemporaryDirectory() as d:
        subprocess.run(["ffmpeg", "-v", "error", "-i", str(path), f"{d}/f_%04d.png"], check=True)
        return [keyed(Image.open(p)) for p in sorted(pathlib.Path(d).glob("f_*.png"))]


def gray(rgb, al):
    return (rgb.mean(-1) * al).astype(np.float32)


def register(ref_patch, rgb, al, rows, cols):
    """Sub-pixel (dx, dy) of the face/collar patch relative to the reference (phase correlation)."""
    patch = gray(rgb, al)[rows, cols]
    win = cv2.createHanningWindow(patch.shape[::-1], cv2.CV_32F)
    (dx, dy), _ = cv2.phaseCorrelate(ref_patch, patch, win)
    return dx, dy


def smooth(v, w, loop):
    """Moving average (circular for loops)."""
    if len(v) < 3:
        return v
    k = np.ones(w) / w
    if loop:
        pad = np.concatenate([v[-(w // 2):], v, v[: w // 2]])
        return np.convolve(pad, k, "valid")[: len(v)]
    pad = np.concatenate([np.full(w // 2, v[0]), v, np.full(w // 2, v[-1])])
    return np.convolve(pad, k, "valid")[: len(v)]


class Ref:
    """Geometry of the shared first frame."""
    def __init__(self, path):
        keep, _, _ = keyed(Image.open(path))
        ys, xs = np.nonzero(keep)
        self.top, self.bottom = ys.min(), ys.max()
        self.h = self.bottom - self.top
        self.t0, self.t1 = int(self.top + 0.25 * self.h), int(self.top + 0.55 * self.h)
        self.cx = self.torso_x(keep)
        # Face + collar patch: rigid enough to register on, away from swinging arms and hair ends.
        self.rows = slice(int(self.top + 0.06 * self.h), int(self.top + 0.36 * self.h))
        self.cols = slice(int(self.cx - 0.14 * self.h), int(self.cx + 0.20 * self.h))
        _, rgb, al = keyed(Image.open(path))
        self.patch = gray(rgb, al)[self.rows, self.cols]

    def torso_x(self, mask):
        band = mask[self.t0:self.t1]
        cols = band.sum(0)
        return float((cols * np.arange(mask.shape[1])).sum() / max(cols.sum(), 1))


def silhouette(mask, ref, shift):
    """Low-res mask in the shared box (after drift correction), for comparing poses."""
    box = (int(ref.cx - shift - 0.5 * ref.h), ref.top - 10, int(ref.cx - shift + 0.5 * ref.h), ref.bottom + 10)
    im = Image.fromarray((mask * 255).astype(np.uint8)).crop(box).resize((40, 80), Image.BILINEAR)
    return np.asarray(im, np.float32) / 255


def drift(frames, ref, idx):
    """Per-frame sideways shift (straight-line fit of the torso centre over `idx`)."""
    xs = np.array([ref.torso_x(frames[i][0]) for i in idx])
    if len(idx) < 2:
        return {i: x - ref.cx for i, x in zip(idx, xs)}
    a, b = np.polyfit(np.array(idx, float), xs, 1)
    return {i: (a * i + b) - ref.cx for i in idx}


def best_loop(frames, ref, lo, hi, skip):
    idx = list(range(len(frames)))
    sh = drift(frames, ref, idx)
    sil = [silhouette(frames[i][0], ref, sh[i]) for i in idx]
    best = None
    for i in range(skip, len(frames)):
        for j in range(i + lo, min(len(frames), i + hi + 1)):
            e = float(np.abs(sil[i] - sil[j]).mean())
            if best is None or e < best[0]:
                best = (e, i, j)
    return best


def reach(mask, ref, shift):
    """How far the silhouette reaches forward (right) of the torso at chest height."""
    band = mask[int(ref.top + 0.22 * ref.h):int(ref.top + 0.5 * ref.h)]
    cols = np.nonzero(band.any(0))[0]
    return (cols.max() - (ref.cx + shift)) / ref.h if len(cols) else 0.0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("hero")
    ap.add_argument("--first", required=True)
    ap.add_argument("--height", type=float, default=53.5, help="standing height in world units")
    ap.add_argument("--density", type=float, default=6.0, help="texels per world unit")
    a = ap.parse_args()
    ref = Ref(a.first)
    k = a.height * a.density / ref.h
    # Shared crop box in video pixels (feet at the bottom edge, room for a throwing arm).
    bw, bh = int(1.0 * ref.h), int(1.1 * ref.h)
    bottom = ref.bottom + 4
    W, H = round(bw * k), round(bh * k)
    W += W % 2

    clips = {n: clip_frames(RAW / f"{a.hero}_{n}_m720.mp4") for n in ("walk", "idle", "attack", "wattack")}
    sel, meta, release = {}, [], {}

    e, i, j = best_loop(clips["walk"], ref, 10, 30, 4)
    sel["walk"] = list(range(i, j)); print(f"walk loop {i}..{j} ({j - i}) err {e:.4f}")
    e, i, j = best_loop(clips["idle"], ref, 14, 44, 2)
    sel["idle"] = list(range(i, j)); print(f"idle loop {i}..{j} ({j - i}) err {e:.4f}")

    # Standing throw: wind-up and release, but the arm stays out for one frame only (a held
    # straight arm reads as a point or a salute), then the return.
    fr = clips["attack"]
    sh = drift(fr, ref, list(range(len(fr))))
    r = np.array([reach(fr[t][0], ref, sh[t]) for t in range(len(fr))])
    p = int(np.argmax(r[3:])) + 3
    thr = 0.7 * r[p]
    out = p
    while out > 0 and r[out - 1] > thr:
        out -= 1
    back = p
    while back < len(fr) - 1 and r[back + 1] > thr:
        back += 1
    s0 = max(0, out - 7)
    e0 = min(len(fr) - 1, back + 7)
    sel["attack"] = list(range(s0, out + 1)) + list(range(back + 1, e0 + 1))
    release["attack"] = out - s0
    print(f"attack {s0}..{out} + {back + 1}..{e0} (arm out {out}..{back}, dropped {back - out} held frames)")

    # Walking throw: around the furthest forward reach; resume the walk where it matches.
    fr = clips["wattack"]
    sh = drift(fr, ref, list(range(len(fr))))
    r = np.array([reach(fr[t][0], ref, sh[t]) for t in range(len(fr))])
    p = int(np.argmax(r[3:])) + 3
    s, e_ = max(0, p - 6), min(len(fr) - 1, p + 8)
    sel["wattack"] = list(range(s, e_ + 1)); print(f"wattack {s}..{e_} (reach peak {p})")
    # Release = the first frame with the arm well forward.
    release["wattack"] = next(t for t in range(s, p + 1) if r[t] > 0.7 * r[p]) - s

    # Transitions.
    wfr = clips["walk"]; wsh = drift(wfr, ref, sel["walk"])
    walk_sil = [silhouette(wfr[t][0], ref, wsh[t]) for t in sel["walk"]]
    ifr = clips["idle"]; ish = drift(ifr, ref, sel["idle"])
    idle0 = silhouette(ifr[sel["idle"][0]][0], ref, ish[sel["idle"][0]])
    walk_start = int(np.argmin([np.abs(w - idle0).mean() for w in walk_sil]))
    last = silhouette(fr[e_][0], ref, sh[e_])
    resume = int(np.argmin([np.abs(w - last).mean() for w in walk_sil]))

    for old in OUT.glob(f"{a.hero}_anim_*.png"):
        old.unlink()
    for name, idx in sel.items():
        fr = clips[name]
        loop = name in ("walk", "idle")
        reg = np.array([register(ref.patch, fr[t][1], fr[t][2], ref.rows, ref.cols) for t in idx])
        # Remove the frame-to-frame jitter (what a 5-frame moving average doesn't explain) on both
        # axes, plus the straight-line sideways drift; keep slow real motion (body bob, a lean).
        t_ = np.arange(len(idx), dtype=float)
        line = np.polyval(np.polyfit(t_, reg[:, 0], 1), t_) if len(idx) > 1 else reg[:, 0]
        ox = (reg[:, 0] - smooth(reg[:, 0], 5, loop)) + line
        oy = reg[:, 1] - smooth(reg[:, 1], 5, loop)
        for n, t in enumerate(idx):
            _, rgb, al = fr[t]
            im = Image.fromarray(np.dstack([rgb, al * 255]).clip(0, 255).astype(np.uint8), "RGBA")
            x0 = ref.cx - bw / 2 + ox[n]
            y0 = bottom - bh + oy[n]
            crop = im.transform((bw, bh), Image.AFFINE, (1, 0, x0, 0, 1, y0), resample=Image.BICUBIC)
            bleed(crop.resize((W, H), Image.LANCZOS)).save(OUT / f"{a.hero}_anim_{name}_{n}.png")
        print(f"  {name}: x shift {ox.min():+.1f}..{ox.max():+.1f}px, y jitter {oy.min():+.1f}..{oy.max():+.1f}px")
        meta.append(f"{name} {len(idx)}")
    meta += [f"attack_release {release['attack']}", f"wattack_release {release['wattack']}", "fps 16", f"walk_start {walk_start}", f"wattack_resume {resume}",
             f"size {W / a.density:.2f} {H / a.density:.2f}", f"density {a.density}"]
    (OUT / f"{a.hero}_anim.txt").write_text("\n".join(meta) + "\n")
    print("\n".join(meta))
    print(f"canvas {W}x{H} texels")


if __name__ == "__main__":
    main()
