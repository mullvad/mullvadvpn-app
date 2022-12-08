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

# "sc" exit code
!define SERVICE_STARTED 0
!define SERVICE_START_PENDING 2

# Generic return codes for Mullvad nsis plugins
!define MULLVAD_GENERAL_ERROR 0
!define MULLVAD_SUCCESS 1

# Return codes from driverlogic
!define DL_GENERAL_SUCCESS 0
!define DL_GENERAL_ERROR 1

# Log targets
!define LOG_INSTALL 0
!define LOG_UNINSTALL 1
!define LOG_VOID 2

# Windows error codes
!define ERROR_SERVICE_DOES_NOT_EXIST 1060
!define ERROR_SERVICE_MARKED_FOR_DELETE 1072
!define ERROR_SERVICE_DEPENDENCY_DELETED 1075

# mullvad-setup status codes
!define MVSETUP_OK 0
!define MVSETUP_ERROR 1
!define MVSETUP_VERSION_NOT_OLDER 2
!define MVSETUP_DAEMON_NOT_RUNNING 3

# Override electron-builder generated application settings key.
# electron-builder uses a GUID here rather than the application name.
!define INSTALL_REGISTRY_KEY "Software\${PRODUCT_NAME}"

!define BLOCK_OUTBOUND_IPV4_FILTER_GUID "{a81c5411-0fd0-43a9-a9be-313f299de64f}"
!define PERSISTENT_BLOCK_OUTBOUND_IPV4_FILTER_GUID "{79860c64-9a5e-48a3-b5f3-d64b41659aa5}"

#
# UnloadPlugins
#
# Ensures that temporary files can be removed by the installer.
#
!macro UnloadPlugins

	Push $0

	log::SetLogTarget ${LOG_VOID}

	# Horrendous hack for unpinning log.dll. Since we do not know the reference count
	# it is safest to unload it from here.
	UnloadPlugins_free_logger:
	System::Call "KERNEL32::GetModuleHandle(t $\"$PLUGINSDIR\log.dll$\")p.r0"
	${If} $0 P<> 0
		System::Call "KERNEL32::FreeLibrary(pr0)"
		Goto UnloadPlugins_free_logger
	${EndIf}

	# The working directory cannot be deleted, so make sure it's set to $TEMP.
	SetOutPath "$TEMP"

	# $PLUGINSDIR is deleted for us as long as nothing is in use.

	Pop $0

!macroend

!define UnloadPlugins '!insertmacro "UnloadPlugins"'

#
# ExtractDriverlogic
#
# Extract device setup tools to $PLUGINSDIR
#
!macro ExtractDriverlogic

	SetOutPath "$PLUGINSDIR"
	File "${BUILD_RESOURCES_DIR}\..\windows\driverlogic\bin\x64-$%CPP_BUILD_MODE%\driverlogic.exe"

!macroend

!define ExtractDriverlogic '!insertmacro "ExtractDriverlogic"'

#
# ExtractWireGuard
#
# Extract Wintun and WireGuardNT installer into $PLUGINSDIR
#
!macro ExtractWireGuard

	SetOutPath "$PLUGINSDIR"
	File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\wintun\wintun.dll"
	File "${BUILD_RESOURCES_DIR}\binaries\x86_64-pc-windows-msvc\wireguard-nt\mullvad-wireguard.dll"

!macroend

!define ExtractWireGuard '!insertmacro "ExtractWireGuard"'

#
# ExtractMullvadSetup
#
# Extract mullvad-setup into $PLUGINSDIR
#
!macro ExtractMullvadSetup

	SetOutPath "$PLUGINSDIR"
	File "${BUILD_RESOURCES_DIR}\mullvad-setup.exe"
	File "${BUILD_RESOURCES_DIR}\..\windows\winfw\bin\x64-$%CPP_BUILD_MODE%\winfw.dll"

!macroend

!define ExtractMullvadSetup '!insertmacro "ExtractMullvadSetup"'

