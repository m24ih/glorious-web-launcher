#!/usr/bin/env python3
"""Extract Glorious mouse definitions from the Core app's SupportDevice.DB.

SupportDevice.DB is not SQLite despite the extension: it is newline-delimited
JSON (one device object per line). Records with ModelType == 1 are the mouse /
mouse-family entries. That set also contains the "Dongle V2 Kit" USB receiver
(StateType "Dongle"), which is not a mouse, so it is excluded to leave the 16
actual mouse devices.

Usage:
    python3 scripts/extract_devices.py
    python3 scripts/extract_devices.py --db /path/SupportDevice.DB --out data/glorious_devices.json
"""

from __future__ import annotations

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

DEFAULT_DB = Path(
    "/home/melih/Projects/Glorious-Core-Linux/src/electron-process/"
    "renderer-interop/database/SupportDevice.DB"
)
DEFAULT_OUT = Path(__file__).resolve().parent.parent / "data" / "glorious_devices.json"
DEFAULT_SOURCE_IMAGES = Path(
    "/home/melih/Projects/Glorious-Core-Linux/src/renderer-process/assets/images"
)

MOUSE_MODEL_TYPE = 1


def is_mouse(record: dict[str, Any]) -> bool:
    """True for mouse-family records (ModelType 1, excluding pure USB receivers).

    Wireless mice also list "Dongle" among their transports, so a record is a
    receiver only when every transport it advertises is a dongle.
    """
    if record.get("ModelType") != MOUSE_MODEL_TYPE:
        return False
    states = {str(s).lower() for s in record.get("StateType", [])}
    return not states or states != {"dongle"}


def connections(record: dict[str, Any]) -> list[dict[str, Any]]:
    """Zip parallel StateType/vid/pid arrays into per-connection objects."""
    states = record.get("StateType") or []
    vids = record.get("vid") or []
    pids = record.get("pid") or []
    rows: list[dict[str, Any]] = []
    for index in range(max(len(states), len(vids), len(pids))):
        rows.append(
            {
                "type": states[index] if index < len(states) else None,
                "vid": vids[index] if index < len(vids) else None,
                "pid": pids[index] if index < len(pids) else None,
            }
        )
    return rows


def slugify(value: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")


def resolve_image(img_key: str | None, images_dir: Path) -> dict[str, Any] | None:
    """Map the DB `img` key to a real render file using the app's naming scheme."""
    if not img_key:
        return None
    for candidate in (f"{img_key}3.png", f"{img_key}.png", f"{img_key}3.svg", f"{img_key}.svg"):
        if (images_dir / candidate).is_file():
            return {
                "key": img_key,
                "file": candidate,
                "path": f"assets/images/{candidate}",
            }
    return {"key": img_key, "file": None, "path": None}


def build_device(record: dict[str, Any], images_dir: Path) -> dict[str, Any]:
    name = record.get("devicename", "")
    return {
        "id": record.get("_id") or slugify(name),
        "name": name,
        "slug": slugify(name),
        "routerID": record.get("routerID"),
        "modelType": record.get("ModelType"),
        "image": resolve_image(record.get("img"), images_dir),
        "connections": connections(record),
        "connectionTypes": record.get("StateType", []),
        "battery": record.get("battery"),
        "layerMaxNumber": record.get("layerMaxNumber"),
        "sideLightSwitch": record.get("sideLightSwitch"),
        "pairingFlag": record.get("pairingFlag"),
        "fwUpdateExtension": record.get("FWUpdateExtension", []),
        "hid": {
            "set": record.get("set", []),
            "get": record.get("get", []),
        },
    }


def read_records(db_path: Path) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    with db_path.open(encoding="utf-8") as handle:
        for lineno, line in enumerate(handle, 1):
            line = line.strip()
            if not line:
                continue
            try:
                records.append(json.loads(line))
            except json.JSONDecodeError as exc:
                raise SystemExit(f"{db_path}:{lineno}: invalid JSON ({exc})") from exc
    return records


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--db", type=Path, default=DEFAULT_DB)
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    parser.add_argument(
        "--source-images",
        type=Path,
        default=DEFAULT_SOURCE_IMAGES,
        help="Directory holding the app's render files, used to resolve image paths.",
    )
    parser.add_argument(
        "--keep-non-mouse",
        action="store_true",
        help="Do not drop receivers/dongles that share ModelType 1.",
    )
    args = parser.parse_args()

    records = read_records(args.db)
    candidates = [r for r in records if r.get("ModelType") == MOUSE_MODEL_TYPE]

    if args.keep_non_mouse:
        mice, excluded = candidates, []
    else:
        mice = [r for r in candidates if is_mouse(r)]
        excluded = [
            {
                "id": r.get("_id"),
                "name": r.get("devicename"),
                "reason": "USB receiver/dongle, not a mouse (ModelType 1)",
            }
            for r in candidates
            if not is_mouse(r)
        ]

    devices = [build_device(r, args.source_images) for r in mice]
    missing = [d["name"] for d in devices if not d["image"] or not d["image"]["file"]]

    payload = {
        "meta": {
            "generatedAt": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "source": str(args.db),
            "sourceFormat": "newline-delimited JSON",
            "totalRecords": len(records),
            "modelType1Records": len(candidates),
            "mouseCount": len(devices),
            "excluded": excluded,
            "imagesMissing": missing,
        },
        "devices": devices,
    }

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    print(f"Wrote {len(devices)} mice -> {args.out}")
    if excluded:
        print("Excluded:", ", ".join(f"{e['name']} ({e['id']})" for e in excluded))
    if missing:
        print("Missing renders:", ", ".join(missing))
    for device in devices:
        image = (device["image"] or {}).get("file") or "-"
        print(f"  {device['name']:<34} {device['id']:<20} {image}")


if __name__ == "__main__":
    main()
