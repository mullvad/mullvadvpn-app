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
!define DEPRECATED_TAP_HARDWARE_ID "tap0901"
!define TAP_HARDWARE_ID "tapmullvad0901"

# "sc" exit code
!define SERVICE_STARTED 0
!define SERVICE_START_PENDING 2

# Generic return codes for Mullvad nsis plugins
!define MULLVAD_GENERAL_ERROR 0
!define MULLVAD_SUCCESS 1

# Return codes from driverlogic
!define DL_GENERAL_ERROR -1
!define DL_GENERAL_SUCCESS 0

# Log targets
!define LOG_FILE 0
!define LOG_VOID 1

# Windows error codes
!define ERROR_SERVICE_MARKED_FOR_DELETE 1072
!define ERROR_SERVICE_DEPENDENCY_DELETED 1075

# Override electron-builder generated application settings key.
# electron-builder uses a GUID here rather than the application name.
!define INSTALL_REGISTRY_KEY "Software\${PRODUCT_NAME}"

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
# ExtractTapDriver
#
# Extract the correct driver for the current platform
# placing it into $TEMP\tap-driver
#
!macro ExtractTapDriver

	SetOutPath "$TEMP\tap-driver"

	File "${BUILD_RESOURCES_DIR}\..\windows\driverlogic\bin\x64-Release\driverlogic.exe"
	File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\tap-driver\*"

	${If} ${AtLeastWin10}
		File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\tap-driver\win10\*"
	${Else}
		File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\tap-driver\win8\*"
	${EndIf}
	
!macroend

!define ExtractTapDriver '!insertmacro "ExtractTapDriver"'

#
# ExtractWintun
#
# Extract Wintun installer into $TEMP
#
!macro ExtractWintun

	SetOutPath "$TEMP"
	File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\wintun\mullvad-wintun-amd64.msi"

!macroend

!define ExtractWintun '!insertmacro "ExtractWintun"'

#
# RemoveBrandedTap
#
# Try to remove the Mullvad TAP adapter driver
#
!macro RemoveBrandedTap
	Push $0
	Push $1

	nsExec::ExecToStack '"$TEMP\tap-driver\driverlogic.exe" remove ${TAP_HARDWARE_ID}'
	Pop $0
	Pop $1

	Pop $1
	Pop $0

!macroend

!define RemoveBrandedTap '!insertmacro "RemoveBrandedTap"'

#
# RemoveVanillaTap
#
# Try to remove the old Mullvad TAP adapter (with a non-unique hardware ID),
# and uninstall the driver if it's not in use.
#
!macro RemoveVanillaTap
	Push $0
	Push $1

	log::Log "RemoveVanillaTap()"

	nsExec::ExecToStack '"$TEMP\tap-driver\driverlogic.exe" remove-vanilla-tap'

	Pop $0
	Pop $1

	${If} $0 != ${DL_GENERAL_SUCCESS}
		IntFmt $0 "0x%X" $0
		StrCpy $R0 "Failed to remove vanilla TAP adapter: error $0"
		log::LogWithDetails $R0 $1

		Goto RemoveVanillaTap_return
	${EndIf}

	log::Log "RemoveVanillaTap() completed successfully"

	RemoveVanillaTap_return:

	Push 0
	Pop $R0

	Pop $1
	Pop $0

!macroend

!define RemoveVanillaTap '!insertmacro "RemoveVanillaTap"'

