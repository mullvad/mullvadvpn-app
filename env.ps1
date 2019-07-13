$SCRIPT_DIR = split-path -parent $MyInvocation.MyCommand.Definition

$env:OPENSSL_STATIC="1"
$env:OPENSSL_LIB_DIR="$SCRIPT_DIR\dist-assets\binaries\x86_64-pc-windows-msvc"
$env:OPENSSL_INCLUDE_DIR="$SCRIPT_DIR\dist-assets\binaries\x86_64-pc-windows-msvc\include"
