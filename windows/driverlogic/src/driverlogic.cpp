#include "stdafx.h"
#include <iostream>
#include <sstream>
#include <string>
#include <optional>
#include <set>
#include <libcommon/error.h>
#include <libcommon/guid.h>
#include <libcommon/memory.h>
#include <libcommon/network/nci.h>
#include <libcommon/string.h>
#include <setupapi.h>
#include <initguid.h>
#include <devguid.h>
#include <devpkey.h>
#include <newdev.h>


namespace
{

constexpr wchar_t DEPRECATED_TAP_HARDWARE_ID[] = L"tap0901";
constexpr wchar_t TAP_HARDWARE_ID[] = L"tapmullvad0901";
constexpr wchar_t TAP_BASE_ALIAS[] = L"Mullvad";

enum ReturnCodes
{
	GENERAL_ERROR,
	GENERAL_SUCCESS,
	DELETE_NO_ADAPTERS_REMAIN,
	DELETE_SOME_ADAPTERS_REMAIN
};

struct NetworkAdapter
{
	std::wstring guid;
	std::wstring name;
	std::wstring alias;
	std::wstring deviceInstanceId;

	NetworkAdapter(std::wstring guid, std::wstring name, std::wstring alias, std::wstring deviceInstanceId)
		: guid(guid)
		, name(name)
		, alias(alias)
		, deviceInstanceId(deviceInstanceId)
	{
	}

	bool operator<(const NetworkAdapter &rhs) const
	{
		return _wcsicmp(deviceInstanceId.c_str(), rhs.deviceInstanceId.c_str()) < 0;
	}
};

void LogAdapters(const std::wstring &description, const std::set<NetworkAdapter> &adapters)
{
	std::wcout << description << std::endl;

	for (const auto &adapter : adapters)
	{
		std::wcout << L"    Adapter\n"
			<< L"        Guid: " << adapter.guid << L'\n'
			<< L"        Name: " << adapter.name << L'\n'
			<< L"        Alias: " << adapter.alias << L'\n'
			<< L"        Alias: " << adapter.deviceInstanceId
			<< std::endl;
	}
}

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

std::wstring GetDeviceInstanceId(
	HDEVINFO devInfo,
	SP_DEVINFO_DATA *devInfoData
)
{
	DWORD requiredSize = 0;

	SetupDiGetDeviceInstanceIdW(
		devInfo,
		devInfoData,
		nullptr,
		0,
		&requiredSize
	);

	std::vector<wchar_t> deviceInstanceId(1 + requiredSize);

	const auto status = SetupDiGetDeviceInstanceIdW(
		devInfo,
		devInfoData,
		&deviceInstanceId[0],
		requiredSize,
		nullptr
	);

	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "SetupDiGetDeviceInstanceIdW");
	}

	return deviceInstanceId.data();
}

std::wstring GetNetCfgInstanceId(HDEVINFO devInfo, const SP_DEVINFO_DATA &devInfoData)
{
	HKEY hNet = SetupDiOpenDevRegKey(
		devInfo,
		const_cast<SP_DEVINFO_DATA *>(&devInfoData),
		DICS_FLAG_GLOBAL,
		0,
		DIREG_DRV,
		KEY_READ
	);

	if (hNet == INVALID_HANDLE_VALUE)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "SetupDiOpenDevRegKey");
	}

	std::vector<wchar_t> instanceId(MAX_PATH + 1);
	DWORD strSize = static_cast<DWORD>(instanceId.size() * sizeof(wchar_t));

	const auto status = RegGetValueW(
		hNet,
		nullptr,
		L"NetCfgInstanceId",
		RRF_RT_REG_SZ,
		nullptr,
		instanceId.data(),
		&strSize
	);

	RegCloseKey(hNet);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "RegGetValueW");
	}

	return instanceId.data();
}

