# Rebuild the supported ASX DLL against an explicitly reviewed Windows artifact reference.
# Missing references fail before compilation; the local xwin reference is a different builder.
param([string]$ExpectedManifest = '', [string]$TargetDir = '')
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if (-not $ExpectedManifest) {
    $ExpectedManifest = Join-Path $root 'validation/asx-reproducible-build.json'
}
if (-not (Test-Path -LiteralPath $ExpectedManifest -PathType Leaf)) {
    throw 'No reviewed Windows-native ASX reference. The xwin reference cannot substitute for it.'
}
try {
    $expected = Get-Content -LiteralPath $ExpectedManifest -Raw | ConvertFrom-Json
} catch {
    throw 'Invalid reviewed Windows-native ASX reference.'
}
if ($expected.schema -ne 1 -or $expected.game_version -ne '0.125.0' -or
    $expected.artifact -ne 'avrt.dll' -or $expected.builder -ne 'windows-msvc' -or
    $expected.sha256 -cnotmatch '^[0-9a-f]{64}$') {
    throw 'Invalid reviewed Windows-native ASX reference.'
}
Write-Host '[ .... ] reproducible native Windows ASX build'
$builder = Join-Path $root 'tools/build/release.ps1'
if ($TargetDir) {
    & powershell -NoProfile -File $builder -TargetDir $TargetDir
} else {
    & powershell -NoProfile -File $builder
}
if ($LASTEXITCODE -ne 0) { throw "ASX release build failed: $LASTEXITCODE" }
if ($TargetDir) {
    $targetRoot = if ([IO.Path]::IsPathRooted($TargetDir)) { [IO.Path]::GetFullPath($TargetDir) } else { [IO.Path]::GetFullPath((Join-Path $root $TargetDir)) }
} elseif ($env:CARGO_TARGET_DIR) {
    $targetRoot = if ([IO.Path]::IsPathRooted($env:CARGO_TARGET_DIR)) { [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR) } else { [IO.Path]::GetFullPath((Join-Path $root $env:CARGO_TARGET_DIR)) }
} else {
    $targetRoot = Join-Path $root 'target'
}
$file = Join-Path $targetRoot 'x86_64-pc-windows-msvc\release\avrt.dll'
$actual = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -cne $expected.sha256) { throw 'Rebuilt ASX DLL differs from the reviewed Windows-native artifact.' }
Write-Host '[  OK  ] ASX DLL matches the reviewed Windows-native artifact.'
