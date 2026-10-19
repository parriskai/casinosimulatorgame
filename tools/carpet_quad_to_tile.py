import json
import math
import os
from pathlib import Path

from PIL import Image

ROOT = Path(".")
INPUT = ROOT / "temp-assets" / "carpet-temp-quads.png"
OUTPUT_DIR = ROOT / "generated-tiles"
OUTPUT_JSON = OUTPUT_DIR / "carpet.json"

original: Image.Image  = Image.open(INPUT)

dirs = ["n", "ne", "e", "se", "s", "sw", "w", "nw"]

def get_bit(number: int, bit: int) -> bool:
    return number & (1 << bit) != 0

quads = []
for y in range(2):
    for x in range(4):
        quads.append(original.crop((x * 32, y * 32, x * 32 + 32, y * 32 + 32)))
quad_table = {tuple(get_bit(quad_index, bit) for bit in range(3)): quad for (quad_index, quad) in enumerate(quads)}

files = []
for adjacent_index in range(256):
    bits = [get_bit(adjacent_index, bit) for bit in range(8)]
    suffix = [dirs[bit_index] for (bit_index, bit_set) in enumerate(bits) if bit_set]
    joined_suffix = "_".join(suffix) if suffix else "none"
    name = "carpet_" + joined_suffix

    file = OUTPUT_DIR / (name + ".png")

    output = Image.new("RGBA", (64, 64), (0,0,0,0))

    for (trans, idx, pos) in [(None, [0, 1, 2], (32, 0)), (Image.Transpose.ROTATE_270, [2, 3, 4], (32, 32)), (Image.Transpose.ROTATE_180, [4, 5, 6], (0, 32)), (Image.Transpose.ROTATE_90, [6, 7, 0], (0, 0))]:
        im = quad_table[tuple(not bits[id] for id in idx)]
        if trans is not None:
            im = im.transpose(trans)
        output.paste(im, pos)
    
    output.save(file)
    files.append({"file": str(file)})

with open(OUTPUT_JSON, "wt") as output_json:
    json.dump({"size": [64, 64], "tiles": {"carpet":files}, "include": []}, output_json, indent=4)