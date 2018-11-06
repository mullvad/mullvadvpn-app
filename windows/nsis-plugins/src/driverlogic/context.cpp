#include "stdafx.h"
#include "context.h"
#include <libcommon/string.h>
#include <libcommon/wmi/connection.h>
#include <libcommon/wmi/resultset.h>
#include <libcommon/wmi/wmi.h>
#include <log/log.h>
#include <vector>
#include <stdexcept>
#include <algorithm>
#include <memory>
#include <sstream>

using namespace common;

namespace
{

std::vector<std::wstring> BlockToRows(const std::wstring &textBlock)
{
	//
	// This is such a hack :-(
	//
	// It only works because the tokenizer is greedy and because we don't care about
	// empty lines for this usage.
	//
	return common::string::Tokenize(textBlock, L"\r\n");
}

void LogAllAdapters(wmi::Connection &connection)
{
	auto resultset = connection.query(L"SELECT * from Win32_NetworkAdapter");

	struct NetworkAdapter
	{
		size_t interfaceIndex;
		std::wstring manufacturer;
		std::wstring name;
		std::wstring pnpDeviceId;
		std::wstring alias;
	};

	std::vector<NetworkAdapter> adapters;

	//
	// Find all adapters and extract the most important data.
	//

	auto StringOrNa = [](const _variant_t &variant)
	{
		if (VT_BSTR == V_VT(&variant))
		{
			return std::wstring(V_BSTR(&variant));
		}

		return std::wstring(L"n/a");
	};

	while(resultset.advance())
	{
		auto interfaceIndex = wmi::WmiGetPropertyAlways(resultset.result(), L"InterfaceIndex");
		auto manufacturer = wmi::WmiGetProperty(resultset.result(), L"Manufacturer");
		auto name = wmi::WmiGetProperty(resultset.result(), L"Name");
		auto pnpDeviceId = wmi::WmiGetProperty(resultset.result(), L"PNPDeviceID");
		auto alias = wmi::WmiGetProperty(resultset.result(), L"NetConnectionID");

		NetworkAdapter adapter;

		adapter.interfaceIndex = static_cast<size_t>(V_UI8(&interfaceIndex));
		adapter.manufacturer = StringOrNa(manufacturer);
		adapter.name = StringOrNa(name);
		adapter.pnpDeviceId = StringOrNa(pnpDeviceId);
		adapter.alias = StringOrNa(alias);

		adapters.emplace_back(adapter);
	}

	//
	// Flatten the adapter information so we can log it more easily.
	//

	std::vector<std::wstring> details;

	for (const auto &adapter : adapters)
	{
		details.emplace_back(L"Adapter");

		{
			std::wstringstream ss;

			ss << L"    InterfaceIndex: " << adapter.interfaceIndex;

			details.emplace_back(ss.str());
		}

		details.emplace_back(std::wstring(L"    Manufacturer: ").append(adapter.manufacturer));
		details.emplace_back(std::wstring(L"    Name: ").append(adapter.name));
		details.emplace_back(std::wstring(L"    PnpDeviceId: ").append(adapter.pnpDeviceId));
		details.emplace_back(std::wstring(L"    Alias: ").append(adapter.alias));
	}

	PluginLogWithDetails(L"Adapters known to WMI", details);
}

std::wstring DoubleBackslashes(const std::wstring &str)
{
	auto result(str);

	size_t offset = 0;

	for (size_t index = 0; index < str.size(); ++index)
	{
		if (L'\\' == str[index])
		{
			result.insert(index + offset, 1, L'\\');
			++offset;
		}
	}

	return result;
}

} // anonymous namespace

Context::Context()
	: m_connection(wmi::Connection::Namespace::Cimv2)
{
}

Context::BaselineStatus Context::establishBaseline(const std::wstring &textBlock)
{
	m_baseline = ParseVirtualNics(textBlock);

	if (m_baseline.empty())
	{
		return BaselineStatus::NO_INTERFACES_PRESENT;
	}

	for (const auto &nic : m_baseline)
	{
		if (0 == _wcsicmp(nic.alias.c_str(), L"mullvad"))
		{
			return BaselineStatus::MULLVAD_INTERFACE_PRESENT;
		}
	}

	return BaselineStatus::SOME_INTERFACES_PRESENT;
}

void Context::recordCurrentState(const std::wstring &textBlock)
{
	m_currentState = ParseVirtualNics(textBlock);
}

Context::VirtualNic Context::getNewAdapter()
{
	std::vector<VirtualNic> added;

	for (const auto &nic : m_currentState)
	{
		if (m_baseline.end() == m_baseline.find(nic))
		{
			added.push_back(nic);
		}
	}

	if (added.size() != 1)
	{
		throw std::runtime_error("Unable to identify newly added virtual adapter");
	}

	return added[0];
}

std::set<Context::VirtualNic> Context::ParseVirtualNics(const std::wstring &textBlock)
{
	// ROOT\NET\0000
	//     Name: TAP - Windows Adapter V9
	//     Hardware IDs :
	//         tap0901
	// 1 matching device(s) found.

	std::set<VirtualNic> nics;

	auto text = BlockToRows(textBlock);

	size_t line = 0;

	while (nullptr != wcschr(text.at(line).c_str(), L'\\'))
	{
		auto nameDelimiter = wcschr(text.at(line + 1).c_str(), L':');

		if (nullptr == nameDelimiter)
		{
			throw std::runtime_error("Unexpected formatting in input data");
		}

		VirtualNic nic;

		nic.node = text.at(line);
		nic.name = std::wstring(nameDelimiter + 2);
		nic.alias = GetNicAlias(nic.node, nic.name);

		nics.emplace(std::move(nic));
		line += 4;
	}

	return nics;
}

std::wstring Context::GetNicAlias(const std::wstring &node, const std::wstring &name)
{
	//
	// The name cannot be used when querying WMI, because WMI sometimes normalizes the
	// names in its dataset, thereby destroying their uniqueness.
	//
	// E.g. if a network interface has a name of "TAP-Windows Adapter V9 #2" it will
	// sometimes be reported by WMI as "TAP-Windows Adapter V9".
	//
	// Also, the node string cannot be used as-is. We have to double the backslashes in it
	// or the string will be rejected by WMI.
	//

	const auto formattedNode = DoubleBackslashes(node);

	std::wstringstream ss;

	ss << L"SELECT * FROM Win32_NetworkAdapter WHERE PNPDeviceID = \"" << formattedNode << L"\"";

	auto resultset = m_connection.query(ss.str().c_str());

	if (false == resultset.advance())
	{
		PluginLog(std::wstring(L"WMI query failed for adapter: ").append(name));
		LogAllAdapters(m_connection);

		throw std::runtime_error("Unable to look up virtual adapter using WMI");
	}

	auto alias = wmi::WmiGetPropertyAlways(resultset.result(), L"NetConnectionID");

	return V_BSTR(&alias);
}
