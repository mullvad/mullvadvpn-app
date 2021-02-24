#include "stdafx.h"
#include "error.h"
#include <iostream>
#include <chrono>
#include <sstream>
#include <string>
#include <optional>
#include <set>
#include <filesystem>
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
#include <wintun.h>


namespace
{

constexpr std::chrono::milliseconds REGISTRY_GET_TIMEOUT_MS{ 10000 };

enum ReturnCodes
{
	GENERAL_SUCCESS = 0,
	GENERAL_ERROR = -1,
	ADAPTER_NOT_FOUND = -2
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
			THROW_SETUPAPI_ERROR(lastError, "SetupDiGetDeviceRegistryPropertyW");
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
		THROW_SETUPAPI_ERROR(GetLastError(), "Failed to read device property");
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
			THROW_SETUPAPI_ERROR(lastError, "SetupDiGetDevicePropertyW");
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
		THROW_SETUPAPI_ERROR(GetLastError(), "Failed to read device property");
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
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiGetDeviceInstanceIdW");
	}

	return deviceInstanceId.data();
}

bool TryGetRegistryValueTimeout(
	HKEY key,
	const wchar_t *subkey,
	const wchar_t *value,
	DWORD flags,
	DWORD *type,
	void *data,
	DWORD *dataSize
)
{
	HANDLE changeEvent = nullptr;

	common::memory::ScopeDestructor scopeDestructor;
	scopeDestructor += [changeEvent]() {
		if (nullptr != changeEvent)
		{
			CloseHandle(changeEvent);
		}
	};

	auto initialTime = std::chrono::steady_clock::now();

	for (;;)
	{
		const auto status = RegGetValueW(key, subkey, value, flags, type, data, dataSize);

		if (ERROR_SUCCESS == status)
		{
			// We're done
			return true;
		}

		if (ERROR_FILE_NOT_FOUND != status)
		{
			THROW_WINDOWS_ERROR(status, "RegGetValueW");
		}

		if (nullptr == changeEvent)
		{
			changeEvent = CreateEventW(nullptr, FALSE, FALSE, nullptr);

			if (nullptr == changeEvent)
			{
				THROW_WINDOWS_ERROR(GetLastError(), "CreateEventW");
			}
		}

		//
		// Wait for the registry value to be created
		//

		auto currentTime = std::chrono::steady_clock::now();
		auto elapsedTime = std::chrono::duration_cast<std::chrono::milliseconds>(currentTime - initialTime);
		auto timeDelta = (REGISTRY_GET_TIMEOUT_MS - elapsedTime).count();

		if (timeDelta <= 0)
		{
			return false;
		}

		const auto notifyResult = RegNotifyChangeKeyValue(
			key,
			subkey != nullptr, // Watch subkeys
			REG_NOTIFY_CHANGE_LAST_SET,
			changeEvent,
			TRUE
		);

		if (ERROR_SUCCESS != notifyResult)
		{
			THROW_WINDOWS_ERROR(notifyResult, "RegNotifyChangeKeyValue");
		}

		const auto waitResult = WaitForSingleObject(changeEvent, static_cast<DWORD>(timeDelta));
		if (WAIT_OBJECT_0 == waitResult)
		{
			// Try again
			continue;
		}

		if (WAIT_TIMEOUT != waitResult)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "WaitForSingleObject");
		}

		return false;
	}
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
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiOpenDevRegKey");
	}

	common::memory::ScopeDestructor scopeDestructor;
	scopeDestructor += [hNet]() {
		RegCloseKey(hNet);
	};

	std::vector<wchar_t> instanceId(MAX_PATH + 1);
	DWORD strSize = static_cast<DWORD>(instanceId.size() * sizeof(wchar_t));

	if (!TryGetRegistryValueTimeout(
		hNet,
		nullptr,
		L"NetCfgInstanceId",
		RRF_RT_REG_SZ,
		nullptr,
		instanceId.data(),
		&strSize
	))
	{
		THROW_ERROR("Timed out waiting for NetCfgInstanceId.");
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
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiSetClassInstallParamsW");
	}

	status = SetupDiCallClassInstaller(DIF_REMOVE, devInfo, data);
	if (FALSE == status)
	{
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiCallClassInstaller");
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
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiGetClassDevsW");
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

			THROW_SETUPAPI_ERROR(lastError, "Enumerating network adapters");
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
				ss << L"Skipping virtual adapter due to exception caught while iterating: "
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

void throwUpdateException(DWORD lastError, const char *operation)
{
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
			throw common::error::WindowsException(
				"Device installs must be enabled to continue. "
				"Enable them in the Local Group Policy editor, or "
				"update the registry value DeviceInstallDisabled in "
				"[HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\DeviceInstall\\Parameters]",
				lastError
			);
		}
	}

	THROW_SETUPAPI_ERROR(lastError, operation);
}

