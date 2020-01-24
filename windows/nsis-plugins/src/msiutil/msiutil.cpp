#include "stdafx.h"
#include <msi.h>
#include <windows.h>
#include <nsis/pluginapi.h>
#include "../error.h"
#include <log/log.h>
#include <libcommon/string.h>
#include <stdexcept>

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

EXTERN_C IMAGE_DOS_HEADER __ImageBase;

void PinDll()
{
	//
	// Apparently NSIS loads and unloads the plugin module for EVERY call it makes to the plugin.
	// This makes it kind of difficult to maintain state.
	//
	// We can work around this by incrementing the module reference count.
	// When NSIS calls FreeLibrary() the reference count decrements and becomes one.
	//

	wchar_t self[MAX_PATH];

	if (0 == GetModuleFileNameW((HINSTANCE)&__ImageBase, self, _countof(self)))
	{
		throw std::runtime_error("Failed to pin plugin module");
	}

	//
	// For some reason, NSIS frees this particular DLL more times than it loads it
	// so we have to up the reference count significantly.
	//
	for (int i = 0; i < 100; ++i)
	{
		LoadLibraryW(self);
	}
}

int WINAPI InstallerHandler(
	LPVOID context,
	UINT type,
	LPCWSTR message
)
{
	PluginLog(message);
	// return 0 to pass it on to the installer
	return 0;
}

} // anonymous namespace


//
// SilentInstall "installer.msi"
//
// Performs a silent install and logs the results.
//
// Return: Empty string and NsisStatus::SUCCESS on success.
//         Otherwise an error string and NsisStatus::GENERAL_ERROR.
//

void __declspec(dllexport) NSISCALL SilentInstall
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
		const auto msiFile = PopString();

		MsiSetInternalUI(INSTALLUILEVEL_NONE, nullptr);
		MsiSetExternalUIW(
			InstallerHandler,
			INSTALLLOGMODE_INFO |
			INSTALLLOGMODE_WARNING |
			INSTALLLOGMODE_ERROR |
			INSTALLLOGMODE_FATALEXIT |
			INSTALLLOGMODE_OUTOFDISKSPACE |
			INSTALLLOGMODE_RMFILESINUSE |
			INSTALLLOGMODE_FILESINUSE,
			nullptr
		);

		const auto installResult = MsiInstallProductW(
			msiFile.c_str(),
			L"ACTION=INSTALL "
			L"REBOOT=ReallySuppress"
		);

		if (ERROR_SUCCESS != installResult)
		{
			std::wstringstream ss;
			ss << L"Install failed: " << installResult;
			pushstring(ss.str().c_str());
			pushint(NsisStatus::GENERAL_ERROR);
			return;
		}

		pushstring(L"");
		pushint(NsisStatus::SUCCESS);
	}
	catch (std::exception & err)
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
// SilentUninstall "installer.msi"
//
// Performs a silent uninstall and logs the results.
//
// Return: Empty string and NsisStatus::SUCCESS on success.
//         Otherwise an error string and NsisStatus::GENERAL_ERROR.
//

void __declspec(dllexport) NSISCALL SilentUninstall
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
		const auto msiFile = PopString();

		MsiSetInternalUI(INSTALLUILEVEL_NONE, nullptr);
		MsiSetExternalUIW(
			InstallerHandler,
			INSTALLLOGMODE_INFO |
			INSTALLLOGMODE_WARNING |
			INSTALLLOGMODE_ERROR |
			INSTALLLOGMODE_FATALEXIT |
			INSTALLLOGMODE_OUTOFDISKSPACE |
			INSTALLLOGMODE_RMFILESINUSE |
			INSTALLLOGMODE_FILESINUSE,
			nullptr
		);

		const auto installResult = MsiInstallProductW(
			msiFile.c_str(),
			L"REMOVE=ALL "
			L"REBOOT=ReallySuppress"
		);

		if (ERROR_SUCCESS != installResult)
		{
			std::wstringstream ss;
			ss << L"Uninstall failed: " << installResult;
			pushstring(ss.str().c_str());
			pushint(NsisStatus::GENERAL_ERROR);
			return;
		}

		pushstring(L"");
		pushint(NsisStatus::SUCCESS);
	}
	catch (std::exception & err)
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
