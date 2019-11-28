#include "stdafx.h"
#include "context.h"
#include "ncicontext.h"

#include <libcommon/string.h>
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <log/log.h>

#include <winsock2.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <windows.h>

#include <vector>
#include <list>
#include <stdexcept>
#include <sstream>
#include <algorithm>

#include <setupapi.h>
#include <devguid.h>
#include <combaseapi.h>
#include <initguid.h>
#include <devpkey.h>

namespace
{

const wchar_t TAP_HARDWARE_ID[] = L"tap0901";

template<typename T>
void LogAdapters(const std::wstring &description, const T &adapters)
{
	//
	// Flatten the information so we can log it more easily.
	//

	std::vector<std::wstring> details;

	for (const auto &adapter : adapters)
	{
		details.emplace_back(L"Adapter");

		details.emplace_back(std::wstring(L"    Guid: ").append(adapter.guid));
		details.emplace_back(std::wstring(L"    Name: ").append(adapter.name));
		details.emplace_back(std::wstring(L"    Alias: ").append(adapter.alias));
	}

	PluginLogWithDetails(description, details);
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
		throw std::runtime_error("SetupDiOpenDevRegKey Failed");
	}

	std::vector<wchar_t> instanceId(MAX_PATH + sizeof(L'\0'));
	DWORD strSize = instanceId.size() * sizeof(wchar_t);

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
		throw std::runtime_error("RegGetValue for NetCfgInstanceId failed");
	}

	instanceId[strSize / sizeof(wchar_t)] = L'\0';

	return instanceId.data();
}

std::set<Context::NetworkAdapter> GetTapAdapters()
{
	std::set<Context::NetworkAdapter> adapters;

	HDEVINFO devInfo = SetupDiGetClassDevs(
		&GUID_DEVCLASS_NET,
		nullptr,
		nullptr,
		DIGCF_PRESENT
	);

	if (INVALID_HANDLE_VALUE == devInfo)
	{
		throw std::runtime_error("SetupDiGetClassDevs() failed");
	}

	common::memory::ScopeDestructor scopeDest;
	scopeDest += [devInfo]()
	{
		SetupDiDestroyDeviceInfoList(devInfo);
	};

	SP_DEVINFO_DATA devInfoData;
	devInfoData.cbSize = sizeof(devInfoData);

	std::wstring hardwareId;
	DWORD hardwareIdReqSize;
	DWORD type;

	NciContext nci;

	for (int memberIndex = 0; ; memberIndex++)
	{
		devInfoData = { 0 };
		devInfoData.cbSize = sizeof(devInfoData);

		if (FALSE == SetupDiEnumDeviceInfo(devInfo, memberIndex, &devInfoData))
		{
			if (GetLastError() == ERROR_NO_MORE_ITEMS)
			{
				/* done */
				break;
			}
			THROW_GLE("SetupDiEnumDeviceInfo() failed while enumerating network adapters");
		}

		//
		// Check whether this is a TAP adapter
		//

READ_HARDWARE_ID:

		const auto status = SetupDiGetDeviceRegistryPropertyW(
			devInfo,
			&devInfoData,
			SPDRP_HARDWAREID,
			&type,
			reinterpret_cast<PBYTE>(&hardwareId[0]),
			hardwareId.capacity() * sizeof(wchar_t),
			&hardwareIdReqSize
		);

		if (ERROR_INVALID_DATA == status)
		{
			// property does not exist
			continue;
		}

		if (FALSE == status)
		{
			if (ERROR_INSUFFICIENT_BUFFER == GetLastError())
			{
				hardwareId.resize(hardwareIdReqSize / sizeof(wchar_t));
				goto READ_HARDWARE_ID;
			}
			else
			{
				throw std::runtime_error("Failed to read adapter hardware ID");
			}
		}

		hardwareId.resize(hardwareIdReqSize / sizeof(wchar_t));

		if (wcscmp(hardwareId.c_str(), TAP_HARDWARE_ID) != 0)
		{
			continue;
		}

		//
		// Obtain GUID
		//

		std::wstring guid = GetNetCfgInstanceId(devInfo, devInfoData);

		//
		// Obtain device instance ID
		//

		DWORD requiredSize = 0;

		SetupDiGetDeviceInstanceIdW(
			devInfo,
			&devInfoData,
			nullptr,
			0,
			&requiredSize
		);

		std::wstring deviceInstanceId;
		deviceInstanceId.resize(requiredSize);

		const auto deviceInstIdStatus = SetupDiGetDeviceInstanceIdW(
			devInfo,
			&devInfoData,
			&deviceInstanceId[0],
			deviceInstanceId.size(),
			nullptr
		);
		THROW_GLE_IF(FALSE, deviceInstIdStatus, "SetupDiGetDeviceInstanceIdW() failed");

		//
		// Obtain alias
		//

		IID guidObj = { 0 };
		if (S_OK != IIDFromString(&guid[0], &guidObj))
		{
			throw std::runtime_error("IIDFromString() failed");
		}

		std::wstring alias = nci.getConnectionName(guidObj);

		//
		// Obtain description
		//

		DEVPROPTYPE descType;
		DWORD descSize;

		SetupDiGetDevicePropertyW(
			devInfo,
			&devInfoData,
			&DEVPKEY_Device_DriverDesc,
			&descType,
			nullptr,
			0,
			&descSize,
			0
		);

		std::wstring name;
		name.resize(descSize / sizeof(wchar_t));

		const auto descStatus = SetupDiGetDevicePropertyW(
			devInfo,
			&devInfoData,
			&DEVPKEY_Device_DriverDesc,
			&descType,
			reinterpret_cast<PBYTE>(&name[0]),
			name.size() * sizeof(wchar_t),
			nullptr,
			0
		);
		THROW_GLE_IF(FALSE, descStatus, "SetupDiGetDevicePropertyW() failed");

		adapters.emplace(Context::NetworkAdapter(
			guid,
			name,
			alias,
			deviceInstanceId
		));
	}

	return adapters;
}

} // anonymous namespace