//
// Broken adapters may use our "Mullvad" name, so find one that is not in use.
// NOTE: Enumerating adapters first and picking the next free name is not sufficient,
//       because the broken adapter may not be included.
//
void RenameAdapter(const std::wstring &guid, const std::wstring &baseName)
{
	common::network::Nci nci;

	try
	{
		nci.setConnectionName(common::Guid::FromString(guid), baseName.c_str());
		return;
	}
	catch (...)
	{
	}

	for (int i = 1; i < 10; i++)
	{
		std::wstringstream ss;
		ss << baseName << L"-" << i;

		try
		{
			nci.setConnectionName(common::Guid::FromString(guid), ss.str().c_str());
			return;
		}
		catch (...)
		{
		}
	}

	THROW_ERROR("Unable to rename network adapter");
}

void CreateNetDevice(const std::wstring &hardwareId, const std::optional<std::wstring> alias, bool installDeviceDriver)
{
	GUID classGuid = GUID_DEVCLASS_NET;

	const auto deviceInfoSet = SetupDiCreateDeviceInfoList(&classGuid, 0);
	if (INVALID_HANDLE_VALUE == deviceInfoSet)
	{
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiCreateDeviceInfoList");
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
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiCreateDeviceInfoW");
	}

	status = SetupDiSetDeviceRegistryPropertyW(
		deviceInfoSet,
		&devInfoData,
		SPDRP_HARDWAREID,
		reinterpret_cast<const BYTE *>(hardwareId.c_str()),
		static_cast<DWORD>(sizeof(wchar_t) * hardwareId.size())
	);

	if (FALSE == status)
	{
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiSetDeviceRegistryPropertyW");
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
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiCallClassInstaller");
	}

	Log(L"Created new network adapter successfully");

	if (installDeviceDriver)
	{
		BOOL rebootRequired = FALSE;

		if (FALSE == DiInstallDevice(
			nullptr,
			deviceInfoSet,
			&devInfoData,
			nullptr,
			0,
			&rebootRequired
		))
		{
			throwUpdateException(GetLastError(), "DiInstallDevice");
		}

		std::wstringstream ss;
		ss << L"Installed driver on device. Reboot required: "
			<< rebootRequired;
		Log(ss.str());
	}

	if (alias.has_value())
	{
		RenameAdapter(
			GetNetCfgInstanceId(deviceInfoSet, devInfoData),
			alias.value()
		);
	}
}

std::wstring FindFreeAdapterAlias(const std::set<NetworkAdapter> &adapters, const std::wstring &baseName)
{
	if (adapters.empty())
	{
		return baseName;
	}

	auto findByAlias = [](const std::set<NetworkAdapter> &adapters, const std::wstring &alias)
	{
		const auto it = std::find_if(adapters.begin(), adapters.end(), [&alias](const NetworkAdapter &candidate)
		{
			return 0 == _wcsicmp(candidate.alias.c_str(), alias.c_str());
		});

		return it;
	};

	const auto foundAdapter = findByAlias(adapters, baseName);

	if (adapters.end() == foundAdapter)
	{
		return baseName;
	}

	for (auto i = 1; i < 100; ++i)
	{
		std::wstringstream ss;

		ss << baseName << L"-" << i;

		const auto alias = ss.str();
		const auto nextAdapter = findByAlias(adapters, alias);

		if (adapters.end() == nextAdapter)
		{
			return alias;
		}
	}

	THROW_ERROR("Cannot find an unused adapter alias")
}

