#include "stdafx.h"
#include <iostream>
#include <optional>
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <setupapi.h>
#include <devguid.h>
#include <newdev.h>


namespace
{

const wchar_t TAP_HARDWARE_ID[] = L"tap0901";

enum ReturnCodes
{
	MULLVAD_GENERAL_ERROR,
	MULLVAD_SUCCESS
};

std::optional<std::wstring> GetDeviceRegistryStringProperty(
	HDEVINFO devInfo,
	SP_DEVINFO_DATA *devInfoData,
	DWORD property
)
{
	//
	// Obtain required buffer size
	//

	DWORD requiredSize = 0;

	const auto sizeStatus = SetupDiGetDeviceRegistryPropertyW(
		devInfo,
		devInfoData,
		property,
		nullptr,
		nullptr,
		0,
		&requiredSize
	);

	const DWORD lastError = GetLastError();
	if (FALSE == sizeStatus && ERROR_INSUFFICIENT_BUFFER != lastError)
	{
		// ERROR_INVALID_DATA may mean that the property does not exist
		// TODO: Check if there may be other causes.
		if (ERROR_INVALID_DATA != lastError)
		{
			THROW_WINDOWS_ERROR(lastError, "SetupDiGetDeviceRegistryPropertyW");
		}

		return std::nullopt;
	}

	//
	// Read property
	//

	std::vector<wchar_t> buffer(requiredSize / sizeof(wchar_t));

	const auto status = SetupDiGetDeviceRegistryPropertyW(
		devInfo,
		devInfoData,
		property,
		nullptr,
		reinterpret_cast<PBYTE>(&buffer[0]),
		requiredSize,
		nullptr
	);

	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Failed to read device property");
	}

	return std::make_optional(buffer.data());
}

std::wstring GetDeviceStringProperty(
	HDEVINFO devInfo,
	SP_DEVINFO_DATA *devInfoData,
	const DEVPROPKEY *property
)
{
	//
	// Obtain required buffer size
	//

	DWORD requiredSize = 0;
	DEVPROPTYPE type;

	const auto sizeStatus = SetupDiGetDevicePropertyW(
		devInfo,
		devInfoData,
		property,
		&type,
		nullptr,
		0,
		&requiredSize,
		0
	);

	if (FALSE == sizeStatus)
	{
		const auto lastError = GetLastError();

		if (ERROR_INSUFFICIENT_BUFFER != lastError)
		{
			THROW_WINDOWS_ERROR(lastError, "SetupDiGetDevicePropertyW");
		}
	}

	std::vector<wchar_t> buffer(requiredSize / sizeof(wchar_t));

	//
	// Read property
	//

	const auto status = SetupDiGetDevicePropertyW(
		devInfo,
		devInfoData,
		property,
		&type,
		reinterpret_cast<PBYTE>(&buffer[0]),
		requiredSize,
		nullptr,
		0
	);

	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Failed to read device property");
	}

	return buffer.data();
}

void CreateTapDevice()
{
	GUID classGuid = GUID_DEVCLASS_NET;

	const auto deviceInfoSet = SetupDiCreateDeviceInfoList(&classGuid, 0);
	if (INVALID_HANDLE_VALUE == deviceInfoSet)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "SetupDiCreateDeviceInfoList");
	}

	common::memory::ScopeDestructor scopeDestructor;
	scopeDestructor += [&deviceInfoSet]()
	{
		SetupDiDestroyDeviceInfoList(deviceInfoSet);
	};

	SP_DEVINFO_DATA devInfoData;
	devInfoData.cbSize = sizeof(SP_DEVINFO_DATA);

	auto status = SetupDiCreateDeviceInfoW(
		deviceInfoSet,
		L"NET",
		&classGuid,
		nullptr,
		0,
		DICD_GENERATE_ID,
		&devInfoData
	);

	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "SetupDiCreateDeviceInfoW");
	}

	status = SetupDiSetDeviceRegistryPropertyW(
		deviceInfoSet,
		&devInfoData,
		SPDRP_HARDWAREID,
		reinterpret_cast<const BYTE *>(TAP_HARDWARE_ID),
		sizeof(TAP_HARDWARE_ID) - sizeof(L'\0')
	);

	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "SetupDiSetDeviceRegistryPropertyW");
	}

	//
	// Create a devnode in the PnP HW tree
	//
	status = SetupDiCallClassInstaller(
		DIF_REGISTERDEVICE,
		deviceInfoSet,
		&devInfoData
	);

	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "SetupDiCallClassInstaller");
	}

	std::wcout << L"Created new TAP adapter successfully" << std::endl;
}

void UpdateTapDriver(const std::wstring &infPath)
{
	std::wcout << L"Attempting to install new driver" << std::endl;

	DWORD installFlags = 0;
	BOOL rebootRequired = FALSE;

ATTEMPT_UPDATE:

	auto result = UpdateDriverForPlugAndPlayDevicesW(
		nullptr,
		TAP_HARDWARE_ID,
		infPath.c_str(),
		installFlags,
		&rebootRequired
	);

	if (FALSE == result)
	{
		const auto lastError = GetLastError();

		if (ERROR_NO_MORE_ITEMS == lastError
			&& (installFlags ^ INSTALLFLAG_FORCE))
		{
			std::wcout << L"Driver update failed. Attempting forced install." << std::endl;
			installFlags |= INSTALLFLAG_FORCE;

			goto ATTEMPT_UPDATE;
		}

		THROW_WINDOWS_ERROR(lastError, "UpdateDriverForPlugAndPlayDevicesW");
	}

	//
	// Driver successfully installed or updated
	//

	std::wcout << L"TAP driver update complete. Reboot required: "
		<< rebootRequired << std::endl;
}

} // anonymous namespace

int wmain(int argc, const wchar_t * argv[], const wchar_t * [])
{
	if (2 > argc)
	{
		goto INVALID_ARGUMENTS;
	}

	//
	// Run setup
	//

	try
	{
		if (0 == _wcsicmp(argv[1], L"install"))
		{
			if (3 != argc)
			{
				goto INVALID_ARGUMENTS;
			}

			CreateTapDevice();
			UpdateTapDriver(argv[2]);
		}
		else if (0 == _wcsicmp(argv[1], L"update"))
		{
			if (3 != argc)
			{
				goto INVALID_ARGUMENTS;
			}

			UpdateTapDriver(argv[2]);
		}
		else
		{
			goto INVALID_ARGUMENTS;
		}
	}
	catch (const std::exception &e)
	{
		std::cerr << e.what() << std::endl;
		return MULLVAD_GENERAL_ERROR;
	}
	catch (...)
	{
		std::wcerr << L"Unhandled exception." << std::endl;
		return MULLVAD_GENERAL_ERROR;
	}
	return MULLVAD_SUCCESS;

INVALID_ARGUMENTS:

	std::wcerr << L"Invalid arguments.";
	return MULLVAD_GENERAL_ERROR;
}
