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

using namespace common::registry;
using ValueStringType = RegistryKey::ValueStringType;
using common::string::Lower;

using std::vector;
using std::wstring;
using std::wstring;

static vector<wstring>::const_iterator FindSysPath(const vector<wstring> &pathTokens, const wstring &pathStr)
{
	wstring lowerPathStr(Lower(pathStr));

	return std::find_if(
		pathTokens.begin(),
		pathTokens.end(),
		[&lowerPathStr](const wstring &elem)
		{
			return Lower(elem).compare(lowerPathStr) == 0;
		}
	);
}

static bool SysPathExists(const wstring &allPaths, const wstring &pathToFind)
{
	auto pathTokens = common::string::Tokenize(allPaths, L";");
	return FindSysPath(pathTokens, pathToFind) != pathTokens.end();
}

static const wchar_t pathKeyName[] = L"SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";
static const wchar_t pathValName[] = L"Path";

//
// AddSysEnvPath "path"
//
// Adds "path" to the system PATH environment variable,
// or does nothing if it already exists.
//
// Example usage:
//
// AddSysEnvPath "C:\path\to\directory"
//

enum class UpdatePathStatus
{
	GENERAL_ERROR = 0,
	SUCCESS
};

void __declspec(dllexport) NSISCALL AddSysEnvPath
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

		auto pathRegKey = Registry::OpenKey(
			HKEY_LOCAL_MACHINE,
			pathKeyName,
			true,
			RegistryView::Force64
		);
		auto pathStr = pathRegKey->readString(pathValName, ValueStringType::ExpandableString);

		if (SysPathExists(pathStr, pathToAppend))
		{
			pushstring(L"");
			pushint(UpdatePathStatus::SUCCESS);
			return;
		}

		if (!pathStr.empty())
		{
			pathStr.append(L";");
		}
		pathStr.append(pathToAppend);

		pathRegKey->writeValue(pathValName, pathStr, ValueStringType::ExpandableString);

		SendMessageTimeout(
			HWND_BROADCAST,
			WM_SETTINGCHANGE,
			0,
			(LPARAM)L"Environment",
			SMTO_ABORTIFHUNG,
			5000,
			NULL
		);

		pushstring(L"");
		pushint(UpdatePathStatus::SUCCESS);
	}
	catch (const std::exception &err)
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
// RemoveSysEnvPath "path"
//
// Removes "path" to the system PATH environment variable,
// or does nothing if it doesn't exist.
//
// Example usage:
//
// RemoveSysEnvPath "C:\path\to\directory"
//

void __declspec(dllexport) NSISCALL RemoveSysEnvPath
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

		auto pathRegKey = Registry::OpenKey(
			HKEY_LOCAL_MACHINE,
			pathKeyName,
			true,
			RegistryView::Force64
		);
		auto pathStr = pathRegKey->readString(pathValName, ValueStringType::ExpandableString);

		// remove value if it exists in PATH
		auto pathTokens = common::string::Tokenize(pathStr, L";");
		auto match = FindSysPath(pathTokens, pathToRemove);
		if (match != pathTokens.end())
		{
			pathTokens.erase(match);
			pathStr = common::string::Join(pathTokens, L";");
			pathRegKey->writeValue(pathValName, pathStr, ValueStringType::ExpandableString);

			SendMessageTimeout(HWND_BROADCAST, WM_SETTINGCHANGE, 0, (LPARAM)L"Environment", SMTO_ABORTIFHUNG, 5000, NULL);
		}

		pushstring(L"");
		pushint(UpdatePathStatus::SUCCESS);
	}
	catch (const std::exception &err)
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
