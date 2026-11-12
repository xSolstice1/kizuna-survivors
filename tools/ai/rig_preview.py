"""Animate a cut-out rig (from rig_cut.py) to GIFs, to judge the motion before it goes in the engine.

    python3 -I tools/ai/rig_preview.py art/rig/ren [idle walk ...]

Heroes have no attack animation (see design/art-style.md); "throw" is kept as a motion test only.

Writes <dir>/preview_<anim>.gif (large) and art/ai/preview/<hero>_rig.gif (in-game sizes:
desktop 64 px and retina 128 px tall), at 50 fps.

How the motion is made (the engine will do the same, every frame):
  - Primary motion (legs, body bob, lean, squash) comes from keyed curves, so foot contacts
    land exactly.
  - Secondary motion follows it through damped springs, simulated at 240 Hz: the head lags the
    body, each arm joint drags and overshoots, the hair is a 5-link spring chain driven by the
    head's acceleration (outer locks lag the inner ones and ripple, so it moves as strands, not
    a sheet), and the skirt sways and flares on each bounce.
  - Loops are recorded after the springs have settled into a steady cycle, so they loop clean.
"""
import json, math, pathlib, sys
import cv2
import numpy as np
from PIL import Image

ROOT = pathlib.Path(__file__).resolve().parents[2]
FPS = 50          # GIF frame rate (20 ms frames)
SIM = 240         # spring steps per second
S = 0.5           # render scale (the 1024 source at half size is plenty for in-game sizes)


def load(d):
    rig = json.loads((d / "rig.json").read_text())
    parts, P = {}, {}
    for n in rig["order"]:
        im = Image.open(d / f"{n}.png").convert("RGBA")
        im = im.resize((max(1, round(im.width * S)), max(1, round(im.height * S))), Image.LANCZOS)
        a = np.asarray(im).astype(np.float32) / 255
        a[..., :3] *= a[..., 3:]          # premultiplied
        parts[n] = a
        p = rig["parts"][n]
        P[n] = {"offset": np.array(p["offset"]) * S, "pivot": np.array(p["pivot"], float) * S}
    W, H = (np.array(rig["canvas"]) * S).astype(int)
    return rig["order"], parts, P, (W, H)


# --- springs ---------------------------------------------------------------------------------

class Spring:
    """x'' = w^2 (target - x) - 2 zeta w x' + force."""
    def __init__(self, hz, zeta, x=0.0):
        self.w, self.z, self.x, self.v = 2 * math.pi * hz, zeta, x, 0.0

    def step(self, target, dt, force=0.0):
        a = self.w ** 2 * (target - self.x) - 2 * self.z * self.w * self.v + force
        self.v += a * dt
        self.x += self.v * dt
        return self.x


class Chain:
    """A hanging strand as N angle springs; each link chases its parent, so motion travels down."""
    def __init__(self, n, hz, zeta, falloff=0.85):
        self.links = [Spring(hz * falloff ** i, zeta) for i in range(n)]

    def step(self, rest, dt, force):
        prev = rest
        for i, l in enumerate(self.links):
            l.step(prev, dt, force * (1 + 0.35 * i))
            prev = l.x
        return [l.x for l in self.links]


# --- curves ----------------------------------------------------------------------------------

def soft(x, lim):
    """Soft limit: linear near 0, never past +-lim."""
    return lim * math.tanh(x / lim) if np.isscalar(x) else lim * np.tanh(np.asarray(x) / lim)


def smooth(t):
    t = min(1.0, max(0.0, t))
    return t * t * (3 - 2 * t)


def keys(u, ks):
    """Smoothly interpolated keyframes [(time, value), ...]."""
    if u <= ks[0][0]:
        return ks[0][1]
    for (t0, v0), (t1, v1) in zip(ks, ks[1:]):
        if u <= t1:
            return v0 + (v1 - v0) * smooth((u - t0) / (t1 - t0))
    return ks[-1][1]


