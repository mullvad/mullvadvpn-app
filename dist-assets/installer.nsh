!include stdutils.nsh
!include winver.nsh
#!include strcontains.nsh

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

	Delete "$INSTDIR\mullvadvpn.exe"

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
	File "${PROJECT_DIR}\client-binaries\windows\openvpn\driver\amd64\*"

	${If} ${IsWin7}
		File "${PROJECT_DIR}\client-binaries\windows\openvpn\driver\amd64\ndis5\*"
	${Else}
		File "${PROJECT_DIR}\client-binaries\windows\openvpn\driver\amd64\ndis6\*"
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

	Push $0
	Push $1

	nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" hwids ${TAP_HARDWARE_ID}'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to list hardware IDs: error $0"
		Goto InstallDriver_return
	${EndIf}

	# If the driver is already installed, the hardware ID will be echoed in the command output
	# $1 holds the output from "tapinstall hwids"
	${StrContains} $0 ${TAP_HARDWARE_ID} $1
	StrCmp $0 "" InstallDriver_install_driver

	# Update driver
	nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" update "$TEMP\driver\OemVista.inf" ${TAP_HARDWARE_ID}'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to update TAP driver: error $0"
		Goto InstallDriver_return
	${EndIf}
	
	Goto InstallDriver_return_success

	InstallDriver_install_driver:

	nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" install "$TEMP\driver\OemVista.inf" ${TAP_HARDWARE_ID}'
	
	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to install TAP driver: error $0"
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
# customInstall
#
# This macro is activated towards the end of the installation
# after all files are copied, shortcuts created, etc
#
!macro customInstall

	Push $R0

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
	
	Pop $0

!macroend
