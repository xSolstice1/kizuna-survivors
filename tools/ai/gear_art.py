"""Gear art: an icon for every base at each tier, and worn outfit pieces for every rigged hero.

    python3 -I tools/ai/gear_art.py icons [--only hat]   # generate + cut icons (4 per pro call)
    python3 -I tools/ai/gear_art.py icons-cut             # re-cut saved icon sheets
    python3 -I tools/ai/gear_art.py wear ren [--tier 0] [--only a0]   # outfit edits (nano)
    python3 -I tools/ai/gear_art.py slice ren             # cut the edits into worn pieces
    python3 tools/ai/gear_art.py hero <name>              # a new rigged hero: wear + slice (~$1.20)
                                                          # (or free: tools/ai/gear_fit.py hero <name>)

Icons: art/hd/gear/<slot>/<base>_<tier>.png (256 palette PNGs), descriptions in gear_data.py.

Worn pieces: an outfit edit is the hero's design image ("the same character, now wearing X")
from the image model, which keeps the pose aligned to within a pixel or two. Each edit changes
one group of slots (gear_data.GROUPS); slicing it gives, per slot, a replacement for the parts
of the rig that slot covers:
  hat     the head (its hat may reach past the head's old outline)
  top     the torso above the waist, both upper arms, both forearms except the hands
  gloves  the hands (the far end of each forearm)
  bottom  the torso below the waist, the legs above the ankles
  shoes   the legs below the ankles
A piece is a patch over the part, in the part's texture scale: inside the slot's region the
part's pixels are replaced by the patch's (including clearing them, where the new outfit is
slimmer); outside it, the part is untouched. Patch alpha 0 = outside the region, 1 = inside
but clear (the RGB is bled colour), 2..255 = the pixel. Written to art/hd/wear/<hero>/ with
the patch rects (source pixels) in art/rig/<hero>/wear.json.
"""
import argparse, base64, csv, json, pathlib, sys, time, urllib.request
import concurrent.futures as cf
import numpy as np
from PIL import Image
from scipy import ndimage

sys.path.insert(0, str(pathlib.Path(__file__).parent))
import gear_data as G  # noqa: E402
from icons_hd import PROMPT, cell, place, save, style_sheet  # noqa: E402
from process import key_green, bleed  # noqa: E402
from rig_export import TEXELS_PER_UNIT  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[2]
HD = ROOT / "art" / "hd"
SPEND = ROOT / "art" / "ai" / "spend.csv"
ICON_RAW = ROOT / "art" / "ai" / "gear_icons"
WEAR_RAW = ROOT / "art" / "ai" / "wear"
PRO = "google/gemini-3-pro-image"
NANO = "google/gemini-nano-banana-2.1"


def api_key():
    for line in (ROOT / ".env").read_text().splitlines():
        if line.startswith("OPENROUTER_API_KEY="):
            return line.split("=", 1)[1].strip()
    raise SystemExit("OPENROUTER_API_KEY missing from .env")


def call(model, text, refs, name, size=None):
    content = [{"type": "text", "text": text}]
    for r in refs:
        content.append({"type": "image_url", "image_url": {"url": "data:image/png;base64," + base64.b64encode(pathlib.Path(r).read_bytes()).decode()}})
    cfg = {"aspect_ratio": "1:1"}
    if size:
        cfg["image_size"] = size
    body = {"model": model, "modalities": ["image", "text"], "usage": {"include": True}, "image_config": cfg,
            "messages": [{"role": "user", "content": content}]}
    req = urllib.request.Request("https://openrouter.ai/api/v1/chat/completions", data=json.dumps(body).encode(),
                                 headers={"Authorization": f"Bearer {api_key()}", "Content-Type": "application/json"})
    for attempt in range(3):
        try:
            r = json.load(urllib.request.urlopen(req, timeout=300))
            break
        except Exception as e:  # network hiccup: try again
            if attempt == 2:
                return None, f"{name}: {e}"
            time.sleep(5)
    cost = float(r.get("usage", {}).get("cost") or 0)
    with SPEND.open("a", newline="") as fh:
        csv.writer(fh).writerow([time.strftime("%Y-%m-%d %H:%M:%S"), name, model, f"{cost:.6f}"])
    imgs = (r.get("choices") or [{}])[0].get("message", {}).get("images") or []
    if not imgs:
        return None, f"{name}: no image (${cost:.3f})"
    return base64.b64decode(imgs[0]["image_url"]["url"].split(",", 1)[1]), f"{name} ${cost:.3f}"