def primary(anim, t):
    """Keyed motion at time t: body bob/lean/squash, leg angles and lift, arm targets, hair wind."""
    if anim == "idle":
        T = 3.0
        b = math.sin(2 * math.pi * t / T)
        return dict(bob=4 * b, lean=0.0, squash=0.006 * b, leg=(0, 0), lift=(0, 0),
                    upper=(3 + 2 * b, -3 - 2 * b), fore=(6 + 2 * b, 6 + 2 * b),
                    wind=0.012 * math.sin(2 * math.pi * t / T * 2) + 0.004)
    if anim == "walk":
        T = 0.8
        p = 2 * math.pi * t / T
        s, c = math.sin(p), math.cos(p)
        contact = s * s                      # 1 when the legs are fully apart (feet strike)
        return dict(bob=16 * contact - 6,    # low on contact, high while passing
                    lean=-4.0, squash=0.03 * contact ** 3,
                    leg=(26 * s, -26 * s),
                    lift=(0.16 * max(0, c) ** 1.5, 0.16 * max(0, -c) ** 1.5),   # the swinging foot rises
                    upper=(-22 * s, 20 * s), fore=(14 + 12 * max(0, -s), 14 + 12 * max(0, s)),
                    wind=-0.07)
    if anim == "throw":
        u = t / 0.75                         # 0..1 over the throw, then idle
        return dict(bob=keys(u, [(0, 0), (0.25, 6), (0.45, -4), (0.7, 3), (1, 0)]),
                    lean=keys(u, [(0, 0), (0.28, 4), (0.42, -7), (0.7, -2), (1, 0)]),
                    squash=keys(u, [(0, 0), (0.25, 0.03), (0.42, -0.02), (0.6, 0.01), (1, 0)]),
                    leg=(keys(u, [(0, 0), (0.3, 6), (0.45, -10), (1, 0)]), keys(u, [(0, 0), (0.3, -4), (0.45, 8), (1, 0)])),
                    lift=(0, 0),
                    upper=(keys(u, [(0, 3), (0.28, -40), (0.42, 72), (0.6, 55), (1, 3)]), keys(u, [(0, -3), (0.3, 12), (0.45, -20), (1, -3)])),
                    fore=(keys(u, [(0, 6), (0.28, -55), (0.4, 5), (0.6, 22), (1, 6)]), keys(u, [(0, 6), (0.45, 25), (1, 6)])),
                    wind=keys(u, [(0, 0.004), (0.42, -0.06), (1, 0.004)]),
                    release=u >= 0.42)
    raise ValueError(anim)


# --- rig state and rendering -----------------------------------------------------------------

class Rig:
    def __init__(self, P):
        self.P = P
        self.head = Spring(4.5, 0.45)               # head tilt lags the body
        self.head_dy = Spring(5.0, 0.4)             # and bobs a beat late
        self.upper = [Spring(3.2, 0.55), Spring(3.2, 0.55)]
        self.fore = [Spring(4.0, 0.5), Spring(4.0, 0.5)]
        self.hair = Chain(5, 2.2, 0.32)
        self.hair_spread = Spring(2.6, 0.3)
        self.skirt = Chain(2, 4.0, 0.35)
        self.skirt_flare = Spring(4.5, 0.3)
        self.hist = []                              # hair chain history, for per-strand lag
        self.prev = None

    def step(self, q, dt):
        # body motion of the head pivot, for the inertial forces
        hp = self.P["head"]["pivot"]
        feet = self.P["torso"]["pivot"] * np.array([1, 0]) + np.array([0, 990 * S])
        ang = math.radians(q["lean"])
        pos = np.array([feet[0] + (hp[1] - feet[1]) * -math.sin(ang), q["bob"] * S])
        if self.prev is None:
            self.prev = [pos, np.zeros(2)]
        vel = (pos - self.prev[0]) / dt
        acc = (vel - self.prev[1]) / dt
        self.prev = [pos, vel]
        ax, ay = acc / (1000 * S)                   # in body-heights / s^2
        self.head.step(q["lean"] * 0.7, dt, force=-ax * 250)
        self.head_dy.step(0, dt, force=-ay * 1000 * S)
        for i in range(2):
            self.upper[i].step(q["upper"][i], dt, force=-ax * 600)
            self.fore[i].step(q["fore"][i], dt, force=-ax * 450)
        self.hair.step(q["wind"], dt, force=-ax * 2.5)
        self.hair_spread.step(0, dt, force=ay * 3.0)
        self.skirt.step(q["wind"] * 0.6 - math.radians(q["leg"][0] - q["leg"][1]) * 0.05, dt, force=-ax * 1.5)
        self.skirt_flare.step(0, dt, force=ay * 2.5)
        self.hist.append([l.x for l in self.hair.links])
        if len(self.hist) > SIM:
            self.hist.pop(0)

    def hair_field(self, shape, top):
        """Per-pixel sideways offset for the hair layer: chain shape down its length, with outer
        columns sampling the chain a little in the past and a fine ripple, so locks separate."""
        h, w = shape
        gy, gx = 48, 24
        depth = np.clip((np.linspace(0, h, gy) - top) / (h - top), 0, 1)
        u = np.linspace(-1, 1, gx)
        L = (h - top) * 0.9 / len(self.hair.links)
        field = np.zeros((gy, gx), np.float32)
        hist = np.array(self.hist)
        for j, uu in enumerate(u):
            lag = int(abs(uu) * 0.09 * SIM)                     # outer locks trail by up to 90 ms
            th = hist[max(0, len(hist) - 1 - lag)]
            gain = 0.8 + 0.45 * abs(uu)
            seg = depth * len(th)
            disp = np.zeros(gy)
            for i, a in enumerate(th):
                disp += L * soft(a, 0.22) * np.clip(seg - i, 0, 1)
            field[:, j] = gain * disp + soft(self.hair_spread.x, 0.5) * uu * depth ** 1.5 * h * 0.08
        t = len(self.hist) / SIM
        ripple = np.sin(2 * math.pi * 1.7 * t - depth[:, None] * 7 + u[None, :] * 6) * depth[:, None] ** 2 * h * 0.006
        field += ripple * (0.4 + 6 * abs(np.mean([l.v for l in self.hair.links])))
        return cv2.resize(field, (w, h), interpolation=cv2.INTER_CUBIC)

    def skirt_field(self, shape, top):
        h, w = shape
        ys, xs = np.mgrid[0:h, 0:w].astype(np.float32)
        d = np.clip((ys - top) / (h - top), 0, 1) ** 1.3
        sway = soft(sum(l.x for l in self.skirt.links), 0.15) * (h - top) * 0.6
        flare = soft(self.skirt_flare.x, 0.4) * (xs / w - 0.5) * 2 * (h - top) * 0.25
        return d * (sway + flare)