#
# RemoveWintun
#
# Try to remove Wintun
#
!macro RemoveWintun
	Push $0
	Push $1

	log::Log "RemoveWintun()"

	nsExec::ExecToStack '"$PLUGINSDIR\driverlogic.exe" wintun-delete-driver'
	Pop $0
	Pop $1

	${If} $0 != ${DL_GENERAL_SUCCESS}
		StrCpy $R0 "Failed to remove Wintun driver. It may be in use."
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
# RemoveWireGuardNt
#
# Try to remove WireGuardNT
#
!macro RemoveWireGuardNt
	Push $0
	Push $1

	log::Log "RemoveWireGuardNt()"

	nsExec::ExecToStack '"$PLUGINSDIR\driverlogic.exe" wg-nt-cleanup'
	Pop $0
	Pop $1

	${If} $0 != ${DL_GENERAL_SUCCESS}
		IntFmt $0 "0x%X" $0
		StrCpy $R0 "Failed to remove WireGuardNT pool: error $0"
		log::LogWithDetails $R0 $1
		Goto RemoveWireGuardNt_return_only
	${EndIf}

	log::Log "RemoveWireGuardNt() completed successfully"

	Push 0
	Pop $R0

	RemoveWireGuardNt_return_only:

	Pop $1
	Pop $0

!macroend

!define RemoveWireGuardNt '!insertmacro "RemoveWireGuardNt"'
#
# RemoveAbandonedWintunAdapter
#
# Removes old Wintun interface, even if it belongs to a different pool.
#
!macro RemoveAbandonedWintunAdapter
	Push $0
	Push $1

	log::Log "RemoveAbandonedWintunAdapter()"

	nsExec::ExecToStack '"$PLUGINSDIR\driverlogic.exe" wintun-delete-abandoned-device'
	Pop $0
	Pop $1

	${If} $0 != ${DL_GENERAL_SUCCESS}
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

	log::Log "Running $\"mullvad-daemon$\" for it to self-register as a service"
	nsExec::ExecToStack '"$INSTDIR\resources\mullvad-daemon.exe" --register-service'

	Pop $0
	Pop $1

	${If} $0 != 0
		StrCpy $R0 "Failed to install Mullvad service"
		log::LogWithDetails $R0 $1

		#
		# NSIS documentation indicates that failure to launch the target will return
		# the string "error" on the top of the stack ($0 after we pop).
		#
		# However in practice, the failure code from CreateProcess is used.
		# And naturally, comparing to 0xC0000139 fails because... NSIS
		#
		${If} $0 == -1073741511
			log::Log "Failed to launch $\"mullvad-daemon$\" (API issue)"
			Goto InstallService_return
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
# RemoveSplitTunnelDriver
#
# Reset and remove split tunnel driver
#
!macro RemoveSplitTunnelDriver

	log::Log "RemoveSplitTunnelDriver()"

	Push $0
	Push $1

	log::Log "Removing Split Tunneling driver"
	nsExec::ExecToStack '"$PLUGINSDIR\driverlogic.exe" st-remove'
	
	Pop $0
	Pop $1

	${If} $0 != ${DL_GENERAL_SUCCESS}
		IntFmt $0 "0x%X" $0
		StrCpy $R0 "Failed to remove driver: error $0"
		log::LogWithDetails $R0 $1
		Goto RemoveSplitTunnelDriver_return
	${EndIf}
	
	log::Log "RemoveSplitTunnelDriver() completed successfully"

	Push 0
	Pop $R0

	RemoveSplitTunnelDriver_return:

	Pop $1
	Pop $0

!macroend

!define RemoveSplitTunnelDriver '!insertmacro "RemoveSplitTunnelDriver"'

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

	log::Log "ClearFirewallRules()"

	Push $0
	Push $1

	nsExec::ExecToStack '"$PLUGINSDIR\mullvad-setup.exe" reset-firewall'
	Pop $0
	Pop $1

	${If} $0 != ${MVSETUP_OK}
		log::LogWithDetails "ClearFirewallRules() failed" $1
	${Else}
		log::Log "ClearFirewallRules() completed successfully"
	${EndIf}

	Pop $1
	Pop $0

