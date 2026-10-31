"""Hygiene gate for the public audit repo: fails (exit 1) if anything that must not be published is present.

Run from the repo root:  python scripts/hygiene_gate.py
Run it manually before publication; a release must not be published while it fails.

It refuses:
  * local filesystem paths (C:\\Users\\<name>, D:\\..., /home/<name>) that leak the author's machine;
  * private keys / secrets / tokens (PEM private keys, SSH private keys, AWS keys, hard-coded tokens);
  * game assets or executables (the game's own exe/Data/MPQ/audio/texture files) and decompiled game code;
  * references to the private reverse-engineering material (the re-source tree, Ghidra projects);
  * live-test switches (AZCOOP_TEST_*): a real env read that is NOT behind `#[cfg(feature = "test-hooks")]`;
  * non-loopback public server addresses hard-coded as literals.

Allowed: loopback (127.0.0.1, ::1, localhost); the documentation TEST-NET ranges (192.0.2/198.51.100/203.0.113);
RFC1918 LAN (10/172.16-31/192.168); link-local (169.254); CGNAT / Tailscale (100.64.0.0/10); IPv4 multicast
(224-239, e.g. SSDP 239.255.255.250); the public resolvers 8.8.8.8 / 8.8.4.4 (used only to read the local outbound
interface address, no data sent); and ALL-CAPS placeholders such as __CLIENT_SECRET__ or <token>.
"""
import hashlib
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SELF = Path(__file__).resolve()
BINARY_EXT = {'.exe', '.dll', '.mpq', '.bin', '.png', '.jpg', '.jpeg', '.blp', '.wav', '.mp3', '.ogg', '.zip', '.7z',
              '.pdb', '.dmp', '.so', '.dylib', '.ico', '.ttf', '.dat', '.m2', '.wmo', '.adt', '.wdt', '.wdl', '.dbc'}
RESOURCE_PATH = 'resources/windows/version.res'
RESOURCE_SHA256 = 'a6f67c43eab06f32b9cb887f9e7053af4056a58feaa8551fae0dd768d93b1263'

# ASXUI Combat: renamed OFL Barlow derivative, never a native game font.
ASXUI_FONT_PATH = 'addons/ASXUI/Fonts/ASXUI-Numbers.ttf'
ASXUI_FONT_SHA256 = '7957386f307c66a996ff986518a3a28f4039009144b3c8af6f4587ac063238fa'
ASXUI_FONT_LICENSE = 'addons/ASXUI/Fonts/OFL.txt'
ASXUI_FONT_LICENSE_SHA256 = '10b0e6bb43166a50fbe7cde2895bd7f703738a5da843acb5c97c95dc9ea26817'

ASSET_NAME_RULE = re.compile(r'\b\w+\.(?:mpq|blp|m2|wmo|adt|wdt|wdl|dbc)\b', re.I)

