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

!define WINTUN_POOL "Mullvad"

# "sc" exit code
!define SERVICE_STARTED 0
!define SERVICE_START_PENDING 2

# Generic return codes for Mullvad nsis plugins
!define MULLVAD_GENERAL_ERROR 0
!define MULLVAD_SUCCESS 1

# Return codes for KB2921916 check
!define PATCH_ERROR 0
!define PATCH_PRESENT 1
!define PATCH_MISSING 2

# Return codes from driverlogic
!define DL_ADAPTER_NOT_FOUND -2
!define DL_GENERAL_ERROR -1
!define DL_GENERAL_SUCCESS 0

# Log targets
!define LOG_FILE 0
!define LOG_VOID 1

# Windows error codes
!define ERROR_SERVICE_DOES_NOT_EXIST 1060
!define ERROR_SERVICE_MARKED_FOR_DELETE 1072
!define ERROR_SERVICE_DEPENDENCY_DELETED 1075

# mullvad-setup status codes
!define MVSETUP_OK 0
!define MVSETUP_ERROR 1
!define MVSETUP_VERSION_NOT_OLDER 2

# Override electron-builder generated application settings key.
# electron-builder uses a GUID here rather than the application name.
!define INSTALL_REGISTRY_KEY "Software\${PRODUCT_NAME}"

!define BLOCK_OUTBOUND_IPV4_FILTER_GUID "{a81c5411-0fd0-43a9-a9be-313f299de64f}"
!define PERSISTENT_BLOCK_OUTBOUND_IPV4_FILTER_GUID "{79860c64-9a5e-48a3-b5f3-d64b41659aa5}"
!define WINTUN_ADAPTER_GUID "{AFE43773-E1F8-4EBB-8536-576AB86AFE9A}"

#
# ExtractWintun
#
# Extract Wintun installer into $TEMP
#
!macro ExtractWintun

	SetOutPath "$TEMP"
	File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\wintun.dll"
	File "${BUILD_RESOURCES_DIR}\..\windows\driverlogic\bin\x64-Release\driverlogic.exe"

!macroend

!define ExtractWintun '!insertmacro "ExtractWintun"'

#
# ExtractMullvadSetup
#
# Extract mullvad-setup into $TEMP
#
!macro ExtractMullvadSetup

	SetOutPath "$TEMP"
	File "${BUILD_RESOURCES_DIR}\mullvad-setup.exe"
	File "${BUILD_RESOURCES_DIR}\..\windows\winfw\bin\x64-Release\winfw.dll"

!macroend

!define ExtractMullvadSetup '!insertmacro "ExtractMullvadSetup"'

#
# InstallWin7Hotfix
#
# Installs KB2921916. Fixes the "untrusted publisher" issue on Windows 7.
# Returns: 0 in $R0 on success. Otherwise, a non-zero value is returned.
#
!macro InstallWin7Hotfix
	Push $0
	Push $1

	log::Log "InstallWin7Hotfix()"

	osinfo::HasWindows7Sha2Fix
	Pop $0
	Pop $1

	${If} $0 == ${PATCH_PRESENT}
		log::Log "KB2921916 is already installed or superseded"
		Goto InstallWin7Hotfix_return_success
	${EndIf}

	${If} $0 == ${PATCH_ERROR}
		log::LogWithDetails "Detection of KB2921916 failed" $1
		MessageBox MB_OK "Detection of KB2921916 failed"
		Goto InstallWin7Hotfix_return_abort
	${EndIf}

	MessageBox MB_ICONINFORMATION|MB_YESNO "Windows hotfix KB2921916 must be installed for this app to work. Continue?" IDNO InstallWin7Hotfix_return_abort

	log::Log "Extracting KB2921916"

	SetOutPath "$TEMP"
	File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\Windows6.1-KB2921916-x64.msu"

	log::Log "Installing KB2921916"

	nsExec::ExecToStack '"$SYSDIR\wusa.exe" "$TEMP\Windows6.1-KB2921916-x64.msu" /quiet /norestart'
	Pop $0
	Pop $1

	${If} $0 != 0
		${If} $0 == 3010
			SetRebootFlag true
		${Else}
			StrCpy $R0 "Failed to install the hotfix: error $0"
			log::Log $R0
			MessageBox MB_OK "Failed to install the hotfix."
			Goto InstallWin7Hotfix_return_abort
		${EndIf}
	${EndIf}

	InstallWin7Hotfix_return_success:

	Push 0
	Pop $R0

	log::Log "InstallWin7Hotfix() completed successfully"

	Goto InstallWin7Hotfix_return

	InstallWin7Hotfix_return_abort:

	Push 1
	Pop $R0

	log::Log "InstallWin7Hotfix() failed"

	InstallWin7Hotfix_return:

	Pop $1
	Pop $0

