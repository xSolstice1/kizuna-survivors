"""Generates Ren Amane (original shrine maiden) as .px sources in art/src/.

    python3 tools/gen_ren.py

ren.px          a traditional miko: long dark hair with a short fringe and a white folded-paper
                hair tie, white kimono (hakui) with a red under-collar, red pleated hakama, white
                tabi, and a gohei (purification wand) in her right hand.
ren_divine.px   Divine Purification: gold-trimmed robes, a brighter hakama, golden eyes and a
                glowing hair tie. (Yin-yang orbs, the magic circle and talismans are drawn in code.)
icons/yinyang_orbs.px, spirit_seal.px, purification_wave.px
"""
import math
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_stage  # noqa: E402
from gen_stage import Canvas  # noqa: E402

gen_stage.OUT = pathlib.Path(__file__).resolve().parent.parent / "art/src"

BASE = {
    "H": "#3a2622", "h": "#22150f", "L": "#6e4a3a",
    "S": "#ffe2cc", "s": "#f2b89a", "E": "#24163f", "I": "#6a7cff", "W": "#ffffff",
    "O": "#f6f2ec", "k": "#c8c0b8", "R": "#c42a3a", "r": "#8a1a26",
    "T": "#ffffff", "B": "#8a5a3a", "Z": "#4a2e2a", "M": "#ff9aa8",
}
DIVINE = {**BASE, "R": "#ff3a4a", "r": "#c42a3a", "I": "#ffd23a", "E": "#a0400a", "Y": "#ffd23a", "L": "#a07a5a", "k": "#ffe8a8"}


def draw(c, divine):
    # Long hair behind the shoulders.
    c.rect(4, 10, 6, 21, "H"); c.rect(17, 10, 19, 21, "H")
    c.rect(4, 19, 5, 21, "h"); c.rect(18, 19, 19, 21, "h")
    # Red hakama with pleats, white tabi, zori.
    c.rect(6, 19, 17, 26, "R")
    for x in (8, 11, 14):
        c.rect(x, 21, x, 26, "r")
    for x0 in (7, 14):
        c.rect(x0, 27, x0 + 2, 28, "T")
        c.rect(x0, 29, x0 + 2, 30, "Z")
    # White kimono top crossed at the front, red under-collar.
    c.rect(6, 14, 17, 19, "O")
    c.rect(15, 14, 17, 19, "k")
    c.px(11, 15, "R"); c.px(12, 15, "R"); c.px(10, 16, "R"); c.px(13, 16, "R")
    c.rect(11, 16, 12, 18, "k")
    # Sleeves (wide), hands.
    c.rect(3, 15, 5, 20, "O"); c.rect(18, 15, 20, 19, "k")
    c.rect(4, 21, 5, 21, "S"); c.rect(19, 20, 20, 20, "S")
    # Head and hair: rounded crown, straight short fringe.
    c.ellipse(12, 8.0, 7.2, 6.4, "H")
    c.rect(7, 8, 16, 13, "S")
    c.rect(8, 13, 15, 13, "s")
    c.rect(6, 6, 17, 8, "H")
    for x in range(7, 17):
        c.px(x, 9, "H" if x % 3 else "h")
    c.rect(9, 3, 11, 3, "L"); c.rect(8, 4, 9, 4, "L")
    c.rect(5, 8, 6, 13, "H"); c.rect(17, 8, 18, 13, "H")
    # White folded-paper hair tie on top, with a red cord.
    c.rect(9, 0, 14, 1, "T")
    c.px(8, 1, "T"); c.px(15, 1, "T")
    c.px(11, 2, "R"); c.px(12, 2, "R")
    # Eyes.
    for ex in (8, 13):
        c.rect(ex, 10, ex + 2, 10, "E")
        c.px(ex, 11, "I"); c.px(ex + 1, 11, "W"); c.px(ex + 2, 11, "I")
    c.px(8, 12, "M"); c.px(15, 12, "M")
    c.rect(11, 12, 12, 12, "s")
    # Gohei: a wooden wand with zigzag paper streamers, in her right hand.
    c.rect(20, 10, 20, 21, "B")
    for y, x in ((9, 21), (10, 22), (11, 21), (12, 22), (13, 21)):
        c.px(x, y, "T")
    for y, x in ((10, 19), (11, 18), (12, 19), (13, 18)):
        c.px(x, y, "T")
    c.px(20, 9, "Y" if divine else "B")
    if divine:
        c.rect(6, 19, 17, 19, "Y")
        c.rect(6, 26, 17, 26, "Y")
        c.px(6, 14, "Y"); c.px(17, 14, "Y")
        c.rect(9, 0, 14, 0, "Y")


def icons():
    from gen_icons import Icon
    i = Icon({"W": "#ffffff", "B": "#1a1423", "R": "#e83a3a", "r": "#ffb0b8"})
    i.circle(8, 8, 7.0, "W")
    for y in range(16):
        for x in range(16):
            dx, dy = x + 0.5 - 8, y + 0.5 - 8
            if dx * dx + dy * dy <= 49 and (dx > 0 if True else False):
                # Classic S-split: right half dark, with the two lobes swapped.
                top = (dx * dx + (dy + 3.5) ** 2) <= 12.25
                bot = (dx * dx + (dy - 3.5) ** 2) <= 12.25
                if (dx > 0 and not top) or bot:
                    i.px(x, y, "B")
    i.circle(8, 4.5, 1.2, "B")
    i.circle(8, 11.5, 1.2, "W")
    i.save("yinyang_orbs", "Yin-Yang Orbs")
    i = Icon({"P": "#ff9ec4", "p": "#e0609a", "W": "#ffffff", "Y": "#ffe066"})
    for k in range(6):
        a = k * math.pi / 3
        i.circle(8 + math.cos(a) * 5, 8 + math.sin(a) * 5, 2.2, "P" if k % 2 else "p")
    i.circle(8, 8, 2.4, "Y")
    i.px(8, 8, "W")
    i.save("spirit_seal", "Spirit Seal")
    i = Icon({"W": "#ffffff", "P": "#ffc0d8", "R": "#e83a3a"})
    i.circle(8, 8, 7.2, "P")
    i.circle(8, 8, 5.6, "K")
    i.circle(8, 8, 4.4, "W")
    i.circle(8, 8, 3.0, "K")
    i.circle(8, 8, 1.4, "R")
    i.save("purification_wave", "Purification Wave")


def main():
    for name, pal, d in (("ren", BASE, False), ("ren_divine", DIVINE, True)):
        c = Canvas(24, 32, pal)
        draw(c, d)
        c.save(name, "Ren Amane, Divine Purification" if d else "Ren Amane (shrine maiden)", outline=True)
    icons()
    print("wrote ren.px, ren_divine.px and three icons")


if __name__ == "__main__":
    main()