#
# InstallTapDriver
#
# Install OpenVPN TAP adapter driver
#
# Returns: 0 in $R0 on success, otherwise an error message in $R0
#
!macro InstallTapDriver

	log::Log "InstallTapDriver()"
	
	Push $0
	Push $1

	#
	# Remove the old-ID Mullvad TAP, if it exists
	#
	${RemoveVanillaTap}

	#
	# Silently approve the certificate before installing the driver
	#
	${IfNot} ${AtLeastWin10}
		log::Log "Adding OpenVPN certificate to the certificate store"

		nsExec::ExecToStack '"$SYSDIR\certutil.exe" -f -addstore TrustedPublisher "$TEMP\tap-driver\driver.cer"'
		Pop $0
		Pop $1

		${If} $0 != 0
			StrCpy $R0 "Failed to add trusted publisher certificate: error $0"
			log::LogWithDetails $R0 $1
		${EndIf}
	${EndIf}

	log::Log "Creating new virtual adapter"
	nsExec::ExecToStack '"$TEMP\tap-driver\driverlogic.exe" install "$TEMP\tap-driver\OemVista.inf"'

	Pop $0
	Pop $1

	${If} $0 != ${DL_GENERAL_SUCCESS}
		IntFmt $0 "0x%X" $0
		StrCpy $R0 "Failed to create virtual adapter: error $0"
		log::LogWithDetails $R0 $1
		Goto InstallTapDriver_return
	${EndIf}

	log::Log "InstallTapDriver() completed successfully"
	
	Push 0
	Pop $R0
	
	InstallTapDriver_return:

	Pop $1
	Pop $0
	
!macroend

!define InstallTapDriver '!insertmacro "InstallTapDriver"'

#
# RemoveWintun
#
# Try to remove Wintun
#
!macro RemoveWintun
	Push $0
	Push $1

	log::Log "RemoveWintun()"

	msiutil::SilentUninstall "$TEMP\mullvad-wintun-amd64.msi"
	Pop $0
	Pop $1

	${If} $0 != ${MULLVAD_SUCCESS}
		StrCpy $R0 "Failed to remove Wintun: $1"
		log::Log $R0
		Goto RemoveWintun_return_only
	${EndIf}

	log::Log "RemoveWintun() completed successfully"

	Push 0
	Pop $R0

	RemoveWintun_return_only:

	Pop $1
	Pop $0

!macroend

!define RemoveWintun '!insertmacro "RemoveWintun"'

#
# InstallWintun
#
# Install Wintun driver
#
# Returns: 0 in $R0 on success, otherwise an error message in $R0
#
!macro InstallWintun

	log::Log "InstallWintun()"

	Push $0
	Push $1

	msiutil::SilentInstall "$TEMP\mullvad-wintun-amd64.msi"
	Pop $0
	Pop $1

	${If} $0 != ${MULLVAD_SUCCESS}
		StrCpy $R0 "Failed to install Wintun: $1"
		log::Log $R0
		Goto InstallWintun_return
	${EndIf}

	log::Log "InstallWintun() completed successfully"

	Push 0
	Pop $R0

	InstallWintun_return:

	Pop $1
	Pop $0

!macroend

!define InstallWintun '!insertmacro "InstallWintun"'

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
	Push $2

	Var /GLOBAL InstallService_Counter
	Push 0
	Pop $InstallService_Counter

	InstallService_RegisterService:

	log::Log "Running $\"mullvad-daemon$\" for it to self-register as a service"
	nsExec::ExecToStack '"$INSTDIR\resources\mullvad-daemon.exe" --register-service'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to install Mullvad service"
		log::LogWithDetails $R0 $1

		#
		# Parse service error
		#
		string::Find $1 "(os error " 0
		Pop $0

		${If} $0 == -1
			log::Log "Failed to parse service error"
			Goto InstallService_return
		${EndIf}

		IntOp $0 $0 + 10

		string::Find $1 ")" $0
		Pop $2

		IntOp $2 $2 - $0

		${If} $2 < 1
			log::Log "Failed to parse service error"
			Goto InstallService_return
		${EndIf}

		StrCpy $0 $1 $2 $0

		StrCpy $R0 "Service error code: $0"
		log::Log $R0

		#
		# Forcibly kill old process if stuck
		#
		${If} $0 == ${ERROR_SERVICE_MARKED_FOR_DELETE}
			log::Log "Attempt to forcibly kill stuck process"
			nsExec::ExecToStack '"$SYSDIR\taskkill.exe" /f /fi "SERVICES eq mullvadvpn"'
			Pop $0
			Pop $1

			# Retry service installation
			IntOp $InstallService_Counter $InstallService_Counter + 1
			${If} $InstallService_Counter < 2
				Goto InstallService_RegisterService
			${EndIf}
		${EndIf}

		Goto InstallService_return
	${EndIf}

	log::Log "Starting service"
	nsExec::ExecToStack '"$SYSDIR\sc.exe" start mullvadvpn'

	Pop $0
	Pop $1

	${If} $0 != ${SERVICE_STARTED}
	${AndIf} $0 != ${SERVICE_START_PENDING}
		${If} $0 == ${ERROR_SERVICE_DEPENDENCY_DELETED}
			StrCpy $R0 'Failed to start Mullvad service: The firewall service "Base Filtering Engine" is missing or unavailable.'
		${Else}
			StrCpy $R0 "Failed to start Mullvad service: error $0"
		${EndIf}
		log::LogWithDetails $R0 $1
		Goto InstallService_return
	${EndIf}

	log::Log "InstallService() completed successfully"
	
	Push 0
	Pop $R0
	
	InstallService_return:

	Pop $2
	Pop $1
	Pop $0

