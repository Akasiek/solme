param(
    [ValidateSet("nsis", "msi", "both")]
    [string]$Bundle = "both"
)

$ErrorActionPreference = "Stop"

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$mpvDirectory = Join-Path $repositoryRoot "src-tauri\mpv"
$archive = Join-Path $env:TEMP "solme-mpv-dev.7z"
$defaultMpvUrl = "https://downloads.sourceforge.net/project/mpv-player-windows/libmpv/mpv-dev-x86_64-20260607-git-71ebd08.7z"
$mpvUrl = if ($env:MPV_DEV_URL) { $env:MPV_DEV_URL } else { $defaultMpvUrl }
$mpvDll = Join-Path $mpvDirectory "libmpv-2.dll"
$mpvDefinition = Join-Path $mpvDirectory "mpv.def"
$mpvImportLibrary = Join-Path $mpvDirectory "mpv.lib"

foreach ($command in @("pnpm", "cargo", "7z", "curl.exe")) {
    if (!(Get-Command $command -ErrorAction SilentlyContinue)) {
        throw "Required command '$command' was not found in PATH"
    }
}

if (!(Test-Path $mpvDll)) {
    New-Item -ItemType Directory -Force -Path $mpvDirectory | Out-Null
    curl.exe --location --fail --retry 3 --output $archive $mpvUrl
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to download libmpv from $mpvUrl"
    }
    if ((Get-Item $archive).Length -lt 10000000) {
        throw "Downloaded libmpv archive is unexpectedly small"
    }
    7z x $archive "-o$mpvDirectory" -y
    if ($LASTEXITCODE -ne 0 -or !(Test-Path $mpvDll)) {
        throw "Failed to extract libmpv-2.dll"
    }
}

$dumpbin = Get-Command dumpbin.exe -ErrorAction SilentlyContinue
$lib = Get-Command lib.exe -ErrorAction SilentlyContinue
if (!$dumpbin -or !$lib) {
    throw "dumpbin.exe and lib.exe were not found. Run this target from Developer PowerShell for Visual Studio."
}

$exports = & $dumpbin.Source /nologo /exports $mpvDll
$names = $exports | ForEach-Object {
    if ($_ -match '^\s+\d+\s+[0-9A-Fa-f]+\s+[0-9A-Fa-f]+\s+(\S+)$') {
        $matches[1]
    }
}
if (!$names) {
    throw "No exported symbols were found in libmpv-2.dll"
}

@("LIBRARY libmpv-2.dll", "EXPORTS") + $names |
    Set-Content $mpvDefinition -Encoding ASCII
& $lib.Source /nologo "/def:$mpvDefinition" /machine:x64 "/out:$mpvImportLibrary"
if ($LASTEXITCODE -ne 0 -or !(Test-Path $mpvImportLibrary)) {
    throw "Failed to generate mpv.lib"
}

$env:LIB = "$mpvDirectory;$env:LIB"
$bundleArguments = if ($Bundle -eq "both") { "nsis,msi" } else { $Bundle }

Push-Location $repositoryRoot
try {
    pnpm install --frozen-lockfile
    if ($LASTEXITCODE -ne 0) {
        throw "pnpm install failed"
    }
    pnpm tauri build --bundles $bundleArguments
    if ($LASTEXITCODE -ne 0) {
        throw "Tauri build failed"
    }
}
finally {
    Pop-Location
}

[Console]::WriteLine("Windows bundles are available under src-tauri\target\release\bundle")
