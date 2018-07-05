$SCRIPT_DIR = split-path -parent $MyInvocation.MyCommand.Definition

$env:OPENSSL_STATIC="1"
$env:OPENSSL_LIB_DIR="$SCRIPT_DIR\dist-assets\binaries\windows"
$env:OPENSSL_INCLUDE_DIR="$SCRIPT_DIR\dist-assets\binaries\windows\include"
