"""Bounded Windows x64 calls on the pinned review image, entirely inside Unicorn."""
import math
import struct

from unicorn import Uc, UC_ARCH_X86, UC_MODE_64, UC_HOOK_CODE, UC_HOOK_MEM_UNMAPPED
from unicorn import x86_const as reg


class ReviewMachine:
    scratch = 0x1000000
    memory_size = 0x800000
    max_allocation = 65536
    instruction_limit = 2000000
    stop = scratch
    stack = scratch + 0x380008  # Win64 callee entry alignment

    def __init__(self, image):
        self.image = image
        self.base = image.pe.OPTIONAL_HEADER.ImageBase
        self.end = self.base + image.pe.OPTIONAL_HEADER.SizeOfImage
        self.cpu = Uc(UC_ARCH_X86, UC_MODE_64)
        self.cpu.mem_map(self.scratch, self.memory_size)
        self.cpu.hook_add(UC_HOOK_MEM_UNMAPPED, self.map_page)
        self.heap = self.scratch + 0x400000
        self.allocations = {}
        self.local_packets = []
        # These are import thunks. All game code, including field initialization,
        # runs from the supplied PE. The emulator stack is already committed.
        if image.sha == "eb5fd1c1b96e6180fcf5f3d685d90a61471af28d22e5f060c02b7b7663fbf79e":
            crt = (0x613303b, 0x6133041, 0x6133047, 0x613304d, 0x6133071, 0x6133077,
                   0x6133095, 0x61330a1, 0x61330b3, 0x61324d0)
        elif image.sha == "0bb6c557c5f6cfeea212761a33554f9a5647ce7cb7e03b8a163b95489496e908":
            crt = (0x634b1ab, 0x634b1b1, 0x634b1b7, 0x634b1bd, 0x634b1e1, 0x634b1e7,
                   0x634b20b, 0x634b1f3, 0x634b22f, 0x634a670)
        else:
            raise ValueError("Unknown offline executable")
        self.services = dict(zip(crt, ("copy", "copy", "compare", "fill", "sin", "cos",
                                       "ceil", "round", "floor", "stack")))
        if crt[0] == 0x634b1ab:
            self.services[0x60519b0] = "local transport"
        for address in (0x21c0, 0x21d0, 0x21e0, 0x21f0, *self.services):
            at = self.base + address
            self.cpu.hook_add(UC_HOOK_CODE, self.service, begin=at, end=at)

    def map_page(self, cpu, access, address, size, value, user):
        page = address & ~4095
        if not self.base <= page < self.end:
            return False
        cpu.mem_map(page, 4096)
        cpu.mem_write(page, self.image.pe.get_data(page - self.base, 4096))
        return True

    def write(self, at, fmt, *values):
        self.cpu.mem_write(at, struct.pack("<" + fmt, *values))

    def read(self, at, fmt):
        return struct.unpack("<" + fmt, self.cpu.mem_read(at, struct.calcsize("<" + fmt)))

    def buffer(self, header):
        cap, pointer, size = self.read(header, "3Q")
        assert size <= cap <= 65536
        return bytes(self.cpu.mem_read(pointer, size))

    def allocate(self, size, align):
        assert 0 < size <= self.max_allocation and align in (1, 2, 4, 8, 16, 32)
        pointer = (self.heap + align - 1) & -align
        self.heap = pointer + size
        assert self.heap < self.scratch + self.memory_size
        self.allocations[pointer] = (size, align)
        return pointer

    def service(self, cpu, address, size, user):
        rva = address - self.base
        first, second, third, fourth = (cpu.reg_read(r) for r in
            (reg.UC_X86_REG_RCX, reg.UC_X86_REG_RDX, reg.UC_X86_REG_R8, reg.UC_X86_REG_R9))
        if rva in (0x21c0, 0x21f0):
            cpu.reg_write(reg.UC_X86_REG_RAX, self.allocate(first, second))
        elif rva == 0x21d0:
            assert self.allocations.pop(first) == (second, third)
        elif rva == 0x21e0:
            assert self.allocations.pop(first) == (second, third)
            pointer = self.allocate(fourth, third)
            cpu.mem_write(pointer, bytes(cpu.mem_read(first, min(second, fourth))))
            cpu.reg_write(reg.UC_X86_REG_RAX, pointer)
        elif self.services[rva] in ("copy", "compare", "fill"):
            assert third <= 65536
            op = self.services[rva]
            result = first
            if op == "compare":
                a, b = self.bytes_at(first, third), self.bytes_at(second, third)
                result = (a > b) - (a < b)
            elif op == "fill":
                cpu.mem_write(first, bytes([second & 255]) * third)
            else:
                cpu.mem_write(first, self.bytes_at(second, third))
            cpu.reg_write(reg.UC_X86_REG_RAX, result)
        elif self.services[rva] == "local transport":
            assert third <= 65536
            self.local_packets.append((first, bytes(cpu.mem_read(second, third))))
        elif self.services[rva] != "stack":
            bits = cpu.reg_read(reg.UC_X86_REG_XMM0) & 0xffffffff
            operand, = struct.unpack("<f", struct.pack("<I", bits))
            op = self.services[rva]
            if op == "round":
                value = math.copysign(math.floor(abs(operand) + 0.5), operand)
            else:
                value = getattr(math, op)(operand)
                if op in ("ceil", "floor") and value == 0:
                    value = math.copysign(0.0, operand)
            cpu.reg_write(reg.UC_X86_REG_XMM0, struct.unpack("<I", struct.pack("<f", value))[0])
        stack = cpu.reg_read(reg.UC_X86_REG_RSP)
        cpu.reg_write(reg.UC_X86_REG_RIP, self.read(stack, "Q")[0])
        cpu.reg_write(reg.UC_X86_REG_RSP, stack + 8)

    def bytes_at(self, address, size):
        if self.base <= address < self.end:
            return self.image.pe.get_data(address - self.base, size)
        return bytes(self.cpu.mem_read(address, size)) if size else b""

    def call(self, rva, *args):
        self.write(self.stack, "Q", self.stop)
        registers = (reg.UC_X86_REG_RCX, reg.UC_X86_REG_RDX, reg.UC_X86_REG_R8, reg.UC_X86_REG_R9)
        for index, value in enumerate(args):
            if index < 4:
                self.cpu.reg_write(registers[index], value)
            else:
                self.write(self.stack + 40 + 8 * (index - 4), "Q", value)
        self.cpu.reg_write(reg.UC_X86_REG_RSP, self.stack)
        self.cpu.emu_start(self.base + rva, self.stop, count=self.instruction_limit)
        assert self.cpu.reg_read(reg.UC_X86_REG_RIP) == self.stop
        return self.cpu.reg_read(reg.UC_X86_REG_RAX)


