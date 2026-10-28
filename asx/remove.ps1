param([Parameter(Mandatory=$true)][string]$JobPath)
$ErrorActionPreference = 'Stop'
$removed = @(); $preserved = @(); $failures = @(); $installationLock = $null
function Get-ASXHash([string]$Path) {
    $stream = [IO.File]::OpenRead($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try { return [BitConverter]::ToString($hasher.ComputeHash($stream)).Replace('-','').ToLowerInvariant() }
    finally { $hasher.Dispose(); $stream.Dispose() }
}
try {
    $job = Get-Content -LiteralPath $JobPath -Raw | ConvertFrom-Json
    $root = [IO.Path]::GetFullPath($job.root).TrimEnd('\')
    $prefix = $root + '\'
    $owner = Get-Process -Id $job.pid -ErrorAction SilentlyContinue
    if ($owner -and -not $owner.WaitForExit(30000)) { throw 'The launcher is still running; removal stopped.' }
    Start-Sleep -Milliseconds 500
    $game = Get-Process -Name AzerothSurvivors -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq (Join-Path $root 'AzerothSurvivors.exe') }
    if ($game) { throw 'The game is running; removal stopped.' }
    $lockPath = Join-Path $root 'ASX\State\installation.lock'
    $installationLock = [IO.File]::Open($lockPath,[IO.FileMode]::Open,[IO.FileAccess]::ReadWrite,[IO.FileShare]::ReadWrite)
    $installationLock.Lock(0,1)
    $parents = @{}
    foreach ($property in $job.files.PSObject.Properties) {
        $name = [string]$property.Name
        if ($name -eq 'ASX/State/installation.lock') { continue }
        if ($name -match '(^|/)(\.\.?|[^/]*[:\\*?"<>|][^/]*)(/|$)' -or $name -match '(^|/)/') { throw 'Unsafe ASX removal path.' }
        $allowed = $name -in @('start-asx.cmd','README-ASX.txt','avrt.dll','ASX/manifest.json','ASX/State/install.json','ASX/State/installation.lock') -or $name -match '^ASX/(Client|Bin|Runtime|Interface|Defaults)/' -or $name -in @('benilla-config/AddOns/ASX/ASX.toc','benilla-config/AddOns/ASX/Presentation.lua')
        if (-not $allowed) { throw 'Removal inventory contains a non-ASX path.' }
        $target = [IO.Path]::GetFullPath((Join-Path $root $name.Replace('/','\')))
        if (-not $target.StartsWith($prefix,[StringComparison]::OrdinalIgnoreCase)) { throw 'Removal path escaped the game folder.' }
        $cursor = $target
        while ($cursor -ne $root) {
            if (Test-Path -LiteralPath $cursor) {
                if ((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Removal will not follow links.' }
            }
            $cursor = [IO.Path]::GetDirectoryName($cursor)
        }
        if (Test-Path -LiteralPath $target -PathType Leaf) {
            if ((Get-ASXHash $target) -eq $property.Value) {
                try { Remove-Item -LiteralPath $target -Force; $removed += $name }
                catch { $failures += $name + ': ' + $_.Exception.Message }
            } else { $preserved += $name }
        }
        $cursor = [IO.Path]::GetDirectoryName($target)
        while ($cursor.StartsWith((Join-Path $root 'ASX'),[StringComparison]::OrdinalIgnoreCase) -or $cursor.StartsWith((Join-Path $root 'benilla-config\AddOns\ASX'),[StringComparison]::OrdinalIgnoreCase)) {
            $parents[$cursor] = $true
            $cursor = [IO.Path]::GetDirectoryName($cursor)
        }
    }
    foreach ($directory in ($parents.Keys | Sort-Object Length -Descending)) {
        if ((Test-Path -LiteralPath $directory -PathType Container) -and [IO.Directory]::GetFileSystemEntries($directory).Length -eq 0) { Remove-Item -LiteralPath $directory }
    }
} catch { $failures += $_.Exception.Message }
finally { if ($installationLock) { try { $installationLock.Unlock(0,1) } catch {}; $installationLock.Dispose() } }
@{ removed=$removed; preserved=$preserved; errors=$failures; completed=($failures.Count -eq 0) } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path ([IO.Path]::GetDirectoryName($JobPath)) 'result.json') -Encoding UTF8
