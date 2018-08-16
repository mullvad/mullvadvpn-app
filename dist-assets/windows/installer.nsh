!include stdutils.nsh
!include winver.nsh

!addplugindir "${BUILD_RESOURCES_DIR}\..\windows\nsis-plugins\bin\Win32-Release"

#
# NOTES
#
# Do not include certain default header files - results in random errors
# Do not define and use functions - broken
# Do not use DetailPrint - any message sent to DetailPrint is lost
# Do not compare variables using the <> operator - broken
#

# TAP device hardware ID
!define TAP_HARDWARE_ID "tap0901"

# "sc" exit code
!define SERVICE_STARTED 0
!define SERVICE_START_PENDING 2

# Return codes from driverlogic::EstablishBaseline
!define EB_GENERAL_ERROR 0
!define EB_NO_INTERFACES_PRESENT 1
!define EB_SOME_INTERFACES_PRESENT 2
!define EB_MULLVAD_INTERFACE_PRESENT 3

# Return codes from driverlogic::IdentifyNewInterface
!define INI_GENERAL_ERROR 0
!define INI_SUCCESS 1

#
# BreakInstallation
#
# Aborting the customization step does not undo previous steps taken
# by the installer (copy files, create shortcut, etc)
#
# Therefore we have to break the installed application to
# prevent users from running a half-installed product
#
!macro BreakInstallation

	Delete "$INSTDIR\mullvad vpn.exe"

!macroend

!define BreakInstallation '!insertmacro "BreakInstallation"'

#
# ExtractDriver
#
# Extract the correct driver for the current platform
# placing it into $TEMP\driver
#
!macro ExtractDriver

	SetOutPath "$TEMP\driver"
	File "${BUILD_RESOURCES_DIR}\binaries\windows\driver\*"

	${If} ${IsWin7}
		File "${BUILD_RESOURCES_DIR}\binaries\windows\driver\ndis5\*"
	${Else}
		File "${BUILD_RESOURCES_DIR}\binaries\windows\driver\ndis6\*"
	${EndIf}
	
!macroend

!define ExtractDriver '!insertmacro "ExtractDriver"'

#
# InstallDriver
#
# Install tunnel driver or update it if already present on the system
#
# Returns: 0 in $R0 on success, otherwise an error message in $R0
#
!macro InstallDriver

	Var /GLOBAL InstallDriver_BaselineStatus

	Push $0
	Push $1

	nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" hwids ${TAP_HARDWARE_ID}'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to list virtual adapters: error $0"
		Goto InstallDriver_return
	${EndIf}

	driverlogic::EstablishBaseline $1

	Pop $0
	Pop $1

	${If} $0 == ${EB_GENERAL_ERROR}
		StrCpy $R0 "Failed to parse virtual adapter data: $1"
		Goto InstallDriver_return
	${EndIf}

	Push $0
	Pop $InstallDriver_BaselineStatus
	
	IntCmp $0 ${EB_NO_INTERFACES_PRESENT} InstallDriver_install_driver

	#
	# Driver is already installed and there are one or several virtual adapters present.
	# Update driver.
	#
	nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" update "$TEMP\driver\OemVista.inf" ${TAP_HARDWARE_ID}'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to update TAP driver: error $0"
		Goto InstallDriver_return
	${EndIf}
	
	IntCmp $InstallDriver_BaselineStatus ${EB_MULLVAD_INTERFACE_PRESENT} InstallDriver_return_success

	InstallDriver_install_driver:

	#
	# Install driver and create a virtual adapter.
	# If the driver is already installed, this just creates another virtual adapter.
	#
	nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" install "$TEMP\driver\OemVista.inf" ${TAP_HARDWARE_ID}'
	
	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to install TAP driver: error $0"
		Goto InstallDriver_return
	${EndIf}

	nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" hwids ${TAP_HARDWARE_ID}'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to list virtual adapters: error $0"
		Goto InstallDriver_return
	${EndIf}

	driverlogic::IdentifyNewInterface $1
	
	Pop $0
	Pop $1

	${If} $0 != ${INI_SUCCESS}
		StrCpy $R0 "Failed to identify virtual adapter: $1"
		Goto InstallDriver_return
	${EndIf}

	#
	# Rename the newly added virtual adapter to "Mullvad".
	#
	nsExec::ExecToStack '"netsh.exe" interface set interface name = "$1" newname = "Mullvad"'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to rename virtual adapter: error $0"
		Goto InstallDriver_return
	${EndIf}

	InstallDriver_return_success:

	Push 0
	Pop $R0
	
	InstallDriver_return:

	Pop $1
	Pop $0
	