# ───────────── Icons ─────────────

def icon_name(slot, bkey, t):
    return f"{slot}/{bkey}_{t}"


def icons(only):
    ICON_RAW.mkdir(parents=True, exist_ok=True)
    todo = [it for it in G.items() if (not only or it[0] in only) and not (HD / "gear" / f"{icon_name(it[0], it[2], it[3])}.png").exists()]
    # The three tiers of a base share a sheet where they can, so they come out as one design.
    groups = {}
    for it in todo:
        groups.setdefault(it[4], []).append(it)
    batches = [(bg, g[i:i + 4]) for bg, g in groups.items() for i in range(0, len(g), 4)]
    print(f"{len(todo)} icons in {len(batches)} calls (~${0.14 * len(batches):.2f})")
    ref = style_sheet()

    def run(batch):
        bg, group = batch
        subjects = [f"{d} (a {slot} item for a game, shown on its own)" for slot, _, _, _, _, d in group]
        subjects += ["nothing (leave this quarter empty background)"] * (4 - len(group))
        text = PROMPT.format(*subjects, bg="pure " + bg, hex="#FF00FF" if bg == "magenta" else "#00FF00")
        text += " Items that share a name are the same design at different quality levels, so keep their shape and main colour the same and change only the materials and decoration."
        names = [icon_name(s, k, t) for s, _, k, t, _, _ in group]
        tag = "+".join(n.replace("/", "-") for n in names)
        data, msg = call(PRO, text, [ref], "gear_icons_" + tag, "2K")
        if data:
            (ICON_RAW / f"{tag}.png").write_bytes(data)
            (ICON_RAW / f"{tag}.json").write_text(json.dumps({"bg": bg, "files": names}))
        return msg

    with cf.ThreadPoolExecutor(6) as ex:
        for line in ex.map(run, batches):
            print(line)
    icons_cut()


def icons_cut():
    n = 0
    for js in sorted(ICON_RAW.glob("*.json")):
        meta = json.loads(js.read_text())
        for i, name in enumerate(meta["files"]):
            rgb, alpha = cell(js.with_suffix(".png"), i, meta["bg"], False)
            # The model sometimes draws the sheet's dividing lines along the cell's edges: clear
            # a thin band there (the icons sit well inside it).
            b = alpha.shape[0] // 20
            alpha = alpha.copy()
            alpha[:b] = alpha[-b:] = 0
            alpha[:, :b] = alpha[:, -b:] = 0
            out = HD / "gear" / f"{name}.png"
            out.parent.mkdir(parents=True, exist_ok=True)
            save(place(rgb, alpha), out)
            n += 1
    print("cut", n, "icons")


# ───────────── Outfit edits ─────────────

def hero_src(hero):
    cut = json.loads((ROOT / "art" / "rig" / hero / "cut.json").read_text())
    return ROOT / cut["sources"]["full"]


def edit_items(group, idx, t):
    """[(slot, piece name, description, reference icon)] that an edit puts on."""
    if group == "x":
        return [(s, f"{s}_{k}", d, HD / "exotic" / f"{k}.png") for s, (k, d) in G.EXOTIC_WORN.items()]
    out = []
    for slot in G.GROUPS[group]:
        if idx < len(G.BASES[slot]):
            name, _, desc = G.BASES[slot][idx]
            k = G.key(name)
            out.append((slot, f"{slot}_{k}_{t}", f"{desc}, {G.TIERS[t]}", HD / "gear" / slot / f"{k}_{t}.png"))
    return out


def edit_prompt(group, idx, t):
    lines = []
    for slot, _, desc, _ in edit_items(group, idx, t):
        where = {"hat": "on the head", "top": "top: replace the upper-body clothes and their sleeves with",
                 "gloves": "hands:", "bottom": "bottom: replace the skirt, shorts or trousers with",
                 "shoes": "feet: replace the shoes and socks with"}[slot]
        lines.append(f"- {where} {desc} (exactly the {slot} item in reference image {len(lines) + 2})")
    keep = {"a": "the skirt, shorts or trousers, the socks and the shoes", "b": "the hat, top and gloves", "x": "the hands, the socks and the shoes"}[group]
    return (
        "Edit the first image. The other images show the exact items to put on: copy each item's shape, colours, materials, patterns and decoration "
        "exactly as drawn there, fitted to the character's body. The first image is a game character sprite that will be cut into animation layers, so the pose must not change at all: "
        "the head, both arms, both hands, both legs and both feet stay exactly where they are, at exactly the same angle and size, "
        "with the two legs still clearly separate. Same face exactly, including any blindfold, eye patch or mask worn on the face; same hairstyle and hair colour, same body, same art style, same flat pure green background. "
        "Change ONLY these clothes:\n" + "\n".join(lines) + "\n"
        f"Everything else stays exactly as it is, including {keep}. Keep any bag, strap or accessory exactly where it is, worn over the new clothes. "
        "Hair still flows naturally out from under any hat. No writing or letters anywhere. Bold dark navy outline, glossy vinyl-figure shading."
    )


