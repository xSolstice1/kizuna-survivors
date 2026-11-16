"""What every gear base looks like, at each of its three tiers (shared by icons and worn art).

Bases are listed in the same order as `Slot::bases()` in crates/sim/src/gear.rs; files are named
`<slot>/<base_key>_<tier>` with `base_key` = the name lowercased, runs of non-letters as `_`
(`gear::base_key` in the game does the same). Tier 0 = Common/Uncommon, 1 = Rare/Epic,
2 = Legendary/Mythic/Exotic/Set.
"""
import re

# slot: [(name, key colour for icons, description of the object)]
BASES = {
    "hat": [
        ("Straw Kasa", "magenta", "a wide shallow conical woven straw kasa hat with a chin cord"),
        ("Silk Eboshi", "green", "a tall black lacquered silk eboshi court cap, folded at the tip, tied on with a cord"),
        ("Fox-Ear Hood", "green", "a soft cloth hood with two tall pointed fox ears on top, white inside the ears"),
        ("Moon Circlet", "green", "a slim metal circlet worn across the forehead with a crescent-moon jewel at the front"),
        ("Tengu Cap", "green", "a small black hexagonal tokin cap tied on the forehead with a cord, with a row of round pompoms down the front"),
    ],
    "top": [
        ("Haori", "green", "a hip-length haori half-coat with wide sleeves, worn open over a plain undershirt, a crest-like round emblem on each breast"),
        ("Travel Kimono", "green", "a short travel kimono with its sleeves tied back by a cord and a broad obi sash at the waist"),
        ("Lacquer Cuirass", "green", "a lacquered chest armour of overlapping plates laced together with silk cord, over a kimono with fitted sleeves"),
        ("Festival Happi", "green", "a festival happi jacket with elbow-length wide sleeves, open at the front, a wave pattern along the hem and collar"),
        ("Shrine Vestment", "green", "a white priest's robe with very wide sleeves edged by a coloured sleeve cord, a coloured collar band"),
    ],
    "bottom": [
        ("Hakama", "green", "wide pleated hakama trousers cut at the shins, the two legs clearly separate, a tie at the waist"),
        ("Wrapped Leggings", "green", "fitted trousers with cloth kyahan wraps bound around the shins"),
        ("Pleated Skirt", "green", "a short pleated skirt with a waistband"),
        ("Kobakama", "green", "short pleated kobakama shorts that end above the knee, each leg separate"),
    ],
    "gloves": [
        ("Tekko", "green", "cloth tekko hand guards over the backs of the hands with a small metal plate, fingers bare"),
        ("Archer's Glove", "green", "a leather archer's glove (yugake) with a stiff thumb and a wrist strap"),
        ("Ink-Stained Gloves", "magenta", "fingerless white cloth gloves splattered with black calligraphy ink"),
        ("Iron Bracers", "green", "iron bracers over the forearms and the backs of the hands, riveted, with cord lacing"),
    ],
    "shoes": [
        ("Geta", "green", "wooden geta sandals on two tall teeth with a cloth thong, over white tabi socks"),
        ("Waraji", "magenta", "woven straw waraji sandals tied around the ankle with cord, over white tabi socks"),
        ("Tabi Boots", "green", "ankle-high split-toe jika-tabi boots with rubber soles and clasps up the side"),
        ("Cloud Sandals", "green", "light sandals whose soles are puffy swirling cloud shapes, with ribbon ties"),
    ],
    "necklace": [
        ("Prayer Beads", "green", "a loop of round prayer beads with one larger bead and a short tassel"),
        ("Magatama Cord", "green", "a curved comma-shaped magatama jewel hanging on a braided cord"),
        ("Bell Choker", "green", "a ribbon choker with a small round suzu bell at the front"),
        ("Lantern Pendant", "green", "a tiny glowing paper lantern pendant on a thin chain"),
    ],
    "ring": [
        ("Jade Ring", "magenta", "a thick ring carved from a single piece of jade"),
        ("Moonstone Band", "green", "a slim band set with a milky glowing moonstone"),
        ("Cinder Ring", "green", "a blackened ring set with a smouldering ember-red gem"),
        ("Spirit Loop", "green", "a twisted ring with a tiny blue spirit wisp flame floating above it"),
    ],
    "earring": [
        ("Tassel Earring", "green", "a single dangling earring ending in a silk tassel"),
        ("Coral Drop", "green", "a single drop earring with a polished coral bead"),
        ("Star Stud", "green", "a single small star-shaped stud earring"),
        ("Feather Earring", "green", "a single dangling earring with a long feather"),
    ],
    "charm": [
        ("Daruma Charm", "green", "a small round daruma doll charm on a cord, one eye painted in"),
        ("Folded Crane", "green", "a folded origami paper crane charm on a cord"),
        ("Lucky Cat", "green", "a small beckoning lucky cat figurine charm with one paw raised, on a cord"),
        ("Paper Lantern Charm", "green", "a small round paper lantern charm on a cord"),
    ],
    "omamori": [
        ("Health Omamori", "magenta", "a small green silk omamori charm pouch with a knotted cord at the top"),
        ("Victory Omamori", "green", "a small red silk omamori charm pouch with a knotted cord at the top"),
        ("Travel Omamori", "green", "a small blue silk omamori charm pouch with a knotted cord at the top"),
        ("Love Omamori", "green", "a small pink silk omamori charm pouch with a knotted cord at the top"),
    ],
}

