#pragma once

#include <cstdint>

namespace rules
{

// Use weakly typed enum to get implicit promotion to integral types.
enum Ports : uint16_t
{
	DHCPV4_CLIENT_PORT = 68,
	DHCPV4_SERVER_PORT = 67,
	DHCPV6_CLIENT_PORT = 546,
	DHCPV6_SERVER_PORT = 547,

	DNS_SERVER_PORT = 53,
};

}
