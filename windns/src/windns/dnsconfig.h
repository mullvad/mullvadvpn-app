#pragma once

#include "windns/dnshelpers.h"
#include <cstdint>
#include <string>
#include <vector>
#include <memory>
#include <atlbase.h>
#include <wbemidl.h>

class DnsConfiguration
{
public:

	// instance = Win32_NetworkAdapterConfiguration.
	DnsConfiguration(CComPtr<IWbemClassObject> instance);

	DnsConfiguration(const DnsConfiguration &) = delete;
	DnsConfiguration &operator=(const DnsConfiguration &) = delete;
	DnsConfiguration(DnsConfiguration &&) = default;
	DnsConfiguration &operator=(DnsConfiguration &&) = default;

	const std::wstring &id() const
	{
		return m_configId;
	}

	uint32_t interfaceIndex() const
	{
		return m_interfaceIndex;
	}

	std::vector<std::wstring> *servers() const;

	void update(CComPtr<IWbemClassObject> instance);

private:

	std::wstring m_configId;
	uint32_t m_interfaceIndex;
	dnshelpers::OptionalStringList m_servers;
};
