#pragma once

#include <windows.h>
#include <winsock2.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <netioapi.h>
#include <cstdint>
#include <string>

class NetworkInterfaces
{

private:
	PMIB_IPINTERFACE_TABLE mInterfaces;
	bool HasBestMetric(PMIB_IPINTERFACE_ROW targetIface);

public:
	NetworkInterfaces(const NetworkInterfaces &) = delete;
	NetworkInterfaces &operator=(const NetworkInterfaces &) = delete;

	NetworkInterfaces();
	bool SetBestMetricForInterfacesByAlias(const wchar_t *deviceAlias);
	bool SetBestMetricForInterfacesWithLuid(NET_LUID targetIface);
	~NetworkInterfaces();

	static NET_LUID GetInterfaceLuid(const std::wstring &interfaceAlias);
	const MIB_IPINTERFACE_ROW *GetInterface(NET_LUID interfaceLuid, ADDRESS_FAMILY interfaceFamily);
};

constexpr static uint32_t BEST_METRIC = 1;