!macroend

!define InstallService '!insertmacro "InstallService"'

#
# InstallTrayIcon
#
# Create or update registry entry for tray icon.
#
!macro InstallTrayIcon

	log::Log "InstallTrayIcon()"

	Push $0
	Push $1

	tray::PromoteTrayIcon
	
	Pop $0
	Pop $1

	${If} $0 != ${MULLVAD_SUCCESS}
		log::LogWithDetails "Failed to install Mullvad tray icon" $1
		Goto InstallTrayIcon_return
	${EndIf}

	log::Log "InstallTrayIcon() completed successfully"
	
	InstallTrayIcon_return:

	Pop $1
	Pop $0

!macroend

!define InstallTrayIcon '!insertmacro "InstallTrayIcon"'

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
# RemoveRelayCache
#
# Call into helper DLL instructing it to remove all relay cache.
# Currently, errors are only logged and not propagated.
#
!macro RemoveRelayCache

	log::Log "RemoveRelayCache()"
	
	Push $0
	Push $1

	cleanup::RemoveRelayCache
	
	Pop $0
	Pop $1
	
	${If} $0 != ${MULLVAD_SUCCESS}
		log::Log "Failed to remove relay cache: $1"
		Goto RemoveRelayCache_return
	${EndIf}

	log::Log "RemoveRelayCache() completed successfully"
	
	RemoveRelayCache_return:

	Pop $1
	Pop $0

!macroend

!define RemoveRelayCache '!insertmacro "RemoveRelayCache"'

#
# AddCLIToEnvironPath
#
# Add "$INSTDIR\resources" to system env PATH,
# unless it already exists.
#
!macro AddCLIToEnvironPath

	log::Log "AddCLIToEnvironPath()"

	Push $0
	Push $1

	pathedit::AddSysEnvPath "$INSTDIR\resources"

	Pop $0
	Pop $1

	${If} $0 != ${MULLVAD_SUCCESS}
		log::LogWithDetails "Failed to add the CLI tools to the system PATH" $1
		Goto UpdatePath_return
	${EndIf}

	log::Log "AddCLIToEnvironPath() completed successfully"

	UpdatePath_return:

	Pop $1
	Pop $0

!macroend

!define AddCLIToEnvironPath '!insertmacro "AddCLIToEnvironPath"'

#
# RemoveCLIFromEnvironPath
#
# Remove "$INSTDIR\resources" to system env PATH
#
!macro RemoveCLIFromEnvironPath

	log::Log "RemoveCLIFromEnvironPath()"

	Push $0
	Push $1

	pathedit::RemoveSysEnvPath "$INSTDIR\resources"

	Pop $0
	Pop $1

	${If} $0 != ${MULLVAD_SUCCESS}
		log::LogWithDetails "Failed to remove the CLI tools from the system PATH" $1
		Goto RemovePath_return
	${EndIf}

	log::Log "RemoveCLIFromEnvironPath() completed successfully"

	RemovePath_return:

	Pop $1
	Pop $0

