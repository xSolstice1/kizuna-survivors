"""Generate a video clip through OpenRouter's async video API. Reads OPENROUTER_API_KEY from .env.

    python3 -I tools/ai/video.py <prompt.txt> <name> --first first.png [--model M] [--duration 4]
        [--resolution 720p] [--aspect 16:9]

Saves art/ai/raw/<name>.mp4 and appends the cost to art/ai/spend.csv.
"""
import argparse, base64, csv, json, pathlib, time, urllib.request, urllib.error

ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEND = ROOT / "art" / "ai" / "spend.csv"


def key():
    for line in (ROOT / ".env").read_text().splitlines():
        if line.startswith("OPENROUTER_API_KEY="):
            return line.split("=", 1)[1].strip()
    raise SystemExit("OPENROUTER_API_KEY missing from .env")


def req(k, url, body=None, raw=False):
    r = urllib.request.Request(url, data=json.dumps(body).encode() if body else None,
                               headers={"Authorization": f"Bearer {k}", "Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(r, timeout=300) as resp:
            data = resp.read()
    except urllib.error.HTTPError as e:
        raise SystemExit(f"HTTP {e.code}: {e.read().decode()[:400]}")
    return data if raw else json.loads(data)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("prompt"); ap.add_argument("name")
    ap.add_argument("--first", required=True)
    ap.add_argument("--model", default="google/veo-3.1-lite")
    ap.add_argument("--duration", type=int, default=4)
    ap.add_argument("--resolution", default="720p")
    ap.add_argument("--aspect", default="16:9")
    a = ap.parse_args()
    k = key()
    img = base64.b64encode(pathlib.Path(a.first).read_bytes()).decode()
    body = {"model": a.model, "prompt": pathlib.Path(a.prompt).read_text().strip(), "duration": a.duration,
            "resolution": a.resolution, "aspect_ratio": a.aspect, "generate_audio": False,
            "frame_images": [{"type": "image_url", "image_url": {"url": f"data:image/png;base64,{img}"}, "frame_type": "first_frame"}]}
    job = req(k, "https://openrouter.ai/api/v1/videos", body)
    jid = job["id"]
    print("job", jid, job.get("status"))
    while True:
        time.sleep(15)
        st = req(k, f"https://openrouter.ai/api/v1/videos/{jid}")
        print("status", st.get("status"))
        if st.get("status") in ("completed", "failed", "cancelled", "expired"):
            break
    if st.get("status") != "completed":
        raise SystemExit(json.dumps(st)[:600])
    cost = float((st.get("usage") or {}).get("cost") or 0)
    out = ROOT / "art" / "ai" / "raw" / f"{a.name}.mp4"
    out.write_bytes(req(k, f"https://openrouter.ai/api/v1/videos/{jid}/content?index=0", raw=True))
    with SPEND.open("a", newline="") as fh:
        csv.writer(fh).writerow([time.strftime("%Y-%m-%d %H:%M:%S"), a.name, a.model, f"{cost:.6f}"])
    print("saved", out.relative_to(ROOT), f"${cost:.4f}")


if __name__ == "__main__":
    main()
