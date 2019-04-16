#include "stdafx.h"
#include "context.h"
#include <libcommon/string.h>
#include <libcommon/error.h>
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

namespace
{

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

} // anonymous namespace

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
