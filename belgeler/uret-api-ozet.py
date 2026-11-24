#!/usr/bin/env python3
"""Generate the API endpoint reference from the canonical OpenAPI document."""

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC_PATH = ROOT / "sozlesme" / "openapi" / "openapi.json"
OUTPUT_PATH = ROOT / "belgeler" / "api-ozet.md"
METHODS = ("get", "post", "put", "patch", "delete", "head", "options")


def cell(value):
    return str(value or "").replace("|", r"\|").replace("\n", " ").strip()


def operation_rows(spec):
    rows = []
    for path, path_item in spec.get("paths", {}).items():
        for method in METHODS:
            operation = path_item.get(method)
            if not isinstance(operation, dict):
                continue
            parameters = list(path_item.get("parameters", []))
            parameters.extend(operation.get("parameters", []))
            headers = {
                str(parameter.get("name", "")).lower(): parameter
                for parameter in parameters
                if parameter.get("in") == "header"
            }
            permission = operation.get("x-permission")
            description = " ".join(
                (operation.get("summary") or "", operation.get("description") or "")
            ).lower()
            step_up = any(
                marker in description
                for marker in ("second step", "step-up", "passkey check")
            )
            auth = []
            if permission and permission != "none":
                auth.append(str(permission))
            if step_up:
                auth.append("step-up")
            idem = headers.get("idempotency-key")
            if_match = headers.get("if-match")
            rows.append(
                {
                    "tag": (operation.get("tags") or ["untagged"])[0],
                    "method": method.upper(),
                    "path": path,
                    "summary": " ".join((operation.get("summary") or "").split()),
                    "auth": "; ".join(auth),
                    "idem": ("required" if idem.get("required") else "optional") if idem else "",
                    "match": ("required" if if_match.get("required") else "optional") if if_match else "",
                }
            )
    return rows


def render(spec):
    rows = operation_rows(spec)
    tag_descriptions = {
        tag["name"]: tag.get("description", "") for tag in spec.get("tags", [])
    }
    grouped = {}
    for row in rows:
        grouped.setdefault(row["tag"], []).append(row)

    lines = [
        "# API uç nokta özeti",
        "",
        "Bu dosya sozlesme/openapi/openapi.json kaynağından belgeler/uret-api-ozet.py ile üretilir. Kaynak sözleşmeyi değiştirin; bu dosyayı elle düzenlemeyin.",
        "",
        f"Toplam **{len(rows)}** operasyon. Yetki ve adım bilgisi yalnız OpenAPI operasyon alanlarından alınır. Boş hücre, sözleşmede ilgili bilgi bulunmadığını gösterir.",
        "",
    ]
    for tag in sorted(grouped):
        lines.extend([f"## {tag}", ""])
        if tag_descriptions.get(tag):
            lines.extend([tag_descriptions[tag], ""])
        lines.extend([
            "| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |",
            "|---|---|---|---|---|---|",
        ])
        for row in sorted(grouped[tag], key=lambda item: (item["path"], item["method"])):
            lines.append(
                f"| {row['method']} | `{row['path']}` | {cell(row['summary'])} | "
                f"{cell(row['auth'])} | {row['idem']} | {row['match']} |"
            )
        lines.append("")

    schemas = spec.get("components", {}).get("schemas", {})
    codes = schemas.get("ErrorCode", {}).get("enum", [])
    lines.extend(["## Ortak hata kodları", "", "OpenAPI ErrorCode enum:", ""])
    lines.extend(f"- `{code}`" for code in codes)
    problem = schemas.get("ProblemDetails", {})
    properties = problem.get("properties", {})
    required = problem.get("required", [])
    optional = [name for name in properties if name not in required]
    request_id_desc = properties.get("request_id", {}).get("description", "")
    lines.extend([
        "",
        "Hata gövdesi RFC 9457 ProblemDetails biçimindedir. Zorunlu alanlar: "
        + ", ".join(required) + ". İsteğe bağlı alanlar: " + ", ".join(optional)
        + ". request_id açıklaması: " + request_id_desc,
        "",
        "## Ortak ve operasyonel başlıklar",
        "",
        "| Başlık | Kullanım |",
        "|---|---|",
    ])

    headers = {}
    for path_item in spec.get("paths", {}).values():
        for method in METHODS:
            operation = path_item.get(method)
            if not isinstance(operation, dict):
                continue
            for parameter in list(path_item.get("parameters", [])) + list(operation.get("parameters", [])):
                if parameter.get("in") != "header":
                    continue
                name = parameter.get("name", "")
                old = headers.get(name, {"description": "", "required": False})
                headers[name] = {
                    "description": parameter.get("description", "") or old["description"],
                    "required": bool(parameter.get("required")) or old["required"],
                }
    for name in sorted(headers):
        item = headers[name]
        state = "required" if item["required"] else "optional"
        lines.append(f"| `{name}` | {cell(item['description'])} ({state}) |")
    if request_id_desc:
        lines.append(f"| `X-Request-Id` | {cell(request_id_desc)} |")
    return "\n".join(lines).rstrip() + "\n"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    spec = json.loads(SPEC_PATH.read_text(encoding="utf-8"))
    generated = render(spec)
    if args.check:
        current = OUTPUT_PATH.read_text(encoding="utf-8") if OUTPUT_PATH.exists() else None
        if current != generated:
            print(f"Out of date: {OUTPUT_PATH.relative_to(ROOT)}")
            return 1
        print("API reference is up to date.")
        return 0
    OUTPUT_PATH.write_text(generated, encoding="utf-8")
    print(f"Wrote {OUTPUT_PATH.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
