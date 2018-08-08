#include "stdafx.h"

#include "NetworkInterfaces.h"
#include "InterfacePair.h"

#include <memory>
#include <sstream>
#include <stdexcept>
#include <cstdint>

#include <libcommon/string.h>




bool NetworkInterfaces::HasHighestMetric(PMIB_IPINTERFACE_ROW targetIface)
{
	for (unsigned int i = 0; i < mInterfaces->NumEntries; ++i)
	{
		PMIB_IPINTERFACE_ROW iface = &mInterfaces->Table[i];

		if (iface->InterfaceLuid.Value != targetIface->InterfaceLuid.Value
			&& targetIface->Metric >= iface->Metric)
			return false;
	}
	return true;
}


void NetworkInterfaces::EnsureIfaceMetricIsHighest(NET_LUID interfaceLuid)
{
	PMIB_IPINTERFACE_ROW iface;
	DWORD success = 0;
	for (int i = 0; i < (int)mInterfaces->NumEntries; ++i)
	{
		iface = &mInterfaces->Table[i];
		// Ignoring the target interface.
		if (iface->InterfaceLuid.Value == interfaceLuid.Value || iface->UseAutomaticMetric || iface->Metric > MAX_METRIC)
		{
			continue;
		}

		iface->Metric++;
		if (iface->Family == AF_INET) {
			iface->SitePrefixLength = 0;
		}
		success = SetIpInterfaceEntry(iface);
		if (success != NO_ERROR)
		{
			std::stringstream ss;
			ss << "Failed to increment metric for interface with LUID "
				<< &iface->InterfaceLuid.Value
				<< ": "
				<< success;
			throw std::runtime_error(ss.str());
		}

	}
}

NetworkInterfaces::NetworkInterfaces()
{
	mInterfaces = nullptr;
	DWORD success = 0;

	success = GetIpInterfaceTable(AF_UNSPEC, &mInterfaces);
	if (success != NO_ERROR)
	{
		std::stringstream ss;
		ss << "Failed to enumerate network interfaces: " << success;
		throw std::runtime_error(ss.str());
	}
}

bool NetworkInterfaces::SetTopMetricForInterfacesByAlias(const wchar_t * deviceAlias)
{
	NET_LUID targetIfaceLuid;
	DWORD success = 0;
	success = ConvertInterfaceAliasToLuid(deviceAlias, &targetIfaceLuid);
	if (success != NO_ERROR)
	{
		std::wstringstream ss;
		ss << L"Failed to convert interface alias '"
			<< deviceAlias
			<< "' into LUID: "
			<< success;
		throw std::runtime_error(common::string::ToAnsi(ss.str()));
	}
	return SetTopMetricForInterfacesWithLuid(targetIfaceLuid);
}

bool NetworkInterfaces::SetTopMetricForInterfacesWithLuid(NET_LUID targetIfaceId)
{
	InterfacePair targetInterfaces = InterfacePair(targetIfaceId);

	if (targetInterfaces.HighestMetric() == MAX_METRIC)
	{
		return false;
	}

	targetInterfaces.SetMetric(MAX_METRIC);
	return true;
}


NetworkInterfaces::~NetworkInterfaces()
{
	FreeMibTable(mInterfaces);
}
