// pathedit.cpp : Defines the exported functions for the DLL application.
//

#include "stdafx.h"
#include "../error.h"
#include <windows.h>
#include <libcommon/string.h>
#include <libcommon/registry/registry.h>
#include <libcommon/registry/registrypath.h>
#include <libcommon/registry/registrykey.h>
#include <libcommon/error.h>
#include <nsis/pluginapi.h>
#include <string>

// Suppress warnings caused by broken legacy code
#pragma warning (push)
#pragma warning (disable: 4005)
#include <nsis/pluginapi.h>
#pragma warning (pop)

using namespace common::registry;
using ValueStringType = RegistryKey::ValueStringType;
using common::string::Lower;

static constexpr wchar_t pathKeyName[] = L"SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";
static constexpr wchar_t pathValName[] = L"Path";
static constexpr size_t messageTimeoutInterval = 5;

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
		THROW_ERROR("NSIS variable stack is corrupted");
	}

	stack_t *th = *g_stacktop;

	std::wstring copy(th->text);

	*g_stacktop = th->next;
	GlobalFree((HGLOBAL)th);

	return copy;
}

std::vector<std::wstring>::const_iterator FindSysPath(const std::vector<std::wstring> &pathTokens, const std::wstring &path)
{
	const std::wstring lowerPath = Lower(path);

	return std::find_if(
		pathTokens.begin(),
		pathTokens.end(),
		[&lowerPath](const std::wstring &elem)
		{
			return Lower(elem).compare(lowerPath) == 0;
		}
	);
}

bool SysPathExists(const std::wstring &allPaths, const std::wstring &pathToFind)
{
	auto pathTokens = common::string::Tokenize(allPaths, L";");
	return FindSysPath(pathTokens, pathToFind) != pathTokens.end();
}

std::wstring ReadPathValue(const RegistryKey &pathKey)
{
	// Some applications will replace the PATH value with a regular string;
	// use this type as a fallback.
	try
	{
		return pathKey.readString(pathValName, ValueStringType::ExpandableString);
	}
	catch (const std::exception &)
	{
		return pathKey.readString(pathValName, ValueStringType::RegularString);
	}
}

} // anonymous namespace

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
		{
			const auto pathToAppend = PopString();
			auto pathRegKey = Registry::OpenKey(
				HKEY_LOCAL_MACHINE,
				pathKeyName,
				true,
				RegistryView::Force64
			);
			std::wstring path = ReadPathValue(*pathRegKey);

			if (SysPathExists(path, pathToAppend))
			{
				pushstring(L"");
				pushint(NsisStatus::SUCCESS);
				return;
			}

			if (!path.empty())
			{
				path.append(L";");
			}
			path.append(pathToAppend);

			pathRegKey->writeValue(pathValName, path, ValueStringType::ExpandableString);
			pathRegKey->flush();
		}

		DWORD result;

		auto status = SendMessageTimeoutA(HWND_BROADCAST, WM_SETTINGCHANGE,	0,
			(LPARAM)"Environment", SMTO_ABORTIFHUNG, messageTimeoutInterval, &result);

		if (0 == status)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "SendMessageTimeoutA");
		}

		status = SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0,
			(LPARAM)L"Environment", SMTO_ABORTIFHUNG, messageTimeoutInterval, &result);

		if (0 == status)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "SendMessageTimeoutW");
		}

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

		bool updatedPath = false;

		{
			auto pathRegKey = Registry::OpenKey(
				HKEY_LOCAL_MACHINE,
				pathKeyName,
				true,
				RegistryView::Force64
			);
			std::wstring path = ReadPathValue(*pathRegKey);

			// remove value if it exists in PATH
			auto pathTokens = common::string::Tokenize(path, L";");
			auto match = FindSysPath(pathTokens, pathToRemove);
			if (match != pathTokens.end())
			{
				pathTokens.erase(match);
				path = common::string::Join(pathTokens, L";");
				pathRegKey->writeValue(pathValName, path, ValueStringType::ExpandableString);
				updatedPath = true;
			}
		}

		if (updatedPath)
		{
			DWORD result;

			auto status = SendMessageTimeoutA(HWND_BROADCAST, WM_SETTINGCHANGE, 0,
				(LPARAM)"Environment", SMTO_ABORTIFHUNG, messageTimeoutInterval, &result);

			if (0 == status)
			{
				THROW_WINDOWS_ERROR(GetLastError(), "SendMessageTimeoutA");
			}

			status = SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0,
				(LPARAM)L"Environment", SMTO_ABORTIFHUNG, messageTimeoutInterval, &result);

			if (0 == status)
			{
				THROW_WINDOWS_ERROR(GetLastError(), "SendMessageTimeoutW");
			}
		}

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
