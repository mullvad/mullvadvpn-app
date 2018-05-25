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
	DnsConfig(CComPtr<IWbemClassObject> instance);

	DnsConfig(const DnsConfig &) = delete;
	DnsConfig &operator=(const DnsConfig &) = delete;
	DnsConfig(DnsConfig &&) = default;
	DnsConfig &operator=(DnsConfig &&) = default;

	const std::wstring &id() const
	{
		return m_configId;
	}

	uint32_t interfaceIndex() const
	{
		return m_interfaceIndex;
	}

	const std::vector<std::wstring> *servers() const;

//	void update(CComPtr<IWbemClassObject> instance);

private:

	std::wstring m_configId;
	uint32_t m_interfaceIndex;
	nchelpers::OptionalStringList m_servers;
};
