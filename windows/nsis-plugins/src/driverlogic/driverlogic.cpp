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

Context *g_context = nullptr;

namespace
{

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
// Call this function to establish a baseline W.R.T network adapters
// present in the system.
//
// The return value reflects the status of TAP presence in the system.
//
enum class EstablishBaselineStatus
{
	GENERAL_ERROR = 0,
	NO_TAP_ADAPTERS_PRESENT,
	SOME_TAP_ADAPTERS_PRESENT,
	MULLVAD_ADAPTER_PRESENT
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
			value_type(Context::BaselineStatus::NO_TAP_ADAPTERS_PRESENT, EstablishBaselineStatus::NO_TAP_ADAPTERS_PRESENT),
			value_type(Context::BaselineStatus::SOME_TAP_ADAPTERS_PRESENT, EstablishBaselineStatus::SOME_TAP_ADAPTERS_PRESENT),
			value_type(Context::BaselineStatus::MULLVAD_ADAPTER_PRESENT, EstablishBaselineStatus::MULLVAD_ADAPTER_PRESENT)
		};

		const auto status = mapper.map(g_context->establishBaseline());

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
// TapAdapterCount
//
// Return the number of TAP adapters present.
//
enum class TapAdapterCountStatus
{
	GENERAL_ERROR = 0,
	SUCCESS
};

void __declspec(dllexport) NSISCALL TapAdapterCount
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
		g_context->recordCurrentState();

		auto adapters = g_context->getTapAdapters();

		pushint(adapters.size());
		pushint(TapAdapterCountStatus::SUCCESS);
	}
	catch (std::exception &err)
	{
		pushstring(common::string::ToWide(err.what()).c_str());
		pushint(TapAdapterCountStatus::GENERAL_ERROR);
	}
	catch (...)
	{
		pushstring(L"Unspecified error");
		pushint(TapAdapterCountStatus::GENERAL_ERROR);
	}
}


//
// IdentifyNewAdapter
//
// Call this function after installing a TAP adapter.
//
// By comparing with the previously captured baseline we're able to
// identify the new adapter.
//
enum class IdentifyNewAdapterStatus
{
	GENERAL_ERROR = 0,
	SUCCESS
};

void __declspec(dllexport) NSISCALL IdentifyNewAdapter
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
		g_context->recordCurrentState();

		auto adapter = g_context->getNewAdapter();

		pushstring(adapter.alias.c_str());
		pushstring(adapter.guid.c_str());
		pushint(IdentifyNewAdapterStatus::SUCCESS);
	}
	catch (std::exception &err)
	{
		pushstring(common::string::ToWide(err.what()).c_str());
		pushstring(L"");
		pushint(IdentifyNewAdapterStatus::GENERAL_ERROR);
	}
	catch (...)
	{
		pushstring(L"Unspecified error");
		pushstring(L"");
		pushint(IdentifyNewAdapterStatus::GENERAL_ERROR);
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
