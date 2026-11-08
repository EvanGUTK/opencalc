[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Installer,
    [Parameter(Mandatory = $true)][string]$PortableArchive,
    [string]$EvidenceDirectory = (Join-Path $PSScriptRoot '..\target\install-smoke'),
    [ValidateRange(1, 100000)][int]$ExpectedChecks = 138,
    [switch]$SkipLaunch
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$setup = (Resolve-Path -LiteralPath $Installer).Path
$archive = (Resolve-Path -LiteralPath $PortableArchive).Path
$evidence = [IO.Path]::GetFullPath($EvidenceDirectory)
$testRoot = Join-Path $evidence ('run-' + [Guid]::NewGuid().ToString('N'))
if (-not $testRoot.StartsWith($evidence + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Smoke directory escaped its evidence root.' }
New-Item -ItemType Directory -Path $testRoot -Force | Out-Null
$installation = Join-Path $testRoot 'installed'
function Run-Hidden([string]$Path, [string]$Arguments, [int]$Seconds = 60, [int]$ExpectedExit = 0) {
    $process = [Diagnostics.Process]::new()
    $process.StartInfo.FileName = $Path
    $process.StartInfo.Arguments = $Arguments
    $process.StartInfo.UseShellExecute = $false
    $process.StartInfo.CreateNoWindow = $true
    $process.StartInfo.WindowStyle = [Diagnostics.ProcessWindowStyle]::Hidden
    try {
        [void]$process.Start()
        if (-not $process.WaitForExit($Seconds * 1000)) { $process.Kill(); $process.WaitForExit(); throw 'Package smoke process timed out.' }
        if ($process.ExitCode -ne $ExpectedExit) { throw "Package smoke process returned $($process.ExitCode); expected $ExpectedExit." }
    } finally { $process.Dispose() }
}
function Verify-Payload([string]$Directory) {
    $manifest = Get-Content -LiteralPath (Join-Path $Directory 'PACKAGE-MANIFEST.json') -Raw | ConvertFrom-Json
    foreach ($file in $manifest.files) {
        $path = [IO.Path]::GetFullPath((Join-Path $Directory $file.path))
        if (-not $path.StartsWith($Directory + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Unsafe package manifest path.' }
        if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $file.sha256) { throw "Installed payload hash mismatch: $($file.path)" }
    }
    return $manifest
}
$curriculum = [Collections.Generic.List[object]]::new()
function Verify-SelfTest([string]$Directory, [string]$Label, [string]$ExpectedVersion) {
    $executablePath = Join-Path $Directory 'opencalc.exe'
    $stdout = Join-Path $testRoot ($Label + '-self-test.json')
    $stderr = Join-Path $testRoot ($Label + '-self-test.stderr.txt')
    $process = [Diagnostics.Process]::new()
    $process.StartInfo.FileName = $executablePath
    $process.StartInfo.Arguments = '--self-test'
    $process.StartInfo.UseShellExecute = $false
    $process.StartInfo.CreateNoWindow = $true
    $process.StartInfo.RedirectStandardOutput = $true
    $process.StartInfo.RedirectStandardError = $true
    $outputStream = [IO.File]::Create($stdout)
    $errorStream = [IO.File]::Create($stderr)
    try {
        # Direct Process.Start retains the native handle even for very fast GUI exits.
        [void]$process.Start()
        $outputCopy = $process.StandardOutput.BaseStream.CopyToAsync($outputStream)
        $errorCopy = $process.StandardError.BaseStream.CopyToAsync($errorStream)
        if (-not $process.WaitForExit(60000)) { $process.Kill(); $process.WaitForExit(); throw "$Label self-test timed out." }
        [Threading.Tasks.Task]::WaitAll([Threading.Tasks.Task[]]@($outputCopy, $errorCopy))
        if ($process.ExitCode -ne 0) { throw "$Label self-test returned $($process.ExitCode); see its isolated stderr evidence." }
    } finally { $outputStream.Dispose(); $errorStream.Dispose(); $process.Dispose() }
    $report = Get-Content -LiteralPath $stdout -Raw | ConvertFrom-Json
    if ($report.passed -ne $ExpectedChecks -or $report.failed -ne 0 -or $report.rows.Count -ne $ExpectedChecks -or $report.application_version -ne $ExpectedVersion) { throw "$Label self-test JSON does not match the required check count/version." }
    $curriculum.Add([ordered]@{ distribution = $Label; passed = $report.passed; failed = $report.failed; application_version = $report.application_version; executable_sha256 = (Get-FileHash -LiteralPath $executablePath -Algorithm SHA256).Hash.ToLowerInvariant(); report = $stdout })
}
Run-Hidden $setup ('/S /TESTPROFILE /D=' + $installation)
$manifest = Verify-Payload $installation
if (Test-Path -LiteralPath (Join-Path $installation 'opencalc.portable')) { throw 'Normal install unexpectedly contains portable mode marker.' }
$marker = Get-Content -LiteralPath (Join-Path $installation 'opencalc.install') -Raw
if ($marker -ne 'OpenCalc-owned-install-v1:test') { throw 'Installer failed to isolate test mode.' }
# A running/locked payload must leave an uninstaller and ownership marker to retry.
$lockedPayload = [IO.File]::Open((Join-Path $installation 'opencalc.exe'), [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
try {
    Run-Hidden (Join-Path $installation 'Uninstall.exe') ('/S _?=' + $installation) 60 1
    if (-not (Test-Path -LiteralPath (Join-Path $installation 'opencalc.install')) -or -not (Test-Path -LiteralPath (Join-Path $installation 'Uninstall.exe'))) { throw 'Failed uninstall lost retry metadata.' }
} finally { $lockedPayload.Dispose() }
Run-Hidden $setup ('/S /TESTPROFILE /D=' + $installation)
$manifest = Verify-Payload $installation
$profile = Join-Path $testRoot 'profile'
$capture = Join-Path $testRoot 'installed.png'
if (-not $SkipLaunch) {
    Verify-SelfTest $installation 'installed' $manifest.version
    Run-Hidden (Join-Path $installation 'opencalc.exe') ('--data-dir "' + $profile + '" --screenshot "' + $capture + '" --size 800x900')
    if (-not (Test-Path -LiteralPath $capture -PathType Leaf)) { throw 'Installed application did not create its screenshot.' }
}
New-Item -ItemType Directory -Path (Join-Path $installation 'data') -Force | Out-Null
$notebook = Join-Path $installation 'keep.opencalc.json'
$personal = Join-Path $installation 'data\keep.txt'
[IO.File]::WriteAllText($notebook, '{"smoke_test_personal_file":true}')
[IO.File]::WriteAllText($personal, 'User-owned profile sentinel')
Run-Hidden (Join-Path $installation 'Uninstall.exe') '/S'
$uninstallDeadline = [DateTime]::UtcNow.AddSeconds(30)
while ((Test-Path -LiteralPath (Join-Path $installation 'opencalc.install')) -and [DateTime]::UtcNow -lt $uninstallDeadline) { Start-Sleep -Milliseconds 100 }
foreach ($file in $manifest.files) {
    if (Test-Path -LiteralPath (Join-Path $installation $file.path)) { throw "Uninstaller left an owned payload file: $($file.path)" }
}
if (-not (Test-Path -LiteralPath $notebook) -or -not (Test-Path -LiteralPath $personal)) { throw 'Uninstaller removed personal files.' }
if (Test-Path -LiteralPath (Join-Path $installation 'opencalc.install')) { throw 'Uninstaller left ownership marker.' }
$portableRoot = Join-Path $testRoot 'portable'
Expand-Archive -LiteralPath $archive -DestinationPath $portableRoot
$portable = Join-Path $portableRoot 'OpenCalc'
$portableManifest = Verify-Payload $portable
if (-not (Test-Path -LiteralPath (Join-Path $portable 'opencalc.portable'))) { throw 'Portable archive has no portable marker.' }
if (-not $SkipLaunch) {
    Verify-SelfTest $portable 'portable' $portableManifest.version
    $portableCapture = Join-Path $testRoot 'portable.png'
    Run-Hidden (Join-Path $portable 'opencalc.exe') ('--screenshot "' + $portableCapture + '" --size 800x900')
    if (-not (Test-Path -LiteralPath $portableCapture -PathType Leaf)) { throw 'Portable application did not create its screenshot.' }
}
$result = [ordered]@{ version = $manifest.version; installed_files_verified = $manifest.files.Count; portable_files_verified = $portableManifest.files.Count; silent_install = 'passed'; payload_hashes = 'passed'; locked_uninstall_retry = 'passed'; uninstall_owned_files = 'passed'; preserved_personal_files = 'passed'; launch = $(if ($SkipLaunch) { 'skipped' } else { 'installed and portable screenshot passed' }); curriculum = @($curriculum); evidence = $testRoot; profile = $profile; mode = 'isolated /TESTPROFILE; no registry or shortcuts' }
$json = $result | ConvertTo-Json -Depth 4
[IO.File]::WriteAllText((Join-Path $testRoot 'results.json'), $json)
$json