# Exact source-line hashes scope exceptions to reviewed textual references.
# A new line/name or stale exception makes this gate fail pending review.
REVIEWED_ASSET_REFERENCE_LINES = {'asx/supported-games.json': {'3c577b33d4e6a040579477b864c361fb18a82d336044d16cf2be837876c3b8c0': 1,
                              '454e059a30b1343fb25703d6d885f1f652a405e0959699a39af829d74c62b623': 1,
                              '4f60db51367fc1da51fba1b0268f7e7d34cbbdbdd9bd238a44158e90d2e90bdb': 1,
                              '56c6a3d1f018fed53effe0a56880d695add8d68f274ce9770d312c7ea26254ab': 1,
                              '7aef9aff33be71b0e36ded42368a306ff90de706cb7a8aecdaadeb543076c3d0': 1,
                              '7e9a714340dc15bf794ea4ceaee78964ff64d73010bc937b4c2a6c1fb4351eef': 1,
                              '94adb605320fb188e74f8f031033e2cc26f0ed5be2a79666493fa9e1f0c4c8ae': 1,
                              '9ef5f521339d22941d2bc4e81e8cec677728aeef96a0ec722bf3f04996116af8': 1,
                              'b30d46266a30b249f17d0f71ade9b24f2febc6af3c07db3a0666272332db88fb': 1,
                              'bf9f25504aed147519744db3c91348c80a94cddc4b192d7da141826c4aaaa426': 1,
                              'f77569ff58e2540b1e76d826f5e01f85545c183d0edc94a7db15688007d2f652': 1},
 'crates/asx-game/src/official_native/treasure_goblins.rs': {'2040f9652bac09b9b9a0ddb7fa6c5dd0774f28e484e1900896c3d3d844bd0fff': 1,
                                                             '38100239d1f35a2defff487dc1c3c85d76bcefe2cdebc371e0df5f50471fbb9b': 2,
                                                             '5930b855d221e6514e550c21e6fe377993308bc40f94f7b2dcf21f884ab22e4c': 1,
                                                             '5a2c009d64056e224badd8753766203006a3c6830f56253fed63a95984e2ec34': 1,
                                                             'cb0abbfea67a3e197c3feaea0d60cf4427fd03371d052e5a8ea12f0c318a0c80': 1,
                                                             'e2326fcb370f64c08d3fa8fcf1e8b7b669beff7b3e263882a22d4b6b701f18b6': 1},
 'tests/test_asx.py': {'0833cc8e0382e0d94a3f161ab643d8321725e3e6a735fc8ccdfd9fafe6b85e41': 1,
                       '25241c357a2ba72da76249e533ed72f2aa049f90ebde4f62d94908c9a793c7d8': 1,
                       '31638acbbd33e5ced8350c4835fc345f5cdf62c9200a502c7bb2d42c0fb567b7': 1,
                       '3bc245cdfc52416e79aae406dd103a5cd44bbd4aeb78fb8ae55b7541ff4e413a': 1,
                       '8db6365b038a3b75742c13d19e129ae36bbd3356f3e971560a1b43dfaeb14e12': 1,
                       '902003f8de23ed368ca65dc37d2e15303b6cec334f192607b046ab10c621e97f': 1,
                       '909b30f21a9a8e16c8787c90eb8189f1b4bdd3a5a8d64cd4cffd866b10be4e84': 1,
                       '987eee303b7ed4d5db512611753c407296f197734eff0a8433989202196cf048': 1,
                       'ae6d4e51b48b613f1d08bb978435e977ccce5999cbea929822e4c1ebd1db6e43': 2,
                       'bc00817d9234576cce10abdd3ff150417114a0215eb6c52a2032d9e519f8008f': 1,
                       'bc65f4411ba8c1dbac35cd5e65bc32b312f613c02902f783ad92949bdecff3f3': 1,
                       'c6e58abcf8e62bd45abf1cc0ee7fc007ddbac56db902cc520cf73d87f7db069a': 1,
                       'c88519b8fcf03d88aa60189927db84c313681728b121cc9073920b2a85c17a65': 1,
                       'cada535360ad0ff6273d08bd35e003aff192a4759e360e5433a02e02a6a0c0c2': 1,
                       'd13c297b698cc4c7eb21aca3dd50a62ac6a0a10fc2fd43d83088fc0498f412c2': 1,
                       'dd39f74ee2d53cd19d33ad4d5620cac457903ac55442ae55b19d8ae05175d180': 1,
                       'e2899e0fcdfb29507821b67fce521a448e3154464b4a24ee6930150b42a37a3b': 1,
                       'ee07538ad31e9c473f632da38e9d10f3784d7222c6ed5cdb98d3ef388552cc87': 1,
                       'f6e4350af6233b9c3c4956c5c8f3add8fabbd369addc9914f758e1fccbffa1f8': 1,
                       'fd4090474d09cf694b3751763c7bed38ef90ce58e6a7610d0a0ef14dddea5b18': 1},
 'tests/test_asx_reproducibility.py': {'234e5ad9377c95c0f68cec45fa84caf39aee75e119d92ada4e5d6292ed5d5a3a': 1},
 'tests/test_asxui_buffs.py': {'29e76b7cd558287c1e43bba669853531e0cb69d601f7e48351c4e25da99d9c5a': 1,
                               '383fc8a3a940e2567fe77645ddfa9a429c8dd28a16cb90f37301244e1b654cd4': 1,
                               '3c20ed41a7024a4bb383885da64cb94c6b598130e083c2bec0e7ea6a15845c73': 1,
                               '88ede70ffc5bf4b300d1cb50b3a875af62967c3fc80c5389cdd964ecec50ab0e': 1,
                               '98db9237fdf146fcc9288dbdcaac5f06b67e2c3d501031a24129f645b26b8bea': 1},
 'tests/test_client_binding.py': {'2dffd64610c8814803de1b9e2fa363e9f0a568d40f5c7d41ee5ae1a6830ce191': 1},
 'tests/test_client_registry.py': {'2514fd6f33cf6910cd57df9dfa1843388f0926104c1d126fd8604652373a4619': 1,
                                   '79cdd467bceca81f5563077a272242eb99365568051c4d1c6b2dc6e13c23e31e': 1},
 'tests/test_native_runtime_import.py': {'45a8f0070ead71b4811813cd87e7cc44f43a8414bb75d19665945d2596e9cc14': 1,
                                         '7e72ab631c9f89bd4a281dbaf6ad9ad6a9d7495f388cf389bca8c3cf8ceda1ac': 1,
                                         '8054eaee4f05dd9a6e43da87868af10c82656b41907c5d52b3eebfaa5362d5fc': 1,
                                         'aac923aee8d5ee75aee442bd82f82ad5dedaf674304165ad020a2169d1b2280d': 1,
                                         'bb26f75061689e9e6fbedfa2dadb08dbd3108ff7c0bb52cfbd9223792f9fa9e0': 1,
                                         'd433d6cf007ebfdf79ab71284bd35df63f2d63efff37a6cbd81cd6d91c889cb5': 1},
 'tests/test_official_party_lua.py': {'0a00467764bb90279de608384f5c7e99d5ac15a55b1ad47dc59f8292002e3225': 1,
                                      'a5604d8df020dea1d9d5f002493fc356003468f4e9d0d698ae57ec47ef0a4649': 1,
                                      'bbadb2a299d31ba42593b65042fe21db4818fa0592a525d5e34dfe994d9745ba': 1,
                                      'c4910c426541d081376e4144dda15ad1df4502bbf33588eb3d277df4d283127d': 1,
                                      'c75e88ba36fee8141136cd924fd2f5af47e69b431f1f50cbd625d5d1576d5819': 1,
                                      'e9fc312477cdba035620859a5db6a7f7e065d33bbcb4eef593e889d45763a4c0': 1,
                                      'f62e6da63d10ce3fa235dd67ecb2977dca159f6f1b03fe4b4380f3f96882f427': 1,
                                      'fa39ad1ec18d0d0ff67859a55630edc4c95ca340359aa78f25281d4eda0f9c29': 1},
 'tests/test_ui_archive.py': {'fba092889029c54141f1a6d61a8cf04bba46659ffcc37e7cd4202773c94952a3': 1},
 'tools/build/asx_package.py': {'5fc49340d2f478585efbd9ed817f7e47ba2ae6124de7560e07604884c4282d09': 1},
 'tools/dev/native_runtime_import.py': {}}
