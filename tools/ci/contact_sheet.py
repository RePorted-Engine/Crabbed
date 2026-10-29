#!/usr/bin/env python3
"""One image of every scene, labelled, to look at in one go: contact_sheet.py DIR [OUT]. Needs Pillow."""
import sys
from pathlib import Path

from PIL import Image, ImageDraw

folder = Path(sys.argv[1])
out = Path(sys.argv[2]) if len(sys.argv) > 2 else folder / "sheet.png"
shots = sorted(p for p in folder.glob("*.png") if p.name != out.name and not p.name.endswith(".diff.png"))
cell, cols = (640, 360), 3
rows = (len(shots) + cols - 1) // cols
sheet = Image.new("RGB", (cols * cell[0], rows * (cell[1] + 20)), "black")
draw = ImageDraw.Draw(sheet)
for i, path in enumerate(shots):
    x, y = (i % cols) * cell[0], (i // cols) * (cell[1] + 20)
    sheet.paste(Image.open(path).convert("RGB").resize(cell), (x, y + 20))
    draw.text((x + 6, y + 4), path.stem, fill="white")
sheet.save(out)
print(out)
