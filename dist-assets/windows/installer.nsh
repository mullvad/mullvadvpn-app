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

# Generic return codes for Mullvad nsis plugins
!define MULLVAD_GENERAL_ERROR 0
!define MULLVAD_SUCCESS 1

# Return codes from driverlogic::EstablishBaseline
!define EB_GENERAL_ERROR 0
!define EB_NO_TAP_ADAPTERS_PRESENT 1
!define EB_SOME_TAP_ADAPTERS_PRESENT 2
!define EB_MULLVAD_ADAPTER_PRESENT 3

# Return codes from driverlogic::RemoveMullvadTap
!define RMT_GENERAL_ERROR 0
!define RMT_NO_REMAINING_ADAPTERS 1
!define RMT_SOME_REMAINING_ADAPTERS 2

# Return codes from tapinstall
!define DEVCON_EXIT_OK 0
!define DEVCON_EXIT_REBOOT 1
!define DEVCON_EXIT_FAIL 2
!define DEVCON_EXIT_USAGE 3

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
# ExtractDriver
#
# Extract the correct driver for the current platform
# placing it into $TEMP\driver
#
!macro ExtractDriver

	SetOutPath "$TEMP\driver"

	${If} ${AtLeastWin10}
		File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\driver\ndis6\*"
		File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\driver\ndis6\win10\*"
	${ElseIf} ${AtLeastWin8}
		File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\driver\ndis6\*"
		File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\driver\ndis6\win8\*"
	${Else}
		File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\driver\ndis5\*"
	${EndIf}
	
!macroend

!define ExtractDriver '!insertmacro "ExtractDriver"'

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
# ForceRenameAdapter
#
# For when there's a broken TAP adapter present, such that the adapter name
# we're trying to use is already taken.
#
# This function will iteratively find a name that is not taken.
# Names tried follow the pattern "NewAdapterBaseName-1", "NewAdapterBaseName-2", etc.
#
# Returns: 0 in $R0 on success, otherwise an error message in $R0
#
!macro ForceRenameAdapter CurrentAdapterName NewAdapterBaseName

	Var /GLOBAL ForceRenameAdapter_Counter

	log::Log "ForceRenameAdapter()"

	Push $0
	Push $1

	Push 0
	Pop $ForceRenameAdapter_Counter

	ForceRenameAdapter_retry:

	${If} $ForceRenameAdapter_Counter == 10
		StrCpy $R0 "Exhausted namespace when forcing adapter rename"
		log::Log $R0
		Goto ForceRenameAdapter_return
	${EndIf}

	IntOp $ForceRenameAdapter_Counter $ForceRenameAdapter_Counter + 1
	StrCpy $0 "${NewAdapterBaseName}-$ForceRenameAdapter_Counter"
	log::Log "Renaming adapter to $\"$0$\""

	nsExec::ExecToStack '"$SYSDIR\netsh.exe" interface set interface name = "${CurrentAdapterName}" newname = "$0"'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to rename virtual adapter: error $0"
		log::LogWithDetails $R0 $1
		Goto ForceRenameAdapter_retry
	${EndIf}

	log::Log "ForceRenameAdapter() completed successfully"

	Push 0
	Pop $R0

	ForceRenameAdapter_return:

	Pop $1
	Pop $0

!macroend

!define ForceRenameAdapter '!insertmacro "ForceRenameAdapter"'

#
# RemoveTap
#
# Try to remove the Mullvad TAP adapter
# and driver if there are no other TAPs available.
#
!macro RemoveTap
	Push $0
	Push $1

	driverlogic::Initialize

	Pop $0
	Pop $1

	${If} $0 != ${MULLVAD_SUCCESS}
		Goto RemoveTap_return_only
	${EndIf}

	driverlogic::RemoveMullvadTap

	Pop $0
	Pop $1

	${If} $0 == ${RMT_GENERAL_ERROR}
		Goto RemoveTap_return
	${EndIf}

	${If} $0 == ${RMT_NO_REMAINING_ADAPTERS}
		# Remove the driver altogether
		nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" remove ${TAP_HARDWARE_ID}'

		Pop $0
		Pop $1
	${EndIf}
	
	RemoveTap_return:

	driverlogic::Deinitialize
	
	Pop $0
	Pop $1

	RemoveTap_return_only:

	Pop $1
	Pop $0

