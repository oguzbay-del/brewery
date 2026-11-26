"""Build responsive website photo assets from raw JPGs and owner-supplied editorial replacements.

Raw sources live in ../fotograflar-ham/. The 12 curated feature images in
brewing-editorial-photos.zip override the earlier feature-photo versions.
"""
import pathlib
import zipfile
from io import BytesIO
from PIL import Image, features

root = pathlib.Path(__file__).resolve().parents[1]
src = root.parent / "fotograflar-ham"
out = root / "public" / "images" / "photos"
out.mkdir(parents=True, exist_ok=True)

WIDTHS = (480, 640, 768, 960, 1408)
FEATURE_WIDTHS = (480, 640, 768, 960, 1152)
CURATED_WIDTHS = (640, 896)

# Vertical focal position (0 = top, 1 = bottom) for 3:2 feature crops.
FOCUS_Y = {
    "feature-fermentation-monitoring": 0.50,
    "feature-quality-laboratory": 0.50,
    "feature-packaging-compliance": 0.50,
    "feature-team-notifications": 0.50,
    "feature-connected-devices": 0.50,
    "feature-imports-exports-tools": 0.50,
    "feature-analytics-reports": 0.50,
}

# ZIP filename stem -> stable website photo key.
CURATED_MAP = {
    "active-fermentation-carboy": "feature-fermentation-monitoring",
    "Feature-equipment-calibration": "feature-equipment-calibration",
    "feature-planning-scheduling": "feature-planning-scheduling",
    "feature-inventory-costing": "feature-inventory-costing",
    "feature-production-management": "feature-production-management",
    "feature-quality-laboratory": "feature-quality-laboratory",
    "feature-packaging-compliance": "feature-packaging-compliance",
    "feature-team-notifications": "feature-team-notifications",
    "feature-connected-devices": "feature-connected-devices",
    "Feature-imports-exports-tools": "feature-imports-exports-tools",
    "Feature-platform-security-data": "feature-platform-security-data",
    "feature-analytics-reports": "feature-analytics-reports",
}


def crop_feature(image, key):
    width, height = image.size
    target_width = int(height * 3 / 2)
    if target_width < width:
        left = (width - target_width) // 2
        return image.crop((left, 0, left + target_width, height))
    if target_width > width:
        target_height = int(width * 2 / 3)
        center_y = int(height * FOCUS_Y.get(key, 0.5))
        top = min(max(center_y - target_height // 2, 0), height - target_height)
        return image.crop((0, top, width, top + target_height))
    return image


def save_responsive(image, key, widths, include_jpeg=False):
    for target_width in widths:
        target_height = round(image.height * target_width / image.width)
        resized = image if target_width == image.width else image.resize(
            (target_width, target_height), Image.Resampling.LANCZOS
        )
        resized.save(out / f"{key}-{target_width}.avif", quality=65, speed=6)
        resized.save(out / f"{key}-{target_width}.webp", quality=84, method=6)
        if include_jpeg:
            resized.save(
                out / f"{key}-{target_width}.jpg",
                quality=90,
                optimize=True,
                progressive=True,
            )


if not features.check("avif"):
    raise RuntimeError("Pillow AVIF encoder is unavailable; install a Pillow build with AVIF support.")

# Existing editorial sources remain the default for pages and non-curated features.
curated_keys = set(CURATED_MAP.values())
for file in sorted(src.glob("*.jpg")) + sorted(src.glob("*.png")):
    # Do not regenerate superseded low-quality feature derivatives from the old sources.
    if file.stem in curated_keys:
        continue
    image = Image.open(file).convert("RGB")
    is_feature = file.stem.startswith("feature-")
    if is_feature:
        image = crop_feature(image, file.stem)
    widths = FEATURE_WIDTHS if is_feature else WIDTHS
    save_responsive(image, file.stem, widths)

# The owner-supplied 12-photo pack deliberately overrides the older feature sources.
archive = src / "brewing-editorial-photos.zip"
if not archive.is_file():
    raise FileNotFoundError(f"Required curated photo archive not found: {archive}")

with zipfile.ZipFile(archive) as bundle:
    for member in sorted(bundle.namelist()):
        if pathlib.Path(member).suffix.lower() not in {".jpg", ".jpeg"}:
            continue
        source_stem = pathlib.Path(member).stem
        key = CURATED_MAP.get(source_stem)
        if not key:
            continue
        image = Image.open(BytesIO(bundle.read(member))).convert("RGB")
        image = crop_feature(image, key)
        # The supplied originals are 896 px wide after crop; never upscale them.
        save_responsive(image, key, CURATED_WIDTHS, include_jpeg=True)
        print(f"curated {source_stem} -> {key}: {image.width}x{image.height}")

print("Photo optimisation complete.")
