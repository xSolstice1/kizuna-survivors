"""Generates Toya Hyakuashi (original cursed youth carrying the Omukade's blood) as .px sources.

    python3 tools/gen_toya.py

toya.px         base form: messy charcoal hair with a white streak, pale skin, one cursed
                crimson eye, a bandage across the other, black high-collared coat with crimson
                lining. (His four crimson tendrils are drawn in code so they can move.)
toya_mukade.px  Hundred-Leg Rampage: red-black chitin plates, a mandible jaw-guard, wilder
                crimson-tipped hair, a blazing eye. (The centipede tail is drawn in code.)
icons/crimson_tendrils.px, mukade_spines.px, devour.px, venom_fang.px

Same 24x32 frame and leg rows as heroine.px, so the walk-cycle leg lift works.
"""
import math
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_stage  # noqa: E402
from gen_stage import Canvas  # noqa: E402

gen_stage.OUT = pathlib.Path(__file__).resolve().parent.parent / "art/src"

BASE = {
    "H": "#34303e", "h": "#1e1b26", "L": "#f0eef6",   # hair, shade, white streak
    "S": "#f6e8e2", "s": "#dcc4bc",                   # pale skin
    "E": "#1a0a10", "R": "#ff2a3a", "W": "#ffd0d0",   # cursed eye
    "G": "#ece8de", "g": "#b8b2a6",                   # bandage
    "N": "#18161e", "n": "#0c0b10", "C": "#a01c2c",   # coat, crimson lining
    "D": "#2c2a34", "B": "#141218",                   # trousers, boots
}

MUKADE = {
    **BASE,
    "h": "#7a0e1c", "L": "#ff6a6a",
    "R": "#ff4040", "W": "#ffffff",
    "P": "#7a1020", "p": "#2a0608", "Q": "#d43040",   # chitin plates
    "F": "#e8dccb",                                   # mandible tips
}


def tri_up(c, x0, x1, tip_x, tip_y, base_y, col):
    for y in range(tip_y, base_y + 1):
        t = (y - tip_y) / max(1, base_y - tip_y)
        lx = round(tip_x + (x0 - tip_x) * t)
        rx = round(tip_x + (x1 - tip_x) * t)
        c.rect(min(lx, rx), y, max(lx, rx), y, col)