!macroend

!define ClearFirewallRules '!insertmacro "ClearFirewallRules"'

!macro FirewallWarningCheck

	Push $0
	Push $1
	Push $2

	nsExec::ExecToStack '"$SYSDIR\netsh.exe" wfp show security FILTER ${BLOCK_OUTBOUND_IPV4_FILTER_GUID}'
	Pop $1
	Pop $0

	nsExec::ExecToStack '"$SYSDIR\netsh.exe" wfp show security FILTER ${PERSISTENT_BLOCK_OUTBOUND_IPV4_FILTER_GUID}'
	Pop $2
	Pop $0

	${If} $1 == 0
	${OrIf} $2 == 0
		MessageBox MB_ICONEXCLAMATION|MB_OK "Installation failed. Your internet access will be unblocked."
	${EndIf}

	Pop $2
	Pop $1
	Pop $0

!macroend

!define FirewallWarningCheck '!insertmacro "FirewallWarningCheck"'

#
# RemoveCurrentDevice
#
# Remove the device from the account, if there is one
#
!macro RemoveCurrentDevice

	log::Log "RemoveCurrentDevice()"

	Push $0
	Push $1

	nsExec::ExecToStack '"$PLUGINSDIR\mullvad-setup.exe" remove-device'
	Pop $0
	Pop $1

	${If} $0 != ${MVSETUP_OK}
		log::LogWithDetails "RemoveCurrentDevice() failed" $1
	${Else}
		log::Log "RemoveCurrentDevice() completed successfully"
	${EndIf}

	Pop $1
	Pop $0

!macroend

!define RemoveCurrentDevice '!insertmacro "RemoveCurrentDevice"'


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

	${IfNot} ${AtLeastWin10}
		MessageBox MB_ICONSTOP|MB_TOPMOST|MB_OK "Windows versions below 10 are unsupported. The last version to support Windows 7 and 8/8.1 is 2021.6."
		Abort
	${EndIf}

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
# customCheckAppRunning
#
# The default behavior may cause the daemon to disconnect.
#
!macro customCheckAppRunning
	push $R0
	push $R1

	# This must be done here for compatibility with <= 2021.2,
	# since those versions do not kill the GUI in the uninstaller.
	Var /GLOBAL OldVersion
	ReadRegStr $OldVersion HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\${APP_GUID}" "DisplayVersion"
	StrCpy $R0 $OldVersion 4 # Major version
	StrCpy $R1 $OldVersion 1 5 # Minor version
	${If} $R0 > 2021
		Goto customCheckAppRunning_skip_kill
	${OrIf} $R0 == 2021
		${If} $R1 > 2
			Goto customCheckAppRunning_skip_kill
		${EndIf}
	${EndIf}

	# Killing without /f will likely cause the daemon to disconnect.
	nsExec::Exec `taskkill /f /t /im "${APP_EXECUTABLE_FILENAME}"` $R0
	Sleep 500

	customCheckAppRunning_skip_kill:
	pop $R1
	pop $R0

!macroend

#
# customInstall
#
# This macro is activated towards the end of the installation
# after all files are copied, shortcuts created, etc
#
!macro customInstall

	Push $R0

	log::SetLogTarget ${LOG_INSTALL}

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

	${ExtractDriverlogic}
	${RemoveAbandonedWintunAdapter}

	${RemoveSplitTunnelDriver}

	${If} $R0 != 0
		MessageBox MB_OK "$R0"
		Goto customInstall_abort_installation
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

	${FirewallWarningCheck}
	${ExtractMullvadSetup}
	${ClearFirewallRules}

	Abort

	customInstall_skip_abort:

	${UnloadPlugins}

	Pop $R0

!macroend

