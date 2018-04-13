#pragma once

#include <string>
#include <vector>
#include <map>
#include <cstdint>

class DnsRegistry
{
public:

	using DnsServers = std::vector<std::wstring>;

	void registerInterface(uint32_t interfaceIndex, const DnsServers &dnsServers);

	const DnsServers getInterfaceSettings(uint32_t interfaceIndex);

private:

	// Organize by interface index.
	std::map<uint32_t, DnsServers> m_registry;
};