!macroend
!define InstallWin7Hotfix '!insertmacro "InstallWin7Hotfix"'

#
# RemoveWintun
#
# Try to remove Wintun
#
!macro RemoveWintun
	Push $0
	Push $1

	log::Log "RemoveWintun()"

	nsExec::ExecToStack '"$TEMP\driverlogic.exe" wintun delete-pool-driver ${WINTUN_POOL}'
	Pop $0
	Pop $1

	${If} $0 != ${DL_GENERAL_SUCCESS}
		IntFmt $0 "0x%X" $0
		StrCpy $R0 "Failed to remove Wintun pool: error $0"
		log::LogWithDetails $R0 $1
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
# RemoveAbandonedWintunAdapter
#
# Removes old Wintun interface, even if it belongs to a different pool.
#
!macro RemoveAbandonedWintunAdapter
	Push $0
	Push $1

	log::Log "RemoveAbandonedWintunAdapter()"

	nsExec::ExecToStack '"$TEMP\driverlogic.exe" remove-device-by-guid ${WINTUN_ADAPTER_GUID}'
	Pop $0
	Pop $1

	${If} $0 != ${DL_GENERAL_SUCCESS}
	${AndIf} $0 != ${DL_ADAPTER_NOT_FOUND}
		IntFmt $0 "0x%X" $0
		StrCpy $R0 "Failed to remove network adapter: error $0"
		log::LogWithDetails $R0 $1
		Goto RemoveAbandonedWintunAdapter_return_only
	${EndIf}

	log::Log "RemoveAbandonedWintunAdapter() completed successfully"

	Push 0
	Pop $R0

	RemoveAbandonedWintunAdapter_return_only:

	Pop $1
	Pop $0

!macroend

!define RemoveAbandonedWintunAdapter '!insertmacro "RemoveAbandonedWintunAdapter"'

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
# MigrateCache
#
# Move old cache files to the new cache directory.
# This is for upgrades from versions <= 2020.8-beta2.
#
!macro MigrateCache

	log::Log "MigrateCache()"

	Push $0
	Push $1

	cleanup::MigrateCache

	Pop $0
	Pop $1

	${If} $0 != ${MULLVAD_SUCCESS}
		log::Log "Failed to migrate cache: $1"
		Goto MigrateCache_return
	${EndIf}

	log::Log "MigrateCache() completed successfully"

	MigrateCache_return:

	Pop $1
	Pop $0

!macroend

!define MigrateCache '!insertmacro "MigrateCache"'

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
# RemoveApiAddressCache
#
# Call into helper DLL instructing it to remove all API address cache.
# Currently, errors are only logged and not propagated.
#
!macro RemoveApiAddressCache

	log::Log "RemoveApiAddressCache()"

	Push $0
	Push $1

	cleanup::RemoveApiAddressCache

	Pop $0
	Pop $1

	${If} $0 != ${MULLVAD_SUCCESS}
		log::Log "Failed to remove address cache: $1"
		Goto RemoveApiAddressCache_return
	${EndIf}

	log::Log "RemoveApiAddressCache() completed successfully"

	RemoveApiAddressCache_return:

	Pop $1
	Pop $0

!macroend

!define RemoveApiAddressCache '!insertmacro "RemoveApiAddressCache"'

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
# ClearFirewallRules
#
# Removes any WFP filters added by the daemon, using mullvad-setup.
# This fails if the daemon is running.
#
!macro ClearFirewallRules

	Push $0
	Push $1

	nsExec::ExecToStack '"$TEMP\mullvad-setup.exe" reset-firewall'
	Pop $0
	Pop $1

	log::Log "Resetting firewall: $0 $1"

	Pop $1
	Pop $0

