"""Read-only comparison of native hook functions; never approves an unsupported build."""
import os
import bisect
import hashlib
import json
from pathlib import Path
import re
import struct
import capstone
import pefile

ROOT = Path(__file__).resolve().parents[2]
old = pefile.PE(str(ROOT / 'runtime/native/AzerothSurvivors.exe'), fast_load=True)
source = Path(os.environ["ASX_LEGACY_GAME"])
new = pefile.PE(str(source), fast_load=True)
profile = json.loads((ROOT / 'runtime/native/reconstruction/release-19260245.json').read_text())
md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_64)
md.detail = True


def bounds(pe):
    d = pe.OPTIONAL_HEADER.DATA_DIRECTORY[3]
    return sorted((a, b) for a, b, _ in struct.iter_unpack('<III', pe.get_data(d.VirtualAddress, d.Size)))


def normalized(code, rva):
    data = bytearray(code)
    consumed = 0
    for ins in md.disasm(code, rva):
        start = ins.address - rva
        if (ins.group(capstone.CS_GRP_CALL) or ins.group(capstone.CS_GRP_JUMP)) and ins.imm_size:
            data[start + ins.imm_offset:start + ins.imm_offset + ins.imm_size] = bytes(ins.imm_size)
        if any(op.type == capstone.x86.X86_OP_MEM and op.mem.base == capstone.x86.X86_REG_RIP for op in ins.operands):
            data[start + ins.disp_offset:start + ins.disp_offset + ins.disp_size] = bytes(ins.disp_size)
        consumed += ins.size
    return bytes(data) if consumed == len(code) else None


old_bounds, new_bounds = bounds(old), bounds(new)
old_starts = [a for a, _ in old_bounds]
sections = [(s.VirtualAddress, s.get_data()) for s in new.sections if s.Characteristics & 0x20000000]
targets = {row['candidate'] for row in profile['adapter_profile']['addresses']}
groups = {}
for address in targets:
    index = bisect.bisect_right(old_starts, address) - 1
    if index >= 0 and address < old_bounds[index][1]:
        groups.setdefault(old_bounds[index], []).append(address)

rows = []
for (start, end), addresses in groups.items():
    code = old.get_data(start, end - start)
    norm = normalized(code, start)
    candidates = []
    if norm is not None:
        # The full normalized function is checked, including all structure offsets and constants.
        pattern = b''.join(b'.' if a != b else re.escape(bytes([a])) for a, b in zip(code[:96], norm[:96]))
        for base, data in sections:
            for hit in re.finditer(pattern, data, re.DOTALL):
                rva = base + hit.start()
                if normalized(new.get_data(rva, len(code)), rva) == norm:
                    candidates.append(rva)
    rows.append(dict(old=start, size=end - start, targets=addresses, candidates=candidates,
                     status='instruction_equivalent' if len(candidates) == 1 else 'unresolved'))
    print(f'{start:#x}: {rows[-1]["status"]} ({len(candidates)} candidates)', flush=True)
data = source.read_bytes()
version = re.search(rb'(0\.[0-9]+\.[0-9]+).{0,4}update: up to date', data)
report = dict(source=str(source), sha256=hashlib.sha256(data).hexdigest(), size=len(data),
              version=version[1].decode() if version else None, approved=False,
              reason='Function equivalence does not prove data targets, changed functions, or game layout compatibility.', functions=rows)
out = ROOT / 'test-results/game-compatibility.json'
out.write_text(json.dumps(report, indent=2))
print(json.dumps(dict(version=report['version'], functions=len(rows), unresolved=sum(r['status'] == 'unresolved' for r in rows), approved=False)))