ASSET_REFERENCE_REASONS = {'asx/supported-games.json': 'Supported archive names consumed by the native-folder importer; this manifest '
                             'contains paths only, not archives.',
 'crates/asx-game/src/official_native/treasure_goblins.rs': 'Exact game-provided goblin skin identifiers and '
                                                            'synthetic lookup tests; no skin bytes are '
                                                            'distributed.',
 'tests/test_asx.py': 'Temporary archive names used to verify path ownership and safe installation behavior; '
                      'no game asset bytes are fixtures.',
 'tests/test_asx_reproducibility.py': 'Synthetic archive paths exercise source fingerprint coverage; no '
                                      'archive data.',
 'tests/test_asxui_buffs.py': 'Native icon identifiers in Lua order fixtures; no texture data.',
 'tests/test_client_binding.py': 'The native-image fixture refers to an archive suffix in synthetic PE '
                                 'input; it does not contain a game archive.',
 'tests/test_client_registry.py': 'Archive names verify selected registry contracts; no archive data.',
 'tests/test_native_runtime_import.py': 'Temporary archive names exercise the importer’s path and inventory '
                                        'checks; no archive contents are included.',
 'tests/test_official_party_lua.py': 'UI tests mention texture atlas identifiers returned by the game; no '
                                     'texture assets are included.',
 'tests/test_ui_archive.py': 'Temporary addon asset-name string used to check archive filtering; no image '
                             'data is included.',
 'tools/build/asx_package.py': 'Packager emits an addon manifest referencing native game-provided UI assets; '
                               'it does not package those assets.',
 'tools/dev/native_runtime_import.py': 'Importer recognizes native archive filenames inside the selected '
                                       'game installation and never copies archives into the package.'}

