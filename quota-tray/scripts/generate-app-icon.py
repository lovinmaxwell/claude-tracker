#!/usr/bin/env python3
"""Render Quota Tray installer/app icons from the 16x16 mascot grid."""

from __future__ import annotations

import struct
import zlib
from pathlib import Path

# Same grid as quota_tray_core::icon::MASCOT_ART
MASCOT_ART = [
    "................",
    "................",
    "...BBBBBBBBBB...",
    "...BBBBBBBBBB...",
    "...BEEBBBBEEB...",
    "...BEEBBBBEEB...",
    ".BBBBBBBBBBBBBB.",
    ".BBBBBBBBBBBBBB.",
    ".BBBBBBBBBBBBBB.",
    "...BBBBBBBBBB...",
    "...BBBBBBBBBB...",
    "...BB.BB.BB.BB..",
    "...BB.BB.BB.BB..",
    "...BB.BB.BB.BB..",
    "................",
    "................",
]

CREAM = (0xF5, 0xED, 0xE6, 255)
TERRACOTTA = (0xC9, 0x64, 0x42, 255)
EYE = (0x2C, 0x2C, 0x2C, 255)
BG = (0xFA, 0xF6, 0xF1, 255)
TRANSPARENT = (0, 0, 0, 0)


def png_bytes(width: int, height: int, rgba: bytes) -> bytes:
    def chunk(tag: bytes, data: bytes) -> bytes:
        crc = zlib.crc32(tag + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)

    raw = b"".join(b"\x00" + rgba[y * width * 4 : (y + 1) * width * 4] for y in range(height))
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")


def ico_from_pngs(pngs: list[tuple[int, bytes]]) -> bytes:
    count = len(pngs)
    offset = 6 + 16 * count
    entries = b""
    payload = b""
    for size, data in pngs:
        w = 0 if size >= 256 else size
        h = 0 if size >= 256 else size
        entries += struct.pack("<BBBBHHII", w, h, 0, 0, 1, 32, len(data), offset)
        payload += data
        offset += len(data)
    return struct.pack("<HHH", 0, 1, count) + entries + payload


def put_pixel(buf: bytearray, size: int, x: int, y: int, color: tuple[int, int, int, int]) -> None:
    if 0 <= x < size and 0 <= y < size:
        i = (y * size + x) * 4
        buf[i : i + 4] = bytes(color)


def rounded_rect_mask(size: int, radius: int) -> list[bool]:
    mask = [False] * (size * size)
    r2 = radius * radius
    for y in range(size):
        for x in range(size):
            inside = True
            if x < radius and y < radius:
                dx, dy = radius - 1 - x, radius - 1 - y
                inside = dx * dx + dy * dy <= r2
            elif x >= size - radius and y < radius:
                dx, dy = x - (size - radius), radius - 1 - y
                inside = dx * dx + dy * dy <= r2
            elif x < radius and y >= size - radius:
                dx, dy = radius - 1 - x, y - (size - radius)
                inside = dx * dx + dy * dy <= r2
            elif x >= size - radius and y >= size - radius:
                dx, dy = x - (size - radius), y - (size - radius)
                inside = dx * dx + dy * dy <= r2
            mask[y * size + x] = inside
    return mask


def body_row_bounds() -> tuple[int, int]:
    rows = [i for i, line in enumerate(MASCOT_ART) if any(c in "BE" for c in line)]
    return min(rows), max(rows)


def render(size: int, *, fill_pct: float = 40.0, background: bool = True) -> bytes:
    buf = bytearray(size * size * 4)
    if background:
        radius = max(size // 6, 8)
        mask = rounded_rect_mask(size, radius)
        for i, on in enumerate(mask):
            if on:
                buf[i * 4 : i * 4 + 4] = bytes(BG)
    else:
        buf[:] = bytes(TRANSPARENT) * (size * size)

    grid = 16
    pad = size // 8 if background else 0
    inner = size - pad * 2
    cell = max(inner // grid, 1)
    art_px = cell * grid
    ox = pad + (inner - art_px) // 2
    oy = pad + (inner - art_px) // 2
    body_min, body_max = body_row_bounds()
    body_h = body_max - body_min + 1
    fill_rows = round((fill_pct / 100.0) * body_h)

    for y, line in enumerate(MASCOT_ART):
        for x, ch in enumerate(line):
            if ch == ".":
                continue
            if ch == "E":
                color = EYE
            else:
                row_from_feet = body_max - y
                color = TERRACOTTA if row_from_feet < fill_rows else CREAM
            for dy in range(cell):
                for dx in range(cell):
                    put_pixel(buf, size, ox + x * cell + dx, oy + y * cell + dy, color)
    return bytes(buf)


def write_png(path: Path, size: int, rgba: bytes) -> None:
    path.write_bytes(png_bytes(size, size, rgba))


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    icons = root / "src-tauri" / "icons"
    icons.mkdir(parents=True, exist_ok=True)
    source = render(1024)
    write_png(root / "src-tauri" / "icons" / "icon-source.png", 1024, source)
    write_png(icons / "32x32.png", 32, render(32))
    write_png(icons / "128x128.png", 128, render(128))
    write_png(icons / "128x128@2x.png", 256, render(256))
    ico_pngs = [(s, png_bytes(s, s, render(s))) for s in (16, 32, 48, 256)]
    (icons / "icon.ico").write_bytes(ico_from_pngs(ico_pngs))
    print(f"Wrote icons to {icons}")


if __name__ == "__main__":
    main()
