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
#include <libcommon/registry/registry.h>
#include <libcommon/string.h>
#include <setupapi.h>
#include <initguid.h>
#include <devguid.h>
#include <devpkey.h>
#include <newdev.h>
#include <cfgmgr32.h>
#include <io.h>
#include <fcntl.h>


namespace
{

constexpr wchar_t DEPRECATED_TAP_HARDWARE_ID[] = L"tap0901";
constexpr wchar_t TAP_HARDWARE_ID[] = L"tapmullvad0901";
constexpr wchar_t TAP_BASE_ALIAS[] = L"Mullvad";

enum ReturnCodes
{
	GENERAL_ERROR,
	GENERAL_SUCCESS
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
			<< L"        Device instance ID: " << adapter.deviceInstanceId
			<< std::endl;
	}
}

void Log(const std::wstring &str)
{
	std::wcout << str << std::endl;
}

void LogError(const std::wstring &str)
{
	std::wcerr << str << std::endl;
}

std::optional<std::wstring> GetDeviceRegistryStringProperty(
	HDEVINFO devInfo,
	const SP_DEVINFO_DATA &devInfoData,
	DWORD property
)
{
	//
	// Obtain required buffer size
	//

	DWORD requiredSize = 0;

	const auto sizeStatus = SetupDiGetDeviceRegistryPropertyW(
		devInfo,
		const_cast<SP_DEVINFO_DATA*>(&devInfoData),
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
		const_cast<SP_DEVINFO_DATA*>(&devInfoData),
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
	const SP_DEVINFO_DATA &devInfoData,
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
		const_cast<SP_DEVINFO_DATA*>(&devInfoData),
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
		const_cast<SP_DEVINFO_DATA*>(&devInfoData),
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
	const SP_DEVINFO_DATA &devInfoData
)
{
	DWORD requiredSize = 0;

	SetupDiGetDeviceInstanceIdW(
		devInfo,
		const_cast<SP_DEVINFO_DATA*>(&devInfoData),
		nullptr,
		0,
		&requiredSize
	);

	std::vector<wchar_t> deviceInstanceId(1 + requiredSize);

	const auto status = SetupDiGetDeviceInstanceIdW(
		devInfo,
		const_cast<SP_DEVINFO_DATA *>(&devInfoData),
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

bool DeleteDevice(HDEVINFO devInfo, const SP_DEVINFO_DATA &devInfoData)
{
	const auto data = const_cast<SP_DEVINFO_DATA *>(&devInfoData);

	wchar_t devId[MAX_DEVICE_ID_LEN];
	if (CR_SUCCESS != CM_Get_Device_IDW(data->DevInst, devId, sizeof(devId) / sizeof(devId[0]), 0))
	{
		// skip
		return false;
	}

	SP_REMOVEDEVICE_PARAMS rmdParams = { 0 };
	rmdParams.ClassInstallHeader.cbSize = sizeof(SP_CLASSINSTALL_HEADER);
	rmdParams.ClassInstallHeader.InstallFunction = DIF_REMOVE;
	rmdParams.Scope = DI_REMOVEDEVICE_GLOBAL;
	rmdParams.HwProfile = 0;

	auto status = SetupDiSetClassInstallParamsW(devInfo, data, &rmdParams.ClassInstallHeader, sizeof(rmdParams));
	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "SetupDiSetClassInstallParamsW");
	}

	status = SetupDiCallClassInstaller(DIF_REMOVE, devInfo, data);
	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "SetupDiCallClassInstaller");
	}

	return true;
}

