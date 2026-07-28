@echo off
setlocal EnableDelayedExpansion

:: Derive LLVM prefix from our own location
:: We live in <prefix>\bin\llvm-config.cmd
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
if /i "%~1"=="--libs"          echo LLVM-C.lib & goto :eof
if /i "%~1"=="--libnames"      echo LLVM-C.lib & goto :eof
if /i "%~1"=="--libfiles"      echo %PREFIX_FWD%/lib/LLVM-C.lib & goto :eof
if /i "%~1"=="--components"    echo all & goto :eof
if /i "%~1"=="--shared-mode"   echo shared & goto :eof
if /i "%~1"=="--system-libs"   echo( & goto :eof
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
