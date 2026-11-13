"""Drive the web build headlessly and take screenshots (for checking art changes in game).

    bash scripts/build-web.sh
    sed -i 's#url: "https://[^"]*"#url: ""#' dist/config.js     # cloud saves off locally
    (cd dist && python3 -m http.server 8765 &)
    python3 tools/play/shot.py wait:3000 click:163,397 wait:1200 click:1123,602 wait:3000 shot:w1

Steps: click:x,y  key:Name  type:text  down:Key  up:Key  hold:Key:ms  wait:ms  shot:name
(writes /tmp/<name>.png at 2560x1440). Coordinates are CSS pixels of a 1280x720 page.
Useful spots: main menu Start (163,397), Armory (402,397); world select cards W1 (640,192),
W2 (1046,192), W3 (230,435), W4 (640,435), W5 (1046,435); Begin (1123,602). Typing
"kizunadev" on the menu opens the dev panel (unlock all: 488,231 then 794,231, close 640,548);
in a run, Esc then Dev (1074,473) gives max loadout / awaken / fill special. Hero tabs in the
Armory: Riku 85, Shun 127, Toya 169, Kuon 211 at y 96; Select (364,154); Back (1176,675).
Needs `pip install playwright && playwright install chromium`.
"""
import asyncio, sys
from playwright.async_api import async_playwright
# steps: "click:x,y" "key:Name" "hold:Key:ms" "wait:ms" "shot:name"
async def main(steps):
    async with async_playwright() as p:
        b = await p.chromium.launch(args=["--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--autoplay-policy=no-user-gesture-required"])
        pg = await b.new_page(viewport={"width": 1280, "height": 720}, device_scale_factor=2)
        logs = []
        pg.on("console", lambda m: logs.append(m.text))
        await pg.goto("http://localhost:8765/index.html")
        await pg.wait_for_timeout(4000)
        for s in steps:
            k, _, v = s.partition(":")
            if k == "click":
                x, y = map(float, v.split(",")); await pg.mouse.click(x, y)
            elif k == "key":
                await pg.keyboard.press(v)
            elif k == "hold":
                key, ms = v.split(":"); await pg.keyboard.down(key); await pg.wait_for_timeout(int(ms)); await pg.keyboard.up(key)
            elif k == "type":
                await pg.keyboard.type(v, delay=60)
            elif k == "down":
                await pg.keyboard.down(v)
            elif k == "up":
                await pg.keyboard.up(v)
            elif k == "wait":
                await pg.wait_for_timeout(int(v))
            elif k == "shot":
                await pg.screenshot(path=f"/tmp/{v}.png")
        print("\n".join(l for l in logs if "terrain" in l or "rror" in l)[-3000:])
        await b.close()
asyncio.run(main(sys.argv[1:]))
