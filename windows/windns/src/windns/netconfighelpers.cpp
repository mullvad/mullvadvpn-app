#include "stdafx.h"
#include "netconfighelpers.h"
#include "libcommon/com.h"
#include "libcommon/wmi/wmi.h"
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

}