!macroend

!define RemoveCLIFromEnvironPath '!insertmacro "RemoveCLIFromEnvironPath"'

#
# customInit
#
# This macro is activated right when the installer first starts up.
#
# When the installer is starting, take the opportunity to update registry
# keys to use the new app identifier.
#
# This enables subsequent logic in the installer to correctly identify
# that there is a previous version of the app installed.
#
!macro customInit

	Push $0

	# Application settings key
	# Migrate 2018.(x<6) to current
	registry::MoveKey "HKLM\SOFTWARE\8fa2c331-e09e-5709-bc74-c59df61f0c7e" "HKLM\SOFTWARE\${PRODUCT_NAME}"

	# Discard return value
	Pop $0
	Pop $0

	# Application uninstall key
	# Migrate 2018.(x<6) to current
	registry::MoveKey "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\8fa2c331-e09e-5709-bc74-c59df61f0c7e" "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{${APP_GUID}}"

	# Discard return value
	Pop $0
	Pop $0

	# Application uninstall key
	# Migrate 2018.6 through 2019.7 to current
	registry::MoveKey "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Mullvad VPN" "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{${APP_GUID}}"

	# Discard return value
	Pop $0
	Pop $0

	Pop $0

!macroend

#
# customInstall
#
# This macro is activated towards the end of the installation
# after all files are copied, shortcuts created, etc
#
!macro customInstall

	Push $R0

	log::Initialize LOG_FILE

	log::Log "Running installer for ${PRODUCT_NAME} ${VERSION}"
	log::LogWindowsVersion
	
	#
	# The electron-builder NSIS logic, that runs before 'customInstall' is activated,
	# makes a copy of the installer file:
	# C:\Users\%CURRENTUSER%\AppData\Roaming\${PRODUCT_NAME}\__installer.exe
	#
	# Let's undo this and remove the entire "Mullvad" folder under "Roaming".
	#
	SetShellVarContext current
	RMDir /r "$APPDATA\${PRODUCT_NAME}"

	${RemoveRelayCache}
	
	${ExtractTapDriver}
	${InstallTapDriver}

	${If} $R0 != 0
		MessageBox MB_OK "Fatal error during driver installation: $R0"
		${BreakInstallation}
		Abort
	${EndIf}

	${ExtractWintun}
	${InstallWintun}

	${If} $R0 != 0
		MessageBox MB_OK "$R0"
		${BreakInstallation}
		Abort
	${EndIf}

	${InstallService}

	${If} $R0 != 0
		MessageBox MB_OK "$R0"
		${BreakInstallation}
		Abort
	${EndIf}

	${AddCLIToEnvironPath}
	${InstallTrayIcon}

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

	# Check command line arguments
	Var /GLOBAL FullUninstall

	${GetParameters} $0
	${GetOptions} $0 "/S" $1
	${If} ${Errors}
		Push 1
		log::Initialize LOG_VOID
	${Else}
		Push 0
		log::Initialize LOG_FILE
	${EndIf}
	Pop $FullUninstall

	log::Log "Running uninstaller for ${PRODUCT_NAME} ${VERSION}"

	${RemoveCLIFromEnvironPath}

	# Remove the TAP adapter
	${ExtractTapDriver}
	${RemoveBrandedTap}

	# If not ran silently
	${If} $FullUninstall == 1
		# Remove Wintun
		${ExtractWintun}
		${RemoveWintun}

		${RemoveLogsAndCache}
		MessageBox MB_ICONQUESTION|MB_YESNO "Would you like to remove settings files as well?" IDNO customRemoveFiles_after_remove_settings
		${RemoveSettings}
		customRemoveFiles_after_remove_settings:
	${EndIf}

	# Original removal functionality provided by Electron-builder
	RMDir /r $INSTDIR

	Pop $1
	Pop $0

!macroend
