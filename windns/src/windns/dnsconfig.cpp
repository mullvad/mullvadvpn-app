#include "stdafx.h"
#include "dnsconfig.h"
#include "windns/comhelpers.h"

DnsConfig::DnsConfig(CComPtr<IWbemClassObject> instance)
{
	m_configId = nchelpers::GetConfigId(instance);
	m_interfaceIndex = ComGetPropertyAlways(instance, L"InterfaceIndex").uintVal;
	m_servers = nchelpers::GetDnsServers(instance);
}

const std::vector<std::wstring> *DnsConfig::servers() const
{
	return m_servers.get();
}

//void DnsConfig::update(CComPtr<IWbemClassObject> instance)
//{
//	m_servers = nchelpers::GetDnsServers(instance);
//}
