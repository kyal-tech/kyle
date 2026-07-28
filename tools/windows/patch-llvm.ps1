param(
    [string]$LLVM_PREFIX = "",
    [string]$LLVM_URL = "https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/LLVM-18.1.8-win64.exe"
)

function Find-7z {
    $paths = @(
        "C:\Program Files\7-Zip\7z.exe",
        "C:\Program Files (x86)\7-Zip\7z.exe",
        "$env:ProgramFiles\7-Zip\7z.exe",
        "${env:ProgramFiles(x86)}\7-Zip\7z.exe"
    )
    foreach ($p in $paths) { if (Test-Path $p) { return $p } }
    $cmd = Get-Command "7z" -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    return $null
}

# Find LLVM prefix
if (-not $LLVM_PREFIX) {
    $LLVM_PREFIX = $env:LLVM_SYS_181_PREFIX
}
if (-not $LLVM_PREFIX) {
    # Default: assume script lives in repo tools/windows/ and LLVM is at repo root/llvm-18
    $LLVM_PREFIX = Join-Path (Split-Path (Split-Path $PSScriptRoot -Parent) -Parent) "llvm-18"
}
if (-not (Test-Path $LLVM_PREFIX)) {
    $7zPath = Find-7z
    if (-not $7zPath) {
        Write-Host "LLVM prefix not found at: $LLVM_PREFIX"
        Write-Host "7-Zip is required to extract LLVM. Install 7-Zip or download manually:"
        Write-Host "  https://github.com/llvm/llvm-project/releases/tag/llvmorg-18.1.8"
        exit 1
    }
    Write-Host "LLVM prefix not found at: $LLVM_PREFIX"
    Write-Host "Downloading LLVM 18.1.8 for Windows..."
    $tmpExe = "$env:TEMP\LLVM-18.1.8-win64.exe"
    Invoke-WebRequest -Uri $LLVM_URL -OutFile $tmpExe -UseBasicParsing
    New-Item -ItemType Directory -Force -Path $LLVM_PREFIX | Out-Null
    & $7zPath x $tmpExe -o"$LLVM_PREFIX" -y | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Warning: 7z extraction returned exit code $LASTEXITCODE"
        Write-Host "LLVM may be partially installed."
    }
    Remove-Item $tmpExe -Force
    Write-Host "LLVM 18.1.8 installed to $LLVM_PREFIX"
} else {
    Write-Host "LLVM prefix found at: $LLVM_PREFIX"
}

# ─── Create llvm-config.h ────────────────────────────────────────
$configDir = Join-Path $LLVM_PREFIX "include\llvm\Config"
New-Item -ItemType Directory -Force -Path $configDir | Out-Null

$configH = Join-Path $configDir "llvm-config.h"
if (Test-Path $configH) {
    Write-Host "llvm-config.h already exists"
} else {
Write-Host "Creating $configH..."
Set-Content -Path $configH -Encoding ASCII -Value @'
#ifndef LLVM_CONFIG_H
#define LLVM_CONFIG_H

/* LLVM version */
#define LLVM_VERSION_MAJOR 18
#define LLVM_VERSION_MINOR 1
#define LLVM_VERSION_PATCH 8
#define LLVM_VERSION_STRING "18.1.8"

/* Package info */
#define LLVM_PACKAGE_NAME "LLVM"
#define LLVM_PACKAGE_VERSION "18.1.8"
#define LLVM_PACKAGE_BUGREPORT "https://bugs.llvm.org/"

/* Target triple */
#define LLVM_DEFAULT_TARGET_TRIPLE "x86_64-pc-windows-msvc"
#define LLVM_HOST_TRIPLE "x86_64-pc-windows-msvc"

/* Features */
#define LLVM_ENABLE_ASSERTIONS 0
#define LLVM_ENABLE_EXCEPTIONS 1
#define LLVM_ENABLE_RTTI 0
#define LLVM_ENABLE_THREADS 1
#define LLVM_ENABLE_ZLIB 0
#define LLVM_ENABLE_ZSTD 0
#define LLVM_ENABLE_LIBXML2 0
#define LLVM_ENABLE_TERMINFO 0
#define LLVM_ENABLE_FFI 0
#define LLVM_ENABLE_PLUGINS 1
#define LLVM_ENABLE_EH 1
#define LLVM_ENABLE_CRASH_OVERRIDES 1
#define LLVM_ENABLE_BACKTRACES 1
#define LLVM_ENABLE_DIA_SDK 0

/* Includes */
#define LLVM_INCLUDE_TESTS 0
#define LLVM_INCLUDE_DOCS 0
#define LLVM_INCLUDE_EXAMPLES 0
#define LLVM_INCLUDE_BENCHMARKS 0
#define LLVM_INCLUDE_GO_TESTS 0

/* Native target — X86 for Windows */
#define LLVM_NATIVE_ARCH X86
#define LLVM_NATIVE_TARGET LLVMInitializeX86Target
#define LLVM_NATIVE_TARGETINFO LLVMInitializeX86TargetInfo
#define LLVM_NATIVE_TARGETMC LLVMInitializeX86TargetMC
#define LLVM_NATIVE_ASM LLVMInitializeX86AsmPrinter
#define LLVM_NATIVE_ASMPRINTER LLVMInitializeX86AsmPrinter
#define LLVM_NATIVE_ASMPARSER LLVMInitializeX86AsmParser
#define LLVM_NATIVE_DISASSEMBLER LLVMInitializeX86Disassembler

/* Build mode */
#define LLVM_BUILD_MODE "Release"

/* Endian */
#define LLVM_LITTLE_ENDIAN 1
#define LLVM_BIG_ENDIAN 0

#endif
'@
}

