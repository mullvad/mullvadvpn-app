#pragma once

#include <winsock2.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <netioapi.h>
#include <cstdint>

class InterfacePair
{
public:
	InterfacePair(NET_LUID interface_luid);
	int BestMetric();
	int WorstMetric();
	void SetMetric(uint32_t metric);


private:
	MIB_IPINTERFACE_ROW IPv4Iface = { 0 };
	MIB_IPINTERFACE_ROW IPv6Iface = { 0 };

	static void InitializeInterface(MIB_IPINTERFACE_ROW &iface);
	bool HasIPv4();
	bool HasIPv6();
	void SetInterface(const MIB_IPINTERFACE_ROW &iface);

};
