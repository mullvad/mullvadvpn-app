#include <stdafx.h>
#include "cleaningops.h"
#include <libcommon/string.h>
#include <windows.h>
#include <nsis/pluginapi.h>
#include <functional>
#include <vector>

enum class RemoveLogsAndCacheStatus
{
	GENERAL_ERROR = 0,
	SUCCESS
};

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
		cleaningops::RemoveLogsServiceUser,
		cleaningops::RemoveCacheServiceUser,
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

	pushint(success ? RemoveLogsAndCacheStatus::SUCCESS : RemoveLogsAndCacheStatus::GENERAL_ERROR);
}

enum class RemoveSettingsStatus
{
	GENERAL_ERROR = 0,
	SUCCESS
};

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
		pushint(RemoveSettingsStatus::SUCCESS);
	}
	catch (...)
	{
		pushint(RemoveSettingsStatus::GENERAL_ERROR);
	}
}

enum class RemoveRelayCacheStatus
{
	GENERAL_ERROR = 0,
	SUCCESS
};

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
		pushint(RemoveRelayCacheStatus::SUCCESS);
	}
	catch (const std::exception &err)
	{
		pushstring(common::string::ToWide(err.what()).c_str());
		pushint(RemoveRelayCacheStatus::GENERAL_ERROR);
	}
	catch (...)
	{
		pushstring(L"Unspecified error");
		pushint(RemoveRelayCacheStatus::GENERAL_ERROR);
	}
}
