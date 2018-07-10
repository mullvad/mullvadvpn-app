#include "stdafx.h"
#include "context.h"
#include <libcommon/string.h>
#include <libcommon/wmi/connection.h>
#include <libcommon/wmi/resultset.h>
#include <libcommon/wmi/wmi.h>
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

} // anonymous namespace

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

//static
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
		nic.alias = GetNicAlias(nic.name);

		nics.emplace(std::move(nic));
		line += 4;
	}

	return nics;
}

//static
std::wstring Context::GetNicAlias(const std::wstring &name)
{
	static wmi::Connection connection(wmi::Connection::Namespace::Cimv2);

	std::wstringstream ss;

	ss << L"SELECT * from Win32_NetworkAdapter WHERE Name = \"" << name << L"\"";

	auto resultset = connection.query(ss.str().c_str());

	if (false == resultset.advance())
	{
		throw std::runtime_error("Unable to look up virtual adapter using WMI");
	}

	auto alias = wmi::WmiGetPropertyAlways(resultset.result(), L"NetConnectionID");

	return V_BSTR(&alias);
}