def rot(pivot, deg, shift=(0, 0), scale=(1, 1)):
    c, s = math.cos(math.radians(deg)), math.sin(math.radians(deg))
    R = np.array([[c, s], [-s, c]]) @ np.diag(scale)      # cv2 convention: + = counter-clockwise
    m = np.zeros((2, 3))
    m[:, :2] = R
    m[:, 2] = pivot - R @ pivot + np.asarray(shift, float)
    return m


def compose(*ms):
    out = np.eye(3)
    for m in ms:
        out = out @ np.vstack([m, [0, 0, 1]])
    return out[:2]


def place(canvas, layer, offset, m, field=None):
    H, W = canvas.shape[:2]
    if field is not None:
        h, w = layer.shape[:2]
        field = field.astype(np.float32)
        pad = int(np.abs(field).max()) + 2
        layer = cv2.copyMakeBorder(layer, 0, 0, pad, pad, cv2.BORDER_CONSTANT, value=0)
        fieldp = cv2.copyMakeBorder(field, 0, 0, pad, pad, cv2.BORDER_REPLICATE)
        ys, xs = np.mgrid[0:h, 0:w + 2 * pad].astype(np.float32)
        layer = cv2.remap(layer, xs - fieldp, ys, cv2.INTER_LINEAR, borderMode=cv2.BORDER_CONSTANT, borderValue=0)
        offset = offset - np.array([pad, 0])
    full = compose(m, np.array([[1, 0, offset[0]], [0, 1, offset[1]]], float))
    out = cv2.warpAffine(layer, full, (W, H), flags=cv2.INTER_LINEAR, borderValue=0)
    canvas *= (1 - out[..., 3:])
    canvas += out


