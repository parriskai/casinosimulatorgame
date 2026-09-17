#!/usr/bin/env python3

import argparse
import json
import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


def main():
    parser = argparse.ArgumentParser(
        description="Convert a TTF font into a PNG glyph atlas."
    )

    parser.add_argument("font", type=Path)
    parser.add_argument("-o", "--output", type=Path, default=Path("atlas.png"))
    parser.add_argument(
        "--size",
        type=int,
        default=48,
        help="Font size in pixels (default: 48)",
    )
    parser.add_argument(
        "--padding",
        type=int,
        default=4,
        help="Padding around each glyph (default: 4)",
    )
    parser.add_argument(
        "--columns",
        type=int,
        default=16,
        help="Number of columns in the atlas (default: 16)",
    )
    parser.add_argument(
        "--start",
        type=int,
        default=32,
        help="First Unicode codepoint (default: 32)",
    )
    parser.add_argument(
        "--end",
        type=int,
        default=126,
        help="Last Unicode codepoint (default: 126)",
    )

    args = parser.parse_args()

    font = ImageFont.truetype(str(args.font), args.size)

    chars = [
        chr(codepoint)
        for codepoint in range(args.start, args.end + 1)
    ]

    # Determine the size required by each glyph.
    glyphs = []

    for char in chars:
        # bbox is relative to the drawing origin.
        bbox = font.getbbox(char)

        if bbox is None:
            continue

        x0, y0, x1, y1 = bbox

        width = x1 - x0
        height = y1 - y0

        advance = font.getlength(char)

        glyphs.append({
            "char": char,
            "codepoint": ord(char),
            "bbox": (x0, y0, x1, y1),
            "width": width,
            "height": height,
            "advance": advance,
        })

    if not glyphs:
        raise RuntimeError("Font contains no glyphs in the requested range.")

    # Make every cell large enough for the largest glyph.
    cell_width = max(g["width"] for g in glyphs) + args.padding * 2
    cell_height = max(g["height"] for g in glyphs) + args.padding * 2

    columns = args.columns
    rows = math.ceil(len(glyphs) / columns)

    atlas_width = columns * cell_width
    atlas_height = rows * cell_height

    # RGBA is convenient for GPU texture use.
    atlas = Image.new("RGBA", (atlas_width, atlas_height), (0, 0, 0, 0))
    draw = ImageDraw.Draw(atlas)

    metadata = {
        "font": str(args.font),
        "font_size": args.size,
        "atlas_width": atlas_width,
        "atlas_height": atlas_height,
        "glyphs": {},
    }

    for index, glyph in enumerate(glyphs):
        char = glyph["char"]

        column = index % columns
        row = index // columns

        cell_x = column * cell_width
        cell_y = row * cell_height

        x0, y0, x1, y1 = glyph["bbox"]

        # Position the glyph inside its cell.
        #
        # Because Pillow's bbox can have negative offsets, compensate
        # for x0/y0 so the entire glyph fits inside the cell.
        draw_x = cell_x + args.padding - x0
        draw_y = cell_y + args.padding - y0

        # White glyph, transparent background.
        draw.text(
            (draw_x, draw_y),
            char,
            font=font,
            fill=(255, 255, 255, 255),
        )

        metadata["glyphs"][str(ord(char))] = {
            "char": char,

            # Pixel rectangle in the atlas.
            "x": cell_x,
            "y": cell_y,
            "width": cell_width,
            "height": cell_height,

            # Actual glyph dimensions.
            "glyph_width": glyph["width"],
            "glyph_height": glyph["height"],

            # Font metrics.
            "advance": glyph["advance"],

            # Useful if constructing quads yourself.
            "offset_x": x0,
            "offset_y": y0,

            # Normalized UV coordinates.
            "u0": cell_x / atlas_width,
            "v0": cell_y / atlas_height,
            "u1": (cell_x + cell_width) / atlas_width,
            "v1": (cell_y + cell_height) / atlas_height,
        }

    atlas.save(args.output)

    json_path = args.output.with_suffix(".json")

    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(metadata, f, indent=2, ensure_ascii=False)

    print(f"Atlas:    {args.output}")
    print(f"Metadata: {json_path}")
    print(f"Size:     {atlas_width}x{atlas_height}")
    print(f"Glyphs:   {len(glyphs)}")


if __name__ == "__main__":
    main()