std::optional<NetworkAdapter> FindAdapterByAlias(const std::set<NetworkAdapter> &tapAdapters, const std::wstring &baseName)
{
	if (tapAdapters.empty())
	{
		return std::nullopt;
	}

	//
	// Look for TAP adapter with aliases starting with baseName.
	//

	auto findByAlias = [](const std::set<NetworkAdapter> &adapters, const std::wstring &alias)
	{
		const auto it = std::find_if(adapters.begin(), adapters.end(), [&alias](const NetworkAdapter &candidate)
		{
			return 0 == _wcsicmp(candidate.alias.c_str(), alias.c_str());
		});

		return it;
	};

	const auto firstMullvadAdapter = findByAlias(tapAdapters, baseName);

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

		ss << baseName << L"-" << i;

		const auto alias = ss.str();

		const auto mullvadAdapter = findByAlias(tapAdapters, alias);

		if (tapAdapters.end() != mullvadAdapter)
		{
			return { *mullvadAdapter };
		}
	}

	return std::nullopt;
}

bool RemoveNetDevice(const std::optional<std::wstring> tapHardwareId, const std::wstring &guid)
{
	bool deletedAdapter = false;

	ForEachNetworkDevice(tapHardwareId, [&](HDEVINFO devInfo, const SP_DEVINFO_DATA &devInfoData) {
		try
		{
			if (0 == GetNetCfgInstanceId(devInfo, devInfoData).compare(guid))
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
			ss << L"Skipping virtual adapter due to exception caught while iterating: "
				<< common::string::ToWide(e.what());
			LogError(ss.str());
		}
		return true;
	});

	return deletedAdapter;
}

void RemoveNetAdapterByAlias(const std::wstring &hardwareId, const std::wstring &baseName)
{
	auto tapAdapters = GetNetworkAdapters(hardwareId);
	std::optional<NetworkAdapter> adapter = FindAdapterByAlias(tapAdapters, baseName);

	if (!adapter.has_value())
	{
		return;
	}

	const auto guid = adapter.value().guid;

	//
	// Enumerate over all network devices with the hardware ID,
	// and delete any adapter whose GUID matches that of the "Mullvad" adapter.
	//

	if (!RemoveNetDevice(std::make_optional(hardwareId), guid))
	{
		THROW_ERROR("The virtual adapter could not be removed");
	}
}

std::filesystem::path GetCurrentModulePath()
{
	std::vector<wchar_t> pathBuffer;

	SetLastError(ERROR_SUCCESS);

	size_t nextCapacity = MAX_PATH;
	DWORD writtenChars = 0;

	do
	{
		pathBuffer.resize(nextCapacity);
		writtenChars = GetModuleFileNameW(nullptr, &pathBuffer[0], static_cast<DWORD>(pathBuffer.size()));

		if (0 == writtenChars)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "GetModuleFileNameW");
		}

		nextCapacity = 2 * pathBuffer.size();
	} while (ERROR_INSUFFICIENT_BUFFER == GetLastError());

	pathBuffer.resize(writtenChars);

	return std::filesystem::path(pathBuffer.begin(), pathBuffer.end());
}

class WintunDll
{
public:

	WintunDll() : dllHandle(nullptr)
	{
		auto wintunPath = GetCurrentModulePath().replace_filename(L"wintun.dll");
		dllHandle = LoadLibraryExW(wintunPath.c_str(), nullptr, LOAD_WITH_ALTERED_SEARCH_PATH);

		if (nullptr == dllHandle)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "LoadLibraryExW");
		}

		try
		{
			createAdapter = getProcAddressOrThrow<WINTUN_CREATE_ADAPTER_FUNC>("WintunCreateAdapter");
			openAdapter = getProcAddressOrThrow<WINTUN_OPEN_ADAPTER_FUNC>("WintunOpenAdapter");
			freeAdapter = getProcAddressOrThrow<WINTUN_FREE_ADAPTER_FUNC>("WintunFreeAdapter");
			deletePoolDriver = getProcAddressOrThrow<WINTUN_DELETE_POOL_DRIVER_FUNC>("WintunDeletePoolDriver");
		}
		catch (...)
		{
			FreeLibrary(dllHandle);
			throw;
		}
	}

	~WintunDll()
	{
		if (nullptr != dllHandle)
		{
			FreeLibrary(dllHandle);
		}
	}

	WINTUN_CREATE_ADAPTER_FUNC createAdapter;
	WINTUN_OPEN_ADAPTER_FUNC openAdapter;
	WINTUN_FREE_ADAPTER_FUNC freeAdapter;
	WINTUN_DELETE_POOL_DRIVER_FUNC deletePoolDriver;

private:

	template<typename T>
	T getProcAddressOrThrow(const char *procName)
	{
		const T result = reinterpret_cast<T>(GetProcAddress(dllHandle, procName));
		if (nullptr == result)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "GetProcAddress");
		}
		return result;
	}

	HMODULE dllHandle;
};

