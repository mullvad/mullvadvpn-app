#include "stdafx.h"
#include "InterfacePair.h"
#include <libcommon/error.h>
#include <sstream>
#include <algorithm>

#ifndef STATUS_NOT_FOUND
#define STATUS_NOT_FOUND 0xC0000225
#endif

InterfacePair::InterfacePair(NET_LUID interface_luid)
{
	IPv4Iface.Family = AF_INET;
	IPv4Iface.InterfaceLuid = interface_luid;
	InitializeInterface(IPv4Iface);

	IPv6Iface.Family = AF_INET6;
	IPv6Iface.InterfaceLuid = interface_luid;
	InitializeInterface(IPv6Iface);

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
	return std::max(IPv6Iface.Metric, IPv4Iface.Metric);
}

int InterfacePair::BestMetric()
{
	return std::min(IPv4Iface.Metric, IPv6Iface.Metric);
}

void InterfacePair::SetMetric(uint32_t metric)
{
	if (HasIPv4() && (IPv4Iface.UseAutomaticMetric || metric != IPv4Iface.Metric))
	{
		IPv4Iface.SitePrefixLength = 0;
		IPv4Iface.Metric = metric;
		IPv4Iface.UseAutomaticMetric = false;
		SetInterface(IPv4Iface);
	}

	if (HasIPv6() && (IPv6Iface.UseAutomaticMetric || metric != IPv6Iface.Metric))
	{
		IPv6Iface.Metric = metric;
		IPv6Iface.UseAutomaticMetric = false;
		SetInterface(IPv6Iface);
	}
}

void InterfacePair::SetInterface(const MIB_IPINTERFACE_ROW &iface)
{
	MIB_IPINTERFACE_ROW row = iface;
	const auto status = SetIpInterfaceEntry(&row);

	if (NO_ERROR != status)
	{
		std::stringstream ss;

		ss << "Set metric for "
			<< (row.Family == AF_INET ? "IPv4" : "IPv6")
			<< " on interface with LUID 0x"
			<< std::hex << row.InterfaceLuid.Value;

		THROW_WINDOWS_ERROR(status, ss.str().c_str());
	}
}

bool InterfacePair::HasIPv4()
{
	return AF_UNSPEC != IPv4Iface.Family;
}

bool InterfacePair::HasIPv6()
{
	return AF_UNSPEC != IPv6Iface.Family;
}

//static
void InterfacePair::InitializeInterface(MIB_IPINTERFACE_ROW &iface)
{
	const auto status = GetIpInterfaceEntry(&iface);

	if (NO_ERROR == status)
	{
		return;
	}

	if (STATUS_NOT_FOUND == status || ERROR_NOT_FOUND == status)
	{
		iface.Family = AF_UNSPEC;
	}
	else
	{
		std::stringstream ss;

		ss << "Retrieve info on network interface with LUID 0x"
			<< std::hex << iface.InterfaceLuid.Value;

		THROW_WINDOWS_ERROR(status, ss.str().c_str());
	}
}