!macroend

!define RemoveTap '!insertmacro "RemoveTap"'

#
# InstallDriver
#
# Install tunnel driver or update it if already present on the system
#
# Returns: 0 in $R0 on success, otherwise an error message in $R0
#
!macro InstallDriver

	Var /GLOBAL InstallDriver_BaselineStatus
	Var /GLOBAL InstallDriver_TapName

	log::Log "InstallDriver()"
	
	Push $0
	Push $1

	driverlogic::Initialize
	
	Pop $0
	Pop $1
	
	${If} $0 != ${MULLVAD_SUCCESS}
		StrCpy $R0 "Failed to initialize plugin 'driverlogic': $1"
		log::Log $R0
		Goto InstallDriver_return_only
	${EndIf}
	
	log::Log "Calling on plugin to enumerate network adapters"
	driverlogic::EstablishBaseline

	Pop $0
	Pop $1

	Push $0
	Pop $InstallDriver_BaselineStatus

	${If} $0 == ${EB_GENERAL_ERROR}
		StrCpy $R0 "Failed to enumerate network adapters: $1"
		log::Log $R0
		Goto InstallDriver_return
	${EndIf}

	${IfNot} ${AtLeastWin10}
		#
		# Silently approve the certificate before installing the driver
		#
		log::Log "Adding OpenVPN certificate to the certificate store"

		nsExec::ExecToStack '"$SYSDIR\certutil.exe" -f -addstore TrustedPublisher "$TEMP\driver\driver.cer"'
		Pop $0
		Pop $1

		${If} $0 != 0
			StrCpy $R0 "Failed to add trusted publisher certificate: error $0"
			log::LogWithDetails $R0 $1
		${EndIf}
	${EndIf}

	IntCmp $InstallDriver_BaselineStatus ${EB_NO_TAP_ADAPTERS_PRESENT} InstallDriver_install_driver

	#
	# Driver is already installed and there are one or several virtual adapters present.
	# Update driver.
	#
	log::Log "TAP driver is already installed - Updating to latest version"
	nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" update "$TEMP\driver\OemVista.inf" ${TAP_HARDWARE_ID}'

	Pop $0
	Pop $1

	${If} $0 != ${DEVCON_EXIT_OK}
	${AndIf} $0 != ${DEVCON_EXIT_REBOOT}
		StrCpy $R0 "Failed to update TAP driver: error $0"
		log::LogWithDetails $R0 $1
		Goto InstallDriver_return
	${EndIf}

	#
	# Driver updates will replace the GUIDs and names
	# of our adapters, so let's restore them.
	#
	log::Log "Restoring any changed TAP adapter aliases"
	driverlogic::RollbackTapAliases

	Pop $0
	Pop $1

	${If} $0 != ${MULLVAD_SUCCESS}
		StrCpy $R0 "Failed to roll back TAP adapter aliases: error $0"
		log::LogWithDetails $R0 $1
		Goto InstallDriver_return
	${EndIf}

	${If} $InstallDriver_BaselineStatus == ${EB_MULLVAD_ADAPTER_PRESENT}
		log::Log "Virtual adapter with custom name already present on system"

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

	${If} $0 != ${DEVCON_EXIT_OK}
	${AndIf} $0 != ${DEVCON_EXIT_REBOOT}
		StrCpy $R0 "Failed to create virtual adapter: error $0"
		log::LogWithDetails $R0 $1
		Goto InstallDriver_return
	${EndIf}

	log::Log "Calling on plugin to identify recently added adapter"
	driverlogic::IdentifyNewAdapter
	
	Pop $0
	Pop $1

	${If} $0 != ${MULLVAD_SUCCESS}
		StrCpy $R0 "Failed to identify new adapter: $1"
		log::Log $R0
		Goto InstallDriver_return
	${EndIf}

	StrCpy $InstallDriver_TapName $1
	
	log::Log "New virtual adapter is named $\"$1$\""
	log::Log "Renaming adapter to $\"Mullvad$\""

	nsExec::ExecToStack '"$SYSDIR\netsh.exe" interface set interface name = "$1" newname = "Mullvad"'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to rename virtual adapter: error $0"
		log::LogWithDetails $R0 $1

		${ForceRenameAdapter} $InstallDriver_TapName "Mullvad"

		${If} $R0 != 0
			Goto InstallDriver_return
		${EndIf}
	${EndIf}

	InstallDriver_return_success:

	log::Log "InstallDriver() completed successfully"
	
	Push 0
	Pop $R0
	
	InstallDriver_return:

	driverlogic::Deinitialize
	
	Pop $0
	Pop $1
	
	${If} $0 != ${MULLVAD_SUCCESS}
		# Do not update $R0
		log::Log "Failed to deinitialize plugin 'driverlogic': $1"
	${EndIf}

	InstallDriver_return_only:

	Pop $1
	Pop $0
	