std::set<NetworkAdapter> GetTapAdapters(const std::wstring &tapHardwareId)
{
	std::set<NetworkAdapter> adapters;

	HDEVINFO devInfo = SetupDiGetClassDevs(
		&GUID_DEVCLASS_NET,
		nullptr,
		nullptr,
		DIGCF_PRESENT
	);

	if (INVALID_HANDLE_VALUE == devInfo)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "SetupDiGetClassDevs() failed");
	}

	common::memory::ScopeDestructor scopeDestructor;
	scopeDestructor += [devInfo]()
	{
		SetupDiDestroyDeviceInfoList(devInfo);
	};

	common::network::Nci nci;

	for (int memberIndex = 0; ; memberIndex++)
	{
		SP_DEVINFO_DATA devInfoData = { 0 };
		devInfoData.cbSize = sizeof(devInfoData);

		if (FALSE == SetupDiEnumDeviceInfo(devInfo, memberIndex, &devInfoData))
		{
			const auto lastError = GetLastError();

			if (ERROR_NO_MORE_ITEMS == lastError)
			{
				// Done
				break;
			}

			THROW_WINDOWS_ERROR(lastError, "SetupDiEnumDeviceInfo() failed while enumerating network adapters");
		}

		try
		{
			//
			// Check whether this is a TAP adapter
			//

			const auto hardwareId = GetDeviceRegistryStringProperty(devInfo, &devInfoData, SPDRP_HARDWAREID);
			if (!hardwareId.has_value()
				|| wcscmp(hardwareId.value().c_str(), tapHardwareId.c_str()) != 0)
			{
				continue;
			}

			//
			// Construct NetworkAdapter
			//

			const std::wstring guid = GetNetCfgInstanceId(devInfo, devInfoData);
			GUID guidObj = common::Guid::FromString(guid);

			adapters.emplace(NetworkAdapter(
				guid,
				GetDeviceStringProperty(devInfo, &devInfoData, &DEVPKEY_Device_DriverDesc),
				nci.getConnectionName(guidObj),
				GetDeviceInstanceId(devInfo, &devInfoData)
			));
		}
		catch (const std::exception & e)
		{
			//
			// Log exception and skip this adapter
			//

			std::cerr << "Skipping TAP adapter due to exception caught while iterating: "
				<< e.what();
		}
	}

	return adapters;
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

std::optional<NetworkAdapter> FindMullvadAdapter(const std::set<NetworkAdapter> &tapAdapters)
{
	if (tapAdapters.empty())
	{
		return std::nullopt;
	}

	//
	// Look for TAP adapter with alias "Mullvad".
	//

	auto findByAlias = [](const std::set<NetworkAdapter> &adapters, const std::wstring &alias)
	{
		const auto it = std::find_if(adapters.begin(), adapters.end(), [&alias](const NetworkAdapter &candidate)
			{
				return 0 == _wcsicmp(candidate.alias.c_str(), alias.c_str());
			});

		return it;
	};

	const auto firstMullvadAdapter = findByAlias(tapAdapters, TAP_BASE_ALIAS);

	if (tapAdapters.end() != firstMullvadAdapter)
	{
		return { *firstMullvadAdapter };
	}

	//
	// Look for TAP adapter with alias "Mullvad-1", "Mullvad-2", etc.
	//

	for (auto i = 0; i < 10; ++i)
	{
		std::wstringstream ss;

		ss << TAP_BASE_ALIAS << L"-" << i;

		const auto alias = ss.str();

		const auto mullvadAdapter = findByAlias(tapAdapters, alias);

		if (tapAdapters.end() != mullvadAdapter)
		{
			return { *mullvadAdapter };
		}
	}

	return std::nullopt;
}

NetworkAdapter FindBrandedTap()
{
	std::set<NetworkAdapter> added = GetTapAdapters(TAP_HARDWARE_ID);

	if (added.empty())
	{
		THROW_ERROR("Could not identify TAP");
	}
	else if (added.size() > 1)
	{
		LogAdapters(L"Enumerable network TAP adapters", added);

		THROW_ERROR("Identified more TAP adapters than expected");
	}

	return *added.begin();
}

enum class DeletionResult
{
	NO_REMAINING_TAP_ADAPTERS,
	SOME_REMAINING_TAP_ADAPTERS
};

