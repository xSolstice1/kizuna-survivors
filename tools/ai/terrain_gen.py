"""Generate a world's terrain source textures at 2K through OpenRouter (Gemini 3 Pro Image).

    python3 -I tools/ai/terrain_gen.py w1 ground low high trail cliff

Prompts: art/prompts/terrain2/<world>.json ({"world": theme, "<kind>": what that surface is}).
Saves art/ai/raw/<world>_t2_<kind>_<k>.png and logs cost to art/ai/spend.csv. The textures are
made tileable afterwards (tools/ai/terrain_hd.py), so the prompt only asks for even, fine detail.
"""
import base64, concurrent.futures as cf, csv, json, pathlib, sys, time, urllib.request, urllib.error

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from gen import key  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[2]
RAW = ROOT / "art" / "ai" / "raw"
SPEND = ROOT / "art" / "ai" / "spend.csv"
MODEL = "google/gemini-3-pro-image"

GROUND = (
    "A top-down ground texture for a polished 2D anime action game, seen from DIRECTLY ABOVE (orthographic, no perspective, no horizon), "
    "covering a patch of ground about 4 x 4 metres and filling the whole square edge to edge. {world} This surface: {what} "
    "The detail is FINE and EVENLY spread over the whole image like a material swatch: many small elements of similar size, no single "
    "feature bigger than about a twentieth of the image, no paths, no lines or rows running across it, no focal point, no clusters, no empty areas. "
    "Even flat lighting with no light spot, no shadow gradient and no vignette: every part of the image equally bright. "
    "Art style: crisp hand-painted anime game background art, clean shapes with soft painted shading, sharp detail, "
    "LOW CONTRAST and slightly muted so characters stand out on it. No objects, no characters, no text, no border."
)
CLIFF = (
    "A wide texture strip for a polished 2D anime action game in 3/4 top-down view: the vertical FACE of a low cliff or wall "
    "seen straight from the front, running left to right, filling the whole image edge to edge. {world} This face: {what} "
    "The top edge of the image is the lip of the ground above; the face below is evenly detailed all the way along, with no "
    "single big feature, no gaps and no change of material from left to right, getting a little darker towards the bottom. "
    "Even lighting from the front. Art style: crisp hand-painted anime game background art, clean shapes with soft painted shading, "
    "sharp detail. No sky, no ground below the face, no characters, no text, no border."
)


def call(k, prompt, aspect):
    body = {"model": MODEL, "modalities": ["image", "text"], "messages": [{"role": "user", "content": [{"type": "text", "text": prompt}]}],
            "usage": {"include": True}, "image_config": {"aspect_ratio": aspect, "image_size": "2K"}}
    req = urllib.request.Request("https://openrouter.ai/api/v1/chat/completions", data=json.dumps(body).encode(),
                                 headers={"Authorization": f"Bearer {k}", "Content-Type": "application/json"})
    try:
        return json.load(urllib.request.urlopen(req, timeout=300))
    except urllib.error.HTTPError as e:
        return {"error": f"{e.code} {e.read().decode()[:300]}"}


def job(k, world, kind, spec):
    tpl = CLIFF if kind == "cliff" else GROUND
    prompt = tpl.format(world=spec["world"], what=spec[kind])
    r = call(k, prompt, "21:9" if kind == "cliff" else "1:1")
    if "error" in r:
        return f"{world} {kind}: error {r['error']}"
    cost = float(r.get("usage", {}).get("cost") or 0)
    with SPEND.open("a", newline="") as fh:
        csv.writer(fh).writerow([time.strftime("%Y-%m-%d %H:%M:%S"), f"{world}_t2_{kind}", MODEL, f"{cost:.6f}"])
    imgs = r["choices"][0]["message"].get("images") or []
    if not imgs:
        return f"{world} {kind}: no image (${cost:.3f})"
    n = len(list(RAW.glob(f"{world}_t2_{kind}_*.png")))
    out = RAW / f"{world}_t2_{kind}_{n}.png"
    out.write_bytes(base64.b64decode(imgs[0]["image_url"]["url"].split(",", 1)[1]))
    return f"saved {out.relative_to(ROOT)} ${cost:.3f}"


def main():
    world, kinds = sys.argv[1], sys.argv[2:]
    spec = json.loads((ROOT / "art" / "prompts" / "terrain2" / f"{world}.json").read_text())
    k = key()
    with cf.ThreadPoolExecutor(5) as ex:
        for line in ex.map(lambda kind: job(k, world, kind, spec), kinds):
            print(line)


if __name__ == "__main__":
    main()
