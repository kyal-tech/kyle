# install.ps1 — Windows PowerShell installer for Kyle
#
# Usage:
#   iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.ps1" | iex
#
# Environment variables:
#   $env:KY_VERSION = "v0.6.1"     Version to install (default: latest)
#   $env:KY_PREFIX = "C:\ky"       Install directory (default: ~\.ky)

param(
    [string]$Version = "v0.8.6",
    [string]$Prefix = ""
)

$Repo = "IT-KYNERA/KYLE"

# ─── Detect architecture ────────────────────────────────────

$Arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "arm64" }
$Platform = "windows-$Arch"
$Bundle = "ky-windows-$Arch.zip"
$BundleUrl = "https://github.com/$Repo/releases/download/$Version/$Bundle"

Write-Host "Detected: $Platform"

# ─── Determine install prefix ───────────────────────────────

if (-not $Prefix) {
    $env:KY_PREFIX = if ($env:KY_PREFIX) { $env:KY_PREFIX } else { "$env:USERPROFILE\.ky" }
} else {
    $env:KY_PREFIX = $Prefix
}

$BinDir = "$env:KY_PREFIX\bin"
$LibDir = "$env:KY_PREFIX\lib"

# ─── Download ───────────────────────────────────────────────

$TmpDir = "$env:TEMP\ky-install"
if (Test-Path $TmpDir) { Remove-Item -Recurse -Force $TmpDir }
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

$OutFile = "$TmpDir\$Bundle"
Write-Host "Downloading Kyle $Version for $Platform..."
Write-Host "  $BundleUrl"

try {
    Invoke-WebRequest -Uri $BundleUrl -OutFile $OutFile -UseBasicParsing
} catch {
    Write-Host "Error: failed to download $Bundle"
    Write-Host "Check that $Version exists at:"
    Write-Host "  https://github.com/$Repo/releases"
    exit 1
}

# ─── Extract ─────────────────────────────────────────────────

Write-Host "Extracting..."
try {
    Expand-Archive -Path $OutFile -DestinationPath $TmpDir -Force
} catch {
    Write-Host "Error: failed to extract $Bundle"
    exit 1
}

if (-not (Test-Path "$TmpDir\ky.exe")) {
    Write-Host "Error: ky.exe not found in archive"
    Get-ChildItem $TmpDir
    exit 1
}

# ─── Install ─────────────────────────────────────────────────

Write-Host "Installing to $env:KY_PREFIX..."

# Create directories
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
New-Item -ItemType Directory -Force -Path $LibDir | Out-Null

# Copy files
Copy-Item "$TmpDir\ky.exe" "$BinDir\ky.exe" -Force
if (Test-Path "$TmpDir\libkyc_runtime.a") {
    Copy-Item "$TmpDir\libkyc_runtime.a" "$LibDir\libkyc_runtime.a" -Force
} elseif (Test-Path "$TmpDir\kyc_runtime.lib") {
    Copy-Item "$TmpDir\kyc_runtime.lib" "$LibDir\kyc_runtime.lib" -Force
}

# Clean up ky install temp
Remove-Item -Recurse -Force $TmpDir

# ─── Add to PATH ────────────────────────────────────────────

$regPath = "HKCU:\Environment"
$currentPath = (Get-ItemProperty -Path $regPath -Name PATH -ErrorAction SilentlyContinue).PATH
$addedDirs = @()
if ($currentPath -notlike "*$BinDir*") { $addedDirs += $BinDir }
if ($addedDirs.Count -gt 0) {
    $newPath = ($addedDirs -join ';') + ";" + $currentPath
    Set-ItemProperty -Path $regPath -Name PATH -Value $newPath
    Write-Host "  Added to PATH:"
    foreach ($d in $addedDirs) { Write-Host "    $d" }
    try {
        $sig = '[DllImport("user32.dll", SetLastError=true, CharSet=CharSet.Auto)] public static extern IntPtr SendMessageTimeout(IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam, uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);'
        Add-Type -MemberDefinition $sig -Name NativeMethods -Namespace Win32 -ErrorAction Stop | Out-Null
        $result = [UIntPtr]::Zero
        [Win32.NativeMethods]::SendMessageTimeout(0xffff, 0x001a, [UIntPtr]::Zero, "Environment", 2, 5000, [ref]$result) | Out-Null
    } catch { }
} else {
    Write-Host "  PATH already configured"
}
$env:PATH = "$BinDir;$env:PATH"

# ─── Verify ─────────────────────────────────────────────────

Write-Host ""
try {
    $version = & "$BinDir\ky.exe" --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[OK] Kyle $Version installed successfully!"
        Write-Host ""
        Write-Host "  Binary:  $BinDir\ky.exe"
        Write-Host ""
        Write-Host "  Use now:  ky --version"
        Write-Host "  Try:      ky run examples\hello.ky"
    } else {
        Write-Host "[WARN] Installation completed but verification failed."
    }
} catch {
    Write-Host "[WARN] Installation completed but 'ky.exe' not found in PATH."
    Write-Host "   Restart your terminal or add manually:"
    Write-Host "    [Environment]::SetEnvironmentVariable('PATH', ""$BinDir;`$env:PATH"", 'User')"
}
