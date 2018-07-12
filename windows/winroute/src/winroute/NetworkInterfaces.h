#pragma once

#include <windows.h>
#include <winsock2.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <netioapi.h>
#include <cstdint>

class NetworkInterfaces
{

private:
	PMIB_IPINTERFACE_TABLE mInterfaces;
	bool HasHighestMetric(PMIB_IPINTERFACE_ROW targetIface);

public:
	void EnsureIfaceMetricIsHighest(NET_LUID interfaceLuid);
	NetworkInterfaces();
	bool SetTopMetricForInterfacesByAlias(const wchar_t *deviceAlias);
	bool SetTopMetricForInterfacesWithLuid(NET_LUID targetIface);
	~NetworkInterfaces();
};

const static uint32_t MAX_METRIC = 1;
