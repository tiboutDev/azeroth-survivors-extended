# Reproducible native Windows ASX build. Requires the separately reviewed Windows
# manifest through scripts/reproduce.ps1 before it can be treated as reproducible.
param([string]$TargetDir = '')
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
if (-not $TargetDir) {
    if ($env:CARGO_TARGET_DIR) {
        $TargetDir = $env:CARGO_TARGET_DIR
    } else {
        $TargetDir = Join-Path $root 'target'
    }
}
if (-not [IO.Path]::IsPathRooted($TargetDir)) {
    $TargetDir = Join-Path $root $TargetDir
}
$targetRoot = [IO.Path]::GetFullPath($TargetDir)
$cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
$sysroot = (& rustc --print sysroot).Trim()
if ($LASTEXITCODE -ne 0 -or -not $sysroot) { throw 'Cannot identify the Rust sysroot.' }
$flags = @(
    "--remap-path-prefix=$root=/asx",
    "--remap-path-prefix=$cargoHome=/cargo",
    "--remap-path-prefix=$sysroot=/rust",
    '-Cdebuginfo=0',
    '-Clink-arg=/DEBUG:NONE',
    '-Clink-arg=/TIMESTAMP:1'
)
# Cargo prefers CARGO_ENCODED_RUSTFLAGS over RUSTFLAGS. Set both to the same
# reviewed arguments so inherited developer overrides cannot change the build.
$env:CARGO_ENCODED_RUSTFLAGS = [string]::Join([string][char]31, [string[]]$flags)
$env:RUSTFLAGS = [string]::Join(' ', $flags)
$env:CARGO_PROFILE_RELEASE_DEBUG = '0'
$env:CARGO_BUILD_JOBS = '2'
Write-Host '[ .... ] cargo build --release --locked -p azeroth-coop-native (native Windows)'
$t = [Diagnostics.Stopwatch]::StartNew()
& cargo build --release --locked --target x86_64-pc-windows-msvc --target-dir $targetRoot -p azeroth-coop-native --jobs 2 --manifest-path (Join-Path $root 'Cargo.toml')
if ($LASTEXITCODE -ne 0) { Write-Host "[FAILED] cargo build exited with $LASTEXITCODE"; exit 1 }
Write-Host ("[  OK  ] cargo build --release --locked ({0:N1} s)" -f $t.Elapsed.TotalSeconds)
$needles = @($env:USERPROFILE, $env:USERNAME, $root, $cargoHome, $sysroot, $targetRoot) |
    Where-Object { $_ -and $_.Length -ge 4 } | Select-Object -Unique
$file = Join-Path $targetRoot 'x86_64-pc-windows-msvc\release\avrt.dll'
if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw 'The native Windows build did not produce avrt.dll at the expected target path.' }
$bytes = [IO.File]::ReadAllBytes($file)
$ascii = [Text.Encoding]::GetEncoding(28591).GetString($bytes)
$utf16 = [Text.Encoding]::Unicode.GetString($bytes)
$bad = $false
foreach ($needle in $needles) {
    foreach ($variant in @($needle, $needle.Replace('\', '/'))) {
        # Search the encoded bytes through Latin-1 as well, including odd offsets.
        $wideBytes = [Text.Encoding]::GetEncoding(28591).GetString([Text.Encoding]::Unicode.GetBytes($variant))
        if ($ascii.IndexOf($variant, [StringComparison]::OrdinalIgnoreCase) -ge 0 -or
            $ascii.IndexOf($wideBytes, [StringComparison]::OrdinalIgnoreCase) -ge 0 -or
            $utf16.IndexOf($variant, [StringComparison]::OrdinalIgnoreCase) -ge 0) {
            $bad = $true
            Write-Host '[FAILED] avrt.dll contains a local path.'
            break
        }
    }
}
# Live-test switches must never appear in a normal distribution binary.
$wideMarker = [Text.Encoding]::GetEncoding(28591).GetString([Text.Encoding]::Unicode.GetBytes('AZCOOP_TEST_'))
if ($ascii.IndexOf('AZCOOP_TEST_', [StringComparison]::Ordinal) -ge 0 -or
    $ascii.IndexOf($wideMarker, [StringComparison]::Ordinal) -ge 0 -or
    $utf16.IndexOf('AZCOOP_TEST_', [StringComparison]::Ordinal) -ge 0) {
    $bad = $true
    Write-Host '[FAILED] avrt.dll contains a live-test marker.'
}
Write-Host ("[ INFO ] avrt.dll: {0:N0} bytes, sha256 {1}" -f $bytes.Length, (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLower())
if ($bad) { exit 1 }
Write-Host '[  OK  ] no local paths or live-test markers in avrt.dll'