int HandleWintunCommands(int argc, const wchar_t *argv[])
{
	WintunDll wintun;

	if (argc < 3)
	{
		goto INVALID_ARGUMENTS;
	}

	if (0 == _wcsicmp(argv[2], L"create-adapter"))
	{
		if (argc < 5)
		{
			goto INVALID_ARGUMENTS;
		}

		const wchar_t *pool = argv[3];
		const wchar_t *adapter = argv[4];

		GUID guidObject;
		const GUID *requestGuid = nullptr;
		if (argc >= 6)
		{
			guidObject = common::Guid::FromString(argv[5]);
			requestGuid = &guidObject;
		}

		const auto adapters = GetNetworkAdapters(std::nullopt);
		const auto freeAdapterName = FindFreeAdapterAlias(adapters, adapter);

		const auto handle = wintun.createAdapter(
			pool,
			freeAdapterName.c_str(),
			requestGuid,
			nullptr
		);

		if (nullptr == handle)
		{
			const auto status = GetLastError();
			if (ERROR_FILE_NOT_FOUND == status)
			{
				return ADAPTER_NOT_FOUND;
			}
			else
			{
				THROW_WINDOWS_ERROR(status, "wintun.createAdapter");
			}
		}
		wintun.freeAdapter(handle);
	}
	else if (0 == _wcsicmp(argv[2], L"delete-pool-driver"))
	{
		if (4 != argc)
		{
			goto INVALID_ARGUMENTS;
		}

		const wchar_t *pool = argv[3];

		wintun.deletePoolDriver(pool, nullptr);
	}
	else if (0 == _wcsicmp(argv[2], L"adapter-exists"))
	{
		if (5 != argc)
		{
			goto INVALID_ARGUMENTS;
		}

		const wchar_t *pool = argv[3];
		const wchar_t *adapter = argv[4];

		const auto handle = wintun.openAdapter(pool, adapter);

		if (nullptr == handle)
		{
			const auto status = GetLastError();
			if (ERROR_FILE_NOT_FOUND == status)
			{
				return ADAPTER_NOT_FOUND;
			}
			else
			{
				THROW_WINDOWS_ERROR(status, "wintun.openAdapter");
			}
		}
		wintun.freeAdapter(handle);
	}
	else
	{
		goto INVALID_ARGUMENTS;
	}

	return GENERAL_SUCCESS;

INVALID_ARGUMENTS:

	LogError(L"Invalid arguments.");
	return GENERAL_ERROR;
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
		if (0 == _wcsicmp(argv[1], L"new-device"))
		{
			if (4 != argc)
			{
				goto INVALID_ARGUMENTS;
			}

			const wchar_t *hardwareId = argv[2];
			const wchar_t *baseName = argv[3];

			CreateNetDevice(hardwareId, baseName, true);
		}
		else if (0 == _wcsicmp(argv[1], L"remove-device"))
		{
			if (4 != argc)
			{
				goto INVALID_ARGUMENTS;
			}

			const wchar_t *hardwareId = argv[2];
			const wchar_t *baseName = argv[3];

			RemoveNetAdapterByAlias(hardwareId, baseName);
		}
		else if (0 == _wcsicmp(argv[1], L"remove-device-by-guid"))
		{
			if (3 != argc)
			{
				goto INVALID_ARGUMENTS;
			}

			const wchar_t *guid = argv[2];

			if (!RemoveNetDevice(std::nullopt, guid))
			{
				return ADAPTER_NOT_FOUND;
			}
		}
		else if (0 == _wcsicmp(argv[1], L"device-exists"))
		{
			if (4 != argc)
			{
				goto INVALID_ARGUMENTS;
			}

			const wchar_t *hardwareId = argv[2];
			const wchar_t *baseName = argv[3];

			const auto virtualAdapters = GetNetworkAdapters(hardwareId);
			const auto adapter = FindAdapterByAlias(virtualAdapters, baseName);

			if (!adapter.has_value())
			{
				return ADAPTER_NOT_FOUND;
			}
		}
		else if (0 == _wcsicmp(argv[1], L"wintun"))
		{
			return HandleWintunCommands(argc, argv);
		}
		else
		{
			goto INVALID_ARGUMENTS;
		}
	}
	catch (const common::error::WindowsException &e)
	{
		LogError(common::string::ToWide(e.what()));
		return e.errorCode();
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
