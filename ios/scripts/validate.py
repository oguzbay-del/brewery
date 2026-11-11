#!/usr/bin/env python3
"""Linux-side checks for the iOS skeleton (no Xcode needed).

Usage: python3 -I ios/scripts/validate.py
"""
import json
import subprocess
import sys
from pathlib import Path

IOS = Path(__file__).resolve().parents[1]
errors = []


def err(msg):
    errors.append(msg)


def check_project_yml():
    path = IOS / "project.yml"
    if not path.exists():
        return err("project.yml missing")
    text = path.read_text(encoding="utf-8")
    if "\t" in text:
        err("project.yml contains tab characters")
    try:
        import yaml  # optional
    except ImportError:
        yaml = None
    if yaml:
        try:
            doc = yaml.safe_load(text)
        except yaml.YAMLError as e:
            return err(f"project.yml is not valid YAML: {e}")
        if not isinstance(doc, dict):
            return err("project.yml top level is not a mapping")
        targets = doc.get("targets", {})
        for t in ("Demleme", "DemlemeTests", "DemlemeUITests"):
            if t not in targets:
                err(f"project.yml missing target {t}")
    else:
        for needle in ("name:", "targets:", "  Demleme:", "  DemlemeTests:", "  DemlemeUITests:"):
            if needle not in text:
                err(f"project.yml missing {needle!r} (simple check; PyYAML not installed)")


def check_colorsets():
    sets = sorted((IOS / "Demleme/Resources/Assets.xcassets/Colors").glob("*.colorset/Contents.json"))
    if not sets:
        return err("no colorsets found")
    for p in sets:
        rel = p.relative_to(IOS)
        try:
            d = json.loads(p.read_text(encoding="utf-8"))
        except json.JSONDecodeError as e:
            err(f"{rel}: invalid JSON: {e}")
            continue
        colors = d.get("colors")
        if not colors or not isinstance(colors, list):
            err(f"{rel}: missing colors")
            continue
        if d.get("info", {}).get("version") != 1:
            err(f"{rel}: missing info.version")
        for c in colors:
            comps = c.get("color", {}).get("components", {})
            if c.get("color", {}).get("color-space") != "srgb" or set(comps) != {"red", "green", "blue", "alpha"}:
                err(f"{rel}: bad color entry")
                continue
            for k, v in comps.items():
                if not 0.0 <= float(v) <= 1.0:
                    err(f"{rel}: component {k} out of range")
        if not any("appearances" not in c for c in colors):
            err(f"{rel}: no default (light) entry")
        if not any(a.get("value") == "dark" for c in colors for a in c.get("appearances", [])):
            err(f"{rel}: no dark entry")


def check_generator():
    r = subprocess.run(
        [sys.executable, "-I", str(IOS / "scripts/gen_colors.py"), "--check"],
        capture_output=True, text=True,
    )
    if r.returncode != 0:
        err("gen_colors.py --check failed: " + (r.stderr or r.stdout).strip())


def main():
    check_project_yml()
    check_colorsets()
    check_generator()
    if errors:
        print("\n".join(f"ERROR: {e}" for e in errors), file=sys.stderr)
        return 1
    print("OK: project.yml, colorsets and generated files are valid")
    return 0


if __name__ == "__main__":
    sys.exit(main())
