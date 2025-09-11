#!/usr/bin/env python3

from PIL import Image
from pathlib import Path

INPUT_PATH = Path("assets/atlas_original.png")
OUTPUT_PATH = Path("assets/atlas_generated.png")

TILE_SIZE = 16
GRID_W, GRID_H = 64, 32

GRASS_COLOR = (0.6, 0.9, 0.2)


def clamp8(v):
    return 0 if v < 0 else 255 if v > 255 else int(round(v))


def tile_box(tx, ty):
    """Return (left, top, right, bottom) pixel box for a tile at grid (tx, ty)."""
    left = tx * TILE_SIZE
    top = ty * TILE_SIZE
    return (left, top, left + TILE_SIZE, top + TILE_SIZE)


def multiply_tile(img, tx, ty, mul):
    """Multiply RGB of a tile by mul (leave alpha as-is)."""
    box = tile_box(tx, ty)
    region = img.crop(box).convert("RGBA")
    px = region.load()
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            r, g, b, a = px[x, y]
            r2 = clamp8(r * mul[0])
            g2 = clamp8(g * mul[1])
            b2 = clamp8(b * mul[2])
            px[x, y] = (r2, g2, b2, a)
    img.paste(region, box)


def replace_with_masked_mult(
    img,
    src_xy,
    dst_xy,
    mul,
    alpha_threshold=0,
):
    """
    In destination tile, replace pixels wherever source tile's alpha > alpha_threshold.
    New destination pixel = (src_rgb * mul, src_alpha).
    """
    src_box = tile_box(*src_xy)
    dst_box = tile_box(*dst_xy)

    src = img.crop(src_box).convert("RGBA")
    dst = img.crop(dst_box).convert("RGBA")

    s_px = src.load()
    d_px = dst.load()

    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            r, g, b, a = s_px[x, y]
            if a > alpha_threshold:
                r2 = clamp8(r * mul[0])
                g2 = clamp8(g * mul[1])
                b2 = clamp8(b * mul[2])
                d_px[x, y] = (r2, g2, b2, a)  # replace (including alpha from source)

    img.paste(dst, dst_box)


def main():
    # Load and sanity-check dimensions
    img = Image.open(INPUT_PATH).convert("RGBA")
    w, h = img.size
    expected_w, expected_h = GRID_W * TILE_SIZE, GRID_H * TILE_SIZE
    if (w, h) != (expected_w, expected_h):
        raise ValueError(
            f"Atlas size {w}x{h} does not match expected {expected_w}x{expected_h}."
        )

    # 1) Multiply tile (31, 2)
    multiply_tile(img, 31, 2, GRASS_COLOR)

    # 2) In tile (30, 15), replace where tile (31, 0) is non-transparent, using multiplied source
    replace_with_masked_mult(
        img, src_xy=(31, 0), dst_xy=(30, 15), mul=GRASS_COLOR, alpha_threshold=0
    )

    # 3) Save
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    img.save(OUTPUT_PATH)


if __name__ == "__main__":
    main()
