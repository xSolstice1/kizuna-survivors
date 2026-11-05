from PIL import Image
PAL = {
 '.': None, 'K': '#1a1423', 'H': '#e8609a', 'h': '#a83a72', 'L': '#ffb3d6',
 'S': '#ffe2cc', 's': '#f2b89a', 'r': '#ff9aa8', 'E': '#24163f', 'I': '#6a7cff',
 'W': '#ffffff', 'M': '#b04050', 'C': '#2f3b70', 'c': '#1d2550', 'R': '#e83a3a', 'B': '#4a2e2a',
}
HALF = [
 "........KKKK", "......KKHHHH", ".....KHHLLHH", "....KHHLLHHH", "...KHHLHHHHH",
 "...KHHHHHHHH", "..KHHHHHHHHH", "..KHHhHHHhHH", "..KHhSShhSSS", "..KHhSSSSSSS",
 "..KHSKKKSSSS", "..KHSWEISSSS", "..KHSEIISSSS", "..KHhSrrSSSS", "..KHhsSSSSsM",
 "..KHHhsSSSSS", "..KHhhKsSSSS", "...KhK..KKss", "...KhK.KCCRR", "...KhKKCCCRR",
 "....KKCCCCcR", "....KSKCCCCC", "....KSKCCCCC", "....KKCCCCCc", ".....KCCCCCC",
 "....KCCcCCCc", "....KKKKKKKK", "......KSSK..", "......KSSK..", "......KBBK..",
 ".....KBBBK..", ".....KKKKK..",
]
rows = [h + h[::-1] for h in HALF]
assert all(len(r) == 24 for r in rows), [len(r) for r in rows]
img = Image.new("RGBA", (24, 32), (0, 0, 0, 0))
for y, row in enumerate(rows):
    for x, ch in enumerate(row):
        if PAL[ch]: img.putpixel((x, y), tuple(int(PAL[ch][i:i+2], 16) for i in (1, 3, 5)) + (255,))
bg = Image.new("RGBA", (24 * 10 + 20, 32 * 10 + 20), "#3a5a40")
bg.alpha_composite(img.resize((240, 320), Image.NEAREST), (10, 10))
small = Image.new("RGBA", (24 * 3, 32 * 3), "#3a5a40"); small.alpha_composite(img.resize((72, 96), Image.NEAREST))
bg.alpha_composite(small, (bg.width - 72 - 4, bg.height - 96 - 4)) if False else None
bg.save("/tmp/arttest/heroine_x10.png")
img.save("/tmp/arttest/heroine.png")
