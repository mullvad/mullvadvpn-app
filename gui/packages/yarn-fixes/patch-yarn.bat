@echo off
setlocal

rem Yarn 1.9.4 has a path lookup bug on Windows, when it looks for the binaries referenced in 
rem scripts under '\gui\node_modules\node_modules' instead of '\gui\node_modules'. 
rem This patch adds a junction between those two to keep that house of cards from falling apart.

echo Applying a workspace patch for yarn on Windows...

call :NORMALIZEPATH "%~dp0\..\.."
set ROOT_WORKSPACE_DIR=%RETVAL%

echo Root workspace path: %ROOT_WORKSPACE_DIR%

set NODE_MODULES_DIR="%ROOT_WORKSPACE_DIR%\node_modules"
set DOUBLE_NODE_MODULES_DIR="%ROOT_WORKSPACE_DIR%\node_modules\node_modules"

if exist %DOUBLE_NODE_MODULES_DIR% (
  rmdir %DOUBLE_NODE_MODULES_DIR%
)

mklink /j %DOUBLE_NODE_MODULES_DIR% %NODE_MODULES_DIR%

rem Normalizes relative path to absolute
:NORMALIZEPATH
  set RETVAL=%~dpfn1
  exit /B