def icon_on_grey(path):
    """An icon flattened on neutral grey (a transparent PNG reads as black to the model)."""
    out = WEAR_RAW / "refs" / (path.parent.name + "_" + path.name)
    if not out.exists():
        out.parent.mkdir(parents=True, exist_ok=True)
        im = Image.open(path).convert("RGBA")
        bg = Image.new("RGBA", im.size, (128, 128, 128, 255))
        bg.alpha_composite(im)
        bg.convert("RGB").save(out)
    return out


def edits(hero, tiers, only):
    WEAR_RAW.mkdir(parents=True, exist_ok=True)
    src = hero_src(hero)
    jobs = []
    tags = [(g, idx, t) for t in tiers for g, slots in G.GROUPS.items() for idx in range(max(len(G.BASES[s]) for s in slots))]
    tags.append(("x", 0, 2))
    for g, idx, t in tags:
        tag = f"{g}{idx}_{t}"
        out = WEAR_RAW / f"{hero}_{tag}.png"
        if (only and tag not in only and f"{g}{idx}" not in only) or out.exists():
            continue
        refs = [r for *_, r in edit_items(g, idx, t)]
        jobs.append((tag, edit_prompt(g, idx, t), out, refs))
    print(f"{hero}: {len(jobs)} edits (~${0.039 * len(jobs):.2f})")

    def run(job):
        tag, text, out, refs = job
        data, msg = call(NANO, text, [src, *[icon_on_grey(r) for r in refs]], f"wear_{hero}_{tag}")
        if data:
            out.write_bytes(data)
        return msg

    with cf.ThreadPoolExecutor(6) as ex:
        for line in ex.map(run, jobs):
            print(line)


# ───────────── Slicing ─────────────

PARTS = ["hair_back", "arm_far_upper", "arm_far_fore", "leg_far", "leg_near", "torso", "head", "arm_near_upper", "arm_near_fore"]
ARMS_UP, ARMS_FORE, LEGS = ["arm_far_upper", "arm_near_upper"], ["arm_far_fore", "arm_near_fore"], ["leg_far", "leg_near"]
HAND = 0.45     # the far 45% of a forearm (from its pivot) is the hand
ANKLE = 0.55    # legs: below 55% of the leg's height is the foot


def rig_parts(hero):
    """Each part's full-res pixels placed on the 1024 canvas: {name: (rgba canvas, rect, pivot)}."""
    d = ROOT / "art" / "rig" / hero
    rig = json.loads((d / "rig.json").read_text())
    game = json.loads((d / "game.json").read_text())
    out = {}
    for n in PARTS:
        im = np.asarray(Image.open(d / f"{n}.png").convert("RGBA")).astype(np.float32) / 255
        x, y = rig["parts"][n]["offset"]
        c = np.zeros((1024, 1024, 4), np.float32)
        h, w = im.shape[:2]
        c[y:y + h, x:x + w] = im[: 1024 - y, : 1024 - x]
        out[n] = (c, game["parts"][n]["rect"], game["parts"][n]["pivot"])
    return out, game


