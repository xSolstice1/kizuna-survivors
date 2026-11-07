"""HD redraws: Toya (base + Hundred-Leg Rampage), Kuon (base + Endless Firmament), and the small
projectiles / pickups (familiar, talisman, shuriken, orb, gem, coin).

    python3 tools/hd/heroes_b.py

Coordinates are art pixels of the matching .px (pixel (x, y) covers [x, x+1] x [y, y+1]).
"""
import math
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from hdkit import Art  # noqa: E402


# ── helpers ──
def qb(p0, p1, p2, n=8):
    out = []
    for i in range(n + 1):
        t = i / n
        u = 1 - t
        out.append((u * u * p0[0] + 2 * u * t * p1[0] + t * t * p2[0],
                    u * u * p0[1] + 2 * u * t * p1[1] + t * t * p2[1]))
    return out


def spike(a, tip, b, bend=(0.0, 0.0)):
    """A curved pointed lock: from base a to the tip and back to base b."""
    m1 = ((a[0] + tip[0]) / 2 + bend[0], (a[1] + tip[1]) / 2 + bend[1])
    m2 = ((b[0] + tip[0]) / 2 + bend[0], (b[1] + tip[1]) / 2 + bend[1])
    return qb(a, m1, tip) + qb(tip, m2, b)[1:]


def star(cx, cy, r1, r2, n=4, rot=-math.pi / 2):
    pts = []
    for k in range(n * 2):
        r = r1 if k % 2 == 0 else r2
        a = rot + k * math.pi / n
        pts.append((cx + math.cos(a) * r, cy + math.sin(a) * r))
    return pts


def legs(a, trouser, trouser_dk, boot, boot_hi):
    for x0 in (7.0, 14.0):
        a.rect(x0, 25.0, x0 + 3.0, 29.4, ("lin", trouser, trouser_dk, 0), radius=0.5)
        a.rect(x0 + 1.9, 25.0, x0 + 3.0, 29.0, trouser_dk, alpha=0.6)
        # Boot with a toe nudged forward (facing right).
        a.blob([(x0 - 0.05, 28.7), (x0 + 3.0, 28.7), (x0 + 3.35, 30.2), (x0 + 3.0, 30.95),
                (x0 + 0.1, 30.95), (x0 - 0.15, 30.0)], ("lin", boot_hi, boot, 90))
        a.stroke([(x0 + 0.5, 29.2), (x0 + 1.8, 29.1)], 0.35, "#ffffff", alpha=0.18)


def face(a, skin, skin_dk, chin=15.2):
    a.blob([(7.0, 8.6), (17.0, 8.6), (17.3, 11.8), (16.0, 14.0), (12.5, chin), (9.0, 14.0), (6.9, 11.8)],
           ("lin", skin, skin, 90))
    # Soft shadow under the fringe and on the far cheek.
    a.rect(6.5, 8.0, 17.6, 9.9, skin_dk, alpha=0.45, clip=True)
    a.blob([(15.4, 9.0), (17.4, 9.0), (17.2, 12.5), (15.6, 14.2), (15.2, 12.0)], skin_dk, alpha=0.5, clip=True)


def blush(a, xs, y=13.0):
    for x in xs:
        a.ellipse(x, y, 0.9, 0.35, "#ff8a9a", alpha=0.35)


def anime_eye(a, cx, cy, iris_hi, iris_lo, lash="#1a0a14", pupil="#2a0010", glow=None, rx=1.25, ry=1.55):
    a.ellipse(cx, cy, rx + 0.15, ry, "#fff6f6")
    a.ellipse(cx, cy + 0.15, rx, ry - 0.05, ("lin", iris_lo, iris_hi, 90))
    a.ellipse(cx, cy + 0.3, rx * 0.5, ry * 0.55, pupil)
    a.ellipse(cx + 0.35, cy - 0.5, 0.42, 0.42, "#ffffff")
    a.ellipse(cx - 0.45, cy + 0.75, 0.22, 0.22, "#ffffff", alpha=0.8)
    # Upper lash line, thicker toward the outer corner.
    a.stroke([(cx - rx - 0.5, cy - ry + 0.55), (cx, cy - ry - 0.05), (cx + rx + 0.4, cy - ry + 0.35)], 0.65, lash)


