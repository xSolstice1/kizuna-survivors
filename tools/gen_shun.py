"""Generates Shun Yamikiri (original spirit ferryman) as .px sources in art/src/.

    python3 tools/gen_shun.py

shun.px             base form: wild ash-silver hair, charcoal haori over a violet kimono,
                    crimson rope belt with a soul-lantern charm, dark hakama.
shun_hannya.px      Hannya Unbound: a horned white hannya mask with gold eyes and fangs,
                    wilder crimson-tipped hair, bone armour plates and dark markings.
cleaver.px          his oversized bone-white nodachi (drawn separately so it can swing).
cleaver_hannya.px   the blade in Hannya form: black with a jagged crimson edge.
icons/grave_cleaver.px

Same 24x32 frame and leg rows as heroine.px, so the client's walk-cycle leg lift works.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_stage  # noqa: E402
from gen_stage import Canvas  # noqa: E402

gen_stage.OUT = pathlib.Path(__file__).resolve().parent.parent / "art/src"

BASE = {
    "H": "#c8ccd8", "h": "#8a8fa8", "L": "#ffffff",   # hair
    "S": "#ffe2cc", "s": "#f2b89a",                   # skin
    "E": "#1c1426", "A": "#9a6ae0", "W": "#ffffff",   # eyes
    "N": "#3a3a46", "n": "#24242e", "V": "#5a3a8a",   # haori, kimono
    "C": "#c42a3a", "c": "#7e1828",                   # rope belt
    "D": "#2a2433", "B": "#4a2e2a",                   # hakama, sandals
    "Y": "#ffd27a", "r": "#e8613a",                   # soul-lantern charm
}

HANNYA = {
    **BASE,
    "h": "#c42a3a", "L": "#ffffff",                   # crimson-tipped hair
    "N": "#1a1418", "n": "#0e0a0e", "V": "#6a1028",
    "M": "#f6f2ea", "m": "#b8b0a0",                   # mask
    "G": "#ffd23a", "k": "#120808",                   # gold eyes, sockets / fangs
    "P": "#e8e0d0", "p": "#a8a090",                   # bone armour
    "X": "#4a0a1a",                                   # dark markings
    "O": "#d49a2a", "o": "#7a4a10",                   # bronze horns
}


def tri_up(c, x0, x1, tip_x, tip_y, base_y, col):
    for y in range(tip_y, base_y + 1):
        t = (y - tip_y) / max(1, base_y - tip_y)
        lx = round(tip_x + (x0 - tip_x) * t)
        rx = round(tip_x + (x1 - tip_x) * t)
        c.rect(min(lx, rx), y, max(lx, rx), y, col)


def draw(c, hannya):
    # ── Legs: wide hakama, then sandals (rows 26..30) ──
    for x0 in (7, 14):
        c.rect(x0, 26, x0 + 2, 28, "D")
        c.rect(x0, 29, x0 + 2, 30, "B")
    c.rect(7, 25, 16, 26, "D")
    # ── Body: haori open over the violet kimono ──
    c.rect(6, 17, 17, 24, "N")
    c.rect(9, 17, 14, 24, "V")
    c.rect(15, 17, 17, 24, "n")
    c.rect(11, 17, 12, 19, "S")            # collar V-neck
    c.rect(6, 21, 17, 22, "C")             # rope belt
    c.rect(6, 22, 17, 22, "c")
    c.rect(7, 23, 8, 24, "Y"); c.px(7, 24, "r"); c.px(8, 23, "r")   # soul-lantern charm
    # Sleeves: wide, the right hand forward on the sword's grip.
    c.rect(3, 17, 5, 22, "N")
    c.rect(4, 23, 5, 23, "S")
    c.rect(18, 17, 20, 20, "n")
    c.rect(19, 21, 20, 21, "S")
    # ── Head ──
    c.ellipse(12, 8.5, 7.8, 7.0, "H")
    c.rect(7, 8, 16, 14, "S")
    c.rect(8, 14, 15, 14, "s")
    # Wild, swept spikes.
    tri_up(c, 3, 8, 2, 0, 5, "H")
    tri_up(c, 8, 12, 9, 0, 3, "H")
    tri_up(c, 12, 16, 15, 0, 3, "H")
    tri_up(c, 16, 20, 21, 1, 5, "H")
    tri_up(c, 1, 4, 0, 8, 12, "H")
    tri_up(c, 19, 22, 23, 8, 12, "H")
    c.rect(4, 9, 6, 14, "H"); c.rect(17, 9, 19, 14, "H")
    c.rect(4, 13, 5, 15, "h"); c.rect(18, 13, 19, 15, "h")
    for x, d in zip(range(7, 17), [10, 8, 9, 10, 8, 9, 10, 8, 9, 10]):
        c.rect(x, 7, x, d, "H")
    for x in (9, 12, 15):
        c.px(x, 8, "h")
    c.rect(10, 2, 12, 2, "L"); c.rect(9, 3, 10, 3, "L")
    if not hannya:
        # Sharp, narrow eyes.
        for ex in (8, 13):
            c.rect(ex, 10, ex + 2, 10, "E")
            c.px(ex, 11, "A"); c.px(ex + 1, 11, "A"); c.px(ex + 2, 11, "E")
            c.px(ex + 1, 11, "W")
        c.rect(11, 13, 12, 13, "s")
    else:
        # Hannya mask over the whole face: horns, gold eyes in black sockets, a fanged grin.
        c.rect(7, 7, 16, 14, "M")
        c.rect(8, 15, 15, 15, "M")
        c.rect(7, 14, 7, 14, "m"); c.rect(16, 14, 16, 14, "m")
        # Two thick horns curving up and out, gold-tipped.
        for side in (-1, 1):
            base_x = 9 if side < 0 else 14
            for k in range(7):
                x = base_x + side * (k // 2)
                y = 7 - k
                c.px(x, y, "O"); c.px(x + side, y, "o")
            c.px(base_x + side * 3, 0, "G"); c.px(base_x + side * 4, 0, "G")
        for ex in (8, 13):
            c.rect(ex, 9, ex + 2, 11, "k")
            c.px(ex + 1, 10, "G"); c.px(ex + (2 if ex == 8 else 0), 10, "G")
        c.px(11, 9, "m"); c.px(12, 9, "m")               # furrowed brow
        c.rect(9, 13, 14, 14, "k")                       # mouth
        for x in (9, 11, 12, 14):
            c.px(x, 13, "M")                              # fangs
        c.px(10, 14, "M"); c.px(13, 14, "M")
        # Wilder hair: longer side locks with crimson tips.
        c.rect(3, 13, 4, 17, "H"); c.rect(19, 13, 20, 17, "H")
        c.px(3, 17, "h"); c.px(20, 17, "h"); c.px(4, 16, "h"); c.px(19, 16, "h")
        # Bone armour on the shoulders and chest, dark markings on the sleeves.
        c.rect(5, 16, 8, 17, "P"); c.rect(15, 16, 18, 17, "P")
        c.rect(5, 17, 8, 17, "p"); c.rect(15, 17, 18, 17, "p")
        c.rect(10, 18, 13, 19, "P"); c.rect(10, 19, 13, 19, "p")
        for y in (18, 20):
            c.px(3, y, "X"); c.px(4, y + 1, "X"); c.px(20, y, "X")


def sword(hannya):
    pal = {"W": "#f2ece0", "w": "#b8b0a0", "e": "#ffffff", "C": "#c42a3a", "c": "#7e1828", "G": "#2a2433", "g": "#c9a040"}
    if hannya:
        pal.update({"W": "#1a1418", "w": "#2e2630", "e": "#e0203c"})
    c = Canvas(9, 38, pal)
    # Blade pointing up (tip at the top), slightly curved.
    for y in range(1, 27):
        x0 = 3 + (1 if y < 8 else 0)
        c.rect(x0, y, x0 + 2, y, "W")
        c.px(x0 + 2, y, "w")
        c.px(x0, y, "e")
    c.px(4, 0, "e")
    if hannya:
        for y in range(3, 26, 3):        # jagged crimson teeth along the edge
            c.px(2, y, "e"); c.px(2, y + 1, "e")
    c.rect(1, 27, 7, 28, "g")            # guard
    c.rect(3, 29, 5, 36, "C")            # wrapped hilt
    for y in range(30, 36, 2):
        c.px(4, y, "c")
    c.save("cleaver_hannya" if hannya else "cleaver", "Shun's nodachi" + (" (Hannya)" if hannya else ""), outline=True)


def sword_rest(hannya):
    """The nodachi held at rest, drawn at exactly 45 degrees so it needs no rotation.
    Grip at the bottom-left (hand anchor = pixel (4, 25)), tip at the top-right."""
    pal = {"W": "#f2ece0", "w": "#b8b0a0", "e": "#ffffff", "C": "#c42a3a", "c": "#7e1828", "g": "#c9a040"}
    if hannya:
        pal.update({"W": "#1a1418", "w": "#2e2630", "e": "#e0203c"})
    c = Canvas(30, 30, pal)
    # Blade: a 3-pixel-wide diagonal band from just above the guard to the tip.
    for t in range(8, 27):
        x, y = 1 + t, 28 - t
        c.px(x - 1, y, "w")          # shaded back
        c.px(x, y, "W")
        c.px(x + 1, y, "e")          # bright cutting edge
        if hannya and t % 3 == 0:
            c.px(x + 2, y, "e")      # jagged crimson teeth
    c.px(28, 1, "e")                  # tip
    # Guard: perpendicular bar.
    for k in range(-2, 3):
        c.px(8 + k, 20 + k, "g")
    # Grip: wrapped hilt running down-left from the guard.
    for t in range(1, 7):
        x, y = 8 - t, 20 + t
        c.px(x, y, "C")
        c.px(x - 1, y, "c" if t % 2 else "C")
    c.save("cleaver_rest_hannya" if hannya else "cleaver_rest", "Shun's nodachi at rest" + (" (Hannya)" if hannya else ""), outline=True)


def icon():
    from gen_icons import Icon
    i = Icon({"W": "#f2ece0", "w": "#b8b0a0", "C": "#c42a3a", "g": "#c9a040", "V": "#b08aff"})
    i.line(13.5, 1.5, 4, 11, "W")
    i.line(14.5, 2.5, 5, 12, "W")
    i.line(14.5, 1.5, 5, 11, "w")
    i.line(2, 10, 6, 14, "g")
    i.line(1, 13, 3, 15, "C")
    i.line(2, 13, 3, 14, "C")
    for x, y in ((3, 3), (6, 1), (1, 6)):
        i.px(x, y, "V")                   # spirit wisps
    i.save("grave_cleaver", "Grave Cleaver")


def more_icons():
    import math
    from gen_icons import Icon
    # Soul Tether: a spectral chain curling into a hook.
    i = Icon({"V": "#b08aff", "v": "#6a4ac8", "W": "#e8e0ff"})
    for k in range(5):
        cx, cy = 2.5 + k * 2.6, 13 - k * 2.4
        i.circle(cx, cy, 1.8, "V" if k % 2 == 0 else "v")
        i.px(round(cx), round(cy), "K")
    i.line(13, 2, 15, 4, "W"); i.line(13, 2, 11, 1, "W")
    i.save("soul_tether", "Soul Tether")
    # Lantern Vigil: a pale soul-lantern with a blue flame.
    i = Icon({"P": "#f2ece0", "p": "#b8b0a0", "B": "#8fd0ff", "W": "#ffffff", "G": "#2a2433"})
    i.rect(6, 1, 9, 2, "G")
    i.circle(8, 8.5, 5.2, "P")
    for y in (5, 8, 11):
        i.rect(4, y, 11, y, "p")
    i.circle(8, 8.5, 2.2, "B")
    i.px(8, 8, "W")
    i.rect(6, 14, 9, 15, "G")
    i.save("lantern_vigil", "Lantern Vigil")
    # Iai Flash: a blade half-drawn from its sheath with a bright cut line.
    i = Icon({"S": "#2a2433", "W": "#f2ece0", "C": "#c42a3a", "Y": "#fff6c0"})
    i.line(1, 14, 7, 8, "S"); i.line(2, 14, 8, 8, "S")
    i.line(8, 8, 14, 2, "W")
    i.line(7, 9, 8, 10, "C")
    i.line(0, 6, 15, 6, "Y")
    i.save("iai_flash", "Iai Flash")


def main():
    more_icons()
    for name, pal, hannya in (("shun", BASE, False), ("shun_hannya", HANNYA, True)):
        c = Canvas(24, 32, pal)
        draw(c, hannya)
        c.save(name, "Shun Yamikiri, Hannya Unbound" if hannya else "Shun Yamikiri (spirit ferryman)", outline=True)
    sword(False)
    sword(True)
    sword_rest(False)
    sword_rest(True)
    icon()
    print("wrote shun.px, shun_hannya.px, cleaver*.px, icons/grave_cleaver.px")


if __name__ == "__main__":
    main()