def regions(parts, waist_y, neck_y):
    """Slot -> {part: bool mask on the canvas}: where each slot owns each part's pixels."""
    yy, xx = np.mgrid[0:1024, 0:1024]
    m = {n: parts[n][0][..., 3] > 0.02 for n in PARTS}

    def along(n, frac):  # pixels of part n further than `frac` of its length from its pivot
        _, r, p = parts[n]
        far = max(np.hypot(cx - p[0], cy - p[1]) for cx in (r[0], r[0] + r[2]) for cy in (r[1], r[1] + r[3]))
        return np.hypot(xx - p[0], yy - p[1]) > frac * far

    reg = {s: {} for s in G.WORN}
    reg["hat"]["head"] = np.ones((1024, 1024), bool)   # the head and anything the hat adds
    reg["hat"]["hair_back"] = yy < neck_y               # back hair the hat hides or presses down
    reg["top"]["torso"] = yy < waist_y
    reg["bottom"]["torso"] = yy >= waist_y
    for n in ARMS_UP:
        reg["top"][n] = np.ones((1024, 1024), bool)
    for n in ARMS_FORE:
        hand = along(n, HAND)
        reg["gloves"][n] = hand
        reg["top"][n] = ~hand
    for n in LEGS:
        _, r, _ = parts[n]
        foot = yy >= r[1] + ANKLE * r[3]
        reg["shoes"][n] = foot
        reg["bottom"][n] = ~foot
    return reg, m


def owner_map(parts, m, changed, neck_y):
    """Which part each pixel of the edit belongs to: the topmost part there in the original.
    Pixels the new outfit adds outside the body (or over the back hair) go to the head above the
    neck, and below it to the nearest body part, so a fuller sleeve or a hat brim is kept."""
    top = np.full((1024, 1024), -1)
    for i, n in enumerate(PARTS):  # back to front
        top[m[n]] = i
    body = top > 0
    dist, idx = ndimage.distance_transform_edt(~body, return_indices=True)
    nearest = top[idx[0], idx[1]]
    yy = np.mgrid[0:1024, 0:1024][0]
    own = top.copy()
    extra = (top <= 0) & changed
    # Above the neck: the large changed areas (a hat), not re-drawn strands of hair.
    lab, n = ndimage.label(extra & (yy < neck_y))
    if n:
        areas = ndimage.sum(np.ones_like(lab), lab, range(1, n + 1))
        own[np.isin(lab, [i + 1 for i, ar in enumerate(areas) if ar >= 3000])] = PARTS.index("head")
    # Below it: only close to the body (a fuller sleeve or hem), not hair beside it.
    low = extra & (yy >= neck_y) & (dist < 30)
    own[low] = nearest[low]
    # A sleeve wider than the arm it was cut from: what changed just around an arm, over the
    # torso, belongs to the arm, so it swings with it instead of staying painted on the chest.
    arms = [PARTS.index(n) for n in (*ARMS_UP, *ARMS_FORE)]
    arm = np.isin(top, arms)
    adist, aidx = ndimage.distance_transform_edt(~arm, return_indices=True)
    sleeve = changed & (own == PARTS.index("torso")) & (adist < 12) & (yy >= neck_y)
    own[sleeve] = top[aidx[0], aidx[1]][sleeve]
    return own


def anchors(hero, parts, cut, waist_y):
    """Where small gear hangs (design-image pixels), derived from the rig so a new hero needs no
    hand-picking: the collar below the neck point, the back of the face for the earring, the
    front of the hip at the waist, the near hand's knuckles, and the far shoulder (Arsenal, worn
    on the back). gear_data.ANCHORS overrides any of them."""
    nx, ny = cut["neck"]
    hr = parts["head"][1]
    tr, tp = parts["torso"][1], parts["torso"][2]
    m = parts["arm_near_fore"][0][..., 3] > 0.5
    ys, xs = np.where(m)
    low = ys >= np.percentile(ys, 85)              # hands hang down: the forearm's lowest part
    hand = [xs[low].mean(), ys[low].mean() - 0.1 * (ys.max() - ys.min())]
    shoulder = parts["arm_far_upper"][2]
    a = {
        "neck": [nx, ny + 45],
        "ear": [hr[0] + 0.3 * hr[2], ny - 0.14 * hr[3]],
        "hip": [tp[0] + 0.16 * tr[2], waist_y + 25],
        "hand": hand,
        "back": [shoulder[0] - 20, shoulder[1] - 10],
    }
    a.update(G.ANCHORS.get(hero, {}))
    return {k: [round(float(v)) for v in xy] for k, xy in a.items()}


