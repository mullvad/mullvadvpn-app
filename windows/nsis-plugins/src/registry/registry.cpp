#include "stdafx.h"
#include "../error.h"
#include <windows.h>
#include <libcommon/string.h>
#include <libcommon/error.h>
#include <libcommon/registry/registry.h>
#include <libcommon/registry/registrypath.h>
#include <nsis/pluginapi.h>
#include <string>

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

} // anonymous namespace

//
// MoveKey "source" "destination"
//
// Moves a registry key.
//
// Example usage:
//
// MoveKey "HKLM\Software\A" "HKLM\Software\B"
//

void __declspec(dllexport) NSISCALL MoveKey
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
		const auto source = PopString();
		const auto destination = PopString();

		auto typedSource = common::registry::RegistryPath(source);
		auto typedDestination = common::registry::RegistryPath(destination);

		common::registry::Registry::MoveKey(typedSource.key(), typedSource.subkey(), typedDestination.key(),
			typedDestination.subkey(), common::registry::RegistryView::Force64);

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
