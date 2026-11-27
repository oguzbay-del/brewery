#!/usr/bin/env python3
"""Generate the app icon and launch images from the brand symbol (G6, ADR-0012 > 3).

Usage: python3 -I ios/scripts/gen_brand.py

Input:  tasarim-sistemi/assets/brand/gristora-symbol.svg (do not edit that file)
Output: ios/Demleme/Resources/Assets.xcassets/AppIcon.appiconset (light, dark, tinted)
        ios/Demleme/Resources/Assets.xcassets/LaunchSymbol.imageset (light, dark)

Needs the Python package `cairosvg`. The PNG files are committed, so CI does not need it.
Icon rules: 1024x1024, opaque, no rounded corners (iOS masks the icon), symbol inside the
safe area. The tinted variant is grayscale; iOS applies the tint.
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SYMBOL = ROOT / "tasarim-sistemi/assets/brand/gristora-symbol.svg"
CATALOG = ROOT / "ios/Demleme/Resources/Assets.xcassets"

FOREST = "#0B2F26"
AMBER = "#F4A261"
DARK_BG = "#071A15"
SYMBOL_W, SYMBOL_H = 130, 160

# (file, background, color of the ring path, color of the three amber paths)
ICONS = [
    ("icon-light.png", FOREST, AMBER, AMBER),
    ("icon-dark.png", DARK_BG, AMBER, AMBER),
    ("icon-tinted.png", "#000000", "#ffffff", "#ffffff"),
]


def paths():
    text = SYMBOL.read_text(encoding="utf-8")
    found = re.findall(r'<path d="([^"]+)"', text)
    if len(found) != 4:
        sys.exit(f"error: expected 4 paths in {SYMBOL}, found {len(found)}")
    return found[0], found[1:]


def symbol_group(ring_color, piece_color):
    ring, pieces = paths()
    body = f'<path fill="{ring_color}" d="{ring}"/>'
    body += "".join(f'<path fill="{piece_color}" d="{d}"/>' for d in pieces)
    return body


def render(svg, out, size):
    import cairosvg  # pip install cairosvg

    cairosvg.svg2png(
        bytestring=svg.encode("utf-8"), write_to=str(out), output_width=size[0], output_height=size[1]
    )


def strip_alpha(path):
    """App Store icons must have no alpha channel."""
    from PIL import Image

    Image.open(path).convert("RGB").save(path)


def icon_svg(background, ring, piece):
    scale = 3.5  # symbol is 560 px high: inside the safe area of a 1024 px icon
    x = (1024 - SYMBOL_W * scale) / 2
    y = (1024 - SYMBOL_H * scale) / 2
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">'
        f'<rect width="1024" height="1024" fill="{background}"/>'
        f'<g transform="translate({x} {y}) scale({scale})">{symbol_group(ring, piece)}</g></svg>'
    )


def symbol_svg(ring, piece, scale):
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{SYMBOL_W * scale}" height="{SYMBOL_H * scale}" '
        f'viewBox="0 0 {SYMBOL_W} {SYMBOL_H}">{symbol_group(ring, piece)}</svg>'
    )


def write_json(path, data):
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def make_icons():
    folder = CATALOG / "AppIcon.appiconset"
    for name, background, ring, piece in ICONS:
        render(icon_svg(background, ring, piece), folder / name, (1024, 1024))
        strip_alpha(folder / name)
    base = {"idiom": "universal", "platform": "ios", "size": "1024x1024"}
    write_json(folder / "Contents.json", {
        "images": [
            {**base, "filename": "icon-light.png"},
            {**base, "filename": "icon-dark.png",
             "appearances": [{"appearance": "luminosity", "value": "dark"}]},
            {**base, "filename": "icon-tinted.png",
             "appearances": [{"appearance": "luminosity", "value": "tinted"}]},
        ],
        "info": {"author": "xcode", "version": 1},
    })


def make_launch_symbol():
    folder = CATALOG / "LaunchSymbol.imageset"
    folder.mkdir(parents=True, exist_ok=True)
    images = []
    # Light: Forest Green ring and Amber pieces on the light background.
    # Dark: all Amber, because Forest Green does not show on the dark background.
    for variant, ring in (("light", FOREST), ("dark", AMBER)):
        for scale in (1, 2, 3):
            name = f"launch-{variant}@{scale}x.png"
            render(symbol_svg(ring, AMBER, scale * 1.0), folder / name, (SYMBOL_W * scale, SYMBOL_H * scale))
            entry = {"idiom": "universal", "scale": f"{scale}x", "filename": name}
            if variant == "dark":
                entry["appearances"] = [{"appearance": "luminosity", "value": "dark"}]
            images.append(entry)
    write_json(folder / "Contents.json", {"images": images, "info": {"author": "xcode", "version": 1}})


if __name__ == "__main__":
    make_icons()
    make_launch_symbol()
    print("OK: app icon and launch symbol written")