//static
std::optional<Context::NetworkAdapter> Context::FindMullvadAdapter(const std::set<Context::NetworkAdapter> &tapAdapters)
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

	static const wchar_t baseAlias[] = L"Mullvad";

	const auto mullvadAdapter = findByAlias(tapAdapters, baseAlias);

	if (tapAdapters.end() != mullvadAdapter)
	{
		return { *mullvadAdapter };
	}

	//
	// Look for TAP adapter with alias "Mullvad-1", "Mullvad-2", etc.
	//

	for (auto i = 0; i < 10; ++i)
	{
		std::wstringstream ss;

		ss << baseAlias << L"-" << i;

		const auto alias = ss.str();

		const auto mullvadAdapter = findByAlias(tapAdapters, alias);

		if (tapAdapters.end() != mullvadAdapter)
		{
			return { *mullvadAdapter };
		}
	}

	return std::nullopt;
}

Context::BaselineStatus Context::establishBaseline()
{
	m_baseline = GetTapAdapters();

	if (m_baseline.empty())
	{
		return BaselineStatus::NO_TAP_ADAPTERS_PRESENT;
	}
	
	if (FindMullvadAdapter(m_baseline).has_value())
	{
		return BaselineStatus::MULLVAD_ADAPTER_PRESENT;
	}

	return BaselineStatus::SOME_TAP_ADAPTERS_PRESENT;
}

void Context::recordCurrentState()
{
	m_currentState = GetTapAdapters();
}

void Context::rollbackTapAliases()
{
	NciContext nci;

	for (const auto &adapter : m_currentState)
	{
		const auto oldInfo = m_baseline.find(adapter);
		if (m_baseline.end() != oldInfo)
		{
			IID guidObj = { 0 };
			if (S_OK != IIDFromString(&adapter.guid[0], &guidObj))
			{
				throw std::runtime_error("IIDFromString() failed");
			}

			nci.setConnectionName(guidObj, oldInfo->alias.c_str());
		}
	}
}

