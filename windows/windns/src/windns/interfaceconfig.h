#pragma once

#include "windns/netconfighelpers.h"
#include "libcommon/serialization/deserializer.h"
#include "libcommon/serialization/serializer.h"
#include <cstdint>
#include <string>
#include <vector>
#include <memory>
#include <atlbase.h>
#include <wbemidl.h>

class InterfaceConfig
{
public:

	// instance = Win32_NetworkAdapterConfiguration.
	explicit InterfaceConfig(CComPtr<IWbemClassObject> instance);

	explicit InterfaceConfig(common::serialization::Deserializer &deserializer);
	void serialize(common::serialization::Serializer &serializer) const;

	void updateServers(const InterfaceConfig &rhs)
	{
		m_servers = rhs.m_servers;
	}

	uint32_t configIndex() const
	{
		return m_configIndex;
	}

	bool dhcp() const
	{
		return m_dhcp;
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

	bool m_dhcp;

	uint32_t m_interfaceIndex;
	std::wstring m_interfaceGuid;

	nchelpers::OptionalStringList m_servers;
};
