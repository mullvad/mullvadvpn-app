#include "stdafx.h"
#include "../error.h"
#include "trayparser.h"
#include "trayjuggler.h"
#include "resource.h"
#include <windows.h>
#include <log/log.h>
#include <libcommon/string.h>
#include <libcommon/error.h>
#include <libcommon/registry/registry.h>
#include <libcommon/resourcedata.h>
#include <libcommon/filesystem.h>
#include <libcommon/process/process.h>
#include <libcommon/security.h>
#include <nsis/pluginapi.h>
#include <filesystem>

namespace
{

EXTERN_C IMAGE_DOS_HEADER __ImageBase;

void InjectMullvadRecord(TrayJuggler &juggler)
{
	//
	// There's no existing mullvad tray record in the system.
	// Load a template mullvad record, which is then fixed up and injected.
	//

	auto moduleHandle = reinterpret_cast<HMODULE>(&__ImageBase);

	auto resource = common::resourcedata::LoadBinaryResource(moduleHandle, MULLVAD_TRAY_RECORD);

	if (resource.size != sizeof(ICON_STREAMS_RECORD))
	{
		THROW_ERROR("Invalid tray template, size mismatch");
	}

	ICON_STREAMS_RECORD newRecord(*reinterpret_cast<const ICON_STREAMS_RECORD *>(resource.data));

	juggler.injectRecord(newRecord);
}

void UpdateRegistry(common::registry::RegistryKey &regkey, const std::wstring &valueName, const TrayJuggler &juggler)
{
	//
	// Construct path to 'explorer.exe'
	//

	const auto windir = common::fs::GetKnownFolderPath(FOLDERID_Windows);
	const auto explorer = std::filesystem::path(windir).append(L"explorer.exe");

	//
	// Determine process id of active instance(s).
	// There's a setting in 'explorer' that will open all folder views as separate processes.
	//

	const auto pids = common::process::GetAllProcessIdsFromName(explorer);

	if (pids.empty())
	{
		THROW_ERROR("Could not determine PID of explorer.exe");
	}

	//
	// Make a copy of the process security context before we start terminating processes.
	// Should we make an effort to choose the process that has been alive the longest?
	//
	auto context = common::security::DuplicateSecurityContext(*pids.begin());

	size_t terminated = 0;

	for (auto pid : pids)
	{
		auto handle = OpenProcess(PROCESS_TERMINATE, FALSE, pid);

		if (nullptr != handle)
		{
			//
			// 'winlogon' is monitoring 'explorer' and immediately
			// restarts it if the exit code is 0.
			//
			if (FALSE != TerminateProcess(handle, 1))
			{
				WaitForSingleObject(handle, INFINITE);
				++terminated;
			}

			CloseHandle(handle);
		}
	}

	if (0 == terminated)
	{
		THROW_ERROR("Could not terminate explorer.exe");
	}

	//
	// We've terminated one/more instances of explorer.exe so we have to follow through.
	//

	if (pids.size() != terminated)
	{
		PluginLog(L"Could not terminate all instances of explorer.exe");
	}

	regkey.writeValue(valueName, juggler.pack());

	common::process::RunInContext(*context, explorer);
}

} // anonymous namespace

//
// PromoteTrayIcon
//
// Ensure the GUI's tray icon is placed in the visible part of the notification area.
// This is accomplished by updating a binary blob in the registry.
//

void __declspec(dllexport) NSISCALL PromoteTrayIcon
(
	HWND hwndParent,
	int string_size,
	LPTSTR variables,
	stack_t **stacktop,
	extra_parameters *extra,
	...
)
{
	EXDLL_INIT();

	try
	{
		static const wchar_t keyName[] = L"Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\TrayNotify";
		static const wchar_t valueName[] = L"IconStreams";

		auto regkey = common::registry::Registry::OpenKey(HKEY_CURRENT_USER, keyName, true);

		TrayParser parser(regkey->readBinaryBlob(valueName));

		TrayJuggler juggler(parser);

		bool updateRegistry = true;

		if (auto mullvadRecord = juggler.findRecord(L"Mullvad VPN"))
		{
			if (ICON_STREAMS_VISIBILITY::SHOW_ICON_AND_NOTIFICATIONS == mullvadRecord->Visibility)
			{
				updateRegistry = false;
			}
			else
			{
				juggler.promoteRecord(mullvadRecord);
			}
		}
		else
		{
			InjectMullvadRecord(juggler);
		}

		//
		// Only update the registry if the record/record set was updated.
		//

		if (updateRegistry)
		{
			UpdateRegistry(*regkey, valueName, juggler);
		}

		pushstring(L"");
		pushint(NsisStatus::SUCCESS);
	}
	catch (std::exception &err)
	{
		pushstring(common::string::ToWide(err.what()).c_str());
		pushint(NsisStatus::GENERAL_ERROR);
	}
	catch (...)
	{
		pushstring(L"Unspecified error");
		pushint(NsisStatus::GENERAL_ERROR);
	}
}
