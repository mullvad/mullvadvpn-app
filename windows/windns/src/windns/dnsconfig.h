#pragma once

#include "windns/netconfighelpers.h"
#include <cstdint>
#include <string>
#include <vector>
#include <memory>
#include <atlbase.h>
#include <wbemidl.h>

class DnsConfig
{
public:

	// instance = Win32_NetworkAdapterConfiguration.
	explicit DnsConfig(CComPtr<IWbemClassObject> instance);

	uint32_t configIndex() const
	{
		return m_configIndex;
	}

	uint32_t interfaceIndex() const
	{
		return m_interfaceIndex;
	}

	const std::wstring &interfaceGuid() const
	{
		return m_interfaceGuid;
	}

	const std::vector<std::wstring> *servers() const
	{
		return m_servers.get();
	}

private:

	uint32_t m_configIndex;

	uint32_t m_interfaceIndex;
	std::wstring m_interfaceGuid;

	nchelpers::OptionalStringList m_servers;
};
