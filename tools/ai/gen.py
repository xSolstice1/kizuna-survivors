"""Generate images through OpenRouter. Reads OPENROUTER_API_KEY from .env.

    python3 -I tools/ai/gen.py <prompt.txt> <name> [--model M] [--n N] [--ref img.png ...] [--aspect 16:9]

Saves art/ai/raw/<name>_<model>_<k>.png and appends each call's cost to art/ai/spend.csv.
"""
import argparse, base64, concurrent.futures as cf, csv, json, pathlib, time, urllib.request, urllib.error

ROOT = pathlib.Path(__file__).resolve().parents[2]
RAW = ROOT / "art" / "ai" / "raw"
SPEND = ROOT / "art" / "ai" / "spend.csv"


def key():
    for line in (ROOT / ".env").read_text().splitlines():
        if line.startswith("OPENROUTER_API_KEY="):
            return line.split("=", 1)[1].strip()
    raise SystemExit("OPENROUTER_API_KEY missing from .env")


def call(k, model, prompt, refs, aspect):
    content = [{"type": "text", "text": prompt}]
    for r in refs:
        b = base64.b64encode(pathlib.Path(r).read_bytes()).decode()
        content.append({"type": "image_url", "image_url": {"url": f"data:image/png;base64,{b}"}})
    body = {"model": model, "modalities": ["image", "text"], "messages": [{"role": "user", "content": content}],
            "usage": {"include": True}}
    if aspect:
        body["image_config"] = {"aspect_ratio": aspect}
    req = urllib.request.Request("https://openrouter.ai/api/v1/chat/completions", data=json.dumps(body).encode(),
                                 headers={"Authorization": f"Bearer {k}", "Content-Type": "application/json"})
    try:
        return json.load(urllib.request.urlopen(req, timeout=240))
    except urllib.error.HTTPError as e:
        return {"error": f"{e.code} {e.read().decode()[:300]}"}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("prompt"); ap.add_argument("name")
    ap.add_argument("--model", default="google/gemini-3.1-flash-lite-image")
    ap.add_argument("--n", type=int, default=1)
    ap.add_argument("--ref", nargs="*", default=[])
    ap.add_argument("--aspect", default=None)
    a = ap.parse_args()
    k, prompt = key(), pathlib.Path(a.prompt).read_text()
    RAW.mkdir(parents=True, exist_ok=True)
    tag = a.model.split("/")[-1]
    start = len(list(RAW.glob(f"{a.name}_{tag}_*.png")))
    total = 0.0
    with cf.ThreadPoolExecutor(4) as ex:
        futs = [ex.submit(call, k, a.model, prompt, a.ref, a.aspect) for _ in range(a.n)]
        for i, f in enumerate(futs):
            r = f.result()
            if "error" in r:
                print("error:", r["error"]); continue
            msg = r["choices"][0]["message"]
            cost = float(r.get("usage", {}).get("cost") or 0)
            total += cost
            imgs = msg.get("images") or []
            for j, im in enumerate(imgs):
                url = im["image_url"]["url"]
                out = RAW / f"{a.name}_{tag}_{start + i}{'' if j == 0 else f'_{j}'}.png"
                out.write_bytes(base64.b64decode(url.split(",", 1)[1]))
                print("saved", out.relative_to(ROOT), f"${cost:.4f}")
            if not imgs:
                print("no image returned:", (msg.get("content") or "")[:200])
            new = not SPEND.exists()
            with SPEND.open("a", newline="") as fh:
                w = csv.writer(fh)
                if new: w.writerow(["time", "name", "model", "cost"])
                w.writerow([time.strftime("%Y-%m-%d %H:%M:%S"), a.name, a.model, f"{cost:.6f}"])
    print(f"batch cost ${total:.4f}")


if __name__ == "__main__":
    main()
