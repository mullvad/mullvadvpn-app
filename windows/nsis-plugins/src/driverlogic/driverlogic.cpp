#include <stdafx.h>
#include "context.h"
#include <libcommon/string.h>
#include <libcommon/valuemapper.h>
#include <windows.h>

// Suppress warnings caused by broken legacy code
#pragma warning (push)
#pragma warning (disable: 4005)
#include <nsis/pluginapi.h>
#pragma warning (pop)

#include <string>
#include <vector>

Context *g_context = nullptr;

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
	// NSIS sometimes frees a plugin module more times than it loads it.
	// This hasn't been observed for this particular plugin but let's up the
	// reference count a bit extra anyway.
	//
	for (int i = 0; i < 100; ++i)
	{
		LoadLibraryW(self);
	}
}

} // anonymous namespace

//
// Initialize
//
// Call this function once during startup.
//
enum class InitializeStatus
{
	GENERAL_ERROR = 0,
	SUCCESS,
};

void __declspec(dllexport) NSISCALL Initialize
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
		if (nullptr == g_context)
		{
			g_context = new Context;

			PinDll();
		}

		pushstring(L"");
		pushint(InitializeStatus::SUCCESS);
	}
	catch (std::exception &err)
	{
		pushstring(common::string::ToWide(err.what()).c_str());
		pushint(InitializeStatus::GENERAL_ERROR);
	}
	catch (...)
	{
		pushstring(L"Unspecified error");
		pushint(InitializeStatus::GENERAL_ERROR);
	}
}

//
// EstablishBaseline
//
// Invoke with the output from "tapinstall hwids tap0901"
// e.g.: driverlogic::EstablishBaseline $1
//
enum class EstablishBaselineStatus
{
	GENERAL_ERROR = 0,
	NO_INTERFACES_PRESENT,
	SOME_INTERFACES_PRESENT,
	MULLVAD_INTERFACE_PRESENT
};

void __declspec(dllexport) NSISCALL EstablishBaseline
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

	if (nullptr == g_context)
	{
		pushstring(L"Initialize() function was not called or was not successful");
		pushint(EstablishBaselineStatus::GENERAL_ERROR);
	}

	try
	{
		using value_type = common::ValueMapper<Context::BaselineStatus, EstablishBaselineStatus>::value_type;

		const common::ValueMapper<Context::BaselineStatus, EstablishBaselineStatus> mapper =
		{
			value_type(Context::BaselineStatus::NO_INTERFACES_PRESENT, EstablishBaselineStatus::NO_INTERFACES_PRESENT),
			value_type(Context::BaselineStatus::SOME_INTERFACES_PRESENT, EstablishBaselineStatus::SOME_INTERFACES_PRESENT),
			value_type(Context::BaselineStatus::MULLVAD_INTERFACE_PRESENT, EstablishBaselineStatus::MULLVAD_INTERFACE_PRESENT)
		};

		const auto status = mapper.map(g_context->establishBaseline(PopString()));

		pushstring(L"");
		pushint(status);
	}
	catch (std::exception &err)
	{
		pushstring(common::string::ToWide(err.what()).c_str());
		pushint(EstablishBaselineStatus::GENERAL_ERROR);
	}
	catch (...)
	{
		pushstring(L"Unspecified error");
		pushint(EstablishBaselineStatus::GENERAL_ERROR);
	}
}

//
// IdentifyNewInterface
//
// Invoke with the output from "tapinstall hwids tap0901"
// e.g.: driverlogic::IdentifyNewInterface $1
//
enum class IdentifyNewInterfaceStatus
{
	GENERAL_ERROR = 0,
	SUCCESS
};

void __declspec(dllexport) NSISCALL IdentifyNewInterface
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

	if (nullptr == g_context)
	{
		pushstring(L"Initialize() function was not called or was not successful");
		pushint(EstablishBaselineStatus::GENERAL_ERROR);
	}

	try
	{
		g_context->recordCurrentState(PopString());

		auto nic = g_context->getNewAdapter();

		pushstring(nic.alias.c_str());
		pushint(IdentifyNewInterfaceStatus::SUCCESS);
	}
	catch (std::exception &err)
	{
		pushstring(common::string::ToWide(err.what()).c_str());
		pushint(IdentifyNewInterfaceStatus::GENERAL_ERROR);
	}
	catch (...)
	{
		pushstring(L"Unspecified error");
		pushint(IdentifyNewInterfaceStatus::GENERAL_ERROR);
	}
}

//
// Deinitialize
//
// Call this function once during shutdown.
//
enum class DeinitializeStatus
{
	GENERAL_ERROR = 0,
	SUCCESS,
};

void __declspec(dllexport) NSISCALL Deinitialize
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
		delete g_context;

		pushstring(L"");
		pushint(InitializeStatus::SUCCESS);
	}
	catch (std::exception &err)
	{
		pushstring(common::string::ToWide(err.what()).c_str());
		pushint(DeinitializeStatus::GENERAL_ERROR);
	}
	catch (...)
	{
		pushstring(L"Unspecified error");
		pushint(DeinitializeStatus::GENERAL_ERROR);
	}

	g_context = nullptr;
}