def save_piece(arr, path):
    """A patch as a palette PNG (~40% smaller than RGBA) whose first two entries are exactly
    alpha 0 (outside the region) and 1 (cleared), so quantising can't turn one into the other."""
    a = arr[..., 3]
    draw = a >= 2
    idx = np.zeros(a.shape, np.uint8)
    idx[a == 1] = 1
    pal = [(0, 0, 0, 0), tuple(int(v) for v in arr[a == 1][:, :3].mean(0)) + (1,) if (a == 1).any() else (0, 0, 0, 1)]
    # Opaque pixels and the soft edge are quantised apart, so the edge keeps its alpha steps.
    for sel, n in ((a >= 250, 200), (draw & (a < 250), 54)):
        if not sel.any():
            continue
        strip = Image.fromarray(arr[sel].reshape(1, -1, 4), "RGBA").quantize(n, method=Image.Quantize.FASTOCTREE, dither=Image.Dither.NONE)
        q = np.asarray(strip).reshape(-1).astype(int)
        cols = np.asarray(strip.getpalette("RGBA")[: 4 * (q.max() + 1)], int).reshape(-1, 4)
        cols[:, 3] = np.maximum(cols[:, 3], 2)
        idx[sel] = (q + len(pal)).astype(np.uint8)
        pal += [tuple(int(v) for v in c) for c in cols]
    im = Image.fromarray(idx, "P")
    flat = [v for c in pal for v in c[:3]]
    im.putpalette(flat + [0] * (768 - len(flat)))
    im.info["transparency"] = bytes([c[3] for c in pal])
    im.save(path, optimize=True, transparency=bytes([c[3] for c in pal]))


def find_waist(hero, base_rgb, base_a, torso):
    """Where the hero's own top ends: the bottom + shoes edits keep the top, so the highest
    torso row they change is its hem (lower than the waist for a long jacket or robe). The top
    owns the torso above it, so a new top replaces the whole of the old one."""
    votes, n = np.zeros(1024), 0
    for raw in sorted(WEAR_RAW.glob(f"{hero}_b*.png")):
        rgb, a = key_green(Image.open(raw).convert("RGB").resize((1024, 1024), Image.LANCZOS))
        ch = ((np.linalg.norm(rgb - base_rgb, axis=-1) > 40) | (np.abs(a - base_a) > 0.3)) & torso
        ch = ndimage.binary_opening(ch, iterations=2)
        votes += ch.sum(1) > 30
        n += 1
    # Some edits (a hakama over a jacket) redraw higher than the rest: take the first row that
    # at least half of them change.
    rows = np.where(votes >= max(1, n / 2))[0]
    return float(rows.min()) if n and len(rows) else None


