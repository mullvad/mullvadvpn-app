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
!define EB_NO_TAP_ADAPTERS_PRESENT 1
!define EB_SOME_TAP_ADAPTERS_PRESENT 2
!define EB_MULLVAD_ADAPTER_PRESENT 3

# Return codes from driverlogic::IdentifyNewAdapter
!define INA_GENERAL_ERROR 0
!define INA_SUCCESS 1

# Return codes from driverlogic::TAPAdapterCount
!define TAC_GENERAL_ERROR 0
!define TAC_SUCCESS 1

# Return codes from driverlogic::Initialize/Deinitialize
!define DRIVERLOGIC_GENERAL_ERROR 0
!define DRIVERLOGIC_SUCCESS 1

# Return codes from tray::PromoteTrayIcon
!define PTI_GENERAL_ERROR 0
!define PTI_SUCCESS 1

# Return codes from cleanup::RemoveRelayCache
!define RRC_GENERAL_ERROR 0
!define RRC_SUCCESS 1

# Return codes from pathedit::AddSysEnvPath/pathedit::RemoveSysEnvPath
!define PE_GENERAL_ERROR 0
!define PE_SUCCESS 1

# Windows error codes
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
	File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\driver\*"

	${If} ${IsWin7}
		File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\driver\ndis5\*"
	${Else}
		File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\driver\ndis6\*"
	${EndIf}
	
!macroend

!define ExtractDriver '!insertmacro "ExtractDriver"'

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
# RemoveTAP
#
# Try to remove the Mullvad TAP adapter
# and driver if there are no other TAPs available.
#
!macro RemoveTAP
	Push $0
	Push $1

	driverlogic::Initialize

	Pop $0
	Pop $1

	${If} $0 != ${DRIVERLOGIC_SUCCESS}
		Goto RemoveTAP_return_only
	${EndIf}

	driverlogic::TAPAdapterCount

	Pop $0
	Pop $1

	${If} $0 != ${TAC_SUCCESS}
		Goto RemoveTAP_return
	${EndIf}

	${If} $1 == 1
		# Remove the driver altogether
		nsExec::ExecToStack '"$TEMP\driver\tapinstall.exe" remove ${TAP_HARDWARE_ID}'

		Pop $0
		Pop $1
	${ElseIf} $1 > 1
		driverlogic::RemoveMullvadTap

		Pop $0
		Pop $1
	${EndIf}
	
	RemoveTAP_return:

	driverlogic::Deinitialize
	
	Pop $0
	Pop $1

	RemoveTAP_return_only:

	Pop $1
	Pop $0

!macroend

!define RemoveTAP '!insertmacro "RemoveTAP"'

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
	Push $2

	driverlogic::Initialize
	
	Pop $0
	Pop $1
	
	${If} $0 != ${DRIVERLOGIC_SUCCESS}
		StrCpy $R0 "Failed to initialize plugin 'driverlogic': $1"
		log::Log $R0
		Goto InstallDriver_return_only
	${EndIf}
	
	log::Log "Calling on plugin to enumerate network adapters"
	driverlogic::EstablishBaseline

	Pop $0
	Pop $1

	${If} $0 == ${EB_GENERAL_ERROR}
		StrCpy $R0 "Failed to enumerate network adapters: $1"
		log::Log $R0
		Goto InstallDriver_return
	${EndIf}

	Push $0
	Pop $InstallDriver_BaselineStatus
	
	IntCmp $0 ${EB_NO_TAP_ADAPTERS_PRESENT} InstallDriver_install_driver

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

	${If} $InstallDriver_BaselineStatus == ${EB_MULLVAD_ADAPTER_PRESENT}
		log::Log "Virtual adapter with custom name already present on system"

		Goto InstallDriver_return_success
	${EndIf}

	InstallDriver_install_driver:

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

	log::Log "Calling on plugin to identify recently added adapter"
	driverlogic::IdentifyNewAdapter
	
	Pop $0
	Pop $2
	Pop $1

	${If} $0 != ${INA_SUCCESS}
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

	Pop $0
	Pop $1

	log::Log "InstallDriver() completed successfully"
	
	Push 0
	Pop $R0

	# Discard return value
	Pop $0
	Pop $1
	
	InstallDriver_return:

	driverlogic::Deinitialize
	
	Pop $0
	Pop $1

	${If} $0 != ${DRIVERLOGIC_SUCCESS}
		# Do not update $R0
		log::Log "Failed to deinitialize plugin 'driverlogic': $1"
	${EndIf}

	InstallDriver_return_only:

	Pop $2
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

	${If} $0 != ${PTI_SUCCESS}
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
	
	${If} $0 != ${RRC_SUCCESS}
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

	${If} $0 != ${PE_SUCCESS}
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

	${If} $0 != ${PE_SUCCESS}
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

	log::Initialize

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

	SetShellVarContext current
	${RemoveRelayCache}

	# Original removal functionality provided by Electron-builder
    RMDir /r $INSTDIR

	# Check command line arguments
	${GetParameters} $0
	${GetOptions} $0 "/S" $1

	# If not ran silently
	${If} ${Errors}
		# Remove the TAP adapter
		${ExtractDriver}
		${RemoveTAP}

		${RemoveLogsAndCache}
		MessageBox MB_ICONQUESTION|MB_YESNO "Would you like to remove settings files as well?" IDNO customRemoveFiles_after_remove_settings
		${RemoveSettings}
		customRemoveFiles_after_remove_settings:
	${EndIf}

	${RemoveCLIFromEnvironPath}

	Pop $1
	Pop $0

!macroend
