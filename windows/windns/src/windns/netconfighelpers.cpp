#include "stdafx.h"
#include "netconfighelpers.h"
#include "libcommon/com.h"
#include "libcommon/wmi/wmi.h"
#include "libcommon/trace/xtrace.h"
#include "netsh.h"

using namespace common;

namespace nchelpers
{

OptionalStringList GetDnsServers(CComPtr<IWbemClassObject> instance)
{
	OptionalStringList result;

	auto servers = wmi::WmiGetProperty(instance, L"DNSServerSearchOrder");

	if (VT_EMPTY == V_VT(&servers) || VT_NULL == V_VT(&servers))
	{
		return result;
	}

	result = std::make_shared<std::vector<std::wstring> >(
		ComConvertStringArray(V_ARRAY(&servers)));

	return result;
}

uint32_t GetInterfaceIndex(CComPtr<IWbemClassObject> instance)
{
	return wmi::WmiGetPropertyAlways(instance, L"InterfaceIndex").ulVal;
}

void SetDnsServers(uint32_t interfaceIndex, const std::vector<std::wstring> &servers)
{
	NetSh::SetIpv4PrimaryDns(interfaceIndex, servers[0]);

	if (servers.size() > 1)
	{
		NetSh::SetIpv4SecondaryDns(interfaceIndex, servers[1]);
	}
}

void RevertDnsServers(const InterfaceConfig &config, uint32_t timeout)
{
	XTRACE("Reverting DNS configuration for interface with index=", config.interfaceIndex());

	auto servers = config.servers();

	if (config.dhcp() || nullptr == servers || 0 == servers->size())
	{
		NetSh::SetIpv4Dhcp(config.interfaceIndex(), timeout);
		return;
	}

	NetSh::SetIpv4PrimaryDns(config.interfaceIndex(), (*servers)[0], timeout);

	if (servers->size() > 1)
	{
		NetSh::SetIpv4SecondaryDns(config.interfaceIndex(), (*servers)[1], timeout);
	}
}

}
