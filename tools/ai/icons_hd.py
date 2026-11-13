"""Icons (art/hd/icons, gear, exotic, pickups) at 256x256.

    python3 tools/ai/icons_hd.py export          # re-cut every icon from its restyle raw at full size
    python3 -I tools/ai/icons_hd.py fix [id ...]  # redraw the icons in FIXES from a text brief
    python3 tools/ai/icons_hd.py cut              # re-cut the saved fix raws

The restyle pass (restyle.py) drew the icons on 512 px cells but saved them at 64 px; export cuts
them again at 256. Some cells came back wrong (the style reference, chibi Ren, drawn instead of
the item, or the wrong object): those are redrawn by `fix`, four per call on a key colour, from
a written description, with a sheet of the good icons (not Ren) as the style reference.
Raws: art/ai/icons/. Every icon is centred on a transparent 256 canvas filling FILL of it.
"""
import base64, csv, json, pathlib, sys, time
import numpy as np
from PIL import Image
from scipy import ndimage

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from restyle import key_alpha, bbox, RAW as RESTYLE  # noqa: E402
from process import bleed  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[2]
HD = ROOT / "art" / "hd"
OUT = ROOT / "art" / "ai" / "icons"
SPEND = ROOT / "art" / "ai" / "spend.csv"
S, FILL = 256, 0.9
MODEL = "google/gemini-3-pro-image"

# file: (key colour, description). Subjects the restyle got wrong.
FIXES = {
    "pickups/shrine_bell": ("green", "a round golden Japanese shrine bell (suzu) with a slit at the bottom, hanging from a short red-and-white braided cord"),
    "pickups/swift_geta": ("green", "a pair of wooden Japanese geta sandals with red thongs, side by side, with short white speed streaks behind them"),
    "gear/shoes": ("green", "a pair of dark navy cloth tabi boots with white rubber soles and a red ankle tie, side by side"),
    "icons/star_fall": ("green", "a golden five-pointed star falling diagonally down-left with a long bright orange-gold trail"),
    "icons/venom_fang": ("magenta", "a single large curved white serpent fang dripping thick bright green venom drops"),
    "icons/lantern_vigil": ("green", "a glowing round Japanese paper lantern (chochin) with black top and bottom bands, lit from inside with soft pale blue spirit light"),
    "icons/foxfire_orb": ("green", "a floating ball of blue-white foxfire flame (kitsunebi) with flickering wisps rising from it"),
    "icons/grave_cleaver": ("green", "a huge heavy black-iron cleaver blade with a pale glowing edge and a cloth-wrapped handle, held diagonally"),
    "icons/expanse": ("magenta", "a glowing teal ring with four short arrows pointing outward from it, up, down, left and right"),
    "icons/relic_w2": ("green", "a red cloth headband (hachimaki) tied in a knot with the two ends flowing, a small orange flame emblem on the front"),
    "icons/purification_wave": ("green", "a ring of pale golden holy light spreading outward like a wave, with a white zigzag paper streamer (shide) in the centre"),
    "icons/mukade_spines": ("green", "a dark red centipede (mukade) curled in an arch, many yellow legs, sharp bone spines along its back, no ground under it"),
    "icons/void_lance": ("green", "a long spear of swirling dark purple and black void energy pointing diagonally up-right, with a glowing violet tip"),
    "exotic/sovereign_finger": ("green", "a single severed finger: grey-purple, mummified, with a long dark curved fingernail, its base wrapped in a faded paper talisman strip, a faint dark purple aura around it"),
    "exotic/skeleton_hand": ("green", "a giant clawed skeletal hand made of old ivory bones, reaching upward with fingers spread, a faint dark purple aura"),
    "exotic/nine_seal": ("green", "an unrolled old paper scroll with nine small red circular seal stamps on it in a grid, a small orange fox flame burning at one corner"),
    "exotic/jeweled_branch": ("green", "a golden tree branch with silver leaves bearing glowing round jewels in pink, pale blue and white like fruit"),
    "exotic/white_haori": ("green", "a white Japanese haori half-coat laid flat and open, with a black inner lining and a white diamond pattern along the hem, no writing"),
    "exotic/thousand_counter": ("green", "a dark crimson iron ring set with seven small red gems around its band, one of them glowing bright"),
    "gear/bottom": ("green", "a pair of dark navy hakama trousers with crisp pleats and a white waist tie, laid flat"),
    "gear/omamori": ("green", "a small red silk omamori charm pouch with gold embroidery and a knotted purple cord at the top"),
    "gear/draw": ("green", "a hexagonal wooden omikuji fortune box with gold trim and a thin wooden fortune stick sticking out of the hole on top"),
}

