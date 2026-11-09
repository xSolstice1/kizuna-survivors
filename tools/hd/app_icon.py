"""The PWA home-screen icon: Ren on the game's background, written to web/icon-{192,512}.png.

    python3 tools/hd/app_icon.py

Ren is rendered from the supersampled canvas (not the 4x PNG) so the 512px icon stays sharp,
and kept inside the maskable safe zone (the centre circle, 80% of the icon) so launchers that
crop to a circle or squircle never clip him.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
import hdkit  # noqa: E402
import heroes_a  # noqa: E402
from PIL import Image  # noqa: E402

WEB = hdkit.ROOT / "web"
BG = "#1c1828"

captured = []
hdkit.Art.save = lambda self, path: captured.append(self.img)
heroes_a.ren(False)
ren = captured[0]

for size in (512, 192):
    h = int(size * 0.74)
    sprite = ren.resize((h * ren.width // ren.height, h), Image.LANCZOS)
    icon = Image.new("RGBA", (size, size), hdkit.rgb(BG))
    icon.alpha_composite(sprite, ((size - sprite.width) // 2, (size - h) // 2))
    icon.convert("RGB").save(WEB / f"icon-{size}.png")
    print(WEB / f"icon-{size}.png")
