#pragma once

#include <string>
#include <cstdint>

class NetSh
{
public:

	static void SetIpv4PrimaryDns(uint32_t interfaceIndex, std::wstring server);
	
	//
	// Caveat: This sets the primary DNS server if there isn't already one.
	//
	static void SetIpv4SecondaryDns(uint32_t interfaceIndex, std::wstring server);

	static void SetIpv4Dhcp(uint32_t interfaceIndex);

	static void SetIpv6PrimaryDns(uint32_t interfaceIndex, std::wstring server);
	static void SetIpv6SecondaryDns(uint32_t interfaceIndex, std::wstring server);
	static void SetIpv6Dhcp(uint32_t interfaceIndex);

private:

	NetSh();
};