FORBIDDEN = [
    ('windows user path', re.compile(r'[A-Za-z]:\\\\?Users\\\\?[^\s"\']+', re.I), 'leaks a local user profile path'),
    ('D: absolute path', re.compile(r'\bD:\\\\?[A-Za-z0-9 _.\\-]'), 'leaks a local build-machine path'),
    ('unix home path', re.compile(r'/(?:home|Users)/(?!runner\b)[a-z0-9_.-]+/', re.I), 'leaks a local home path'),
    ('PEM private key', re.compile(r'-----BEGIN (?:RSA |EC |OPENSSH |DSA |PGP )?PRIVATE KEY-----'), 'private key'),
    ('AWS access key', re.compile(r'\b(?:AKIA|ASIA)[0-9A-Z]{16}\b'), 'AWS access key id'),
    ('AWS secret', re.compile(r'aws_secret_access_key\s*=\s*\S+', re.I), 'AWS secret'),
    ('bearer/token literal', re.compile(r'(?:secret|token|password|passwd|api[_-]?key)\s*[=:]\s*["\'](?!__[A-Z0-9_]+__)(?!<)[^"\']{12,}["\']', re.I), 'possible hard-coded secret'),
    ('re-source reference', re.compile(r're-source|AzerothCoopNativee?-source|ghidra|\.gpr\b', re.I), 'private reverse-engineering material'),
    ('game asset name', ASSET_NAME_RULE, 'game asset filename'),
]
IP = re.compile(r'\b(?:\d{1,3}\.){3}\d{1,3}\b')
ALLOWED_IP_PREFIX = ('127.', '0.0.0.0', '255.', '192.0.2.', '198.51.100.', '203.0.113.', '169.254.',
                     '10.', '192.168.', '8.8.8.8', '8.8.4.4') + tuple(f'172.{n}.' for n in range(16, 32))
TESTHOOK = re.compile(r'(?:env::var|std::env::var|getenv|environ(?:\.get)?\(|GetEnvironmentVariable)[^\n]{0,40}AZCOOP_TEST_')
FEATURE_GUARD = re.compile(r'feature\s*=\s*"test-hooks"')


def publication_files():
    """Inspect tracked and unignored new content, never private ignored caches."""
    raw = subprocess.check_output(['git', '-C', str(ROOT), 'ls-files', '--cached', '--others', '--exclude-standard', '-z'])
    return sorted({ROOT / name.decode('utf-8', 'surrogateescape') for name in raw.split(b'\0') if name})


