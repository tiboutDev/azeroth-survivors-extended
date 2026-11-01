#!/usr/bin/env python3
"""Read-only, exact-image native balance reference generator (requires pefile)."""
import argparse
import hashlib
import json
import math
import re
import struct
from pathlib import Path

VERSION = "0.135.1"
SHA256 = "71e5f1767279db4bd428d1cd9c5d50d37759b72295386a69f9e3f6e70bc59249"
STAT_NAMES = {0: "Damage", 1: "Cooldown reduction", 2: "Maximum health",
              3: "Critical chance", 4: "Critical damage", 5: "Damage reduction",
              6: "Health per second", 7: "Movement speed", 11: "Gold",
              12: "Luck", 13: "Echo", 14: "Lifesteal", 15: "Heal per kill", 22: "Thorns"}


class Image:
    def __init__(self, blob):
        if hashlib.sha256(blob).hexdigest() != SHA256:
            raise ValueError("Unsupported native executable: review a new schema before extraction")
        import pefile
        pe = pefile.PE(data=blob, fast_load=True)
        self.blob, self.base = blob, pe.OPTIONAL_HEADER.ImageBase
        self.sections = [(s.VirtualAddress, s.SizeOfRawData, s.PointerToRawData) for s in pe.sections]
        pe.close()

    def read(self, rva, size):
        if rva < 0 or not 0 <= size <= 1048576:
            raise ValueError("Invalid read")
        for va, length, offset in self.sections:
            if va <= rva and rva + size <= va + length:
                result = self.blob[offset + rva - va:offset + rva - va + size]
                if len(result) == size:
                    return result
        raise ValueError("Read outside file-backed sections")

    def string(self, rva):
        pointer, length = struct.unpack("<QQ", self.read(rva, 16))
        if not 0 < length <= 8192:
            raise ValueError("Invalid native string length")
        return self.read(pointer - self.base, length).decode("utf8")


def effect(raw):
    if len(raw) != 64:
        raise ValueError("Truncated native effect")
    result = {"tag": raw[0], "raw_hex": raw.hex(), "status": "Reviewed common special effect; see reference and native_balance_math"}
    if raw[0] == 43:
        value, kind = struct.unpack_from("<fI", raw, 4)
        if not math.isfinite(value):
            raise ValueError("Non-finite coefficient")
        result.update(stat_id=kind, stat=STAT_NAMES.get(kind, "Unresolved stat"),
                      coefficient=value, status="Base stat coefficient; multiplied by amulet empowerment factor")
    return result


def native_stats(blob):
    start = blob.index(b"function AZS_Update()")
    end = blob.index(b"-- Each stock setter", start)
    lua = blob[start:end].decode("utf8")
    rows = []
    patterns = [r'AZS_Row\("[^"]+", "([^"]+)", (.*?),\s*"([^"]+)"',
                r'\{ "([^"]+)", (.*?), "([^"]+)"']
    for pattern in patterns:
        for name, expression, description in re.findall(pattern, lua):
            keys = re.findall(r's\.(\w+)', expression)
            if len(keys) != 1:
                raise ValueError("Ambiguous native stat expression")
            rows.append(dict(key=keys[0], name=name, display_expression=expression,
                             description=description))
    if len(rows) != 29 or len({r['key'] for r in rows}) != 29:
        raise ValueError("Native stat coverage changed")
    return rows


def extract(blob):
    image = Image(blob)
    result = {"schema_version": 2, "native_version": VERSION, "executable_sha256": SHA256,
              "stats": native_stats(blob), "jewelry": [], "amulets": []}
    for i in range(23):
        rva = 0x747c720 + i * 72
        value, kind, slot = struct.unpack("<fBB", image.read(rva + 64, 6))
        if not math.isfinite(value) or slot != (0 if i < 9 else 1):
            raise ValueError("Jewelry schema mismatch")
        result['jewelry'].append(dict(key=image.string(rva), name=image.string(rva + 16),
            icon=image.string(rva + 32), description_template=image.string(rva + 48),
            coefficient=value, stat_kind=kind, slot="Ring" if slot == 0 else "Trinket", rva=hex(rva)))
    for i in range(16):
        rva = 0x747d980 + i * 80
        pointer, count = struct.unpack("<QQ", image.read(rva + 48, 16))
        if not 1 <= count <= 16:
            raise ValueError("Invalid amulet effect count")
        effects = []
        for j in range(count):
            at = pointer - image.base + j * 64
            effects.append(dict(effect(image.read(at, 64)), rva=hex(at)))
        result['amulets'].append(dict(key=image.string(rva), name=image.string(rva + 16),
            icon=image.string(rva + 32), effects=effects, rarity=struct.unpack("<Q", image.read(rva+72,8))[0], rva=hex(rva)))
    return result


def clean(text):
    return str(text).replace('|', '\\|').replace('\n', ' ')


def number(value):
    return format(value, '.7g')


from native_balance_reference import render, rank_tables


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    data = extract(args.exe.read_bytes())
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / 'native-balance.json').write_text(json.dumps(data, indent=2, ensure_ascii=False)+'\n', encoding='utf8')
    (args.output / 'NATIVE-BALANCE.md').write_text(render(data), encoding='utf8')
    for name, text in rank_tables(data).items():
        (args.output / name).write_text(text, encoding="utf8")
    print(f"Extracted {len(data['stats'])} stats, {len(data['jewelry'])} rings/trinkets, {len(data['amulets'])} amulets")


if __name__ == '__main__':
    main()
