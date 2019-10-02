// pathedit.cpp : Defines the exported functions for the DLL application.
//

#include "stdafx.h"
#include <windows.h>
#include <libcommon/string.h>
#include <libcommon/registry/registry.h>
#include <libcommon/registry/registrypath.h>
#include <libcommon/registry/registrykey.h>
#include <nsis/pluginapi.h>
#include <string>
//#include <log/log.h>

// Suppress warnings caused by broken legacy code
#pragma warning (push)
#pragma warning (disable: 4005)
#include <nsis/pluginapi.h>
#pragma warning (pop)

namespace
{

	std::wstring PopString()
	{
		//
		// NSIS functions popstring() and popstringn() require that you definitely size the buffer
		// before popping the string. Let's do it ourselves instead.
		//

		if (!g_stacktop || !*g_stacktop)
		{
			throw std::runtime_error("NSIS variable stack is corrupted");
		}

		stack_t *th = *g_stacktop;

		std::wstring copy(th->text);

		*g_stacktop = th->next;
		GlobalFree((HGLOBAL)th);

		return copy;
	}

} // anonymous namespace

//
// UpdatePath "path"
//
// Adds "path" to the system PATH environment variable,
// or does nothing if it already exists.
//
// Example usage:
//
// UpdatePath "C:\path\to\directory"
//

enum class UpdatePathStatus
{
	GENERAL_ERROR = 0,
	SUCCESS
};

void __declspec(dllexport) NSISCALL UpdatePath
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
		const auto pathToAppend = PopString();
		static const wchar_t pathKeyName[] = L"SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";
		static const wchar_t pathValName[] = L"Path";

		auto pathRegKey = common::registry::Registry::OpenKey(
			HKEY_LOCAL_MACHINE,
			pathKeyName,
			true,
			common::registry::RegistryView::Force64
		);
		auto pathStr = pathRegKey->readString(pathValName, common::registry::RegistryKey::ValueStringType::ExpandableString);

		// ensure it's not already added
		auto pathTokens = common::string::Tokenize(pathStr, L";");
		if (std::find(pathTokens.begin(), pathTokens.end(), pathToAppend) !=
			pathTokens.end())
		{
			pushstring(L"");
			pushint(UpdatePathStatus::SUCCESS);
			return;
		}

		if (!pathStr.empty()) {
			pathStr.append(L";");
		}
		pathStr.append(pathToAppend);

		pathRegKey->writeValue(pathValName, pathStr, common::registry::RegistryKey::ValueStringType::ExpandableString);

		//SendMessageTimeout(HWND_BROADCAST, WM_SETTINGCHANGE, 0, (LPARAM)L"Environment", SMTO_ABORTIFHUNG, 5000, NULL);

		pushstring(L"");
		pushint(UpdatePathStatus::SUCCESS);
	}
	catch (std::exception &err)
	{
		pushstring(common::string::ToWide(err.what()).c_str());
		pushint(UpdatePathStatus::GENERAL_ERROR);
	}
	catch (...)
	{
		pushstring(L"Unspecified error");
		pushint(UpdatePathStatus::GENERAL_ERROR);
	}
}

//
// RemovePath "path"
//
// Removes "path" to the system PATH environment variable,
// or does nothing if it doesn't exist.
//
// Example usage:
//
// RemovePath "C:\path\to\directory"
//

void __declspec(dllexport) NSISCALL RemovePath
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
		const auto pathToRemove = PopString();
		static const wchar_t pathKeyName[] = L"SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";
		static const wchar_t pathValName[] = L"Path";

		auto pathRegKey = common::registry::Registry::OpenKey(
			HKEY_LOCAL_MACHINE,
			pathKeyName,
			true,
			common::registry::RegistryView::Force64
		);
		auto pathStr = pathRegKey->readString(pathValName, common::registry::RegistryKey::ValueStringType::ExpandableString);

		// check whether the path exists
		auto pathTokens = common::string::Tokenize(pathStr, L";");
		auto match = std::find(pathTokens.begin(), pathTokens.end(), pathToRemove);
		if (match != pathTokens.end())
		{
			pathTokens.erase(match);
			pathStr = common::string::Join(pathTokens, L";");
			pathRegKey->writeValue(pathValName, pathStr, common::registry::RegistryKey::ValueStringType::ExpandableString);

			//SendMessageTimeout(HWND_BROADCAST, WM_SETTINGCHANGE, 0, (LPARAM)L"Environment", SMTO_ABORTIFHUNG, 5000, NULL);
		}

		pushstring(L"");
		pushint(UpdatePathStatus::SUCCESS);
	}
	catch (std::exception &err)
	{
		pushstring(common::string::ToWide(err.what()).c_str());
		pushint(UpdatePathStatus::GENERAL_ERROR);
	}
	catch (...)
	{
		pushstring(L"Unspecified error");
		pushint(UpdatePathStatus::GENERAL_ERROR);
	}
}