def ip_allowed(ip):
    if ip.startswith(ALLOWED_IP_PREFIX):
        return True
    try:
        o = [int(x) for x in ip.split('.')]
    except ValueError:
        return True
    if not all(0 <= x <= 255 for x in o):
        return True  # not a real IPv4 (e.g. a version number)
    if o[0] == 100 and 64 <= o[1] <= 127:
        return True  # CGNAT / Tailscale 100.64.0.0/10
    if 224 <= o[0] <= 239:
        return True  # IPv4 multicast (SSDP/UPnP)
    return False


def gated_module_files():
    """Rust source files whose module is declared behind #[cfg(feature = "test-hooks")] in a lib.rs/mod.rs
    (e.g. `#[cfg(feature = "test-hooks")] pub mod stress;` gates all of stress.rs)."""
    stems = set()
    decl = re.compile(r'(?:^|\])\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([a-z0-9_]+)\s*;')
    for parent in publication_files():
        if parent.suffix != '.rs' or not parent.is_file() or parent.is_symlink():
            continue
        lines = parent.read_text(encoding='utf-8', errors='replace').splitlines()
        for n, line in enumerate(lines):
            m = decl.search(line)
            if m and any(FEATURE_GUARD.search(lines[k]) for k in range(max(0, n - 3), n + 1)):
                directory = parent.parent if parent.name in {'lib.rs', 'mod.rs'} else parent.with_suffix('')
                stems.add((directory / f'{m.group(1)}.rs').resolve())
                stems.add((directory / m.group(1) / 'mod.rs').resolve())
    return stems


def reviewed_asset_reference(rel, line):
    """Allow only exact, counted text lines whose purpose was reviewed."""
    digest = hashlib.sha256(line.encode('utf-8')).hexdigest()
    return digest in REVIEWED_ASSET_REFERENCE_LINES.get(rel, {})


def reviewed_reference_counts_match(observed):
    """Every exception must occur the reviewed number of times; reject stale entries."""
    expected = {
        (path, digest): count
        for path, lines in REVIEWED_ASSET_REFERENCE_LINES.items()
        for digest, count in lines.items()
    }
    return expected == dict(observed)


def approved_embedded_resource(rel, raw):
    """The sole accepted .res is the committed build input with its reviewed bytes."""
    return rel == RESOURCE_PATH and hashlib.sha256(raw).hexdigest() == RESOURCE_SHA256


def forbidden_binary(rel, suffix, raw=None):
    suffix = suffix.lower()
    if suffix == '.res':
        return not approved_embedded_resource(rel, raw or b'')
    if rel == ASXUI_FONT_PATH and suffix == '.ttf':
        license_path = ROOT / ASXUI_FONT_LICENSE
        return not (hashlib.sha256(raw or b'').hexdigest() == ASXUI_FONT_SHA256
                    and license_path.is_file()
                    and hashlib.sha256(license_path.read_bytes()).hexdigest() == ASXUI_FONT_LICENSE_SHA256)
    return suffix in BINARY_EXT


