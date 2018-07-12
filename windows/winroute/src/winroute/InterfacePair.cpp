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
	InitializeRow(&IPv4Iface);

	IPv6Iface.Family = AF_INET6;
	IPv6Iface.InterfaceLuid = interface_luid;
	InitializeRow(&IPv6Iface);

	if (!(HasIPv4() || HasIPv6())) {
		std::stringstream ss;
		ss << L"LUID "
			<< interface_luid.Value
			<< " does not specify any IPv4 or IPv6 interfaces";
		throw std::runtime_error(ss.str());
	}
}

InterfacePair::~InterfacePair()
{
}

int InterfacePair::HighestMetric()
{
	return IPv6Iface.Metric < IPv4Iface.Metric ? IPv4Iface.Metric : IPv6Iface.Metric;
}

void InterfacePair::SetMetric(int metric)
{
	DWORD status;

	if (HasIPv4()) {
		IPv4Iface.SitePrefixLength = 0;
		IPv4Iface.Metric = metric;
		IPv4Iface.UseAutomaticMetric = false;
		status = SetIpInterfaceEntry(&IPv4Iface);
		if (status != NO_ERROR) {
			std::stringstream ss;
			ss << L"Failed to set metric to "
				<< metric
				<< " for IPv4 interface with LUID"
				<< IPv4Iface.InterfaceLuid.Value
				<< " with error code "
				<< status;
			throw std::runtime_error(ss.str());
		}
	}

	if (HasIPv6()) {
		IPv6Iface.Metric = metric;
		IPv6Iface.UseAutomaticMetric = false;
		status = SetIpInterfaceEntry(&IPv6Iface);
		if (status != NO_ERROR) {
			std::stringstream ss;
			ss << L"Failed to set metric to "
				<< metric
				<< " for IPv6 interface with LUID"
				<< IPv6Iface.InterfaceLuid.Value
				<< " with error code "
				<< status;
			throw std::runtime_error(ss.str());
		}
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

void InterfacePair::InitializeRow(PMIB_IPINTERFACE_ROW iface)
{
	DWORD status = GetIpInterfaceEntry(iface);

	if (status != NO_ERROR) {
		if (status == STATUS_NOT_FOUND) {
			iface->Family = AF_UNSPEC;
		}
		else {
			std::stringstream ss;
			ss << L"Failed get network interface with LUID "
				<< &iface->InterfaceLuid.Value
				<< ": "
				<< status;
			throw std::runtime_error(ss.str());
		}
	}
}
