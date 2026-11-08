[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Executable,
    [Parameter(Mandatory = $true)][ValidatePattern('^\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?$')][string]$Version,
    [Parameter(Mandatory = $true)][string]$Makensis,
    [Parameter(Mandatory = $true)][string]$DependencyLicenseDirectory,
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '..\target\releases'),
    [ValidatePattern('^[0-9a-f]{7,40}$')][string]$SourceCommit = ''
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$binary = (Resolve-Path -LiteralPath $Executable).Path
$compiler = (Resolve-Path -LiteralPath $Makensis).Path
$licenses = (Resolve-Path -LiteralPath $DependencyLicenseDirectory).Path
$output = [IO.Path]::GetFullPath($OutputDirectory)
$cargoText = Get-Content -LiteralPath (Join-Path $repo 'Cargo.toml') -Raw
$packageSection = [regex]::Match($cargoText, '(?ms)^\[package\]\s*(.*?)(?=^\[|\z)').Groups[1].Value
$cargoVersion = [regex]::Match($packageSection, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
if ($cargoVersion -ne $Version) { throw "Package version must match Cargo.toml ($cargoVersion)." }
$numericParts = $Version.Split('-')[0].Split('.')
foreach ($part in $numericParts) { if ([int]$part -gt 65535) { throw 'Windows version components must fit 16 bits.' } }
$fileVersion = ($numericParts -join '.') + '.0'
if (-not (Test-Path -LiteralPath (Join-Path $licenses 'index.json') -PathType Leaf)) { throw 'Dependency notice index is required.' }
if ([IO.Path]::GetExtension($binary) -ne '.exe') { throw 'Expected a Windows executable.' }
$stream = [IO.File]::OpenRead($binary)
try {
    $reader = [IO.BinaryReader]::new($stream)
    if ($reader.ReadUInt16() -ne 0x5a4d) { throw 'Executable has no DOS/PE signature.' }
    $stream.Position = 0x3c
    $peOffset = $reader.ReadUInt32()
    $stream.Position = $peOffset
    if ($reader.ReadUInt32() -ne 0x00004550 -or $reader.ReadUInt16() -ne 0x8664) { throw 'Only Windows x86_64 PE executables are supported.' }
} finally { $stream.Dispose() }
$noticeIndex = Get-Content -LiteralPath (Join-Path $licenses 'index.json') -Raw | ConvertFrom-Json
$noticePaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
[void]$noticePaths.Add('index.json')
[void]$noticePaths.Add('README.txt')
$lockHash = (Get-FileHash -LiteralPath (Join-Path $repo 'Cargo.lock') -Algorithm SHA256).Hash.ToLowerInvariant()
if ($noticeIndex.cargo_lock_sha256 -ne $lockHash) { throw 'Dependency bundle does not match Cargo.lock.' }
foreach ($package in $noticeIndex.packages) {
    foreach ($notice in $package.files) {
        $noticePath = [IO.Path]::GetFullPath((Join-Path $licenses $notice.path))
        if (-not $noticePath.StartsWith($licenses + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Unsafe dependency notice path.' }
        if ((Get-FileHash -LiteralPath $noticePath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $notice.sha256) { throw 'Dependency notice hash mismatch.' }
        if ((Get-Item -LiteralPath $noticePath).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Dependency notices cannot be symbolic links.' }
        [void]$noticePaths.Add($notice.path)
    }
}
New-Item -ItemType Directory -Path $output -Force | Out-Null
$stage = Join-Path $output ('stage-' + [Guid]::NewGuid().ToString('N'))
$payload = Join-Path $stage 'OpenCalc'
New-Item -ItemType Directory -Path $payload -Force | Out-Null
Copy-Item -LiteralPath $binary -Destination (Join-Path $payload 'opencalc.exe')
foreach ($name in @('LICENSE', 'THIRD_PARTY_NOTICES.md', 'README.md')) { Copy-Item -LiteralPath (Join-Path $repo $name) -Destination $payload }
$fontOutput = Join-Path $payload 'assets\licenses'
New-Item -ItemType Directory -Path $fontOutput -Force | Out-Null
foreach ($name in @('Hack-Regular.txt', 'NotoEmoji-OFL.txt', 'Ubuntu-UFL.txt', 'emoji-icon-font-MIT.txt')) {
    Copy-Item -LiteralPath (Join-Path $repo ('assets\licenses\' + $name)) -Destination $fontOutput
}
foreach ($relative in $noticePaths) {
    $destination = Join-Path $payload ('dependency-licenses\' + $relative)
    New-Item -ItemType Directory -Path ([IO.Path]::GetDirectoryName($destination)) -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $licenses $relative) -Destination $destination
}
$compilerNotice = Join-Path ([IO.Path]::GetDirectoryName($compiler)) 'COPYING'
if (-not (Test-Path -LiteralPath $compilerNotice -PathType Leaf)) { throw 'Use the root makensis.exe from the complete portable NSIS distribution; COPYING is required.' }
Copy-Item -LiteralPath $compilerNotice -Destination (Join-Path $payload 'assets\licenses\NSIS-COPYING.txt')
$manifestFiles = @(Get-ChildItem -LiteralPath $payload -File -Recurse | Sort-Object FullName | ForEach-Object {
    [ordered]@{ path = $_.FullName.Substring($payload.Length + 1).Replace('\', '/'); sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant(); bytes = $_.Length }
})
$manifest = [ordered]@{ format = 1; product = 'OpenCalc'; version = $Version; platform = 'windows-x86_64'; source_commit = $SourceCommit; cargo_lock_sha256 = $lockHash; files = $manifestFiles }
$utf8 = [Text.UTF8Encoding]::new($false)
[IO.File]::WriteAllText((Join-Path $payload 'PACKAGE-MANIFEST.json'), (($manifest | ConvertTo-Json -Depth 8) + "`n"), $utf8)
function Quote-Nsis([string]$Text) { return $Text.Replace('$', '$$').Replace('"', '$\"') }
$installLines = [Collections.Generic.List[string]]::new()
$deleteLines = [Collections.Generic.List[string]]::new()
$directories = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
$installLines.Add('!macro InstallPayload')
$deleteLines.Add('!macro UninstallPayload')
foreach ($file in (Get-ChildItem -LiteralPath $payload -File -Recurse | Sort-Object FullName)) {
    $relative = $file.FullName.Substring($payload.Length + 1)
    $parent = [IO.Path]::GetDirectoryName($relative)
    $suffix = if ($parent) { '\' + (Quote-Nsis $parent) } else { '' }
    $installLines.Add('SetOutPath "$INSTDIR' + $suffix + '"')
    $installLines.Add('File "' + (Quote-Nsis $file.FullName) + '"')
    $deleteLines.Add('Delete "$INSTDIR\' + (Quote-Nsis $relative) + '"')
    $deleteLines.Add('IfFileExists "$INSTDIR\' + (Quote-Nsis $relative) + '" 0 +2')
    $deleteLines.Add('StrCpy $UninstallFailed "1"')
    while ($parent) { [void]$directories.Add($parent); $parent = [IO.Path]::GetDirectoryName($parent) }
}
$installLines.Add('!macroend')
foreach ($directory in ($directories | Sort-Object { $_.Length } -Descending)) { $deleteLines.Add('RMDir "$INSTDIR\' + (Quote-Nsis $directory) + '"') }
$deleteLines.Add('!macroend')
$include = Join-Path $stage 'payload.nsh'
[IO.File]::WriteAllLines($include, @($installLines) + @($deleteLines), $utf8)
$setup = Join-Path $output "OpenCalc-$Version-windows-x86_64-setup.exe"
$archive = Join-Path $output "OpenCalc-$Version-windows-x86_64-portable.zip"
if ((Test-Path -LiteralPath $setup) -or (Test-Path -LiteralPath $archive)) { throw 'Release artifacts already exist; choose a new output directory.' }
& $compiler '/V2' "/DAPP_VERSION=$Version" "/DFILE_VERSION=$fileVersion" "/DOUTPUT_FILE=$setup" "/DAPP_LICENSE=$(Join-Path $repo 'LICENSE')" "/DPAYLOAD_INCLUDE=$include" (Join-Path $repo 'packaging\opencalc.nsi')
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $setup)) { throw "NSIS compilation failed ($LASTEXITCODE)." }
[IO.File]::WriteAllText((Join-Path $payload 'opencalc.portable'), "OpenCalc portable profile: data beside this executable.`n", $utf8)
$portableFiles = @(Get-ChildItem -LiteralPath $payload -File -Recurse | Where-Object Name -ne 'PACKAGE-MANIFEST.json' | Sort-Object FullName | ForEach-Object {
    [ordered]@{ path = $_.FullName.Substring($payload.Length + 1).Replace('\', '/'); sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant(); bytes = $_.Length }
})
$manifest.files = $portableFiles
[IO.File]::WriteAllText((Join-Path $payload 'PACKAGE-MANIFEST.json'), (($manifest | ConvertTo-Json -Depth 8) + "`n"), $utf8)
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zipStream = [IO.File]::Create($archive)
try {
    $zip = [IO.Compression.ZipArchive]::new($zipStream, [IO.Compression.ZipArchiveMode]::Create, $false)
    try {
        foreach ($file in (Get-ChildItem -LiteralPath $payload -File -Recurse | Sort-Object FullName)) {
            $entry = $zip.CreateEntry(('OpenCalc/' + $file.FullName.Substring($payload.Length + 1).Replace('\', '/')), [IO.Compression.CompressionLevel]::Optimal)
            $entry.LastWriteTime = [DateTimeOffset]::new(2026, 1, 1, 0, 0, 0, [TimeSpan]::Zero)
            $destination = $entry.Open()
            $input = [IO.File]::OpenRead($file.FullName)
            try { $input.CopyTo($destination) } finally { $input.Dispose(); $destination.Dispose() }
        }
    } finally { $zip.Dispose() }
} finally { $zipStream.Dispose() }
$sums = @($setup, $archive) | ForEach-Object { (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash.ToLowerInvariant() + '  ' + [IO.Path]::GetFileName($_) }
[IO.File]::WriteAllLines((Join-Path $output 'SHA256SUMS.txt'), $sums, $utf8)
[ordered]@{ version = $Version; installer = $setup; portable = $archive; sha256sums = (Join-Path $output 'SHA256SUMS.txt'); installer_payload_files = $manifestFiles.Count + 1; portable_payload_files = $portableFiles.Count + 1; stage = $stage } | ConvertTo-Json