# ── Toya ──
T = {
    "H": "#34303e", "Hl": "#4a4558", "h": "#1e1b26", "L": "#f0eef6",
    "S": "#f8ece6", "s": "#dcc4bc",
    "R": "#ff2a3a", "Rd": "#8a0818",
    "G": "#ece8de", "g": "#b8b2a6",
    "N": "#221f2a", "Nl": "#2e2a38", "n": "#0e0d13", "C": "#b01e30",
    "D": "#2c2a34", "Dd": "#1c1a22", "B": "#141218", "Bl": "#2a2632",
}


def toya(mukade):
    c = dict(T)
    if mukade:
        c.update({"h": "#7a0e1c", "L": "#ff6a6a", "R": "#ff4040"})
    a = Art(24, 32)
    hair = ("lin", c["Hl"], c["H"], 70)

    # Back hair: the big messy mass plus side flares.
    a.ellipse(12.5, 8.6, 8.0, 6.6, hair)
    a.poly(spike((4.5, 9.0), (0.6, 8.2), (5.0, 13.6), (-0.4, 1.2)), hair)
    a.poly(spike((20.0, 9.0), (23.4, 8.2), (19.8, 13.6), (0.4, 1.2)), hair)
    a.poly(spike((4.6, 11.0), (2.2, 13.6), (7.0, 13.4), (0.0, 0.4)), hair)
    a.poly(spike((17.5, 13.4), (21.8, 13.6), (19.8, 11.0), (0.0, 0.4)), hair)
    if mukade:
        a.poly(spike((4.5, 7.0), (0.6, 3.4), (5.5, 5.0), (0.6, 0.4)), hair)
        a.poly(spike((18.5, 5.0), (23.4, 3.4), (19.6, 7.4), (-0.6, 0.4)), hair)

    legs(a, c["D"], c["Dd"], c["B"], c["Bl"])

    # Long coat with sleeves.
    coat = ("lin", c["Nl"], c["N"], 80)
    a.blob([(6.3, 16.0), (17.7, 16.0), (18.6, 20.0), (18.9, 24.4), (18.4, 26.0), (5.6, 26.0), (5.1, 24.4), (5.4, 20.0)], coat)
    a.blob([(6.8, 16.0), (4.4, 17.2), (3.9, 21.0), (4.1, 23.4), (6.4, 23.3), (7.0, 19.0)], coat)
    a.blob([(17.2, 16.0), (19.6, 17.2), (20.1, 21.0), (19.9, 23.4), (17.6, 23.3), (17.0, 19.0)], c["N"])
    # Right half in shadow, the front edge lined crimson.
    a.poly([(12.4, 16.0), (19.0, 16.0), (21.0, 26.0), (12.4, 26.0)], c["n"], clip=True, alpha=0.85)
    a.stroke([(11.6, 17.0), (11.5, 21.5), (11.6, 25.8)], 0.9, c["C"])
    a.stroke([(5.9, 25.6), (18.1, 25.6)], 0.35, c["Nl"], alpha=0.5)
    # Sleeve fold lines and cuffs.
    a.stroke([(5.3, 18.0), (5.0, 22.0)], 0.3, "#4a4558", alpha=0.5)
    a.stroke([(4.2, 22.9), (6.3, 22.9)], 0.5, c["C"], alpha=0.8)
    a.stroke([(17.7, 22.9), (19.8, 22.9)], 0.5, c["C"], alpha=0.6)
    # Hands.
    a.ellipse(5.1, 23.9, 1.0, 0.75, c["S"])
    a.ellipse(18.9, 23.9, 1.0, 0.75, c["s"])

    # High collar, flared, crimson-edged.
    a.blob([(6.6, 14.4), (17.4, 14.4), (17.9, 17.0), (12.0, 17.6), (6.1, 17.0)], ("lin", c["Nl"], c["n"], 0))
    a.stroke([(7.2, 14.6), (7.0, 16.9)], 0.65, c["C"])
    a.stroke([(16.8, 14.6), (17.0, 16.9)], 0.65, c["C"])

    if mukade:
        plate = ("lin", c.get("Q", "#d43040"), "#7a1020", 90)
        # Shoulder pauldrons.
        a.blob([(3.8, 16.9), (5.0, 15.0), (8.6, 14.9), (8.8, 16.6), (6.0, 17.6)], plate)
        a.blob([(15.2, 16.6), (15.4, 14.9), (19.0, 15.0), (20.2, 16.9), (18.0, 17.6)], ("lin", "#b02838", "#5a0c18", 90))
        # Arm bands.
        for y in (18.4, 20.4, 22.4):
            a.rect(3.3, y, 6.4, y + 0.9, plate, radius=0.4)
            a.rect(17.6, y, 20.7, y + 0.9, ("lin", "#a82434", "#5a0c18", 90), radius=0.4)
        # Chest: overlapping centipede segments.
        for k, y in enumerate((17.8, 18.9, 20.0)):
            w = 3.6 - k * 0.6
            a.blob([(12.0 - w, y), (12.0 + w, y), (12.0 + w - 0.4, y + 1.0), (12.0 - w + 0.4, y + 1.0)],
                   ("lin", "#d43040", "#5a0c18", 90))
            a.stroke([(12.0 - w + 0.5, y + 0.25), (12.0 + w - 0.5, y + 0.25)], 0.2, "#ff8a8a", alpha=0.6)
        # Shin plates (inside the leg columns).
        for x0 in (7.0, 14.0):
            a.rect(x0 + 0.1, 26.0, x0 + 2.9, 26.9, "#d43040", radius=0.35)
            a.rect(x0 + 0.1, 26.9, x0 + 2.9, 28.9, ("lin", "#7a1020", "#3a0610", 90), radius=0.35)
            a.stroke([(x0 + 0.5, 27.6), (x0 + 2.4, 27.6)], 0.25, "#d43040", alpha=0.7)

    # Face.
    face(a, c["S"], c["s"])
    # Cursed crimson eye (viewer's left).
    anime_eye(a, 9.6, 11.4, c["R"], c["Rd"], pupil="#3a0008")
    if mukade:
        # Blazing: the iris spills over in a flaring streak.
        a.stroke([(8.0, 12.5), (7.2, 13.4)], 0.5, c["R"], taper=True)
        a.stroke([(11.1, 10.0), (12.2, 9.0)], 0.45, c["R"], taper=True)
        a.ellipse(9.6, 11.6, 0.55, 0.55, "#ffd0d0")
    # Bandage across the other eye, wrapping off over the hair.
    band = ("lin", c["G"], c["g"], 90)
    a.poly([(12.7, 9.9), (17.8, 9.2), (17.9, 11.4), (12.8, 12.4)], band)
    a.stroke([(13.4, 10.5), (17.2, 10.0)], 0.25, c["g"], alpha=0.8)
    a.stroke([(14.6, 9.7), (15.6, 11.9)], 0.25, c["g"], alpha=0.7)
    a.poly([(16.8, 11.2), (18.0, 10.9), (18.4, 13.0), (17.6, 13.4)], band)
    if not mukade:
        a.stroke([(11.6, 13.7), (12.9, 13.6)], 0.35, "#a0706a")
        blush(a, [9.2], 13.1)
    else:
        # Mandible jaw-guard: two chitin pincers closing over the lower face, fangs at the tips.
        a.blob([(7.4, 12.6), (9.4, 12.9), (11.6, 13.6), (11.9, 14.6), (10.0, 15.0), (8.2, 14.6), (7.2, 13.6)],
               ("lin", "#c02838", "#4a0810", 90))
        a.blob([(16.6, 12.6), (14.6, 12.9), (12.4, 13.6), (12.1, 14.6), (14.0, 15.0), (15.8, 14.6), (16.8, 13.6)],
               ("lin", "#9a1c2c", "#3a0608", 90))
        a.stroke([(8.0, 13.2), (10.6, 13.7)], 0.25, "#ff7a80", alpha=0.7)
        for x, d in ((11.4, -1), (12.6, 1)):
            a.poly([(x - 0.45, 14.2), (x + 0.45, 14.2), (x + 0.1 * d, 15.6)], "#e8dccb")
        for x in (8.4, 15.6):
            a.poly([(x - 0.5, 14.3), (x + 0.5, 14.3), (x, 15.7)], "#e8dccb")

    # Front hair: a jagged fringe and the top spikes.
    for tip, base_l, base_r, bend in (
        ((3.6, 0.2), (3.6, 6.0), (8.0, 4.0), (-0.6, 0.4)),
        ((10.4, -0.1), (7.6, 4.2), (12.4, 3.2), (-0.2, 0.0)),
        ((14.6, 0.5), (12.4, 3.6), (16.4, 3.6), (0.2, 0.0)),
        ((20.6, 1.5), (16.2, 4.6), (20.4, 7.0), (0.6, 0.4)),
    ):
        a.poly(spike(base_l, tip, base_r, bend), hair)
    for x0, tip_y, lean in ((6.4, 10.4, -0.5), (8.4, 9.8, -0.4), (10.4, 10.0, -0.2), (12.4, 9.6, 0.1),
                            (14.4, 9.8, 0.3), (16.4, 10.6, 0.5)):
        a.poly(spike((x0 - 1.4, 6.0), (x0 + lean, tip_y), (x0 + 1.4, 6.0)), hair)
    a.poly(spike((5.0, 8.0), (5.8, 13.4), (7.2, 8.0), (-0.3, 0.0)), hair)
    a.poly(spike((16.8, 8.0), (18.0, 13.2), (19.2, 8.0), (0.3, 0.0)), hair)
    # Hair shading: strand shadows and a soft sheen.
    for pts in (((10.6, 4.0), (11.0, 6.4)), ((13.6, 3.8), (13.4, 6.2)), ((17.4, 5.0), (18.4, 8.0)), ((6.4, 6.0), (5.8, 8.4))):
        a.stroke(list(pts), 0.4, c["h"], alpha=0.7, taper=True)
    a.stroke([(6.0, 4.0), (9.0, 2.6), (12.0, 2.2)], 0.6, "#6a6478", alpha=0.6)
    # The white streak over the left of the fringe.
    a.stroke([(8.0, 3.6), (8.6, 6.0), (9.2, 8.2), (9.4, 9.6)], 0.9, c["L"], taper=True)
    if mukade:
        for tip in ((3.6, 0.2), (10.4, -0.1), (14.6, 0.5), (20.6, 1.5), (0.6, 3.4), (23.4, 3.4), (0.6, 8.2), (23.4, 8.2)):
            a.ellipse(tip[0], tip[1] + 0.9, 1.3, 1.5, "#c01828", clip=True, alpha=0.9)

    a.rim("#ffffff", 0.22)
    a.outline()
    if mukade:
        a.glow("#ff2030", 1.2, 0.25)
    a.save("toya_mukade" if mukade else "toya")


