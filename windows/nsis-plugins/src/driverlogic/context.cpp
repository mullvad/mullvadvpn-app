#include "stdafx.h"
#include "context.h"
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
#include <iostream>
#include <setupapi.h>
#include <devguid.h>
#include <combaseapi.h>
#include <initguid.h>
#include <devpkey.h>
#include <libcommon/registry/registry.h>

namespace
{

const wchar_t TAP_HARDWARE_ID[] = L"tap0901";

std::set<Context::NetworkAdapter> GetAllAdapters()
{
	ULONG bufferSize = 0;

	const ULONG flags = GAA_FLAG_SKIP_UNICAST | GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER;

	auto status = GetAdaptersAddresses(AF_INET, flags, nullptr, nullptr, &bufferSize);

	THROW_UNLESS(ERROR_BUFFER_OVERFLOW, status, "Probe for adapter listing buffer size");

	// Memory is cheap, this avoids a looping construct.
	bufferSize *= 2;

	std::vector<uint8_t> buffer(bufferSize);

	status = GetAdaptersAddresses(AF_INET, flags, nullptr,
		reinterpret_cast<PIP_ADAPTER_ADDRESSES>(&buffer[0]), &bufferSize);

	THROW_UNLESS(ERROR_SUCCESS, status, "Retrieve adapter listing");

	std::set<Context::NetworkAdapter> adapters;

	for (auto it = (PIP_ADAPTER_ADDRESSES)&buffer[0]; nullptr != it; it = it->Next)
	{
		adapters.emplace(Context::NetworkAdapter(common::string::ToWide(it->AdapterName),
			it->Description, it->FriendlyName));
	}

	return adapters;
}

std::set<Context::NetworkAdapter> GetTapAdapters(const std::set<Context::NetworkAdapter> &adapters)
{
	std::set<Context::NetworkAdapter> tapAdapters;

	for (const auto &adapter : adapters)
	{
		static const wchar_t name[] = L"TAP-Windows Adapter V9";

		//
		// Compare partial name, because once you start having more TAP adapters
		// they're named "TAP-Windows Adapter V9 #2" and so on.
		//

		if (0 == adapter.name.compare(0, _countof(name) - 1, name))
		{
			tapAdapters.insert(adapter);
		}
	}

	return tapAdapters;
}

void LogAdapters(const std::wstring &description, const std::set<Context::NetworkAdapter> &adapters)
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
	std::vector<wchar_t> instanceId(MAX_PATH + sizeof(L'\0'));
	DWORD strSize = instanceId.size() * sizeof(wchar_t);
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

} // anonymous namespace

std::set<Context::NetworkAdapter> Context::getTapAdapters()
{
	return GetTapAdapters(m_currentState);
}

Context::BaselineStatus Context::establishBaseline()
{
	m_baseline = GetAllAdapters();

	auto tapAdapters = GetTapAdapters(m_baseline);

	if (tapAdapters.empty())
	{
		return BaselineStatus::NO_TAP_ADAPTERS_PRESENT;
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

		return it != adapters.end();
	};

	static const wchar_t baseAlias[] = L"Mullvad";

	if (findByAlias(tapAdapters, baseAlias))
	{
		return BaselineStatus::MULLVAD_ADAPTER_PRESENT;
	}

	//
	// Look for TAP adapter with alias "Mullvad-1", "Mullvad-2", etc.
	//

	for (auto i = 0; i < 10; ++i)
	{
		std::wstringstream ss;

		ss << baseAlias << L"-" << i;

		const auto alias = ss.str();

		if (findByAlias(tapAdapters, alias))
		{
			return BaselineStatus::MULLVAD_ADAPTER_PRESENT;
		}
	}

	return BaselineStatus::SOME_TAP_ADAPTERS_PRESENT;
}

void Context::recordCurrentState()
{
	m_currentState = GetAllAdapters();
}

Context::NetworkAdapter Context::getNewAdapter()
{
	std::list<NetworkAdapter> added;

	const auto baselineTaps = GetTapAdapters(m_baseline);
	const auto currentTaps = GetTapAdapters(m_currentState);

	for (const auto &adapter : currentTaps)
	{
		if (baselineTaps.end() == baselineTaps.find(adapter))
		{
			added.push_back(adapter);
		}
	}

	if (added.size() != 1)
	{
		LogAdapters(L"Enumerable network adapters", m_currentState);

		throw std::runtime_error("Unable to identify recently added TAP adapter");
	}

	return *added.begin();
}

//static
void Context::DeleteMullvadAdapter()
{
	const auto regkey = common::registry::Registry::OpenKey(
		HKEY_LOCAL_MACHINE,
		L"SOFTWARE\\Mullvad VPN",
		false,
		common::registry::RegistryView::Force64
	);
	const auto mullvadGuid = regkey->readString(L"TapAdapterGuid");

	HDEVINFO devInfo = SetupDiGetClassDevs(
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
	devInfoData.cbSize = sizeof(devInfoData);

	std::vector<wchar_t> buffer;
	DWORD nameLen;

	for (int memberIndex = 0; ; memberIndex++)
	{
		if (FALSE == SetupDiEnumDeviceInfo(devInfo, memberIndex, &devInfoData))
		{
			if (GetLastError() == ERROR_NO_MORE_ITEMS)
			{
				/* done */
				break;
			}
			else
			{
				THROW_GLE("Error enumerating network adapters");
			}
		}

		if (FALSE == SetupDiGetDeviceRegistryProperty(
			devInfo,
			&devInfoData,
			SPDRP_HARDWAREID,
			nullptr,
			nullptr,
			0,
			&nameLen
		))
		{
			THROW_GLE("Error obtaining network adapter hardware ID");
		}

		buffer.resize(nameLen);

		if (FALSE == SetupDiGetDeviceRegistryProperty(
			devInfo,
			&devInfoData,
			SPDRP_HARDWAREID,
			nullptr,
			reinterpret_cast<PBYTE>(buffer.data()),
			buffer.size(),
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
}
