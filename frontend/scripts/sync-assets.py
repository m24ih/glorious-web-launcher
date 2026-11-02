#!/usr/bin/env python3
"""Build web-ready mouse renders from the extracted Glorious CORE assets.

The originals in ``assets/images`` are 4K product shots with an opaque white
plate behind an alpha-outline of the mouse. This script:

1. removes that white plate with a connected-component flood fill,
2. crops to the actual product pixels,
3. resizes to a retina-friendly 1700 px tall WebP with alpha,
4. writes one image per device slug into ``public/mice``.

Run from ``frontend/``:

    npm run assets

Requires Pillow, numpy and scipy.
"""

from __future__ import annotations

import shutil
import sys
from pathlib import Path

import numpy as np
from PIL import Image
from scipy import ndimage

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "assets" / "images"
OUT_MICE = Path(__file__).resolve().parents[1] / "public" / "mice"
OUT_BRAND = Path(__file__).resolve().parents[1] / "public" / "brand"

# slug -> source render in assets/images (without extension)
DEVICE_RENDERS = {
    "model-d-wireless": "ModelDWireless3",
    "model-o-wireless": "ModelO3",
    "model-o": "ModelO3",
    "model-i": "ModelI3",
    "model-o-pro-wireless": "ModelOPROWL3",
    "model-d-pro-wireless": "ModelDPROWL3",
    "model-o-2-pro": "ModelO2PROWL3",
    "model-d-2-pro": "ModelD2PROWL3",
    "model-o-2-pro-4k-8khz-edition": "ModelO2PROWL3",
    "model-d-2-pro-4k-8khz-edition": "ModelD2PROWL3",
    "model-o-2-wireless": "MOW23",
    "model-o-2-wired": "MOWired23",
    "series-one-pro-wireless": "SeriesOneProWL3",
    "model-i-2-wireless": "MI2Wireless3",
}

TARGET_HEIGHT = 1700
WEBP_QUALITY = 88


def strip_white_plate(image: Image.Image) -> Image.Image:
    """Make the opaque white studio plate transparent, keeping the product."""
    rgba = np.array(image.convert("RGBA"))
    rgb = rgba[:, :, :3].astype(np.int16)
    alpha = rgba[:, :, 3]

    near_white = (
        (alpha > 0)
        & (rgb.min(axis=2) >= 222)
        & ((rgb.max(axis=2) - rgb.min(axis=2)) <= 14)
    )
    if not near_white.any():
        return image

    labels, count = ndimage.label(near_white)
    if count == 0:
        return image

    # The studio plate is always one enormous near-white blob (0.5M-2.6M px)
    # while genuine white details on the product are small. Drop large blobs
    # instead of relying on edge connectivity: several source shots have a
    # transparent shadow that separates the plate from the canvas edge.
    sizes = ndimage.sum(near_white, labels, range(1, count + 1))
    plate_labels = [index + 1 for index, size in enumerate(sizes) if size >= 50_000]
    if not plate_labels:
        return image

    remove = np.isin(labels, plate_labels)
    rgba[:, :, 3] = np.where(remove, 0, alpha)

    # A few source files carry near-invisible (alpha 1-10) specks across the
    # whole canvas that would defeat a plain getbbox() crop.
    rgba[:, :, 3] = np.where(rgba[:, :, 3] < 12, 0, rgba[:, :, 3])
    return Image.fromarray(rgba, "RGBA")


def build_render(source: Path, destination: Path) -> tuple[int, int, int]:
    image = strip_white_plate(Image.open(source))
    # Crop on the solid product silhouette. Faint alpha haze around studio
    # shots can span the whole canvas and would otherwise defeat the crop.
    silhouette = image.getchannel("A").point(lambda value: 255 if value >= 32 else 0)
    bbox = silhouette.getbbox()
    if bbox:
        image = image.crop(bbox)

    width, height = image.size
    scale = TARGET_HEIGHT / height
    image = image.resize(
        (max(1, round(width * scale)), TARGET_HEIGHT), Image.Resampling.LANCZOS
    )
    destination.parent.mkdir(parents=True, exist_ok=True)
    image.save(destination, "WEBP", quality=WEBP_QUALITY, method=6)
    return image.width, image.height, destination.stat().st_size


def main() -> int:
    if not SRC.is_dir():
        print(f"missing asset directory: {SRC}", file=sys.stderr)
        return 1

    OUT_MICE.mkdir(parents=True, exist_ok=True)
    OUT_BRAND.mkdir(parents=True, exist_ok=True)

    built: dict[str, Path] = {}
    for slug, stem in DEVICE_RENDERS.items():
        source = SRC / f"{stem}.png"
        if not source.exists():
            print(f"skip {slug}: {source.name} not found", file=sys.stderr)
            continue
        destination = OUT_MICE / f"{slug}.webp"
        if slug not in built:
            width, height, size = build_render(source, destination)
            print(f"{destination.name:46} {width}x{height}  {size / 1024:6.1f} KiB")
            built[slug] = destination
        else:
            shutil.copyfile(built[slug], destination)
            print(f"{destination.name:46} copy of {built[slug].name}")

    fallback = OUT_MICE / "model-o-wireless.webp"
    if fallback.exists():
        shutil.copyfile(fallback, OUT_MICE / "fallback.webp")

    logo = SRC / "logo.png"
    if logo.exists():
        shutil.copyfile(logo, OUT_BRAND / "logo.png")
        print("brand/logo.png")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
