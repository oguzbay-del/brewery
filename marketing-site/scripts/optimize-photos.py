"""Convert raw photos in ../fotograflar-ham/ to responsive AVIF/WebP in public/images/photos/.
Hero/page photos keep their ratio; feature-* photos are center-cropped to 3:2."""
import pathlib, sys
from PIL import Image
root = pathlib.Path(__file__).resolve().parents[1]
src = root.parent / "fotograflar-ham"
out = root / "public" / "images" / "photos"
out.mkdir(parents=True, exist_ok=True)
WIDTHS = (640, 960, 1408)
for f in sorted(src.glob("*.jpg")) + sorted(src.glob("*.png")):
    im = Image.open(f).convert("RGB")
    if f.stem.startswith("feature-"):
        w, h = im.size; tw = int(h * 3 / 2)
        if tw < w: im = im.crop(((w - tw) // 2, 0, (w - tw) // 2 + tw, h))
    for tw in WIDTHS:
        if tw > im.width: tw = im.width
        r = im.resize((tw, round(im.height * tw / im.width)), Image.LANCZOS)
        r.save(out / f"{f.stem}-{tw}.avif", quality=55)
        r.save(out / f"{f.stem}-{tw}.webp", quality=78, method=6)
    print(f.stem, im.size)