#
# customUnInstallCheck
#
# This is called from the installer during an upgrade after the old version
# has been uninstalled or failed to uninstall.
#
# The error flag is set if the uninstaller failed to run. Otherwise, $R0
# contains the exit status.
#
!macro customUnInstallCheck

	IfErrors 0 customUnInstallCheck_CheckReturnCode

	log::SetLogTarget ${LOG_UNINSTALL}
	log::Log "Unable to launch uninstaller for previous ${PRODUCT_NAME} version"

	#
	# If $INSTDIR is gone or can be removed, proceed anyway
	#
	IfFileExists $INSTDIR\*.* 0 customUnInstallCheck_Done
	ClearErrors
	RMDir /r $INSTDIR
	IfErrors 0 customUnInstallCheck_Done

	log::Log "Aborting since $INSTDIR exists"
	Goto customUnInstallCheck_Abort

	customUnInstallCheck_CheckReturnCode:

	${if} $R0 == 0
		Goto customUnInstallCheck_Done
	${endif}

	customUnInstallCheck_Abort:

	${FirewallWarningCheck}
	${ExtractMullvadSetup}
	${ClearFirewallRules}

	MessageBox MB_OK "Failed to uninstall a previous version. Contact support or review the logs for more information."
	SetErrorLevel 5
	Abort

	customUnInstallCheck_Done:

!macroend

###############################################################################
#
# Uninstaller
#
###############################################################################

#
# StopAndDeleteService
#
# Stops and deletes the service, attempting to forcibly kill it if necessary.
#
# Returns: 0 in $R0 on success. Otherwise, an error message in $R0 is returned.
#
!macro StopAndDeleteService

	log::Log "StopAndDeleteService()"

	Push $0
	Push $1

	nsExec::ExecToStack '"$SYSDIR\sc.exe" query mullvadvpn'

	Pop $0
	Pop $1

	${If} $0 == ${ERROR_SERVICE_DOES_NOT_EXIST}
		Goto StopAndDeleteService_success
	${EndIf}

	log::Log "Stopping Mullvad service"

	nsExec::ExecToStack '"$SYSDIR\net.exe" stop mullvadvpn'

	Pop $0
	Pop $1

	${If} $0 != 0
		log::LogWithDetails "Failed to stop the service: $0" $1

		# It may be possible to recover by force-killing the service
		# This also "fails" with a generic error if the service isn't running
	${EndIf}

	# Copy over the daemon log from the old install for debugging purposes
	SetShellVarContext all
	CopyFiles /SILENT /FILESONLY "$LOCALAPPDATA\Mullvad VPN\daemon.log" "$LOCALAPPDATA\Mullvad VPN\old-install-daemon.log"

	log::Log "Removing Mullvad service"
	nsExec::ExecToStack '"$SYSDIR\sc.exe" delete mullvadvpn'

	Pop $0
	Pop $R0

	${If} $0 != 0
	${AndIf} $0 != ${ERROR_SERVICE_MARKED_FOR_DELETE}
		log::Log "Failed to delete the service: $0"
		Goto StopAndDeleteService_return_only
	${EndIf}

	Sleep 1000

	#
	# Forcibly kill the service (if marked for deletion)
	#

	Var /GLOBAL DeleteService_Counter
	Push 0
	Pop $DeleteService_Counter

	StopAndDeleteService_check_delete:

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
			Goto StopAndDeleteService_check_delete
		${EndIf}

		StrCpy $R0 "Failed to kill Mullvad service"
		log::Log $R0
		Goto StopAndDeleteService_return_only
	${EndIf}

	StopAndDeleteService_success:

	Push 0
	Pop $R0

	StopAndDeleteService_return_only:

	${If} $R0 == 0
		log::Log "StopAndDeleteService() completed successfully"
	${Else}
		log::Log "StopAndDeleteService() failed"
	${EndIf}

	Pop $1
	Pop $0

!macroend

!define StopAndDeleteService '!insertmacro "StopAndDeleteService"'