# ── Kuon ──
K = {
    "H": "#dcd4f4", "Hl": "#f4f0ff", "h": "#9a90c4", "L": "#ffffff",
    "S": "#ffe8da", "s": "#e8c4b0",
    "V": "#f6f2ff", "v": "#b8bce6", "Y": "#ffd23a",
    "N": "#1e2850", "Nl": "#2c3868", "n": "#10162e", "W": "#e8ecff", "P": "#6a4ac8", "Pl": "#9a7af0",
    "D": "#242a48", "Dd": "#181c34", "B": "#141830", "Bl": "#2a3050",
    "C": "#7af0ff",
}


def kuon(domain):
    c = dict(K)
    if domain:
        c.update({"H": "#e8eeff", "Hl": "#ffffff", "h": "#8ab8ff", "N": "#1a2250", "Nl": "#26326a", "C": "#9af8ff", "Y": "#4ac8ff"})
    a = Art(24, 32)
    hair = ("lin", c["Hl"], c["H"], 80)

    # Back hair mass.
    a.ellipse(12.0, 8.4, 7.6, 6.4, hair)
    a.blob([(4.2, 8.0), (5.5, 13.4), (8.0, 13.6), (7.5, 8.0)], hair)
    a.blob([(16.5, 8.0), (16.0, 13.6), (18.5, 13.4), (19.8, 8.0)], hair)

    legs(a, c["D"], c["Dd"], c["B"], c["Bl"])

    # Onmyoji coat: long body, wide sleeves.
    coat = ("lin", c["Nl"], c["N"], 80)
    a.blob([(6.0, 15.8), (18.0, 15.8), (18.6, 20.0), (18.8, 25.0), (18.0, 27.0), (6.0, 27.0), (5.2, 25.0), (5.4, 20.0)], coat)
    a.blob([(7.0, 15.8), (3.6, 17.0), (2.9, 21.0), (3.2, 24.0), (6.6, 24.0), (7.2, 19.0)], coat)
    a.blob([(17.0, 15.8), (20.4, 17.0), (21.1, 21.0), (20.8, 24.0), (17.4, 24.0), (16.8, 19.0)], c["N"])
    a.poly([(14.6, 15.0), (22.0, 15.0), (22.0, 27.5), (14.6, 27.5)], c["n"], clip=True, alpha=0.75)
    # Sleeve hems.
    a.stroke([(3.2, 23.5), (6.6, 23.5)], 0.45, c["W"], alpha=0.7)
    a.stroke([(17.4, 23.5), (20.8, 23.5)], 0.45, c["W"], alpha=0.45)
    a.stroke([(4.6, 18.0), (4.2, 22.0)], 0.3, "#4a5890", alpha=0.6)
    a.ellipse(5.0, 24.6, 1.0, 0.75, c["S"])
    a.ellipse(19.0, 24.6, 1.0, 0.75, c["s"])
    # White inner collar (V) and the violet sash.
    a.poly([(9.8, 15.0), (14.2, 15.0), (13.6, 18.4), (12.0, 19.4), (10.4, 18.4)], ("lin", "#ffffff", c["W"], 90))
    a.stroke([(12.0, 16.4), (12.0, 19.0)], 0.3, "#b8bce6", alpha=0.8)
    a.rect(5.6, 21.0, 18.6, 22.2, ("lin", c["Pl"], c["P"], 90), radius=0.3)
    a.poly([(12.6, 22.0), (13.8, 22.0), (14.2, 24.6), (13.0, 24.2)], c["P"])
    a.stroke([(5.8, 26.6), (18.2, 26.6)], 0.35, "#4a5890", alpha=0.5)
    # Star flecks.
    for x, y, r in ((8.5, 19.5, 0.75), (15.5, 18.5, 0.7), (7.5, 24.5, 0.7), (11.5, 24.0, 0.65), (16.5, 25.5, 0.6),
                    (4.5, 20.5, 0.7), (19.5, 21.6, 0.6)):
        a.poly(star(x, y, r, r * 0.35), c["Y"])
    if domain:
        for x in (8.5, 15.5):
            a.stroke([(x, 22.4), (x, 26.6)], 0.5, "#4ac8ff")
            a.stroke([(x, 22.4), (x, 26.6)], 0.18, "#e0faff")

    face(a, c["S"], c["s"], chin=15.0)
    if not domain:
        # The paper veil across the eyes, with a little gold star.
        a.blob([(5.2, 9.2), (18.8, 9.2), (19.0, 10.6), (18.6, 12.0), (12.0, 12.2), (5.4, 12.0), (5.0, 10.6)],
               ("lin", "#ffffff", c["V"], 90), smooth=False)
        a.stroke([(5.6, 11.6), (18.4, 11.6)], 0.4, c["v"], alpha=0.9)
        a.poly(star(12.0, 10.4, 1.3, 0.45), c["Y"])
        a.ellipse(12.0, 10.4, 0.25, 0.25, "#fff4c0")
        for x in (7.2, 16.8):
            a.ellipse(x, 10.4, 0.25, 0.25, "#d8c8ff", alpha=0.9)
        a.poly([(18.0, 11.4), (19.4, 11.6), (19.2, 13.4)], c["v"])
        a.stroke([(11.6, 13.7), (12.6, 13.75), (13.4, 13.4)], 0.32, "#a0706a")  # the smirk
    else:
        for ex in (9.4, 14.6):
            anime_eye(a, ex, 11.3, "#eaffff", "#2a8aff", lash="#1a2050", pupil="#0e3a8a", rx=1.2, ry=1.45)
            a.poly(star(ex, 11.6, 0.6, 0.2), "#ffffff")
        a.stroke([(11.6, 13.7), (12.6, 13.75), (13.4, 13.4)], 0.32, "#a0706a")
        blush(a, [8.6, 15.4], 13.0)

    # Front hair: swept up into three points, a loose fringe.
    for tip, bl, br, bend in (
        ((6.4, 0.1), (4.8, 5.0), (10.2, 3.6), (-0.6, 0.2)),
        ((12.4, -0.1), (9.8, 3.6), (15.0, 3.4), (0.0, 0.0)),
        ((18.6, 1.1), (14.4, 3.8), (19.4, 5.4), (0.4, 0.2)),
    ):
        a.poly(spike(bl, tip, br, bend), hair)
    a.blob([(5.0, 6.0), (8.0, 4.4), (16.0, 4.4), (19.0, 6.0), (17.0, 8.6), (12.0, 7.8), (7.0, 8.6)], hair)
    a.poly(spike((6.2, 7.0), (7.0, 9.6), (8.6, 7.4), (-0.2, 0.0)), hair)
    a.poly(spike((15.4, 7.4), (17.0, 9.6), (17.8, 7.0), (0.2, 0.0)), hair)
    a.poly(spike((10.0, 7.6), (10.8, 9.2), (12.4, 7.8), (0.0, 0.0)), hair)
    a.poly(spike((3.8, 8.0), (4.4, 13.0), (6.2, 8.0), (-0.3, 0.0)), hair)
    a.poly(spike((17.8, 8.0), (19.6, 13.0), (20.2, 8.0), (0.3, 0.0)), hair)
    # Strand shading and a bright sheen.
    for pts in (((8.6, 3.0), (9.6, 6.4)), ((14.6, 2.8), (14.0, 6.4)), ((17.6, 4.2), (18.6, 7.6)), ((5.6, 6.0), (5.2, 9.0))):
        a.stroke(list(pts), 0.45, c["h"], alpha=0.8, taper=True)
    a.stroke([(16.2, 7.8), (17.2, 8.4), (18.2, 11.6)], 0.5, c["h"], alpha=0.6, taper=True)
    a.stroke([(8.4, 3.0), (10.0, 2.0), (11.6, 2.0)], 0.7, c["L"], alpha=0.9)
    if domain:
        a.stroke([(5.0, 6.4), (4.6, 10.0)], 0.4, "#4ac8ff", alpha=0.6, taper=True)

    a.rim("#ffffff", 0.22)
    a.outline()
    if domain:
        a.glow("#4ac8ff", 1.3, 0.3)
    a.save("kuon_domain" if domain else "kuon")


