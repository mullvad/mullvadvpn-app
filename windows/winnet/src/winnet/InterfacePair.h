#pragma once

#include <winsock2.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <netioapi.h>

class InterfacePair
{
public:
	InterfacePair(NET_LUID interface_luid);
	int HighestMetric();
	void SetMetric(int metric);


private:
	MIB_IPINTERFACE_ROW IPv4Iface = { 0 };
	MIB_IPINTERFACE_ROW IPv6Iface = { 0 };

	void InitializeInterface(PMIB_IPINTERFACE_ROW iface);
	bool HasIPv4();
	bool HasIPv6();
    void SetInterface(PMIB_IPINTERFACE_ROW iface);

};