# Each tier keeps the base's shape and main colour and changes the materials and finish.
TIERS = [
    "a humble everyday version: simple plain materials, muted colours, no decoration",
    "a fine-quality version: rich deep colours, polished silver trim and a subtle woven pattern",
    "a legendary version: lavish gold embroidery, gilded metal fittings, small glowing jewels and a faint shimmering spirit glow",
]

# Slots drawn as part of the hero (outfit edits); the rest hang on the rig as small sprites.
WORN = ["hat", "top", "bottom", "gloves", "shoes"]
# Which slots each outfit edit changes (one edit per base index per tier).
GROUPS = {"a": ["hat", "top", "gloves"], "b": ["bottom", "shoes"]}


def key(name):
    return re.sub(r"[^a-z]+", "_", name.lower()).strip("_")


def items():
    """(slot, base index, base key, tier, key colour, description) for every icon."""
    for slot, bases in BASES.items():
        for i, (name, bg, desc) in enumerate(bases):
            for t, tier in enumerate(TIERS):
                yield slot, i, key(name), t, bg, f"{desc}; {tier}"

# Hand-picked overrides of the anchors gear_art.anchors() derives from each rig (design-image
# pixels, 1024 canvas): the base of the neck (necklace), the visible earlobe (earring), the
# front of the hip (charms, omamori), the near hand's knuckles (ring), the far shoulder (back,
# Arsenal). A new hero needs none; add one only if a derived point looks off.
ANCHORS = {
    "ren": {"neck": [569, 544], "ear": [459, 451], "hip": [645, 711], "hand": [382, 737]},
    "riku": {"neck": [552, 584], "ear": [412, 424], "hip": [632, 711], "hand": [385, 751]},
    "shun": {"neck": [559, 544], "ear": [385, 424], "hip": [619, 777], "hand": [352, 757]},
    "toya": {"neck": [582, 537], "ear": [365, 424], "hip": [632, 777], "hand": [365, 757]},
    "kuon": {"neck": [555, 527], "ear": [372, 404], "hip": [505, 704], "hand": [359, 757]},
}

# Exotics in worn slots: their own look on the body (one edit per hero, tag "x0_2"), named
# `<slot>_<exotic key>` instead of a base and tier.
EXOTIC_WORN = {
    "hat": ("yata_mirror", "a small round mirror with a polished gold rim and studs, worn at the front of the head on a thin gold headband"),
    "top": ("white_haori", "a white haori half-coat worn open over a black inner kimono, white diamond marks along the hem, no writing"),
    "bottom": ("hunger_organ", "dark crimson hakama trousers cut at the shins, the two legs separate, a faceted glowing red crystal heart fixed at the front of the waist with short fleshy red tendrils curling from it"),
}