def draw(c, mukade):
    # Legs.
    for x0 in (7, 14):
        c.rect(x0, 26, x0 + 2, 28, "D")
        c.rect(x0, 29, x0 + 2, 30, "B")
    # Coat: long, black, crimson-lined front edges.
    c.rect(6, 16, 17, 25, "N")
    c.rect(15, 16, 17, 25, "n")
    c.rect(11, 17, 11, 25, "C")
    c.rect(12, 17, 12, 25, "n")
    c.rect(4, 17, 5, 22, "N"); c.rect(4, 23, 5, 23, "S")
    c.rect(18, 17, 19, 22, "n"); c.rect(18, 23, 19, 23, "S")
    # High collar.
    c.rect(7, 14, 16, 16, "N")
    c.rect(7, 14, 7, 16, "C"); c.rect(16, 14, 16, 16, "C")
    # Head and messy hair.
    c.ellipse(12, 8.5, 7.4, 6.8, "H")
    c.rect(7, 8, 16, 14, "S")
    c.rect(8, 14, 15, 14, "s")
    tri_up(c, 4, 8, 3, 1, 5, "H")
    tri_up(c, 8, 12, 10, 0, 3, "H")
    tri_up(c, 12, 16, 14, 1, 3, "H")
    tri_up(c, 16, 20, 20, 2, 6, "H")
    tri_up(c, 2, 5, 1, 8, 12, "H")
    tri_up(c, 18, 21, 22, 8, 12, "H")
    c.rect(4, 9, 6, 13, "H"); c.rect(17, 9, 19, 13, "H")
    for x, d in zip(range(7, 17), [9, 8, 9, 8, 8, 8, 9, 8, 9, 8]):
        c.rect(x, 7, x, d, "H")
    # A thin white streak falling over the left side of the fringe.
    for k in range(5):
        c.px(8 + k // 3, 4 + k, "L")
    for x in (10, 13):
        c.px(x, 8, "h")
    # Right side: bandage across the eye. Left: the cursed crimson eye.
    c.rect(13, 10, 16, 11, "G")
    c.px(13, 12, "G"); c.px(16, 9, "G")
    c.px(14, 10, "g"); c.px(16, 11, "g")
    c.rect(8, 10, 10, 10, "E")
    c.px(8, 11, "E"); c.px(9, 11, "R"); c.px(10, 11, "R")
    c.px(9, 12, "R"); c.px(10, 12, "E")
    c.px(10, 11, "W")
    c.rect(11, 13, 12, 13, "s")
    if mukade:
        # Chitin plates over shoulders, arms and shins.
        c.rect(4, 15, 8, 16, "P"); c.rect(15, 15, 19, 16, "P")
        c.rect(4, 16, 8, 16, "Q"); c.rect(15, 16, 19, 16, "Q")
        for y in (18, 20, 22):
            c.rect(3, y, 5, y, "P"); c.rect(18, y, 20, y, "P")
        for x0 in (7, 14):
            c.rect(x0, 26, x0 + 2, 26, "Q")
            c.rect(x0, 27, x0 + 2, 28, "P")
        c.rect(8, 18, 15, 19, "p"); c.rect(9, 18, 14, 18, "P")
        # Mandible jaw-guard over the lower face, fangs at the corners.
        c.rect(8, 13, 15, 14, "p")
        c.px(8, 15, "F"); c.px(15, 15, "F"); c.px(9, 14, "F"); c.px(14, 14, "F")
        c.rect(10, 13, 13, 13, "P")
        # Blazing eye: the iris spills over.
        c.px(8, 11, "R"); c.px(11, 11, "R"); c.px(9, 10, "R")
        # Wilder hair with crimson tips.
        tri_up(c, 1, 3, 0, 3, 7, "H")
        tri_up(c, 20, 23, 23, 3, 7, "H")
        for x, y in ((0, 3), (3, 1), (10, 0), (14, 1), (20, 2), (23, 3)):
            c.px(x, y, "h")


def icons():
    from gen_icons import Icon
    # Crimson Tendrils: four tendrils fanning up from a single root, curling at the tips.
    i = Icon({"R": "#d43040", "r": "#6a0e1a"})
    for k, ang in enumerate((-1.1, -0.4, 0.4, 1.1)):
        for t in range(13):
            u = t / 12
            x = 7.5 + ang * t * 0.5 + math.sin(u * 5.0 + k) * 1.2 * u
            y = 15 - t * 1.1
            col = "r" if t % 3 == 0 else "R"
            i.px(round(x), round(y), col)
            if t < 9:
                i.px(round(x) + 1, round(y), "R")
    i.save("crimson_tendrils", "Crimson Tendrils")
    # Mukade Spines: a segmented centipede spine arcing out of the ground, legs along it.
    i = Icon({"P": "#d43040", "p": "#6a0e1a", "W": "#e8dccb", "G": "#3a3040"})
    i.rect(0, 14, 15, 15, "G")
    for k in range(7):
        u = k / 6
        x = 3 + u * 9
        y = 13 - math.sin(u * math.pi * 0.9) * 10
        i.circle(x, y, 2.1 - u * 0.6, "P" if k % 2 == 0 else "p")
        i.px(round(x), round(y) + 2, "W")
    i.poly([(13, 2), (15, 0.5), (14.5, 3.5)], "W")
    i.save("mukade_spines", "Mukade Spines")
    # Devour: a fanged maw.
    i = Icon({"R": "#8a0e1c", "r": "#3a0608", "W": "#f2ece0"})
    i.circle(8, 8, 7.0, "R")
    i.circle(8, 8.5, 4.6, "r")
    for x in (4, 7, 10):
        i.poly([(x, 4), (x + 2, 4), (x + 1, 7)], "W")
    for x in (5, 8, 11):
        i.poly([(x, 13), (x + 2, 13), (x + 1, 10)], "W")
    i.save("devour", "Devour")
    # Venom Fang: a curved fang dripping venom.
    i = Icon({"W": "#f2ece0", "w": "#b8b0a0", "V": "#7ae05a", "v": "#3a8a2a"})
    i.poly([(3, 1), (10, 2), (13, 9), (9, 14), (8, 8)], "W")
    i.line(9, 3, 11, 9, "w")
    i.px(9, 15, "V"); i.px(12, 12, "V"); i.px(13, 13, "v")
    i.save("venom_fang", "Venom Fang")


def main():
    for name, pal, m in (("toya", BASE, False), ("toya_mukade", MUKADE, True)):
        c = Canvas(24, 32, pal)
        draw(c, m)
        c.save(name, "Toya Hyakuashi, Hundred-Leg Rampage" if m else "Toya Hyakuashi (cursed youth)", outline=True)
    icons()
    print("wrote toya.px, toya_mukade.px and four icons")


if __name__ == "__main__":
    main()
