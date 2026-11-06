"""Generates Kuon Hoshimi (original star-reading onmyoji) as .px sources in art/src/.

    python3 tools/gen_kuon.py

kuon.px         base: swept pale-lavender hair, a white star-embroidered paper veil over the
                eyes, deep navy onmyoji coat with gold star flecks and a violet sash.
kuon_domain.px  Endless Firmament: veil lifted, eyes blazing star-blue, hair lit white-blue,
                glowing lines through the coat.
icons/gravity_well.px, repulsion_star.px, void_lance.px
"""
import math
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_stage  # noqa: E402
from gen_stage import Canvas  # noqa: E402

gen_stage.OUT = pathlib.Path(__file__).resolve().parent.parent / "art/src"

BASE = {
    "H": "#dcd4f4", "h": "#9a90c4", "L": "#ffffff",
    "S": "#ffe6d6", "s": "#e8c4b0",
    "V": "#f6f2ff", "v": "#b8bce6", "Y": "#ffd23a",
    "N": "#1c2448", "n": "#10162e", "W": "#e8ecff", "P": "#6a4ac8",
    "D": "#242a48", "B": "#141830",
    "E": "#1a2050", "C": "#7af0ff",
}
DOMAIN = {**BASE, "H": "#f2f6ff", "h": "#8ab8ff", "L": "#ffffff", "N": "#18204a", "C": "#9af8ff", "G": "#4ac8ff"}


def tri_up(c, x0, x1, tip_x, tip_y, base_y, col):
    for y in range(tip_y, base_y + 1):
        t = (y - tip_y) / max(1, base_y - tip_y)
        lx = round(tip_x + (x0 - tip_x) * t)
        rx = round(tip_x + (x1 - tip_x) * t)
        c.rect(min(lx, rx), y, max(lx, rx), y, col)


def draw(c, domain):
    for x0 in (7, 14):
        c.rect(x0, 26, x0 + 2, 28, "D")
        c.rect(x0, 29, x0 + 2, 30, "B")
    # Long onmyoji coat with wide sleeves and a white inner collar.
    c.rect(6, 16, 17, 26, "N")
    c.rect(15, 16, 17, 26, "n")
    c.rect(3, 17, 5, 23, "N"); c.rect(18, 17, 20, 23, "n")
    c.rect(4, 24, 5, 24, "S"); c.rect(18, 24, 19, 24, "S")
    c.rect(10, 15, 13, 18, "W")
    c.rect(6, 21, 17, 21, "P")
    for x, y in ((8, 19), (15, 18), (7, 24), (12, 23), (16, 25), (4, 20), (19, 21)):
        c.px(x, y, "G" if domain else "Y")
    if domain:
        c.rect(8, 22, 8, 26, "G"); c.rect(15, 22, 15, 26, "G")
    # Head: hair swept back and up, a few loose strands.
    c.ellipse(12, 8.5, 7.2, 6.6, "H")
    c.rect(7, 8, 16, 14, "S")
    c.rect(8, 14, 15, 14, "s")
    tri_up(c, 5, 10, 6, 0, 4, "H")
    tri_up(c, 10, 15, 12, 0, 3, "H")
    tri_up(c, 14, 19, 18, 1, 4, "H")
    c.rect(4, 8, 6, 12, "H"); c.rect(17, 8, 19, 12, "H")
    c.rect(7, 7, 16, 7, "H")
    c.px(7, 8, "H"); c.px(16, 8, "H")
    c.rect(9, 2, 11, 2, "L"); c.rect(8, 3, 9, 3, "L")
    c.px(6, 9, "h"); c.px(17, 9, "h")
    if not domain:
        # The paper veil: a band across the eyes with a little gold star.
        c.rect(6, 9, 17, 11, "V")
        c.rect(6, 11, 17, 11, "v")
        c.px(11, 10, "Y"); c.px(12, 10, "Y"); c.px(11, 9, "Y"); c.px(12, 11, "Y")
        c.px(5, 10, "V"); c.px(18, 10, "V"); c.px(18, 11, "v")
    else:
        # Star-blue eyes, blazing.
        for ex in (8, 13):
            c.rect(ex, 10, ex + 2, 10, "E")
            c.px(ex, 11, "C"); c.px(ex + 1, 11, "C"); c.px(ex + 2, 11, "E")
            c.px(ex + 1, 11, "L")
            c.px(ex + 1, 12, "C")
    c.rect(11, 13, 12, 13, "s")
    c.px(13, 13, "s")  # the smirk


def icons():
    from gen_icons import Icon
    i = Icon({"N": "#10162e", "B": "#2a4aa8", "b": "#5a8aff", "W": "#dce8ff"})
    i.circle(8, 8, 7.0, "B")
    i.circle(8, 8, 5.2, "N")
    for k in range(3):
        for t in range(8):
            a = k * 2.094 + t * 0.45
            r = 6.0 - t * 0.7
            i.px(round(8 + math.cos(a) * r), round(8 + math.sin(a) * r), "b")
    i.circle(8, 8, 1.4, "W")
    i.save("gravity_well", "Gravity Well")
    import gen_icons
    i = Icon({"R": "#ff3a4a", "r": "#a01828", "W": "#ffe0e0"})
    i.poly(gen_icons.star_points(8, 8, 7.4, 2.6, 4, -math.pi / 2), "R")
    i.poly(gen_icons.star_points(8, 8, 4.0, 1.4, 4, -math.pi / 4), "r")
    i.circle(8, 8, 1.6, "W")
    i.save("repulsion_star", "Repulsion Star")
    i = Icon({"P": "#9a4aff", "p": "#4a1a8a", "V": "#e0c8ff", "N": "#0e0820"})
    i.circle(8, 8, 7.0, "P")
    i.circle(8, 8, 4.8, "N")
    i.circle(6.5, 6.5, 1.6, "V")
    for k in range(8):
        a = k * math.pi / 4
        i.px(round(8 + math.cos(a) * 7.4), round(8 + math.sin(a) * 7.4), "V")
    i.save("void_lance", "Void Lance")


def main():
    for name, pal, d in (("kuon", BASE, False), ("kuon_domain", DOMAIN, True)):
        c = Canvas(24, 32, pal)
        draw(c, d)
        c.save(name, "Kuon Hoshimi, Endless Firmament" if d else "Kuon Hoshimi (star-reading onmyoji)", outline=True)
    icons()
    print("wrote kuon.px, kuon_domain.px and three icons")


if __name__ == "__main__":
    main()