# ── small sprites ──
def familiar():
    a = Art(12, 10)
    fur = ("lin", "#ffb84a", "#f07a18", 90)
    # Tail behind, curling up to the right with a golden tip.
    a.blob([(7.0, 5.6), (8.6, 3.6), (10.2, 2.4), (11.2, 3.0), (11.1, 5.2), (9.6, 6.8), (7.6, 7.0)], fur)
    a.blob([(9.6, 2.5), (10.6, 2.0), (11.25, 3.0), (11.2, 5.0), (10.0, 5.4), (9.4, 3.8)], ("lin", "#fff0b0", "#ffd36a", 90), clip=True)
    # Head with two ears.
    a.blob([(1.2, 3.4), (1.4, 1.2), (3.6, 2.4), (5.8, 2.4), (7.6, 1.2), (8.0, 3.6), (8.0, 6.0), (6.4, 7.8),
            (3.0, 7.8), (1.1, 6.0)], fur)
    a.poly([(1.9, 2.0), (2.9, 2.6), (2.0, 3.2)], "#ffd0a0", alpha=0.9)
    a.poly([(7.1, 2.0), (6.1, 2.6), (7.0, 3.2)], "#ffd0a0", alpha=0.9)
    a.blob([(6.0, 2.8), (8.0, 3.4), (8.0, 6.0), (6.6, 7.6)], "#d4560e", clip=True, alpha=0.4)
    # White muzzle and eyes.
    a.blob([(2.8, 6.0), (4.5, 5.6), (6.2, 6.0), (6.4, 7.0), (4.5, 7.8), (2.6, 7.0)], "#fff6dc")
    a.ellipse(3.5, 5.1, 0.55, 0.65, "#2a0a00")
    a.ellipse(6.0, 5.1, 0.55, 0.65, "#2a0a00")
    a.ellipse(3.65, 4.85, 0.2, 0.2, "#ffffff")
    a.ellipse(6.15, 4.85, 0.2, 0.2, "#ffffff")
    a.ellipse(4.75, 6.3, 0.3, 0.22, "#3a1408")
    a.rim("#ffffff", 0.3)
    a.outline()
    a.glow("#ffb050", 0.8, 0.3)
    a.save("familiar")