!macroend

!define InstallDriver '!insertmacro "InstallDriver"'

#
# RemoveWintun
#
# Try to remove Wintun
#
!macro RemoveWintun
	Push $0

	log::Log "RemoveWintun()"

	${DisableX64FSRedirection}
	msiutil::SilentUninstall "$TEMP\mullvad-wintun-amd64.msi"
	Pop $0
	Pop $1
	${EnableX64FSRedirection}

	${If} $0 != ${MULLVAD_SUCCESS}
		StrCpy $R0 "Failed to remove Wintun: error $0"
		log::LogWithDetails $R0 $1
		Goto RemoveWintun_return_only
	${EndIf}

	log::Log "RemoveWintun() completed successfully"

	RemoveWintun_return_only:

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

	${DisableX64FSRedirection}
	msiutil::SilentInstall "$TEMP\mullvad-wintun-amd64.msi"
	Pop $0
	Pop $1
	${EnableX64FSRedirection}

	${If} $0 != ${MULLVAD_SUCCESS}
		StrCpy $R0 "Failed to install Wintun: error $0"
		log::LogWithDetails $R0 $1
		Goto InstallWintun_return
	${EndIf}

	log::Log "InstallWintun() completed successfully"

	Push 0
	Pop $R0

	InstallWintun_return:

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
		StrCpy $R0 "Failed to install Mullvad service: error $0"
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
		log::Log "AddCLIToEnvironPath() failed: $0 $1"
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
		log::Log "RemoveCLIFromEnvironPath() failed: $0 $1"
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
	
	${ExtractDriver}
	${InstallDriver}

	${If} $R0 != 0
		MessageBox MB_OK "Fatal error during driver installation: $R0"
		${BreakInstallation}
		Abort
	${EndIf}

	${ExtractWintun}
	${InstallWintun}

	${If} $R0 != 0
		MessageBox MB_OK "Fatal error during Wintun installation: $R0"
		${BreakInstallation}
		Abort
	${EndIf}

	${InstallService}

	${If} $R0 != 0
		MessageBox MB_OK "Fatal error during service installation: $R0"
		${BreakInstallation}
		Abort
	${EndIf}
	
	${InstallTrayIcon}

	${AddCLIToEnvironPath}

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

	# If not ran silently
	${If} $FullUninstall == 1
		# Remove Wintun
		${ExtractWintun}
		${RemoveWintun}

		# Remove the TAP adapter
		${ExtractDriver}
		${RemoveTap}

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