class CatalogMachine(ReviewMachine):
    """Native catalog initialization with no host filesystem, TLS or entropy dependencies."""
    memory_size = 0x2000000
    max_allocation = 0x100000
    instruction_limit = 200000000

    def __init__(self, image):
        super().__init__(image)
        import pefile
        pe = image.pe
        pe.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY[name] for name in
                                               ("IMAGE_DIRECTORY_ENTRY_TLS", "IMAGE_DIRECTORY_ENTRY_IMPORT")])
        tls = pe.DIRECTORY_ENTRY_TLS.struct
        teb = self.scratch + 0x100000
        storage = teb + 0x2000
        self.cpu.reg_write(reg.UC_X86_REG_GS_BASE, teb)
        self.write(teb + 0x58, "Q", teb + 0x1000)
        self.write(teb + 0x1000, "Q", storage)
        self.cpu.mem_write(storage, pe.get_data(tls.StartAddressOfRawData - self.base,
                                                tls.EndAddressOfRawData - tls.StartAddressOfRawData))
        self.image_word(tls.AddressOfIndex, "I", 0)
        self.last_error = 0
        services = {b"SetLastError", b"GetLastError", b"GetEnvironmentVariableW", b"ProcessPrng",
                    b"GetModuleFileNameW", b"CreateFileW", b"trunc", b"round", b"powf", b"atan2f", b"fmodf"}
        address = teb + 0x6000
        for dll in pe.DIRECTORY_ENTRY_IMPORT:
            for imp in dll.imports:
                if imp.name in services:
                    self.image_word(imp.address, "Q", address)
                    self.cpu.mem_write(address, bytes([0xc3]))
                    self.cpu.hook_add(UC_HOOK_CODE, self.catalog_import, imp.name, begin=address, end=address)
                    address += 16

    def image_word(self, address, fmt, value):
        if not any(first <= address <= last for first, last, _ in self.cpu.mem_regions()):
            assert self.map_page(self.cpu, 0, address, 8, 0, None)
        self.write(address, fmt, value)

    def catalog_import(self, cpu, address, size, name):
        first, second, third = [cpu.reg_read(r) for r in (reg.UC_X86_REG_RCX, reg.UC_X86_REG_RDX, reg.UC_X86_REG_R8)]
        if name == b"ProcessPrng":
            assert second <= 64
            cpu.mem_write(first, bytes((i*51 + 9) % 256 for i in range(second)))
            result = 1
        elif name == b"SetLastError":
            self.last_error = first
            result = 0
        elif name == b"GetLastError":
            result = self.last_error
        elif name == b"GetEnvironmentVariableW":
            self.last_error = 203
            result = 0
        elif name == b"GetModuleFileNameW":
            path = "C:/AZSX-trial/AzerothSurvivors.exe"
            assert first == 0 and third > len(path)
            cpu.mem_write(second, (path + "\0").encode("utf-16-le"))
            result = len(path)
        elif name == b"CreateFileW":
            # External translations and the optional game-data directory are absent in this fixture.
            path = bytes(cpu.mem_read(first, 512)).decode("utf-16-le", errors="replace").split("\0")[0]
            normalized = path.replace("\\", "/").lower()
            assert not normalized or normalized.endswith("/data") or (normalized.startswith("c:/azsx-trial/lang/") and normalized.endswith(".json"))
            self.last_error = 2
            result = (1 << 64) - 1
        else:
            def scalar(register, fmt):
                length = struct.calcsize(fmt)
                bits = cpu.reg_read(register) & ((1 << (8 * length)) - 1)
                return struct.unpack(fmt, bits.to_bytes(length, "little"))[0]
            if name in (b"trunc", b"round"):
                fmt = "<d"
                x = scalar(reg.UC_X86_REG_XMM0, fmt)
                result = math.trunc(x) if name == b"trunc" else math.copysign(math.floor(abs(x) + 0.5), x)
            else:
                fmt = "<f"
                function = {b"powf": math.pow, b"atan2f": math.atan2, b"fmodf": math.fmod}[name]
                result = function(scalar(reg.UC_X86_REG_XMM0, fmt), scalar(reg.UC_X86_REG_XMM1, fmt))
            cpu.reg_write(reg.UC_X86_REG_XMM0, int.from_bytes(struct.pack(fmt, result), "little"))
            return
        cpu.reg_write(reg.UC_X86_REG_RAX, result)