#
# customRemoveFiles
#
# This macro is activated just after the removal of files have started.
# Shortcuts etc may have been removed but application files remain.
#
!macro customRemoveFiles

	Push $0
	Push $1
	Push $R0

	# Check command line arguments
	Var /GLOBAL FullUninstall
	Var /GLOBAL Silent
	Var /GLOBAL NewVersion

	log::SetLogTarget ${LOG_UNINSTALL}

	log::Log "Running uninstaller for ${PRODUCT_NAME} ${VERSION}"

	${GetParameters} $0
	${GetOptions} $0 "/S" $1
	${If} ${Errors}
		Push 0
	${Else}
		Push 1
	${EndIf}

	Pop $Silent

	${ExtractDriverlogic}
	${ExtractMullvadSetup}

	${If} ${isUpdated}
		ReadRegStr $NewVersion HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\${APP_GUID}" "NewVersion"

		nsExec::ExecToStack '"$PLUGINSDIR\mullvad-setup.exe" is-older-version $0'
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

	nsExec::Exec `taskkill /t /im "${APP_EXECUTABLE_FILENAME}"` $0
	Sleep 500
	nsExec::Exec `taskkill /f /t /im "${APP_EXECUTABLE_FILENAME}"` $0

	${If} $FullUninstall == 0
		# Save the target tunnel state if we're upgrading
		nsExec::ExecToStack '"$PLUGINSDIR\mullvad-setup.exe" prepare-restart'
		Pop $0
		Pop $1

		${If} $0 != ${MVSETUP_OK}
		${AndIf} $0 != ${MVSETUP_DAEMON_NOT_RUNNING}
			StrCpy $R0 "Failed to send prepare-restart to service"
			log::LogWithDetails $R0 $1
			Goto customRemoveFiles_abort
		${EndIf}
	${EndIf}

	${StopAndDeleteService}

	${If} $R0 != 0
		Goto customRemoveFiles_abort
	${EndIf}

	# Precaution: If the daemon fails to exit gracefully,
	# attempt to remove the driver here. Otherwise, the
	# installer may fail to delete the install dir.

	${RemoveSplitTunnelDriver}

	${If} $R0 != 0
		Goto customRemoveFiles_abort
	${EndIf}

	# Remove application files
	log::Log "Deleting $INSTDIR"
	ClearErrors
	RMDir /r $INSTDIR
	IfErrors 0 customRemoveFiles_final_cleanup

	log::Log "Failed to remove application files"

	customRemoveFiles_abort:

	# Break the install due to inconsistent state
	Delete "$INSTDIR\mullvad vpn.exe"

	${If} $FullUninstall == 1
		${ClearFirewallRules}
	${EndIf}

	log::Log "Aborting uninstaller"

	SetErrorLevel 1
	Abort

	customRemoveFiles_final_cleanup:

	${RemoveCLIFromEnvironPath}

	${If} $FullUninstall == 1
		${ClearFirewallRules}
		${RemoveCurrentDevice}

		${ExtractWireGuard}
		${RemoveWintun}
		${RemoveWireGuardNt}

		log::SetLogTarget ${LOG_VOID}

		${RemoveLogsAndCache}
		${If} $Silent != 1
			MessageBox MB_ICONQUESTION|MB_YESNO "Would you like to remove settings files as well?" IDNO customRemoveFiles_after_remove_settings
		${ElseIf} ${isUpdated}
			Goto customRemoveFiles_after_remove_settings
		${EndIf}

		${RemoveSettings}
		DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "net.mullvad.vpn"

		customRemoveFiles_after_remove_settings:
	${Else}
		log::SetLogTarget ${LOG_VOID}

		SetShellVarContext all
		Delete "$LOCALAPPDATA\Mullvad VPN\uninstall.log"
	${EndIf}

	${UnloadPlugins}

	ClearErrors

	Pop $R0
	Pop $1
	Pop $0

!macroend