void ForEachNetworkDevice(const std::optional<std::wstring> hwId, std::function<bool(HDEVINFO, const SP_DEVINFO_DATA &)> func)
{
	HDEVINFO devInfo = SetupDiGetClassDevsW(
		&GUID_DEVCLASS_NET,
		nullptr,
		nullptr,
		DIGCF_PRESENT
	);

	if (INVALID_HANDLE_VALUE == devInfo)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "SetupDiGetClassDevsW");
	}

	common::memory::ScopeDestructor cleanupDevList;
	cleanupDevList += [&devInfo]()
	{
		SetupDiDestroyDeviceInfoList(devInfo);
	};

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

			THROW_WINDOWS_ERROR(lastError, "Enumerating network adapters");
		}

		if (hwId.has_value())
		{
			try
			{
				const auto hardwareId = GetDeviceRegistryStringProperty(devInfo, devInfoData, SPDRP_HARDWAREID);

				if (!hardwareId.has_value() ||
					0 != hwId->compare(hardwareId.value()))
				{
					continue;
				}
			}
			catch (const std::exception & e)
			{
				//
				// Skip this adapter
				//

				std::wstringstream ss;
				ss << L"Skipping TAP adapter due to exception caught while iterating: "
					<< common::string::ToWide(e.what());
				LogError(ss.str());
				continue;
			}
		}

		if (!func(devInfo, devInfoData))
		{
			break;
		}
	}
}

std::set<NetworkAdapter> GetNetworkAdapters(const std::optional<std::wstring> hardwareId)
{
	std::set<NetworkAdapter> adapters;
	common::network::Nci nci;

	ForEachNetworkDevice(hardwareId, [&](HDEVINFO devInfo, const SP_DEVINFO_DATA &devInfoData) {
		try
		{
			//
			// Construct NetworkAdapter
			//

			const std::wstring guid = GetNetCfgInstanceId(devInfo, devInfoData);
			GUID guidObj = common::Guid::FromString(guid);

			adapters.emplace(NetworkAdapter(
				guid,
				GetDeviceStringProperty(devInfo, devInfoData, &DEVPKEY_Device_DriverDesc),
				nci.getConnectionName(guidObj),
				GetDeviceInstanceId(devInfo, devInfoData)
			));
		}
		catch (const std::exception & e)
		{
			//
			// Skip this adapter
			//

			std::wstringstream ss;
			ss << L"Skipping adapter due to exception caught while iterating: "
				<< common::string::ToWide(e.what());
			LogError(ss.str());
		}
		return true;
	});

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

	Log(L"Created new TAP adapter successfully");
}

void UpdateTapDriver(const std::wstring &infPath)
{
	Log(L"Attempting to install new driver");

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
			Log(L"Driver update failed. Attempting forced install.");
			installFlags |= INSTALLFLAG_FORCE;

			goto ATTEMPT_UPDATE;
		}

		if (ERROR_DEVICE_INSTALLER_NOT_READY == lastError)
		{
			bool deviceInstallDisabled = false;

			try
			{
				const auto key = common::registry::Registry::OpenKey(
					HKEY_LOCAL_MACHINE,
					L"SYSTEM\\CurrentControlSet\\Services\\DeviceInstall\\Parameters"
				);
				deviceInstallDisabled = (0 != key->readUint32(L"DeviceInstallDisabled"));
			}
			catch (...)
			{
			}

			if (deviceInstallDisabled)
			{
				THROW_ERROR(
					"Device installs must be enabled to continue. "
					"Enable them in the Local Group Policy editor, or"
					" update the registry value DeviceInstallDisabled in"
					" [HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\DeviceInstall\\Parameters]"
				);
			}
		}

		THROW_WINDOWS_ERROR(lastError, "UpdateDriverForPlugAndPlayDevicesW");
	}

	//
	// Driver successfully installed or updated
	//

	std::wstringstream ss;
	ss << L"TAP driver update complete. Reboot required: "
		<< rebootRequired;
	Log(ss.str());
}

