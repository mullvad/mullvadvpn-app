#include "stdafx.h"
#include <winsock2.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <netioapi.h>

#pragma once
class InterfacePair
{
public:
	InterfacePair(NET_LUID interface_luid);
	~InterfacePair();
	int HighestMetric();
	void SetMetric(int metric);



private:
	MIB_IPINTERFACE_ROW IPv4Iface;
	MIB_IPINTERFACE_ROW IPv6Iface;

	void InitializeRow(PMIB_IPINTERFACE_ROW iface);
	bool HasIPv4();
	bool HasIPv6();
};
