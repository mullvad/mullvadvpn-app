#include "stdafx.h"
#include "../error.h"
#include "driverlogicops.h"
#include <libcommon/string.h>
#include <libcommon/error.h>
#include <libcommon/valuemapper.h>
#include <windows.h>

// Suppress warnings caused by broken legacy code
#pragma warning (push)
#pragma warning (disable: 4005)
#include <nsis/pluginapi.h>
#pragma warning (pop)

//
// RemoveVanillaMullvadTap
//
// Deletes the old Mullvad TAP adapter with ID tap0901.
//
//
enum class RemoveVanillaMullvadTapStatus
{
	GENERAL_ERROR = 0,
	SUCCESS_NO_REMAINING_TAP_ADAPTERS,
	SUCCESS_SOME_REMAINING_TAP_ADAPTERS
};

void __declspec(dllexport) NSISCALL RemoveVanillaMullvadTap
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
		
		switch (driverlogic::DeleteOldMullvadAdapter())
		{
			case driverlogic::DeletionResult::NO_REMAINING_TAP_ADAPTERS:
			{
				pushint(RemoveVanillaMullvadTapStatus::SUCCESS_NO_REMAINING_TAP_ADAPTERS);
				break;
			}

			case driverlogic::DeletionResult::SOME_REMAINING_TAP_ADAPTERS:
			{
				pushint(RemoveVanillaMullvadTapStatus::SUCCESS_SOME_REMAINING_TAP_ADAPTERS);
				break;
			}

			default:
			{
				THROW_ERROR("Unexpected case");
			}
		}
	}
	catch (std::exception &err)
	{
		pushstring(common::string::ToWide(err.what()).c_str());
		pushint(RemoveVanillaMullvadTapStatus::GENERAL_ERROR);
	}
	catch (...)
	{
		pushstring(L"Unspecified error");
		pushint(RemoveVanillaMullvadTapStatus::GENERAL_ERROR);
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

	try
	{
		auto adapter = driverlogic::GetAdapter();

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