def slice_hero(hero):
    parts, game = rig_parts(hero)
    cut = json.loads((ROOT / "art" / "rig" / hero / "cut.json").read_text())
    base_rgb, base_a = key_green(Image.open(hero_src(hero)))
    m0 = {n: parts[n][0][..., 3] > 0.02 for n in PARTS}
    waist_y = cut.get("waist_y") or find_waist(hero, base_rgb, base_a, m0["torso"]) or (cut["neck"][1] + cut["hem_y"]) / 2 + 40
    print(hero, "waist at", round(waist_y))
    reg, m = regions(parts, waist_y, cut["neck"][1])
    k = TEXELS_PER_UNIT * game["units_per_px"]   # texels per source px, as the rig's own parts
    out_dir = HD / "wear" / hero
    out_dir.mkdir(parents=True, exist_ok=True)
    meta = {}
    for raw in sorted(WEAR_RAW.glob(f"{hero}_*.png")):
        tag = raw.stem[len(hero) + 1:]
        g, rest = tag[0], tag[1:]
        idx, t = (int(v) for v in rest.split("_"))
        rgb, a = key_green(Image.open(raw).convert("RGB").resize((1024, 1024), Image.LANCZOS))
        changed = (np.linalg.norm(rgb - base_rgb, axis=-1) > 40) | (np.abs(a - base_a) > 0.3)
        changed = ndimage.binary_opening(changed, iterations=1) & (a > 0.02)
        own = owner_map(parts, m, changed, cut["neck"][1])
        # Similar colours (grey shorts over a navy skirt) can fall under the threshold: close the
        # gaps and fill whatever the changed area encloses.
        soft = (np.linalg.norm(rgb - base_rgb, axis=-1) > 18) & (a > 0.02)
        # Pixels the edit removed (slimmer clothes, hair hidden by a hat) are changes too.
        # The old outline's soft anti-aliased rim counts too, or a faint ring of it stays.
        removed = ndimage.binary_opening((base_a > 0.5) & (a < 0.1), iterations=1)
        removed = ndimage.binary_dilation(removed, iterations=3) & (base_a > 0.02) & (a < 0.1)
        grown = ndimage.binary_fill_holes(ndimage.binary_closing(changed | removed | (soft & ndimage.binary_dilation(changed, iterations=6)), iterations=5))
        grown = ndimage.binary_dilation(grown, iterations=2)
        for slot, name, _, _ in edit_items(g, idx, t):
            pieces = {}
            for part, region in reg[slot].items():
                pi = PARTS.index(part)
                # Only what the edit changed: unchanged pixels (hair locks over the body, the face
                # under a hat) stay the rig's own, so pieces never leave a seam across them.
                inside = region & (own == pi) & grown & ((a > 0.02) | m[part])
                if slot == "hat" and part == "hair_back":
                    inside = region & grown & m[part]
                if not inside.any():
                    continue
                # Clothes: the part's hidden pixels (behind an arm, the body) get the new outfit
                # too, so a limb moving aside never shows the old clothes underneath.
                clothes = slot != "hat"
                if clothes:
                    inside |= region & m[part] & (own != pi)
                inside = ndimage.binary_dilation(inside, iterations=1) & region
                ys, xs = np.where(inside)
                x0, y0, x1, y1 = xs.min(), ys.min(), xs.max() + 1, ys.max() + 1
                patch = np.zeros((y1 - y0, x1 - x0, 4), np.float32)
                patch[..., :3] = rgb[y0:y1, x0:x1] / 255
                # Where this part was hidden under another part in the original, the edit shows
                # the part in front: keep the part's own pixels there.
                pc = parts[part][0][y0:y1, x0:x1]
                hidden = m[part][y0:y1, x0:x1] & (own[y0:y1, x0:x1] != pi)
                pa = a[y0:y1, x0:x1].copy()
                if clothes:
                    # Hidden pixels take the colour of the nearest pixel of this part in the
                    # edit, with the part's own shape.
                    src = (own[y0:y1, x0:x1] == pi) & (a[y0:y1, x0:x1] > 0.6) & inside[y0:y1, x0:x1]
                    if src.any():
                        _, (iy, ix) = ndimage.distance_transform_edt(~src, return_indices=True)
                        fill = patch[iy, ix, :3]
                        patch[..., :3] = np.where(hidden[..., None], fill, patch[..., :3])
                    else:
                        patch[..., :3] = np.where(hidden[..., None], pc[..., :3], patch[..., :3])
                else:
                    patch[..., :3] = np.where(hidden[..., None], pc[..., :3], patch[..., :3])
                pa = np.where(hidden, pc[..., 3], pa)
                if slot == "hat" and part == "hair_back":
                    # Back hair hidden behind the head: where the hat edit took hair away or
                    # covered it, clear it too, or it shows once the head's own hair is gone.
                    pa = np.where(hidden, 0.0, pa)
                patch[..., 3] = pa
                im = Image.fromarray((patch * 255).clip(0, 255).astype(np.uint8), "RGBA")
                tw, th = max(1, round(im.width * k)), max(1, round(im.height * k))
                im = bleed(im.resize((tw, th), Image.LANCZOS))
                reg_small = Image.fromarray((inside[y0:y1, x0:x1] * 255).astype(np.uint8)).resize((tw, th), Image.BILINEAR)
                arr = np.asarray(im).copy()
                rs = np.asarray(reg_small) > 127
                al = arr[..., 3].astype(np.int32)
                arr[..., 3] = np.where(rs, np.maximum(al, 1), 0).astype(np.uint8)
                save_piece(arr, out_dir / f"{name}.{part}.png")
                pieces[part] = [int(x0), int(y0), int(x1 - x0), int(y1 - y0)]
            meta[name] = pieces
    wear = {"anchors": anchors(hero, parts, cut, waist_y), "pieces": meta}
    (ROOT / "art" / "rig" / hero / "wear.json").write_text(json.dumps(wear, indent=0, sort_keys=True))
    print(hero, len(meta), "pieces")


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd")
    ap.add_argument("hero", nargs="?")
    ap.add_argument("--only", nargs="*")
    ap.add_argument("--tier", type=int, nargs="*", default=[0, 1, 2])
    a = ap.parse_args()
    if a.cmd == "icons":
        icons(a.only)
    elif a.cmd == "icons-cut":
        icons_cut()
    elif a.cmd == "wear":
        edits(a.hero, a.tier, a.only)
    elif a.cmd == "slice":
        slice_hero(a.hero)
    elif a.cmd == "hero":
        # Everything a newly rigged hero needs to wear gear: all outfit edits, then the slice.
        edits(a.hero, a.tier, a.only)
        slice_hero(a.hero)
