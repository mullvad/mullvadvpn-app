#include "stdafx.h"
#include "interfaceconfig.h"
#include "netconfighelpers.h"
#include "libcommon/com.h"
#include "libcommon/wmi/wmi.h"

using namespace common;

InterfaceConfig::InterfaceConfig(CComPtr<IWbemClassObject> instance)
{
	//
	// V_xxx macros seem to require an l-value so access the correct field directly instead.
	//

	m_configIndex = wmi::WmiGetPropertyAlways(instance, L"Index").ulVal;

	m_dhcp = wmi::WmiGetPropertyAlways(instance, L"DHCPEnabled").boolVal;

	m_interfaceIndex = wmi::WmiGetPropertyAlways(instance, L"InterfaceIndex").ulVal;
	m_interfaceGuid = ComConvertString(wmi::WmiGetPropertyAlways(instance, L"SettingID").bstrVal);

	m_servers = nchelpers::GetDnsServers(instance);
}

InterfaceConfig::InterfaceConfig(common::serialization::Deserializer &deserializer)
{
	common::serialization::Deserializer &d = deserializer;

	d >> m_configIndex;
	d >> (uint8_t &)m_dhcp;
	d >> m_interfaceIndex;
	d >> m_interfaceGuid;

	bool serversAvailable;

	d >> (uint8_t &)serversAvailable;

	if (serversAvailable)
	{
		m_servers = std::make_shared<std::vector<std::wstring> >();
		d >> *m_servers;
	}
}

void InterfaceConfig::serialize(common::serialization::Serializer &serializer) const
{
	common::serialization::Serializer &s = serializer;

	s << m_configIndex;
	s << (uint8_t)m_dhcp;
	s << m_interfaceIndex;
	s << m_interfaceGuid;

	//
	// TODO: Encapsulate this inside a new type.
	//
	if (nullptr == m_servers.get())
	{
		s << (uint8_t)0;
	}
	else
	{
		s << (uint8_t)1;
		s << *m_servers;
	}
}
