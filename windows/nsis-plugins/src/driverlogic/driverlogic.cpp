#include <stdafx.h>
#include "../error.h"
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
		return;
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
// RemoveMullvadTap
//
// Deletes the Mullvad TAP adapter.
//
//
enum class RemoveMullvadTapStatus
{
	GENERAL_ERROR = 0,
	SUCCESS_NO_REMAINING_TAP_ADAPTERS,
	SUCCESS_SOME_REMAINING_TAP_ADAPTERS
};

void __declspec(dllexport) NSISCALL RemoveMullvadTap
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
		pushstring(L"");
		
		switch (Context::DeleteMullvadAdapter())
		{
			case Context::DeletionResult::NO_REMAINING_TAP_ADAPTERS:
			{
				pushint(RemoveMullvadTapStatus::SUCCESS_NO_REMAINING_TAP_ADAPTERS);
				break;
			}

			case Context::DeletionResult::SOME_REMAINING_TAP_ADAPTERS:
			{
				pushint(RemoveMullvadTapStatus::SUCCESS_SOME_REMAINING_TAP_ADAPTERS);
				break;
			}

			default:
			{
				throw std::runtime_error("Unexpected case");
			}
		}
	}
	catch (std::exception &err)
	{
		pushstring(common::string::ToWide(err.what()).c_str());
		pushint(RemoveMullvadTapStatus::GENERAL_ERROR);
	}
	catch (...)
	{
		pushstring(L"Unspecified error");
		pushint(RemoveMullvadTapStatus::GENERAL_ERROR);
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
		pushint(NsisStatus::GENERAL_ERROR);
		return;
	}

	try
	{
		g_context->recordCurrentState();

		auto adapter = g_context->getNewAdapter();

		pushstring(adapter.alias.c_str());
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

//
// RollbackTapAliases
//
// Updating the TAP driver may replace GUIDs and aliases.
// Use this to restore the aliases to their baseline state.
//

void __declspec(dllexport) NSISCALL RollbackTapAliases
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

	if (nullptr == g_context)
	{
		pushstring(L"Initialize() function was not called or was not successful");
		pushint(NsisStatus::GENERAL_ERROR);
		return;
	}

	try
	{
		g_context->recordCurrentState();
		g_context->rollbackTapAliases();

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
// Deinitialize
//
// Call this function once during shutdown.
//

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

	g_context = nullptr;
}