//
// Broken adapters may use our "Mullvad" name, so find one that is not in use.
// NOTE: Enumerating adapters first and picking the next free name is not sufficient,
//       because the broken TAP may not be included.
//
void RenameAdapterToMullvad(const NetworkAdapter &adapter)
{
	common::network::Nci nci;

	try
	{
		nci.setConnectionName(common::Guid::FromString(adapter.guid), TAP_BASE_ALIAS);
		return;
	}
	catch (...)
	{
	}

	for (int i = 1; i < 10; i++)
	{
		std::wstringstream ss;
		ss << TAP_BASE_ALIAS << L"-" << i;

		try
		{
			nci.setConnectionName(common::Guid::FromString(adapter.guid), ss.str().c_str());
			return;
		}
		catch (...)
		{
		}
	}

	THROW_ERROR("Exhausted TAP adapter namespace");
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

	for (auto i = 1; i < 10; ++i)
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
	std::set<NetworkAdapter> added = GetNetworkAdapters(TAP_HARDWARE_ID);

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

void RemoveTapDriver(const std::wstring &tapHardwareId)
{
	ForEachNetworkDevice(tapHardwareId, [](HDEVINFO devInfo, const SP_DEVINFO_DATA &devInfoData) {
		try
		{
			DeleteDevice(devInfo, devInfoData);
		}
		catch (const std::exception & e)
		{
			//
			// Skip this adapter
			//

			std::wstringstream ss;
			ss << L"Skipping TAP adapter due to exception caught while iterating: "
				<< common::string::ToWide(e.what());
			LogError(ss.str());
		}
		return true;
	});
}

void DeleteVanillaMullvadAdapter()
{
	auto tapAdapters = GetNetworkAdapters(DEPRECATED_TAP_HARDWARE_ID);
	std::optional<NetworkAdapter> mullvadAdapter = FindMullvadAdapter(tapAdapters);

	if (!mullvadAdapter.has_value())
	{
		return;
	}

	const auto mullvadGuid = mullvadAdapter.value().guid;
	bool deletedAdapter = false;

	//
	// Enumerate over all network devices with the hardware ID DEPRECATED_TAP_HARDWARE_ID,
	// and delete any adapter whose GUID matches that of the "Mullvad" adapter.
	//

	ForEachNetworkDevice(DEPRECATED_TAP_HARDWARE_ID, [&](HDEVINFO devInfo, const SP_DEVINFO_DATA &devInfoData) {
		try
		{
			if (0 == GetNetCfgInstanceId(devInfo, devInfoData).compare(mullvadGuid))
			{
				deletedAdapter = DeleteDevice(devInfo, devInfoData);
				return false;
			}
		}
		catch (const std::exception & e)
		{
			//
			// Skip this adapter
			//

			std::wstringstream ss;
			ss << L"Skipping TAP adapter due to exception caught while iterating: "
				<< common::string::ToWide(e.what());
			LogError(ss.str());
		}
		return true;
	});

	if (!deletedAdapter)
	{
		THROW_ERROR("The TAP adapter could not be removed");
	}
}

} // anonymous namespace

int wmain(int argc, const wchar_t * argv[], const wchar_t * [])
{
	if (-1 == _setmode(_fileno(stdout), _O_U16TEXT)
		|| -1 == _setmode(_fileno(stderr), _O_U16TEXT))
	{
		LogError(L"Failed to set translation mode");
	}

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
			RenameAdapterToMullvad(FindBrandedTap());
		}
		else if (0 == _wcsicmp(argv[1], L"update"))
		{
			if (3 != argc)
			{
				goto INVALID_ARGUMENTS;
			}

			UpdateTapDriver(argv[2]);
		}
		else if (0 == _wcsicmp(argv[1], L"remove"))
		{
			if (3 != argc)
			{
				goto INVALID_ARGUMENTS;
			}

			RemoveTapDriver(argv[2]);
		}
		else if (0 == _wcsicmp(argv[1], L"remove-vanilla-tap"))
		{
			DeleteVanillaMullvadAdapter();
		}
		else
		{
			goto INVALID_ARGUMENTS;
		}
	}
	catch (const std::exception &e)
	{
		LogError(common::string::ToWide(e.what()));
		return GENERAL_ERROR;
	}
	catch (...)
	{
		LogError(L"Unhandled exception.");
		return GENERAL_ERROR;
	}
	return GENERAL_SUCCESS;

INVALID_ARGUMENTS:

	LogError(L"Invalid arguments.");
	return GENERAL_ERROR;
}