# ─── Create .def files ──────────────────────────────────────────
$defs = @{
    "Targets.def"        = "X86`r`nAArch64`r`nARM"
    "AsmPrinters.def"    = "X86`r`nAArch64`r`nARM"
    "AsmParsers.def"     = "X86`r`nAArch64`r`nARM"
    "Disassemblers.def"  = "X86`r`nAArch64`r`nARM"
}
foreach ($name in $defs.Keys) {
    $path = Join-Path $configDir $name
    if (Test-Path $path) { continue }
    Write-Host "Creating $path..."
    Set-Content -Path $path -Value $defs[$name] -Encoding ASCII
}

# ─── Create llvm-config.cmd ──────────────────────────────────────
$binDir = Join-Path $LLVM_PREFIX "bin"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null

$llvmConfigCmd = Join-Path $binDir "llvm-config.cmd"
if (Test-Path $llvmConfigCmd) {
    Write-Host "llvm-config.cmd already exists"
} else {
Write-Host "Creating $llvmConfigCmd..."
Set-Content -Path $llvmConfigCmd -Encoding ASCII -Value @'
@echo off
setlocal EnableDelayedExpansion
set "SCRIPT_DIR=%~dp0"
for %%I in ("%SCRIPT_DIR%..") do set "PREFIX=%%~fI"
set "PREFIX_FWD=%PREFIX:\=/%"
if /i "%~1"==""      echo 18.1.8 & goto :eof
if /i "%~1"=="--version"       echo 18.1.8 & goto :eof
if /i "%~1"=="--prefix"        echo %PREFIX% & goto :eof
if /i "%~1"=="--includedir"    echo %PREFIX%\include & goto :eof
if /i "%~1"=="--libdir"        echo %PREFIX%\lib & goto :eof
if /i "%~1"=="--bindir"        echo %PREFIX%\bin & goto :eof
if /i "%~1"=="--cflags"        echo -I%PREFIX_FWD%/include & goto :eof
if /i "%~1"=="--cxxflags"      echo -I%PREFIX_FWD%/include & goto :eof
if /i "%~1"=="--ldflags"       echo -LIBPATH:%PREFIX_FWD%/lib & goto :eof
if /i "%~1"=="--libs"          echo -lLLVM-C & goto :eof
if /i "%~1"=="--libnames"      echo LLVM-C.lib & goto :eof
if /i "%~1"=="--libfiles"      echo %PREFIX_FWD%/lib/LLVM-C.lib & goto :eof
if /i "%~1"=="--components"    echo all & goto :eof
if /i "%~1"=="--shared-mode"   echo shared & goto :eof
if /i "%~1"=="--system-libs"   echo psapi.lib shell32.lib ole32.lib uuid.lib advapi32.lib ws2_32.lib legacy_stdio_definitions.lib dbghelp.lib kernel32.lib ntdll.lib userenv.lib bcrypt.lib & goto :eof
if /i "%~1"=="--targets-built" echo AArch64 ARM X86 & goto :eof
if /i "%~1"=="--host-target"   echo x86_64-pc-windows-msvc & goto :eof
if /i "%~1"=="--has-rtti"      echo NO & goto :eof
if /i "%~1"=="--assertion-mode" echo OFF & goto :eof
if /i "%~1"=="--build-mode"    echo Release & goto :eof
if /i "%~1"=="--link-shared"   echo -DLLVM_LINK_SHARED=1 & goto :eof
if /i "%~1"=="--link-static"   echo( & goto :eof
if /i "%~1"=="--obj-root"      echo %PREFIX_FWD% & goto :eof
if /i "%~1"=="--src-root"      echo %PREFIX_FWD% & goto :eof
exit /b 1
'@
}

# ─── Compile llvm-config.exe (real .exe, needed by llvm-sys) ───
# llvm-sys on Windows looks for llvm-config.exe in LLVM_SYS_181_PREFIX/bin/.
# We compile a small C# program via csc.exe (always available on Windows with .NET).

$cscPaths = @(
    "$env:SystemRoot\Microsoft.NET\Framework64\v4.0.30319\csc.exe",
    "$env:SystemRoot\Microsoft.NET\Framework\v4.0.30319\csc.exe",
    (Get-Command "csc.exe" -ErrorAction SilentlyContinue | ForEach-Object Source)
)
$cscPath = $cscPaths | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1

$csSrcDir = Join-Path $LLVM_PREFIX "src"
New-Item -ItemType Directory -Force -Path $csSrcDir | Out-Null
$csSrc = Join-Path $csSrcDir "llvm-config.cs"

Set-Content -Path $csSrc -Encoding ASCII -Value @'
using System;
using System.IO;

class LlvmConfig {
    static int Main(string[] args) {
        Console.Out.NewLine = "\n";
        if (args.Length < 1) { Console.WriteLine("18.1.8"); return 0; }
        string me = System.Reflection.Assembly.GetEntryAssembly().Location;
        string bindir = Path.GetDirectoryName(me);
        string prefix = Path.GetDirectoryName(bindir);
        string fwd = prefix.Replace('\\', '/');
        string a = args[0];
        if (a == "--version")        { Console.WriteLine("18.1.8"); return 0; }
        if (a == "--prefix")         { Console.WriteLine(prefix); return 0; }
        if (a == "--includedir")     { Console.WriteLine(prefix + "\\include"); return 0; }
        if (a == "--libdir")         { Console.WriteLine(prefix + "\\lib"); return 0; }
        if (a == "--bindir")         { Console.WriteLine(prefix + "\\bin"); return 0; }
        if (a == "--cflags")         { Console.WriteLine("-I" + fwd + "/include"); return 0; }
        if (a == "--cxxflags")       { Console.WriteLine("-I" + fwd + "/include"); return 0; }
        if (a == "--ldflags")        { Console.WriteLine("-LIBPATH:" + fwd + "/lib"); return 0; }
        if (a == "--libs")           { Console.WriteLine("-lLLVM-C"); return 0; }
        if (a == "--libnames")       { Console.WriteLine("LLVM-C.lib"); return 0; }
        if (a == "--libfiles")       { Console.WriteLine(fwd + "/lib/LLVM-C.lib"); return 0; }
        if (a == "--components")     { Console.WriteLine("all"); return 0; }
        if (a == "--shared-mode")    { Console.WriteLine("shared"); return 0; }
        if (a == "--system-libs")    { Console.WriteLine("psapi.lib shell32.lib ole32.lib uuid.lib advapi32.lib ws2_32.lib legacy_stdio_definitions.lib dbghelp.lib kernel32.lib ntdll.lib userenv.lib bcrypt.lib"); return 0; }
        if (a == "--targets-built")  { Console.WriteLine("AArch64 ARM X86"); return 0; }
        if (a == "--host-target")    { Console.WriteLine("x86_64-pc-windows-msvc"); return 0; }
        if (a == "--has-rtti")       { Console.WriteLine("NO"); return 0; }
        if (a == "--assertion-mode") { Console.WriteLine("OFF"); return 0; }
        if (a == "--build-mode")     { Console.WriteLine("Release"); return 0; }
        if (a == "--link-shared")    { Console.WriteLine("-DLLVM_LINK_SHARED=1"); return 0; }
        if (a == "--link-static")    { Console.WriteLine(); return 0; }
        if (a == "--obj-root")       { Console.WriteLine(fwd); return 0; }
        if (a == "--src-root")       { Console.WriteLine(fwd); return 0; }
        return 1;
    }
}
'@

$llvmExe = Join-Path $binDir "llvm-config.exe"
if ($cscPath) {
    Write-Host "Compiling llvm-config.exe with csc.exe (overwriting tarball version)..."
    & $cscPath /nologo /target:exe /out:$llvmExe $csSrc 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0 -and (Test-Path $llvmExe)) {
        Write-Host "  -> $llvmExe ($([math]::Round((Get-Item $llvmExe).Length / 1KB)) KB)"
    } else {
        Write-Host "  -> csc.exe compile failed (exit $LASTEXITCODE), using llvm-config.cmd fallback"
        Remove-Item $llvmExe -ErrorAction SilentlyContinue
    }
} else {
    Write-Host "  -> csc.exe not found, using llvm-config.cmd"
    Remove-Item $llvmExe -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "LLVM setup complete."
Write-Host "  Prefix: $LLVM_PREFIX"
