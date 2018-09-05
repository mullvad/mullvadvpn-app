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

	log::Log "InstallDriver()"
	
	Push $0
	Push $1

	log::Log "Listing virtual adapters"
	nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" hwids ${TAP_HARDWARE_ID}'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to list virtual adapters: error $0"
		log::LogWithDetails $R0 $1
		Goto InstallDriver_return
	${EndIf}

	log::LogWithDetails "Virtual adapters listing" $1

	log::Log "Calling on plugin to parse adapter data"
	driverlogic::EstablishBaseline $1

	Pop $0
	Pop $1

	${If} $0 == ${EB_GENERAL_ERROR}
		StrCpy $R0 "Failed to parse adapter data: $1"
		log::Log $R0
		Goto InstallDriver_return
	${EndIf}

	Push $0
	Pop $InstallDriver_BaselineStatus
	
	IntCmp $0 ${EB_NO_INTERFACES_PRESENT} InstallDriver_install_driver

	#
	# Driver is already installed and there are one or several virtual adapters present.
	# Update driver.
	#
	log::Log "TAP driver is already installed - Updating to latest version"
	nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" update "$TEMP\driver\OemVista.inf" ${TAP_HARDWARE_ID}'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to update TAP driver: error $0"
		log::LogWithDetails $R0 $1
		Goto InstallDriver_return
	${EndIf}

	${If} $InstallDriver_BaselineStatus == ${EB_MULLVAD_INTERFACE_PRESENT}
		log::Log "Virtual adapter named $\"Mullvad$\" already present on system"
		Goto InstallDriver_return_success
	${EndIf}

	InstallDriver_install_driver:

	#
	# Install driver and create a virtual adapter.
	# If the driver is already installed, this just creates another virtual adapter.
	#
	log::Log "Creating new virtual adapter (this also installs the TAP driver, as necessary)"
	nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" install "$TEMP\driver\OemVista.inf" ${TAP_HARDWARE_ID}'
	
	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to create virtual adapter: error $0"
		log::LogWithDetails $R0 $1
		Goto InstallDriver_return
	${EndIf}

	log::Log "Listing virtual adapters"
	nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" hwids ${TAP_HARDWARE_ID}'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to list virtual adapters: error $0"
		log::LogWithDetails $R0 $1
		Goto InstallDriver_return
	${EndIf}

	log::LogWithDetails "Updated virtual adapters listing" $1

	log::Log "Calling on plugin to diff adapter listings"
	driverlogic::IdentifyNewInterface $1
	
	Pop $0
	Pop $1

	${If} $0 != ${INI_SUCCESS}
		StrCpy $R0 "Failed to identify new virtual adapter: $1"
		log::Log $R0
		Goto InstallDriver_return
	${EndIf}

	log::Log "New virtual adapter is named $\"$1$\""
	
	log::Log "Renaming adapter to $\"Mullvad$\""
	nsExec::ExecToStack '"$SYSDIR\netsh.exe" interface set interface name = "$1" newname = "Mullvad"'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to rename virtual adapter: error $0"
		log::LogWithDetails $R0 $1
		Goto InstallDriver_return
	${EndIf}

	InstallDriver_return_success:

	log::Log "InstallDriver() completed successfully"
	
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

	log::Log "InstallService()"

	Push $0
	Push $1

	log::Log "Running $\"mullvad-daemon$\" for it to self-register as a service"
	nsExec::ExecToStack '"$INSTDIR\resources\mullvad-daemon.exe" --register-service'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to install Mullvad service: error $0"
		log::LogWithDetails $R0 $1
		Goto InstallService_return
	${EndIf}

	log::Log "Starting service"
	nsExec::ExecToStack '"$SYSDIR\sc.exe" start mullvadvpn'

	Pop $0
	Pop $1

	${If} $0 != ${SERVICE_STARTED}
	${AndIf} $0 != ${SERVICE_START_PENDING}
		StrCpy $R0 "Failed to start Mullvad service: error $0"
		log::LogWithDetails $R0 $1
		Goto InstallService_return
	${EndIf}

	log::Log "InstallService() completed successfully"
	
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

	log::Initialize
	log::Log "Running installer for ${PRODUCT_NAME} ${VERSION}"
	
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

	nsExec::ExecToStack '"$SYSDIR\sc.exe" stop mullvadvpn'

	# Discard return value
	Pop $0

	Sleep 5000

	nsExec::ExecToStack '"$SYSDIR\sc.exe" delete mullvadvpn'

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
