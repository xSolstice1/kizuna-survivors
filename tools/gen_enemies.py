"""Generates newer enemy sprites as .px sources in art/src/.

    python3 tools/gen_enemies.py
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_stage  # noqa: E402
from gen_stage import Canvas  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent.parent / "art/src"
gen_stage.OUT = OUT  # Canvas.save writes to gen_stage.OUT


def main():
    # Chochin-obake: one-eyed paper lantern with a long tongue.
    c = Canvas(16, 20, {"R": "#e8613a", "r": "#a8402a", "Y": "#ffd27a", "B": "#2a1a1a", "W": "#ffffff", "T": "#ff7aa0"})
    c.rect(6, 0, 9, 1, "B")
    c.rect(3, 2, 12, 3, "B")
    c.ellipse(8, 10, 7, 7, "R")
    for y in (5, 8, 11, 14):
        c.rect(1, y, 14, y, "r")
    c.ellipse(8, 9, 3, 2.6, "W")
    c.rect(7, 8, 8, 10, "B")
    c.rect(6, 13, 9, 13, "B")
    c.rect(7, 14, 8, 17, "T")
    c.rect(4, 17, 11, 18, "B")
    c.save("chochin", "Chochin-obake (lantern yokai)", outline=True)

    # Nopperabo: faceless figure in a dark kimono.
    c = Canvas(16, 24, {"S": "#f2dcc8", "s": "#d4b8a0", "H": "#2a1a2e", "K2": "#3a2a5a", "k": "#2a1d44", "O": "#c9a46a"})
    c.pal["J"] = c.pal.pop("K2")
    c.ellipse(8, 5.5, 4.5, 5, "H")
    c.ellipse(8, 7, 3.8, 4.2, "S")
    c.rect(5, 9, 10, 10, "s")
    c.rect(3, 11, 12, 22, "J")
    c.rect(7, 11, 8, 22, "k")
    c.rect(3, 15, 12, 16, "O")
    c.rect(1, 12, 3, 18, "J")
    c.rect(12, 12, 14, 18, "J")
    c.save("nopperabo", "Nopperabo (faceless ghost)", outline=True)

    # Night Lord: Chochin Daimyo, a lantern warlord in a horned helmet.
    c = Canvas(48, 48, {
        "R": "#e8613a", "r": "#a8402a", "Y": "#ffe08a", "B": "#2a1a1a", "G": "#c9a040", "g": "#8a6a24",
        "W": "#ffffff", "E": "#ff3a3a", "P": "#5a2a6a", "p": "#3a1a4a",
    })
    # kabuto horns + helmet
    for k in range(9):
        c.rect(6 + k, 2 + k // 2, 7 + k, 3 + k // 2, "G")
        c.rect(40 - k, 2 + k // 2, 41 - k, 3 + k // 2, "G")
    c.ellipse(24, 10, 12, 6, "B")
    c.rect(13, 9, 35, 12, "g")
    c.rect(22, 4, 25, 9, "G")
    # lantern body
    c.ellipse(24, 26, 16, 14, "R")
    for y in (17, 21, 25, 29, 33):
        c.rect(9, y, 39, y, "r")
    c.ellipse(24, 25, 8, 6, "Y")
    c.ellipse(24, 25, 4.5, 4.5, "E")
    c.rect(23, 23, 24, 27, "B")
    # mouth
    c.rect(16, 32, 31, 34, "B")
    for x in range(17, 31, 3):
        c.rect(x, 32, x, 33, "W")
    # armour skirt + base
    c.rect(10, 40, 37, 42, "B")
    c.rect(8, 42, 39, 46, "P")
    for x in range(9, 39, 4):
        c.rect(x, 43, x + 1, 45, "p")
    c.save("night_lord", "Night Lord: Chochin Daimyo", outline=True)
    print("wrote enemy sprites")


if __name__ == "__main__":
    main()