def talisman():
    w, h = 26, 44
    im = np.zeros((h, w, 4), np.float32)
    cv2.rectangle(im, (1, 1), (w - 2, h - 2), (0.12, 0.12, 0.2, 1), -1)
    cv2.rectangle(im, (4, 4), (w - 5, h - 5), (0.98, 0.96, 0.9, 1), -1)
    cv2.line(im, (w // 2, 9), (w // 2, h - 10), (0.8, 0.1, 0.15, 1), 3)
    cv2.line(im, (8, 16), (w - 9, 16), (0.8, 0.1, 0.15, 1), 2)
    im[..., :3] *= im[..., 3:]
    return im


def render(rig, parts, P, size, q):
    W, H = size
    canvas = np.zeros((H, W, 4), np.float32)
    feet = np.array([P["torso"]["pivot"][0], 990 * S])
    sq = q["squash"]
    body = compose(rot(feet, q["lean"], (0, q["bob"] * S), (1 + sq * 0.6, 1 - sq)))
    def child(name, deg, parent=body, extra=(0, 0)):
        return compose(parent, rot(P[name]["pivot"], deg, extra))
    head = child("head", rig.head.x, extra=(0, rig.head_dy.x))
    uppers = [child("arm_near_upper", rig.upper[0].x), child("arm_far_upper", rig.upper[1].x)]
    fores = [compose(uppers[0], rot(P["arm_near_fore"]["pivot"], rig.fore[0].x)),
             compose(uppers[1], rot(P["arm_far_fore"]["pivot"], rig.fore[1].x))]
    def leg(name, deg, lift):
        return compose(rot(P[name]["pivot"], deg, (0, q["bob"] * S * 0.35), (1, 1 - lift)))
    hair = parts["hair_back"]
    torso = parts["torso"]
    layers = [
        ("hair_back", head, rig.hair_field(hair.shape[:2], hair.shape[0] * 0.3)),
        ("arm_far_upper", uppers[1], None),
        ("arm_far_fore", fores[1], None),
        ("leg_far", leg("leg_far", q["leg"][1], q["lift"][1]), None),
        ("leg_near", leg("leg_near", q["leg"][0], q["lift"][0]), None),
        ("torso", body, rig.skirt_field(torso.shape[:2], torso.shape[0] * 0.6)),
        ("head", head, None),
        ("arm_near_upper", uppers[0], None),
        ("arm_near_fore", fores[0], None),
    ]
    for name, m, field in layers:
        place(canvas, parts[name], P[name]["offset"], m, field)
    if "release" in q and not q["release"]:
        hand = fores[0] @ np.array([384 * S, 770 * S, 1])
        tal = talisman()
        place(canvas, tal, hand - np.array([13, 30]), rot(hand, rig.fore[0].x + rig.upper[0].x))
    rgb = np.where(canvas[..., 3:] > 1e-4, canvas[..., :3] / np.maximum(canvas[..., 3:], 1e-4), 0)
    return Image.fromarray((np.dstack([rgb, canvas[..., 3:]]) * 255).clip(0, 255).astype(np.uint8), "RGBA")


def run(anim, P, parts, size):
    """Simulate; return the recorded frames (one clean loop, or the throw)."""
    rig = Rig(P)
    dt = 1 / SIM
    if anim == "throw":
        warm, length, src = 2.0, 1.2, "throw"
    else:
        T = 3.0 if anim == "idle" else 0.8
        warm, length, src = T * 4, T, anim
    frames = []
    n = int((warm + length) * SIM)
    every = SIM // FPS if SIM % FPS == 0 else SIM / FPS
    next_frame = warm * SIM
    for i in range(n):
        t = i / SIM
        if anim == "throw":
            q = primary("idle", t) if t < warm else primary("throw", t - warm)
        else:
            q = primary(src, t)
        rig.step(q, dt)
        if i >= next_frame - 1e-6:
            frames.append(render(rig, parts, P, size, q))
            next_frame += every
    return frames


def on_bg(im, height, bg=(44, 52, 48)):
    w, h = im.size
    im = im.crop((int(w * 0.08), 0, int(w * 0.92), h))
    im = im.resize((round(im.width * height / im.height), height), Image.LANCZOS)
    b = Image.new("RGBA", im.size, bg + (255,))
    b.alpha_composite(im)
    return b.convert("RGB")


def main():
    d = pathlib.Path(sys.argv[1])
    order, parts, P, size = load(d)
    anims = {a: run(a, P, parts, size) for a in (sys.argv[2:] or ["idle", "walk"])}
    for a, fs in anims.items():
        big = [on_bg(f, 384) for f in fs]
        big[0].save(d / f"preview_{a}.gif", save_all=True, append_images=big[1:], duration=1000 // FPS, loop=0)
        strip = [on_bg(f, 200) for f in fs[:: max(1, len(fs) // 12)]]
        s = Image.new("RGB", (sum(x.width for x in strip), 200))
        for i, x in enumerate(strip):
            s.paste(x, (i * strip[0].width, 0))
        s.save(d / f"strip_{a}.png")
    # in-game sizes, all animations side by side (hero = 64 px desktop, 128 px retina)
    L = max(len(f) for f in anims.values())
    sheet = []
    for i in range(L):
        row = []
        for fs in anims.values():
            f = fs[i % len(fs)]
            row += [on_bg(f, 64 * 1024 // 1000), on_bg(f, 128 * 1024 // 1000)]
        c = Image.new("RGB", (sum(r.width for r in row) + 12 * len(row), max(r.height for r in row)), (20, 20, 20))
        x = 0
        for r in row:
            c.paste(r, (x, c.height - r.height))
            x += r.width + 12
        sheet.append(c)
    out = ROOT / "art" / "ai" / "preview" / f"{d.name}_rig.gif"
    sheet[0].save(out, save_all=True, append_images=sheet[1:], duration=1000 // FPS, loop=0)
    print("wrote", out.relative_to(ROOT), {a: len(f) for a, f in anims.items()})


if __name__ == "__main__":
    main()