!macroend

!define InstallDriver '!insertmacro "InstallDriver"'

#
# InstallService
#
# Register the service with Windows and start it
#
# Returns: 0 in $R0 on success, otherwise an error message in $R0
#
!macro InstallService

	Push $0
	Push $1

	nsExec::ExecToStack '"$INSTDIR\resources\mullvad-daemon.exe" --register-service'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to install Mullvad service: error $0"
		Goto InstallService_return
	${EndIf}

	nsExec::ExecToStack '"sc.exe" start mullvadvpn'

	Pop $0
	Pop $1

	${If} $0 != ${SERVICE_STARTED}
	${AndIf} $0 != ${SERVICE_START_PENDING}
		StrCpy $R0 "Failed to start Mullvad service: error $0"
		Goto InstallService_return
	${EndIf}

	Push 0
	Pop $R0
	
	InstallService_return:

	Pop $1
	Pop $0

!macroend

!define InstallService '!insertmacro "InstallService"'

#
# RemoveLogsAndCache
#
# Call into helper DLL instructing it to remove all logs and cache
# for the current user, other regular users and the SYSTEM (service) user.
#
!macro RemoveLogsAndCache

	Push $0

	cleanup::RemoveLogsAndCache

	# Discard return value
	Pop $0

	Pop $0

!macroend

!define RemoveLogsAndCache '!insertmacro "RemoveLogsAndCache"'

#
# RemoveSettings
#
# Call into helper DLL instructing it to remove all settings
# for the current user, other regular users and the SYSTEM (service) user.
#
!macro RemoveSettings

	Push $0

	cleanup::RemoveSettings

	# Discard return value
	Pop $0

	Pop $0

!macroend

!define RemoveSettings '!insertmacro "RemoveSettings"'

#
# customInstall
#
# This macro is activated towards the end of the installation
# after all files are copied, shortcuts created, etc
#
!macro customInstall

	Push $R0

	#
	# The electron-builder NSIS logic, that runs before 'customInstall' is activated,
	# makes a copy of the installer file:
	# C:\Users\%CURRENTUSER%\AppData\Roaming\${PRODUCT_NAME}\__installer.exe
	#
	# Let's undo this and remove the entire "Mullvad" folder under "Roaming".
	#
	SetShellVarContext current
	RMDir /r "$APPDATA\${PRODUCT_NAME}"

	${ExtractDriver}
	${InstallDriver}

	${If} $R0 != 0
		MessageBox MB_OK "Fatal error during driver installation: $R0"
		${BreakInstallation}
		Abort
	${EndIf}

	${InstallService}

	${If} $R0 != 0
		MessageBox MB_OK "Fatal error during service installation: $R0"
		${BreakInstallation}
		Abort
	${EndIf}

	Pop $R0

!macroend

###############################################################################
#
# Uninstaller
#
###############################################################################

#
# customRemoveFiles
#
# This macro is activated just after the removal of files have started.
# Shortcuts etc may have been removed but application files remain.
#
!macro customRemoveFiles

	Push $0
	Push $1

	nsExec::ExecToStack '"sc.exe" stop mullvadvpn'

	# Discard return value
	Pop $0

	Sleep 5000

	nsExec::ExecToStack '"sc.exe" delete mullvadvpn'

	# Discard return value
	Pop $0

	Sleep 1000

	# Original removal functionality provided by Electron-builder
    RMDir /r $INSTDIR

	# Check command line arguments
	${GetParameters} $0
	${GetOptions} $0 "/S" $1

	# If not ran silently
	${If} ${Errors}
		${RemoveLogsAndCache}
		MessageBox MB_ICONQUESTION|MB_YESNO "Would you like to remove settings files as well?" IDNO customRemoveFiles_after_remove_settings
		${RemoveSettings}
		customRemoveFiles_after_remove_settings:
	${EndIf}

	Pop $1
	Pop $0

!macroend
