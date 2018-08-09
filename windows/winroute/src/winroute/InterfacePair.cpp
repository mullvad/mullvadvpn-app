#include "stdafx.h"
#include "InterfacePair.h"

#include <sstream>
#include <stdexcept>

#ifndef STATUS_NOT_FOUND
#define STATUS_NOT_FOUND 0xC0000225
#endif

InterfacePair::InterfacePair(NET_LUID interface_luid)
{
	IPv4Iface.Family = AF_INET;
	IPv4Iface.InterfaceLuid = interface_luid;
	InitializeInterface(&IPv4Iface);

	IPv6Iface.Family = AF_INET6;
	IPv6Iface.InterfaceLuid = interface_luid;
	InitializeInterface(&IPv6Iface);

	if (!(HasIPv4() || HasIPv6())) {
		std::stringstream ss;
		ss << "LUID "
			<< interface_luid.Value
			<< " does not specify any IPv4 or IPv6 interfaces";
		throw std::runtime_error(ss.str());
	}
}

int InterfacePair::HighestMetric()
{
	return IPv6Iface.Metric < IPv4Iface.Metric ? IPv4Iface.Metric : IPv6Iface.Metric;
}

void InterfacePair::SetMetric(int metric)
{
	if (HasIPv4())
    {
		IPv4Iface.SitePrefixLength = 0;
		IPv4Iface.Metric = metric;
		IPv4Iface.UseAutomaticMetric = false;
        SetInterface(&IPv4Iface);
	}

	if (HasIPv6())
    {
		IPv6Iface.Metric = metric;
		IPv6Iface.UseAutomaticMetric = false;
        SetInterface(&IPv6Iface);
	}
}

void InterfacePair::SetInterface(PMIB_IPINTERFACE_ROW iface) {

    DWORD status = SetIpInterfaceEntry(iface);
    if (status != NO_ERROR) 
    {
        std::stringstream ss;
        ss << "Failed to set metric for "
			<< (iface->Family == AF_INET ? "IPv4" : "IPv6")
            << " interface with LUID"
            << iface->InterfaceLuid.Value
            << " with error code "
            << status;
        throw std::runtime_error(ss.str());
    }
}

bool InterfacePair::HasIPv4()
{
	return IPv4Iface.Family != AF_UNSPEC;
}

bool InterfacePair::HasIPv6()
{
	return IPv6Iface.Family != AF_UNSPEC;
}

void InterfacePair::InitializeInterface(PMIB_IPINTERFACE_ROW iface)
{
	DWORD status = GetIpInterfaceEntry(iface);

	if (status != NO_ERROR) {
		if (status == STATUS_NOT_FOUND || status == ERROR_NOT_FOUND) {
			iface->Family = AF_UNSPEC;
		}
		else {
			std::stringstream ss;
			ss << "Failed to get network interface with LUID "
				<< &iface->InterfaceLuid.Value
				<< ": "
				<< status;
			throw std::runtime_error(ss.str());
		}
	}
}
