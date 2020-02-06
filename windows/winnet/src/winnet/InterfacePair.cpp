#include "stdafx.h"
#include "InterfacePair.h"
#include <libcommon/error.h>
#include <sstream>

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

	if (!HasIPv4() && !HasIPv6())
	{
		std::stringstream ss;

		ss << "LUID 0x" << std::hex << interface_luid.Value
			<< " does not specify any IPv4 or IPv6 interfaces";

		THROW_ERROR(ss.str().c_str());
	}
}

int InterfacePair::WorstMetric()
{
	return IPv6Iface.Metric >= IPv4Iface.Metric ? IPv6Iface.Metric : IPv4Iface.Metric;
}

int InterfacePair::BestMetric()
{
	return IPv6Iface.Metric < IPv4Iface.Metric ? IPv4Iface.Metric : IPv6Iface.Metric;
}

void InterfacePair::SetMetric(unsigned int metric)
{
	if (HasIPv4() && (IPv4Iface.UseAutomaticMetric || metric != IPv4Iface.Metric))
    {
		IPv4Iface.SitePrefixLength = 0;
		IPv4Iface.Metric = metric;
		IPv4Iface.UseAutomaticMetric = false;
        SetInterface(&IPv4Iface);
	}

	if (HasIPv6() && (IPv6Iface.UseAutomaticMetric || metric != IPv6Iface.Metric))
    {
		IPv6Iface.Metric = metric;
		IPv6Iface.UseAutomaticMetric = false;
        SetInterface(&IPv6Iface);
	}
}

void InterfacePair::SetInterface(PMIB_IPINTERFACE_ROW iface) {

    const auto status = SetIpInterfaceEntry(iface);

    if (status != NO_ERROR) 
    {
        std::stringstream ss;

        ss << "Set metric for "
			<< (iface->Family == AF_INET ? "IPv4" : "IPv6")
            << " on interface with LUID 0x"
            << std::hex << iface->InterfaceLuid.Value;

        THROW_WINDOWS_ERROR(status, ss.str().c_str());
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

//static
void InterfacePair::InitializeInterface(PMIB_IPINTERFACE_ROW iface)
{
	const auto status = GetIpInterfaceEntry(iface);

	if (NO_ERROR == status)
	{
		return;
	}

	if (STATUS_NOT_FOUND == status || ERROR_NOT_FOUND == status)
	{
		iface->Family = AF_UNSPEC;
	}
	else
	{
		std::stringstream ss;

		ss << "Retrieve info on network interface with LUID 0x"
			<< std::hex << iface->InterfaceLuid.Value;

		THROW_WINDOWS_ERROR(status, ss.str().c_str());
	}
}
