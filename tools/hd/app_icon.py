"""The PWA home-screen icon: Ren on the game's background, written to web/icon-{192,512}.png.

    python3 tools/hd/app_icon.py

Ren is the art-pass portrait (art/hd/ren_front.png), kept inside the maskable safe zone (the
centre circle, 80% of the icon) so launchers that crop to a circle or squircle never clip him.
"""
import pathlib

from PIL import Image

ROOT = pathlib.Path(__file__).resolve().parents[2]
BG = (0x1C, 0x18, 0x28)

ren = Image.open(ROOT / "art/hd/ren_front.png").convert("RGBA")
ren = ren.crop(ren.getchannel("A").getbbox())

for size in (512, 192):
    h = int(size * 0.74)
    sprite = ren.resize((h * ren.width // ren.height, h), Image.LANCZOS)
    icon = Image.new("RGBA", (size, size), BG)
    icon.alpha_composite(sprite, ((size - sprite.width) // 2, (size - h) // 2))
    icon.convert("RGB").save(ROOT / f"web/icon-{size}.png")
    print(ROOT / f"web/icon-{size}.png")
