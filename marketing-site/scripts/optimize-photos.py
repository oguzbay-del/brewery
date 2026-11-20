"""Convert raw photos in ../fotograflar-ham/ to responsive AVIF/WebP in public/images/photos/.
Hero/page photos keep their ratio; feature-* photos are center-cropped to 3:2."""
import pathlib, sys
from PIL import Image
root = pathlib.Path(__file__).resolve().parents[1]
src = root.parent / "fotograflar-ham"
out = root / "public" / "images" / "photos"
out.mkdir(parents=True, exist_ok=True)
WIDTHS = (640, 960, 1408)
# Vertical focus (0 = top, 1 = bottom) for portrait sources cropped to 3:2.
FOCUS_Y = {"feature-fermentation-monitoring": .37, "feature-quality-laboratory": .6, "feature-packaging-compliance": .55,
           "feature-team-notifications": .42, "feature-connected-devices": .55, "feature-imports-exports-tools": .58,
           "feature-analytics-reports": .7}
for f in sorted(src.glob("*.jpg")) + sorted(src.glob("*.png")):
    im = Image.open(f).convert("RGB")
    if f.stem.startswith("feature-"):
        w, h = im.size; tw = int(h * 3 / 2)
        if tw < w: im = im.crop(((w - tw) // 2, 0, (w - tw) // 2 + tw, h))
        elif tw > w:  # portrait source: crop height around FOCUS_Y
            th = int(w * 2 / 3); cy = int(h * FOCUS_Y.get(f.stem, .5))
            top = min(max(cy - th // 2, 0), h - th); im = im.crop((0, top, w, top + th))
    widths = (640, 960, 1152) if f.stem.startswith("feature-") else WIDTHS
    for tw in widths:
        # Always emit every width so PagePhoto can use fixed srcset names.
        r = im.resize((tw, round(im.height * tw / im.width)), Image.LANCZOS)
        r.save(out / f"{f.stem}-{tw}.avif", quality=55)
        r.save(out / f"{f.stem}-{tw}.webp", quality=78, method=6)
    print(f.stem, im.size)