PROMPT = (
    "A 2x2 sheet of four separate game icons for a polished modern 2D anime action game, one centred in each quarter of the image, "
    "each filling about 80% of its quarter, on a perfectly flat solid {bg} ({hex}) background everywhere. "
    "Top left: {0}. Top right: {1}. Bottom left: {2}. Bottom right: {3}. "
    "Draw them in exactly the art style of the reference sheet: bold clean dark navy outlines, glossy vinyl-figure shading with soft gradients "
    "and bright specular highlights, rich but tasteful colours, simple bold shapes that read at small size. "
    "Objects only: no people, no characters, no faces, no hands except where asked, no text, no letters, no numbers, no frames, no borders, no shadows on the background."
)


def save(im, path):
    """256-colour palette PNG with alpha (a sixth of the size, no visible loss on these icons)."""
    im.convert("RGBA").quantize(256, method=Image.Quantize.FASTOCTREE, dither=Image.Dither.FLOYDSTEINBERG).save(path, optimize=True)


def plan():
    return {f[:-4]: (b["id"], i, b["bg"]) for b in json.loads((RESTYLE / "plan.json").read_text()) if b["kind"] == "icons" for i, f in enumerate(b["files"])}


def place(rgb, alpha, old_box=None, old_size=64):
    """Cut the keyed subject and centre it on a 256 canvas (inside the old icon's box, x4, when given)."""
    lab, n = ndimage.label(alpha > 0.3)
    if n:
        areas = ndimage.sum(alpha > 0.3, lab, range(1, n + 1))
        keep = np.isin(lab, [i + 1 for i, ar in enumerate(areas) if ar >= 0.01 * areas.max()])
        alpha = alpha * ndimage.binary_dilation(keep, iterations=2)
    nb = bbox(alpha)
    piece = Image.fromarray(np.dstack([rgb, alpha * 255]).clip(0, 255).astype(np.uint8)[nb[1]:nb[3], nb[0]:nb[2]], "RGBA")
    if old_box:
        x0, y0, x1, y1 = (v * S / old_size for v in old_box)
    else:
        m = S * (1 - FILL) / 2
        x0, y0, x1, y1 = m, m, S - m, S - m
    k = min((x1 - x0) / piece.width, (y1 - y0) / piece.height)
    piece = piece.resize((max(1, round(piece.width * k)), max(1, round(piece.height * k))), Image.LANCZOS)
    c = Image.new("RGBA", (S, S))
    c.alpha_composite(piece, (round((x0 + x1 - piece.width) / 2), round((y0 + y1 - piece.height) / 2)))
    return bleed(c)


