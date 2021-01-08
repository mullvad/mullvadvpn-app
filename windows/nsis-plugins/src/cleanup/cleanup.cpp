#include <stdafx.h>
#include "../error.h"
#include "cleaningops.h"
#include <libcommon/string.h>
#include <windows.h>
#include <nsis/pluginapi.h>
#include <functional>
#include <vector>

void __declspec(dllexport) NSISCALL RemoveLogsAndCache
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

	std::vector<std::function<void()> > functions =
	{
		cleaningops::RemoveLogsCacheCurrentUser,
		cleaningops::RemoveLogsCacheOtherUsers,
		cleaningops::RemoveCacheServiceUser,
		cleaningops::RemoveLogsServiceUser,
	};

	bool success = true;

	//
	// Invoke all functions and take note of any failure.
	//
	for (const auto &function : functions)
	{
		try
		{
			function();
		}
		catch (...)
		{
			success = false;
		}
	}

	pushint(success ? NsisStatus::SUCCESS : NsisStatus::GENERAL_ERROR);
}

void __declspec(dllexport) NSISCALL MigrateCache
(
	HWND hwndParent,
	int string_size,
	LPTSTR variables,
	stack_t** stacktop,
	extra_parameters* extra,
	...
)
{
	EXDLL_INIT();

	try
	{
		cleaningops::MigrateCacheServiceUser();
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

void __declspec(dllexport) NSISCALL RemoveSettings
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
		cleaningops::RemoveSettingsServiceUser();
		pushint(NsisStatus::SUCCESS);
	}
	catch (...)
	{
		pushint(NsisStatus::GENERAL_ERROR);
	}
}

void __declspec(dllexport) NSISCALL RemoveRelayCache
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
		cleaningops::RemoveRelayCacheServiceUser();

		pushstring(L"");
		pushint(NsisStatus::SUCCESS);
	}
	catch (const std::exception &err)
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

void __declspec(dllexport) NSISCALL RemoveApiAddressCache
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
		cleaningops::RemoveApiAddressCacheServiceUser();

		pushstring(L"");
		pushint(NsisStatus::SUCCESS);
	}
	catch (const std::exception &err)
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
