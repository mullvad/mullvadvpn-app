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
	PMIB_IPINTERFACE_ROW RowByLuid(NET_LUID rowId);
	bool HasHighestMetric(PMIB_IPINTERFACE_ROW targetIface);
	void EnsureIfaceMetricIsHighest(PMIB_IPINTERFACE_ROW iface);

public:
	NetworkInterfaces();
	bool SetTopMetricForInterfaceByAlias (const wchar_t *deviceAlias);
	bool SetTopMetricForInterfaceWithLuid(NET_LUID targetIface);
	~NetworkInterfaces();
};

const static uint32_t MAX_METRIC = 1;