def scan():
    hits, testhooks = [], []
    asset_reference_counts = {}
    gated = gated_module_files()
    for path in publication_files():
        if path.is_symlink():
            hits.append((path.relative_to(ROOT).as_posix(), 0, 'symbolic link', 'publication must not depend on external local content'))
            continue
        if not path.is_file() or path.resolve() == SELF:
            continue  # skip VCS, Python caches, and build output (target/ is gitignored, not repo content)
        rel = path.relative_to(ROOT).as_posix()
        suffix = path.suffix.lower()
        resource_bytes = None
        if suffix == '.res':
            try:
                resource_bytes = path.read_bytes()
            except OSError:
                resource_bytes = b''
            if approved_embedded_resource(rel, resource_bytes):
                continue
            hits.append((rel, 0, 'binary/asset file', 'unreviewed Windows resource file'))
            continue
        if rel == ASXUI_FONT_PATH and suffix == '.ttf':
            raw = path.read_bytes()
            if not forbidden_binary(rel, suffix, raw):
                continue
        if forbidden_binary(rel, suffix, resource_bytes):
            note = f'a {suffix} file must not be in the audit repo'
            hits.append((rel, 0, 'binary/asset file', note))
            continue
        try:
            lines = path.read_text(encoding='utf-8').splitlines()
        except (UnicodeDecodeError, OSError):
            hits.append((rel, 0, 'non-text file', 'not valid UTF-8; a binary slipped in'))
            continue
        for n, line in enumerate(lines, 1):
            reviewed_asset = False
            if ASSET_NAME_RULE.search(line):
                digest = hashlib.sha256(line.encode('utf-8')).hexdigest()
                count = REVIEWED_ASSET_REFERENCE_LINES.get(rel, {}).get(digest)
                if count is not None:
                    key = (rel, digest)
                    asset_reference_counts[key] = asset_reference_counts.get(key, 0) + 1
                    reviewed_asset = True
            for name, rx, note in FORBIDDEN:
                if name == 'game asset name' and reviewed_asset:
                    continue
                if rx.search(line):
                    hits.append((rel, n, name, note))
            for m in IP.finditer(line):
                if not ip_allowed(m.group(0)):
                    hits.append((rel, n, 'non-loopback IP literal', f'{m.group(0)}: a real server address must not be hard-coded'))
            if TESTHOOK.search(line):
                guarded = (path.resolve() in gated
                           or any(FEATURE_GUARD.search(lines[k]) for k in range(0, n)))
                testhooks.append((rel, n, line.strip()[:100], guarded))
    if not reviewed_reference_counts_match(asset_reference_counts):
        expected_counts = {
            (path, digest): count
            for path, lines in REVIEWED_ASSET_REFERENCE_LINES.items()
            for digest, count in lines.items()
        }
        expected = set(expected_counts)
        seen = set(asset_reference_counts)
        for path, digest in sorted(expected - seen):
            hits.append((path, 0, 'stale asset-name exception', 'Reviewed reference changed or disappeared; re-audit and update the exact-line inventory'))
        for path, digest in sorted(seen - expected):
            hits.append((path, 0, 'unreviewed asset-name exception', 'An asset-name line is not in the reviewed exact-line inventory'))
        for path, digest in sorted(seen & expected):
            if asset_reference_counts[(path, digest)] != expected_counts[(path, digest)]:
                hits.append((path, 0, 'asset-name reference count mismatch', 'A reviewed line occurs a different number of times; re-audit its uses'))
    return hits, testhooks


def main():
    sys.stdout.reconfigure(encoding='utf-8', line_buffering=True)
    print('[ .... ] hygiene gate: scanning the audit repo for content that must not be published')
    hits, testhooks = scan()
    unguarded = [t for t in testhooks if not t[3]]
    for rel, n, name, note in hits:
        print(f'[FAILED] {rel}:{n}: {name} — {note}')
    for rel, n, line, _ in unguarded:
        print(f'[FAILED] {rel}:{n}: AZCOOP_TEST_ env read outside a test-hooks feature guard — {line}')
    guarded = [t for t in testhooks if t[3]]
    if guarded:
        print(f'[ INFO ] {len(guarded)} AZCOOP_TEST_ env read(s) behind #[cfg(feature="test-hooks")] (never shipped): '
              + ', '.join(f'{r}:{n}' for r, n, _, _ in guarded))
    total = len(hits) + len(unguarded)
    if total:
        print(f'[FAILED] hygiene gate: {total} issue(s); the repo is NOT clean to publish')
        return 1
    files = sum(1 for p in publication_files() if p.is_file())
    print(f'[  OK  ] hygiene gate: {files} files, no local paths, secrets, keys, game assets, RE material, or unguarded test hooks')
    return 0


if __name__ == '__main__':
    sys.exit(main())
