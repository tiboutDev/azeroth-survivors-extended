param([Parameter(Mandatory=$true)][string]$JobPath)
$ErrorActionPreference = 'Stop'
$removed = @(); $preserved = @(); $failures = @(); $installationLock = $null
function Get-AZSXHash([string]$Path) {
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
    $lockPath = Join-Path $root 'AZSX\Data\State\installation.lock'
    $installationLock = [IO.File]::Open($lockPath,[IO.FileMode]::Open,[IO.FileAccess]::ReadWrite,[IO.FileShare]::ReadWrite)
    $installationLock.Lock(0,1)
    $parents = @{}
    foreach ($property in $job.files.PSObject.Properties) {
        $name = [string]$property.Name
        if ($name -eq 'AZSX/Data/State/installation.lock') { continue }
        if ($name -match '(^|/)(\.\.?|[^/]*[:\\*?"<>|][^/]*)(/|$)' -or $name -match '(^|/)/') { throw 'Unsafe AZSX removal path.' }
        $allowed = $name -in @('start-azsx.cmd','README-AZSX.txt','avrt.dll','AZSX/App/manifest.json','AZSX/Data/State/install.json','AZSX/Data/State/installation.lock') -or $name -match '^AZSX/App/(Launcher|Bin|Python|AddOns|Defaults)/' -or $name -in @('benilla-config/AddOns/AZSX/AZSX.toc','benilla-config/AddOns/AZSX/Presentation.lua','benilla-config/AddOns/AZSXUI/AZSXUI.toc','benilla-config/AddOns/AZSXUI/AZSXUI.lua','benilla-config/AddOns/AZSXUI/Bindings.xml','benilla-config/AddOns/AZSXUI/Camera.lua','benilla-config/AddOns/AZSXUI/Minimap.lua','benilla-config/AddOns/AZSXUI/Events.lua','benilla-config/AddOns/AZSXUI/Buffs.lua','benilla-config/AddOns/AZSXUI/Hud.lua','benilla-config/AddOns/AZSXUI/Theme.lua','benilla-config/AddOns/AZSXUI/Frames.lua','benilla-config/AddOns/AZSXUI/Fonts/AZSXUI-Numbers.ttf','benilla-config/AddOns/AZSXUI/Fonts/OFL.txt','benilla-config/AddOns/AZSXUI/LICENSE','benilla-config/AddOns/AZSXUI/README.md')
        if (-not $allowed) { throw 'Removal inventory contains a non-AZSX path.' }
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
            if ((Get-AZSXHash $target) -eq $property.Value) {
                try { Remove-Item -LiteralPath $target -Force; $removed += $name }
                catch { $failures += $name + ': ' + $_.Exception.Message }
            } else { $preserved += $name }
        }
        $cursor = [IO.Path]::GetDirectoryName($target)
        while ($cursor.StartsWith((Join-Path $root 'AZSX'),[StringComparison]::OrdinalIgnoreCase) -or $cursor.StartsWith((Join-Path $root 'benilla-config\AddOns\AZSX'),[StringComparison]::OrdinalIgnoreCase)) {
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
