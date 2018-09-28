#include "stdafx.h"
#include "interfacesnap.h"
#include "registrypaths.h"
#include <libcommon/registry/registry.h>
#include <libcommon/string.h>
#include <cstdint>

using namespace common::registry;

namespace
{

enum class NameServerType
{
	Static,
	Dhcp
};

std::vector<std::wstring> GetNameServers(const std::wstring &interfaceGuid,
	Protocol protocol, NameServerType nameServerType)
{
	const auto interfacePath = RegistryPaths::InterfaceKey(interfaceGuid, protocol);

	const auto regKey = Registry::OpenKey(HKEY_LOCAL_MACHINE, interfacePath);

	std::wstring nameservers;

	try
	{
		//
		// This particular value is a string array packed into a string data type.
		// REG_MULTI_SZ would have been the correct type to use, but there
		// are probably historical reasons for the value type currently being used.
		//
		nameservers = regKey->readString(NameServerType::Static == nameServerType ? L"NameServer" : L"DhcpNameServer");
	}
	catch (...)
	{
	}

	if (nameservers.empty())
	{
		return std::vector<std::wstring>();
	}

	return common::string::Tokenize(nameservers, L",");
}

bool GetDhcpEnabled(const std::wstring &interfaceGuid, Protocol protocol)
{
	const auto interfacePath = RegistryPaths::InterfaceKey(interfaceGuid, protocol);

	const auto regKey = Registry::OpenKey(HKEY_LOCAL_MACHINE, interfacePath);

	bool enabled = false;

	try
	{
		const auto flag = regKey->readUint32(L"EnableDHCP");

		enabled = (1 == flag);
	}
	catch (...)
	{
	}

	return enabled;
}

} // anonymous namespace

InterfaceSnap::InterfaceSnap(Protocol protocol, const std::wstring &interfaceGuid)
	: m_protocol(protocol)
	, m_interfaceGuid(interfaceGuid)
{
	m_configuredForDhcp = GetDhcpEnabled(m_interfaceGuid, m_protocol);

	// Static name servers are configured by the user.
	m_staticNameServers = GetNameServers(m_interfaceGuid, m_protocol, NameServerType::Static);

	// DHCP name servers are the servers most recently supplied by DHCP.
	// An adapter can be configured for DHCP and static name servers at the same time.
	// Static name servers always have precedence.
	m_dhcpNameServers = GetNameServers(m_interfaceGuid, m_protocol, NameServerType::Dhcp);
}

InterfaceSnap::InterfaceSnap(common::serialization::Deserializer &deserializer)
{
	common::serialization::Deserializer &d = deserializer;

	d >> (uint8_t &)m_protocol;

	if (m_protocol != Protocol::IPv4
		&& m_protocol != Protocol::IPv6)
	{
		throw std::runtime_error("Serialized data for 'InterfaceSnap' instance is invalid (protocol)");
	}

	d >> m_interfaceGuid;
	d >> (uint8_t &)m_configuredForDhcp;
	d >> m_staticNameServers;
	d >> m_dhcpNameServers;
}

void InterfaceSnap::serialize(common::serialization::Serializer &serializer) const
{
	common::serialization::Serializer &s = serializer;

	s << (uint8_t &)m_protocol;
	s << m_interfaceGuid;
	s << (uint8_t &)m_configuredForDhcp;
	s << m_staticNameServers;
	s << m_dhcpNameServers;
}

bool InterfaceSnap::needsOverriding(const std::vector<std::wstring> &enforcedServers) const
{
	if (internalInterface())
	{
		return false;
	}

	//
	// The interface has static DNS, or
	// The interface has DNS provided by the DHCP server, or
	// The interface *will get* DNS provided to it by the DHCP server
	//

	//
	// It's not enough that m_staticNameServers has the same elements.
	// The order defines primary and secondary name server and has to match.
	//

	return m_staticNameServers != enforcedServers;
}

const std::wstring &InterfaceSnap::interfaceGuid() const
{
	return m_interfaceGuid;
}

const std::vector<std::wstring> &InterfaceSnap::nameServers() const
{
	return m_staticNameServers;
}

bool InterfaceSnap::internalInterface() const
{
	return false == m_configuredForDhcp
		&& m_staticNameServers.empty();
}
