"""Render every art/src/*.px into one preview sheet: python3 tools/preview_px.py out.png [scale]"""
import sys, pathlib
from PIL import Image, ImageDraw

def parse(text):
    mirror, pal, rows, body = False, {}, [], False
    for line in text.splitlines():
        line = line.rstrip()
        if not line or line.startswith("#"):
            continue
        if body:
            rows.append(line + line[::-1] if mirror else line)
        elif line == "---":
            body = True
        elif line == "mirror":
            mirror = True
        else:
            c, hexv = line.split(" ", 1)
            h = hexv.strip().lstrip("#")
            pal[c] = tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))
    w = len(rows[0])
    for i, r in enumerate(rows):
        assert len(r) == w, f"row {i} width {len(r)} != {w}"
    img = Image.new("RGBA", (w, len(rows)))
    for y, r in enumerate(rows):
        for x, ch in enumerate(r):
            if ch != ".":
                img.putpixel((x, y), pal[ch] + (255,))
    return img

out, scale = sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 6
srcs = sorted(pathlib.Path(__file__).resolve().parent.parent.joinpath("art/src").glob("*.px"))
imgs = [(p.stem, parse(p.read_text())) for p in srcs]
W = sum(i.width * scale + 20 for _, i in imgs) + 20
H = max(i.height for _, i in imgs) * scale + 50
sheet = Image.new("RGBA", (W, H), "#3a3550")
d = ImageDraw.Draw(sheet)
x = 20
for name, i in imgs:
    sheet.alpha_composite(i.resize((i.width * scale, i.height * scale), Image.NEAREST), (x, 30))
    d.text((x, 8), name, fill="white")
    x += i.width * scale + 20
sheet.save(out)
