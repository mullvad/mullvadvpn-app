#include "stdafx.h"
#include "dnsconfiguration.h"
#include "windns/comhelpers.h"

DnsConfiguration::DnsConfiguration(CComPtr<IWbemClassObject> instance)
{
	m_configId = dnshelpers::GetId(instance);
	m_interfaceIndex = ComGetPropertyAlways(instance, L"InterfaceIndex").uintVal;
	m_servers = dnshelpers::GetServers(instance);
}

std::vector<std::wstring> *DnsConfiguration::servers() const
{
	return m_servers.get();
}

void DnsConfiguration::update(CComPtr<IWbemClassObject> instance)
{
	m_servers = dnshelpers::GetServers(instance);
}
