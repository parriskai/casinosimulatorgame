#!/usr/bin/env python3

import json
import math
import os
from pathlib import Path

from PIL import Image


ROOT = Path(".")
INPUT = ROOT / "tileset.json"
OUTPUT_DIR = ROOT / "generated-assets"
OUTPUT_JSON = OUTPUT_DIR / "tileset.json"
OUTPUT_PNG = OUTPUT_DIR / "tileatlas.png"


def load_tileset(path):
    with path.open("r", encoding="utf-8") as f:
        jsn = json.load(f)
    
    for inc in jsn.get("include", []):
        jsn2 = load_tileset(path.parent.joinpath(inc))
        if jsn2["size"] != jsn["size"]:
            raise RuntimeError("Size mismatch")
        jsn["tiles"].update(jsn2["tiles"])
    print(jsn)
    return jsn


def get_tile_image(tile):
    path = tile.get("file")

    if path == "$NONE":
        return None

    if not path:
        raise ValueError("Tile is missing 'file'")

    image = Image.open(ROOT / path).convert("RGBA")

    rect = tile.get("rect")
    if rect is not None:
        if len(rect) != 4:
            raise ValueError(f"Invalid rect: {rect}")

        x, y, w, h = rect
        image = image.crop((x, y, x + w, y + h))

    return image


def fit_image(image, tile_w, tile_h):
    """
    Scale image down to fit inside tile_w x tile_h,
    preserving aspect ratio.

    The image is NOT scaled up.
    """
    w, h = image.size

    if w <= tile_w and h <= tile_h:
        return image

    scale = min(tile_w / w, tile_h / h)

    new_size = (
        max(1, round(w * scale)),
        max(1, round(h * scale)),
    )

    return image.resize(new_size, Image.Resampling.LANCZOS)


def clamp_copyrect(copyrect, tile_w, tile_h):
    """
    copyrect is [x, y, w, h].
    Clamp it to the tile dimensions.
    """
    if len(copyrect) != 4:
        raise ValueError(f"Invalid copyrect: {copyrect}")

    x, y, w, h = copyrect

    x = max(0, min(x, tile_w))
    y = max(0, min(y, tile_h))
    w = max(0, min(w, tile_w - x))
    h = max(0, min(h, tile_h - y))

    return x, y, w, h


def main():
    data = load_tileset(INPUT)

    tile_size = data["size"]
    tile_w, tile_h = tile_size

    if tile_w <= 0 or tile_h <= 0:
        raise ValueError("Tile size must be positive")

    tiles = data["tiles"]

    tile_count = sum(1 if isinstance(tile, dict) else len(tile) for tile in tiles.values())

    if tile_count == 0:
        raise ValueError("No tiles found")

    # Make the atlas roughly square, while accounting for
    # non-square tile dimensions.
    columns = max(
        1,
        math.ceil(math.sqrt(tile_count * tile_h / tile_w))
    )
    rows = math.ceil(tile_count / columns)

    atlas_w = columns * tile_w
    atlas_h = rows * tile_h

    atlas = Image.new("RGBA", (atlas_w, atlas_h), (0, 0, 0, 0))

    output_tiles = {}

    index = 0
    for (name, tile) in tiles.items():
        if isinstance(tile, dict):
            tile = [tile]
        
        output = []
        for frame in tile:
            column = index % columns
            row = index // columns
            index += 1

            slot_x = column * tile_w
            slot_y = row * tile_h

            output.append({
                "u0": slot_x / atlas_w,
                "v0": slot_y / atlas_h,
                "u1": (slot_x + tile_w) / atlas_w,
                "v1": (slot_y + tile_h) / atlas_h,
            })
            
            copyrect = frame.get("copyrect")

            if frame.get("file") == "$NONE":
                # Blank tile. Nothing needs to be copied.
                continue

            image = get_tile_image(frame)

            if image is None:
                continue

            if copyrect is not None:
                # copyrect specifies the destination rectangle in the tile.
                #
                # The source image is resized to exactly that rectangle.
                x, y, w, h = clamp_copyrect(copyrect, tile_w, tile_h)

                if w > 0 and h > 0:
                    image = image.resize(
                        (w, h),
                        Image.Resampling.LANCZOS
                    )

                    dest_x = slot_x + x
                    dest_y = slot_y + y

                    atlas.alpha_composite(
                        image,
                        (dest_x, dest_y)
                    )

            else:
                # Normal behavior: scale down if necessary, then center.
                image = fit_image(image, tile_w, tile_h)

                image_x = slot_x + (tile_w - image.width) // 2
                image_y = slot_y + (tile_h - image.height) // 2

                atlas.alpha_composite(
                    image,
                    (image_x, image_y)
                )
        output_tiles[name] = output

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    atlas.save(OUTPUT_PNG)

    output = {
        "size": [float(tile_w), float(tile_h)],
        "tiles": output_tiles,
    }

    with OUTPUT_JSON.open("w", encoding="utf-8") as f:
        json.dump(output, f, indent=4)

    print(f"Generated atlas: {OUTPUT_PNG}")
    print(f"Generated metadata: {OUTPUT_JSON}")
    print(f"Atlas size: {atlas_w}x{atlas_h}")
    print(f"Tiles: {tile_count}")
    print(f"Grid: {columns}x{rows}")


if __name__ == "__main__":
    main()