def talisman():
    a = Art(6, 10)
    a.rect(0.9, 0.9, 5.1, 9.1, ("lin", "#fffaf0", "#f0e2c4", 90), radius=0.3)
    # Brushed red seal strokes.
    a.stroke([(2.2, 2.4), (3.8, 2.4)], 0.75, "#e83a3a")
    a.stroke([(3.0, 2.6), (3.0, 6.6)], 0.7, "#e83a3a")
    a.stroke([(2.0, 4.6), (4.0, 4.6)], 0.6, "#e83a3a")
    a.stroke([(2.1, 6.4), (3.0, 5.6), (3.9, 6.4)], 0.55, "#e83a3a")
    a.ellipse(3.0, 7.9, 0.75, 0.6, "#c42020")
    a.rect(0.9, 0.9, 5.1, 1.5, "#d0b890", alpha=0.6, clip=True)
    a.rect(0.9, 8.5, 5.1, 9.1, "#d0b890", alpha=0.6, clip=True)
    a.outline(width=0.8)
    a.save("talisman")


def shuriken():
    a = Art(16, 16)
    cx, cy = 8.2, 7.8
    steel = ("lin", "#eef2fa", "#8a92a8", 45)
    tips = [(3.0, 1.6), (14.6, 2.6), (12.6, 14.4), (1.4, 12.6)]
    for i, (tx, ty) in enumerate(tips):
        ang = math.atan2(ty - cy, tx - cx)
        # Curved blade: leading edge straight-ish, trailing edge swept.
        perp = (math.cos(ang + math.pi / 2), math.sin(ang + math.pi / 2))
        b1 = (cx + perp[0] * 1.6, cy + perp[1] * 1.6)
        b2 = (cx - perp[0] * 1.6, cy - perp[1] * 1.6)
        mid = ((cx + tx) / 2, (cy + ty) / 2)
        c1 = (mid[0] + perp[0] * 1.9, mid[1] + perp[1] * 1.9)
        c2 = (mid[0] - perp[0] * 0.3, mid[1] - perp[1] * 0.3)
        pts = qb(b1, c1, (tx, ty)) + qb((tx, ty), c2, b2)[1:]
        a.poly(pts, steel)
        # Edge highlight and a dark spine.
        a.stroke(qb(b1, c1, (tx, ty), 6)[1:], 0.35, "#ffffff", alpha=0.7, smooth=False)
        a.stroke([(cx, cy), (mid[0], mid[1]), (tx, ty)], 0.35, "#5a6278", alpha=0.6, taper=True, smooth=False)
    a.ellipse(cx, cy, 2.4, 2.4, ("rad", "#ff5a5a", "#a01828"))
    a.ellipse(cx - 0.6, cy - 0.7, 0.55, 0.55, "#ffffff", alpha=0.85)
    a.ellipse(cx, cy, 0.8, 0.8, "#2a0a12")
    a.outline()
    a.save("shuriken")