DeletionResult DeleteVanillaMullvadAdapter()
{
	auto tapAdapters = GetTapAdapters(DEPRECATED_TAP_HARDWARE_ID);
	std::optional<NetworkAdapter> mullvadAdapter = FindMullvadAdapter(tapAdapters);

	if (!mullvadAdapter.has_value())
	{
		THROW_ERROR("Mullvad TAP adapter not found");
	}

	const auto mullvadGuid = mullvadAdapter.value().guid;

	HDEVINFO devInfo = SetupDiGetClassDevsW(
		&GUID_DEVCLASS_NET,
		nullptr,
		nullptr,
		DIGCF_PRESENT
	);

	if (INVALID_HANDLE_VALUE == devInfo)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "SetupDiGetClassDevsW() failed");
	}

	common::memory::ScopeDestructor cleanupDevList;
	cleanupDevList += [&devInfo]()
	{
		SetupDiDestroyDeviceInfoList(devInfo);
	};

	int numRemainingAdapters = 0;

	for (int memberIndex = 0; ; memberIndex++)
	{
		SP_DEVINFO_DATA devInfoData = { 0 };
		devInfoData.cbSize = sizeof(devInfoData);

		if (FALSE == SetupDiEnumDeviceInfo(devInfo, memberIndex, &devInfoData))
		{
			const auto lastError = GetLastError();

			if (ERROR_NO_MORE_ITEMS == lastError)
			{
				break;
			}

			THROW_WINDOWS_ERROR(lastError, "Error enumerating network adapters");
		}

		try
		{
			const auto hardwareId = GetDeviceRegistryStringProperty(devInfo, &devInfoData, SPDRP_HARDWAREID);

			if (!hardwareId.has_value())
			{
				continue;
			}
			if (0 != wcscmp(DEPRECATED_TAP_HARDWARE_ID, hardwareId.value().data()))
			{
				continue;
			}
			if (0 != GetNetCfgInstanceId(devInfo, devInfoData).compare(mullvadGuid))
			{
				numRemainingAdapters++;
				continue;
			}

			//
			// Delete existing device
			//

			SP_REMOVEDEVICE_PARAMS rmdParams;
			rmdParams.ClassInstallHeader.cbSize = sizeof(SP_CLASSINSTALL_HEADER);
			rmdParams.ClassInstallHeader.InstallFunction = DIF_REMOVE;
			rmdParams.Scope = DI_REMOVEDEVICE_GLOBAL;
			rmdParams.HwProfile = 0;

			auto status = SetupDiSetClassInstallParamsW(devInfo, &devInfoData, &rmdParams.ClassInstallHeader, sizeof(rmdParams));
			if (FALSE == status)
			{
				THROW_WINDOWS_ERROR(GetLastError(), "SetupDiSetClassInstallParamsW");
			}

			status = SetupDiCallClassInstaller(DIF_REMOVE, devInfo, &devInfoData);
			if (FALSE == status)
			{
				THROW_WINDOWS_ERROR(GetLastError(), "SetupDiCallClassInstaller");
			}
		}
		catch (const std::exception & e)
		{
			//
			// Log exception and skip this adapter
			//

			std::cerr << "Skipping TAP adapter due to exception caught while iterating: "
				<< e.what();
		}
	}

	return (numRemainingAdapters > 0)
		? DeletionResult::SOME_REMAINING_TAP_ADAPTERS
		: DeletionResult::NO_REMAINING_TAP_ADAPTERS;
}

} // anonymous namespace

int wmain(int argc, const wchar_t * argv[], const wchar_t * [])
{
	if (2 > argc)
	{
		goto INVALID_ARGUMENTS;
	}

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
		else if (0 == _wcsicmp(argv[1], L"remove-vanilla-tap"))
		{
			switch (DeleteVanillaMullvadAdapter())
			{
				case DeletionResult::NO_REMAINING_TAP_ADAPTERS:
					std::wcout << L"Removed vanilla Mullvad TAP.";
					return DELETE_NO_ADAPTERS_REMAIN;

				case DeletionResult::SOME_REMAINING_TAP_ADAPTERS:
					std::wcout << L"Removed vanilla Mullvad TAP.";
					return DELETE_SOME_ADAPTERS_REMAIN;
			}
		}
		else if (0 == _wcsicmp(argv[1], L"find-tap"))
		{
			const auto tap = FindBrandedTap();
			std::wcout << tap.alias;
		}
		else
		{
			goto INVALID_ARGUMENTS;
		}
	}
	catch (const std::exception &e)
	{
		std::cerr << e.what() << std::endl;
		return GENERAL_ERROR;
	}
	catch (...)
	{
		std::wcerr << L"Unhandled exception." << std::endl;
		return GENERAL_ERROR;
	}
	return GENERAL_SUCCESS;

INVALID_ARGUMENTS:

	std::wcerr << L"Invalid arguments.";
	return GENERAL_ERROR;
}