Context::NetworkAdapter Context::getNewAdapter()
{
	std::list<NetworkAdapter> added;

	for (const auto &adapter : m_currentState)
	{
		if (m_baseline.end() == m_baseline.find(adapter))
		{
			added.push_back(adapter);
		}
	}

	if (added.size() != 1)
	{
		LogAdapters(L"Enumerable network TAP adapters", m_currentState);
		LogAdapters(L"New TAP adapters:", added);

		throw std::runtime_error("Unable to identify recently added TAP adapter");
	}

	return *added.begin();
}

//static
Context::DeletionResult Context::DeleteMullvadAdapter()
{
	auto tapAdapters = GetTapAdapters();
	std::optional<NetworkAdapter> mullvadAdapter = FindMullvadAdapter(tapAdapters);

	if (!mullvadAdapter.has_value())
	{
		throw std::runtime_error("Mullvad TAP adapter not found");
	}

	const auto mullvadGuid = mullvadAdapter.value().guid;

	HDEVINFO devInfo = SetupDiGetClassDevsW(
		&GUID_DEVCLASS_NET,
		nullptr,
		nullptr,
		DIGCF_PRESENT
	);

	THROW_GLE_IF(INVALID_HANDLE_VALUE, devInfo, "SetupDiGetClassDevs() failed");

	common::memory::ScopeDestructor cleanupDevList;
	cleanupDevList += [&devInfo]()
	{
		SetupDiDestroyDeviceInfoList(devInfo);
	};

	SP_DEVINFO_DATA devInfoData;

	std::vector<wchar_t> buffer;
	DWORD nameLen;

	int numRemainingAdapters = 0;

	for (int memberIndex = 0; ; memberIndex++)
	{
		devInfoData = { 0 };
		devInfoData.cbSize = sizeof(devInfoData);
		
		if (FALSE == SetupDiEnumDeviceInfo(devInfo, memberIndex, &devInfoData))
		{
			if (GetLastError() == ERROR_NO_MORE_ITEMS)
			{
				/* done */
				break;
			}
			THROW_GLE("Error enumerating network adapters");
		}

		if (FALSE == SetupDiGetDeviceRegistryPropertyW(
			devInfo,
			&devInfoData,
			SPDRP_HARDWAREID,
			nullptr,
			nullptr,
			0,
			&nameLen
		))
		{
			const auto status = GetLastError();
			if (ERROR_INSUFFICIENT_BUFFER != status)
			{
				/* ERROR_INSUFFICIENT_BUFFER is expected */
				if (ERROR_INVALID_DATA == status)
				{
					/* ERROR_INVALID_DATA may mean that the property does not exist */
					continue;
				}
				THROW_GLE("Error obtaining network adapter hardware ID length");
			}
		}

		buffer.resize(nameLen / sizeof(wchar_t) + 1);
		buffer[nameLen / sizeof(wchar_t)] = L'\0';

		if (FALSE == SetupDiGetDeviceRegistryPropertyW(
			devInfo,
			&devInfoData,
			SPDRP_HARDWAREID,
			nullptr,
			reinterpret_cast<PBYTE>(buffer.data()),
			(buffer.size() - 1) * sizeof(wchar_t),
			nullptr
		))
		{
			THROW_GLE("Error obtaining network adapter hardware ID");
		}

		if (wcscmp(TAP_HARDWARE_ID, buffer.data()) == 0)
		{
			std::wstring netCfgInstanceId = GetNetCfgInstanceId(devInfo, devInfoData);
			if (netCfgInstanceId.compare(mullvadGuid) != 0)
			{
				numRemainingAdapters++;
				continue;
			}

			if (FALSE == SetupDiRemoveDevice(
				devInfo,
				&devInfoData
			))
			{
				THROW_GLE("Error removing Mullvad TAP device");
			}
		}
	}

	if (numRemainingAdapters > 0)
	{
		return DeletionResult::SOME_REMAINING_TAP_ADAPTERS;
	}
	
	return DeletionResult::NO_REMAINING_TAP_ADAPTERS;
}
