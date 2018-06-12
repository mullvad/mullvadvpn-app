#include "stdafx.h"

#include <windows.h>

#include "NetworkInterfaces.h"

#include <memory>
#include <sstream>
#include <stdexcept>
#include <cstdint>

#include <libcommon/string.h>





PMIB_IPINTERFACE_ROW NetworkInterfaces::RowByLuid(NET_LUID rowId)
{
	for (int i = 0; i < (int)mInterfaces->NumEntries; ++i)
	{
		PMIB_IPINTERFACE_ROW row = &mInterfaces->Table[i];
		// Currnetly, only IPv4 is supported
		if (row->InterfaceLuid.Value == rowId.Value && row->Family == AF_INET)
		{
			return row;
		}
	}
	return nullptr;
}

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


void NetworkInterfaces::EnsureIfaceMetricIsHighest(PMIB_IPINTERFACE_ROW targetIface)
{
	PMIB_IPINTERFACE_ROW iface;
	DWORD success = 0;
	for (int i = 0; i < (int)mInterfaces->NumEntries; ++i)
	{
		iface = &mInterfaces->Table[i];
		// Ignoring the target interface.
		if (iface->InterfaceLuid.Value == targetIface->InterfaceLuid.Value)
		{
			continue;
		}

		if (iface->UseAutomaticMetric)
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
			ss << L"Failed to increment metric for interface with LUID "
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
		ss << L"Failed to enumerate network interfaces: " << success;
		throw std::runtime_error(ss.str());
	}
}

bool NetworkInterfaces::SetTopMetricForInterfaceByAlias(const wchar_t * deviceAlias)
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
	return SetTopMetricForInterfaceWithLuid(targetIfaceLuid);
}

bool NetworkInterfaces::SetTopMetricForInterfaceWithLuid(NET_LUID targetIfaceId)
{

	DWORD success = 0;

	PMIB_IPINTERFACE_ROW targetIface = RowByLuid(targetIfaceId);
	if (targetIface == nullptr)
	{
		std::stringstream ss;
		ss << L"No interface with LUID " << targetIfaceId.Value;
		throw std::runtime_error(ss.str());
	}

	if (targetIface->Metric == MAX_METRIC)
	{
		return false;
	}

	targetIface->UseAutomaticMetric = false;
	targetIface->Metric = MAX_METRIC;
	if (targetIface->Family == AF_INET) {
		targetIface->SitePrefixLength = 0;
	}

	success = SetIpInterfaceEntry(targetIface);
	if (success != NO_ERROR)
	{
		std::stringstream ss;
		ss << L"Failed to set metric "
			<< MAX_METRIC
			<< " for interface with LUID "
			<< targetIfaceId.Value
			<< ". Error code - "
			<< success;
		throw std::runtime_error(ss.str());
	}
	if (!HasHighestMetric(targetIface))
	{
		EnsureIfaceMetricIsHighest(targetIface);
	}
	return true;
}


NetworkInterfaces::~NetworkInterfaces()
{
	FreeMibTable(mInterfaces);
}
