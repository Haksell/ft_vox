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

LEVELS = 5
SIZES = [TILE_SIZE >> i for i in range(LEVELS)]


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
    Resize an RGBA image with BOX filter using premultiplied alpha.
    """
    if img.mode != "RGBA":
        img = img.convert("RGBA")

    arr = np.asarray(img, dtype=np.float32) / 255.0  # HxWx4
    rgb = arr[..., :3]
    a = arr[..., 3:4]
    rgb_p = rgb * a

    def resample_f(ch, size):
        return np.asarray(
            Image.fromarray(ch, mode="F").resize(size, Image.Resampling.BOX),
            dtype=np.float32,
        )

    # Resize premultiplied RGB and alpha
    target_w, target_h = size
    r = resample_f(rgb_p[..., 0], (target_w, target_h))
    g = resample_f(rgb_p[..., 1], (target_w, target_h))
    b = resample_f(rgb_p[..., 2], (target_w, target_h))
    A = resample_f(a[..., 0], (target_w, target_h))

    eps = 1e-8
    r = np.where(A > eps, r / (A + eps), 0.0)
    g = np.where(A > eps, g / (A + eps), 0.0)
    b = np.where(A > eps, b / (A + eps), 0.0)

    out = np.stack([r, g, b, A], axis=-1)
    out = (np.clip(out, 0.0, 1.0) * 255.0 + 0.5).astype(np.uint8)
    return Image.fromarray(out, mode="RGBA")


def main():
    # Load and sanity-check base atlas
    img = Image.open(INPUT_PATH).convert("RGBA")
    w, h = img.size
    expected_w, expected_h = GRID_W * TILE_SIZE, GRID_H * TILE_SIZE
    if (w, h) != (expected_w, expected_h):
        raise ValueError(
            f"Atlas size {w}x{h} does not match expected {expected_w}x{expected_h}."
        )

    # Content fixes (unchanged)
    multiply_tile(img, 31, 2, GRASS_COLOR)
    replace_with_masked_mult(
        img, src_xy=(31, 0), dst_xy=(30, 15), mul=GRASS_COLOR, alpha_threshold=0
    )
    for x, y in [(6, 4), (6, 5), (7, 4), (7, 5)]:
        multiply_tile(img, x, y, WATER_COLOR)

    # Prepare 5x5 anisotropic grid layout
    # Columns vary X: 16,8,4,2,1; Rows vary Y: 16,8,4,2,1
    col_widths = [GRID_W * sx for sx in SIZES]
    row_heights = [GRID_H * sy for sy in SIZES]
    x_offsets = [0]
    for i in range(1, len(col_widths)):
        x_offsets.append(x_offsets[-1] + col_widths[i - 1])
    y_offsets = [0]
    for i in range(1, len(row_heights)):
        y_offsets.append(y_offsets[-1] + row_heights[i - 1])

    composite = Image.new("RGBA", (2 * w, 2 * h), (0, 0, 0, 0))

    # Build each cell: a full atlas reassembled from anisotropically-resized tiles
    for row_idx, sy in enumerate(SIZES):  # Y sizes: 16..1
        for col_idx, sx in enumerate(SIZES):  # X sizes: 16..1
            cell_w, cell_h = GRID_W * sx, GRID_H * sy
            cell_img = Image.new("RGBA", (cell_w, cell_h), (0, 0, 0, 0))

            for ty in range(GRID_H):
                for tx in range(GRID_W):
                    base_tile = img.crop(tile_box(tx, ty)).convert("RGBA")
                    if base_tile.size != (TILE_SIZE, TILE_SIZE):
                        base_tile = base_tile.resize(
                            (TILE_SIZE, TILE_SIZE), Image.NEAREST
                        )

                    # Anisotropic downscale for this tile
                    tile_resized = resize_rgba_box(base_tile, (sx, sy))

                    cell_img.paste(tile_resized, (tx * sx, ty * sy))

            composite.paste(cell_img, (x_offsets[col_idx], y_offsets[row_idx]))

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    composite.save(OUTPUT_PATH)
    print(f"Saved 5x5 anisotropic atlas grid to: {OUTPUT_PATH}")
    print("Layout rows/cols are [16, 8, 4, 2, 1] in pixels per tile.")


if __name__ == "__main__":
    main()
