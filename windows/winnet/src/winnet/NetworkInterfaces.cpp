#include "stdafx.h"
#include "NetworkInterfaces.h"
#include "InterfacePair.h"
#include <libcommon/string.h>
#include <libcommon/error.h>
#include <memory>
#include <sstream>
#include <cstdint>

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

NetworkInterfaces::NetworkInterfaces()
{
	mInterfaces = nullptr;

	const auto status = GetIpInterfaceTable(AF_UNSPEC, &mInterfaces);

	if (NO_ERROR != status)
	{
		THROW_WINDOWS_ERROR(status, "Failed to enumerate network interfaces");
	}
}

bool NetworkInterfaces::SetTopMetricForInterfacesByAlias(const wchar_t * deviceAlias)
{
	return SetTopMetricForInterfacesWithLuid(GetInterfaceLuid(deviceAlias));
}

bool NetworkInterfaces::SetTopMetricForInterfacesWithLuid(NET_LUID targetIfaceId)
{
	InterfacePair targetInterfaces = InterfacePair(targetIfaceId);
	if (MAX_METRIC == targetInterfaces.WorstMetric())
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

//static
NET_LUID NetworkInterfaces::GetInterfaceLuid(const std::wstring &interfaceAlias)
{
	NET_LUID interfaceLuid;

	const auto status = ConvertInterfaceAliasToLuid(interfaceAlias.c_str(), &interfaceLuid);

	if (NO_ERROR != status)
	{
		const auto msg = std::wstring(L"Failed to resolve LUID from interface alias \"")
			.append(interfaceAlias).append(L"\"");

		THROW_WINDOWS_ERROR(status, common::string::ToAnsi(msg).c_str());
	}

	return interfaceLuid;
}

const MIB_IPINTERFACE_ROW *NetworkInterfaces::GetInterface(NET_LUID interfaceLuid, ADDRESS_FAMILY interfaceFamily)
{
	for (unsigned int i = 0; i < mInterfaces->NumEntries; ++i)
	{
		MIB_IPINTERFACE_ROW &candidateInterface = mInterfaces->Table[i];

		if (candidateInterface.InterfaceLuid.Value == interfaceLuid.Value
			&& candidateInterface.Family == interfaceFamily)
		{
			return &candidateInterface;
		}
	}

	return nullptr;
}