def cell(raw_path, i, bg, single):
    a = np.asarray(Image.open(raw_path).convert("RGB")).astype(np.float32)
    n = a.shape[0]
    if not single:
        h = n // 2
        a = a[(i // 2) * h:(i // 2 + 1) * h, (i % 2) * h:(i % 2 + 1) * h]
    return key_alpha(a, bg)


def export():
    import io, subprocess
    p = plan()
    for name, (bid, i, bg) in sorted(p.items()):
        if name in FIXES and (OUT / f"{name.replace('/', '_')}.png").exists():
            continue
        raw = RESTYLE / f"{bid}.png"
        single = len([1 for v in p.values() if v[0] == bid]) == 1
        rgb, alpha = cell(raw, i, bg, single)
        # The committed icon's box (64 px as first drawn, or 256) keeps each icon's size and place.
        r = subprocess.run(["git", "show", f"HEAD:art/hd/{name}.png"], cwd=ROOT, capture_output=True)
        old = Image.open(io.BytesIO(r.stdout)).convert("RGBA")
        ob = bbox(np.asarray(old)[..., 3].astype(np.float32) / 255)
        save(place(rgb, alpha, ob, old.width), HD / f"{name}.png")
    print("exported", len(p))


def style_sheet():
    good = ["icons/abundance", "icons/flame_pool", "icons/guard", "icons/magnetism", "icons/taiko", "icons/yinyang_orbs",
            "exotic/lantern_heart", "exotic/tengu_fan", "exotic/kusanagi", "gear/hat", "gear/top", "pickups/oni_sake"]
    sheet = Image.new("RGBA", (4 * S, 3 * S), (58, 60, 72, 255))
    for j, g in enumerate(good):
        sheet.alpha_composite(Image.open(HD / f"{g}.png").convert("RGBA"), ((j % 4) * S, (j // 4) * S))
    path = OUT / "style_sheet.png"
    sheet.convert("RGB").save(path)
    return path


def fix(ids):
    from gen import key
    import urllib.request
    OUT.mkdir(parents=True, exist_ok=True)
    names = [n for n in FIXES if not ids or n in ids or n.split("/")[1] in ids]
    groups = {}
    for n in names:
        groups.setdefault(FIXES[n][0], []).append(n)
    batches = [(bg, g[i:i + 4]) for bg, g in groups.items() for i in range(0, len(g), 4)]
    ref = base64.b64encode(style_sheet().read_bytes()).decode()
    k = key()

    def run(batch):
        bg, group = batch
        subjects = [FIXES[n][1] for n in group] + ["nothing (leave this quarter empty background)"] * (4 - len(group))
        text = PROMPT.format(*subjects, bg="pure " + bg, hex="#FF00FF" if bg == "magenta" else "#00FF00")
        body = {"model": MODEL, "modalities": ["image", "text"], "usage": {"include": True},
                "image_config": {"aspect_ratio": "1:1", "image_size": "2K"},
                "messages": [{"role": "user", "content": [{"type": "text", "text": text}, {"type": "image_url", "image_url": {"url": f"data:image/png;base64,{ref}"}}]}]}
        req = urllib.request.Request("https://openrouter.ai/api/v1/chat/completions", data=json.dumps(body).encode(),
                                     headers={"Authorization": f"Bearer {k}", "Content-Type": "application/json"})
        r = json.load(urllib.request.urlopen(req, timeout=300))
        cost = float(r.get("usage", {}).get("cost") or 0)
        with SPEND.open("a", newline="") as fh:
            csv.writer(fh).writerow([time.strftime("%Y-%m-%d %H:%M:%S"), "icons_fix_" + "+".join(n.split("/")[1] for n in group), MODEL, f"{cost:.6f}"])
        imgs = r["choices"][0]["message"].get("images") or []
        if not imgs:
            return f"no image for {group}"
        sheet = OUT / f"sheet_{'+'.join(n.split('/')[1] for n in group)}.png"
        sheet.write_bytes(base64.b64decode(imgs[0]["image_url"]["url"].split(",", 1)[1]))
        (sheet.with_suffix(".json")).write_text(json.dumps({"bg": bg, "files": group}))
        return f"{sheet.name} ${cost:.3f}"

    import concurrent.futures as cf
    with cf.ThreadPoolExecutor(5) as ex:
        for line in ex.map(run, batches):
            print(line)
    cut()


def cut():
    for js in sorted(OUT.glob("sheet_*.json")):
        meta = json.loads(js.read_text())
        for i, n in enumerate(meta["files"]):
            rgb, alpha = cell(js.with_suffix(".png"), i, meta["bg"], False)
            im = place(rgb, alpha)
            im.save(OUT / f"{n.replace('/', '_')}.png")
            save(im, HD / f"{n}.png")
    print("cut fixes")


if __name__ == "__main__":
    {"export": lambda: export(), "fix": lambda: fix(sys.argv[2:]), "cut": lambda: cut()}[sys.argv[1]]()