!macroend

!define ClearFirewallRules '!insertmacro "ClearFirewallRules"'

#
# ClearAccountHistory
#
# Removes account history and any associated keys
#
!macro ClearAccountHistory

	Push $0
	Push $1

	nsExec::ExecToStack '"$TEMP\mullvad-setup.exe" clear-history'
	Pop $0
	Pop $1

	log::Log "Remove account history: $0 $1"

	Pop $1
	Pop $0

!macroend

!define ClearAccountHistory '!insertmacro "ClearAccountHistory"'


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
	registry::MoveKey "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\8fa2c331-e09e-5709-bc74-c59df61f0c7e" "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\${APP_GUID}"

	# Discard return value
	Pop $0
	Pop $0

	# Application uninstall key
	# Migrate 2018.6 through 2019.7 to current
	registry::MoveKey "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Mullvad VPN" "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\${APP_GUID}"

	# Discard return value
	Pop $0
	Pop $0

	# Migrate 2019.8 through 2020.5 to current
	registry::MoveKey "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{${APP_GUID}}" "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\${APP_GUID}"

	# Discard return value
	Pop $0
	Pop $0

	WriteRegStr HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\${APP_GUID}" "NewVersion" "${VERSION}"

	Pop $0

!macroend

#
# customInstall
#
# This macro is activated towards the end of the installation
# after all files are copied, shortcuts created, etc
#
!macro customInstall

	Var /GLOBAL BlockFilterResult
	Var /GLOBAL PersistentBlockFilterResult

	Push $R0

	log::Initialize ${LOG_FILE}

	log::Log "Running installer for ${PRODUCT_NAME} ${VERSION}"
	log::LogWindowsVersion

	#
	# The electron-builder NSIS logic, that runs before 'customInstall' is activated,
	# makes a copy of the installer file:
	# C:\Users\%CURRENTUSER%\AppData\Local\mullvad-vpn-updater\installer.exe
	#
	# Let's undo this and remove the entire folder in AppData.
	#
	SetShellVarContext current
	RMDir /r "$LOCALAPPDATA\mullvad-vpn-updater"

	${MigrateCache}
	${RemoveRelayCache}
	${RemoveApiAddressCache}

	SetOutPath "$TEMP"
	File "${BUILD_RESOURCES_DIR}\..\windows\driverlogic\bin\x64-Release\driverlogic.exe"
	${RemoveAbandonedWintunAdapter}

	${If} ${AtMostWin7}
		${InstallWin7Hotfix}
		${If} $R0 != 0
			Goto customInstall_abort_installation
		${EndIf}
	${EndIf}

	${InstallService}

	${If} $R0 != 0
		MessageBox MB_OK "$R0"
		Goto customInstall_abort_installation
	${EndIf}

	${AddCLIToEnvironPath}
	${InstallTrayIcon}

	Goto customInstall_skip_abort

	customInstall_abort_installation:

	# Aborting the customization step does not undo previous steps taken
	# by the installer (copy files, create shortcut, etc)
	#
	# Therefore we have to break the installed application to
	# prevent users from running a half-installed product
	#
	Delete "$INSTDIR\mullvad vpn.exe"

	nsExec::ExecToStack '"$SYSDIR\netsh.exe" wfp show security FILTER ${BLOCK_OUTBOUND_IPV4_FILTER_GUID}'
	Pop $BlockFilterResult
	Pop $1

	nsExec::ExecToStack '"$SYSDIR\netsh.exe" wfp show security FILTER ${PERSISTENT_BLOCK_OUTBOUND_IPV4_FILTER_GUID}'
	Pop $PersistentBlockFilterResult
	Pop $1

	${If} $BlockFilterResult == 0
	${OrIf} $PersistentBlockFilterResult == 0
		MessageBox MB_ICONEXCLAMATION|MB_YESNO "Do you wish to unblock your internet access? Doing so will leave you with an unsecure connection." IDNO customInstall_abortInstallation_skip_firewall_revert
		${ExtractMullvadSetup}
		${ClearFirewallRules}
	${EndIf}

	customInstall_abortInstallation_skip_firewall_revert:

	Abort

	customInstall_skip_abort:

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

	# Check command line arguments
	Var /GLOBAL FullUninstall
	Var /GLOBAL Silent
	Var /GLOBAL NewVersion

	${GetParameters} $0
	${GetOptions} $0 "/S" $1
	${If} ${Errors}
		Push 0
		log::Initialize ${LOG_VOID}
	${Else}
		Push 1
		log::Initialize ${LOG_FILE}
	${EndIf}

	Pop $Silent

	log::Log "Running uninstaller for ${PRODUCT_NAME} ${VERSION}"

	${ExtractMullvadSetup}

	${If} $Silent == 1
		ReadRegStr $NewVersion HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\${APP_GUID}" "NewVersion"

		nsExec::ExecToStack '"$TEMP\mullvad-setup.exe" is-older-version $0'
		Pop $0
		Pop $1

		${If} $0 == ${MVSETUP_OK}
		${OrIf} $NewVersion == ""
			log::Log "Downgrading. Performing a full uninstall"
			Push 1
		${Else}
			Push 0
		${EndIf}
	${Else}
		Push 1
	${EndIf}

	Pop $FullUninstall

	${If} $FullUninstall == 0
		# Save the target tunnel state if we're upgrading
		nsExec::ExecToStack '"$TEMP\mullvad-setup.exe" prepare-restart'
		Pop $0
		Pop $1
	${EndIf}

	nsExec::ExecToStack '"$SYSDIR\net.exe" stop mullvadvpn'

	# Discard return value
	Pop $0
	Pop $1

	# Copy over the daemon log from the old install for debugging purposes
	SetShellVarContext all
	CopyFiles /SILENT /FILESONLY "$LOCALAPPDATA\Mullvad VPN\daemon.log" "$LOCALAPPDATA\Mullvad VPN\old-install-daemon.log"

	nsExec::ExecToStack '"$SYSDIR\sc.exe" delete mullvadvpn'

	# Discard return value
	Pop $0
	Pop $1

	${If} $0 != 0
	${AndIf} $0 != ${ERROR_SERVICE_MARKED_FOR_DELETE}
		log::Log "Failed to delete Mullvad service: $0"
	${EndIf}

	Sleep 1000

	#
	# Forcibly kill the service (likely marked for deletion)
	#

	Var /GLOBAL DeleteService_Counter
	Push 0
	Pop $DeleteService_Counter

	customRemoveFiles_CheckServiceDeleted:

	nsExec::ExecToStack '"$SYSDIR\sc.exe" query mullvadvpn'

	Pop $0
	Pop $1

	${If} $0 != ${ERROR_SERVICE_DOES_NOT_EXIST}
		log::Log "Attempting to forcibly kill Mullvad service"

		nsExec::ExecToStack '"$SYSDIR\taskkill.exe" /f /fi "SERVICES eq mullvadvpn"'
		Pop $0
		Pop $1

		# Check again whether it was deleted
		IntOp $DeleteService_Counter $DeleteService_Counter + 1
		${If} $DeleteService_Counter < 3
			Sleep 1000
			Goto customRemoveFiles_CheckServiceDeleted
		${EndIf}

		log::Log "Failed to kill Mullvad service"
	${EndIf}

	${RemoveCLIFromEnvironPath}

	${If} $FullUninstall == 1
		${ClearFirewallRules}
		${ClearAccountHistory}

		${ExtractWintun}
		${RemoveWintun}

		${RemoveLogsAndCache}
		${If} $Silent != 1
			MessageBox MB_ICONQUESTION|MB_YESNO "Would you like to remove settings files as well?" IDNO customRemoveFiles_after_remove_settings
			${RemoveSettings}
		${EndIf}
		customRemoveFiles_after_remove_settings:
	${EndIf}

	# Original removal functionality provided by Electron-builder
	RMDir /r $INSTDIR

	Pop $1
	Pop $0

!macroend
