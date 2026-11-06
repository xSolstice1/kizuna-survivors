"""Generates the floor pickup icons (16x16) as .px sources in art/src/pickups/.

    python3 tools/gen_pickups.py
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_icons  # noqa: E402
from gen_icons import Icon  # noqa: E402

gen_icons.OUT = pathlib.Path(__file__).resolve().parent.parent / "art/src/pickups"


def main():
    gen_icons.OUT.mkdir(parents=True, exist_ok=True)
    # Onigiri Feast: two rice balls on a leaf.
    i = Icon({"W": "#ffffff", "w": "#d8d8e0", "N": "#2a3a2a", "G": "#5ac85a", "g": "#2a7a2a"})
    i.poly([(0.5, 13), (15.5, 13), (13, 15.5), (3, 15.5)], "G")
    i.line(2, 14, 13, 14, "g")
    for cx in (5, 11):
        i.poly([(cx, 3), (cx + 4, 11), (cx - 4, 11)], "W")
        i.circle(cx, 9, 3.4, "W")
        i.rect(cx - 2, 9, cx + 1, 12, "N")
    i.save("feast", "Onigiri Feast")
    # Swift Geta: a wooden sandal with wind lines.
    i = Icon({"B": "#c98a4a", "b": "#7a4a20", "R": "#e83a3a", "W": "#bff0ff"})
    i.rect(3, 6, 13, 9, "B")
    i.rect(4, 10, 6, 13, "b"); i.rect(10, 10, 12, 13, "b")
    i.line(5, 6, 8, 3, "R"); i.line(11, 6, 8, 3, "R")
    for y in (5, 8, 11):
        i.line(0, y, 2, y, "W")
    i.save("swift_geta", "Swift Geta")
    # Oni Sake: a red gourd.
    i = Icon({"R": "#e8413a", "r": "#a02a24", "Y": "#ffd27a", "B": "#5a3a2a"})
    i.circle(8, 11, 4.6, "R")
    i.circle(8, 5.5, 3.0, "R")
    i.circle(9.5, 12, 2.5, "r")
    i.rect(7, 7, 9, 8, "Y")
    i.rect(7, 1, 9, 2, "B")
    i.save("oni_sake", "Oni Sake")
    # Shrine Bell: a golden suzu bell with a red cord.
    i = Icon({"Y": "#ffd23a", "y": "#c9962a", "R": "#e83a3a", "K2": "#2a1a10"})
    i.line(8, 0, 8, 3, "R")
    i.circle(8, 9, 5.6, "Y")
    i.circle(9.5, 10.5, 3.0, "y")
    i.rect(4, 9, 12, 9, "y")
    i.rect(6, 12, 10, 12, "K")
    i.save("shrine_bell", "Shrine Bell")
    # Frozen Hourglass: a blue-ice hourglass.
    i = Icon({"F": "#bff0ff", "B": "#5ab8ff", "W": "#ffffff", "G": "#3a4a7a"})
    i.rect(3, 1, 12, 2, "G"); i.rect(3, 13, 12, 14, "G")
    i.poly([(4, 3), (12, 3), (8, 8)], "F")
    i.poly([(8, 8), (12, 12.5), (4, 12.5)], "B")
    i.px(7, 5, "W")
    i.save("hourglass", "Frozen Hourglass")
    # Exorcism Seal: a glowing violet talisman with a star.
    i = Icon({"P": "#f4ecff", "V": "#9a6aff", "v": "#5a3ac8", "Y": "#ffe066"})
    i.rect(4, 1, 11, 14, "P")
    i.rect(5, 2, 10, 13, "V")
    i.poly(gen_icons.star_points(8, 7.5, 3.6, 1.6), "Y")
    i.rect(6, 11, 9, 12, "v")
    i.save("exorcism_seal", "Exorcism Seal")
    # Sen Purse: a drawstring pouch with coins.
    i = Icon({"B": "#9a5a3a", "b": "#5a3420", "Y": "#ffd24a", "R": "#e83a3a"})
    i.circle(8, 10, 5.2, "B")
    i.circle(9.5, 11.5, 2.8, "b")
    i.poly([(5, 5), (11, 5), (9, 7), (7, 7)], "B")
    i.line(5, 6, 11, 6, "R")
    i.circle(3.5, 13.5, 2.0, "Y"); i.circle(12.5, 13.8, 1.8, "Y")
    i.save("sen_purse", "Sen Purse")
    print("wrote pickup icons")


if __name__ == "__main__":
    main()
