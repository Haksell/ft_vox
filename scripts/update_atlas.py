#!/usr/bin/env python3
from PIL import Image
from pathlib import Path
import numpy as np

INPUT_PATH = Path("assets/atlas_original.png")
OUTPUT_PATH = Path("assets/atlas_generated.png")

TILE_SIZE = 16
GRID_W, GRID_H = 64, 32

GRASS_COLOR = (0.6, 0.9, 0.2)
WATER_COLOR = (0, 0.5, 0.9)

LEVELS = 5  # 16, 8, 4, 2, 1


def clamp8(v):
    return 0 if v < 0 else 255 if v > 255 else int(round(v))


def tile_box(tx, ty, tile_size=TILE_SIZE):
    left = tx * tile_size
    top = ty * tile_size
    return (left, top, left + tile_size, top + tile_size)


def multiply_tile(img, tx, ty, mul):
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


def replace_with_masked_mult(img, src_xy, dst_xy, mul, alpha_threshold=0):
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
                d_px[x, y] = (r2, g2, b2, a)

    img.paste(dst, dst_box)


def resize_rgba_box(img: Image.Image, size):
    """
    Resize an RGBA image with BOX filter.
    Uses premultiplied alpha if NumPy is available (recommended).
    """
    if img.mode != "RGBA":
        img = img.convert("RGBA")

    # Premultiplied-alpha path
    arr = np.asarray(img, dtype=np.float32) / 255.0  # HxWx4
    rgb = arr[..., :3]
    a = arr[..., 3:4]
    rgb_p = rgb * a

    # Resize each channel as 'F' (float) with BOX
    def resample_f(ch):
        return np.asarray(
            Image.fromarray(ch, mode="F").resize(size, Image.Resampling.BOX),
            dtype=np.float32,
        )

    r = resample_f(rgb_p[..., 0])
    g = resample_f(rgb_p[..., 1])
    b = resample_f(rgb_p[..., 2])
    A = resample_f(a[..., 0])

    eps = 1e-8
    r = np.where(A > eps, r / (A + eps), 0.0)
    g = np.where(A > eps, g / (A + eps), 0.0)
    b = np.where(A > eps, b / (A + eps), 0.0)

    out = np.stack([r, g, b, A], axis=-1)
    out = (np.clip(out, 0.0, 1.0) * 255.0 + 0.5).astype(np.uint8)
    return Image.fromarray(out, mode="RGBA")


def build_tile_mips(tile_rgba: Image.Image, levels=LEVELS):
    """Return list [16x16, 8x8, 4x4, 2x2, 1x1] for a single tile."""
    mips = []
    cur = tile_rgba
    for i in range(levels):
        w = max(1, TILE_SIZE >> i)
        size = (w, w)
        if i == 0:
            # Ensure base is exactly TILE_SIZE
            cur = cur.resize(size, Image.NEAREST) if cur.size != size else cur
        else:
            cur = resize_rgba_box(cur, size)
        mips.append(cur)
    return mips


def main():
    # load
    img = Image.open(INPUT_PATH).convert("RGBA")
    w, h = img.size
    expected_w, expected_h = GRID_W * TILE_SIZE, GRID_H * TILE_SIZE
    if (w, h) != (expected_w, expected_h):
        raise ValueError(
            f"Atlas size {w}x{h} does not match expected {expected_w}x{expected_h}."
        )

    # fix grass top
    multiply_tile(img, 31, 2, GRASS_COLOR)

    # fix grass side
    replace_with_masked_mult(
        img, src_xy=(31, 0), dst_xy=(30, 15), mul=GRASS_COLOR, alpha_threshold=0
    )

    # fix water
    for x, y in [(6, 4), (6, 5), (7, 4), (7, 5)]:
        multiply_tile(img, x, y, WATER_COLOR)

    # save
    level_sizes = [
        (GRID_W * max(1, TILE_SIZE >> level), GRID_H * max(1, TILE_SIZE >> level))
        for level in range(LEVELS)
    ]
    level_imgs = [Image.new("RGBA", size, (0, 0, 0, 0)) for size in level_sizes]

    for ty in range(GRID_H):
        for tx in range(GRID_W):
            base_tile = img.crop(tile_box(tx, ty)).convert("RGBA")
            mips = build_tile_mips(base_tile, LEVELS)
            for level, tile in enumerate(mips):
                s = max(1, TILE_SIZE >> level)
                level_imgs[level].paste(tile, (tx * s, ty * s))

    composite_w = level_imgs[0].width  # 1024
    composite = Image.new("RGBA", (composite_w, 2 * h), (0, 0, 0, 0))

    y = 0
    for im in level_imgs:
        composite.paste(im, (0, y))
        y += im.height

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    composite.save(OUTPUT_PATH)
    print(f"Saved stacked composite to: {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
