#include <stdafx.h>
#include "../error.h"
#include "update.h"
#include <libcommon/string.h>
#include <windows.h>
#include <nsis/pluginapi.h>
#include <functional>
#include <vector>

enum PatchStatus
{
	PATCH_ERROR = 0,
	PATCH_PRESENT,
	PATCH_MISSING
};

void __declspec(dllexport) NSISCALL HasWindows7Sha2Fix
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
		const auto success = update::HasSetupApiSha2Fix();
		pushstring(L"");
		pushint(success ? PatchStatus::PATCH_PRESENT : PatchStatus::PATCH_MISSING);
	}
	catch (const std::exception& err)
	{
		pushstring(common::string::ToWide(err.what()).c_str());
		pushint(PatchStatus::PATCH_ERROR);
	}
	catch (...)
	{
		pushstring(L"Unspecified error");
		pushint(PatchStatus::PATCH_ERROR);
	}
}