def orb():
    a = Art(8, 8)
    a.ellipse(4.0, 4.0, 3.05, 3.05, ("rad", "#fffbe8", "#d9b866", (0.38, 0.35)))
    a.blob([(6.8, 3.0), (6.6, 5.6), (4.6, 7.0), (6.2, 5.0)], "#c09a4a", clip=True, alpha=0.45)
    a.ellipse(3.0, 2.8, 0.9, 0.7, "#ffffff", alpha=0.95)
    a.ellipse(4.8, 5.0, 0.45, 0.4, "#c8a858", alpha=0.5)
    a.outline(width=0.75)
    a.save("orb")


def gem():
    a = Art(7, 7)
    top, l, r, bot = (3.5, 0.75), (0.75, 3.6), (6.25, 3.6), (3.5, 6.25)
    a.poly([top, r, bot, l], "#4fd1ff")
    a.poly([top, (3.5, 3.6), l], "#c8f6ff")       # lit upper-left facet
    a.poly([top, r, (3.5, 3.6)], "#4fd1ff")
    a.poly([l, (3.5, 3.6), bot], "#38b0e8")
    a.poly([(3.5, 3.6), r, bot], "#2a8fc4")
    a.ellipse(2.6, 2.6, 0.45, 0.45, "#ffffff")
    a.outline(width=0.7)
    a.save("gem")


def coin():
    a = Art(10, 10)
    a.ellipse(5.0, 5.0, 4.1, 4.1, ("lin", "#ffe68a", "#c8962a", 60))
    a.ellipse(5.0, 5.0, 3.0, 3.0, ("lin", "#ffd24a", "#e0a830", 60))
    # Cut the square hole.
    from PIL import ImageDraw
    from hdkit import SS
    alpha = a.img.getchannel("A")
    ImageDraw.Draw(alpha).rectangle([3.35 * SS, 3.35 * SS, 6.65 * SS, 6.65 * SS], fill=0)
    a.img.putalpha(alpha)
    a.ellipse(3.0, 2.4, 0.9, 0.5, "#fff8d0", alpha=0.9)
    a.outline("#3a2a10", width=0.75)
    a.save("coin")


def main():
    toya(False)
    toya(True)
    kuon(False)
    kuon(True)
    familiar()
    talisman()
    shuriken()
    orb()
    gem()
    coin()
    print("wrote 10 sprites")


if __name__ == "__main__":
    main()
